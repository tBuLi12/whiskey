pub mod formats;
mod linux;

macro_rules! flags {
    ($flags:ident, $inner:ty) => {
        #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
        #[repr(transparent)]
        pub struct $flags(pub(crate) $inner);
    };
    ($base:ident, $flags:ident, $inner:ty) => {
        #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
        #[repr(transparent)]
        pub struct $flags(pub(crate) $inner);

        impl ops::BitOr<Self> for $flags {
            type Output = Self;

            fn bitor(self, rhs: Self) -> Self {
                Self(self.0 | rhs.0)
            }
        }

        impl ops::BitAnd<Self> for $flags {
            type Output = Self;

            fn bitand(self, rhs: Self) -> Self {
                Self(self.0 & rhs.0)
            }
        }

        impl ops::BitOr<$base> for $flags {
            type Output = Self;

            fn bitor(self, rhs: $base) -> Self {
                Self(self.0 | rhs as $inner)
            }
        }

        impl ops::BitOr<Self> for $base {
            type Output = $flags;

            fn bitor(self, rhs: Self) -> Self::Output {
                $flags(self as $inner | rhs as $inner)
            }
        }

        impl From<Option<Self>> for $flags {
            fn from(value: Option<Self>) -> Self {
                value.unwrap_or(Self(0))
            }
        }

        impl From<$base> for $flags {
            fn from(value: $base) -> Self {
                Self(value as $inner)
            }
        }
    };
}

pub mod vec_buffer;
mod vk;
// mod windows;

use core::slice;
use std::{
    cell::{Cell, RefCell},
    cmp,
    collections::{BTreeMap, HashMap},
    ffi::{self, CStr},
    fmt,
    marker::PhantomData,
    mem::{self, MaybeUninit},
    ops::{self, Deref, Range, RangeBounds},
    process,
    ptr::{self},
    sync::{Arc, Mutex, mpsc::Sender},
};

use anyhow::Result;
use guillotiere::{AtlasAllocator, euclid};
use range_alloc::RangeAllocator;
use slab::Slab;

use crate::{
    formats::{B8G8R8A8Unorm, Format},
    linux::{Platform, PlatformWindow, SwapchainImage},
};

pub use crate::vk::{
    AttachmentLoadOp, AttachmentStoreOp, BlendFactor, BlendOp, BufferUsageBit, BufferUsageFlags,
    ColorBlendEquation, ColorComponentBit, ColorComponentFlags, CommandBuffer,
    CommandBufferSubmitInfo, CullModeBit, CullModeFlags, DependencyBit, DependencyFlags,
    DescriptorType, DrawIndirectCommand, Extent2D, Extent3D, Fence, Filter, ImageAspectBit,
    ImageAspectFlags, ImageLayout, ImageUsageBit, ImageUsageFlags, MemoryPropertyBit,
    MemoryPropertyFlags, Offset2D, Offset3D, PolygonMode, PrimitiveTopology, Rect2D,
    ResolveModeFlags, SampleCountBit, SamplerAddressMode, SamplerMipmapMode, SemaphoreSubmitInfo,
    ShaderCreateBit, ShaderCreateFlags, ShaderStageBit, ShaderStageFlags, Viewport,
};

pub struct App {
    cx: Context,
    platform: Platform,
    windows: Slab<Option<WindowData>>,
}

impl Deref for App {
    type Target = Context;

    fn deref(&self) -> &Self::Target {
        &self.cx
    }
}

impl App {
    pub fn run(on_init: impl FnOnce(&mut Self)) -> ! {
        if let Err(error) = Platform::run(on_init) {
            eprintln!("Error: {error}");
        }
        process::exit(0);
    }

    pub fn new_window(&mut self) -> WindowBuilder<'_> {
        WindowBuilder {
            app: self,
            max_queued_frames: 2,
            image_usage: ImageUsageFlags(0),
            concurrent_sharing: None,
        }
    }

    fn with_window_data<T>(
        &mut self,
        key: usize,
        fun: impl FnOnce(&mut Self, &mut WindowData) -> T,
    ) -> T {
        let mut window = self.windows[key].take().unwrap();
        let result = fun(self, &mut window);
        self.windows[key] = Some(window);
        result
    }

    fn set_window_dimentions(&mut self, key: usize, width: u32, height: u32) {
        let window = &mut self.windows[key].as_mut().unwrap().window;

        window.width = width;
        window.height = height;
    }

    fn redraw_window(&mut self, key: usize) {
        self.with_window_data(key, |this, window_data| {
            let window = &mut window_data.window;
            // if window.swapchain.is_null() {
            //     log::warn!("attempted to redraw uninitialized window");
            //     return;
            // };

            match (window_data.render_frame)(this, window) {
                Ok(()) => {}
                Err(err) => {
                    panic!("failed to render frame: {err}");
                }
            }
        })
    }

    fn new(
        vk_create_instance: unsafe extern "system" fn(
            p_create_info: *const vk::InstanceCreateInfo,
            p_allocator: *const vk::AllocationCallbacks,
            p_instance: *mut *const vk::Instance,
        ) -> vk::Result,
        vk_get_instance_proc_addr: unsafe extern "system" fn(
            &vk::Instance,
            *const ffi::c_char,
        ) -> *const u8,
        create_platform: impl FnOnce(
            &vk::Instance,
            &vk::PhysicalDeviceDrmProperties,
        ) -> Result<Platform>,
        // create_platform: impl FnOnce(&Instance, &PhysicalDeviceIDProperties) -> Result<Platform>,
    ) -> Result<Self> {
        unsafe {
            let mut instance = ptr::null();

            (vk_create_instance)(
                &vk::InstanceCreateInfo {
                    s_type: vk::StructureType::InstanceCreateInfo,
                    p_next: ptr::null(),
                    flags: vk::InstanceCreateFlags(0),
                    p_application_info: &vk::ApplicationInfo {
                        s_type: vk::StructureType::ApplicationInfo,
                        p_next: ptr::null(),
                        p_application_name: ptr::null(),
                        application_version: 0,
                        p_engine_name: ptr::null(),
                        engine_version: 0,
                        api_version: (1 << 22) | (4 << 12),
                    },
                    enabled_layer_count: 1,
                    pp_enabled_layer_names: [c"VK_LAYER_KHRONOS_validation".as_ptr()].as_ptr(),
                    enabled_extension_count: 1,
                    pp_enabled_extension_names: [c"VK_EXT_debug_utils".as_ptr()].as_ptr(),
                },
                ptr::null(),
                &mut instance,
            )
            .as_result()
            .unwrap();

            let instance = &*instance;

            log::info!("vulkan instance created");

            // debug hooks
            {
                let vk_create_debug_utils_messenger =
                    vk_get_instance_proc_addr(instance, c"vkCreateDebugUtilsMessengerEXT".as_ptr());
                if vk_create_debug_utils_messenger.is_null() {
                    panic!("symbol not found");
                }

                #[repr(C)]
                struct DebugUtilsMessengerCreateInfo {
                    s_type: i32,
                    p_next: *const ffi::c_void,
                    flags: u32,
                    message_severity: u32,
                    message_type: u32,
                    pfn_user_callback: extern "system" fn(
                        message_severity: u32,
                        message_types: u32,
                        callback_data: Option<&DebugUtilsMessengerCallbackDataEXT>,
                        user_data: *mut ffi::c_void,
                    ) -> vk::Bool32,
                    p_user_data: *mut ffi::c_void,
                }

                #[repr(C)]
                pub struct DebugUtilsMessengerCallbackDataEXT {
                    s_type: vk::StructureType,
                    p_next: *const ffi::c_void,
                    flags: u32,
                    p_message_id_name: *const ffi::c_char,
                    message_id_number: i32,
                    p_message: *const ffi::c_char,
                    queue_label_count: u32,
                    p_queue_labels: *const u8,
                    cmd_buf_label_count: u32,
                    p_cmd_buf_labels: *const u8,
                    object_count: u32,
                    p_objects: *const u8,
                }

                let vk_create_debug_utils_messenger: extern "system" fn(
                    &vk::Instance,
                    &DebugUtilsMessengerCreateInfo,
                    *const vk::AllocationCallbacks,
                    *mut *const u8,
                )
                    -> vk::Result = mem::transmute(vk_create_debug_utils_messenger);

                extern "system" fn callback(
                    message_severity: u32,
                    message_types: u32,
                    callback_data: Option<&DebugUtilsMessengerCallbackDataEXT>,
                    user_data: *mut ffi::c_void,
                ) -> vk::Bool32 {
                    if let Some(data) = callback_data
                        && !data.p_message.is_null()
                    {
                        unsafe {
                            let message = CStr::from_ptr(data.p_message).to_string_lossy();

                            log::warn!("Vulkan - {}", message.lines().next().unwrap_or(&*message));
                            log::trace!("Vulkan - {}", message);
                        }
                    }

                    0
                }

                vk_create_debug_utils_messenger(
                    instance,
                    &DebugUtilsMessengerCreateInfo {
                        s_type: 1000128004,
                        p_next: ptr::null(),
                        flags: 0,
                        message_severity: 0x1111,
                        message_type: 0xf,
                        pfn_user_callback: callback,
                        p_user_data: ptr::null_mut(),
                    },
                    ptr::null(),
                    &mut ptr::null(),
                )
                .as_result()
                .unwrap();
            }

            let instance_fns = vk::InstanceFns::load_with(|name| {
                let sym = vk_get_instance_proc_addr(instance, name.as_ptr());
                if sym.is_null() {
                    panic!("symbol not found: {name:?}");
                }
                sym
            });

            log::info!("instance functions loaded");

            let physical_devices = get_list(|count, ptr| {
                (instance_fns.enumerate_physical_devices)(instance, count, ptr)
            })?;

            log::info!("obtained physical devices");
            let (physical_device, props, drm_props, heap_props) = physical_devices
                .into_iter()
                .map(|device| {
                    let mut props = MaybeUninit::<vk::PhysicalDeviceProperties2>::uninit();
                    let mut drm_props = vk::PhysicalDeviceDrmProperties {
                        s_type: vk::StructureType::PhysicalDeviceDrmProperties,
                        p_next: ptr::null_mut(),
                        has_primary: 0,
                        has_render: 0,
                        primary_major: 0,
                        primary_minor: 0,
                        render_major: 0,
                        render_minor: 0,
                    };
                    let mut _device_id_props = vk::PhysicalDeviceIDProperties {
                        s_type: vk::StructureType::PhysicalDeviceIdProperties,
                        p_next: ptr::null_mut(),
                        device_uuid: Default::default(),
                        driver_uuid: Default::default(),
                        device_luid: Default::default(),
                        device_node_mask: Default::default(),
                        device_luid_valid: Default::default(),
                    };
                    let mut heap_props = vk::PhysicalDeviceDescriptorHeapProperties {
                        s_type: vk::StructureType::PhysicalDeviceDescriptorHeapProperties,
                        p_next: ptr::from_mut(&mut drm_props).cast(),
                        ..Default::default()
                    };
                    (&raw mut (*props.as_mut_ptr()).s_type)
                        .write(vk::StructureType::PhysicalDeviceProperties2);
                    (&raw mut (*props.as_mut_ptr()).p_next)
                        .write(ptr::from_mut(&mut heap_props).cast());
                    // (&raw mut (*props.as_mut_ptr()).p_next)
                    //     .write(ptr::from_mut(&mut device_id_props).cast());
                    (instance_fns.get_physical_device_properties2)(device, props.as_mut_ptr());
                    (device, props.assume_init(), drm_props, heap_props)
                })
                .min_by_key(|(_, props, _drm_props, _heap_props)| {
                    match props.properties.device_type {
                        vk::PhysicalDeviceType::DISCRETE_GPU => 0,
                        vk::PhysicalDeviceType::INTEGRATED_GPU => 1,
                        vk::PhysicalDeviceType::VIRTUAL_GPU => 2,
                        vk::PhysicalDeviceType::CPU => 3,
                        vk::PhysicalDeviceType::OTHER => 4,
                        _ => 5,
                    }
                })
                .unwrap();

            let props = props.properties;

            let name = CStr::from_ptr(props.device_name.as_ptr());
            log::info!("using physical device {name:?}");

            let queue_props = get_list(|count, ptr| {
                (instance_fns.get_physical_device_queue_family_properties)(
                    physical_device,
                    count,
                    ptr,
                );
                vk::Result::SUCCESS
            })?;

            for prop in &queue_props {
                eprintln!(
                    "{} {} {}",
                    vk::QueueFlags::from(vk::QueueBit::Graphics) & prop.queue_flags
                        == vk::QueueFlags::from(vk::QueueBit::Graphics),
                    vk::QueueFlags::from(vk::QueueBit::Compute) & prop.queue_flags
                        == vk::QueueFlags::from(vk::QueueBit::Compute),
                    vk::QueueFlags::from(vk::QueueBit::Transfer) & prop.queue_flags
                        == vk::QueueFlags::from(vk::QueueBit::Transfer),
                );
            }

            let queue_idx = queue_props
                .iter()
                .position(|props| {
                    let value = props.queue_flags;
                    let required = vk::QueueBit::Graphics | vk::QueueBit::Compute;
                    required & value == required
                })
                .unwrap();

            // let transfer_queue_idx = queue_props
            //     .iter()
            //     .position(|props| {
            //         let value = props.queue_flags;
            //         let required = vk::QueueFlags::from(vk::QueueBit::Transfer);
            //         required & value == required
            //             && (vk::QueueBit::Graphics | vk::QueueBit::Compute) & value
            //                 == vk::QueueFlags(0)
            //     })
            //     .unwrap();

            let mut memory_props = MaybeUninit::uninit();
            (instance_fns.get_physical_device_memory_properties)(
                physical_device,
                memory_props.as_mut_ptr(),
            );
            let memory_props = memory_props.assume_init();

            let device_memory_allocator = DeviceMemoryAllocator {
                memories: Mutex::new(vec![
                    (1024 * 1024, vec![]);
                    memory_props.memory_type_count as usize
                ]),
                memory_props,
            };
            // MEMORY_PROPS.write(memory_props);

            // let features = vk::PhysicalDeviceShaderObjectFeatures {
            //     s_type: vk::StructureType::PhysicalDeviceShaderObjectFeatures,
            //     p_next: ptr::from_ref(&features).cast_mut().cast(),
            //     shader_object: 1,
            // };

            // let features = vk::PhysicalDeviceMeshShaderFeatures {
            //     s_type: vk::StructureType::PhysicalDeviceMeshShaderFeatures,
            //     p_next: ptr::null_mut(),
            //     task_shader: 1,
            //     mesh_shader: 1,
            //     multiview_mesh_shader: 0,
            //     primitive_fragment_shading_rate_mesh_shader: 0,
            //     mesh_shader_queries: 0,
            // };

            let features = vk::PhysicalDeviceShaderObjectFeatures {
                s_type: vk::StructureType::PhysicalDeviceShaderObjectFeatures,
                // p_next: ptr::from_ref(&features).cast_mut().cast(),
                p_next: ptr::null_mut(),
                shader_object: 1,
            };

            let features = vk::PhysicalDeviceShaderUntypedPointersFeatures {
                s_type: vk::StructureType::PhysicalDeviceShaderUntypedPointersFeatures,
                p_next: ptr::from_ref(&features).cast_mut().cast(),
                shader_untyped_pointers: 1,
            };

            let features = vk::PhysicalDeviceDescriptorHeapFeatures {
                s_type: vk::StructureType::PhysicalDeviceDescriptorHeapFeatures,
                p_next: ptr::from_ref(&features).cast_mut().cast(),
                descriptor_heap: 1,
                descriptor_heap_capture_replay: 0,
            };

            let features = vk::PhysicalDeviceVulkan14Features {
                s_type: vk::StructureType::PhysicalDeviceVulkan14Features,
                p_next: ptr::from_ref(&features).cast_mut().cast(),
                dynamic_rendering_local_read: 1,
                ..Default::default()
            };

            let features = vk::PhysicalDeviceVulkan13Features {
                s_type: vk::StructureType::PhysicalDeviceVulkan13Features,
                p_next: ptr::from_ref(&features).cast_mut().cast(),
                dynamic_rendering: 1,
                synchronization_2: 1,
                ..Default::default()
            };

            let features = vk::PhysicalDeviceVulkan12Features {
                s_type: vk::StructureType::PhysicalDeviceVulkan12Features,
                p_next: ptr::from_ref(&features).cast_mut().cast(),
                timeline_semaphore: 1,
                buffer_device_address: 1,
                ..Default::default()
            };

            let features = vk::PhysicalDeviceVulkan11Features {
                s_type: vk::StructureType::PhysicalDeviceVulkan11Features,
                p_next: ptr::from_ref(&features).cast_mut().cast(),
                shader_draw_parameters: 1,
                ..Default::default()
            };

            let queue_create_infos = [
                vk::DeviceQueueCreateInfo {
                    s_type: vk::StructureType::DeviceQueueCreateInfo,
                    p_next: ptr::null(),
                    flags: vk::DeviceQueueCreateFlags(0),
                    queue_family_index: queue_idx as u32,
                    queue_count: 1,
                    p_queue_priorities: &1.0,
                },
                // vk::DeviceQueueCreateInfo {
                //     s_type: vk::StructureType::DeviceQueueCreateInfo,
                //     p_next: ptr::null(),
                //     flags: vk::DeviceQueueCreateFlags(0),
                //     queue_family_index: transfer_queue_idx as u32,
                //     queue_count: 1,
                //     p_queue_priorities: &1.0,
                // },
            ];

            let mut device = ptr::null();
            (instance_fns.create_device)(
                physical_device,
                &vk::DeviceCreateInfo {
                    s_type: vk::StructureType::DeviceCreateInfo,
                    p_next: ptr::from_ref(&features).cast_mut().cast(),
                    flags: vk::DeviceCreateFlags(0),
                    // queue_create_info_count: 2,
                    queue_create_info_count: 1,
                    p_queue_create_infos: queue_create_infos.as_ptr(),
                    enabled_layer_count: 0,
                    pp_enabled_layer_names: ptr::null(),
                    enabled_extension_count: 8,
                    pp_enabled_extension_names: [
                        c"VK_EXT_descriptor_heap".as_ptr(),
                        c"VK_EXT_shader_object".as_ptr(),
                        c"VK_KHR_shader_untyped_pointers".as_ptr(),
                        // c"VK_EXT_mesh_shader".as_ptr(),
                        c"VK_KHR_external_memory_fd".as_ptr(),
                        c"VK_EXT_external_memory_dma_buf".as_ptr(),
                        c"VK_EXT_image_drm_format_modifier".as_ptr(),
                        c"VK_KHR_external_semaphore_fd".as_ptr(),
                        c"VK_EXT_queue_family_foreign".as_ptr(),
                        // c"VK_KHR_external_memory_win32".as_ptr(),
                        // c"VK_KHR_external_semaphore_win32".as_ptr(),
                    ]
                    .as_ptr(),
                    p_enabled_features: &vk::PhysicalDeviceFeatures {
                        multi_draw_indirect: 1,
                        sampler_anisotropy: 1,
                        ..Default::default()
                    },
                },
                ptr::null(),
                &mut device,
            )
            .as_result()?;

            let device = &*device;

            let sym = vk_get_instance_proc_addr(instance, c"vkGetDeviceProcAddr".as_ptr());
            if sym.is_null() {
                panic!("symbol not found: get addr");
            }
            let vk_get_device_proc_addr: unsafe extern "system" fn(
                &vk::Device,
                *const ffi::c_char,
            ) -> *const u8 = mem::transmute(sym);

            let device_fns = vk::DeviceFns::load_with(|name| {
                let ptr = vk_get_device_proc_addr(device, name.as_ptr());
                if ptr.is_null() {
                    panic!("fn not found: {name:?}");
                }
                ptr
            });

            let mut queue = ptr::null();
            (device_fns.get_device_queue)(device, queue_idx as u32, 0, &mut queue);

            // let mut transfer_queue = ptr::null();
            // (device_fns.get_device_queue)(
            //     device,
            //     transfer_queue_idx as u32,
            //     0,
            //     &mut transfer_queue,
            // );

            #[allow(static_mut_refs)]
            CONTEXT_DATA.write(ContextData {
                instance_fns,
                device_fns,
                heap_properties: heap_props,
                instance,
                device,
                physical_device,
                allocator: device_memory_allocator,
                // transfer: Mutex::new(None),
                graphics_queue: Queue {
                    vk: Mutex::new(&*queue),
                    family_idx: queue_idx as u32,
                },
                user_queue_indices: [queue_idx as u32],
                // all_queue_indices: [queue_idx as u32, transfer_queue_idx as u32],
                all_queue_indices: [queue_idx as u32],
            });

            let cx = Context(());

            // start_transfer_thread(Queue {
            //     vk: Mutex::new(&*transfer_queue),
            //     family_idx: transfer_queue_idx as u32,
            // });

            Ok(Self {
                cx,
                platform: create_platform(instance, &drm_props)?,
                windows: Slab::new(),
            })
        }
    }
}

impl std::error::Error for vk::Result {}
impl fmt::Display for vk::Result {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "vulkan error: {:?}", self)
    }
}

// macro_rules! flags {
//     ($flags:ident, $inner:ty) => {
//         #[derive(Clone, Copy, Debug)]
//         #[repr(transparent)]
//         pub struct $flags(pub(crate) $inner);
//     };
//     ($base:ident, $flags:ident, $inner:ty) => {
//         #[derive(Clone, Copy, Debug)]
//         #[repr(transparent)]
//         pub struct $flags(pub(crate) $inner);

//         impl BitOr<Self> for $flags {
//             type Output = Self;

//             fn bitor(self, rhs: Self) -> Self {
//                 Self(self.0 | rhs.0)
//             }
//         }

//         impl BitOr<$base> for $flags {
//             type Output = Self;

//             fn bitor(self, rhs: $base) -> Self {
//                 Self(self.0 | rhs as $inner)
//             }
//         }

//         impl BitOr<Self> for $base {
//             type Output = $flags;

//             fn bitor(self, rhs: Self) -> Self::Output {
//                 $flags(self as $inner | rhs as $inner)
//             }
//         }

//         impl From<Option<Self>> for $flags {
//             fn from(value: Option<Self>) -> Self {
//                 value.unwrap_or(Self(0))
//             }
//         }
//     };
// }

trait AsPtrExt<T> {
    fn as_ptr(self) -> *const T;
}

impl<T> AsPtrExt<T> for Option<&T> {
    fn as_ptr(self) -> *const T {
        self.map_or(ptr::null(), |this| this)
    }
}

pub trait AsResult: Sized {
    fn as_result(self) -> Result<(), vk::Result>;

    #[track_caller]
    fn must_succeed(self) {
        self.as_result().unwrap()
    }
}

impl AsResult for vk::Result {
    fn as_result(self) -> Result<(), vk::Result> {
        if self == vk::Result::SUCCESS {
            Ok(())
        } else {
            Err(self)
        }
    }
}

flags!(PipelineStageBit, PipelineStageFlags, u64);
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[repr(i64)]
pub enum PipelineStageBit {
    TopOfPipe = 1 << 0,
    DrawIndirect = 1 << 1,
    VertexInput = 1 << 2,
    VertexShader = 1 << 3,
    TessellationControlShader = 1 << 4,
    TessellationEvaluationShader = 1 << 5,
    GeometryShader = 1 << 6,
    FragmentShader = 1 << 7,
    TaskShader = 0x00080000,
    MeshShader = 0x00100000,
    EarlyFragmentTests = 1 << 8,
    LateFragmentTests = 1 << 9,
    ColorAttachmentOutput = 1 << 10,
    ComputeShader = 1 << 11,
    AllTransfer = 1 << 12,
    BottomOfPipe = 1 << 13,
    Host = 1 << 14,
    AllGraphics = 1 << 15,
    AllCommands = 1 << 16,
    Copy = 1 << 32,
    Resolve = 1 << 33,
    Blit = 1 << 34,
    Clear = 1 << 35,
    IndexInput = 1 << 36,
    VertexAttributeInput = 1 << 37,
    PreRasterizationShaders = 1 << 38,
}

flags!(AccessBit, AccessFlags, u64);
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[repr(i64)]
pub enum AccessBit {
    IndirectCommandRead = 1 << 0,
    IndexRead = 1 << 1,
    VertexAttributeRead = 1 << 2,
    UniformRead = 1 << 3,
    InputAttachmentRead = 1 << 4,
    ShaderRead = 1 << 5,
    ShaderWrite = 1 << 6,
    ColorAttachmentRead = 1 << 7,
    ColorAttachmentWrite = 1 << 8,
    DepthStencilAttachmentRead = 1 << 9,
    DepthStencilAttachmentWrite = 1 << 10,
    TransferRead = 1 << 11,
    TransferWrite = 1 << 12,
    HostRead = 1 << 13,
    HostWrite = 1 << 14,
    MemoryRead = 1 << 15,
    MemoryWrite = 1 << 16,
    ShaderSampledRead = 1 << 32,
    ShaderStorageRead = 1 << 33,
    ShaderStorageWrite = 1 << 34,
    SamplerHeapRead = 1 << 57,
    ResourceHeapRead = 1 << 58,
}

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct MemoryBarrier {
    s_type: vk::StructureType,
    p_next: *const ffi::c_void,
    src_stage_mask: PipelineStageFlags,
    src_access_mask: AccessFlags,
    dst_stage_mask: PipelineStageFlags,
    dst_access_mask: AccessFlags,
}

impl MemoryBarrier {
    pub fn new() -> Self {
        Self {
            s_type: vk::StructureType::MemoryBarrier2,
            p_next: ptr::null(),
            src_stage_mask: PipelineStageFlags(0),
            src_access_mask: AccessFlags(0),
            dst_stage_mask: PipelineStageFlags(0),
            dst_access_mask: AccessFlags(0),
        }
    }

    pub fn src_access(
        mut self,
        stage: impl Into<PipelineStageFlags>,
        access: impl Into<AccessFlags>,
    ) -> Self {
        self.src_access_mask = access.into();
        self.src_stage_mask = stage.into();
        self
    }

    pub fn dst_access(
        mut self,
        stage: impl Into<PipelineStageFlags>,
        access: impl Into<AccessFlags>,
    ) -> Self {
        self.dst_access_mask = access.into();
        self.dst_stage_mask = stage.into();
        self
    }
}

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct ImageMemoryBarrier {
    s_type: vk::StructureType,
    p_next: *const ffi::c_void,
    src_stage_mask: PipelineStageFlags,
    src_access_mask: AccessFlags,
    dst_stage_mask: PipelineStageFlags,
    dst_access_mask: AccessFlags,
    old_layout: ImageLayout,
    new_layout: ImageLayout,
    src_queue_family_index: u32,
    dst_queue_family_index: u32,
    image: *const vk::Image,
    subresource_range: vk::ImageSubresourceRange,
}

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct BufferCopy {
    s_type: vk::StructureType,
    p_next: *const ffi::c_void,
    src_offset: u64,
    dst_offset: u64,
    size: u64,
}

impl BufferCopy {
    pub fn new<T>(src_offset: usize, dst_offset: usize, size: usize) -> Self {
        Self {
            s_type: vk::StructureType::BufferCopy2,
            p_next: ptr::null(),
            src_offset: (src_offset * mem::size_of::<T>()) as u64,
            dst_offset: (dst_offset * mem::size_of::<T>()) as u64,
            size: (size * mem::size_of::<T>()) as u64,
        }
    }
}

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct ImageCopy {
    s_type: vk::StructureType,
    p_next: *const ffi::c_void,
    src_subresource: vk::ImageSubresourceLayers,
    src_offset: Offset3D,
    dst_subresource: vk::ImageSubresourceLayers,
    dst_offset: Offset3D,
    extent: Extent3D,
}

impl ImageCopy {
    pub fn new<Fmt: Format>(
        src_offset: Offset3D,
        src_layers: Range<u32>,
        src_mip_level: u32,
        dst_offset: Offset3D,
        dst_layers: Range<u32>,
        dst_mip_level: u32,
        extent: Extent3D,
    ) -> Self {
        Self {
            s_type: vk::StructureType::ImageCopy2,
            p_next: ptr::null(),
            src_subresource: vk::ImageSubresourceLayers {
                aspect_mask: Fmt::all_aspects(),
                mip_level: src_mip_level,
                base_array_layer: src_layers.start,
                layer_count: src_layers.end - src_layers.start,
            },
            src_offset,
            dst_subresource: vk::ImageSubresourceLayers {
                aspect_mask: Fmt::all_aspects(),
                mip_level: dst_mip_level,
                base_array_layer: dst_layers.start,
                layer_count: dst_layers.end - dst_layers.start,
            },
            dst_offset,
            extent,
        }
    }
}

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct ImageBlit {
    s_type: vk::StructureType,
    p_next: *const ffi::c_void,
    src_subresource: vk::ImageSubresourceLayers,
    src_offsets: [Offset3D; 2 as usize],
    dst_subresource: vk::ImageSubresourceLayers,
    dst_offsets: [Offset3D; 2 as usize],
}

impl ImageBlit {
    pub fn new<Fmt: Format>(
        src_offsets: [Offset3D; 2],
        src_layers: Range<u32>,
        src_mip_level: u32,
        dst_offsets: [Offset3D; 2],
        dst_layers: Range<u32>,
        dst_mip_level: u32,
    ) -> Self {
        Self {
            s_type: vk::StructureType::ImageBlit2,
            p_next: ptr::null(),
            src_subresource: vk::ImageSubresourceLayers {
                aspect_mask: Fmt::all_aspects(),
                mip_level: src_mip_level,
                base_array_layer: src_layers.start,
                layer_count: src_layers.end - src_layers.start,
            },
            src_offsets,
            dst_subresource: vk::ImageSubresourceLayers {
                aspect_mask: Fmt::all_aspects(),
                mip_level: dst_mip_level,
                base_array_layer: dst_layers.start,
                layer_count: dst_layers.end - dst_layers.start,
            },
            dst_offsets,
        }
    }
}

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct BufferImageCopy {
    s_type: vk::StructureType,
    p_next: *const ffi::c_void,
    buffer_offset: u64,
    buffer_row_length: u32,
    buffer_image_height: u32,
    image_subresource: vk::ImageSubresourceLayers,
    image_offset: Offset3D,
    image_extent: Extent3D,
}

impl BufferImageCopy {
    pub fn new<Fmt: Format>(
        buffer_offset: usize,
        image_offset: Offset3D,
        layers: Range<u32>,
        mip_level: u32,
        image_extent: Extent3D,
    ) -> Self {
        Self {
            s_type: vk::StructureType::BufferImageCopy2,
            p_next: ptr::null(),
            image_subresource: vk::ImageSubresourceLayers {
                aspect_mask: Fmt::all_aspects(),
                mip_level,
                base_array_layer: layers.start,
                layer_count: layers.end - layers.start,
            },
            image_offset,
            image_extent,
            buffer_offset: (buffer_offset * mem::size_of::<Fmt>()) as u64,
            buffer_row_length: 0,
            buffer_image_height: 0,
        }
    }
}

struct SemaphoreHandle {
    vk: &'static vk::Semaphore,
}

#[derive(Clone)]
pub struct Semaphore {
    handle: Arc<SemaphoreHandle>,
}

#[derive(Clone)]
pub struct TimelineSemaphore {
    handle: Arc<SemaphoreHandle>,
}

impl TimelineSemaphore {
    pub fn wait(&self, value: u64) {
        let cx = Context(());

        unsafe {
            (cx.device_fns.wait_semaphores)(
                cx.device,
                &vk::SemaphoreWaitInfo {
                    s_type: vk::StructureType::SemaphoreWaitInfo,
                    p_next: ptr::null(),
                    flags: vk::SemaphoreWaitFlags(0),
                    semaphore_count: 1,
                    p_semaphores: &ptr::from_ref(self.handle.vk),
                    p_values: &value,
                },
                !0,
            )
            .must_succeed();
        }
    }
}

struct BufferHandle {
    vk: &'static vk::Buffer,
    block: Block,
}

impl BufferHandle {
    fn device_address(&self) -> u64 {
        let cx = Context(());
        unsafe {
            (cx.device_fns.get_buffer_device_address)(
                cx.device,
                &vk::BufferDeviceAddressInfo {
                    s_type: vk::StructureType::BufferDeviceAddressInfo,
                    p_next: ptr::null(),
                    buffer: self.vk,
                },
            )
        }
    }
}

// pub trait TransferS

pub struct Buffer<T> {
    handle: Arc<BufferHandle>,
    _owns_t: PhantomData<T>,
}

impl<T> Buffer<T> {
    pub fn device_address(&self) -> u64 {
        self.handle.device_address()
    }

    pub fn len(&self) -> usize {
        self.handle.block.size as usize / mem::size_of::<T>()
    }

    pub fn range(&self, range: impl RangeBounds<usize>) -> BufferRange<'_, T> {
        let start = match range.start_bound() {
            ops::Bound::Included(&start) => start,
            ops::Bound::Excluded(&start) => start + 1,
            ops::Bound::Unbounded => 0,
        };

        let end = match range.end_bound() {
            ops::Bound::Included(&end) => end + 1,
            ops::Bound::Excluded(&end) => end,
            ops::Bound::Unbounded => self.len(),
        };

        assert!(start <= end && end <= self.len());

        BufferRange {
            buffer: &self,
            offset: start,
            len: end - start,
        }
    }

    pub fn host_ptr(&self) -> *mut T {
        self.handle.block.host_ptr.cast()
    }

    pub fn cast<U>(self) -> Buffer<U> {
        Buffer {
            handle: self.handle,
            _owns_t: PhantomData,
        }
    }

    pub fn cast_ref<U>(&self) -> &Buffer<U> {
        unsafe { &*ptr::from_ref(self).cast() }
    }
}

impl<T> Clone for Buffer<T> {
    fn clone(&self) -> Self {
        Self {
            handle: self.handle.clone(),
            _owns_t: PhantomData,
        }
    }
}

impl<T> cmp::PartialEq for Buffer<T> {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.handle, &other.handle)
    }
}

pub struct BufferRange<'buf, T> {
    buffer: &'buf Buffer<T>,
    offset: usize,
    len: usize,
}

impl<'buf, T> BufferRange<'buf, T> {
    pub fn buffer(&self) -> &'buf Buffer<T> {
        self.buffer
    }

    pub fn offset(&self) -> usize {
        self.offset
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn cast<U>(self) -> BufferRange<'buf, U> {
        let offset = self.offset * mem::size_of::<T>() / mem::size_of::<U>();
        eprintln!(
            "{} {} {}",
            self.offset,
            mem::size_of::<T>(),
            mem::size_of::<U>()
        );
        assert_eq!(self.offset * mem::size_of::<T>() % mem::size_of::<U>(), 0);

        let len = self.len * mem::size_of::<T>() / mem::size_of::<U>();
        assert_eq!(self.len * mem::size_of::<T>() % mem::size_of::<U>(), 0);

        BufferRange {
            buffer: self.buffer.cast_ref(),
            offset,
            len,
        }
    }

    pub fn host_ptr(&self) -> *mut T {
        unsafe { self.buffer.host_ptr().add(self.offset) }
    }
}

impl Drop for BufferHandle {
    fn drop(&mut self) {
        // TODO: destroy buffer and free mem
    }
}

pub struct StagingMemoryPool {
    buffers: Vec<Buffer<u8>>,
    offset: usize,
}

impl StagingMemoryPool {
    pub fn alloc<T>(&mut self, len: usize) -> BufferRange<'_, T> {
        let last_buf_len = self.buffers.last().map(|buf| buf.len()).unwrap_or(0);

        let byte_len = len * mem::size_of::<T>();

        if last_buf_len - self.offset < byte_len {
            let cx = Context(());

            self.buffers.push(
                cx.new_buffer(last_buf_len.max(1024).max(byte_len) * 2)
                    .memory(
                        MemoryPropertyBit::DeviceLocal
                            | MemoryPropertyBit::HostVisible
                            | MemoryPropertyBit::HostCoherent,
                    )
                    .usage(BufferUsageBit::TransferSrc)
                    .create(),
            );
            self.offset = 0;
        }

        let buffer = self.buffers.last().unwrap();
        let start = self
            .offset
            .next_multiple_of(mem::align_of::<T>().max(mem::size_of::<T>()));
        self.offset = start + byte_len;

        buffer.range(dbg!(start..self.offset)).cast()
    }

    pub fn insert<T>(&mut self, data: &[T]) -> BufferRange<'_, T> {
        let range = self.alloc(data.len());
        unsafe { ptr::copy_nonoverlapping(data.as_ptr(), range.host_ptr(), data.len()) };
        range
    }

    pub fn reset(&mut self) {
        self.buffers.clear();
        self.offset = 0;
    }
}

pub struct Copies {
    stages: Vec<CopyStage>,
}

struct CopyStage {
    barriers: Vec<ImageMemoryBarrier>,
    commands: Vec<Box<dyn FnOnce(&CommandBuffer)>>,
}

pub struct CopyRecorder<'c> {
    copies: &'c mut Vec<CopyStage>,
    idx: usize,
}

impl Copies {
    pub fn record(&mut self) -> CopyRecorder<'_> {
        CopyRecorder {
            copies: &mut self.stages,
            idx: 0,
        }
    }

    pub fn flush(&mut self, command_buffer: &CommandBuffer) {
        for stage in self.stages.drain(..) {
            command_buffer.barriers(
                None,
                &[MemoryBarrier::new()
                    .src_access(
                        PipelineStageBit::AllCommands,
                        AccessBit::MemoryRead | AccessBit::MemoryWrite,
                    )
                    .dst_access(
                        PipelineStageBit::AllTransfer,
                        AccessBit::MemoryWrite | AccessBit::MemoryRead,
                    )],
                &[],
                &stage.barriers,
            );
            for cmd in stage.commands {
                cmd(command_buffer);
            }
        }
    }
}

impl CopyRecorder<'_> {
    pub fn barrier(&mut self, barrier: ImageMemoryBarrier) -> &mut Self {
        if self.idx == self.copies.len() {
            self.copies.push(CopyStage {
                commands: vec![],
                barriers: vec![],
            });
        }

        self.copies[self.idx].barriers.push(barrier);
        self
    }

    pub fn next_stage(&mut self, fun: impl FnOnce(&CommandBuffer) + 'static) -> &mut Self {
        if self.idx == self.copies.len() {
            self.copies.push(CopyStage {
                commands: vec![],
                barriers: vec![],
            });
        }

        self.copies[self.idx].commands.push(Box::new(fun));
        self.idx += 1;
        self
    }
}

pub struct Arena<T> {
    inner: Buffer<T>,
    usage: BufferUsageFlags,
    allocator: RangeAllocator<usize>,
    len: usize,
    uploads: BTreeMap<usize, (Buffer<T>, usize, usize)>,
}

impl<T: 'static> Arena<T> {
    pub fn alloc(&mut self, values: &[T], staging_pool: &mut StagingMemoryPool) -> usize {
        log::info!(
            "arena alloc {} - {} items",
            std::any::type_name::<T>(),
            values.len()
        );
        if values.is_empty() {
            return 0;
        }

        self.alloc_staged(staging_pool.insert(values))
    }

    fn alloc_staged(&mut self, staging_range: BufferRange<'_, T>) -> usize {
        let range = self
            .allocator
            .allocate_range(staging_range.len())
            .unwrap_or_else(|_| {
                self.len = self.len.max(staging_range.len()) * 2;
                self.allocator.grow_to(self.len);
                self.allocator.allocate_range(staging_range.len()).unwrap()
            });

        assert_eq!(range.len(), staging_range.len());

        self.uploads.insert(
            range.start,
            (
                staging_range.buffer.clone(),
                range.end - range.start,
                staging_range.offset,
            ),
        );

        range.start
    }

    pub fn device_address(&self) -> u64 {
        self.inner.device_address()
    }

    pub fn record_uploads(&mut self, copies: &mut Copies) -> bool {
        if self.len != self.inner.len() {
            let cx = Context(());

            let new_buffer = cx
                .new_buffer(self.len)
                .concurrent_sharing(&cx.all_queue_indices)
                .usage(self.usage)
                .create();

            copies.record().next_stage({
                let uploads = mem::take(&mut self.uploads);
                let ranges: Vec<_> = self.allocator.allocated_ranges().collect();
                let inner = self.inner.clone();
                let new_buffer = new_buffer.clone();

                move |cmd_buffer| {
                    log::info!("arena realloc {}", std::any::type_name::<T>());
                    let mut uploads = uploads.iter().peekable();
                    for mut range in ranges {
                        while let Some(&(start, (src_buffer, len, src_offset))) = uploads.peek()
                            && range.contains(start)
                        {
                            if range.start < *start {
                                log::info!("copying old range: {range:?}");
                                cmd_buffer.copy_buffer(
                                    &inner,
                                    &new_buffer,
                                    &[BufferCopy::new::<T>(
                                        range.start,
                                        range.start,
                                        *start - range.start,
                                    )],
                                );
                            }

                            uploads.next();
                            log::info!("copying new range: {range:?}");
                            cmd_buffer.copy_buffer(
                                &src_buffer,
                                &new_buffer,
                                &[BufferCopy::new::<T>(*src_offset, range.start, dbg!(*len))],
                            );
                            range.start += len;
                        }

                        if !range.is_empty() {
                            log::info!("copying old range: {range:?}");
                            cmd_buffer.copy_buffer(
                                &inner,
                                &new_buffer,
                                &[BufferCopy::new::<T>(range.start, range.start, range.len())],
                            );
                        }
                    }
                }
            });

            self.inner = new_buffer;

            self.uploads.clear();

            true
        } else if !self.uploads.is_empty() {
            let uploads = mem::take(&mut self.uploads);
            let inner = self.inner.clone();
            copies.record().next_stage(move |cmd_buffer| {
                for (dst_offset, (src_buffer, len, src_offset)) in &uploads {
                    cmd_buffer.copy_buffer(
                        &src_buffer,
                        &inner,
                        &[BufferCopy::new::<T>(*src_offset, *dst_offset, *len)],
                    );
                }
            });

            true
        } else {
            false
        }
    }
}

pub struct ArenaBuilder<'cx, T> {
    cx: &'cx Context,
    marker: PhantomData<T>,
    usage: vk::BufferUsageFlags,
    size: usize,
}

impl<T> ArenaBuilder<'_, T> {
    pub fn create(self) -> Arena<T> {
        let buffer = self.cx.create_buffer(
            &vk::BufferCreateInfo {
                s_type: vk::StructureType::BufferCreateInfo,
                p_next: ptr::null(),
                flags: vk::BufferCreateFlags(0),
                size: (self.size * mem::size_of::<T>()) as u64,
                usage: self.usage,
                sharing_mode: vk::SharingMode::Concurrent,
                queue_family_index_count: self.cx.all_queue_indices.len() as u32,
                p_queue_family_indices: self.cx.all_queue_indices.as_ptr(),
            },
            MemoryPropertyBit::DeviceLocal.into(),
        );

        Arena {
            inner: buffer,
            usage: self.usage,
            allocator: RangeAllocator::new(0..self.size),
            uploads: BTreeMap::new(),
            len: self.size,
        }
    }

    pub fn usage(mut self, usage: impl Into<BufferUsageFlags>) -> Self {
        self.usage = usage.into()
            | vk::BufferUsageBit::TransferSrc
            | vk::BufferUsageBit::TransferDst
            | vk::BufferUsageBit::ShaderDeviceAddress;
        self
    }

    pub fn capacity(mut self, capacity: usize) -> Self {
        assert!(capacity > 0);
        self.size = capacity;
        self
    }
}

#[derive(Clone, Copy)]
#[repr(C)]
pub struct DescriptorHandle(u32, u32);

pub struct DescriptorHeap {
    arena: Arena<u8>,
}

impl DescriptorHeap {
    pub fn view_descriptor<Fmt: Format>(
        &mut self,
        descriptor_type: DescriptorType,
        layout: ImageLayout,
        image: &impl Image<Fmt>,
        staging_mem_pool: &mut StagingMemoryPool,
    ) -> u32 {
        let cx = Context(());

        let staging_range =
            staging_mem_pool.alloc::<u8>(cx.heap_properties.image_descriptor_size as usize);

        unsafe {
            (cx.device_fns.write_resource_descriptors_ext)(
                cx.device,
                1,
                &vk::ResourceDescriptorInfo {
                    s_type: vk::StructureType::ResourceDescriptorInfo,
                    p_next: ptr::null(),
                    r#type: descriptor_type,
                    data: vk::ResourceDescriptorData {
                        p_image: &vk::ImageDescriptorInfo {
                            s_type: vk::StructureType::ImageDescriptorInfo,
                            p_next: ptr::null(),
                            p_view: &vk::ImageViewCreateInfo {
                                s_type: vk::StructureType::ImageViewCreateInfo,
                                p_next: ptr::null(),
                                flags: vk::ImageViewCreateFlags(0),
                                image: image.raw_handle().vk,
                                view_type: image.default_view_type(),
                                format: Fmt::vk(),
                                components: vk::ComponentMapping {
                                    r: vk::ComponentSwizzle::Identity,
                                    g: vk::ComponentSwizzle::Identity,
                                    b: vk::ComponentSwizzle::Identity,
                                    a: vk::ComponentSwizzle::Identity,
                                },
                                subresource_range: vk::ImageSubresourceRange {
                                    aspect_mask: Fmt::all_aspects(),
                                    base_mip_level: 0,
                                    level_count: !0,
                                    base_array_layer: 0,
                                    layer_count: image.layer_count(),
                                },
                            },
                            layout,
                        },
                    },
                },
                &vk::HostAddressRange {
                    address: staging_range.host_ptr().cast(),
                    size: staging_range.len,
                },
            )
            .must_succeed();
        };

        let offset = self.arena.alloc_staged(staging_range);

        (offset as u64 / cx.heap_properties.image_descriptor_size) as u32
    }

    pub fn sampler_descriptor(
        &mut self,
        mag_filter: Filter,
        min_filter: Filter,
        mipmap_mode: SamplerMipmapMode,
        address_mode_u: SamplerAddressMode,
        address_mode_v: SamplerAddressMode,
        address_mode_w: SamplerAddressMode,
        mip_lod_bias: f32,
        anisotropy_enable: bool,
        max_anisotropy: f32,
        min_lod: f32,
        max_lod: f32,
        staging_mem_pool: &mut StagingMemoryPool,
    ) -> u32 {
        let cx = Context(());

        let staging_range =
            staging_mem_pool.alloc::<u8>(cx.heap_properties.sampler_descriptor_size as usize);

        unsafe {
            (cx.device_fns.write_sampler_descriptors_ext)(
                cx.device,
                1,
                &vk::SamplerCreateInfo {
                    s_type: vk::StructureType::SamplerCreateInfo,
                    p_next: ptr::null(),
                    flags: vk::SamplerCreateFlags(0),
                    mag_filter,
                    min_filter,
                    mipmap_mode,
                    address_mode_u,
                    address_mode_v,
                    address_mode_w,
                    mip_lod_bias,
                    anisotropy_enable: anisotropy_enable as u32,
                    max_anisotropy,
                    compare_enable: 0,
                    compare_op: vk::CompareOp::Never,
                    min_lod,
                    max_lod,
                    border_color: vk::BorderColor::FloatTransparentBlack,
                    unnormalized_coordinates: 0,
                },
                &vk::HostAddressRange {
                    address: staging_range.host_ptr().cast(),
                    size: staging_range.len,
                },
            )
            .must_succeed();
        };

        let offset = self.arena.alloc_staged(staging_range);

        (offset as u64 / cx.heap_properties.sampler_heap_alignment) as u32
    }

    pub fn record_uploads(&mut self, copies: &mut Copies) -> bool {
        self.arena.record_uploads(copies)
    }
}

struct ImageHandle {
    vk: &'static vk::Image,
    block: Block,
}

struct ImageViewHandle {
    vk: &'static vk::ImageView,
    image: Arc<ImageHandle>,
}

pub struct ImageView<Fmt> {
    _format: PhantomData<Fmt>,
    handle: Arc<ImageViewHandle>,
}

impl<Fmt> Clone for ImageView<Fmt> {
    fn clone(&self) -> Self {
        Self {
            _format: PhantomData,
            handle: self.handle.clone(),
        }
    }
}

pub trait Image<Fmt> {
    fn raw_handle(&self) -> &Arc<ImageHandle>;
    fn layer_count(&self) -> u32;
    fn default_view_type(&self) -> vk::ImageViewType;
}

pub struct Image2DArray<Fmt> {
    _format: PhantomData<Fmt>,
    inner: Arc<ImageHandle>,
    extent: vk::Extent2D,
    layers: u32,
}

impl<Fmt> Clone for Image2DArray<Fmt> {
    fn clone(&self) -> Self {
        Self {
            _format: PhantomData,
            inner: self.inner.clone(),
            extent: self.extent,
            layers: self.layers,
        }
    }
}

impl<Fmt: Format> Image<Fmt> for Image2DArray<Fmt> {
    fn raw_handle(&self) -> &Arc<ImageHandle> {
        &self.inner
    }

    fn layer_count(&self) -> u32 {
        self.layers
    }

    fn default_view_type(&self) -> vk::ImageViewType {
        vk::ImageViewType::_2DArray
    }
}

impl<Fmt: Format> Image2DArray<Fmt> {
    pub fn barrier(&self, subresource_range: impl Into<ImageAspectFlags>) -> ImageMemoryBarrier {
        ImageMemoryBarrier {
            s_type: vk::StructureType::ImageMemoryBarrier2,
            p_next: ptr::null(),
            src_stage_mask: PipelineStageFlags(0),
            src_access_mask: AccessFlags(0),
            dst_stage_mask: PipelineStageFlags(0),
            dst_access_mask: AccessFlags(0),
            old_layout: ImageLayout::Undefined,
            new_layout: ImageLayout::Undefined,
            src_queue_family_index: !0,
            dst_queue_family_index: !0,
            image: self.inner.vk,
            subresource_range: vk::ImageSubresourceRange {
                aspect_mask: subresource_range.into(),
                base_mip_level: 0,
                level_count: !0,
                base_array_layer: 0,
                layer_count: self.layers,
            },
        }
    }
}

pub struct Image2D<Fmt> {
    _format: PhantomData<Fmt>,
    inner: Arc<ImageHandle>,
    extent: vk::Extent2D,
}

impl<Fmt> Clone for Image2D<Fmt> {
    fn clone(&self) -> Self {
        Self {
            _format: PhantomData,
            inner: self.inner.clone(),
            extent: self.extent,
        }
    }
}

impl<Fmt: Format> Image<Fmt> for Image2D<Fmt> {
    fn raw_handle(&self) -> &Arc<ImageHandle> {
        &self.inner
    }

    fn layer_count(&self) -> u32 {
        1
    }

    fn default_view_type(&self) -> vk::ImageViewType {
        vk::ImageViewType::_2D
    }
}

pub struct Atlas<Fmt> {
    image: Image2D<Fmt>,
    usage: vk::ImageUsageFlags,
    allocator: AtlasAllocator,
    size: vk::Extent2D,
    uploads: HashMap<euclid::Point2D<i32, euclid::UnknownUnit>, (Buffer<Fmt>, usize, (u32, u32))>,
    init: bool,
}

impl<Fmt: Format + 'static> Atlas<Fmt> {
    // TODO: remove
    pub fn image(&self) -> &Image2D<Fmt> {
        &self.image
    }

    pub fn upload(
        &mut self,
        data: ImageData<'_, Fmt>,
        staging_pool: &mut StagingMemoryPool,
    ) -> (u32, u32) {
        assert!(data.extent.depth == 1 && data.layers == 1);
        if data.extent.height == 0 || data.extent.width == 0 {
            return (0, 0);
        }

        let alloc_size = guillotiere::size2(data.extent.width as i32, data.extent.height as i32);

        let allocation = self.allocator.allocate(alloc_size).unwrap_or_else(|| {
            eprintln!("ATLAS REALLOC");
            self.size = vk::Extent2D {
                width: data.extent.width.max(self.size.width) * 2,
                height: data.extent.height.max(self.size.height) * 2,
            };

            // cx.queue_image_transfer(
            //     self.image.range(.., ..),
            //     new_image.range(..self.image.extent.width, ..self.image.extent.height),
            // );

            self.allocator.grow(guillotiere::size2(
                self.size.width as i32,
                self.size.height as i32,
            ));

            self.allocator.allocate(alloc_size).unwrap()
        });

        assert!(allocation.rectangle.width() >= data.extent.width as i32);
        assert!(allocation.rectangle.height() >= data.extent.height as i32);

        let staging_range = staging_pool.insert(data.as_slice());

        self.uploads.insert(
            allocation.rectangle.min,
            (
                staging_range.buffer.clone(),
                staging_range.offset,
                (
                    allocation.rectangle.width() as u32,
                    allocation.rectangle.height() as u32,
                ),
            ),
        );

        // self.timeline_point = cx.queue_data_to_image_transfer(
        //     data,
        //     self.image.range(
        //         allocation.rectangle.min.x as u32..allocation.rectangle.min.x as u32 + extent.width,
        //         allocation.rectangle.min.y as u32
        //             ..allocation.rectangle.min.y as u32 + extent.height,
        //     ),
        //     vk::ImageLayout::General,
        //     false,
        // );

        (
            allocation.rectangle.min.x as u32,
            allocation.rectangle.min.y as u32,
        )
    }

    // Initial image transition
    // Realloc copy
    // Upload copy

    pub fn record_uploads(&mut self, copies: &mut Copies) -> bool {
        let mut copies = copies.record();

        if mem::take(&mut self.init) {
            copies.barrier(
                self.image
                    .barrier(Fmt::all_aspects())
                    .new_layout(vk::ImageLayout::General)
                    .dst_access(
                        vk::PipelineStageBit2::AllTransfer,
                        vk::AccessBit2::TransferWrite,
                    ),
            );
        }

        if self.image.extent.width != self.size.width
            || self.image.extent.height != self.size.height
        {
            let cx = Context(());

            let new_image = cx
                .new_image(self.size)
                .concurrent_sharing()
                .usage(self.usage)
                .create();

            copies
                .barrier(
                    new_image
                        .barrier(Fmt::all_aspects())
                        .new_layout(vk::ImageLayout::General)
                        .dst_access(PipelineStageBit::AllTransfer, AccessBit::TransferWrite),
                )
                .barrier(
                    self.image
                        .barrier(Fmt::all_aspects())
                        .src_access(PipelineStageBit::AllTransfer, AccessBit::TransferWrite)
                        .dst_access(PipelineStageBit::AllTransfer, AccessBit::TransferRead),
                )
                .next_stage({
                    let uploads = mem::take(&mut self.uploads);
                    let new_image = new_image.clone();
                    let image = self.image.clone();
                    let mut rects = vec![];
                    self.allocator.for_each_allocated_rectangle(|_, rect| {
                        rects.push(*rect);
                    });

                    move |cmd_buffer| {
                        for rect in rects {
                            if let Some((src_buffer, src_offset, (width, height))) =
                                uploads.get(&rect.min)
                            {
                                cmd_buffer.copy_buffer_to_image(
                                    &src_buffer,
                                    &new_image,
                                    vk::ImageLayout::General,
                                    &[BufferImageCopy {
                                        s_type: vk::StructureType::BufferImageCopy2,
                                        p_next: ptr::null(),
                                        buffer_offset: (src_offset * mem::size_of::<Fmt>()) as u64,
                                        buffer_row_length: 0,
                                        buffer_image_height: 0,
                                        image_subresource: vk::ImageSubresourceLayers {
                                            aspect_mask: Fmt::all_aspects(),
                                            mip_level: 0,
                                            base_array_layer: 0,
                                            layer_count: 1,
                                        },
                                        image_offset: vk::Offset3D {
                                            x: rect.min.x,
                                            y: rect.min.y,
                                            z: 0,
                                        },
                                        image_extent: vk::Extent3D {
                                            width: *width,
                                            height: *height,
                                            depth: 1,
                                        },
                                    }],
                                );
                            } else {
                                let subresource = vk::ImageSubresourceLayers {
                                    aspect_mask: Fmt::all_aspects(),
                                    mip_level: 0,
                                    base_array_layer: 0,
                                    layer_count: 1,
                                };

                                let offset = vk::Offset3D {
                                    x: rect.min.x,
                                    y: rect.min.y,
                                    z: 0,
                                };

                                cmd_buffer.copy_image(
                                    &image,
                                    vk::ImageLayout::General,
                                    &new_image,
                                    vk::ImageLayout::General,
                                    &[ImageCopy {
                                        s_type: vk::StructureType::ImageCopy2,
                                        p_next: ptr::null(),
                                        src_subresource: subresource,
                                        src_offset: offset,
                                        dst_subresource: subresource,
                                        dst_offset: offset,
                                        extent: vk::Extent3D {
                                            width: rect.width() as u32,
                                            height: rect.height() as u32,
                                            depth: 1,
                                        },
                                    }],
                                );
                            }
                        }
                    }
                });

            self.image = new_image;

            true
        } else if !self.uploads.is_empty() {
            let uploads = mem::take(&mut self.uploads);
            let image = self.image.clone();
            copies.next_stage(move |cmd_buffer| {
                for (min, (src_buffer, src_offset, (width, height))) in uploads {
                    cmd_buffer.copy_buffer_to_image(
                        &src_buffer,
                        &image,
                        vk::ImageLayout::General,
                        &[BufferImageCopy {
                            s_type: vk::StructureType::BufferImageCopy2,
                            p_next: ptr::null(),
                            buffer_offset: (src_offset * mem::size_of::<Fmt>()) as u64,
                            buffer_row_length: 0,
                            buffer_image_height: 0,
                            image_subresource: vk::ImageSubresourceLayers {
                                aspect_mask: Fmt::all_aspects(),
                                mip_level: 0,
                                base_array_layer: 0,
                                layer_count: 1,
                            },
                            image_offset: vk::Offset3D {
                                x: min.x,
                                y: min.y,
                                z: 0,
                            },
                            image_extent: vk::Extent3D {
                                width,
                                height,
                                depth: 1,
                            },
                        }],
                    );
                }
            });

            true
        } else {
            false
        }
    }
}

pub struct AtlasBuilder<'cx, Fmt> {
    cx: &'cx Context,
    format: PhantomData<Fmt>,
    usage: vk::ImageUsageFlags,
    size: vk::Extent2D,
}

impl<Fmt: Format> AtlasBuilder<'_, Fmt> {
    pub fn create(self) -> Atlas<Fmt> {
        let image = self.cx.create_image::<Fmt>(
            self.usage,
            vk::ImageTiling::Optimal,
            self.size,
            Some(&self.cx.all_queue_indices),
            1,
            1,
            ptr::null(),
            ptr::null(),
        );

        Atlas {
            image: Image2D {
                _format: PhantomData,
                inner: image,
                extent: self.size,
            },
            usage: self.usage,
            allocator: AtlasAllocator::new(guillotiere::size2(
                self.size.width as i32,
                self.size.height as i32,
            )),
            size: self.size,
            uploads: HashMap::new(),
            init: true,
        }
    }

    pub fn usage(mut self, usage: impl Into<ImageUsageFlags>) -> Self {
        self.usage = usage.into() | ImageUsageBit::TransferSrc | ImageUsageBit::TransferDst;
        self
    }

    pub fn capacity(mut self, capacity: vk::Extent2D) -> Self {
        assert!(capacity.width > 0 && capacity.height > 0);
        self.size = capacity;
        self
    }
}

#[derive(Clone, Copy)]
pub struct ImageData<'data, Fmt> {
    _format: PhantomData<&'data Fmt>,
    ptr: *const Fmt,
    extent: vk::Extent3D,
    layers: u32,
}

impl<'data, Fmt> ImageData<'data, Fmt> {
    pub fn from_slice(data: &'data [Fmt], size: vk::Extent3D) -> Self {
        assert_eq!(
            data.len(),
            size.width as usize * size.height as usize * size.depth as usize
        );

        Self {
            _format: PhantomData,
            ptr: data.as_ptr(),
            extent: size,
            layers: 1,
        }
    }

    pub fn as_slice(self) -> &'data [Fmt] {
        unsafe {
            slice::from_raw_parts(
                self.ptr,
                self.extent.width as usize
                    * self.extent.height as usize
                    * self.extent.depth as usize,
            )
        }
    }
}

// TODO: account for arrayed XOR 3D?
pub struct ImageRange<'img, Fmt> {
    _format: PhantomData<Fmt>,
    image: &'img Arc<ImageHandle>,
    offset: vk::Offset3D,
    first_layer: u32,
    extent: vk::Extent3D,
    layers: u32,
    aspect: ImageAspectFlags,
    mip: u32,
}

// GpuBuffer - Device local static buffer (uninit or preset content)
// GpuImage - Device local static image (uninit or preset content)

// resize? yes

// GpuVec - Host visible vec-like buffer

// GpuArena - Device local suballocated buffer
// GpuAtlas - Device local suballocated image

// fn wait_for_upload()

unsafe impl Send for Block {}
unsafe impl Sync for Block {}

#[derive(Clone, Copy)]
struct Block {
    size: u64,
    offset: u64,
    memory: *const vk::DeviceMemory,
    host_ptr: *mut u8,
}

struct DeviceMemoryAllocator {
    memories: Mutex<Vec<(usize, Vec<Block>)>>,
    memory_props: vk::PhysicalDeviceMemoryProperties,
}

impl DeviceMemoryAllocator {
    fn alloc_for_image(
        &self,
        image: *const vk::Image,
        extend_alloc_info: *const ffi::c_void,
    ) -> Block {
        let cx = Context(());

        unsafe {
            let mut dedicated_requirements = vk::MemoryDedicatedRequirements {
                s_type: vk::StructureType::MemoryDedicatedRequirements,
                p_next: ptr::null_mut(),
                prefers_dedicated_allocation: 0,
                requires_dedicated_allocation: 0,
            };

            let mut mem_requirements = vk::MemoryRequirements2 {
                s_type: vk::StructureType::MemoryRequirements2,
                p_next: ptr::from_mut(&mut dedicated_requirements).cast(),
                memory_requirements: vk::MemoryRequirements {
                    size: 0,
                    alignment: 0,
                    memory_type_bits: 0,
                },
            };

            (cx.device_fns.get_image_memory_requirements2)(
                cx.device,
                &vk::ImageMemoryRequirementsInfo2 {
                    s_type: vk::StructureType::ImageMemoryRequirementsInfo2,
                    p_next: ptr::null(),
                    image,
                },
                &mut mem_requirements,
            );

            let block = if !extend_alloc_info.is_null()
                || dedicated_requirements.prefers_dedicated_allocation != 0
                || dedicated_requirements.requires_dedicated_allocation != 0
            {
                self.alloc_dedicated(
                    MemoryPropertyBit::DeviceLocal.into(),
                    mem_requirements.memory_requirements.memory_type_bits,
                    mem_requirements.memory_requirements.size,
                    image,
                    ptr::null(),
                    extend_alloc_info,
                )
            } else {
                self.alloc(
                    MemoryPropertyBit::DeviceLocal.into(),
                    mem_requirements.memory_requirements.memory_type_bits,
                    mem_requirements.memory_requirements.size,
                    mem_requirements.memory_requirements.alignment,
                )
            };

            (cx.device_fns.bind_image_memory)(cx.device, image, block.memory, block.offset)
                .must_succeed();

            block
        }
    }

    fn alloc(
        &self,
        mem_props: MemoryPropertyFlags,
        mem_type_bits: u32,
        size: u64,
        align: u64,
    ) -> Block {
        let cx = Context(());

        unsafe {
            let memory_type_index = self.find_memory(mem_props, mem_type_bits);

            let mut memories = self.memories.lock().unwrap();
            let (next_grow_size, holes) = &mut memories[memory_type_index as usize];

            let i = holes.partition_point(|hole| hole.size < size);
            let idx = holes[i..]
                .iter()
                .position(|hole| {
                    (hole.offset.next_multiple_of(align) - hole.offset) + size <= hole.size
                })
                .map(|j| j + i);

            let hole = match idx {
                Some(idx) => holes.remove(idx),
                None => {
                    let size = (*next_grow_size as u64).max(size);
                    let mut device_memory = ptr::null();
                    (cx.device_fns.allocate_memory)(
                        cx.device,
                        &vk::MemoryAllocateInfo {
                            s_type: vk::StructureType::MemoryAllocateInfo,
                            p_next: ptr::from_ref(&vk::MemoryAllocateFlagsInfo {
                                s_type: vk::StructureType::MemoryAllocateFlagsInfo,
                                p_next: ptr::null(),
                                flags: vk::MemoryAllocateBit::DeviceAddress.into(),
                                device_mask: 0,
                            })
                            .cast(),
                            allocation_size: size,
                            memory_type_index,
                        },
                        ptr::null(),
                        &mut device_memory,
                    )
                    .as_result()
                    .unwrap();

                    *next_grow_size *= 2;

                    let mut ptr = ptr::null_mut();

                    if mem_props & MemoryPropertyBit::HostVisible.into()
                        == MemoryPropertyBit::HostVisible.into()
                    {
                        (cx.device_fns.map_memory)(
                            cx.device,
                            device_memory,
                            0,
                            size,
                            vk::MemoryMapFlags(0),
                            &mut ptr,
                        )
                        .as_result()
                        .unwrap();
                    }

                    Block {
                        size,
                        offset: 0,
                        memory: device_memory,
                        host_ptr: ptr.cast(),
                    }
                }
            };

            let padding = hole.offset.next_multiple_of(align) - hole.offset;
            if padding != 0 {
                let idx = holes.partition_point(|hole| hole.size < padding);
                holes.insert(
                    idx,
                    Block {
                        size: padding,
                        ..hole
                    },
                );
            }
            let tail = hole.size - padding - size;
            if tail > 0 {
                let idx = holes.partition_point(|hole| hole.size < tail);
                let diff = padding + size;
                holes.insert(
                    idx,
                    Block {
                        size: tail,
                        offset: hole.offset + diff,
                        host_ptr: hole.host_ptr.add(diff as usize),
                        ..hole
                    },
                );
            }

            debug_assert!((hole.offset + padding).is_multiple_of(align));

            Block {
                size,
                offset: hole.offset + padding,
                memory: hole.memory,
                host_ptr: hole.host_ptr.add(padding as usize),
            }
        }
    }

    fn dealloc(&self, mem_type_idx: u32, block: Block) {
        let mut memories = self.memories.lock().unwrap();
        let blocks = &mut memories[mem_type_idx as usize].1;
        let idx = blocks.partition_point(|hole| hole.size < block.size);
        blocks.insert(idx, block);
    }

    fn alloc_dedicated(
        &self,
        mem_props: MemoryPropertyFlags,
        mem_type_bits: u32,
        size: u64,
        image: *const vk::Image,
        buffer: *const vk::Buffer,
        p_next: *const ffi::c_void,
    ) -> Block {
        let cx = Context(());

        let memory_type_index = self.find_memory(mem_props, mem_type_bits);

        let mut device_memory = ptr::null();
        unsafe {
            (cx.device_fns.allocate_memory)(
                cx.device,
                &vk::MemoryAllocateInfo {
                    s_type: vk::StructureType::MemoryAllocateInfo,
                    p_next: ptr::from_ref(&vk::MemoryDedicatedAllocateInfo {
                        s_type: vk::StructureType::MemoryDedicatedAllocateInfo,
                        p_next,
                        image,
                        buffer,
                    })
                    .cast(),
                    allocation_size: size,
                    memory_type_index,
                },
                ptr::null(),
                &mut device_memory,
            )
            .must_succeed();

            let mut host_ptr = ptr::null_mut();

            if mem_props & MemoryPropertyBit::HostVisible.into()
                == MemoryPropertyBit::HostVisible.into()
            {
                (cx.device_fns.map_memory)(
                    cx.device,
                    device_memory,
                    0,
                    size,
                    vk::MemoryMapFlags(0),
                    &mut host_ptr,
                )
                .must_succeed();
            }

            Block {
                size,
                offset: 0,
                memory: device_memory,
                host_ptr: host_ptr.cast(),
            }
        }
    }

    fn find_memory(&self, mem_props: MemoryPropertyFlags, mem_type_bits: u32) -> u32 {
        self.memory_props.memory_types[..self.memory_props.memory_type_count as _]
            .iter()
            .enumerate()
            .position(|(i, mem_type)| {
                (1 << i) & mem_type_bits != 0 && mem_type.property_flags & mem_props == mem_props
            })
            .unwrap() as u32
    }
}

pub struct BufferBuilder<'cx, T> {
    marker: PhantomData<T>,
    cx: &'cx Context,
    memory_type: vk::MemoryPropertyFlags,
    size: usize,
    usage: BufferUsageFlags,
    queues: Option<&'cx [u32]>,
}

// Memory Type
// Sharing
// Usage flags
impl<'cx, T> BufferBuilder<'cx, T> {
    pub fn create(self) -> Buffer<T> {
        self.cx.create_buffer(
            &vk::BufferCreateInfo {
                s_type: vk::StructureType::BufferCreateInfo,
                p_next: ptr::null(),
                flags: vk::BufferCreateFlags(0),
                size: (self.size * mem::size_of::<T>()) as u64,
                usage: self.usage,
                sharing_mode: match self.queues {
                    Some(_) => vk::SharingMode::Concurrent,
                    None => vk::SharingMode::Exclusive,
                },
                queue_family_index_count: self.queues.map(|q| q.len() as u32).unwrap_or(0),
                p_queue_family_indices: self.queues.map(|q| q.as_ptr()).unwrap_or(ptr::null()),
            },
            self.memory_type,
        )
    }

    pub fn memory(mut self, flags: impl Into<vk::MemoryPropertyFlags>) -> Self {
        self.memory_type = flags.into();
        self
    }

    pub fn usage(mut self, usage: impl Into<BufferUsageFlags>) -> Self {
        self.usage = usage.into();
        self
    }

    pub fn concurrent_sharing(mut self, queues: &'cx [u32]) -> Self {
        self.queues = Some(queues);
        self
    }
}

pub struct Image2DBuilder<'cx, Fmt> {
    cx: &'cx Context,
    format: PhantomData<Fmt>,
    usage: vk::ImageUsageFlags,
    size: vk::Extent2D,
    concurrent_sharing: bool,
}

impl<Fmt: Format> Image2DBuilder<'_, Fmt> {
    pub fn create(self) -> Image2D<Fmt> {
        let inner = self.cx.create_image::<Fmt>(
            self.usage,
            vk::ImageTiling::Optimal,
            self.size,
            match self.concurrent_sharing {
                true => Some(&self.cx.user_queue_indices),
                false => None,
            },
            1,
            1,
            ptr::null(),
            ptr::null(),
        );

        Image2D {
            _format: PhantomData,
            inner,
            extent: self.size,
        }
    }

    pub fn usage(mut self, usage: impl Into<ImageUsageFlags>) -> Self {
        self.usage = usage.into();
        self
    }

    pub fn concurrent_sharing(mut self) -> Self {
        self.concurrent_sharing = true;
        self
    }
}

pub struct Image2DArrayBuilder<'cx, Fmt> {
    cx: &'cx Context,
    format: PhantomData<Fmt>,
    usage: vk::ImageUsageFlags,
    size: vk::Extent2D,
    layers: u32,
    mip_levels: u32,
    concurrent_sharing: bool,
}

impl<Fmt: Format> Image2DArrayBuilder<'_, Fmt> {
    pub fn create(self) -> Image2DArray<Fmt> {
        let inner = self.cx.create_image::<Fmt>(
            self.usage,
            vk::ImageTiling::Optimal,
            self.size,
            match self.concurrent_sharing {
                true => Some(&self.cx.user_queue_indices),
                false => None,
            },
            self.layers,
            self.mip_levels,
            ptr::null(),
            ptr::null(),
        );

        Image2DArray {
            _format: PhantomData,
            inner,
            extent: self.size,
            layers: self.layers,
        }
    }

    pub fn usage(mut self, usage: impl Into<ImageUsageFlags>) -> Self {
        self.usage = usage.into();
        self
    }

    pub fn concurrent_sharing(mut self) -> Self {
        self.concurrent_sharing = true;
        self
    }

    pub fn mip_levels(mut self, levels: u32) -> Self {
        self.mip_levels = levels;
        self
    }
}

// enum WindowState {
//     Initializing {
//         max_width: Option<i32>,
//         max_height: Option<i32>,
//     },
// }
pub trait Render: 'static {
    type Frame;

    fn new_frame(app: &mut App) -> Self::Frame;

    fn render(
        &mut self,
        app: &mut App,
        window: &mut Window,
        frame: &mut Self::Frame,
        // n: u64,
    );
}

#[derive(Clone, Copy, Debug)]
pub struct ScaleFactor(u32);

impl ops::Mul<ScaleFactor> for u32 {
    type Output = u32;

    fn mul(self, scale: ScaleFactor) -> Self::Output {
        let m = self * scale.0;
        let even = m / 120;
        let rem = m % 120;
        match rem {
            ..60 => even,
            _ => even + 1,
        }
    }
}

struct WindowData {
    window: Window,
    render_frame: Box<dyn FnMut(&mut App, &mut Window) -> Result<(), vk::Result>>,
}

pub struct Window {
    platform: PlatformWindow,
    width: u32,
    height: u32,
    scale_factor: ScaleFactor,
    mouse_position: Option<(u32, u32)>,
    next_image_idx: usize,
    images: Slab<SwapchainImage>,
    // image_views: Vec<*const ImageView>,
    // wait_semaphores: Vec<*const Semaphore>,
    // signal_semaphores: Vec<*const Semaphore>,
}

impl Window {
    pub fn size(&self) -> Extent2D {
        Extent2D {
            width: self.width,
            height: self.height,
        }
    }

    pub fn physical_size(&self) -> Extent2D {
        Extent2D {
            width: self.width * self.scale_factor,
            height: self.height * self.scale_factor,
        }
    }

    pub fn mouse_position(&self) -> Option<(u32, u32)> {
        self.mouse_position
    }

    pub fn physical_mouse_position(&self) -> Option<(u32, u32)> {
        self.mouse_position
            .map(|(x, y)| (x * self.scale_factor, y * self.scale_factor))
    }

    pub fn image(&self) -> &Image2D<B8G8R8A8Unorm> {
        &self.images[self.next_image_idx].image
    }

    pub fn image_view(&self) -> &ImageView<B8G8R8A8Unorm> {
        &self.images[self.next_image_idx].view
    }

    pub fn wait_semaphore(&self) -> &Semaphore {
        &self.images[self.next_image_idx].wait_semaphore
    }

    pub fn signal_semaphore(&self) -> &Semaphore {
        &self.images[self.next_image_idx].signal_semaphore
    }
}

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct SubmitInfo {
    s_type: vk::StructureType,
    p_next: *const ffi::c_void,
    flags: vk::SubmitFlags,
    wait_semaphore_info_count: u32,
    p_wait_semaphore_infos: *const SemaphoreSubmitInfo,
    command_buffer_info_count: u32,
    p_command_buffer_infos: *const CommandBufferSubmitInfo,
    signal_semaphore_info_count: u32,
    p_signal_semaphore_infos: *const SemaphoreSubmitInfo,
}

pub struct Queue {
    vk: Mutex<&'static vk::Queue>,
    family_idx: u32,
}

impl Queue {
    pub fn create_command_pool(&self) -> CommandPool {
        let cx = Context(());

        let mut this = ptr::null();
        unsafe {
            (cx.device_fns.create_command_pool)(
                cx.device,
                &vk::CommandPoolCreateInfo {
                    s_type: vk::StructureType::CommandPoolCreateInfo,
                    p_next: ptr::null(),
                    flags: vk::CommandPoolCreateFlags(0),
                    queue_family_index: self.family_idx,
                },
                ptr::null(),
                &mut this,
            )
            .must_succeed();
            CommandPool {
                vk: &*this,
                next_empty_buffer: Cell::new(0),
                buffers: vec![],
            }
        }
    }

    pub fn submit(&self, submit_infos: &[SubmitInfo]) {
        let queue = self.vk.lock().unwrap();
        let cx = Context(());

        unsafe {
            (cx.device_fns.queue_submit_2)(
                *queue,
                submit_infos.len() as u32,
                submit_infos.as_ptr(),
                ptr::null(),
            )
            .must_succeed();
        }
    }

    pub fn family_idx(&self) -> u32 {
        self.family_idx
    }
}

#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct Shader(&'static vk::Shader);

const _: () = assert!(mem::size_of::<Option<Shader>>() == mem::size_of::<*const vk::Shader>());

// TRANSFER THREAD:
// one data buffer
// two command buffers
// other threads lock a mutex, write into the data buffer, and drop a message
//
// what if the buffer is full? requesting threads block, queue up, and wait (Fair mutex). How do we notify them?
//
// How to avoid premature submittion? Explicit flush commands in the queue

struct TransferContext {
    buffer: Buffer<u8>,
    sender: Sender<TransferCommand>,
    // semaphore: Owned<Semaphore>,
    written_bytes: usize,
}

// impl TransferContext {
//     fn reserve_range(&mut self, len: usize) -> *mut u8 {
//         let cx = Context(());

//         let start_idx = self.written_bytes % self.buffer.len();
//         let tail_bytes = self.buffer.len() - start_idx;
//         let (wait_for, start_offset) = if tail_bytes >= len {
//             (
//                 (self.written_bytes + len).saturating_sub(self.buffer.len()),
//                 start_idx,
//             )
//         } else if self.buffer.len() >= len {
//             (self.written_bytes - start_idx + len, 0)
//         } else {
//             panic!("transfer too large");
//         };

//         // FIXME: NOOO, buffer wraparound
//         self.written_bytes += len;

//         unsafe {
//             (cx.device_fns.wait_semaphores)(
//                 cx.device,
//                 &vk::SemaphoreWaitInfo {
//                     s_type: vk::StructureType::SemaphoreWaitInfo,
//                     p_next: ptr::null(),
//                     flags: vk::SemaphoreWaitFlags(0),
//                     semaphore_count: 1,
//                     p_semaphores: &self.semaphore.0.as_ptr().cast_const(),
//                     p_values: &(wait_for as u64),
//                 },
//                 !0,
//             )
//             .must_succeed();

//             self.buffer.handle.block.host_ptr.add(start_offset)
//         }
//     }
// }

enum TransferCommand {
    Flush,
    DataToBuffer {
        buffer: Arc<BufferHandle>,
        dst_offset: u32,
        size: u32,
    },
    DataToImage {
        image: Arc<ImageHandle>,
        subresource: vk::ImageSubresourceLayers,
        layout: vk::ImageLayout,
        offset: vk::Offset3D,
        extent: vk::Extent3D,
        block_size: u32,
    },
    TransitionImage {
        image: Arc<ImageHandle>,
        aspects: vk::ImageAspectFlags,
        layout: vk::ImageLayout,
    },
    ImageToImage {
        src: Arc<ImageHandle>,
        dst: Arc<ImageHandle>,
        extent: vk::Extent2D,
        aspects: vk::ImageAspectFlags,
    },
    BufferToBuffer {
        src: Arc<BufferHandle>,
        dst: Arc<BufferHandle>,
        size: usize,
    },
}

// fn start_transfer_thread(transfer_queue: Queue) {
//     let cx = Context(());

//     let (sender, receiver) = mpsc::channel();

//     let upload_buffer_size = 64 * 1024 * 1024;
//     let buffer = cx
//         .new_buffer(upload_buffer_size)
//         .usage(BufferUsageBit::TransferSrc)
//         .create();

//     let src_buffer = buffer.handle.vk;

//     *cx.transfer.lock().unwrap() = Some(TransferContext {
//         buffer,
//         sender,
//         semaphore: cx.create_timeline_semaphore(0),
//         written_bytes: 0,
//     });

//     thread::spawn(move || -> ! {
//         let semaphore = cx
//             .transfer
//             .lock()
//             .unwrap()
//             .as_mut()
//             .unwrap()
//             .semaphore
//             .0
//             .as_ptr()
//             .cast_const();

//         let mut ready_pool = transfer_queue.create_command_pool();
//         ready_pool.allocate_command_buffers(1);
//         let mut ready_value = 0;
//         let mut pending_pool = transfer_queue.create_command_pool();
//         pending_pool.allocate_command_buffers(1);
//         let mut pending_value = 0;

//         loop {
//             unsafe {
//                 (cx.device_fns.wait_semaphores)(
//                     cx.device,
//                     &vk::SemaphoreWaitInfo {
//                         s_type: vk::StructureType::SemaphoreWaitInfo,
//                         p_next: ptr::null(),
//                         flags: vk::SemaphoreWaitFlags(0),
//                         semaphore_count: 1,
//                         p_semaphores: &semaphore,
//                         p_values: &(ready_value as u64),
//                     },
//                     !0,
//                 )
//                 .must_succeed();
//             }
//             ready_value = pending_value;
//             ready_pool.reset();
//             let buffer = ready_pool.get_buffer();

//             buffer.begin();
//             while let Ok(command) = receiver.recv() {
//                 match command {
//                     TransferCommand::DataToBuffer {
//                         buffer: dst_buffer,
//                         dst_offset,
//                         size,
//                     } => unsafe {
//                         eprintln!("data to buffer @ {dst_offset} ({size})");
//                         let src_offset = ready_value % upload_buffer_size;
//                         ready_value += size as usize;
//                         (cx.device_fns.cmd_copy_buffer_2)(
//                             buffer,
//                             &vk::CopyBufferInfo2 {
//                                 s_type: vk::StructureType::CopyBufferInfo2,
//                                 p_next: ptr::null(),
//                                 src_buffer,
//                                 dst_buffer: dst_buffer.vk,
//                                 region_count: 1,
//                                 p_regions: &vk::BufferCopy2 {
//                                     s_type: vk::StructureType::BufferCopy2,
//                                     p_next: ptr::null(),
//                                     src_offset: src_offset as u64,
//                                     dst_offset: dst_offset as u64,
//                                     size: size as u64,
//                                 },
//                             },
//                         );
//                     },
//                     TransferCommand::DataToImage {
//                         image,
//                         subresource,
//                         layout,
//                         offset,
//                         extent,
//                         block_size,
//                     } => unsafe {
//                         eprintln!(
//                             "data to image {}",
//                             extent.width as usize
//                                 * extent.height as usize
//                                 * extent.depth as usize
//                                 * subresource.layer_count as usize
//                                 * block_size as usize
//                         );
//                         let src_offset = ready_value % upload_buffer_size;
//                         ready_value += extent.width as usize
//                             * extent.height as usize
//                             * extent.depth as usize
//                             * subresource.layer_count as usize
//                             * block_size as usize;
//                         (cx.device_fns.cmd_copy_buffer_to_image_2)(
//                             buffer,
//                             &vk::CopyBufferToImageInfo2 {
//                                 s_type: vk::StructureType::CopyBufferToImageInfo2,
//                                 p_next: ptr::null(),
//                                 src_buffer,
//                                 dst_image: image.vk,
//                                 dst_image_layout: layout,
//                                 region_count: 1,
//                                 p_regions: &vk::BufferImageCopy2 {
//                                     s_type: vk::StructureType::BufferImageCopy2,
//                                     p_next: ptr::null(),
//                                     buffer_offset: src_offset as u64,
//                                     buffer_row_length: 0,
//                                     buffer_image_height: 0,
//                                     image_subresource: subresource,
//                                     image_offset: offset,
//                                     image_extent: extent,
//                                 },
//                             },
//                         );
//                     },
//                     TransferCommand::TransitionImage {
//                         image,
//                         aspects,
//                         layout,
//                     } => {
//                         buffer.barriers(
//                             None,
//                             &[],
//                             &[],
//                             &[ImageMemoryBarrier2 {
//                                 s_type: vk::StructureType::ImageMemoryBarrier2,
//                                 p_next: ptr::null(),
//                                 src_stage_mask: vk::PipelineStageFlags2(0),
//                                 src_access_mask: vk::AccessFlags2(0),
//                                 dst_stage_mask: PipelineStageBit2::AllTransfer.into(),
//                                 dst_access_mask: AccessBit2::TransferWrite.into(),
//                                 old_layout: vk::ImageLayout::Undefined,
//                                 new_layout: layout,
//                                 src_queue_family_index: 0,
//                                 dst_queue_family_index: 0,
//                                 image: image.vk,
//                                 subresource_range: vk::ImageSubresourceRange {
//                                     aspect_mask: aspects,
//                                     base_mip_level: 0,
//                                     level_count: !0,
//                                     base_array_layer: 0,
//                                     layer_count: !0,
//                                 },
//                             }],
//                         );
//                     }
//                     TransferCommand::ImageToImage {
//                         src,
//                         dst,
//                         extent,
//                         aspects,
//                     } => unsafe {
//                         eprintln!("image to iamge");
//                         let subresource = vk::ImageSubresourceLayers {
//                             aspect_mask: aspects,
//                             mip_level: 0,
//                             base_array_layer: 0,
//                             layer_count: 1,
//                         };

//                         (cx.device_fns.cmd_copy_image_2)(
//                             buffer,
//                             &vk::CopyImageInfo2 {
//                                 s_type: vk::StructureType::CopyImageInfo2,
//                                 p_next: ptr::null(),
//                                 src_image: src.vk,
//                                 src_image_layout: ImageLayout::General,
//                                 dst_image: dst.vk,
//                                 dst_image_layout: ImageLayout::General,
//                                 region_count: 1,
//                                 p_regions: &vk::ImageCopy2 {
//                                     s_type: vk::StructureType::ImageCopy2,
//                                     p_next: ptr::null(),
//                                     src_subresource: subresource,
//                                     src_offset: vk::Offset3D { x: 0, y: 0, z: 0 },
//                                     dst_subresource: subresource,
//                                     dst_offset: vk::Offset3D { x: 0, y: 0, z: 0 },
//                                     extent: vk::Extent3D {
//                                         width: extent.width,
//                                         height: extent.height,
//                                         depth: 0,
//                                     },
//                                 },
//                             },
//                         );
//                     },
//                     TransferCommand::BufferToBuffer { src, dst, size } => unsafe {
//                         eprintln!("buffer to buffer");
//                         (cx.device_fns.cmd_copy_buffer_2)(
//                             buffer,
//                             &vk::CopyBufferInfo2 {
//                                 s_type: vk::StructureType::CopyBufferInfo2,
//                                 p_next: ptr::null(),
//                                 src_buffer: src.vk,
//                                 dst_buffer: dst.vk,
//                                 region_count: 1,
//                                 p_regions: &vk::BufferCopy2 {
//                                     s_type: vk::StructureType::BufferCopy2,
//                                     p_next: ptr::null(),
//                                     src_offset: 0,
//                                     dst_offset: 0,
//                                     size: size as u64,
//                                 },
//                             },
//                         );
//                     },
//                     TransferCommand::Flush => break,
//                 }
//             }
//             buffer.end();

//             eprintln!("submit SIGNAL {ready_value} WAIT {pending_value}");
//             transfer_queue.submit(
//                 &[SubmitInfo2::new(&[buffer.submit_info()])
//                     .signal_semaphores(&[SemaphoreSubmitInfo::new(
//                         unsafe { &*semaphore },
//                         ready_value as u64,
//                         PipelineStageBit2::AllTransfer,
//                     )])
//                     .wait_semaphores(&[SemaphoreSubmitInfo::new(
//                         unsafe { &*semaphore },
//                         pending_value as u64,
//                         PipelineStageBit2::AllTransfer,
//                     )])],
//                 None,
//             );

//             mem::swap(&mut ready_pool, &mut pending_pool);
//             mem::swap(&mut ready_value, &mut pending_value);
//         }
//     });
// }

pub struct ContextData {
    instance_fns: vk::InstanceFns,
    device_fns: vk::DeviceFns,
    heap_properties: vk::PhysicalDeviceDescriptorHeapProperties,
    instance: &'static vk::Instance,
    device: &'static vk::Device,
    physical_device: *const vk::PhysicalDevice,
    allocator: DeviceMemoryAllocator,
    // change to fair mutex?
    // transfer: Mutex<Option<TransferContext>>,
    graphics_queue: Queue,
    user_queue_indices: [u32; 1],
    all_queue_indices: [u32; 1],
}

static mut CONTEXT_DATA: MaybeUninit<ContextData> = MaybeUninit::uninit();

#[derive(Clone, Copy)]
pub struct Context(());

impl Deref for Context {
    type Target = ContextData;

    fn deref(&self) -> &Self::Target {
        #[allow(static_mut_refs)]
        unsafe {
            CONTEXT_DATA.assume_init_ref()
        }
    }
}

unsafe fn get_list<T>(
    mut get: impl FnMut(&mut u32, *mut T) -> vk::Result,
) -> Result<Vec<T>, vk::Result> {
    let mut count = 0;
    get(&mut count, ptr::null_mut()).as_result()?;
    let mut items = Vec::with_capacity(count as usize);
    get(&mut count, items.as_mut_ptr()).as_result()?;
    unsafe { items.set_len(count as usize) };
    Ok(items)
}

impl Context {
    pub fn graphics_queue(&self) -> &Queue {
        &self.graphics_queue
    }

    pub fn create_staging_memory_pool(&self) -> StagingMemoryPool {
        StagingMemoryPool {
            buffers: vec![],
            offset: 0,
        }
    }

    pub fn create_copies(&self) -> Copies {
        Copies { stages: vec![] }
    }

    // pub fn create_fence(&self, signaled: bool) -> Owned<Fence> {
    //     unsafe {
    //         let mut fence = ptr::null();
    //         (self.device_fns.create_fence)(
    //             self.device,
    //             &vk::FenceCreateInfo {
    //                 s_type: vk::StructureType::FenceCreateInfo,
    //                 p_next: ptr::null(),
    //                 flags: match signaled {
    //                     true => vk::FenceCreateBit::Signaled.into(),
    //                     false => None.into(),
    //                 },
    //             },
    //             ptr::null(),
    //             &mut fence,
    //         )
    //         .must_succeed();

    //         Owned(NonNull::new(fence.cast_mut()).unwrap())
    //     }
    // }

    pub fn create_timeline_semaphore(&self, initial_value: u64) -> TimelineSemaphore {
        let handle = self
            .create_semaphore_with_next(
                ptr::from_ref(&vk::SemaphoreTypeCreateInfo {
                    s_type: vk::StructureType::SemaphoreTypeCreateInfo,
                    p_next: ptr::null(),
                    semaphore_type: vk::SemaphoreType::Timeline,
                    initial_value,
                })
                .cast(),
            )
            .handle;

        TimelineSemaphore { handle }
    }

    pub fn create_semaphore(&self) -> Semaphore {
        self.create_semaphore_with_next(ptr::null())
    }

    fn create_semaphore_with_next(&self, next: *const ffi::c_void) -> Semaphore {
        unsafe {
            let mut this = ptr::null();
            (self.device_fns.create_semaphore)(
                self.device,
                &vk::SemaphoreCreateInfo {
                    s_type: vk::StructureType::SemaphoreCreateInfo,
                    p_next: next,
                    flags: vk::SemaphoreCreateFlags(0),
                },
                ptr::null(),
                &mut this,
            )
            .must_succeed();

            Semaphore {
                handle: Arc::new(SemaphoreHandle { vk: &*this }),
            }
        }
    }

    pub fn new_buffer<T>(&self, size: usize) -> BufferBuilder<'_, T> {
        assert!(mem::size_of::<T>() > 0 && size > 0);

        BufferBuilder {
            marker: PhantomData,
            cx: self,
            memory_type: MemoryPropertyBit::DeviceLocal.into(),
            size,
            usage: BufferUsageFlags(0),
            queues: None,
        }
    }

    pub fn new_arena<T>(&self) -> ArenaBuilder<'_, T> {
        ArenaBuilder {
            cx: self,
            marker: PhantomData,
            usage: vk::BufferUsageBit::TransferSrc
                | vk::BufferUsageBit::TransferDst
                | vk::BufferUsageBit::ShaderDeviceAddress,
            size: 4,
        }
    }

    pub fn new_image<Fmt>(&self, size: Extent2D) -> Image2DBuilder<'_, Fmt> {
        Image2DBuilder {
            cx: self,
            format: PhantomData,
            usage: ImageUsageFlags(0),
            size,
            concurrent_sharing: false,
        }
    }

    pub fn new_image_array<Fmt>(
        &self,
        size: Extent2D,
        layers: u32,
    ) -> Image2DArrayBuilder<'_, Fmt> {
        Image2DArrayBuilder {
            cx: self,
            format: PhantomData,
            usage: ImageUsageFlags(0),
            size,
            layers,
            mip_levels: 1,
            concurrent_sharing: false,
        }
    }

    pub fn new_atlas<Fmt>(&self) -> AtlasBuilder<'_, Fmt> {
        AtlasBuilder {
            cx: self,
            format: PhantomData,
            usage: vk::ImageUsageBit::TransferSrc | vk::ImageUsageBit::TransferDst,
            size: vk::Extent2D {
                width: 1024,
                height: 1024,
            },
        }
    }

    pub fn create_descriptor_heap(&self) -> DescriptorHeap {
        let arena = self
            .new_arena()
            .capacity(
                (self.heap_properties.min_resource_heap_reserved_range as usize).max(0x1000) * 2,
            )
            .usage(BufferUsageBit::DescriptorHeap)
            .create();

        DescriptorHeap { arena }
    }

    pub fn create_sampler_heap(&self) -> DescriptorHeap {
        let arena = self
            .new_arena()
            .capacity(
                (self.heap_properties.min_sampler_heap_reserved_range as usize).max(0x1000) * 2,
            )
            .usage(BufferUsageBit::DescriptorHeap)
            .create();

        DescriptorHeap { arena }
    }

    pub fn create_shaders(&self, shader_create_infos: &[ShaderCreateInfo]) -> Vec<Shader> {
        thread_local! {
            static CREATE_INFOS: RefCell<(Vec<vk::ShaderCreateInfo>, Vec<vk::ShaderDescriptorSetAndBindingMappingInfo>)> = RefCell::new((Vec::new(), Vec::new()));
        }

        CREATE_INFOS.with_borrow_mut(|(create_infos, mapping_infos)| {
            mapping_infos.clear();
            create_infos.clear();
            create_infos.reserve(shader_create_infos.len());

            for info in shader_create_infos {
                let p_next = match info.input_attachment_mappings {
                    Some(mappings) => ptr::from_mut(mapping_infos.push_mut(
                        vk::ShaderDescriptorSetAndBindingMappingInfo {
                            s_type: vk::StructureType::ShaderDescriptorSetAndBindingMappingInfo,
                            p_next: ptr::null(),
                            mapping_count: mappings.len() as u32,
                            p_mappings: mappings.as_ptr(),
                        },
                    ))
                    .cast(),
                    None => ptr::null(),
                };

                create_infos.push(vk::ShaderCreateInfo {
                    s_type: vk::StructureType::ShaderCreateInfo,
                    p_next,
                    flags: info.flags,
                    stage: info.stage,
                    next_stage: info.next_stage,
                    code_type: vk::ShaderCodeType::Spirv,
                    code_size: info.code.len(),
                    p_code: info.code.as_ptr().cast(),
                    p_name: info.name.as_ptr(),
                    set_layout_count: 0,
                    p_set_layouts: ptr::null(),
                    push_constant_range_count: 0,
                    p_push_constant_ranges: ptr::null(),
                    p_specialization_info: ptr::null(),
                });
            }

            let mut shaders = Vec::<Shader>::with_capacity(shader_create_infos.len());
            unsafe {
                (self.device_fns.create_shaders_ext)(
                    self.device,
                    create_infos.len() as u32,
                    create_infos.as_ptr(),
                    ptr::null(),
                    shaders.as_mut_ptr().cast(),
                )
                .must_succeed();

                shaders.set_len(shader_create_infos.len());
                shaders
            }
        })
    }

    fn create_buffer<T>(
        &self,
        create_info: &vk::BufferCreateInfo,
        memory: MemoryPropertyFlags,
    ) -> Buffer<T> {
        unsafe {
            let mut buffer = ptr::null();
            (self.device_fns.create_buffer)(self.device, create_info, ptr::null(), &mut buffer)
                .must_succeed();

            let mut mem_requirements = MaybeUninit::uninit();
            (self.device_fns.get_buffer_memory_requirements)(
                self.device,
                buffer,
                mem_requirements.as_mut_ptr(),
            );
            let mem_requirements = mem_requirements.assume_init();

            let block = self.allocator.alloc(
                memory,
                mem_requirements.memory_type_bits,
                mem_requirements.size,
                // FIXME: heap alignment
                mem_requirements.alignment.max(32),
            );

            (self.device_fns.bind_buffer_memory)(self.device, buffer, block.memory, block.offset)
                .must_succeed();

            Buffer {
                handle: Arc::new(BufferHandle {
                    vk: &*buffer,
                    block,
                }),
                _owns_t: PhantomData,
            }
        }
    }

    fn create_image<'cx, Fmt: Format>(
        &'cx self,
        usage: ImageUsageFlags,
        tiling: vk::ImageTiling,
        extent: Extent2D,
        queues: Option<&'cx [u32]>,
        array_layers: u32,
        mip_levels: u32,
        extend_create_info: *const ffi::c_void,
        extend_alloc_info: *const ffi::c_void,
    ) -> Arc<ImageHandle> {
        unsafe {
            let mut image = ptr::null();

            (self.device_fns.create_image)(
                self.device,
                &vk::ImageCreateInfo {
                    s_type: vk::StructureType::ImageCreateInfo,
                    p_next: extend_create_info,
                    flags: vk::ImageCreateFlags(0),
                    image_type: vk::ImageType::_2D,
                    format: Fmt::vk(),
                    extent: vk::Extent3D {
                        width: extent.width,
                        height: extent.height,
                        depth: 1,
                    },
                    mip_levels,
                    array_layers,
                    samples: vk::SampleCountBit::_1,
                    tiling,
                    usage,
                    sharing_mode: match queues {
                        Some(_) => vk::SharingMode::Concurrent,
                        None => vk::SharingMode::Exclusive,
                    },
                    queue_family_index_count: queues.map(|q| q.len() as u32).unwrap_or(0),
                    p_queue_family_indices: queues.map(|q| q.as_ptr()).unwrap_or(ptr::null()),
                    initial_layout: vk::ImageLayout::Undefined,
                },
                ptr::null(),
                &mut image,
            )
            .must_succeed();

            let block = self.allocator.alloc_for_image(image, extend_alloc_info);

            Arc::new(ImageHandle { vk: &*image, block })
        }
    }

    //     fn queue_data_to_buffer_transfer<T>(&self, data: &[T], target: BufferRange<'_, T>) -> u64 {
    //         assert_eq!(data.len(), target.len);

    //         let mut transfer = self.transfer.lock().unwrap();
    //         let transfer = transfer.as_mut().unwrap();

    //         unsafe {
    //             let byte_len = data.len() * mem::size_of::<T>();
    //             let dst = transfer.reserve_range(byte_len).cast::<T>();
    //             eprintln!("TO BUFFER {byte_len}");

    //             assert!(dst.is_aligned());

    //             ptr::copy_nonoverlapping(data.as_ptr(), dst, data.len());

    //             transfer
    //                 .sender
    //                 .send(TransferCommand::DataToBuffer {
    //                     buffer: target.buffer.handle.clone(),
    //                     dst_offset: (target.offset * mem::size_of::<T>()) as u32,
    //                     size: byte_len as u32,
    //                 })
    //                 .unwrap();

    //             transfer.sender.send(TransferCommand::Flush).unwrap();

    //             transfer.written_bytes as u64
    //         }
    //     }

    //     fn queue_data_to_image_transfer<Fmt: Format>(
    //         &self,
    //         data: ImageData<'_, Fmt>,
    //         target: ImageRange<'_, Fmt>,
    //         layout: vk::ImageLayout,
    //         transition: bool,
    //     ) -> u64 {
    //         assert_eq!(data.extent, target.extent);
    //         assert_eq!(data.layers, target.layers);

    //         let mut transfer = self.transfer.lock().unwrap();
    //         let transfer = transfer.as_mut().unwrap();

    //         unsafe {
    //             let texel_len = data.extent.width as usize
    //                 * data.extent.height as usize
    //                 * data.extent.depth as usize
    //                 * data.layers as usize;

    //             let byte_len = texel_len * mem::size_of::<Fmt>();

    //             let dst = transfer.reserve_range(byte_len).cast::<Fmt>();
    //             eprintln!("TO IMAGE {byte_len}");

    //             assert!(dst.is_aligned());

    //             ptr::copy_nonoverlapping(data.ptr, dst, texel_len);

    //             if transition {
    //                 transfer
    //                     .sender
    //                     .send(TransferCommand::TransitionImage {
    //                         image: target.image.clone(),
    //                         aspects: Fmt::all_aspects(),
    //                         layout,
    //                     })
    //                     .unwrap();
    //             }

    //             transfer
    //                 .sender
    //                 .send(TransferCommand::DataToImage {
    //                     image: target.image.clone(),
    //                     subresource: vk::ImageSubresourceLayers {
    //                         aspect_mask: target.aspect.into(),
    //                         mip_level: target.mip,
    //                         base_array_layer: target.first_layer,
    //                         layer_count: target.layers,
    //                     },
    //                     layout,
    //                     offset: target.offset,
    //                     extent: target.extent,
    //                     block_size: mem::size_of::<Fmt>() as u32,
    //                 })
    //                 .unwrap();

    //             transfer.sender.send(TransferCommand::Flush).unwrap();

    //             transfer.written_bytes as u64
    //         }
    //     }

    //     fn queue_buffer_transfer<T>(&self, src: BufferRange<'_, T>, dst: BufferRange<'_, T>) {
    //         assert_eq!(src.len, dst.len);

    //         let mut transfer = self.transfer.lock().unwrap();
    //         let transfer = transfer.as_mut().unwrap();

    //         transfer
    //             .sender
    //             .send(TransferCommand::BufferToBuffer {
    //                 src: src.buffer.handle.clone(),
    //                 dst: dst.buffer.handle.clone(),
    //                 size: src.len * mem::size_of::<T>(),
    //             })
    //             .unwrap();
    //     }

    //     fn queue_image_transfer<Fmt: Format>(
    //         &self,
    //         src: ImageRange<'_, Fmt>,
    //         dst: ImageRange<'_, Fmt>,
    //     ) {
    //         assert_eq!(src.extent, dst.extent);
    //         assert_eq!(src.layers, dst.layers);
    //         assert_eq!(src.mip, dst.mip);
    //         assert_eq!(src.mip, 0);
    //         assert_eq!(src.layers, 0);
    //         assert_eq!(src.extent.depth, 0);

    //         let mut transfer = self.transfer.lock().unwrap();
    //         let transfer = transfer.as_mut().unwrap();

    //         transfer
    //             .sender
    //             .send(TransferCommand::ImageToImage {
    //                 src: src.image.clone(),
    //                 dst: dst.image.clone(),
    //                 aspects: Fmt::all_aspects(),
    //                 extent: vk::Extent2D {
    //                     width: src.extent.width,
    //                     height: src.extent.height,
    //                 },
    //             })
    //             .unwrap();
    //     }
}

pub struct WindowBuilder<'app> {
    app: &'app mut App,
    max_queued_frames: usize,
    image_usage: ImageUsageFlags,
    concurrent_sharing: Option<&'app [u32]>,
}

impl<'app> WindowBuilder<'app> {
    pub fn image_usage(mut self, usage: impl Into<ImageUsageFlags>) -> Self {
        self.image_usage = usage.into();
        self
    }

    pub fn image_concurrent_sharing(mut self, queues: &'static [u32]) -> Self {
        self.concurrent_sharing = Some(queues);
        self
    }

    pub fn open<R: Render>(
        self,
        get_renderer: impl FnOnce(&mut App, &mut R::Frame) -> R,
    ) -> Result<()> {
        let key = self.app.windows.vacant_key();
        let platform_window = self
            .app
            .platform
            .open_window(self.app.instance, key, 500, 500)?;

        let render_frame = {
            let mut next_frame_idx = 0;
            let mut frame_data: Vec<_> = (0..self.max_queued_frames)
                .map(|_| R::new_frame(self.app))
                .collect();

            let mut renderer = get_renderer(self.app, &mut frame_data[0]);

            move |app: &mut App, window: &mut Window| -> Result<(), vk::Result> {
                let frame_idx = next_frame_idx % self.max_queued_frames;
                let frame = &mut frame_data[frame_idx];

                renderer.render(app, window, frame);
                next_frame_idx += 1;

                Ok(())
            }
        };

        self.app.windows.insert(Some(WindowData {
            window: Window {
                platform: platform_window,
                width: 1000,
                height: 1000,
                scale_factor: ScaleFactor::ONE,
                mouse_position: None,
                next_image_idx: 0,
                images: Slab::new(),
                // image_views: vec![],
                // wait_semaphores: vec![],
                // signal_semaphores: vec![],
            },
            render_frame: Box::new(render_frame),
        }));

        // self.app.windows[key]
        //     .as_ref()
        //     .unwrap()
        //     .window
        //     .platform
        //     .show();

        Ok(())
    }
}

impl Semaphore {
    pub fn submit_info(&self, stage_mask: impl Into<PipelineStageFlags>) -> SemaphoreSubmitInfo {
        SemaphoreSubmitInfo::new(&self.handle, 0, stage_mask)
    }
}

impl TimelineSemaphore {
    pub fn submit_info(
        &self,
        stage_mask: impl Into<PipelineStageFlags>,
        value: u64,
    ) -> SemaphoreSubmitInfo {
        SemaphoreSubmitInfo::new(&self.handle, value, stage_mask)
    }
}

impl SemaphoreSubmitInfo {
    fn new(
        semaphore: &SemaphoreHandle,
        value: u64,
        stage_mask: impl Into<PipelineStageFlags>,
    ) -> Self {
        Self {
            s_type: vk::StructureType::SemaphoreSubmitInfo,
            p_next: ptr::null(),
            semaphore: semaphore.vk,
            value,
            stage_mask: stage_mask.into(),
            device_index: 0,
        }
    }
}

impl CommandBufferSubmitInfo {
    pub fn new(command_buffer: &CommandBuffer) -> Self {
        Self {
            s_type: vk::StructureType::CommandBufferSubmitInfo,
            p_next: ptr::null(),
            command_buffer,
            device_mask: 0,
        }
    }
}

impl SubmitInfo {
    pub fn new(buffers: &[CommandBufferSubmitInfo]) -> Self {
        Self {
            s_type: vk::StructureType::SubmitInfo2,
            p_next: ptr::null(),
            flags: vk::SubmitFlags(0),
            wait_semaphore_info_count: 0,
            p_wait_semaphore_infos: ptr::null(),
            command_buffer_info_count: buffers.len() as u32,
            p_command_buffer_infos: buffers.as_ptr().cast(),
            signal_semaphore_info_count: 0,
            p_signal_semaphore_infos: ptr::null(),
        }
    }

    pub fn wait_semaphores(mut self, wait_semaphores: &[SemaphoreSubmitInfo]) -> Self {
        self.wait_semaphore_info_count = wait_semaphores.len() as u32;
        self.p_wait_semaphore_infos = wait_semaphores.as_ptr();
        self
    }

    pub fn signal_semaphores(mut self, signal_semaphores: &[SemaphoreSubmitInfo]) -> Self {
        self.signal_semaphore_info_count = signal_semaphores.len() as u32;
        self.p_signal_semaphore_infos = signal_semaphores.as_ptr();
        self
    }
}

pub struct ShaderCreateInfo<'a> {
    code_type: vk::ShaderCodeType,
    stage: ShaderStageBit,
    next_stage: ShaderStageFlags,
    flags: ShaderCreateFlags,
    code: &'a [u8],
    name: &'a CStr,
    input_attachment_mappings: Option<&'a [InputAttachmentMapping]>,
}

impl<'a> ShaderCreateInfo<'a> {
    pub fn new_spirv(stage: ShaderStageBit, code: &'a [u8], name: &'a CStr) -> Self {
        Self {
            code_type: vk::ShaderCodeType::Spirv,
            stage,
            next_stage: ShaderStageFlags(0),
            flags: ShaderCreateFlags(0),
            code,
            name,
            input_attachment_mappings: None,
        }
    }

    pub fn input_attachments(mut self, mappings: &'a [InputAttachmentMapping]) -> Self {
        self.input_attachment_mappings = Some(mappings);
        self
    }

    pub fn flags(mut self, flags: impl Into<ShaderCreateFlags>) -> Self {
        self.flags = flags.into() | ShaderCreateBit::DescriptorHeap;
        self
    }

    pub fn next_stage(mut self, stage: impl Into<ShaderStageFlags>) -> Self {
        self.next_stage = stage.into();
        self
    }
}

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct InputAttachmentMapping {
    s_type: vk::StructureType,
    p_next: *const ffi::c_void,
    descriptor_set: u32,
    first_binding: u32,
    binding_count: u32,
    resource_mask: vk::SpirvResourceTypeFlags,
    source: vk::DescriptorMappingSource,
    source_data: vk::DescriptorMappingSourceData,
}

impl InputAttachmentMapping {
    pub fn new(binding_range: Range<u32>, push_offset: usize) -> Self {
        let cx = Context(());

        vk::DescriptorSetAndBindingMapping {
            s_type: vk::StructureType::DescriptorSetAndBindingMapping,
            p_next: ptr::null(),
            descriptor_set: 0,
            first_binding: binding_range.start,
            binding_count: binding_range.end - binding_range.start,
            resource_mask: vk::SpirvResourceTypeFlags(
                vk::SpirvResourceTypeBit::ReadWriteImage as u32,
            ),
            source: vk::DescriptorMappingSource::HeapWithPushIndex,
            source_data: vk::DescriptorMappingSourceData {
                push_index: vk::DescriptorMappingSourcePushIndex {
                    heap_offset: 0,
                    push_offset: push_offset as u32,
                    heap_index_stride: cx.heap_properties.image_descriptor_size as u32,
                    heap_array_stride: 0,
                    p_embedded_sampler: ptr::null(),
                    use_combined_image_sampler_index: 0,
                    sampler_heap_offset: 0,
                    sampler_push_offset: 0,
                    sampler_heap_index_stride: 0,
                    sampler_heap_array_stride: 0,
                },
            },
        }
    }
}

// pub struct InputAttachmentMapping {
//     first_binding: u32,
//     binding_count: u32,
//     push_offset: u32,
//     heap_index_stride: u32,
// }

// impl Fence {
//     pub fn wait(&self) -> Result<(), vk::Result> {
//         Self::wait_all(&[self])
//     }

//     pub fn wait_all(fences: &[impl Handle<Self>]) -> Result<(), vk::Result> {
//         let cx = Context(());

//         let fences = Handle::as_slice_of_ptrs(fences);
//         unsafe {
//             (cx.device_fns.wait_for_fences)(cx.device, fences.len() as u32, fences.as_ptr(), 1, !0)
//                 .as_result()
//         }
//     }

//     pub fn reset(&self) -> Result<(), vk::Result> {
//         Self::reset_all(&[self])
//     }

//     pub fn reset_all(fences: &[impl Handle<Self>]) -> Result<(), vk::Result> {
//         let cx = Context(());

//         let fences = Handle::as_slice_of_ptrs(fences);
//         unsafe {
//             (cx.device_fns.reset_fences)(cx.device, fences.len() as u32, fences.as_ptr())
//                 .as_result()
//         }
//     }
// }

impl<Fmt: Format> Image2D<Fmt> {
    pub fn view(&self) -> ImageView<Fmt> {
        let cx = Context(());

        unsafe {
            let mut img_view = ptr::null();
            (cx.device_fns.create_image_view)(
                cx.device,
                &vk::ImageViewCreateInfo {
                    s_type: vk::StructureType::ImageViewCreateInfo,
                    p_next: ptr::null(),
                    flags: vk::ImageViewCreateFlags(0),
                    image: self.inner.vk,
                    view_type: vk::ImageViewType::_2D,
                    format: Fmt::vk(),
                    components: vk::ComponentMapping {
                        r: vk::ComponentSwizzle::Identity,
                        g: vk::ComponentSwizzle::Identity,
                        b: vk::ComponentSwizzle::Identity,
                        a: vk::ComponentSwizzle::Identity,
                    },
                    subresource_range: vk::ImageSubresourceRange {
                        aspect_mask: Fmt::all_aspects(),
                        base_mip_level: 0,
                        level_count: 1,
                        base_array_layer: 0,
                        layer_count: 1,
                    },
                },
                ptr::null(),
                &mut img_view,
            )
            .must_succeed();

            ImageView {
                _format: PhantomData,
                handle: Arc::new(ImageViewHandle {
                    vk: &*img_view,
                    image: self.inner.clone(),
                }),
            }
        }
    }

    pub fn barrier(&self, subresource_range: impl Into<ImageAspectFlags>) -> ImageMemoryBarrier {
        ImageMemoryBarrier {
            s_type: vk::StructureType::ImageMemoryBarrier2,
            p_next: ptr::null(),
            src_stage_mask: PipelineStageFlags(0),
            src_access_mask: AccessFlags(0),
            dst_stage_mask: PipelineStageFlags(0),
            dst_access_mask: AccessFlags(0),
            old_layout: ImageLayout::Undefined,
            new_layout: ImageLayout::Undefined,
            src_queue_family_index: !0,
            dst_queue_family_index: !0,
            image: self.inner.vk,
            subresource_range: vk::ImageSubresourceRange {
                aspect_mask: subresource_range.into(),
                base_mip_level: 0,
                level_count: 1,
                base_array_layer: 0,
                layer_count: 1,
            },
        }
    }

    pub fn range(
        &self,
        x_range: impl RangeBounds<u32>,
        y_range: impl RangeBounds<u32>,
    ) -> ImageRange<'_, Fmt> {
        let x_start = match x_range.start_bound() {
            ops::Bound::Included(&start) => start,
            ops::Bound::Excluded(&start) => start + 1,
            ops::Bound::Unbounded => 0,
        };

        let x_end = match x_range.end_bound() {
            ops::Bound::Included(&end) => end + 1,
            ops::Bound::Excluded(&end) => end,
            ops::Bound::Unbounded => self.extent.width,
        };

        assert!(x_start <= x_end && x_end <= self.extent.width);

        let y_start = match y_range.start_bound() {
            ops::Bound::Included(&start) => start,
            ops::Bound::Excluded(&start) => start + 1,
            ops::Bound::Unbounded => 0,
        };

        let y_end = match y_range.end_bound() {
            ops::Bound::Included(&end) => end + 1,
            ops::Bound::Excluded(&end) => end,
            ops::Bound::Unbounded => self.extent.height,
        };

        assert!(y_start <= y_end && y_end <= self.extent.height);

        ImageRange {
            _format: PhantomData,
            image: &self.inner,
            offset: vk::Offset3D {
                x: x_start as i32,
                y: y_start as i32,
                z: 0,
            },
            first_layer: 0,
            extent: vk::Extent3D {
                width: x_end - x_start,
                height: y_end - y_start,
                depth: 1,
            },
            layers: 1,
            aspect: Fmt::all_aspects(),
            mip: 0,
        }
    }
}

impl ImageMemoryBarrier {
    pub fn old_layout(mut self, layout: ImageLayout) -> Self {
        self.old_layout = layout;
        self
    }

    pub fn new_layout(mut self, layout: ImageLayout) -> Self {
        self.new_layout = layout;
        self
    }

    pub fn src_access(
        mut self,
        stage: impl Into<PipelineStageFlags>,
        access: impl Into<AccessFlags>,
    ) -> Self {
        self.src_access_mask = access.into();
        self.src_stage_mask = stage.into();
        self
    }

    pub fn dst_access(
        mut self,
        stage: impl Into<PipelineStageFlags>,
        access: impl Into<AccessFlags>,
    ) -> Self {
        self.dst_access_mask = access.into();
        self.dst_stage_mask = stage.into();
        self
    }

    pub fn src_queue(mut self, queue: u32) -> Self {
        self.src_queue_family_index = queue;
        self
    }

    pub fn dst_queue(mut self, queue: u32) -> Self {
        self.dst_queue_family_index = queue;
        self
    }

    pub fn mip_range(mut self, range: Range<u32>) -> Self {
        self.subresource_range.base_mip_level = range.start;
        self.subresource_range.level_count = range.end - range.start;
        self
    }

    pub fn layer_range(mut self, range: Range<u32>) -> Self {
        self.subresource_range.base_array_layer = range.start;
        self.subresource_range.layer_count = range.end - range.start;
        self
    }
}

pub struct RenderingInfo<'a> {
    render_area: Rect2D,
    layer_count: u32,
    view_mask: u32,
    color_attachments: &'a [RenderingAttachmentInfo],
    depth_attachment: Option<&'a RenderingAttachmentInfo>,
    stencil_attachment: Option<&'a RenderingAttachmentInfo>,
}

impl<'a> RenderingInfo<'a> {
    pub fn new(render_area: Rect2D) -> Self {
        Self {
            render_area,
            layer_count: 0,
            view_mask: 0,
            color_attachments: &[],
            depth_attachment: None,
            stencil_attachment: None,
        }
    }

    pub fn layer_count(mut self, layer_count: u32) -> Self {
        self.layer_count = layer_count;
        self
    }

    pub fn color_attachments(mut self, infos: &'a [RenderingAttachmentInfo]) -> Self {
        self.color_attachments = infos;
        self
    }
}

pub struct RenderingAttachmentInfo {
    image_view: Arc<ImageViewHandle>,
    image_layout: ImageLayout,
    resolve_mode: ResolveModeFlags,
    resolve_image_view: Option<Arc<ImageViewHandle>>,
    resolve_image_layout: ImageLayout,
    load_op: AttachmentLoadOp,
    store_op: AttachmentStoreOp,
    clear_value: vk::ClearValue,
}

impl RenderingAttachmentInfo {
    pub fn new_cleared<Fmt: Format>(
        image_view: &ImageView<Fmt>,
        image_layout: ImageLayout,
        clear_value: Fmt::ClearValue,
    ) -> Self {
        Self {
            image_view: image_view.handle.clone(),
            image_layout,
            resolve_mode: ResolveModeFlags(0),
            resolve_image_view: None,
            resolve_image_layout: ImageLayout::Undefined,
            load_op: AttachmentLoadOp::Clear,
            store_op: AttachmentStoreOp::None,
            clear_value: clear_value.into(),
        }
    }

    pub fn new<Fmt>(image_view: &ImageView<Fmt>, image_layout: ImageLayout) -> Self {
        Self {
            image_view: image_view.handle.clone(),
            image_layout,
            resolve_mode: ResolveModeFlags(0),
            resolve_image_view: None,
            resolve_image_layout: ImageLayout::Undefined,
            load_op: AttachmentLoadOp::None,
            store_op: AttachmentStoreOp::None,
            clear_value: vk::ClearValue {
                color: vk::ClearColorValue { uint_32: [0; 4] },
            },
        }
    }

    pub fn load_op(mut self, load_op: AttachmentLoadOp) -> Self {
        self.load_op = load_op;
        self
    }

    pub fn store_op(mut self, store_op: AttachmentStoreOp) -> Self {
        self.store_op = store_op;
        self
    }
}

pub struct CommandPool {
    vk: &'static vk::CommandPool,
    buffers: Vec<&'static CommandBuffer>,
    next_empty_buffer: Cell<usize>,
}

impl Clone for CommandPool {
    fn clone(&self) -> Self {
        panic!()
        // Self {
        //     vk: self.vk.clone(),
        //     buffers: self.buffers.clone(),
        //     next_empty_buffer: self.next_empty_buffer.clone(),
        // }
    }
}

impl CommandPool {
    pub fn allocate_command_buffers(&mut self, n: usize) {
        let cx = Context(());

        // TODO: handle partial success
        self.buffers.reserve(n);
        unsafe {
            (cx.device_fns.allocate_command_buffers)(
                cx.device,
                &vk::CommandBufferAllocateInfo {
                    s_type: vk::StructureType::CommandBufferAllocateInfo,
                    p_next: ptr::null(),
                    command_pool: self.vk,
                    level: vk::CommandBufferLevel::Primary,
                    command_buffer_count: n as u32,
                },
                self.buffers.spare_capacity_mut().as_mut_ptr().cast(),
            )
            .must_succeed();

            self.buffers.set_len(self.buffers.len() + n);
        }
    }

    pub fn get_buffer(&self) -> &CommandBuffer {
        if self.next_empty_buffer.get() == self.buffers.len() {
            panic!("more command buffers requested than allocated");
        }

        let buffer = self.buffers[self.next_empty_buffer.get()];
        self.next_empty_buffer.set(self.next_empty_buffer.get() + 1);
        buffer
    }

    pub fn reset(&mut self) {
        let cx = Context(());

        self.next_empty_buffer.set(0);

        unsafe {
            (cx.device_fns.reset_command_pool)(cx.device, self.vk, vk::CommandPoolResetFlags(0))
                .must_succeed()
        }
    }
}

impl CommandBuffer {
    pub fn submit_info(&self) -> CommandBufferSubmitInfo {
        CommandBufferSubmitInfo::new(self)
    }

    pub fn begin(&self) {
        let cx = Context(());
        unsafe {
            (cx.device_fns.begin_command_buffer)(
                self,
                &vk::CommandBufferBeginInfo {
                    s_type: vk::StructureType::CommandBufferBeginInfo,
                    p_next: ptr::null(),
                    flags: vk::CommandBufferUsageBit::OneTimeSubmit.into(),
                    p_inheritance_info: ptr::null(),
                },
            )
            .must_succeed()
        }
    }

    pub fn end(&self) {
        let cx = Context(());
        unsafe { (cx.device_fns.end_command_buffer)(self).must_succeed() }
    }

    pub fn barriers(
        &self,
        flags: impl Into<DependencyFlags>,
        memory: &[MemoryBarrier],
        buffer: &[()],
        image: &[ImageMemoryBarrier],
    ) -> &Self {
        let cx = Context(());

        unsafe {
            (cx.device_fns.cmd_pipeline_barrier_2)(
                self,
                &vk::DependencyInfo {
                    s_type: vk::StructureType::DependencyInfo,
                    p_next: ptr::null(),
                    dependency_flags: flags.into(),
                    memory_barrier_count: memory.len() as u32,
                    p_memory_barriers: memory.as_ptr(),
                    buffer_memory_barrier_count: 0,
                    p_buffer_memory_barriers: ptr::null(),
                    image_memory_barrier_count: image.len() as u32,
                    p_image_memory_barriers: image.as_ptr(),
                },
            );
        }

        self
    }

    pub fn bind_resource_heap(&self, heap: &DescriptorHeap) {
        let cx = Context(());

        unsafe {
            (cx.device_fns.cmd_bind_resource_heap_ext)(
                self,
                &vk::BindHeapInfo {
                    s_type: vk::StructureType::BindHeapInfo,
                    p_next: ptr::null(),
                    heap_range: vk::DeviceAddressRange {
                        address: heap.arena.inner.device_address(),
                        size: heap.arena.inner.len() as u64,
                    },
                    reserved_range_offset: heap.arena.len as u64
                        - cx.heap_properties.min_resource_heap_reserved_range as u64,
                    reserved_range_size: cx.heap_properties.min_resource_heap_reserved_range,
                },
            )
        }
    }

    pub fn bind_sampler_heap(&self, heap: &DescriptorHeap) {
        let cx = Context(());

        unsafe {
            (cx.device_fns.cmd_bind_sampler_heap_ext)(
                self,
                &vk::BindHeapInfo {
                    s_type: vk::StructureType::BindHeapInfo,
                    p_next: ptr::null(),
                    heap_range: vk::DeviceAddressRange {
                        address: heap.arena.inner.device_address(),
                        size: heap.arena.inner.len() as u64,
                    },
                    reserved_range_offset: heap.arena.len as u64
                        - cx.heap_properties.min_sampler_heap_reserved_range as u64,
                    reserved_range_size: cx.heap_properties.min_sampler_heap_reserved_range,
                },
            )
        }
    }

    pub fn begin_rendering(&self, info: RenderingInfo) -> &Self {
        thread_local! {
            static ATTACHMENTS: RefCell<Vec<vk::RenderingAttachmentInfo>> = RefCell::new(Vec::new());
        }

        ATTACHMENTS.with_borrow_mut(|attachments| {
            attachments.clear();
            attachments.reserve(info.color_attachments.len());

            let to_vk = |attachment: &RenderingAttachmentInfo| vk::RenderingAttachmentInfo {
                s_type: vk::StructureType::RenderingAttachmentInfo,
                p_next: ptr::null(),
                image_view: attachment.image_view.vk,
                image_layout: attachment.image_layout,
                resolve_mode: attachment.resolve_mode,
                resolve_image_view: attachment
                    .resolve_image_view
                    .as_ref()
                    .map_or(ptr::null(), |view| view.vk),
                resolve_image_layout: attachment.resolve_image_layout,
                load_op: attachment.load_op,
                store_op: attachment.store_op,
                clear_value: attachment.clear_value,
            };

            for attachment in info.color_attachments {
                attachments.push(to_vk(attachment));
            }

            let depth_attachment = if let Some(attachment) = info.depth_attachment {
                ptr::from_ref(attachments.push_mut(to_vk(attachment)))
            } else {
                ptr::null()
            };

            let stencil_attachment = if let Some(attachment) = info.stencil_attachment {
                ptr::from_ref(attachments.push_mut(to_vk(attachment)))
            } else {
                ptr::null()
            };

            let cx = Context(());

            unsafe {
                (cx.device_fns.cmd_begin_rendering)(
                    self,
                    &vk::RenderingInfo {
                        s_type: vk::StructureType::RenderingInfo,
                        p_next: ptr::null(),
                        flags: vk::RenderingFlags(0),
                        render_area: info.render_area,
                        layer_count: info.layer_count,
                        view_mask: info.view_mask,
                        color_attachment_count: info.color_attachments.len() as u32,
                        p_color_attachments: attachments.as_ptr(),
                        p_depth_attachment: depth_attachment,
                        p_stencil_attachment: stencil_attachment,
                    },
                );
            }
        });

        self
    }

    pub fn bind_vertex_fragment_shaders(&self, vertex: Shader, fragment: Shader) -> &Self {
        self.bind_shaders(
            &[
                // ShaderStageBit::Task,
                // ShaderStageBit::Mesh,
                ShaderStageBit::Vertex,
                ShaderStageBit::Fragment,
            ],
            &[
                // ptr::null(), ptr::null(),
                vertex.0, fragment.0,
            ],
        )
    }

    fn bind_shaders(
        &self,
        shader_stages: &[ShaderStageBit],
        shaders: &[*const vk::Shader],
    ) -> &Self {
        let cx = Context(());

        unsafe {
            (cx.device_fns.cmd_bind_shaders_ext)(
                self,
                shader_stages.len() as u32,
                shader_stages.as_ptr(),
                shaders.as_ptr().cast(),
            );
        }
        self
    }

    pub fn set_rasterizer_discard_enable(&self, enable: bool) -> &Self {
        let cx = Context(());

        unsafe {
            (cx.device_fns.cmd_set_rasterizer_discard_enable)(self, enable as u32);
        }
        self
    }

    pub fn set_cull_mode(&self, cull_mode: impl Into<CullModeFlags>) -> &Self {
        let cx = Context(());

        unsafe {
            (cx.device_fns.cmd_set_cull_mode)(self, cull_mode.into());
        }
        self
    }

    pub fn set_depth_test_enable(&self, enable: bool) -> &Self {
        let cx = Context(());

        unsafe {
            (cx.device_fns.cmd_set_depth_test_enable)(self, enable as u32);
        }
        self
    }

    pub fn set_stencil_test_enable(&self, enable: bool) -> &Self {
        let cx = Context(());

        unsafe {
            (cx.device_fns.cmd_set_stencil_test_enable)(self, enable as u32);
        }
        self
    }

    pub fn set_depth_bias_enable(&self, enable: bool) -> &Self {
        let cx = Context(());

        unsafe {
            (cx.device_fns.cmd_set_depth_bias_enable)(self, enable as u32);
        }
        self
    }

    pub fn set_polygon_mode(&self, polygon_mode: PolygonMode) -> &Self {
        let cx = Context(());

        unsafe {
            (cx.device_fns.cmd_set_polygon_mode_ext)(self, polygon_mode);
        }
        self
    }

    pub fn set_line_width(&self, width: f32) -> &Self {
        let cx = Context(());

        unsafe {
            (cx.device_fns.cmd_set_line_width)(self, width);
        }
        self
    }

    pub fn set_rasterization_samples(&self, sample_count: SampleCountBit) -> &Self {
        let cx = Context(());

        unsafe {
            (cx.device_fns.cmd_set_rasterization_samples_ext)(self, sample_count);
        }
        self
    }

    pub fn set_sample_mask(&self, sample_count: SampleCountBit, sample_mask: &[u32]) -> &Self {
        let cx = Context(());

        unsafe {
            (cx.device_fns.cmd_set_sample_mask_ext)(self, sample_count, sample_mask.as_ptr());
        }
        self
    }

    pub fn set_alpha_to_coverage_enable(&self, enable: bool) -> &Self {
        let cx = Context(());

        unsafe {
            (cx.device_fns.cmd_set_alpha_to_coverage_enable_ext)(self, enable as u32);
        }
        self
    }

    pub fn set_color_blend_enable(&self, first_attachment: u32, enable: &[u32]) -> &Self {
        let cx = Context(());

        unsafe {
            (cx.device_fns.cmd_set_color_blend_enable_ext)(
                self,
                first_attachment,
                enable.len() as u32,
                enable.as_ptr(),
            );
        }
        self
    }

    pub fn set_color_blend_equation(
        &self,
        first_attachment: u32,
        equation: &[ColorBlendEquation],
    ) -> &Self {
        let cx = Context(());
        unsafe {
            (cx.device_fns.cmd_set_color_blend_equation_ext)(
                self,
                first_attachment,
                equation.len() as u32,
                equation.as_ptr(),
            );
        }
        self
    }

    pub fn set_color_write_mask(
        &self,
        first_attachment: u32,
        color_component_flags: &[ColorComponentFlags],
    ) -> &Self {
        let cx = Context(());
        unsafe {
            (cx.device_fns.cmd_set_color_write_mask_ext)(
                self,
                first_attachment,
                color_component_flags.len() as u32,
                color_component_flags.as_ptr(),
            );
        }
        self
    }

    pub fn set_viewport_with_count(&self, viewports: &[Viewport]) -> &Self {
        let cx = Context(());

        unsafe {
            (cx.device_fns.cmd_set_viewport_with_count)(
                self,
                viewports.len() as u32,
                viewports.as_ptr(),
            );
        }
        self
    }

    pub fn set_scissor_with_count(&self, scissors: &[Rect2D]) -> &Self {
        let cx = Context(());

        unsafe {
            (cx.device_fns.cmd_set_scissor_with_count)(
                self,
                scissors.len() as u32,
                scissors.as_ptr(),
            );
        }
        self
    }

    pub fn set_primitive_topology(&self, primitive_topology: PrimitiveTopology) -> &Self {
        let cx = Context(());

        unsafe {
            (cx.device_fns.cmd_set_primitive_topology)(self, primitive_topology);
        }
        self
    }

    pub fn set_primitive_restart_enable(&self, enable: bool) -> &Self {
        let cx = Context(());

        unsafe {
            (cx.device_fns.cmd_set_primitive_restart_enable)(self, enable as u32);
        }
        self
    }

    pub fn set_vertex_input(&self) -> &Self {
        let cx = Context(());

        unsafe {
            (cx.device_fns.cmd_set_vertex_input_ext)(self, 0, ptr::null(), 0, ptr::null());
        }
        self
    }

    pub fn draw(&self, vertex_range: Range<u32>, instance_range: Range<u32>) -> &Self {
        let cx = Context(());

        unsafe {
            (cx.device_fns.cmd_draw)(
                self,
                vertex_range.len() as u32,
                instance_range.len() as u32,
                vertex_range.start,
                instance_range.start,
            );
        }
        self
    }

    pub fn draw_indirect<T>(&self, range: BufferRange<'_, T>) -> &Self {
        let cx = Context(());

        unsafe {
            (cx.device_fns.cmd_draw_indirect)(
                self,
                range.buffer.handle.vk,
                (mem::size_of::<T>() * range.offset) as u64,
                range.len as u32,
                mem::size_of::<T>() as u32,
            );
        }
        self
    }

    // pub fn draw_mesh_tasks(&self, x: u32, y: u32, z: u32) -> &Self {
    //     let cx = Context(());

    //     unsafe {
    //         (cx.device_fns.cmd_draw_mesh_tasks_ext)(self, x, y, z);
    //     }
    //     self
    // }

    // pub fn draw_mesh_tasks_indirect<T>(&self, range: BufferRange<'_, T>) -> &Self {
    //     let cx = Context(());

    //     unsafe {
    //         (cx.device_fns.cmd_draw_mesh_tasks_indirect_ext)(
    //             self,
    //             range.buffer.handle.vk,
    //             (mem::size_of::<T>() * range.offset) as u64,
    //             range.len as u32,
    //             mem::size_of::<T>() as u32,
    //         );
    //     }
    //     self
    // }

    pub fn end_rendering(&self) -> &Self {
        let cx = Context(());

        unsafe {
            (cx.device_fns.cmd_end_rendering)(self);
        }
        self
    }

    pub fn push_data<T: 'static>(&self, data: &T) -> &Self {
        let cx = Context(());

        unsafe {
            (cx.device_fns.cmd_push_data_ext)(
                self,
                &vk::PushDataInfo {
                    s_type: vk::StructureType::PushDataInfo,
                    p_next: ptr::null(),
                    offset: 0,
                    data: vk::HostAddressRangeConst {
                        address: ptr::from_ref(data).cast(),
                        size: mem::size_of::<T>(),
                    },
                },
            );
        }

        self
    }

    pub fn copy_buffer<T>(&self, src: &Buffer<T>, dst: &Buffer<T>, ranges: &[BufferCopy]) -> &Self {
        let cx = Context(());

        unsafe {
            (cx.device_fns.cmd_copy_buffer_2)(
                self,
                &vk::CopyBufferInfo2 {
                    s_type: vk::StructureType::CopyBufferInfo2,
                    p_next: ptr::null(),
                    src_buffer: src.handle.vk,
                    dst_buffer: dst.handle.vk,
                    region_count: ranges.len() as u32,
                    p_regions: ranges.as_ptr(),
                },
            );
        }

        self
    }

    pub fn copy_image<Fmt>(
        &self,
        src: &Image2D<Fmt>,
        src_image_layout: vk::ImageLayout,
        dst: &Image2D<Fmt>,
        dst_image_layout: vk::ImageLayout,
        ranges: &[ImageCopy],
    ) -> &Self {
        let cx = Context(());

        unsafe {
            (cx.device_fns.cmd_copy_image_2)(
                self,
                &vk::CopyImageInfo2 {
                    s_type: vk::StructureType::CopyImageInfo2,
                    p_next: ptr::null(),
                    src_image: src.inner.vk,
                    dst_image: dst.inner.vk,
                    src_image_layout,
                    dst_image_layout,
                    region_count: ranges.len() as u32,
                    p_regions: ranges.as_ptr(),
                },
            );
        }

        self
    }

    pub fn copy_buffer_to_image<Fmt>(
        &self,
        src: &Buffer<Fmt>,
        dst: &impl Image<Fmt>,
        dst_image_layout: vk::ImageLayout,
        ranges: &[BufferImageCopy],
    ) -> &Self {
        let cx = Context(());

        unsafe {
            (cx.device_fns.cmd_copy_buffer_to_image_2)(
                self,
                &vk::CopyBufferToImageInfo2 {
                    s_type: vk::StructureType::CopyBufferToImageInfo2,
                    p_next: ptr::null(),
                    src_buffer: src.handle.vk,
                    dst_image: dst.raw_handle().vk,
                    dst_image_layout: dst_image_layout,
                    region_count: ranges.len() as u32,
                    p_regions: ranges.as_ptr(),
                },
            );
        }

        self
    }

    pub fn blit_image<Fmt>(
        &self,
        src: &impl Image<Fmt>,
        src_image_layout: vk::ImageLayout,
        dst: &impl Image<Fmt>,
        dst_image_layout: vk::ImageLayout,
        filter: Filter,
        regions: &[ImageBlit],
    ) -> &Self {
        let cx = Context(());

        unsafe {
            (cx.device_fns.cmd_blit_image_2)(
                self,
                &vk::BlitImageInfo2 {
                    s_type: vk::StructureType::BlitImageInfo2,
                    p_next: ptr::null(),
                    src_image: src.raw_handle().vk,
                    src_image_layout,
                    dst_image: dst.raw_handle().vk,
                    dst_image_layout,
                    region_count: regions.len() as u32,
                    p_regions: regions.as_ptr(),
                    filter,
                },
            );
        }

        self
    }
}
