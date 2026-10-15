use core::slice;
use std::{
    ffi::{c_char, c_void},
    mem::{self, MaybeUninit},
    os::fd::OwnedFd,
    ptr,
};

use anyhow::{Result, anyhow, bail};
use rustix::event::{
    self, EventfdFlags,
    epoll::{self, EventData, EventFlags},
};
use wayland_client::{
    Connection, Dispatch, EventQueue, Proxy, QueueHandle,
    protocol::{
        wl_callback::WlCallback, wl_compositor::WlCompositor, wl_display::WlDisplay,
        wl_registry::WlRegistry, wl_seat::WlSeat, wl_surface::WlSurface,
    },
};
use wayland_protocols::xdg::shell::client::{
    xdg_surface::XdgSurface, xdg_toplevel::XdgToplevel, xdg_wm_base::XdgWmBase,
};

use crate::{engine::AsResult, vk};

pub struct WindowManager {
    create_wayland_surface_khr: unsafe extern "system" fn(
        &vk::Instance,
        *const WaylandSurfaceCreateInfoKHR,
        *const vk::AllocationCallbacks,
        *mut *const vk::Surface,
    ) -> vk::Result,
    state: State,
    wl_display: WlDisplay,
    queue_handle: QueueHandle<State>,
    queue: EventQueue<State>,
    epoll: OwnedFd,
}

struct State {
    synced: bool,
    wl_compositor: Option<WlCompositor>,
    xdg_wm_base: Option<XdgWmBase>,
    wl_seat: Option<WlSeat>,
}

impl WindowManager {
    pub fn new(
        instance: &vk::Instance,
        get_proc_addr: unsafe extern "system" fn(&vk::Instance, *const c_char) -> *const u8,
    ) -> Result<Self> {
        let sym = unsafe { get_proc_addr(instance, c"vkCreateWaylandSurfaceKHR".as_ptr()) };
        if sym.is_null() {
            bail!("instance does not provide wayland surface creation");
        }

        let epoll = epoll::create(epoll::CreateFlags::CLOEXEC)?;

        let connection = Connection::connect_to_env()?;

        epoll::add(&epoll, &connection, EventData::new_u64(0), EventFlags::IN);

        let display = connection.display();
        let event_queue = connection.new_event_queue();
        let qh = event_queue.handle();
        display.get_registry(&qh, ());
        display.sync(&qh, ());
        event_queue.flush();

        Ok(Self {
            create_wayland_surface_khr: unsafe { mem::transmute(sym) },
            wl_display: display,
            queue_handle: qh,
            queue: event_queue,
            epoll,
            state: State {
                synced: false,
                wl_compositor: None,
                xdg_wm_base: None,
                wl_seat: None,
            },
        })
    }

    pub fn open_window(
        &mut self,
        instance: &vk::Instance,
        key: usize,
    ) -> Result<*const vk::Surface> {
        while !self.state.synced {
            self.queue.blocking_dispatch(&mut self.state);
        }

        let compositor = self
            .state
            .wl_compositor
            .as_ref()
            .ok_or_else(|| anyhow!("wl_compositor unavailable"))?;
        let xdg_wm_base = self
            .state
            .xdg_wm_base
            .as_ref()
            .ok_or_else(|| anyhow!("xdg_wm_base unavailable"))?;

        let event_fd = event::eventfd(0, event::EventfdFlags::CLOEXEC)?;
        epoll::add(
            &self.epoll,
            &event_fd,
            EventData::new_u64(0),
            EventFlags::IN,
        )?;

        let wl_surface = compositor.create_surface(&self.queue_handle, key);
        let xdg_surface = xdg_wm_base.get_xdg_surface(&wl_surface, &self.queue_handle, key);
        let top_level = xdg_surface.get_toplevel(&self.queue_handle, key);
        wl_surface.commit();

        let mut vk_surface = MaybeUninit::uninit();

        unsafe {
            (self.create_wayland_surface_khr)(
                instance,
                &WaylandSurfaceCreateInfoKHR {
                    s_type: 1000006000,
                    p_next: ptr::null(),
                    flags: 0,
                    display: self.wl_display.id().as_ptr().cast(),
                    surface: wl_surface.id().as_ptr().cast(),
                },
                ptr::null(),
                vk_surface.as_mut_ptr(),
            )
            .as_result()?;

            Ok(vk_surface.assume_init())
        }
    }

    pub fn process_messages(&mut self) -> Result<()> {
        loop {
            let read_guard = self.queue.prepare_read();

            let mut event = epoll::Event {
                flags: Default::default(),
                data: EventData::new_u64(0),
            };

            let read = epoll::wait(&self.epoll, slice::from_mut(&mut event), None)?;

            if read > 0 {
                match event.data.u64().checked_sub(1) {
                    None => {}
                    Some(key) => {}
                }

                self.queue.dispatch_pending(&mut self.state)?;
            }
        }
    }
}

#[repr(C)]
struct WaylandSurfaceCreateInfoKHR {
    s_type: i32,
    p_next: *const c_void,
    flags: u32,
    display: *mut c_void,
    surface: *mut c_void,
}

impl Dispatch<WlCallback, ()> for State {
    fn event(
        state: &mut Self,
        proxy: &WlCallback,
        event: <WlCallback as Proxy>::Event,
        data: &(),
        conn: &Connection,
        qhandle: &QueueHandle<Self>,
    ) {
        use wayland_client::protocol::wl_callback::Event;

        if let Event::Done { .. } = event {
            state.synced = true;
        }
    }
}

impl Dispatch<WlRegistry, ()> for State {
    fn event(
        state: &mut Self,
        proxy: &WlRegistry,
        event: <WlRegistry as Proxy>::Event,
        data: &(),
        conn: &Connection,
        qhandle: &QueueHandle<Self>,
    ) {
        use wayland_client::protocol::wl_registry::Event;

        if let Event::Global {
            name,
            interface,
            version,
        } = event
        {
            match &*interface {
                "wl_compositor" => {
                    let compositor: WlCompositor = proxy.bind(name, version, qhandle, ());
                    state.wl_compositor = Some(compositor);
                }
                "xdg_wm_base" => {
                    let wm_base: XdgWmBase = proxy.bind(name, version, qhandle, ());
                    state.xdg_wm_base = Some(wm_base);
                }
                "wl_seat" => {
                    let seat: WlSeat = proxy.bind(name, version, qhandle, ());
                    state.wl_seat = Some(seat);
                }
                _ => {}
            };
        }
    }
}

impl Dispatch<WlCompositor, ()> for State {
    fn event(
        state: &mut Self,
        proxy: &WlCompositor,
        event: <WlCompositor as wayland_client::Proxy>::Event,
        data: &(),
        conn: &Connection,
        qhandle: &wayland_client::QueueHandle<Self>,
    ) {
        log::warn!("unkown wayland WlCompositor event")
    }
}

impl Dispatch<XdgWmBase, ()> for State {
    fn event(
        state: &mut Self,
        proxy: &XdgWmBase,
        event: <XdgWmBase as wayland_client::Proxy>::Event,
        data: &(),
        conn: &Connection,
        qhandle: &wayland_client::QueueHandle<Self>,
    ) {
        use wayland_protocols::xdg::shell::client::xdg_wm_base::Event;

        match event {
            Event::Ping { serial } => {
                proxy.pong(serial);
            }
            _ => log::warn!("unkown wayland XdgWmBase event"),
        }
    }
}

impl Dispatch<WlSeat, ()> for State {
    fn event(
        state: &mut Self,
        proxy: &WlSeat,
        event: <WlSeat as Proxy>::Event,
        data: &(),
        conn: &Connection,
        qhandle: &wayland_client::QueueHandle<Self>,
    ) {
        use wayland_client::protocol::wl_seat::{self, Event};

        match event {
            Event::Capabilities { capabilities } => {
                // if capabilities
                //     .into_result()
                //     .unwrap()
                //     .contains(wl_seat::Capability::Pointer)
                // {
                //     proxy.get_pointer(qhandle, ());
                // }
            }
            Event::Name { name } => {}
            _ => log::warn!("unkown wayland XdgWmBase event"),
        }
    }
}

impl Dispatch<WlSurface, usize> for State {
    fn event(
        state: &mut Self,
        proxy: &WlSurface,
        event: <WlSurface as Proxy>::Event,
        data: &usize,
        conn: &Connection,
        qhandle: &QueueHandle<Self>,
    ) {
        use wayland_client::protocol::wl_surface::Event;

        dbg!(event);
        // match event {
        //     Event::Enter { output } => todo!(),
        //     Event::Leave { output } => todo!(),
        //     Event::PreferredBufferScale { factor } => todo!(),
        //     Event::PreferredBufferTransform { transform } => todo!(),
        //     _ => todo!(),
        // }
    }
}

impl Dispatch<XdgSurface, usize> for State {
    fn event(
        state: &mut Self,
        proxy: &XdgSurface,
        event: <XdgSurface as Proxy>::Event,
        data: &usize,
        conn: &Connection,
        qhandle: &QueueHandle<Self>,
    ) {
        // respond to configure
        use wayland_protocols::xdg::shell::client::xdg_surface::Event;
        match event {
            Event::Configure { serial } => {
                proxy.ack_configure(serial);
                // apply changes here (ie redraw requested, if size changed)
            }
            _ => todo!(),
        }
    }
}

impl Dispatch<XdgToplevel, usize> for State {
    fn event(
        state: &mut Self,
        proxy: &XdgToplevel,
        event: <XdgToplevel as Proxy>::Event,
        data: &usize,
        conn: &Connection,
        qhandle: &QueueHandle<Self>,
    ) {
        // respond to configure
        use wayland_protocols::xdg::shell::client::xdg_toplevel::Event;
        match event {
            Event::Configure {
                width,
                height,
                states,
            } => {}
            Event::Close => todo!(),
            Event::ConfigureBounds { width, height } => todo!(),
            Event::WmCapabilities { .. } => {}
            _ => log::warn!("unkown wayland XdgToplevel event"),
        }
    }
}
