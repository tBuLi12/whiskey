use core::ffi;
use std::{
    fs::{self, File},
    mem::{self, MaybeUninit},
    process, ptr,
    sync::OnceLock,
    thread,
    time::{Duration, Instant},
};

use crate::{
    App, AsResult, Context, ImageUsageBit, PipelineStageBit2, SemaphoreSubmitInfo, SubmitInfo2,
    vk::{self, SemaphoreTypeCreateInfo},
};

use anyhow::{Result, bail};
use windows::{
    Win32::{
        Foundation::{GENERIC_ALL, HANDLE, HMODULE, HWND, LPARAM, LRESULT, RECT, WPARAM},
        Graphics::{
            Direct3D::D3D_FEATURE_LEVEL_12_0,
            Direct3D12::{
                D3D12_COMMAND_LIST_FLAGS, D3D12_COMMAND_LIST_TYPE_DIRECT, D3D12_COMMAND_QUEUE_DESC,
                D3D12_COMMAND_QUEUE_FLAG_NONE, D3D12_CPU_DESCRIPTOR_HANDLE,
                D3D12_DESCRIPTOR_HEAP_DESC, D3D12_DESCRIPTOR_HEAP_TYPE_RTV,
                D3D12_FENCE_FLAG_SHARED, D3D12_FENCE_FLAGS, D3D12_MESSAGE, D3D12_RESOURCE_BARRIER,
                D3D12_RESOURCE_BARRIER_0, D3D12_RESOURCE_BARRIER_ALL_SUBRESOURCES,
                D3D12_RESOURCE_BARRIER_FLAG_NONE, D3D12_RESOURCE_BARRIER_TYPE_TRANSITION,
                D3D12_RESOURCE_STATE_COMMON, D3D12_RESOURCE_STATE_PRESENT,
                D3D12_RESOURCE_STATE_RENDER_TARGET, D3D12_RESOURCE_STATES,
                D3D12_RESOURCE_TRANSITION_BARRIER, D3D12CreateDevice, D3D12GetDebugInterface,
                ID3D12CommandAllocator, ID3D12CommandQueue, ID3D12Debug, ID3D12DescriptorHeap,
                ID3D12Device12, ID3D12Fence, ID3D12GraphicsCommandList,
                ID3D12GraphicsCommandList10, ID3D12InfoQueue, ID3D12Resource,
            },
            Dxgi::{
                Common::{DXGI_ALPHA_MODE_IGNORE, DXGI_FORMAT_B8G8R8A8_UNORM, DXGI_SAMPLE_DESC},
                CreateDXGIFactory2, DXGI_CREATE_FACTORY_DEBUG, DXGI_CREATE_FACTORY_FLAGS,
                DXGI_PRESENT, DXGI_SCALING_NONE, DXGI_SWAP_CHAIN_DESC1,
                DXGI_SWAP_CHAIN_FLAG_FRAME_LATENCY_WAITABLE_OBJECT, DXGI_SWAP_EFFECT_FLIP_DISCARD,
                DXGI_SWAP_EFFECT_FLIP_SEQUENTIAL, DXGI_USAGE_BACK_BUFFER,
                DXGI_USAGE_RENDER_TARGET_OUTPUT, DXGI_USAGE_UNORDERED_ACCESS, IDXGIAdapter4,
                IDXGIFactory7, IDXGISwapChain4,
            },
            Gdi::HBRUSH,
        },
        System::{
            LibraryLoader::{GetModuleHandleW, GetProcAddress, LoadLibraryW},
            Threading::{INFINITE, WaitForSingleObjectEx},
        },
        UI::WindowsAndMessaging::{
            AdjustWindowRectEx, CS_OWNDC, CreateWindowExW, DefWindowProcW, DispatchMessageW,
            GET_CLASS_LONG_INDEX, GWLP_USERDATA, GetClassLongPtrW, GetMessageW, GetWindowLongPtrW,
            IDC_ARROW, IDI_APPLICATION, LoadCursorW, LoadIconW, MSG, RegisterClassExW, SW_SHOW,
            SetClassLongPtrW, SetForegroundWindow, SetWindowLongPtrW, ShowWindow, TranslateMessage,
            WM_ERASEBKGND, WM_PAINT, WM_SIZE, WNDCLASSEXW, WS_CLIPCHILDREN, WS_CLIPSIBLINGS,
            WS_EX_OVERLAPPEDWINDOW, WS_OVERLAPPEDWINDOW,
        },
    },
    core::{Interface, PCSTR, s, w},
};

pub struct Platform {
    device_luid: [u8; vk::VK_LUID_SIZE as usize],
    node_mask: u32,
    hinstance: HMODULE,
    d3d12_queue: ID3D12CommandQueue,
    d3d12_device: ID3D12Device12,
    dxgi_factory: IDXGIFactory7,
    drawn: usize,
}

static LUID: OnceLock<[u8; vk::VK_LUID_SIZE as usize]> = OnceLock::new();
static NODE_MASK: OnceLock<u32> = OnceLock::new();

impl Platform {
    pub fn open_window(
        &mut self,
        instance: &vk::Instance,
        key: usize,
        width: u32,
        height: u32,
        app: *mut App,
    ) -> Result<(PlatformWindow, RedrawTrigger)> {
        LUID.set(self.device_luid);
        NODE_MASK.set(self.node_mask);

        unsafe {
            let mut rect = RECT {
                left: 0,
                top: 0,
                right: 1000,
                bottom: 1000,
            };
            AdjustWindowRectEx(
                &mut rect,
                WS_OVERLAPPEDWINDOW | WS_CLIPCHILDREN | WS_CLIPSIBLINGS,
                false,
                WS_EX_OVERLAPPEDWINDOW,
            )
            .unwrap();
            log::info!("rect");

            let hwnd = CreateWindowExW(
                WS_EX_OVERLAPPEDWINDOW,
                w!("whiskey"),
                w!("Whiskey window"),
                WS_OVERLAPPEDWINDOW | WS_CLIPCHILDREN | WS_CLIPSIBLINGS,
                i32::MIN,
                i32::MIN,
                rect.right - rect.left,
                rect.bottom - rect.top,
                None,
                None,
                Some(self.hinstance.into()),
                None,
            )
            .unwrap();
            log::info!("hwnd");

            SetWindowLongPtrW(hwnd, GWLP_USERDATA, 0);
            SetClassLongPtrW(hwnd, GET_CLASS_LONG_INDEX(0), app as isize);

            log::info!("platform window created");
            Ok((
                PlatformWindow {
                    hwnd,
                    waitable_object: None,
                    swapchain: None,
                    heap: None,
                    last_draw_end: Instant::now(),
                },
                RedrawTrigger,
            ))
        }
    }

    pub fn run(on_init: impl FnOnce(&mut App)) -> Result<()> {
        let (vk_create_instance, vk_get_instance_proc_addr) = unsafe {
            let vulkan = LoadLibraryW(w!("vulkan-1.dll")).expect("no vulkan");
            let vk_create_instance = GetProcAddress(vulkan, s!("vkCreateInstance"));
            if vk_create_instance.is_none() {
                bail!("");
            }

            let vk_get_instance_proc_addr = GetProcAddress(vulkan, s!("vkGetInstanceProcAddr"));
            if vk_get_instance_proc_addr.is_none() {
                panic!("symbol not found: get addr");
            }
            let get_addr: unsafe extern "system" fn(
                &vk::Instance,
                *const ffi::c_char,
            ) -> *const u8 = mem::transmute(vk_get_instance_proc_addr);

            (mem::transmute(vk_create_instance), get_addr)
        };

        let hinstance = unsafe { GetModuleHandleW(None).unwrap() };

        let mut app = Box::into_raw(Box::new(App::new(
            vk_create_instance,
            vk_get_instance_proc_addr,
            |instance, &drm_props| {
                drm_props.device_node_mask;
                if drm_props.device_luid_valid == 0 {
                    panic!("no valid luid");
                }

                unsafe {
                    {
                        let mut debug: Option<ID3D12Debug> = None;
                        if let Some(debug) = D3D12GetDebugInterface(&mut debug).ok().and(debug) {
                            debug.EnableDebugLayer();
                        } else {
                            eprintln!("no debug :c");
                        }
                    }

                    let dxgi_factory_flags = if true {
                        DXGI_CREATE_FACTORY_DEBUG
                    } else {
                        DXGI_CREATE_FACTORY_FLAGS(0)
                    };

                    let factory: IDXGIFactory7 = CreateDXGIFactory2(dxgi_factory_flags).unwrap();
                    let adapter: IDXGIAdapter4 = factory
                        .EnumAdapterByLuid(mem::transmute(drm_props.device_luid))
                        .unwrap();

                    let mut device = None;
                    D3D12CreateDevice(&adapter, D3D_FEATURE_LEVEL_12_0, &mut device).unwrap();
                    let device: ID3D12Device12 = device.unwrap();

                    // std::panic::set_hook(Box::new(|info| {
                    //     use std::io::Write;
                    //     let mut f = fs::OpenOptions::new()
                    //         .create(true)
                    //         .append(true)
                    //         .open("C:/Users/tbuli/dev/vk-binds-test/crash.txt")
                    //         .unwrap();
                    //     writeln!(f, "{:?} @ {:?}", info.payload_as_str(), info.location()).unwrap();
                    // }));

                    // let info_queue: ID3D12InfoQueue = device.cast().unwrap();

                    // std::panic::set_hook(Box::new(move |info| {
                    //     eprintln!("panic at: {:?}", info.location());
                    //     {
                    //         let error_message_count = info_queue.GetNumStoredMessages();

                    //         // print error message if there is any
                    //         if error_message_count > 0 {
                    //             for idx in 0..error_message_count {
                    //                 // first GetMessage() call to get the error byte length
                    //                 let mut message_byte_length = 0;
                    //                 let _ =
                    //                     info_queue.GetMessage(idx, None, &mut message_byte_length);

                    //                 // second GetMessage() to get the error and print it, be sure to allocate D3D12_MESSAGE with enough size!
                    //                 let mut messages =
                    //                     Vec::<u8>::with_capacity(message_byte_length);
                    //                 let error_message: Option<*mut D3D12_MESSAGE> =
                    //                     Some(messages.as_mut_ptr().cast());

                    //                 if let Ok(()) = info_queue.GetMessage(
                    //                     idx,
                    //                     error_message,
                    //                     &mut message_byte_length,
                    //                 ) {
                    //                     let unwrapped_message: *mut D3D12_MESSAGE =
                    //                         error_message.unwrap();
                    //                     println!(
                    //                         "{}",
                    //                         PCSTR::from_raw((*unwrapped_message).pDescription)
                    //                             .display()
                    //                     );
                    //                 }
                    //             }

                    //             // clear all printed messages
                    //             info_queue.ClearStoredMessages();
                    //         }
                    //     }
                    // }));

                    let node_mask = match device.GetNodeCount() {
                        0..1 => 0,
                        _ => drm_props.device_node_mask,
                    };

                    let queue: ID3D12CommandQueue = device
                        .CreateCommandQueue(&D3D12_COMMAND_QUEUE_DESC {
                            Type: D3D12_COMMAND_LIST_TYPE_DIRECT,
                            Priority: 0,
                            Flags: D3D12_COMMAND_QUEUE_FLAG_NONE,
                            NodeMask: node_mask,
                        })
                        .unwrap();

                    Ok(Platform {
                        drawn: 0,
                        device_luid: drm_props.device_luid,
                        node_mask: drm_props.device_node_mask,
                        hinstance,
                        d3d12_queue: queue,
                        dxgi_factory: factory,
                        d3d12_device: device,
                    })
                }
            },
        )?));

        unsafe extern "system" fn wnd_proc(
            hwnd: HWND,
            msg: u32,
            wparam: WPARAM,
            lparam: LPARAM,
        ) -> LRESULT {
            let win_idx = GetWindowLongPtrW(hwnd, GWLP_USERDATA);
            let app_ptr = GetClassLongPtrW(hwnd, GET_CLASS_LONG_INDEX(0)) as *mut App;
            if app_ptr.is_null() {
                return DefWindowProcW(hwnd, msg, wparam, lparam);
            }
            log::trace!("window message {}", dbg_wm(msg));
            match msg {
                WM_PAINT => unsafe {
                    let app = unsafe { &mut *app_ptr };
                    app.platform_redraw(0);

                    return LRESULT(0);
                },
                WM_SIZE => unsafe {
                    let factory = { &*app_ptr }.platform.dxgi_factory.clone();
                    let queue = { &*app_ptr }.platform.d3d12_queue.clone();
                    let device = { &*app_ptr }.platform.d3d12_device.clone();
                    log::info!("win size {} {}", (lparam.0 & 0xffff) as u32, lparam.0 >> 16);
                    const N: u32 = 4;
                    let swapchain = factory
                        .CreateSwapChainForHwnd(
                            &queue,
                            hwnd,
                            &DXGI_SWAP_CHAIN_DESC1 {
                                Width: 1000,
                                Height: 1000,
                                Format: DXGI_FORMAT_B8G8R8A8_UNORM,
                                Stereo: false.into(),
                                SampleDesc: DXGI_SAMPLE_DESC {
                                    Count: 1,
                                    Quality: 0,
                                },
                                BufferUsage: DXGI_USAGE_RENDER_TARGET_OUTPUT,
                                BufferCount: N,
                                Scaling: DXGI_SCALING_NONE,
                                SwapEffect: DXGI_SWAP_EFFECT_FLIP_SEQUENTIAL,
                                AlphaMode: DXGI_ALPHA_MODE_IGNORE,
                                // Flags: 0,
                                Flags: DXGI_SWAP_CHAIN_FLAG_FRAME_LATENCY_WAITABLE_OBJECT.0 as u32,
                            },
                            None,
                            None,
                        )
                        .unwrap();

                    let swapchain: IDXGISwapChain4 = swapchain.cast().unwrap();

                    swapchain.SetMaximumFrameLatency(N - 2).unwrap();

                    let cx = Context(());

                    let rtv_heap: ID3D12DescriptorHeap = unsafe {
                        device.CreateDescriptorHeap(&D3D12_DESCRIPTOR_HEAP_DESC {
                            NumDescriptors: N,
                            Type: D3D12_DESCRIPTOR_HEAP_TYPE_RTV,
                            ..Default::default()
                        })
                    }
                    .unwrap();

                    let rtv_descriptor_size = unsafe {
                        device.GetDescriptorHandleIncrementSize(D3D12_DESCRIPTOR_HEAP_TYPE_RTV)
                    } as usize;
                    let rtv_handle = unsafe { rtv_heap.GetCPUDescriptorHandleForHeapStart() };

                    for i in 0..N {
                        let buffer: ID3D12Resource = swapchain.GetBuffer(i).unwrap();
                        let desc = buffer.GetDesc();
                        let handle = device
                            .CreateSharedHandle(&buffer, None, GENERIC_ALL.0, None /*some?*/)
                            .unwrap();

                        let wait_fence = device.CreateFence(0, D3D12_FENCE_FLAG_SHARED).unwrap();
                        let wait_fence_handle = device
                            .CreateSharedHandle(&wait_fence, None, GENERIC_ALL.0, None)
                            .unwrap();

                        let command_allocator = device
                            .CreateCommandAllocator(D3D12_COMMAND_LIST_TYPE_DIRECT)
                            .unwrap();
                        let command_allocator2 = device
                            .CreateCommandAllocator(D3D12_COMMAND_LIST_TYPE_DIRECT)
                            .unwrap();

                        let pre_list: ID3D12GraphicsCommandList = device
                            .CreateCommandList(
                                *NODE_MASK.wait(),
                                D3D12_COMMAND_LIST_TYPE_DIRECT,
                                &command_allocator,
                                None,
                            )
                            .unwrap();
                        pre_list.Close().unwrap();
                        let post_list: ID3D12GraphicsCommandList = device
                            .CreateCommandList(
                                *NODE_MASK.wait(),
                                D3D12_COMMAND_LIST_TYPE_DIRECT,
                                &command_allocator2,
                                None,
                            )
                            .unwrap();
                        post_list.Close().unwrap();

                        let mut image = ptr::null();
                        (cx.device_fns.create_image)(
                            cx.device,
                            &vk::ImageCreateInfo {
                                s_type: vk::StructureType::ImageCreateInfo,
                                p_next: ptr::from_ref(&vk::ExternalMemoryImageCreateInfo {
                                    s_type: vk::StructureType::ExternalMemoryImageCreateInfo,
                                    p_next: ptr::null(),
                                    handle_types: vk::ExternalMemoryHandleTypeBit::D3D12Resource
                                        .into(),
                                })
                                .cast(),
                                flags: vk::ImageCreateFlags(0),
                                image_type: vk::ImageType::_2D,
                                format: vk::Format::B8G8R8A8Unorm,
                                extent: dbg!(vk::Extent3D {
                                    width: desc.Width as u32,
                                    height: desc.Height,
                                    depth: 1,
                                }),
                                mip_levels: 1,
                                array_layers: 1,
                                samples: vk::SampleCountBit::_1,
                                tiling: vk::ImageTiling::Optimal,
                                usage: ImageUsageBit::ColorAttachment | ImageUsageBit::Storage,
                                sharing_mode: vk::SharingMode::Exclusive,
                                queue_family_index_count: 0,
                                p_queue_family_indices: ptr::null(),
                                initial_layout: vk::ImageLayout::Undefined,
                            },
                            ptr::null(),
                            &mut image,
                        )
                        .as_result()
                        .unwrap();

                        let mut mem_requirements = MaybeUninit::uninit();
                        (cx.device_fns.get_image_memory_requirements)(
                            cx.device,
                            image,
                            mem_requirements.as_mut_ptr(),
                        );
                        let mem_requirements = mem_requirements.assume_init();

                        let memory = cx
                            .allocator
                            .alloc_dedicated(
                                vk::MemoryPropertyBit::DeviceLocal.into(),
                                mem_requirements.memory_type_bits,
                                mem_requirements.size,
                                ptr::from_ref(&vk::ImportMemoryWin32HandleInfo {
                                    s_type: vk::StructureType::IMPORT_MEMORY_WIN32_HANDLE_INFO,
                                    p_next: ptr::from_ref(&vk::MemoryDedicatedAllocateInfo {
                                        s_type: vk::StructureType::MemoryDedicatedAllocateInfo,
                                        p_next: ptr::null(),
                                        image,
                                        buffer: ptr::null(),
                                    })
                                    .cast(),
                                    handle_type: vk::ExternalMemoryHandleTypeBit::D3D12Resource,
                                    handle: handle,
                                    name: Default::default(),
                                })
                                .cast(),
                            )
                            .unwrap();

                        (cx.device_fns.bind_image_memory)(cx.device, image, memory, 0)
                            .as_result()
                            .unwrap();

                        let mut wait_semaphore = ptr::null();
                        let mut signal_semaphore = ptr::null();
                        let mut img_view = ptr::null();

                        (cx.device_fns.create_semaphore)(
                            cx.device,
                            &vk::SemaphoreCreateInfo {
                                s_type: vk::StructureType::SemaphoreCreateInfo,
                                // p_next: ptr::from_ref(&vk::ExportSemaphoreCreateInfo {
                                //     s_type: StructureType::ExportSemaphoreCreateInfo,
                                //     p_next: ptr::null(),
                                //     handle_types: vk::ExternalSemaphoreHandleTypeBit::SyncFd.into(),
                                // })
                                // .cast(),
                                p_next: ptr::null(),
                                flags: vk::SemaphoreCreateFlags(0),
                            },
                            ptr::null(),
                            &mut signal_semaphore,
                        )
                        .as_result()
                        .unwrap();

                        (cx.device_fns.create_semaphore)(
                            cx.device,
                            &vk::SemaphoreCreateInfo {
                                s_type: vk::StructureType::SemaphoreCreateInfo,
                                p_next: ptr::from_ref(&SemaphoreTypeCreateInfo {
                                    s_type: vk::StructureType::SemaphoreTypeCreateInfo,
                                    p_next: ptr::null(),
                                    semaphore_type: vk::SemaphoreType::Timeline,
                                    initial_value: 0,
                                })
                                .cast(),
                                flags: vk::SemaphoreCreateFlags(0),
                            },
                            ptr::null(),
                            &mut wait_semaphore,
                        )
                        .as_result()
                        .unwrap();

                        (cx.device_fns.import_semaphore_win32_handle)(
                            cx.device,
                            &vk::ImportSemaphoreWin32HandleInfo {
                                s_type: vk::StructureType::IMPORT_SEMAPHORE_WIN32_HANDLE_INFO,
                                p_next: ptr::null(),
                                semaphore: wait_semaphore,
                                flags: vk::SemaphoreImportFlags(0),
                                handle_type: vk::ExternalSemaphoreHandleTypeBit::D3D12Fence,
                                handle: wait_fence_handle,
                                name: Default::default(),
                            },
                        )
                        .as_result()
                        .unwrap();

                        // TODO: release wait_fence_handle?

                        (cx.device_fns.create_image_view)(
                            cx.device,
                            &vk::ImageViewCreateInfo {
                                s_type: vk::StructureType::ImageViewCreateInfo,
                                p_next: ptr::null(),
                                flags: vk::ImageViewCreateFlags(0),
                                image: image,
                                view_type: vk::ImageViewType::_2D,
                                format: vk::Format::B8G8R8A8Unorm,
                                components: vk::ComponentMapping {
                                    r: vk::ComponentSwizzle::Identity,
                                    g: vk::ComponentSwizzle::Identity,
                                    b: vk::ComponentSwizzle::Identity,
                                    a: vk::ComponentSwizzle::Identity,
                                },
                                subresource_range: vk::ImageSubresourceRange {
                                    aspect_mask: vk::ImageAspectBit::Color.into(),
                                    base_mip_level: 0,
                                    level_count: 1,
                                    base_array_layer: 0,
                                    layer_count: 1,
                                },
                            },
                            ptr::null(),
                            &mut img_view,
                        )
                        .as_result()
                        .unwrap();

                        device.CreateRenderTargetView(
                            &buffer,
                            None,
                            D3D12_CPU_DESCRIPTOR_HANDLE {
                                ptr: rtv_handle.ptr + i as usize * rtv_descriptor_size,
                            },
                        );

                        let app = unsafe { &mut *app_ptr };
                        let k =
                            app.windows[0]
                                .as_mut()
                                .unwrap()
                                .window
                                .images
                                .insert(SwapchainImage {
                                    vk_image: image,
                                    view: img_view,
                                    wait_semaphore: wait_semaphore,
                                    wait_fence,
                                    d3d12_image: buffer,
                                    n: 0,
                                    signal_semaphore: signal_semaphore,
                                    command_allocator,
                                    command_allocator2,
                                    pre_list,
                                    post_list,
                                });
                        dbg!(k);
                    }

                    let app = unsafe { &mut *app_ptr };
                    app.windows[0]
                        .as_mut()
                        .unwrap()
                        .window
                        .platform
                        .waitable_object
                        .insert(swapchain.GetFrameLatencyWaitableObject());
                    app.windows[0]
                        .as_mut()
                        .unwrap()
                        .window
                        .platform
                        .swapchain
                        .insert(swapchain);
                    app.windows[0]
                        .as_mut()
                        .unwrap()
                        .window
                        .platform
                        .heap
                        .insert(rtv_heap);

                    return LRESULT(0);
                },
                WM_ERASEBKGND => return LRESULT(0),
                _ => return unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) },
            }
        }

        unsafe {
            let atom = RegisterClassExW(&WNDCLASSEXW {
                cbSize: mem::size_of::<WNDCLASSEXW>() as u32,
                style: CS_OWNDC,
                lpfnWndProc: Some(wnd_proc),
                cbClsExtra: mem::size_of::<*mut App>() as i32,
                cbWndExtra: 0,
                hInstance: hinstance.into(),
                hIcon: LoadIconW(None, IDI_APPLICATION).unwrap(),
                hCursor: LoadCursorW(None, IDC_ARROW).unwrap(),
                hbrBackground: HBRUSH::default(),
                lpszMenuName: Default::default(),
                lpszClassName: w!("whiskey"),
                hIconSm: LoadIconW(None, IDI_APPLICATION).unwrap(),
            });
        }

        log::info!("class");

        unsafe {
            on_init(&mut *app);
        }

        unsafe {
            let mut msg = MSG::default();
            while GetMessageW(&mut msg, None, 0, 0).as_bool() {
                TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }
        }

        // message loop

        Ok(())
    }
}

impl App {
    fn platform_redraw(&mut self, key: usize) {
        // if self.platform.drawn == 11 {
        //     process::exit(0);
        // }
        unsafe {
            let window = &mut self.windows[0].as_mut().unwrap().window;
            let since_last = window.platform.last_draw_end.elapsed();
            log::info!("back after {since_last:?}");
            let start = Instant::now();
            WaitForSingleObjectEx(window.platform.waitable_object.unwrap(), INFINITE, true);
            let idx = window
                .platform
                .swapchain
                .as_mut()
                .unwrap()
                .GetCurrentBackBufferIndex();
            let prev_idx = window.next_image_idx;
            window.next_image_idx = idx as usize;

            let image = &mut window.images[idx as usize];
            image.n += 1;

            image.command_allocator2.Reset().unwrap();
            image
                .post_list
                .Reset(&image.command_allocator2, None)
                .unwrap();
            // let barrier = transition_barrier(
            //     &image.d3d12_image,
            //     D3D12_RESOURCE_STATE_PRESENT,
            //     D3D12_RESOURCE_STATE_RENDER_TARGET,
            // );
            // image.post_list.ResourceBarrier(&[barrier]);

            let rtv_descriptor_size = unsafe {
                self.platform
                    .d3d12_device
                    .GetDescriptorHandleIncrementSize(D3D12_DESCRIPTOR_HEAP_TYPE_RTV)
            } as usize;

            let rtv_handle = D3D12_CPU_DESCRIPTOR_HANDLE {
                ptr: unsafe {
                    window
                        .platform
                        .heap
                        .as_ref()
                        .unwrap()
                        .GetCPUDescriptorHandleForHeapStart()
                }
                .ptr + idx as usize * rtv_descriptor_size,
            };

            image.post_list.ClearRenderTargetView(
                rtv_handle,
                &[0.0_f32, 0.2_f32, 0.4_f32, 1.0_f32],
                None,
            );
            image.post_list.Close().unwrap();

            self.platform
                .d3d12_queue
                .ExecuteCommandLists(&[Some(image.post_list.cast().unwrap())]);

            self.platform
                .d3d12_queue
                .Signal(&image.wait_fence, image.n)
                .unwrap();

            log::debug!("drawing to image {idx} @ ");
            self.redraw_window(0, prev_idx);

            let window = &mut self.windows[0].as_mut().unwrap().window;
            let image = &mut window.images[idx as usize];
            let fence = self.cx.create_fence(false).unwrap();
            self.cx
                .graphics_queue()
                .submit(
                    &[
                        SubmitInfo2::new(&[]).wait_semaphores(&[SemaphoreSubmitInfo::new(
                            &*image.signal_semaphore,
                            0,
                            PipelineStageBit2::AllCommands,
                        )]),
                    ],
                    Some(&fence),
                )
                .unwrap();
            fence.wait().unwrap();
            drop(fence);
            (self.cx.device_fns.device_wait_idle)(self.cx.device)
                .as_result()
                .unwrap();
            // log::info!("drawing done after {:?}", start.elapsed());
            // eprintln!("time since last draw = {since_last:?}");

            log::debug!("drawing complete after {:?}", start.elapsed());
            let start = Instant::now();

            // if image.n == 100 {
            //     panic!();
            // }

            // fn transition_barrier(
            //     resource: &ID3D12Resource,
            //     state_before: D3D12_RESOURCE_STATES,
            //     state_after: D3D12_RESOURCE_STATES,
            // ) -> D3D12_RESOURCE_BARRIER {
            //     D3D12_RESOURCE_BARRIER {
            //         Type: D3D12_RESOURCE_BARRIER_TYPE_TRANSITION,
            //         Flags: D3D12_RESOURCE_BARRIER_FLAG_NONE,
            //         Anonymous: D3D12_RESOURCE_BARRIER_0 {
            //             Transition: std::mem::ManuallyDrop::new(
            //                 D3D12_RESOURCE_TRANSITION_BARRIER {
            //                     pResource: unsafe { std::mem::transmute_copy(resource) },
            //                     StateBefore: state_before,
            //                     StateAfter: state_after,
            //                     Subresource: D3D12_RESOURCE_BARRIER_ALL_SUBRESOURCES,
            //                 },
            //             ),
            //         },
            //     }
            // }

            // image.command_allocator.Reset().unwrap();

            // log::info!("CLEAR STATE");
            // image
            //     .pre_list
            //     .Reset(&image.command_allocator, None)
            //     .unwrap();
            // let barrier = transition_barrier(
            //     &image.d3d12_image,
            //     D3D12_RESOURCE_STATE_RENDER_TARGET,
            //     D3D12_RESOURCE_STATE_PRESENT,
            // );
            // image.pre_list.ResourceBarrier(&[barrier]);
            // image.pre_list.Close().unwrap();

            // self.platform
            //     .d3d12_queue
            //     .ExecuteCommandLists(&[Some(image.pre_list.cast().unwrap())]);

            window
                .platform
                .swapchain
                .as_mut()
                .unwrap()
                .Present(1, DXGI_PRESENT(0))
                .unwrap();
            // log::info!("PESENTED");

            let p_time = start.elapsed();
            if p_time > Duration::from_millis(10) {
                log::warn!("image presented after {:?}", p_time);
            } else {
                log::debug!("image presented after {:?}", p_time);
            }

            window.platform.last_draw_end = Instant::now();
        }
    }
}

pub struct RedrawTrigger;

pub struct PlatformWindow {
    hwnd: HWND,
    swapchain: Option<IDXGISwapChain4>,
    waitable_object: Option<HANDLE>,
    heap: Option<ID3D12DescriptorHeap>,
    last_draw_end: Instant,
}

impl PlatformWindow {
    pub fn show(&self) {
        unsafe {
            let _ = ShowWindow(self.hwnd, SW_SHOW);
            SetForegroundWindow(self.hwnd).unwrap();
        }
    }
}

pub struct SwapchainImage {
    pub(crate) vk_image: *const vk::Image,
    pub(crate) view: *const vk::ImageView,
    pub(crate) wait_semaphore: *const vk::Semaphore,
    pub(crate) n: u64,
    pub(crate) signal_semaphore: *const vk::Semaphore,
    d3d12_image: ID3D12Resource,
    wait_fence: ID3D12Fence,
    command_allocator: ID3D12CommandAllocator,
    command_allocator2: ID3D12CommandAllocator,
    pre_list: ID3D12GraphicsCommandList,
    post_list: ID3D12GraphicsCommandList,
}

fn dbg_wm(msg: u32) -> &'static str {
    match msg {
        6u32 => "WM_ACTIVATE",
        28u32 => "WM_ACTIVATEAPP",
        864u32 => "WM_AFXFIRST",
        895u32 => "WM_AFXLAST",
        32768u32 => "WM_APP",
        793u32 => "WM_APPCOMMAND",
        780u32 => "WM_ASKCBFORMATNAME",
        75u32 => "WM_CANCELJOURNAL",
        31u32 => "WM_CANCELMODE",
        533u32 => "WM_CAPTURECHANGED",
        781u32 => "WM_CHANGECBCHAIN",
        295u32 => "WM_CHANGEUISTATE",
        258u32 => "WM_CHAR",
        47u32 => "WM_CHARTOITEM",
        34u32 => "WM_CHILDACTIVATE",
        771u32 => "WM_CLEAR",
        797u32 => "WM_CLIPBOARDUPDATE",
        16u32 => "WM_CLOSE",
        273u32 => "WM_COMMAND",
        68u32 => "WM_COMMNOTIFY",
        65u32 => "WM_COMPACTING",
        57u32 => "WM_COMPAREITEM",
        123u32 => "WM_CONTEXTMENU",
        769u32 => "WM_COPY",
        74u32 => "WM_COPYDATA",
        1u32 => "WM_CREATE",
        309u32 => "WM_CTLCOLORBTN",
        310u32 => "WM_CTLCOLORDLG",
        307u32 => "WM_CTLCOLOREDIT",
        308u32 => "WM_CTLCOLORLISTBOX",
        306u32 => "WM_CTLCOLORMSGBOX",
        311u32 => "WM_CTLCOLORSCROLLBAR",
        312u32 => "WM_CTLCOLORSTATIC",
        768u32 => "WM_CUT",
        259u32 => "WM_DEADCHAR",
        45u32 => "WM_DELETEITEM",
        2u32 => "WM_DESTROY",
        775u32 => "WM_DESTROYCLIPBOARD",
        537u32 => "WM_DEVICECHANGE",
        27u32 => "WM_DEVMODECHANGE",
        126u32 => "WM_DISPLAYCHANGE",
        736u32 => "WM_DPICHANGED",
        739u32 => "WM_DPICHANGED_AFTERPARENT",
        738u32 => "WM_DPICHANGED_BEFOREPARENT",
        776u32 => "WM_DRAWCLIPBOARD",
        43u32 => "WM_DRAWITEM",
        563u32 => "WM_DROPFILES",
        800u32 => "WM_DWMCOLORIZATIONCOLORCHANGED",
        798u32 => "WM_DWMCOMPOSITIONCHANGED",
        799u32 => "WM_DWMNCRENDERINGCHANGED",
        806u32 => "WM_DWMSENDICONICLIVEPREVIEWBITMAP",
        803u32 => "WM_DWMSENDICONICTHUMBNAIL",
        801u32 => "WM_DWMWINDOWMAXIMIZEDCHANGE",
        10u32 => "WM_ENABLE",
        22u32 => "WM_ENDSESSION",
        289u32 => "WM_ENTERIDLE",
        529u32 => "WM_ENTERMENULOOP",
        561u32 => "WM_ENTERSIZEMOVE",
        20u32 => "WM_ERASEBKGND",
        530u32 => "WM_EXITMENULOOP",
        562u32 => "WM_EXITSIZEMOVE",
        29u32 => "WM_FONTCHANGE",
        281u32 => "WM_GESTURE",
        282u32 => "WM_GESTURENOTIFY",
        135u32 => "WM_GETDLGCODE",
        740u32 => "WM_GETDPISCALEDSIZE",
        49u32 => "WM_GETFONT",
        51u32 => "WM_GETHOTKEY",
        127u32 => "WM_GETICON",
        36u32 => "WM_GETMINMAXINFO",
        61u32 => "WM_GETOBJECT",
        13u32 => "WM_GETTEXT",
        14u32 => "WM_GETTEXTLENGTH",
        831u32 => "WM_GETTITLEBARINFOEX",
        856u32 => "WM_HANDHELDFIRST",
        863u32 => "WM_HANDHELDLAST",
        83u32 => "WM_HELP",
        786u32 => "WM_HOTKEY",
        276u32 => "WM_HSCROLL",
        782u32 => "WM_HSCROLLCLIPBOARD",
        39u32 => "WM_ICONERASEBKGND",
        646u32 => "WM_IME_CHAR",
        271u32 => "WM_IME_COMPOSITION",
        644u32 => "WM_IME_COMPOSITIONFULL",
        643u32 => "WM_IME_CONTROL",
        270u32 => "WM_IME_ENDCOMPOSITION",
        656u32 => "WM_IME_KEYDOWN",
        271u32 => "WM_IME_KEYLAST",
        657u32 => "WM_IME_KEYUP",
        642u32 => "WM_IME_NOTIFY",
        648u32 => "WM_IME_REQUEST",
        645u32 => "WM_IME_SELECT",
        641u32 => "WM_IME_SETCONTEXT",
        269u32 => "WM_IME_STARTCOMPOSITION",
        272u32 => "WM_INITDIALOG",
        278u32 => "WM_INITMENU",
        279u32 => "WM_INITMENUPOPUP",
        255u32 => "WM_INPUT",
        81u32 => "WM_INPUTLANGCHANGE",
        80u32 => "WM_INPUTLANGCHANGEREQUEST",
        254u32 => "WM_INPUT_DEVICE_CHANGE",
        256u32 => "WM_KEYDOWN",
        256u32 => "WM_KEYFIRST",
        265u32 => "WM_KEYLAST",
        257u32 => "WM_KEYUP",
        8u32 => "WM_KILLFOCUS",
        515u32 => "WM_LBUTTONDBLCLK",
        513u32 => "WM_LBUTTONDOWN",
        514u32 => "WM_LBUTTONUP",
        521u32 => "WM_MBUTTONDBLCLK",
        519u32 => "WM_MBUTTONDOWN",
        520u32 => "WM_MBUTTONUP",
        546u32 => "WM_MDIACTIVATE",
        551u32 => "WM_MDICASCADE",
        544u32 => "WM_MDICREATE",
        545u32 => "WM_MDIDESTROY",
        553u32 => "WM_MDIGETACTIVE",
        552u32 => "WM_MDIICONARRANGE",
        549u32 => "WM_MDIMAXIMIZE",
        548u32 => "WM_MDINEXT",
        564u32 => "WM_MDIREFRESHMENU",
        547u32 => "WM_MDIRESTORE",
        560u32 => "WM_MDISETMENU",
        550u32 => "WM_MDITILE",
        44u32 => "WM_MEASUREITEM",
        288u32 => "WM_MENUCHAR",
        294u32 => "WM_MENUCOMMAND",
        291u32 => "WM_MENUDRAG",
        292u32 => "WM_MENUGETOBJECT",
        290u32 => "WM_MENURBUTTONUP",
        287u32 => "WM_MENUSELECT",
        33u32 => "WM_MOUSEACTIVATE",
        512u32 => "WM_MOUSEFIRST",
        526u32 => "WM_MOUSEHWHEEL",
        526u32 => "WM_MOUSELAST",
        512u32 => "WM_MOUSEMOVE",
        522u32 => "WM_MOUSEWHEEL",
        3u32 => "WM_MOVE",
        534u32 => "WM_MOVING",
        134u32 => "WM_NCACTIVATE",
        131u32 => "WM_NCCALCSIZE",
        129u32 => "WM_NCCREATE",
        130u32 => "WM_NCDESTROY",
        132u32 => "WM_NCHITTEST",
        163u32 => "WM_NCLBUTTONDBLCLK",
        161u32 => "WM_NCLBUTTONDOWN",
        162u32 => "WM_NCLBUTTONUP",
        169u32 => "WM_NCMBUTTONDBLCLK",
        167u32 => "WM_NCMBUTTONDOWN",
        168u32 => "WM_NCMBUTTONUP",
        672u32 => "WM_NCMOUSEHOVER",
        674u32 => "WM_NCMOUSELEAVE",
        160u32 => "WM_NCMOUSEMOVE",
        133u32 => "WM_NCPAINT",
        578u32 => "WM_NCPOINTERDOWN",
        579u32 => "WM_NCPOINTERUP",
        577u32 => "WM_NCPOINTERUPDATE",
        166u32 => "WM_NCRBUTTONDBLCLK",
        164u32 => "WM_NCRBUTTONDOWN",
        165u32 => "WM_NCRBUTTONUP",
        173u32 => "WM_NCXBUTTONDBLCLK",
        171u32 => "WM_NCXBUTTONDOWN",
        172u32 => "WM_NCXBUTTONUP",
        40u32 => "WM_NEXTDLGCTL",
        531u32 => "WM_NEXTMENU",
        78u32 => "WM_NOTIFY",
        85u32 => "WM_NOTIFYFORMAT",
        0u32 => "WM_NULL",
        15u32 => "WM_PAINT",
        777u32 => "WM_PAINTCLIPBOARD",
        38u32 => "WM_PAINTICON",
        785u32 => "WM_PALETTECHANGED",
        784u32 => "WM_PALETTEISCHANGING",
        528u32 => "WM_PARENTNOTIFY",
        770u32 => "WM_PASTE",
        896u32 => "WM_PENWINFIRST",
        911u32 => "WM_PENWINLAST",
        587u32 => "WM_POINTERACTIVATE",
        588u32 => "WM_POINTERCAPTURECHANGED",
        568u32 => "WM_POINTERDEVICECHANGE",
        569u32 => "WM_POINTERDEVICEINRANGE",
        570u32 => "WM_POINTERDEVICEOUTOFRANGE",
        582u32 => "WM_POINTERDOWN",
        585u32 => "WM_POINTERENTER",
        591u32 => "WM_POINTERHWHEEL",
        586u32 => "WM_POINTERLEAVE",
        594u32 => "WM_POINTERROUTEDAWAY",
        595u32 => "WM_POINTERROUTEDRELEASED",
        593u32 => "WM_POINTERROUTEDTO",
        583u32 => "WM_POINTERUP",
        581u32 => "WM_POINTERUPDATE",
        590u32 => "WM_POINTERWHEEL",
        72u32 => "WM_POWER",
        536u32 => "WM_POWERBROADCAST",
        791u32 => "WM_PRINT",
        792u32 => "WM_PRINTCLIENT",
        55u32 => "WM_QUERYDRAGICON",
        17u32 => "WM_QUERYENDSESSION",
        783u32 => "WM_QUERYNEWPALETTE",
        19u32 => "WM_QUERYOPEN",
        297u32 => "WM_QUERYUISTATE",
        35u32 => "WM_QUEUESYNC",
        18u32 => "WM_QUIT",
        518u32 => "WM_RBUTTONDBLCLK",
        516u32 => "WM_RBUTTONDOWN",
        517u32 => "WM_RBUTTONUP",
        774u32 => "WM_RENDERALLFORMATS",
        773u32 => "WM_RENDERFORMAT",
        32u32 => "WM_SETCURSOR",
        7u32 => "WM_SETFOCUS",
        48u32 => "WM_SETFONT",
        50u32 => "WM_SETHOTKEY",
        128u32 => "WM_SETICON",
        11u32 => "WM_SETREDRAW",
        12u32 => "WM_SETTEXT",
        26u32 => "WM_SETTINGCHANGE",
        24u32 => "WM_SHOWWINDOW",
        5u32 => "WM_SIZE",
        779u32 => "WM_SIZECLIPBOARD",
        532u32 => "WM_SIZING",
        42u32 => "WM_SPOOLERSTATUS",
        125u32 => "WM_STYLECHANGED",
        124u32 => "WM_STYLECHANGING",
        136u32 => "WM_SYNCPAINT",
        262u32 => "WM_SYSCHAR",
        21u32 => "WM_SYSCOLORCHANGE",
        274u32 => "WM_SYSCOMMAND",
        263u32 => "WM_SYSDEADCHAR",
        260u32 => "WM_SYSKEYDOWN",
        261u32 => "WM_SYSKEYUP",
        704u32 => "WM_TABLET_FIRST",
        735u32 => "WM_TABLET_LAST",
        82u32 => "WM_TCARD",
        794u32 => "WM_THEMECHANGED",
        30u32 => "WM_TIMECHANGE",
        275u32 => "WM_TIMER",
        837u32 => "WM_TOOLTIPDISMISS",
        576u32 => "WM_TOUCH",
        589u32 => "WM_TOUCHHITTESTING",
        772u32 => "WM_UNDO",
        265u32 => "WM_UNICHAR",
        293u32 => "WM_UNINITMENUPOPUP",
        296u32 => "WM_UPDATEUISTATE",
        1024u32 => "WM_USER",
        84u32 => "WM_USERCHANGED",
        46u32 => "WM_VKEYTOITEM",
        277u32 => "WM_VSCROLL",
        778u32 => "WM_VSCROLLCLIPBOARD",
        71u32 => "WM_WINDOWPOSCHANGED",
        70u32 => "WM_WINDOWPOSCHANGING",
        26u32 => "WM_WININICHANGE",
        689u32 => "WM_WTSSESSION_CHANGE",
        525u32 => "WM_XBUTTONDBLCLK",
        523u32 => "WM_XBUTTONDOWN",
        524u32 => "WM_XBUTTONUP",
        _ => "Unknown WM",
    }
}
