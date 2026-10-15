use core::slice;
use std::{
    ffi, io,
    mem::{self, MaybeUninit},
    ops,
    os::fd::{AsFd, AsRawFd, FromRawFd, OwnedFd},
    path::PathBuf,
    ptr,
    sync::{
        Mutex,
        atomic::{AtomicUsize, Ordering},
    },
};

use anyhow::{Result, anyhow, bail};
use rustix::event::{
    self,
    epoll::{self, EventData, EventFlags},
};
use wayland_client::{
    Connection, Dispatch, Proxy, QueueHandle,
    protocol::{
        wl_buffer::{self, WlBuffer},
        wl_callback::WlCallback,
        wl_compositor::WlCompositor,
        wl_display::WlDisplay,
        wl_pointer::WlPointer,
        wl_registry::WlRegistry,
        wl_seat::WlSeat,
        wl_surface::WlSurface,
    },
};
use wayland_protocols::{
    wp::{
        fractional_scale::v1::client::{
            wp_fractional_scale_manager_v1::WpFractionalScaleManagerV1,
            wp_fractional_scale_v1::WpFractionalScaleV1,
        },
        linux_dmabuf::zv1::client::{
            zwp_linux_buffer_params_v1::{self, ZwpLinuxBufferParamsV1},
            zwp_linux_dmabuf_feedback_v1::ZwpLinuxDmabufFeedbackV1,
            zwp_linux_dmabuf_v1::ZwpLinuxDmabufV1,
        },
        viewporter::client::{wp_viewport::WpViewport, wp_viewporter::WpViewporter},
    },
    xdg::shell::client::{
        xdg_surface::XdgSurface, xdg_toplevel::XdgToplevel, xdg_wm_base::XdgWmBase,
    },
};

use crate::{
    App, AsResult, Context, Extent2D, Image2D, ImageUsageBit, ImageView, ScaleFactor, Semaphore,
    formats::B8G8R8A8Unorm,
    get_list,
    vk::{
        self, DrmFormatModifierPropertiesList, ExportMemoryAllocateInfo,
        ExternalMemoryHandleTypeBit, ExternalMemoryImageCreateInfo,
        ImageDrmFormatModifierListCreateInfo, ImageDrmFormatModifierProperties, ImageSubresource,
        ImportSemaphoreFdInfo, MemoryGetFdInfo, SemaphoreGetFdInfo,
    },
};

pub struct Platform {
    dev_nodes: Vec<PathBuf>,
    queue_handle: QueueHandle<App>,
    synced: bool,
    wl_display: WlDisplay,
    wl_compositor: Option<WlCompositor>,
    xdg_wm_base: Option<XdgWmBase>,
    zwp_linux_dmabuf: Option<ZwpLinuxDmabufV1>,
    wl_seat: Option<WlSeat>,
    wp_viewporter: Option<WpViewporter>,
    wp_fractional_scale_manager: Option<WpFractionalScaleManagerV1>,
    focused_window: Option<usize>,
    epoll: OwnedFd,
}

impl Platform {
    pub fn run(on_init: impl FnOnce(&mut App)) -> Result<()> {
        let connection = Connection::connect_to_env()?;
        let mut event_queue = connection.new_event_queue();
        let qh = event_queue.handle();
        let display = connection.display();
        display.get_registry(&qh, ());
        display.sync(&qh, ());
        event_queue.flush()?;

        let epoll = epoll::create(epoll::CreateFlags::CLOEXEC)?;
        epoll::add(&epoll, &connection, EventData::new_u64(0), EventFlags::IN)?;

        let (vk_create_instance, vk_get_instance_proc_addr) = unsafe {
            let vulkan = libc::dlopen(c"libvulkan.so".as_ptr(), libc::RTLD_NOW | libc::RTLD_LOCAL);
            if vulkan.is_null() {
                bail!("");
            }
            let vk_create_instance = libc::dlsym(vulkan, c"vkCreateInstance".as_ptr());
            if vk_create_instance.is_null() {
                bail!("");
            }

            let vk_get_instance_proc_addr = libc::dlsym(vulkan, c"vkGetInstanceProcAddr".as_ptr());
            if vk_get_instance_proc_addr.is_null() {
                panic!("symbol not found: get addr");
            }
            let get_addr: unsafe extern "system" fn(
                &vk::Instance,
                *const ffi::c_char,
            ) -> *const u8 = mem::transmute(vk_get_instance_proc_addr);

            (mem::transmute(vk_create_instance), get_addr)
        };

        let mut app = App::new(
            vk_create_instance,
            vk_get_instance_proc_addr,
            |instance, &drm_props| {
                let dev_nodes: Vec<_> = (drm_props.has_primary != 0)
                    .then_some((drm_props.primary_major, drm_props.primary_minor))
                    .into_iter()
                    .chain(
                        (drm_props.has_render != 0)
                            .then_some((drm_props.render_major, drm_props.render_minor)),
                    )
                    .flat_map(|(major, minor)| resolve_device_id(major as u32, minor as u32))
                    .collect();

                Ok(Platform {
                    dev_nodes,
                    queue_handle: qh,
                    synced: false,
                    wl_display: display,
                    wl_compositor: None,
                    xdg_wm_base: None,
                    zwp_linux_dmabuf: None,
                    wl_seat: None,
                    wp_viewporter: None,
                    wp_fractional_scale_manager: None,
                    focused_window: None,
                    epoll,
                })
            },
        )?;

        while !app.platform.synced {
            event_queue.blocking_dispatch(&mut app).unwrap();
        }

        on_init(&mut app);

        loop {
            event_queue.dispatch_pending(&mut app).unwrap();
            event_queue.flush()?;
            let read_guard = loop {
                match event_queue.prepare_read() {
                    Some(guard) => break guard,
                    None => event_queue.dispatch_pending(&mut app).unwrap(),
                };
            };

            let mut event = epoll::Event {
                flags: Default::default(),
                data: EventData::new_u64(0),
            };

            let read = epoll::wait(&app.platform.epoll, slice::from_mut(&mut event), None)?;
            if read > 0 {
                match event.data.u64().checked_sub(1) {
                    None => {
                        log::debug!("epoll - wayland socket");
                        read_guard.read().unwrap();
                    }
                    Some(key) => {
                        drop(read_guard);
                        log::debug!("epoll - window {key}");
                        rustix::io::read(
                            &app.windows[key as usize]
                                .as_mut()
                                .unwrap()
                                .window
                                .platform
                                .event_fd,
                            &mut [0u8; 8],
                        )
                        .unwrap();
                        app.redraw_window(key as usize);
                    }
                }
            }
        }
    }

    pub fn open_window(
        &mut self,
        instance: &vk::Instance,
        key: usize,
        width: u32,
        height: u32,
    ) -> Result<PlatformWindow> {
        let compositor = self
            .wl_compositor
            .as_ref()
            .ok_or_else(|| anyhow!("wl_compositor unavailable"))?;
        let xdg_wm_base = self
            .xdg_wm_base
            .as_ref()
            .ok_or_else(|| anyhow!("xdg_wm_base unavailable"))?;
        let zwp_linux_dmabuf = self
            .zwp_linux_dmabuf
            .as_ref()
            .ok_or_else(|| anyhow!("zwp_linux_dmabuf_v1 unavailable"))?;
        let wp_viewporter = self
            .wp_viewporter
            .as_ref()
            .ok_or_else(|| anyhow!("wp_viewporter unavailable"))?;

        let wp_fractional_scale_manager = self
            .wp_fractional_scale_manager
            .as_ref()
            .ok_or_else(|| anyhow!("wp_fractional_scale_manager unavailable"))?;

        let event_fd = event::eventfd(0, event::EventfdFlags::CLOEXEC)?;
        epoll::add(
            &self.epoll,
            &event_fd,
            EventData::new_u64(key as u64 + 1),
            EventFlags::IN,
        )?;

        let wl_surface = compositor.create_surface(&self.queue_handle, key);
        zwp_linux_dmabuf.get_surface_feedback(&wl_surface, &self.queue_handle, key);
        let wp_viewport = wp_viewporter.get_viewport(&wl_surface, &self.queue_handle, key);
        let wp_fractional_scale =
            wp_fractional_scale_manager.get_fractional_scale(&wl_surface, &self.queue_handle, key);
        let xdg_surface = xdg_wm_base.get_xdg_surface(&wl_surface, &self.queue_handle, key);
        let top_level = xdg_surface.get_toplevel(&self.queue_handle, key);
        wl_surface.commit();
        eprintln!("surface committed");

        // let mut vk_surface = ptr::null();
        // unsafe {
        //     (self.create_wayland_surface_khr)(
        //         instance,
        //         &WaylandSurfaceCreateInfoKHR {
        //             s_type: 1000006000,
        //             p_next: ptr::null(),
        //             flags: 0,
        //             display: self.wl_display.id().as_ptr().cast(),
        //             surface: wl_surface.id().as_ptr().cast(),
        //         },
        //         ptr::null(),
        //         &mut vk_surface,
        //     )
        //     .as_result()?;

        Ok(
            // vk_surface,
            PlatformWindow {
                width,
                height,
                scale: ScaleFactor(120),
                pending_scale: ScaleFactor(120),
                wl_surface,
                wp_viewport,
                event_fd: rustix::io::dup(&event_fd)?,
                format_table: None,
                tranche: Tranche {
                    format: vk::Format::Undefined,
                    drm_format: 0,
                    drm_modifiers: vec![],
                },
                pending_tranche: Tranche {
                    format: vk::Format::Undefined,
                    drm_format: 0,
                    drm_modifiers: vec![],
                },
                tranche_device_matched: false,
                tranche_found: false,
            },
        )
        // }
    }
}

fn resolve_device_id(major: u32, minor: u32) -> io::Result<PathBuf> {
    PathBuf::from(format!("/sys/dev/char/{major}:{minor}",)).canonicalize()
}

impl ScaleFactor {
    pub const ONE: Self = Self(120);
}

pub struct PlatformWindow {
    height: u32,
    width: u32,
    scale: ScaleFactor,
    pending_scale: ScaleFactor,
    wl_surface: WlSurface,
    wp_viewport: WpViewport,
    event_fd: OwnedFd,
    format_table: Option<FormatTable>,
    tranche: Tranche,
    pending_tranche: Tranche,
    tranche_device_matched: bool,
    tranche_found: bool,
}

pub struct SwapchainImage {
    pub(crate) image: Image2D<B8G8R8A8Unorm>,
    pub(crate) view: ImageView<B8G8R8A8Unorm>,
    pub(crate) wait_semaphore: Semaphore,
    pub(crate) signal_semaphore: Semaphore,
    fd: OwnedFd,
    busy: bool,
    buffer: WlBuffer,
    size: Extent2D,
}

impl Drop for SwapchainImage {
    fn drop(&mut self) {
        assert!(!self.busy, "Attempted to destroy a swaphchain busy image");
    }
}

struct FormatTable(*mut [FormatModifier]);

impl FormatTable {
    pub fn new(fd: OwnedFd, size: u32) -> Self {
        let size = size as usize;
        let table = unsafe {
            let ptr = libc::mmap(
                ptr::null_mut(),
                size,
                libc::PROT_READ,
                libc::MAP_PRIVATE,
                fd.as_raw_fd(),
                0,
            );
            if ptr == libc::MAP_FAILED {
                panic!("failed to map format table") // FIXME: handle
            }
            ptr::slice_from_raw_parts_mut(ptr.cast(), size / mem::size_of::<FormatModifier>())
        };

        Self(table)
    }
}

struct Tranche {
    format: vk::Format,
    drm_format: u32,
    drm_modifiers: Vec<u64>,
}

impl Default for Tranche {
    fn default() -> Self {
        Self {
            format: vk::Format::Undefined,
            drm_format: 0,
            drm_modifiers: vec![],
        }
    }
}

impl ops::Deref for FormatTable {
    type Target = [FormatModifier];

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.0 }
    }
}

impl Drop for FormatTable {
    fn drop(&mut self) {
        unsafe {
            libc::munmap(
                self.0.cast(),
                self.0.len() * mem::size_of::<FormatModifier>(),
            );
        }
    }
}

#[repr(C)]
struct FormatModifier {
    format: u32,
    modifier: u64,
}

impl App {
    fn platform_redraw(&mut self, key: usize) {
        let window = &mut self.windows[key].as_mut().unwrap().window;

        let idx = window
            .images
            .iter()
            .find_map(|(key, image)| (!image.busy).then_some(key));
        match idx {
            Some(idx) => {
                log::debug!("found free image {idx}");
                window.next_image_idx = idx;
                let image = &mut window.images[idx];

                #[repr(C)]
                struct DmaBufImportSyncFile {
                    flags: u32,
                    fd: i32,
                }

                unsafe impl rustix::ioctl::Ioctl for DmaBufImportSyncFile {
                    type Output = ();
                    const IS_MUTATING: bool = true;
                    fn opcode(&self) -> rustix::ioctl::Opcode {
                        rustix::ioctl::opcode::write::<Self>('b' as u8, 3)
                    }

                    fn as_ptr(&mut self) -> *mut rustix::ffi::c_void {
                        ptr::from_mut(self).cast()
                    }

                    unsafe fn output_from_ptr(
                        _: rustix::ioctl::IoctlOutput,
                        _: *mut rustix::ffi::c_void,
                    ) -> rustix::io::Result<Self::Output> {
                        Ok(())
                    }
                }

                #[repr(C)]
                struct DmaBufExportSyncFile {
                    flags: u32,
                    fd: i32,
                }

                unsafe impl rustix::ioctl::Ioctl for DmaBufExportSyncFile {
                    type Output = i32;
                    const IS_MUTATING: bool = true;
                    fn opcode(&self) -> rustix::ioctl::Opcode {
                        rustix::ioctl::opcode::read_write::<Self>('b' as u8, 2)
                    }

                    fn as_ptr(&mut self) -> *mut rustix::ffi::c_void {
                        ptr::from_mut(self).cast()
                    }

                    unsafe fn output_from_ptr(
                        _: rustix::ioctl::IoctlOutput,
                        ptr: *mut rustix::ffi::c_void,
                    ) -> rustix::io::Result<Self::Output> {
                        Ok(unsafe { ptr.cast::<Self>().read().fd })
                    }
                }

                log::trace!("ioctl DmaBufExportSyncFile({})", image.fd.as_raw_fd());
                let mut sync_file = unsafe {
                    rustix::ioctl::ioctl(&image.fd, DmaBufExportSyncFile { flags: 2, fd: 0 })
                }
                .unwrap();

                unsafe {
                    (self.cx.device_fns.import_semaphore_fd)(
                        self.cx.device,
                        &vk::ImportSemaphoreFdInfo {
                            s_type: vk::StructureType::ImportSemaphoreFdInfo,
                            p_next: ptr::null(),
                            semaphore: image.wait_semaphore.handle.vk,
                            flags: vk::SemaphoreImportBit::Temporary.into(),
                            handle_type: vk::ExternalSemaphoreHandleTypeBit::SyncFd,
                            fd: sync_file,
                        },
                    )
                    .as_result()
                    .unwrap()
                };

                self.redraw_window(key);

                let window = &mut self.windows[key].as_mut().unwrap().window;
                let image = &mut window.images[idx];

                unsafe {
                    (self.cx.device_fns.get_semaphore_fd)(
                        self.cx.device,
                        &SemaphoreGetFdInfo {
                            s_type: vk::StructureType::SemaphoreGetFdInfo,
                            p_next: ptr::null(),
                            semaphore: image.signal_semaphore.handle.vk,
                            handle_type: vk::ExternalSemaphoreHandleTypeBit::SyncFd,
                        },
                        &mut sync_file,
                    )
                    .as_result()
                    .unwrap();
                }

                if sync_file != -1 {
                    unsafe {
                        rustix::ioctl::ioctl(
                            &image.fd,
                            DmaBufImportSyncFile {
                                flags: 2,
                                fd: sync_file,
                            },
                        )
                        .unwrap();
                        rustix::io::close(sync_file);
                    }
                }

                image.busy = true;
                window.platform.wl_surface.attach(Some(&image.buffer), 0, 0);
                window
                    .platform
                    .wl_surface
                    .damage_buffer(0, 0, i32::MAX, i32::MAX);
                window
                    .platform
                    .wl_surface
                    .frame(&self.platform.queue_handle, key);
                window.platform.wl_surface.commit();
            }
            None => {
                window
                    .platform
                    .create_swapchain_image(
                        self.platform.zwp_linux_dmabuf.as_ref().unwrap(),
                        &self.platform.queue_handle,
                        key,
                    )
                    .unwrap();
            }
        }
    }
}

impl PlatformWindow {
    pub fn create_swapchain_image(
        &self,
        zwp_linux_dmabuf: &ZwpLinuxDmabufV1,
        qh: &QueueHandle<App>,
        key: usize,
        // format: vk::Format,
        // drm_modifiers: &[u64],
        // width: u32,
        // height: u32,
    ) -> Result<(), vk::Result> {
        let cx = Context(());
        // let mut image = ptr::null();

        dbg!(self.scale);
        let width = dbg!(self.width * self.scale);
        let height = dbg!(self.height * self.scale);

        unsafe {
            let image = cx.create_image::<B8G8R8A8Unorm>(
                // self.tranche.format,
                ImageUsageBit::ColorAttachment | ImageUsageBit::InputAttachment,
                vk::ImageTiling::DrmFormatModifier,
                vk::Extent2D { width, height },
                None,
                1,
                1,
                ptr::from_ref(&ExternalMemoryImageCreateInfo {
                    s_type: vk::StructureType::ExternalMemoryImageCreateInfo,
                    p_next: ptr::from_ref(&ImageDrmFormatModifierListCreateInfo {
                        s_type: vk::StructureType::ImageDrmFormatModifierListCreateInfo,
                        p_next: ptr::null(),
                        drm_format_modifier_count: self.tranche.drm_modifiers.len() as u32,
                        p_drm_format_modifiers: self.tranche.drm_modifiers.as_ptr(),
                    })
                    .cast(),
                    handle_types: ExternalMemoryHandleTypeBit::DmaBuf.into(),
                })
                .cast(),
                ptr::from_ref(&ExportMemoryAllocateInfo {
                    s_type: vk::StructureType::ExportMemoryAllocateInfo,
                    p_next: ptr::null(),
                    handle_types: ExternalMemoryHandleTypeBit::DmaBuf.into(),
                })
                .cast(),
            );

            let image = Image2D {
                _format: std::marker::PhantomData,
                inner: image,
                extent: vk::Extent2D { width, height },
            };

            let modifier_props = get_list(|count, ptr| {
                let mut drm_format_modifier_properties = DrmFormatModifierPropertiesList {
                    s_type: vk::StructureType::DrmFormatModifierPropertiesList,
                    p_next: ptr::null(),
                    drm_format_modifier_count: *count,
                    p_drm_format_modifier_properties: ptr,
                };

                let mut format_properties = vk::FormatProperties2 {
                    s_type: vk::StructureType::FormatProperties2,
                    p_next: ptr::from_mut(&mut drm_format_modifier_properties).cast(),
                    format_properties: vk::FormatProperties {
                        linear_tiling_features: vk::FormatFeatureFlags(0),
                        optimal_tiling_features: vk::FormatFeatureFlags(0),
                        buffer_features: vk::FormatFeatureFlags(0),
                    },
                };

                (cx.instance_fns.get_physical_device_format_properties2)(
                    cx.physical_device,
                    self.tranche.format,
                    &mut format_properties,
                );

                *count = drm_format_modifier_properties.drm_format_modifier_count;

                vk::Result::SUCCESS
            })
            .unwrap();

            let mut properties = ImageDrmFormatModifierProperties {
                s_type: vk::StructureType::ImageDrmFormatModifierProperties,
                p_next: ptr::null(),
                drm_format_modifier: 0,
            };

            (cx.device_fns.get_image_drm_format_modifier_properties)(
                cx.device,
                image.inner.vk,
                &mut properties,
            )
            .as_result()?;

            let modifier_props = modifier_props
                .iter()
                .find(|props| props.drm_format_modifier == properties.drm_format_modifier)
                .unwrap();

            let mut fd = 0;
            (cx.device_fns.get_memory_fd)(
                cx.device,
                &MemoryGetFdInfo {
                    s_type: vk::StructureType::MemoryGetFdInfo,
                    p_next: ptr::null(),
                    memory: image.inner.block.memory,
                    handle_type: ExternalMemoryHandleTypeBit::DmaBuf,
                },
                &mut fd,
            )
            .as_result()?;

            log::debug!("dma buf fd = {fd}");
            let fd = OwnedFd::from_raw_fd(fd);

            let vk_image = image.inner.vk;
            let buffer_params = zwp_linux_dmabuf.create_params(
                qh,
                Mutex::new(Some(BufferCreateParams {
                    key,
                    size: vk::Extent2D { width, height },
                    image,
                    fd: rustix::io::dup(&fd).unwrap(),
                    format: self.tranche.format,
                })),
            );

            for i in 0..modifier_props.drm_format_modifier_plane_count {
                let mut subresource_layout = MaybeUninit::uninit();
                (cx.device_fns.get_image_subresource_layout)(
                    cx.device,
                    vk_image,
                    &ImageSubresource {
                        aspect_mask: vk::ImageAspectFlags(0x80 << i),
                        mip_level: 0,
                        array_layer: 0,
                    },
                    subresource_layout.as_mut_ptr(),
                );
                let layout = subresource_layout.assume_init();
                buffer_params.add(
                    fd.as_fd(),
                    i,
                    layout.offset as u32,
                    layout.row_pitch as u32,
                    (properties.drm_format_modifier >> 32) as u32,
                    (properties.drm_format_modifier & u32::MAX as u64) as u32,
                );
            }

            buffer_params.create(
                width as i32,
                height as i32,
                self.tranche.drm_format,
                zwp_linux_buffer_params_v1::Flags::empty(),
            );
            // buffer_params.destroy();
        }

        Ok(())

        // VK_KHR_external_memory_capabilities
    }
}

pub struct RedrawTrigger(OwnedFd);

impl RedrawTrigger {
    pub fn trigger(&self) {
        if let Err(err) = rustix::io::write(&self.0, &1u64.to_ne_bytes()) {
            log::warn!("failed to send redraw event to main thread: {err}");
        }
    }
}

#[allow(unused_variables)]

impl Dispatch<WlCallback, ()> for App {
    fn event(
        state: &mut Self,
        proxy: &WlCallback,
        event: <WlCallback as Proxy>::Event,
        data: &(),
        conn: &Connection,
        qhandle: &QueueHandle<Self>,
    ) {
        use wayland_client::protocol::wl_callback::Event;

        // eprintln!("wl event: {event:?}");
        if let Event::Done { .. } = event {
            state.platform.synced = true;
        }
    }
}

#[allow(unused_variables)]

impl Dispatch<WlRegistry, ()> for App {
    fn event(
        state: &mut Self,
        proxy: &WlRegistry,
        event: <WlRegistry as Proxy>::Event,
        data: &(),
        conn: &Connection,
        qhandle: &QueueHandle<Self>,
    ) {
        use wayland_client::protocol::wl_registry::Event;

        // eprintln!("wl event: {event:?}");
        if let Event::Global {
            name,
            interface,
            version,
        } = event
        {
            match &*interface {
                "wl_compositor" => {
                    let compositor: WlCompositor = proxy.bind(name, version, qhandle, ());
                    state.platform.wl_compositor = Some(compositor);
                }
                "xdg_wm_base" => {
                    let wm_base: XdgWmBase = proxy.bind(name, version, qhandle, ());
                    state.platform.xdg_wm_base = Some(wm_base);
                }
                "zwp_linux_dmabuf_v1" => {
                    let linux_dmabuf: ZwpLinuxDmabufV1 = proxy.bind(name, version, qhandle, ());
                    state.platform.zwp_linux_dmabuf = Some(linux_dmabuf);
                }
                "wl_seat" => {
                    let seat: WlSeat = proxy.bind(name, version, qhandle, ());
                    state.platform.wl_seat = Some(seat);
                }
                "wp_viewporter" => {
                    let viewporter: WpViewporter = proxy.bind(name, version, qhandle, ());
                    state.platform.wp_viewporter = Some(viewporter);
                }
                "wp_fractional_scale_manager_v1" => {
                    let fractional_scale_manager: WpFractionalScaleManagerV1 =
                        proxy.bind(name, version, qhandle, ());
                    state.platform.wp_fractional_scale_manager = Some(fractional_scale_manager);
                }
                _ => {}
            };
        }
    }
}

#[allow(unused_variables)]

impl Dispatch<WlCompositor, ()> for App {
    fn event(
        state: &mut Self,
        proxy: &WlCompositor,
        event: <WlCompositor as wayland_client::Proxy>::Event,
        data: &(),
        conn: &Connection,
        qhandle: &wayland_client::QueueHandle<Self>,
    ) {
        log::warn!("unkown wayland WlCompositor event");
        // eprintln!("wl event: {event:?}");
    }
}

#[allow(unused_variables)]

impl Dispatch<XdgWmBase, ()> for App {
    fn event(
        state: &mut Self,
        proxy: &XdgWmBase,
        event: <XdgWmBase as wayland_client::Proxy>::Event,
        data: &(),
        conn: &Connection,
        qhandle: &wayland_client::QueueHandle<Self>,
    ) {
        use wayland_protocols::xdg::shell::client::xdg_wm_base::Event;

        // eprintln!("wl event: {event:?}");
        match event {
            Event::Ping { serial } => {
                proxy.pong(serial);
            }
            _ => log::warn!("unkown wayland XdgWmBase event"),
        }
    }
}

#[allow(unused_variables)]
impl Dispatch<WlSeat, ()> for App {
    fn event(
        state: &mut Self,
        proxy: &WlSeat,
        event: <WlSeat as Proxy>::Event,
        data: &(),
        conn: &Connection,
        qhandle: &wayland_client::QueueHandle<Self>,
    ) {
        use wayland_client::protocol::wl_seat::{self, Event};

        // eprintln!("wl event: {event:?}");
        match event {
            Event::Capabilities { capabilities } => {
                if capabilities
                    .into_result()
                    .unwrap()
                    .contains(wl_seat::Capability::Pointer)
                {
                    proxy.get_pointer(qhandle, ());
                }
            }
            Event::Name { name } => {}
            _ => log::warn!("unkown wayland WlSeat event"),
        }
    }
}

#[allow(unused_variables)]

impl Dispatch<WlPointer, ()> for App {
    fn event(
        state: &mut Self,
        proxy: &WlPointer,
        event: <WlPointer as Proxy>::Event,
        data: &(),
        conn: &Connection,
        qhandle: &wayland_client::QueueHandle<Self>,
    ) {
        use wayland_client::protocol::wl_pointer::{self, Event};

        // eprintln!("wl event: {event:?}");
        match event {
            Event::Enter {
                serial,
                surface,
                surface_x,
                surface_y,
            } => {
                let key: usize = *surface.data().unwrap();
                state.platform.focused_window = Some(key);
                state.windows[key].as_mut().unwrap().window.mouse_position =
                    Some((surface_x as u32, surface_y as u32));
            }
            Event::Leave { serial, surface } => {
                let key: usize = *surface.data().unwrap();
                state.platform.focused_window = None;
                state.windows[key].as_mut().unwrap().window.mouse_position = None;
            }
            Event::Motion {
                time,
                surface_x,
                surface_y,
            } => {
                if let Some(key) = state.platform.focused_window {
                    state.windows[key].as_mut().unwrap().window.mouse_position =
                        Some((surface_x as u32, surface_y as u32));
                }
            }
            Event::Button {
                serial,
                time,
                button,
                state,
            } => {}
            Event::Axis { time, axis, value } => {}
            Event::Frame => {}
            Event::AxisSource { axis_source } => {}
            Event::AxisStop { time, axis } => {}
            Event::AxisDiscrete { axis, discrete } => {}
            Event::AxisValue120 { axis, value120 } => {}
            Event::AxisRelativeDirection { axis, direction } => {}
            _ => log::warn!("unkown wayland WlPointer event"),
        }
    }
}

#[allow(unused_variables)]
impl Dispatch<WpViewporter, ()> for App {
    fn event(
        state: &mut Self,
        proxy: &WpViewporter,
        event: <WpViewporter as Proxy>::Event,
        data: &(),
        conn: &Connection,
        qhandle: &wayland_client::QueueHandle<Self>,
    ) {
    }
}

#[allow(unused_variables)]
impl Dispatch<WpFractionalScaleManagerV1, ()> for App {
    fn event(
        state: &mut Self,
        proxy: &WpFractionalScaleManagerV1,
        event: <WpFractionalScaleManagerV1 as Proxy>::Event,
        data: &(),
        conn: &Connection,
        qhandle: &wayland_client::QueueHandle<Self>,
    ) {
    }
}

#[allow(unused_variables)]

impl Dispatch<ZwpLinuxDmabufV1, ()> for App {
    fn event(
        state: &mut Self,
        proxy: &ZwpLinuxDmabufV1,
        event: <ZwpLinuxDmabufV1 as Proxy>::Event,
        data: &(),
        conn: &Connection,
        qhandle: &QueueHandle<Self>,
    ) {
        use wayland_protocols::wp::linux_dmabuf::zv1::client::zwp_linux_dmabuf_v1::Event;

        // eprintln!("wl event: {event:?}");

        match event {
            Event::Format { .. } => {}
            Event::Modifier { .. } => {}
            _ => log::warn!("unkown wayland ZwpLinuxDmabufV1 event"),
        }
    }
}

#[allow(unused_variables)]
impl Dispatch<WlSurface, usize> for App {
    fn event(
        state: &mut Self,
        proxy: &WlSurface,
        event: <WlSurface as Proxy>::Event,
        &key: &usize,
        conn: &Connection,
        qhandle: &QueueHandle<Self>,
    ) {
        use wayland_client::protocol::wl_surface::Event;

        match event {
            Event::Enter { output } => {}
            Event::Leave { output } => {}
            Event::PreferredBufferScale { factor } => {
                // state.windows[key]
                //     .as_mut()
                //     .unwrap()
                //     .window
                //     .platform
                //     .pending_scale = factor as u32;
                // proxy.set_buffer_scale(factor);
            }
            Event::PreferredBufferTransform { transform } => {}
            _ => log::warn!("unknown wayland WlSurface event"),
        }
    }
}

#[allow(unused_variables)]
impl Dispatch<WpViewport, usize> for App {
    fn event(
        state: &mut Self,
        proxy: &WpViewport,
        event: <WpViewport as Proxy>::Event,
        &key: &usize,
        conn: &Connection,
        qhandle: &QueueHandle<Self>,
    ) {
    }
}

#[allow(unused_variables)]
impl Dispatch<WpFractionalScaleV1, usize> for App {
    fn event(
        state: &mut Self,
        proxy: &WpFractionalScaleV1,
        event: <WpFractionalScaleV1 as Proxy>::Event,
        &key: &usize,
        conn: &Connection,
        qhandle: &QueueHandle<Self>,
    ) {
        use wayland_protocols::wp::fractional_scale::v1::client::wp_fractional_scale_v1::Event;

        match event {
            Event::PreferredScale { scale } => {
                state.windows[key]
                    .as_mut()
                    .unwrap()
                    .window
                    .platform
                    .pending_scale = ScaleFactor(scale);
            }
            _ => todo!(),
        }
    }
}

#[allow(unused_variables)]

impl Dispatch<WlCallback, usize> for App {
    fn event(
        state: &mut Self,
        proxy: &WlCallback,
        event: <WlCallback as Proxy>::Event,
        &key: &usize,
        conn: &Connection,
        qhandle: &QueueHandle<Self>,
    ) {
        use wayland_client::protocol::wl_callback::Event;

        // eprintln!("wl event: {event:?}");
        if let Event::Done { .. } = event {
            state.platform_redraw(key);
        }
    }
}

#[allow(unused_variables)]

impl Dispatch<ZwpLinuxDmabufFeedbackV1, usize> for App {
    fn event(
        state: &mut Self,
        proxy: &ZwpLinuxDmabufFeedbackV1,
        event: <ZwpLinuxDmabufFeedbackV1 as Proxy>::Event,
        &data: &usize,
        conn: &Connection,
        qhandle: &QueueHandle<Self>,
    ) {
        use wayland_protocols::wp::linux_dmabuf::zv1::client::zwp_linux_dmabuf_feedback_v1::Event;
        // eprintln!("wl event: {event:?}");

        let window = &mut state.windows[data].as_mut().unwrap().window.platform;

        match event {
            Event::Done => {
                if mem::take(&mut window.tranche_found) {
                    window.tranche = mem::take(&mut window.pending_tranche);
                }
            }
            Event::FormatTable { fd, size } => {
                window.format_table = Some(FormatTable::new(fd, size));
            }
            Event::MainDevice { device } => {}
            Event::TrancheDone => {
                if !window.tranche_found
                    && mem::take(&mut window.tranche_device_matched)
                    && window.pending_tranche.format != vk::Format::Undefined
                {
                    window.tranche_found = true;
                }
            }
            Event::TrancheTargetDevice { device } => {
                if !window.tranche_found {
                    match device.try_into() {
                        Ok(bytes) => {
                            let id = u64::from_ne_bytes(bytes);
                            let minor = libc::minor(id);
                            let major = libc::major(id);
                            let node = resolve_device_id(major, minor);
                            if let Ok(node) = node
                                && state.platform.dev_nodes.contains(&node)
                            {
                                window.tranche_device_matched = true;
                            }
                        }
                        Err(_) => {
                            log::warn!("invalid tranche target device");
                        }
                    }
                }
            }
            Event::TrancheFormats { indices } => {
                if !window.tranche_found
                    && let Some(table) = &window.format_table
                {
                    for idx in indices
                        .as_chunks()
                        .0
                        .iter()
                        .copied()
                        .map(u16::from_ne_bytes)
                    {
                        let fm = &table[idx as usize];

                        fn fourcc_code(a: char, b: char, c: char, d: char) -> u32 {
                            ((a as u8 as u32) | ((b as u8 as u32) << 8)) as u32
                                | (((c as u8 as u32) << 16) as u32)
                                | (((d as u8 as u32) << 24) as u32)
                        }

                        if fm.format == fourcc_code('A', 'R', '2', '4') {
                            window.pending_tranche.format = vk::Format::B8G8R8A8Unorm;
                            window.pending_tranche.drm_format = fm.format;
                            window.pending_tranche.drm_modifiers.push(fm.modifier);
                        }
                    }
                }
            }
            Event::TrancheFlags { .. } => {}
            _ => log::warn!("unkown wayland ZwpLinuxDmabufFeedbackV1 event"),
        }
    }
}

struct BufferCreateParams {
    key: usize,
    size: Extent2D,
    image: Image2D<B8G8R8A8Unorm>,
    fd: OwnedFd,
    format: vk::Format,
}

unsafe impl Send for BufferCreateParams {}
unsafe impl Sync for BufferCreateParams {}

#[allow(unused_variables)]

impl Dispatch<ZwpLinuxBufferParamsV1, Mutex<Option<BufferCreateParams>>> for App {
    fn event(
        state: &mut Self,
        proxy: &ZwpLinuxBufferParamsV1,
        event: <ZwpLinuxBufferParamsV1 as Proxy>::Event,
        params: &Mutex<Option<BufferCreateParams>>,
        conn: &Connection,
        qhandle: &QueueHandle<Self>,
    ) {
        use zwp_linux_buffer_params_v1::Event;
        // eprintln!("wl event: {event:?}");

        let BufferCreateParams {
            key,
            size,
            image,
            fd,
            format,
        } = params.lock().unwrap().take().unwrap();

        let window = &mut state.windows[key].as_mut().unwrap().window;

        match event {
            Event::Created { buffer } => {
                if size.width == window.width * window.platform.scale
                    && size.height == window.height * window.platform.scale
                {
                    let buffer_key: &(AtomicUsize, AtomicUsize) = buffer.data().unwrap();
                    buffer_key.0.store(key, Ordering::Relaxed);
                    buffer_key
                        .1
                        .store(window.images.vacant_key(), Ordering::Relaxed);

                    window.images.insert(SwapchainImage {
                        view: image.view(),
                        image,
                        wait_semaphore: state.cx.create_semaphore(),
                        signal_semaphore: state.cx.create_semaphore_with_next(
                            ptr::from_ref(&vk::ExportSemaphoreCreateInfo {
                                s_type: vk::StructureType::ExportSemaphoreCreateInfo,
                                p_next: ptr::null(),
                                handle_types: vk::ExternalSemaphoreHandleTypeBit::SyncFd.into(),
                            })
                            .cast(),
                        ),
                        fd: rustix::io::dup(fd).unwrap(),
                        busy: false,
                        buffer,
                        size,
                    });
                } else {
                    log::error!(
                        "DISCARD: {size:?}, {} {} {:?}",
                        window.platform.width,
                        window.platform.height,
                        window.platform.scale
                    );
                    buffer.destroy();
                }

                state.platform_redraw(key);
            }
            Event::Failed => {
                panic!("failed to create buffer");
            }
            _ => todo!(),
        }
    }

    fn event_created_child(
        opcode: u16,
        qhandle: &QueueHandle<Self>,
    ) -> std::sync::Arc<dyn wayland_client::backend::ObjectData> {
        qhandle.make_data::<WlBuffer, _>((AtomicUsize::new(0), AtomicUsize::new(0)))
    }
}

#[allow(unused_variables)]

impl Dispatch<WlBuffer, (AtomicUsize, AtomicUsize)> for App {
    fn event(
        state: &mut Self,
        proxy: &WlBuffer,
        event: <WlBuffer as Proxy>::Event,
        data: &(AtomicUsize, AtomicUsize),
        conn: &Connection,
        qhandle: &QueueHandle<Self>,
    ) {
        use wl_buffer::Event;
        // eprintln!("wl event: {event:?}");

        let key = data.0.load(Ordering::Relaxed);
        let image_key = data.1.load(Ordering::Relaxed);

        match event {
            Event::Release => {
                let window = &mut state.windows[key].as_mut().unwrap().window;
                let image = &mut window.images[image_key];
                image.busy = false;
                log::debug!("image {image_key} is now free");

                if image.size.width != window.width * window.platform.scale
                    || image.size.height != window.height * window.platform.scale
                {
                    log::debug!("dropping image {image_key}");
                    window.images.remove(image_key);
                }
            }
            _ => todo!(),
        }
    }
}

#[allow(unused_variables)]

impl Dispatch<XdgSurface, usize> for App {
    fn event(
        state: &mut Self,
        proxy: &XdgSurface,
        event: <XdgSurface as Proxy>::Event,
        &key: &usize,
        conn: &Connection,
        qhandle: &QueueHandle<Self>,
    ) {
        // respond to configure
        use wayland_protocols::xdg::shell::client::xdg_surface::Event;
        // eprintln!("wl event: {event:?}");
        match event {
            Event::Configure { serial } => {
                let window = &mut state.windows[key].as_mut().unwrap().window;
                window.width = window.platform.width;
                window.height = window.platform.height;
                window.platform.scale = window.platform.pending_scale;
                window.scale_factor = window.platform.scale;
                window
                    .platform
                    .wp_viewport
                    .set_destination(window.width as i32, window.height as i32);
                proxy.ack_configure(serial);

                window.images.retain(|_, img| img.busy);

                if window.width != 0 && window.height != 0 {
                    log::info!("creating image");
                    window
                        .platform
                        .create_swapchain_image(
                            &state.platform.zwp_linux_dmabuf.as_ref().unwrap(),
                            qhandle,
                            key,
                        )
                        .unwrap();
                } else {
                    log::info!(
                        "skipping creation for size {}x{}",
                        window.width,
                        window.height
                    );
                }

                // window
                //     .create_swapchain_image(
                //         &state
                //             .platform
                //             .zwp_linux_dmabuf
                //             .as_ref()
                //             .unwrap()
                //             .create_params(qhandle, key),
                //     )
                //     .unwrap();

                // eprintln!("ack configure");
                // apply changes here (ie redraw requested, if size changed)
            }
            _ => todo!(),
        }
    }
}

#[allow(unused_variables)]

impl Dispatch<XdgToplevel, usize> for App {
    fn event(
        state: &mut Self,
        proxy: &XdgToplevel,
        event: <XdgToplevel as Proxy>::Event,
        &key: &usize,
        conn: &Connection,
        qhandle: &QueueHandle<Self>,
    ) {
        use wayland_protocols::xdg::shell::client::xdg_toplevel::Event;

        // eprintln!("wl event: {event:?}");
        let window = &mut state.windows[key].as_mut().unwrap().window.platform;
        match event {
            Event::Configure {
                width,
                height,
                states,
            } => {
                log::info!("toplevel configure -> {width}x{height}");
                if width != 0 {
                    window.width = width as u32;
                }
                if height != 0 {
                    window.height = height as u32;
                }
            }
            Event::Close => {
                eprintln!("close");
            }
            Event::ConfigureBounds { width, height } => {
                window.height = height as u32;
                window.width = width as u32;
                eprintln!("config bounds...");
            }
            Event::WmCapabilities { .. } => {}
            _ => log::warn!("unkown wayland XdgToplevel event"),
        }
    }
}
