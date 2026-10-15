use std::{
    ffi::{CStr, c_char, c_void},
    mem, ops,
};

// use windows::{Win32::Foundation::HANDLE, core::PCWSTR};

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[must_use]
#[repr(transparent)]
pub struct Result(core::ffi::c_int);
impl Result {
    pub const SUCCESS: Self = Self(0);
    pub const NOT_READY: Self = Self(1);
    pub const TIMEOUT: Self = Self(2);
    pub const EVENT_SET: Self = Self(3);
    pub const EVENT_RESET: Self = Self(4);
    pub const INCOMPLETE: Self = Self(5);
    pub const ERROR_OUT_OF_HOST_MEMORY: Self = Self(-1);
    pub const ERROR_OUT_OF_DEVICE_MEMORY: Self = Self(-2);
    pub const ERROR_INITIALIZATION_FAILED: Self = Self(-3);
    pub const ERROR_DEVICE_LOST: Self = Self(-4);
    pub const ERROR_MEMORY_MAP_FAILED: Self = Self(-5);
    pub const ERROR_LAYER_NOT_PRESENT: Self = Self(-6);
    pub const ERROR_EXTENSION_NOT_PRESENT: Self = Self(-7);
    pub const ERROR_FEATURE_NOT_PRESENT: Self = Self(-8);
    pub const ERROR_INCOMPATIBLE_DRIVER: Self = Self(-9);
    pub const ERROR_TOO_MANY_OBJECTS: Self = Self(-10);
    pub const ERROR_FORMAT_NOT_SUPPORTED: Self = Self(-11);
    pub const ERROR_FRAGMENTED_POOL: Self = Self(-12);
    pub const ERROR_UNKNOWN: Self = Self(-13);
    pub const ERROR_OUT_OF_POOL_MEMORY: Self = Self(-1000069000);
    pub const ERROR_INVALID_EXTERNAL_HANDLE: Self = Self(-1000072003);
    pub const ERROR_INVALID_OPAQUE_CAPTURE_ADDRESS: Self = Self(-1000257000);
    pub const ERROR_FRAGMENTATION: Self = Self(-1000161000);
    pub const PIPELINE_COMPILE_REQUIRED: Self = Self(1000297000);
    pub const ERROR_NOT_PERMITTED: Self = Self(-1000174001);
    pub const ERROR_SURFACE_LOST_KHR: Self = Self(-1000000000);
    pub const ERROR_NATIVE_WINDOW_IN_USE_KHR: Self = Self(-1000000001);
    pub const SUBOPTIMAL_KHR: Self = Self(1000001003);
    pub const ERROR_OUT_OF_DATE_KHR: Self = Self(-1000001004);
    pub const INCOMPATIBLE_SHADER_BINARY_EXT: Self = Self(1000482000);
    pub const ERROR_INCOMPATIBLE_SHADER_BINARY_EXT: Self = Self::INCOMPATIBLE_SHADER_BINARY_EXT;
}

#[repr(transparent)]
pub struct Instance([u8; 0]);
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct AllocationCallbacks {
    pub(crate) p_user_data: *mut c_void,
    pub(crate) pfn_allocation: AllocationFunction,
    pub(crate) pfn_reallocation: ReallocationFunction,
    pub(crate) pfn_free: FreeFunction,
    pub(crate) pfn_internal_allocation: InternalAllocationNotification,
    pub(crate) pfn_internal_free: InternalFreeNotification,
}
type InternalFreeNotification = unsafe extern "system" fn(
    p_user_data: *mut c_void,
    size: usize,
    allocation_type: InternalAllocationType,
    allocation_scope: SystemAllocationScope,
);
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[repr(i32)]
pub enum SystemAllocationScope {
    Command = 0,
    Object = 1,
    Cache = 2,
    Device = 3,
    Instance = 4,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[repr(i32)]
pub enum InternalAllocationType {
    Executable = 0,
}

type InternalAllocationNotification = unsafe extern "system" fn(
    p_user_data: *mut c_void,
    size: usize,
    allocation_type: InternalAllocationType,
    allocation_scope: SystemAllocationScope,
);
type FreeFunction = unsafe extern "system" fn(p_user_data: *mut c_void, p_memory: *mut c_void);
type ReallocationFunction = unsafe extern "system" fn(
    p_user_data: *mut c_void,
    p_original: *mut c_void,
    size: usize,
    alignment: usize,
    allocation_scope: SystemAllocationScope,
) -> *mut c_void;
type AllocationFunction = unsafe extern "system" fn(
    p_user_data: *mut c_void,
    size: usize,
    alignment: usize,
    allocation_scope: SystemAllocationScope,
) -> *mut c_void;
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct InstanceCreateInfo {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *const c_void,
    pub(crate) flags: InstanceCreateFlags,
    pub(crate) p_application_info: *const ApplicationInfo,
    pub(crate) enabled_layer_count: u32,
    pub(crate) pp_enabled_layer_names: *const *const c_char,
    pub(crate) enabled_extension_count: u32,
    pub(crate) pp_enabled_extension_names: *const *const c_char,
}
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct ApplicationInfo {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *const c_void,
    pub(crate) p_application_name: *const c_char,
    pub(crate) application_version: u32,
    pub(crate) p_engine_name: *const c_char,
    pub(crate) engine_version: u32,
    pub(crate) api_version: u32,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
#[repr(i32)]
pub enum StructureType {
    #[default]
    ApplicationInfo = 0,
    InstanceCreateInfo = 1,
    DeviceQueueCreateInfo = 2,
    DeviceCreateInfo = 3,
    SubmitInfo = 4,
    MemoryAllocateInfo = 5,
    MappedMemoryRange = 6,
    BindSparseInfo = 7,
    FenceCreateInfo = 8,
    SemaphoreCreateInfo = 9,
    EventCreateInfo = 10,
    QueryPoolCreateInfo = 11,
    BufferCreateInfo = 12,
    BufferViewCreateInfo = 13,
    ImageCreateInfo = 14,
    ImageViewCreateInfo = 15,
    ShaderModuleCreateInfo = 16,
    PipelineCacheCreateInfo = 17,
    PipelineShaderStageCreateInfo = 18,
    PipelineVertexInputStateCreateInfo = 19,
    PipelineInputAssemblyStateCreateInfo = 20,
    PipelineTessellationStateCreateInfo = 21,
    PipelineViewportStateCreateInfo = 22,
    PipelineRasterizationStateCreateInfo = 23,
    PipelineMultisampleStateCreateInfo = 24,
    PipelineDepthStencilStateCreateInfo = 25,
    PipelineColorBlendStateCreateInfo = 26,
    PipelineDynamicStateCreateInfo = 27,
    GraphicsPipelineCreateInfo = 28,
    ComputePipelineCreateInfo = 29,
    PipelineLayoutCreateInfo = 30,
    SamplerCreateInfo = 31,
    DescriptorSetLayoutCreateInfo = 32,
    DescriptorPoolCreateInfo = 33,
    DescriptorSetAllocateInfo = 34,
    WriteDescriptorSet = 35,
    CopyDescriptorSet = 36,
    FramebufferCreateInfo = 37,
    RenderPassCreateInfo = 38,
    CommandPoolCreateInfo = 39,
    CommandBufferAllocateInfo = 40,
    CommandBufferInheritanceInfo = 41,
    CommandBufferBeginInfo = 42,
    RenderPassBeginInfo = 43,
    BufferMemoryBarrier = 44,
    ImageMemoryBarrier = 45,
    MemoryBarrier = 46,
    LoaderInstanceCreateInfo = 47,
    LoaderDeviceCreateInfo = 48,
    BindBufferMemoryInfo = 1000157000,
    BindImageMemoryInfo = 1000157001,
    MemoryDedicatedRequirements = 1000127000,
    MemoryDedicatedAllocateInfo = 1000127001,
    MemoryAllocateFlagsInfo = 1000060000,
    DeviceGroupCommandBufferBeginInfo = 1000060004,
    DeviceGroupSubmitInfo = 1000060005,
    DeviceGroupBindSparseInfo = 1000060006,
    BindBufferMemoryDeviceGroupInfo = 1000060013,
    BindImageMemoryDeviceGroupInfo = 1000060014,
    PhysicalDeviceGroupProperties = 1000070000,
    DeviceGroupDeviceCreateInfo = 1000070001,
    BufferMemoryRequirementsInfo2 = 1000146000,
    ImageMemoryRequirementsInfo2 = 1000146001,
    ImageSparseMemoryRequirementsInfo2 = 1000146002,
    MemoryRequirements2 = 1000146003,
    SparseImageMemoryRequirements2 = 1000146004,
    PhysicalDeviceFeatures2 = 1000059000,
    PhysicalDeviceProperties2 = 1000059001,
    FormatProperties2 = 1000059002,
    ImageFormatProperties2 = 1000059003,
    PhysicalDeviceImageFormatInfo2 = 1000059004,
    QueueFamilyProperties2 = 1000059005,
    PhysicalDeviceMemoryProperties2 = 1000059006,
    SparseImageFormatProperties2 = 1000059007,
    PhysicalDeviceSparseImageFormatInfo2 = 1000059008,
    ImageViewUsageCreateInfo = 1000117002,
    ProtectedSubmitInfo = 1000145000,
    PhysicalDeviceProtectedMemoryFeatures = 1000145001,
    PhysicalDeviceProtectedMemoryProperties = 1000145002,
    DeviceQueueInfo2 = 1000145003,
    PhysicalDeviceExternalImageFormatInfo = 1000071000,
    ExternalImageFormatProperties = 1000071001,
    PhysicalDeviceExternalBufferInfo = 1000071002,
    ExternalBufferProperties = 1000071003,
    PhysicalDeviceIdProperties = 1000071004,
    ExternalMemoryBufferCreateInfo = 1000072000,
    ExternalMemoryImageCreateInfo = 1000072001,
    ExportMemoryAllocateInfo = 1000072002,
    PhysicalDeviceExternalFenceInfo = 1000112000,
    ExternalFenceProperties = 1000112001,
    ExportFenceCreateInfo = 1000113000,
    ExportSemaphoreCreateInfo = 1000077000,
    PhysicalDeviceExternalSemaphoreInfo = 1000076000,
    ExternalSemaphoreProperties = 1000076001,
    PhysicalDeviceSubgroupProperties = 1000094000,
    PhysicalDevice16BitStorageFeatures = 1000083000,
    PhysicalDeviceVariablePointersFeatures = 1000120000,
    DescriptorUpdateTemplateCreateInfo = 1000085000,
    PhysicalDeviceMaintenance3Properties = 1000168000,
    DescriptorSetLayoutSupport = 1000168001,
    SamplerYcbcrConversionCreateInfo = 1000156000,
    SamplerYcbcrConversionInfo = 1000156001,
    BindImagePlaneMemoryInfo = 1000156002,
    ImagePlaneMemoryRequirementsInfo = 1000156003,
    PhysicalDeviceSamplerYcbcrConversionFeatures = 1000156004,
    SamplerYcbcrConversionImageFormatProperties = 1000156005,
    DeviceGroupRenderPassBeginInfo = 1000060003,
    PhysicalDevicePointClippingProperties = 1000117000,
    RenderPassInputAttachmentAspectCreateInfo = 1000117001,
    PipelineTessellationDomainOriginStateCreateInfo = 1000117003,
    RenderPassMultiviewCreateInfo = 1000053000,
    PhysicalDeviceMultiviewFeatures = 1000053001,
    PhysicalDeviceMultiviewProperties = 1000053002,
    PhysicalDeviceShaderDrawParametersFeatures = 1000063000,
    PhysicalDeviceDriverProperties = 1000196000,
    PhysicalDeviceVulkan11Features = 49,
    PhysicalDeviceVulkan11Properties = 50,
    PhysicalDeviceVulkan12Features = 51,
    PhysicalDeviceVulkan12Properties = 52,
    ImageFormatListCreateInfo = 1000147000,
    PhysicalDeviceVulkanMemoryModelFeatures = 1000211000,
    PhysicalDeviceHostQueryResetFeatures = 1000261000,
    PhysicalDeviceTimelineSemaphoreFeatures = 1000207000,
    PhysicalDeviceTimelineSemaphoreProperties = 1000207001,
    SemaphoreTypeCreateInfo = 1000207002,
    TimelineSemaphoreSubmitInfo = 1000207003,
    SemaphoreWaitInfo = 1000207004,
    SemaphoreSignalInfo = 1000207005,
    PhysicalDeviceBufferDeviceAddressFeatures = 1000257000,
    BufferDeviceAddressInfo = 1000244001,
    BufferOpaqueCaptureAddressCreateInfo = 1000257002,
    MemoryOpaqueCaptureAddressAllocateInfo = 1000257003,
    DeviceMemoryOpaqueCaptureAddressInfo = 1000257004,
    PhysicalDevice8BitStorageFeatures = 1000177000,
    PhysicalDeviceShaderAtomicInt64Features = 1000180000,
    PhysicalDeviceShaderFloat16Int8Features = 1000082000,
    PhysicalDeviceFloatControlsProperties = 1000197000,
    DescriptorSetLayoutBindingFlagsCreateInfo = 1000161000,
    PhysicalDeviceDescriptorIndexingFeatures = 1000161001,
    PhysicalDeviceDescriptorIndexingProperties = 1000161002,
    DescriptorSetVariableDescriptorCountAllocateInfo = 1000161003,
    DescriptorSetVariableDescriptorCountLayoutSupport = 1000161004,
    PhysicalDeviceScalarBlockLayoutFeatures = 1000221000,
    PhysicalDeviceSamplerFilterMinmaxProperties = 1000130000,
    SamplerReductionModeCreateInfo = 1000130001,
    PhysicalDeviceUniformBufferStandardLayoutFeatures = 1000253000,
    PhysicalDeviceShaderSubgroupExtendedTypesFeatures = 1000175000,
    AttachmentDescription2 = 1000109000,
    AttachmentReference2 = 1000109001,
    SubpassDescription2 = 1000109002,
    SubpassDependency2 = 1000109003,
    RenderPassCreateInfo2 = 1000109004,
    SubpassBeginInfo = 1000109005,
    SubpassEndInfo = 1000109006,
    PhysicalDeviceDepthStencilResolveProperties = 1000199000,
    SubpassDescriptionDepthStencilResolve = 1000199001,
    ImageStencilUsageCreateInfo = 1000246000,
    PhysicalDeviceImagelessFramebufferFeatures = 1000108000,
    FramebufferAttachmentsCreateInfo = 1000108001,
    FramebufferAttachmentImageInfo = 1000108002,
    RenderPassAttachmentBeginInfo = 1000108003,
    PhysicalDeviceSeparateDepthStencilLayoutsFeatures = 1000241000,
    AttachmentReferenceStencilLayout = 1000241001,
    AttachmentDescriptionStencilLayout = 1000241002,
    PhysicalDeviceVulkan13Features = 53,
    PhysicalDeviceVulkan13Properties = 54,
    PhysicalDeviceToolProperties = 1000245000,
    PhysicalDevicePrivateDataFeatures = 1000295000,
    DevicePrivateDataCreateInfo = 1000295001,
    PrivateDataSlotCreateInfo = 1000295002,
    MemoryBarrier2 = 1000314000,
    BufferMemoryBarrier2 = 1000314001,
    ImageMemoryBarrier2 = 1000314002,
    DependencyInfo = 1000314003,
    SubmitInfo2 = 1000314004,
    SemaphoreSubmitInfo = 1000314005,
    CommandBufferSubmitInfo = 1000314006,
    PhysicalDeviceSynchronization2Features = 1000314007,
    CopyBufferInfo2 = 1000337000,
    CopyImageInfo2 = 1000337001,
    CopyBufferToImageInfo2 = 1000337002,
    CopyImageToBufferInfo2 = 1000337003,
    BufferCopy2 = 1000337006,
    ImageCopy2 = 1000337007,
    BufferImageCopy2 = 1000337009,
    PhysicalDeviceTextureCompressionAstcHdrFeatures = 1000066000,
    FormatProperties3 = 1000360000,
    PhysicalDeviceMaintenance4Features = 1000413000,
    PhysicalDeviceMaintenance4Properties = 1000413001,
    DeviceBufferMemoryRequirements = 1000413002,
    DeviceImageMemoryRequirements = 1000413003,
    PipelineCreationFeedbackCreateInfo = 1000192000,
    PhysicalDeviceShaderTerminateInvocationFeatures = 1000215000,
    PhysicalDeviceShaderDemoteToHelperInvocationFeatures = 1000276000,
    PhysicalDevicePipelineCreationCacheControlFeatures = 1000297000,
    PhysicalDeviceZeroInitializeWorkgroupMemoryFeatures = 1000325000,
    PhysicalDeviceImageRobustnessFeatures = 1000335000,
    PhysicalDeviceSubgroupSizeControlProperties = 1000225000,
    PipelineShaderStageRequiredSubgroupSizeCreateInfo = 1000225001,
    PhysicalDeviceSubgroupSizeControlFeatures = 1000225002,
    PhysicalDeviceInlineUniformBlockFeatures = 1000138000,
    PhysicalDeviceInlineUniformBlockProperties = 1000138001,
    WriteDescriptorSetInlineUniformBlock = 1000138002,
    DescriptorPoolInlineUniformBlockCreateInfo = 1000138003,
    PhysicalDeviceShaderIntegerDotProductFeatures = 1000280000,
    PhysicalDeviceShaderIntegerDotProductProperties = 1000280001,
    PhysicalDeviceTexelBufferAlignmentProperties = 1000281001,
    BlitImageInfo2 = 1000337004,
    ResolveImageInfo2 = 1000337005,
    ImageBlit2 = 1000337008,
    ImageResolve2 = 1000337010,
    RenderingInfo = 1000044000,
    RenderingAttachmentInfo = 1000044001,
    PipelineRenderingCreateInfo = 1000044002,
    PhysicalDeviceDynamicRenderingFeatures = 1000044003,
    CommandBufferInheritanceRenderingInfo = 1000044004,
    PhysicalDeviceVulkan14Features = 55,
    PhysicalDeviceVulkan14Properties = 56,
    DeviceQueueGlobalPriorityCreateInfo = 1000174000,
    PhysicalDeviceGlobalPriorityQueryFeatures = 1000388000,
    QueueFamilyGlobalPriorityProperties = 1000388001,
    PhysicalDeviceIndexTypeUint8Features = 1000265000,
    MemoryMapInfo = 1000271000,
    MemoryUnmapInfo = 1000271001,
    PhysicalDeviceMaintenance5Features = 1000470000,
    PhysicalDeviceMaintenance5Properties = 1000470001,
    DeviceImageSubresourceInfo = 1000470004,
    SubresourceLayout2 = 1000338002,
    ImageSubresource2 = 1000338003,
    BufferUsageFlags2CreateInfo = 1000470006,
    PhysicalDeviceMaintenance6Features = 1000545000,
    PhysicalDeviceMaintenance6Properties = 1000545001,
    BindMemoryStatus = 1000545002,
    PhysicalDeviceHostImageCopyFeatures = 1000270000,
    PhysicalDeviceHostImageCopyProperties = 1000270001,
    MemoryToImageCopy = 1000270002,
    ImageToMemoryCopy = 1000270003,
    CopyImageToMemoryInfo = 1000270004,
    CopyMemoryToImageInfo = 1000270005,
    HostImageLayoutTransitionInfo = 1000270006,
    CopyImageToImageInfo = 1000270007,
    SubresourceHostMemcpySize = 1000270008,
    HostImageCopyDevicePerformanceQuery = 1000270009,
    PhysicalDeviceShaderSubgroupRotateFeatures = 1000416000,
    PhysicalDeviceShaderFloatControls2Features = 1000528000,
    PhysicalDeviceShaderExpectAssumeFeatures = 1000544000,
    PipelineCreateFlags2CreateInfo = 1000470005,
    PhysicalDevicePushDescriptorProperties = 1000080000,
    BindDescriptorSetsInfo = 1000545003,
    PushConstantsInfo = 1000545004,
    PushDescriptorSetInfo = 1000545005,
    PushDescriptorSetWithTemplateInfo = 1000545006,
    PhysicalDevicePipelineProtectedAccessFeatures = 1000466000,
    PipelineRobustnessCreateInfo = 1000068000,
    PhysicalDevicePipelineRobustnessFeatures = 1000068001,
    PhysicalDevicePipelineRobustnessProperties = 1000068002,
    PhysicalDeviceLineRasterizationFeatures = 1000259000,
    PipelineRasterizationLineStateCreateInfo = 1000259001,
    PhysicalDeviceLineRasterizationProperties = 1000259002,
    PhysicalDeviceVertexAttributeDivisorProperties = 1000525000,
    PipelineVertexInputDivisorStateCreateInfo = 1000190001,
    PhysicalDeviceVertexAttributeDivisorFeatures = 1000190002,
    RenderingAreaInfo = 1000470003,
    PhysicalDeviceDynamicRenderingLocalReadFeatures = 1000232000,
    RenderingAttachmentLocationInfo = 1000232001,
    RenderingInputAttachmentIndexInfo = 1000232002,
    SwapchainCreateInfo = 1000001000,
    PresentInfo = 1000001001,
    DeviceGroupPresentCapabilities = 1000060007,
    ImageSwapchainCreateInfo = 1000060008,
    BindImageMemorySwapchainInfo = 1000060009,
    AcquireNextImageInfo = 1000060010,
    DeviceGroupPresentInfo = 1000060011,
    DeviceGroupSwapchainCreateInfo = 1000060012,
    TexelBufferDescriptorInfo = 1000135000,
    ImageDescriptorInfo = 1000135001,
    ResourceDescriptorInfo = 1000135002,
    BindHeapInfo = 1000135003,
    PushDataInfo = 1000135004,
    DescriptorSetAndBindingMapping = 1000135005,
    ShaderDescriptorSetAndBindingMappingInfo = 1000135006,
    OpaqueCaptureDataCreateInfo = 1000135007,
    PhysicalDeviceDescriptorHeapProperties = 1000135008,
    PhysicalDeviceDescriptorHeapFeatures = 1000135009,
    CommandBufferInheritanceDescriptorHeapInfo = 1000135010,
    SamplerCustomBorderColorIndexCreateInfo = 1000135011,
    IndirectCommandsLayoutPushDataTokenNv = 1000135012,
    SubsampledImageFormatProperties = 1000135013,
    PhysicalDeviceDescriptorHeapTensorPropertiesArm = 1000135014,
    PhysicalDeviceShaderObjectFeatures = 1000482000,
    PhysicalDeviceShaderObjectProperties = 1000482001,
    ShaderCreateInfo = 1000482002,
    VertexInputBindingDescription2 = 1000352001,
    VertexInputAttributeDescription2 = 1000352002,
    SurfacePresentMode = 1000274000,
    SurfacePresentScalingCapabilities = 1000274001,
    SurfacePresentModeCompatibility = 1000274002,
    PhysicalDeviceSwapchainMaintenance1Features = 1000275000,
    SwapchainPresentFenceInfo = 1000275001,
    SwapchainPresentModesCreateInfo = 1000275002,
    SwapchainPresentModeInfo = 1000275003,
    SwapchainPresentScalingCreateInfo = 1000275004,
    ReleaseSwapchainImagesInfo = 1000275005,
    PhysicalDevicePresentId2Features = 1000479002,
    PhysicalDevicePresentWait2Features = 1000480001,
    SurfaceCapabilitiesPresentId2 = 1000479000,
    SurfaceCapabilitiesPresentWait2 = 1000480000,
    PresentId2 = 1000479001,
    PresentWait2Info = 1000480002,
    PhysicalDeviceDrmProperties = 1000353000,
    DrmFormatModifierPropertiesList = 1000158000,
    PhysicalDeviceImageDrmFormatModifierInfo = 1000158002,
    ImageDrmFormatModifierListCreateInfo = 1000158003,
    ImageDrmFormatModifierProperties = 1000158005,
    DrmFormatModifierPropertiesList2 = 1000158006,
    MemoryGetFdInfo = 1000074002,
    ImportSemaphoreFdInfo = 1000079000,
    SemaphoreGetFdInfo = 1000079001,
    ImportMemoryWin32HandleInfo = 1000073000,
    ImportSemaphoreWin32HandleInfo = 1000078000,
    PhysicalDeviceMeshShaderFeatures = 1000328000,
    PhysicalDeviceShaderUntypedPointersFeatures = 1000387000,
}

flags!(InstanceCreateFlags, u32);
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct ReleaseSwapchainImagesInfo {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *const c_void,
    pub(crate) swapchain: *const Swapchain,
    pub(crate) image_index_count: u32,
    pub(crate) p_image_indices: *const u32,
}
#[repr(transparent)]
pub struct Swapchain([u8; 0]);
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct SwapchainPresentScalingCreateInfo {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *const c_void,
    pub(crate) scaling_behavior: PresentScalingFlags,
    pub(crate) present_gravity_x: PresentGravityFlags,
    pub(crate) present_gravity_y: PresentGravityFlags,
}
flags!(PresentGravityBit, PresentGravityFlags, u32);
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[repr(i32)]
pub enum PresentGravityBit {
    Min = 1 << 0,
    Max = 1 << 1,
    Centered = 1 << 2,
}

flags!(PresentScalingBit, PresentScalingFlags, u32);
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[repr(i32)]
pub enum PresentScalingBit {
    OneToOne = 1 << 0,
    AspectRatioStretch = 1 << 1,
    Stretch = 1 << 2,
}

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct SwapchainPresentModeInfo {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *const c_void,
    pub(crate) swapchain_count: u32,
    pub(crate) p_present_modes: *const PresentMode,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[repr(i32)]
pub enum PresentMode {
    Immediate = 0,
    Mailbox = 1,
    Fifo = 2,
    FifoRelaxed = 3,
}

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct SwapchainPresentModesCreateInfo {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *const c_void,
    pub(crate) present_mode_count: u32,
    pub(crate) p_present_modes: *const PresentMode,
}
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct SwapchainPresentFenceInfo {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *const c_void,
    pub(crate) swapchain_count: u32,
    pub(crate) p_fences: *const *const Fence,
}
#[repr(transparent)]
pub struct Fence([u8; 0]);
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct PhysicalDeviceSwapchainMaintenance1Features {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *mut c_void,
    pub(crate) swapchain_maintenance_1: Bool32,
}
pub type Bool32 = u32;
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct SurfacePresentModeCompatibility {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *mut c_void,
    pub(crate) present_mode_count: u32,
    pub(crate) p_present_modes: *mut PresentMode,
}
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct SurfacePresentScalingCapabilities {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *mut c_void,
    pub(crate) supported_present_scaling: PresentScalingFlags,
    pub(crate) supported_present_gravity_x: PresentGravityFlags,
    pub(crate) supported_present_gravity_y: PresentGravityFlags,
    pub(crate) min_scaled_image_extent: Extent2D,
    pub(crate) max_scaled_image_extent: Extent2D,
}
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct Extent2D {
    pub width: u32,
    pub height: u32,
}
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct SurfacePresentMode {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *mut c_void,
    pub(crate) present_mode: PresentMode,
}
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct ColorBlendAdvanced {
    pub(crate) advanced_blend_op: BlendOp,
    pub(crate) src_premultiplied: Bool32,
    pub(crate) dst_premultiplied: Bool32,
    pub(crate) blend_overlap: BlendOverlap,
    pub(crate) clamp_results: Bool32,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[repr(i32)]
pub enum BlendOverlap {
    Uncorrelated = 0,
    Disjoint = 1,
    Conjoint = 2,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[repr(i32)]
pub enum BlendOp {
    Add = 0,
    Subtract = 1,
    ReverseSubtract = 2,
    Min = 3,
    Max = 4,
}

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct ColorBlendEquation {
    pub src_color_blend_factor: BlendFactor,
    pub dst_color_blend_factor: BlendFactor,
    pub color_blend_op: BlendOp,
    pub src_alpha_blend_factor: BlendFactor,
    pub dst_alpha_blend_factor: BlendFactor,
    pub alpha_blend_op: BlendOp,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[repr(i32)]
pub enum BlendFactor {
    Zero = 0,
    One = 1,
    SrcColor = 2,
    OneMinusSrcColor = 3,
    DstColor = 4,
    OneMinusDstColor = 5,
    SrcAlpha = 6,
    OneMinusSrcAlpha = 7,
    DstAlpha = 8,
    OneMinusDstAlpha = 9,
    ConstantColor = 10,
    OneMinusConstantColor = 11,
    ConstantAlpha = 12,
    OneMinusConstantAlpha = 13,
    SrcAlphaSaturate = 14,
    Src1Color = 15,
    OneMinusSrc1Color = 16,
    Src1Alpha = 17,
    OneMinusSrc1Alpha = 18,
}

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct VertexInputAttributeDescription2 {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *mut c_void,
    pub(crate) location: u32,
    pub(crate) binding: u32,
    pub(crate) format: Format,
    pub(crate) offset: u32,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[repr(i32)]
pub enum Format {
    Undefined = 0,
    R4G4UnormPack8 = 1,
    R4G4B4A4UnormPack16 = 2,
    B4G4R4A4UnormPack16 = 3,
    R5G6B5UnormPack16 = 4,
    B5G6R5UnormPack16 = 5,
    R5G5B5A1UnormPack16 = 6,
    B5G5R5A1UnormPack16 = 7,
    A1R5G5B5UnormPack16 = 8,
    R8Unorm = 9,
    R8Snorm = 10,
    R8Uscaled = 11,
    R8Sscaled = 12,
    R8Uint = 13,
    R8Sint = 14,
    R8Srgb = 15,
    R8G8Unorm = 16,
    R8G8Snorm = 17,
    R8G8Uscaled = 18,
    R8G8Sscaled = 19,
    R8G8Uint = 20,
    R8G8Sint = 21,
    R8G8Srgb = 22,
    R8G8B8Unorm = 23,
    R8G8B8Snorm = 24,
    R8G8B8Uscaled = 25,
    R8G8B8Sscaled = 26,
    R8G8B8Uint = 27,
    R8G8B8Sint = 28,
    R8G8B8Srgb = 29,
    B8G8R8Unorm = 30,
    B8G8R8Snorm = 31,
    B8G8R8Uscaled = 32,
    B8G8R8Sscaled = 33,
    B8G8R8Uint = 34,
    B8G8R8Sint = 35,
    B8G8R8Srgb = 36,
    R8G8B8A8Unorm = 37,
    R8G8B8A8Snorm = 38,
    R8G8B8A8Uscaled = 39,
    R8G8B8A8Sscaled = 40,
    R8G8B8A8Uint = 41,
    R8G8B8A8Sint = 42,
    R8G8B8A8Srgb = 43,
    B8G8R8A8Unorm = 44,
    B8G8R8A8Snorm = 45,
    B8G8R8A8Uscaled = 46,
    B8G8R8A8Sscaled = 47,
    B8G8R8A8Uint = 48,
    B8G8R8A8Sint = 49,
    B8G8R8A8Srgb = 50,
    A8B8G8R8UnormPack32 = 51,
    A8B8G8R8SnormPack32 = 52,
    A8B8G8R8UscaledPack32 = 53,
    A8B8G8R8SscaledPack32 = 54,
    A8B8G8R8UintPack32 = 55,
    A8B8G8R8SintPack32 = 56,
    A8B8G8R8SrgbPack32 = 57,
    A2R10G10B10UnormPack32 = 58,
    A2R10G10B10SnormPack32 = 59,
    A2R10G10B10UscaledPack32 = 60,
    A2R10G10B10SscaledPack32 = 61,
    A2R10G10B10UintPack32 = 62,
    A2R10G10B10SintPack32 = 63,
    A2B10G10R10UnormPack32 = 64,
    A2B10G10R10SnormPack32 = 65,
    A2B10G10R10UscaledPack32 = 66,
    A2B10G10R10SscaledPack32 = 67,
    A2B10G10R10UintPack32 = 68,
    A2B10G10R10SintPack32 = 69,
    R16Unorm = 70,
    R16Snorm = 71,
    R16Uscaled = 72,
    R16Sscaled = 73,
    R16Uint = 74,
    R16Sint = 75,
    R16Sfloat = 76,
    R16G16Unorm = 77,
    R16G16Snorm = 78,
    R16G16Uscaled = 79,
    R16G16Sscaled = 80,
    R16G16Uint = 81,
    R16G16Sint = 82,
    R16G16Sfloat = 83,
    R16G16B16Unorm = 84,
    R16G16B16Snorm = 85,
    R16G16B16Uscaled = 86,
    R16G16B16Sscaled = 87,
    R16G16B16Uint = 88,
    R16G16B16Sint = 89,
    R16G16B16Sfloat = 90,
    R16G16B16A16Unorm = 91,
    R16G16B16A16Snorm = 92,
    R16G16B16A16Uscaled = 93,
    R16G16B16A16Sscaled = 94,
    R16G16B16A16Uint = 95,
    R16G16B16A16Sint = 96,
    R16G16B16A16Sfloat = 97,
    R32Uint = 98,
    R32Sint = 99,
    R32Sfloat = 100,
    R32G32Uint = 101,
    R32G32Sint = 102,
    R32G32Sfloat = 103,
    R32G32B32Uint = 104,
    R32G32B32Sint = 105,
    R32G32B32Sfloat = 106,
    R32G32B32A32Uint = 107,
    R32G32B32A32Sint = 108,
    R32G32B32A32Sfloat = 109,
    R64Uint = 110,
    R64Sint = 111,
    R64Sfloat = 112,
    R64G64Uint = 113,
    R64G64Sint = 114,
    R64G64Sfloat = 115,
    R64G64B64Uint = 116,
    R64G64B64Sint = 117,
    R64G64B64Sfloat = 118,
    R64G64B64A64Uint = 119,
    R64G64B64A64Sint = 120,
    R64G64B64A64Sfloat = 121,
    B10G11R11UfloatPack32 = 122,
    E5B9G9R9UfloatPack32 = 123,
    D16Unorm = 124,
    X8D24UnormPack32 = 125,
    D32Sfloat = 126,
    S8Uint = 127,
    D16UnormS8Uint = 128,
    D24UnormS8Uint = 129,
    D32SfloatS8Uint = 130,
    Bc1RgbUnormBlock = 131,
    Bc1RgbSrgbBlock = 132,
    Bc1RgbaUnormBlock = 133,
    Bc1RgbaSrgbBlock = 134,
    Bc2UnormBlock = 135,
    Bc2SrgbBlock = 136,
    Bc3UnormBlock = 137,
    Bc3SrgbBlock = 138,
    Bc4UnormBlock = 139,
    Bc4SnormBlock = 140,
    Bc5UnormBlock = 141,
    Bc5SnormBlock = 142,
    Bc6HUfloatBlock = 143,
    Bc6HSfloatBlock = 144,
    Bc7UnormBlock = 145,
    Bc7SrgbBlock = 146,
    Etc2R8G8B8UnormBlock = 147,
    Etc2R8G8B8SrgbBlock = 148,
    Etc2R8G8B8A1UnormBlock = 149,
    Etc2R8G8B8A1SrgbBlock = 150,
    Etc2R8G8B8A8UnormBlock = 151,
    Etc2R8G8B8A8SrgbBlock = 152,
    EacR11UnormBlock = 153,
    EacR11SnormBlock = 154,
    EacR11G11UnormBlock = 155,
    EacR11G11SnormBlock = 156,
    Astc4X4UnormBlock = 157,
    Astc4X4SrgbBlock = 158,
    Astc5X4UnormBlock = 159,
    Astc5X4SrgbBlock = 160,
    Astc5X5UnormBlock = 161,
    Astc5X5SrgbBlock = 162,
    Astc6X5UnormBlock = 163,
    Astc6X5SrgbBlock = 164,
    Astc6X6UnormBlock = 165,
    Astc6X6SrgbBlock = 166,
    Astc8X5UnormBlock = 167,
    Astc8X5SrgbBlock = 168,
    Astc8X6UnormBlock = 169,
    Astc8X6SrgbBlock = 170,
    Astc8X8UnormBlock = 171,
    Astc8X8SrgbBlock = 172,
    Astc10X5UnormBlock = 173,
    Astc10X5SrgbBlock = 174,
    Astc10X6UnormBlock = 175,
    Astc10X6SrgbBlock = 176,
    Astc10X8UnormBlock = 177,
    Astc10X8SrgbBlock = 178,
    Astc10X10UnormBlock = 179,
    Astc10X10SrgbBlock = 180,
    Astc12X10UnormBlock = 181,
    Astc12X10SrgbBlock = 182,
    Astc12X12UnormBlock = 183,
    Astc12X12SrgbBlock = 184,
    G8B8G8R8422Unorm = 1000156000,
    B8G8R8G8422Unorm = 1000156001,
    G8B8R83Plane420Unorm = 1000156002,
    G8B8R82Plane420Unorm = 1000156003,
    G8B8R83Plane422Unorm = 1000156004,
    G8B8R82Plane422Unorm = 1000156005,
    G8B8R83Plane444Unorm = 1000156006,
    R10X6UnormPack16 = 1000156007,
    R10X6G10X6Unorm2Pack16 = 1000156008,
    R10X6G10X6B10X6A10X6Unorm4Pack16 = 1000156009,
    G10X6B10X6G10X6R10X6422Unorm4Pack16 = 1000156010,
    B10X6G10X6R10X6G10X6422Unorm4Pack16 = 1000156011,
    G10X6B10X6R10X63Plane420Unorm3Pack16 = 1000156012,
    G10X6B10X6R10X62Plane420Unorm3Pack16 = 1000156013,
    G10X6B10X6R10X63Plane422Unorm3Pack16 = 1000156014,
    G10X6B10X6R10X62Plane422Unorm3Pack16 = 1000156015,
    G10X6B10X6R10X63Plane444Unorm3Pack16 = 1000156016,
    R12X4UnormPack16 = 1000156017,
    R12X4G12X4Unorm2Pack16 = 1000156018,
    R12X4G12X4B12X4A12X4Unorm4Pack16 = 1000156019,
    G12X4B12X4G12X4R12X4422Unorm4Pack16 = 1000156020,
    B12X4G12X4R12X4G12X4422Unorm4Pack16 = 1000156021,
    G12X4B12X4R12X43Plane420Unorm3Pack16 = 1000156022,
    G12X4B12X4R12X42Plane420Unorm3Pack16 = 1000156023,
    G12X4B12X4R12X43Plane422Unorm3Pack16 = 1000156024,
    G12X4B12X4R12X42Plane422Unorm3Pack16 = 1000156025,
    G12X4B12X4R12X43Plane444Unorm3Pack16 = 1000156026,
    G16B16G16R16422Unorm = 1000156027,
    B16G16R16G16422Unorm = 1000156028,
    G16B16R163Plane420Unorm = 1000156029,
    G16B16R162Plane420Unorm = 1000156030,
    G16B16R163Plane422Unorm = 1000156031,
    G16B16R162Plane422Unorm = 1000156032,
    G16B16R163Plane444Unorm = 1000156033,
    G8B8R82Plane444Unorm = 1000330000,
    G10X6B10X6R10X62Plane444Unorm3Pack16 = 1000330001,
    G12X4B12X4R12X42Plane444Unorm3Pack16 = 1000330002,
    G16B16R162Plane444Unorm = 1000330003,
    A4R4G4B4UnormPack16 = 1000340000,
    A4B4G4R4UnormPack16 = 1000340001,
    Astc4X4SfloatBlock = 1000066000,
    Astc5X4SfloatBlock = 1000066001,
    Astc5X5SfloatBlock = 1000066002,
    Astc6X5SfloatBlock = 1000066003,
    Astc6X6SfloatBlock = 1000066004,
    Astc8X5SfloatBlock = 1000066005,
    Astc8X6SfloatBlock = 1000066006,
    Astc8X8SfloatBlock = 1000066007,
    Astc10X5SfloatBlock = 1000066008,
    Astc10X6SfloatBlock = 1000066009,
    Astc10X8SfloatBlock = 1000066010,
    Astc10X10SfloatBlock = 1000066011,
    Astc12X10SfloatBlock = 1000066012,
    Astc12X12SfloatBlock = 1000066013,
    A1B5G5R5UnormPack16 = 1000470000,
    A8Unorm = 1000470001,
}

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct VertexInputBindingDescription2 {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *mut c_void,
    pub(crate) binding: u32,
    pub(crate) stride: u32,
    pub(crate) input_rate: VertexInputRate,
    pub(crate) divisor: u32,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[repr(i32)]
pub enum VertexInputRate {
    Vertex = 0,
    Instance = 1,
}

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct PipelineShaderStageRequiredSubgroupSizeCreateInfo {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *const c_void,
    pub(crate) required_subgroup_size: u32,
}
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct ShaderCreateInfo {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *const c_void,
    pub(crate) flags: ShaderCreateFlags,
    pub(crate) stage: ShaderStageBit,
    pub(crate) next_stage: ShaderStageFlags,
    pub(crate) code_type: ShaderCodeType,
    pub(crate) code_size: usize,
    pub(crate) p_code: *const c_void,
    pub(crate) p_name: *const c_char,
    pub(crate) set_layout_count: u32,
    pub(crate) p_set_layouts: *const *const DescriptorSetLayout,
    pub(crate) push_constant_range_count: u32,
    pub(crate) p_push_constant_ranges: *const PushConstantRange,
    pub(crate) p_specialization_info: *const SpecializationInfo,
}
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct SpecializationInfo {
    pub(crate) map_entry_count: u32,
    pub(crate) p_map_entries: *const SpecializationMapEntry,
    pub(crate) data_size: usize,
    pub(crate) p_data: *const c_void,
}
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct SpecializationMapEntry {
    pub(crate) constant_id: u32,
    pub(crate) offset: u32,
    pub(crate) size: usize,
}
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct PushConstantRange {
    pub(crate) stage_flags: ShaderStageFlags,
    pub(crate) offset: u32,
    pub(crate) size: u32,
}
flags!(ShaderStageBit, ShaderStageFlags, u32);
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[repr(i32)]
pub enum ShaderStageBit {
    Vertex = 1 << 0,
    TessellationControl = 1 << 1,
    TessellationEvaluation = 1 << 2,
    Geometry = 1 << 3,
    Fragment = 1 << 4,
    Compute = 1 << 5,
    Task = 0x00000040,
    Mesh = 0x00000080,
}

impl ShaderStageFlags {
    pub const ALL_GRAPHICS: Self = Self(31);
    pub const ALL: Self = Self(2147483647);
}
#[repr(transparent)]
pub struct DescriptorSetLayout([u8; 0]);
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[repr(i32)]
pub enum ShaderCodeType {
    Binary = 0,
    Spirv = 1,
}

flags!(ShaderCreateBit, ShaderCreateFlags, u32);
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[repr(i32)]
pub enum ShaderCreateBit {
    LinkStage = 1 << 0,
    DescriptorHeap = 1 << 10,
    AllowVaryingSubgroupSize = 1 << 1,
    RequireFullSubgroups = 1 << 2,
    NoTaskShader = 1 << 3,
    DispatchBase = 1 << 4,
    FragmentShadingRateAttachment = 1 << 5,
    FragmentDensityMapAttachment = 1 << 6,
}

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct PhysicalDeviceShaderObjectProperties {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *mut c_void,
    pub(crate) shader_binary_uuid: [u8; VK_UUID_SIZE as usize],
    pub(crate) shader_binary_version: u32,
}
pub const VK_UUID_SIZE: u32 = 16;
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct PhysicalDeviceShaderObjectFeatures {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *mut c_void,
    pub(crate) shader_object: Bool32,
}
#[repr(transparent)]
pub struct Shader([u8; 0]);
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct PhysicalDeviceDescriptorHeapTensorProperties {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *mut c_void,
    pub(crate) tensor_descriptor_size: DeviceSize,
    pub(crate) tensor_descriptor_alignment: DeviceSize,
    pub(crate) tensor_capture_replay_opaque_data_size: usize,
}
pub type DeviceSize = u64;

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct SubsampledImageFormatProperties {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *const c_void,
    pub(crate) subsampled_image_descriptor_count: u32,
}
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct IndirectCommandsLayoutPushDataTokenNV {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *const c_void,
    pub(crate) push_data_offset: u32,
    pub(crate) push_data_size: u32,
}
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct SamplerCustomBorderColorIndexCreateInfo {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *const c_void,
    pub(crate) index: u32,
}
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct CommandBufferInheritanceDescriptorHeapInfo {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *const c_void,
    pub(crate) p_sampler_heap_bind_info: *const BindHeapInfo,
    pub(crate) p_resource_heap_bind_info: *const BindHeapInfo,
}
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct BindHeapInfo {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *const c_void,
    pub(crate) heap_range: DeviceAddressRange,
    pub(crate) reserved_range_offset: DeviceSize,
    pub(crate) reserved_range_size: DeviceSize,
}
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct DeviceAddressRange {
    pub(crate) address: DeviceAddress,
    pub(crate) size: DeviceSize,
}
pub type DeviceAddress = u64;
#[derive(Clone, Copy, Debug, Default)]
#[repr(C)]
pub struct PhysicalDeviceDescriptorHeapProperties {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *mut c_void,
    pub(crate) sampler_heap_alignment: DeviceSize,
    pub(crate) resource_heap_alignment: DeviceSize,
    pub(crate) max_sampler_heap_size: DeviceSize,
    pub(crate) max_resource_heap_size: DeviceSize,
    pub(crate) min_sampler_heap_reserved_range: DeviceSize,
    pub(crate) min_sampler_heap_reserved_range_with_embedded: DeviceSize,
    pub(crate) min_resource_heap_reserved_range: DeviceSize,
    pub(crate) sampler_descriptor_size: DeviceSize,
    pub(crate) image_descriptor_size: DeviceSize,
    pub(crate) buffer_descriptor_size: DeviceSize,
    pub(crate) sampler_descriptor_alignment: DeviceSize,
    pub(crate) image_descriptor_alignment: DeviceSize,
    pub(crate) buffer_descriptor_alignment: DeviceSize,
    pub(crate) max_push_data_size: DeviceSize,
    pub(crate) image_capture_replay_opaque_data_size: usize,
    pub(crate) max_descriptor_heap_embedded_samplers: u32,
    pub(crate) sampler_ycbcr_conversion_count: u32,
    pub(crate) sparse_descriptor_heaps: Bool32,
    pub(crate) protected_descriptor_heaps: Bool32,
}
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct PhysicalDeviceDescriptorHeapFeatures {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *mut c_void,
    pub(crate) descriptor_heap: Bool32,
    pub(crate) descriptor_heap_capture_replay: Bool32,
}
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct PhysicalDeviceMeshShaderFeatures {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *mut c_void,
    pub(crate) task_shader: Bool32,
    pub(crate) mesh_shader: Bool32,
    pub(crate) multiview_mesh_shader: Bool32,
    pub(crate) primitive_fragment_shading_rate_mesh_shader: Bool32,
    pub(crate) mesh_shader_queries: Bool32,
}
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct PhysicalDeviceShaderUntypedPointersFeatures {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *mut c_void,
    pub(crate) shader_untyped_pointers: Bool32,
}
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct OpaqueCaptureDataCreateInfo {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *const c_void,
    pub(crate) p_data: *const HostAddressRangeConst,
}
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct HostAddressRangeConst {
    pub(crate) address: *const c_void,
    pub(crate) size: usize,
}
unsafe impl Send for ShaderDescriptorSetAndBindingMappingInfo {}
unsafe impl Sync for ShaderDescriptorSetAndBindingMappingInfo {}
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct ShaderDescriptorSetAndBindingMappingInfo {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *const c_void,
    pub(crate) mapping_count: u32,
    pub(crate) p_mappings: *const DescriptorSetAndBindingMapping,
}

pub type DescriptorSetAndBindingMapping = crate::InputAttachmentMapping;

impl std::fmt::Debug for DescriptorMappingSourceData {
    fn fmt(&self, _writer: &mut std::fmt::Formatter) -> std::result::Result<(), std::fmt::Error> {
        Ok(())
    }
}
#[derive(Clone, Copy)]
#[repr(C)]
pub union DescriptorMappingSourceData {
    pub(crate) constant_offset: DescriptorMappingSourceConstantOffset,
    pub(crate) push_index: DescriptorMappingSourcePushIndex,
    pub(crate) indirect_index: DescriptorMappingSourceIndirectIndex,
    pub(crate) indirect_index_array: DescriptorMappingSourceIndirectIndexArray,
    pub(crate) heap_data: DescriptorMappingSourceHeapData,
    pub(crate) push_data_offset: u32,
    pub(crate) push_address_offset: u32,
    pub(crate) indirect_address: DescriptorMappingSourceIndirectAddress,
    pub(crate) shader_record_index: DescriptorMappingSourceShaderRecordIndex,
    pub(crate) shader_record_data_offset: u32,
    pub(crate) shader_record_address_offset: u32,
}
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct DescriptorMappingSourceShaderRecordIndex {
    pub(crate) heap_offset: u32,
    pub(crate) shader_record_offset: u32,
    pub(crate) heap_index_stride: u32,
    pub(crate) heap_array_stride: u32,
    pub(crate) p_embedded_sampler: *const SamplerCreateInfo,
    pub(crate) use_combined_image_sampler_index: Bool32,
    pub(crate) sampler_heap_offset: u32,
    pub(crate) sampler_shader_record_offset: u32,
    pub(crate) sampler_heap_index_stride: u32,
    pub(crate) sampler_heap_array_stride: u32,
}
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct SamplerCreateInfo {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *const c_void,
    pub(crate) flags: SamplerCreateFlags,
    pub(crate) mag_filter: Filter,
    pub(crate) min_filter: Filter,
    pub(crate) mipmap_mode: SamplerMipmapMode,
    pub(crate) address_mode_u: SamplerAddressMode,
    pub(crate) address_mode_v: SamplerAddressMode,
    pub(crate) address_mode_w: SamplerAddressMode,
    pub(crate) mip_lod_bias: f32,
    pub(crate) anisotropy_enable: Bool32,
    pub(crate) max_anisotropy: f32,
    pub(crate) compare_enable: Bool32,
    pub(crate) compare_op: CompareOp,
    pub(crate) min_lod: f32,
    pub(crate) max_lod: f32,
    pub(crate) border_color: BorderColor,
    pub(crate) unnormalized_coordinates: Bool32,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[repr(i32)]
pub enum BorderColor {
    FloatTransparentBlack = 0,
    IntTransparentBlack = 1,
    FloatOpaqueBlack = 2,
    IntOpaqueBlack = 3,
    FloatOpaqueWhite = 4,
    IntOpaqueWhite = 5,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[repr(i32)]
pub enum CompareOp {
    Never = 0,
    Less = 1,
    Equal = 2,
    LessOrEqual = 3,
    Greater = 4,
    NotEqual = 5,
    GreaterOrEqual = 6,
    Always = 7,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[repr(i32)]
pub enum SamplerAddressMode {
    Repeat = 0,
    MirroredRepeat = 1,
    ClampToEdge = 2,
    ClampToBorder = 3,
    MirrorClampToEdge = 4,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[repr(i32)]
pub enum SamplerMipmapMode {
    Nearest = 0,
    Linear = 1,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[repr(i32)]
pub enum Filter {
    Nearest = 0,
    Linear = 1,
}

flags!(SamplerCreateFlags, u32);
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct DescriptorMappingSourceIndirectAddress {
    pub(crate) push_offset: u32,
    pub(crate) address_offset: u32,
}
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct DescriptorMappingSourceHeapData {
    pub(crate) heap_offset: u32,
    pub(crate) push_offset: u32,
}
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct DescriptorMappingSourceIndirectIndexArray {
    pub(crate) heap_offset: u32,
    pub(crate) push_offset: u32,
    pub(crate) address_offset: u32,
    pub(crate) heap_index_stride: u32,
    pub(crate) p_embedded_sampler: *const SamplerCreateInfo,
    pub(crate) use_combined_image_sampler_index: Bool32,
    pub(crate) sampler_heap_offset: u32,
    pub(crate) sampler_push_offset: u32,
    pub(crate) sampler_address_offset: u32,
    pub(crate) sampler_heap_index_stride: u32,
}
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct DescriptorMappingSourceIndirectIndex {
    pub(crate) heap_offset: u32,
    pub(crate) push_offset: u32,
    pub(crate) address_offset: u32,
    pub(crate) heap_index_stride: u32,
    pub(crate) heap_array_stride: u32,
    pub(crate) p_embedded_sampler: *const SamplerCreateInfo,
    pub(crate) use_combined_image_sampler_index: Bool32,
    pub(crate) sampler_heap_offset: u32,
    pub(crate) sampler_push_offset: u32,
    pub(crate) sampler_address_offset: u32,
    pub(crate) sampler_heap_index_stride: u32,
    pub(crate) sampler_heap_array_stride: u32,
}
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct DescriptorMappingSourcePushIndex {
    pub(crate) heap_offset: u32,
    pub(crate) push_offset: u32,
    pub(crate) heap_index_stride: u32,
    pub(crate) heap_array_stride: u32,
    pub(crate) p_embedded_sampler: *const SamplerCreateInfo,
    pub(crate) use_combined_image_sampler_index: Bool32,
    pub(crate) sampler_heap_offset: u32,
    pub(crate) sampler_push_offset: u32,
    pub(crate) sampler_heap_index_stride: u32,
    pub(crate) sampler_heap_array_stride: u32,
}
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct DescriptorMappingSourceConstantOffset {
    pub(crate) heap_offset: u32,
    pub(crate) heap_array_stride: u32,
    pub(crate) p_embedded_sampler: *const SamplerCreateInfo,
    pub(crate) sampler_heap_offset: u32,
    pub(crate) sampler_heap_array_stride: u32,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[repr(i32)]
pub enum DescriptorMappingSource {
    HeapWithConstantOffset = 0,
    HeapWithPushIndex = 1,
    HeapWithIndirectIndex = 2,
    HeapWithIndirectIndexArray = 3,
    ResourceHeapData = 4,
    PushData = 5,
    PushAddress = 6,
    IndirectAddress = 7,
    HeapWithShaderRecordIndex = 8,
    ShaderRecordData = 9,
    ShaderRecordAddress = 10,
}

flags!(SpirvResourceTypeBit, SpirvResourceTypeFlags, u32);
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[repr(i32)]
pub enum SpirvResourceTypeBit {
    Sampler = 1 << 0,
    SampledImage = 1 << 1,
    ReadOnlyImage = 1 << 2,
    ReadWriteImage = 1 << 3,
    CombinedSampledImage = 1 << 4,
    UniformBuffer = 1 << 5,
    ReadOnlyStorageBuffer = 1 << 6,
    ReadWriteStorageBuffer = 1 << 7,
    AccelerationStructure = 1 << 8,
}

impl SpirvResourceTypeFlags {
    pub const ALL: Self = Self(2147483647);
    pub const TENSOR_BIT_ARM: Self = Self(1 << 9);
}
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct PushDataInfo {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *const c_void,
    pub(crate) offset: u32,
    pub(crate) data: HostAddressRangeConst,
}
impl std::fmt::Debug for ResourceDescriptorData {
    fn fmt(&self, _writer: &mut std::fmt::Formatter) -> std::result::Result<(), std::fmt::Error> {
        Ok(())
    }
}
#[derive(Clone, Copy)]
#[repr(C)]
pub union ResourceDescriptorData {
    pub(crate) p_image: *const ImageDescriptorInfo,
    pub(crate) p_texel_buffer: *const TexelBufferDescriptorInfo,
    pub(crate) p_address_range: *const DeviceAddressRange,
    pub(crate) p_tensor_arm: *const (),
}
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct TexelBufferDescriptorInfo {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *const c_void,
    pub(crate) format: Format,
    pub(crate) address_range: DeviceAddressRange,
}
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct ImageDescriptorInfo {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *const c_void,
    pub(crate) p_view: *const ImageViewCreateInfo,
    pub(crate) layout: ImageLayout,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[repr(i32)]
pub enum ImageLayout {
    Undefined = 0,
    General = 1,
    ColorAttachmentOptimal = 2,
    DepthStencilAttachmentOptimal = 3,
    DepthStencilReadOnlyOptimal = 4,
    ShaderReadOnlyOptimal = 5,
    TransferSrcOptimal = 6,
    TransferDstOptimal = 7,
    Preinitialized = 8,
    DepthReadOnlyStencilAttachmentOptimal = 1000117000,
    DepthAttachmentStencilReadOnlyOptimal = 1000117001,
    DepthAttachmentOptimal = 1000241000,
    DepthReadOnlyOptimal = 1000241001,
    StencilAttachmentOptimal = 1000241002,
    StencilReadOnlyOptimal = 1000241003,
    ReadOnlyOptimal = 1000314000,
    AttachmentOptimal = 1000314001,
    RenderingLocalRead = 1000232000,
    PresentSrc = 1000001002,
}

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct ImageViewCreateInfo {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *const c_void,
    pub(crate) flags: ImageViewCreateFlags,
    pub(crate) image: *const Image,
    pub(crate) view_type: ImageViewType,
    pub(crate) format: Format,
    pub(crate) components: ComponentMapping,
    pub(crate) subresource_range: ImageSubresourceRange,
}
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct ImageSubresourceRange {
    pub(crate) aspect_mask: ImageAspectFlags,
    pub(crate) base_mip_level: u32,
    pub(crate) level_count: u32,
    pub(crate) base_array_layer: u32,
    pub(crate) layer_count: u32,
}
flags!(ImageAspectBit, ImageAspectFlags, u32);
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[repr(i32)]
pub enum ImageAspectBit {
    Color = 1 << 0,
    Depth = 1 << 1,
    Stencil = 1 << 2,
    Metadata = 1 << 3,
    Plane0 = 1 << 4,
    Plane1 = 1 << 5,
    Plane2 = 1 << 6,
}

impl ImageAspectFlags {
    pub const NONE: Self = Self(0);
}
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct ComponentMapping {
    pub(crate) r: ComponentSwizzle,
    pub(crate) g: ComponentSwizzle,
    pub(crate) b: ComponentSwizzle,
    pub(crate) a: ComponentSwizzle,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[repr(i32)]
pub enum ComponentSwizzle {
    Identity = 0,
    Zero = 1,
    One = 2,
    R = 3,
    G = 4,
    B = 5,
    A = 6,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[repr(i32)]
pub enum ImageViewType {
    _1D = 0,
    _2D = 1,
    _3D = 2,
    Cube = 3,
    _1DArray = 4,
    _2DArray = 5,
    CubeArray = 6,
}

#[repr(transparent)]
pub struct Image([u8; 0]);
flags!(ImageViewCreateFlags, u32);
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct ResourceDescriptorInfo {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *const c_void,
    pub(crate) r#type: DescriptorType,
    pub(crate) data: ResourceDescriptorData,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[repr(i32)]
pub enum DescriptorType {
    Sampler = 0,
    CombinedImageSampler = 1,
    SampledImage = 2,
    StorageImage = 3,
    UniformTexelBuffer = 4,
    StorageTexelBuffer = 5,
    UniformBuffer = 6,
    StorageBuffer = 7,
    UniformBufferDynamic = 8,
    StorageBufferDynamic = 9,
    InputAttachment = 10,
    InlineUniformBlock = 1000138000,
}

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct HostAddressRange {
    pub(crate) address: *mut c_void,
    pub(crate) size: usize,
}
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct DeviceGroupSwapchainCreateInfo {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *const c_void,
    pub(crate) modes: DeviceGroupPresentModeFlags,
}
flags!(DeviceGroupPresentModeBit, DeviceGroupPresentModeFlags, u32);
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[repr(i32)]
pub enum DeviceGroupPresentModeBit {
    Local = 1 << 0,
    Remote = 1 << 1,
    Sum = 1 << 2,
    LocalMultiDevice = 1 << 3,
}

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct DeviceGroupPresentInfo {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *const c_void,
    pub(crate) swapchain_count: u32,
    pub(crate) p_device_masks: *const u32,
    pub(crate) mode: DeviceGroupPresentModeBit,
}
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct DeviceGroupPresentCapabilities {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *mut c_void,
    pub(crate) present_mask: [u32; VK_MAX_DEVICE_GROUP_SIZE as usize],
    pub(crate) modes: DeviceGroupPresentModeFlags,
}
pub const VK_MAX_DEVICE_GROUP_SIZE: u32 = 32;
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct AcquireNextImageInfo {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *const c_void,
    pub(crate) swapchain: *const Swapchain,
    pub(crate) timeout: u64,
    pub(crate) semaphore: *const Semaphore,
    pub(crate) fence: *const Fence,
    pub(crate) device_mask: u32,
}
#[repr(transparent)]
pub struct Semaphore([u8; 0]);
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct BindImageMemorySwapchainInfo {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *const c_void,
    pub(crate) swapchain: *const Swapchain,
    pub(crate) image_index: u32,
}
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct ImageSwapchainCreateInfo {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *const c_void,
    pub(crate) swapchain: *const Swapchain,
}
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct PresentInfo {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *const c_void,
    pub(crate) wait_semaphore_count: u32,
    pub(crate) p_wait_semaphores: *const *const Semaphore,
    pub(crate) swapchain_count: u32,
    pub(crate) p_swapchains: *const *const Swapchain,
    pub(crate) p_image_indices: *const u32,
    pub(crate) p_results: *mut Result,
}
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct SwapchainCreateInfo {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *const c_void,
    pub(crate) flags: SwapchainCreateFlags,
    pub(crate) surface: *const Surface,
    pub(crate) min_image_count: u32,
    pub(crate) image_format: Format,
    pub(crate) image_color_space: ColorSpace,
    pub(crate) image_extent: Extent2D,
    pub(crate) image_array_layers: u32,
    pub(crate) image_usage: ImageUsageFlags,
    pub(crate) image_sharing_mode: SharingMode,
    pub(crate) queue_family_index_count: u32,
    pub(crate) p_queue_family_indices: *const u32,
    pub(crate) pre_transform: SurfaceTransformBit,
    pub(crate) composite_alpha: CompositeAlphaBit,
    pub(crate) present_mode: PresentMode,
    pub(crate) clipped: Bool32,
    pub(crate) old_swapchain: *const Swapchain,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[repr(i32)]
pub enum CompositeAlphaBit {
    Opaque = 1 << 0,
    PreMultiplied = 1 << 1,
    PostMultiplied = 1 << 2,
    Inherit = 1 << 3,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[repr(i32)]
pub enum SurfaceTransformBit {
    Identity = 1 << 0,
    Rotate90 = 1 << 1,
    Rotate180 = 1 << 2,
    Rotate270 = 1 << 3,
    HorizontalMirror = 1 << 4,
    HorizontalMirrorRotate90 = 1 << 5,
    HorizontalMirrorRotate180 = 1 << 6,
    HorizontalMirrorRotate270 = 1 << 7,
    Inherit = 1 << 8,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[repr(i32)]
pub enum SharingMode {
    Exclusive = 0,
    Concurrent = 1,
}

flags!(ImageUsageBit, ImageUsageFlags, u32);
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[repr(i32)]
pub enum ImageUsageBit {
    TransferSrc = 1 << 0,
    TransferDst = 1 << 1,
    Sampled = 1 << 2,
    Storage = 1 << 3,
    ColorAttachment = 1 << 4,
    DepthStencilAttachment = 1 << 5,
    TransientAttachment = 1 << 6,
    InputAttachment = 1 << 7,
    HostTransfer = 1 << 22,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[repr(i32)]
pub enum ColorSpace {
    SrgbNonlinear = 0,
}

#[repr(transparent)]
pub struct Surface([u8; 0]);
flags!(SwapchainCreateBit, SwapchainCreateFlags, u32);
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[repr(i32)]
pub enum SwapchainCreateBit {
    SplitInstanceBindRegions = 1 << 0,
    Protected = 1 << 1,
    DeferredMemoryAllocation = 1 << 3,
    PresentId2 = 0x00000040,
    PresentWait2 = 0x00000080,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(C)]
pub struct SurfaceFormat {
    pub(crate) format: i32,
    pub(crate) color_space: i32,
}
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct SurfaceCapabilities {
    pub(crate) min_image_count: u32,
    pub(crate) max_image_count: u32,
    pub(crate) current_extent: Extent2D,
    pub(crate) min_image_extent: Extent2D,
    pub(crate) max_image_extent: Extent2D,
    pub(crate) max_image_array_layers: u32,
    pub(crate) supported_transforms: SurfaceTransformFlags,
    pub(crate) current_transform: i32,
    pub(crate) supported_composite_alpha: CompositeAlphaFlags,
    pub(crate) supported_usage_flags: ImageUsageFlags,
}
flags!(CompositeAlphaBit, CompositeAlphaFlags, u32);
flags!(SurfaceTransformBit, SurfaceTransformFlags, u32);
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct RenderingInputAttachmentIndexInfo {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *const c_void,
    pub(crate) color_attachment_count: u32,
    pub(crate) p_color_attachment_input_indices: *const u32,
    pub(crate) p_depth_input_attachment_index: *const u32,
    pub(crate) p_stencil_input_attachment_index: *const u32,
}
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct RenderingAttachmentLocationInfo {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *const c_void,
    pub(crate) color_attachment_count: u32,
    pub(crate) p_color_attachment_locations: *const u32,
}
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct PhysicalDeviceDynamicRenderingLocalReadFeatures {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *mut c_void,
    pub(crate) dynamic_rendering_local_read: Bool32,
}
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct RenderingAreaInfo {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *const c_void,
    pub(crate) view_mask: u32,
    pub(crate) color_attachment_count: u32,
    pub(crate) p_color_attachment_formats: *const Format,
    pub(crate) depth_attachment_format: Format,
    pub(crate) stencil_attachment_format: Format,
}
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct PhysicalDeviceVertexAttributeDivisorFeatures {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *mut c_void,
    pub(crate) vertex_attribute_instance_rate_divisor: Bool32,
    pub(crate) vertex_attribute_instance_rate_zero_divisor: Bool32,
}
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct PipelineVertexInputDivisorStateCreateInfo {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *const c_void,
    pub(crate) vertex_binding_divisor_count: u32,
    pub(crate) p_vertex_binding_divisors: *const VertexInputBindingDivisorDescription,
}
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct VertexInputBindingDivisorDescription {
    pub(crate) binding: u32,
    pub(crate) divisor: u32,
}
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct PhysicalDeviceVertexAttributeDivisorProperties {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *mut c_void,
    pub(crate) max_vertex_attrib_divisor: u32,
    pub(crate) supports_non_zero_first_instance: Bool32,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[repr(i32)]
pub enum LineRasterizationMode {
    Default = 0,
    Rectangular = 1,
    Bresenham = 2,
    RectangularSmooth = 3,
}

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct PipelineRasterizationLineStateCreateInfo {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *const c_void,
    pub(crate) line_rasterization_mode: LineRasterizationMode,
    pub(crate) stippled_line_enable: Bool32,
    pub(crate) line_stipple_factor: u32,
    pub(crate) line_stipple_pattern: u16,
}
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct PhysicalDeviceLineRasterizationProperties {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *mut c_void,
    pub(crate) line_sub_pixel_precision_bits: u32,
}
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct PhysicalDeviceLineRasterizationFeatures {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *mut c_void,
    pub(crate) rectangular_lines: Bool32,
    pub(crate) bresenham_lines: Bool32,
    pub(crate) smooth_lines: Bool32,
    pub(crate) stippled_rectangular_lines: Bool32,
    pub(crate) stippled_bresenham_lines: Bool32,
    pub(crate) stippled_smooth_lines: Bool32,
}
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct PipelineRobustnessCreateInfo {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *const c_void,
    pub(crate) storage_buffers: PipelineRobustnessBufferBehavior,
    pub(crate) uniform_buffers: PipelineRobustnessBufferBehavior,
    pub(crate) vertex_inputs: PipelineRobustnessBufferBehavior,
    pub(crate) images: PipelineRobustnessImageBehavior,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[repr(i32)]
pub enum PipelineRobustnessImageBehavior {
    DeviceDefault = 0,
    Disabled = 1,
    RobustImageAccess = 2,
    RobustImageAccess2 = 3,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[repr(i32)]
pub enum PipelineRobustnessBufferBehavior {
    DeviceDefault = 0,
    Disabled = 1,
    RobustBufferAccess = 2,
    RobustBufferAccess2 = 3,
}

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct PhysicalDevicePipelineRobustnessProperties {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *mut c_void,
    pub(crate) default_robustness_storage_buffers: PipelineRobustnessBufferBehavior,
    pub(crate) default_robustness_uniform_buffers: PipelineRobustnessBufferBehavior,
    pub(crate) default_robustness_vertex_inputs: PipelineRobustnessBufferBehavior,
    pub(crate) default_robustness_images: PipelineRobustnessImageBehavior,
}
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct PhysicalDevicePipelineRobustnessFeatures {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *mut c_void,
    pub(crate) pipeline_robustness: Bool32,
}
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct PhysicalDevicePipelineProtectedAccessFeatures {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *mut c_void,
    pub(crate) pipeline_protected_access: Bool32,
}
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct PushDescriptorSetWithTemplateInfo {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *const c_void,
    pub(crate) descriptor_update_template: *const DescriptorUpdateTemplate,
    pub(crate) layout: *const PipelineLayout,
    pub(crate) set: u32,
    pub(crate) p_data: *const c_void,
}
#[repr(transparent)]
pub struct PipelineLayout([u8; 0]);
#[repr(transparent)]
pub struct DescriptorUpdateTemplate([u8; 0]);
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct PushDescriptorSetInfo {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *const c_void,
    pub(crate) stage_flags: ShaderStageFlags,
    pub(crate) layout: *const PipelineLayout,
    pub(crate) set: u32,
    pub(crate) descriptor_write_count: u32,
    pub(crate) p_descriptor_writes: *const WriteDescriptorSet,
}
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct WriteDescriptorSet {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *const c_void,
    pub(crate) dst_set: *const DescriptorSet,
    pub(crate) dst_binding: u32,
    pub(crate) dst_array_element: u32,
    pub(crate) descriptor_count: u32,
    pub(crate) descriptor_type: DescriptorType,
    pub(crate) p_image_info: *const DescriptorImageInfo,
    pub(crate) p_buffer_info: *const DescriptorBufferInfo,
    pub(crate) p_texel_buffer_view: *const *const BufferView,
}
#[repr(transparent)]
pub struct BufferView([u8; 0]);
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct DescriptorBufferInfo {
    pub(crate) buffer: *const Buffer,
    pub(crate) offset: DeviceSize,
    pub(crate) range: DeviceSize,
}
#[repr(transparent)]
pub struct Buffer([u8; 0]);
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct DescriptorImageInfo {
    pub(crate) sampler: *const Sampler,
    pub(crate) image_view: *const ImageView,
    pub(crate) image_layout: ImageLayout,
}
#[repr(transparent)]
pub struct ImageView([u8; 0]);
#[repr(transparent)]
pub struct Sampler([u8; 0]);
#[repr(transparent)]
pub struct DescriptorSet([u8; 0]);
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct PushConstantsInfo {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *const c_void,
    pub(crate) layout: *const PipelineLayout,
    pub(crate) stage_flags: ShaderStageFlags,
    pub(crate) offset: u32,
    pub(crate) size: u32,
    pub(crate) p_values: *const c_void,
}
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct BindDescriptorSetsInfo {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *const c_void,
    pub(crate) stage_flags: ShaderStageFlags,
    pub(crate) layout: *const PipelineLayout,
    pub(crate) first_set: u32,
    pub(crate) descriptor_set_count: u32,
    pub(crate) p_descriptor_sets: *const *const DescriptorSet,
    pub(crate) dynamic_offset_count: u32,
    pub(crate) p_dynamic_offsets: *const u32,
}
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct PhysicalDevicePushDescriptorProperties {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *mut c_void,
    pub(crate) max_push_descriptors: u32,
}
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct PipelineCreateFlags2CreateInfo {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *const c_void,
    pub(crate) flags: PipelineCreateFlags2,
}
flags!(PipelineCreateBit2, PipelineCreateFlags2, u64);
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[repr(i64)]
pub enum PipelineCreateBit2 {
    DisableOptimization = 1 << 0,
    AllowDerivatives = 1 << 1,
    Derivative = 1 << 2,
    ViewIndexFromDeviceIndex = 1 << 3,
    DispatchBase = 1 << 4,
    FailOnPipelineCompileRequired = 1 << 8,
    EarlyReturnOnFailure = 1 << 9,
    NoProtectedAccess = 1 << 27,
    ProtectedAccessOnly = 1 << 30,
    DescriptorHeap = 1 << 36,
}

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct PhysicalDevicePresentId2Features {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *mut c_void,
    pub(crate) present_id2: Bool32,
}
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct PhysicalDevicePresentWait2Features {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *mut c_void,
    pub(crate) present_wait2: Bool32,
}
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct SurfaceCapabilitiesPresentId2 {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *mut c_void,
    pub(crate) present_id2_supported: Bool32,
}
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct SurfaceCapabilitiesPresentWait2 {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *mut c_void,
    pub(crate) present_wait2_supported: Bool32,
}
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct PresentId2 {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *mut c_void,
    pub(crate) swapchain_count: u32,
    pub(crate) present_ids: *const u64,
}
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct PresentWait2Info {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *mut c_void,
    pub(crate) present_id: u64,
    pub(crate) timeout: u64,
}
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct PhysicalDeviceShaderExpectAssumeFeatures {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *mut c_void,
    pub(crate) shader_expect_assume: Bool32,
}
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct PhysicalDeviceShaderFloatControls2Features {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *mut c_void,
    pub(crate) shader_float_controls_2: Bool32,
}
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct PhysicalDeviceShaderSubgroupRotateFeatures {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *mut c_void,
    pub(crate) shader_subgroup_rotate: Bool32,
    pub(crate) shader_subgroup_rotate_clustered: Bool32,
}
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct HostImageCopyDevicePerformanceQuery {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *mut c_void,
    pub(crate) optimal_device_access: Bool32,
    pub(crate) identical_memory_layout: Bool32,
}
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct SubresourceHostMemcpySize {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *mut c_void,
    pub(crate) size: DeviceSize,
}
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct HostImageLayoutTransitionInfo {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *const c_void,
    pub(crate) image: *const Image,
    pub(crate) old_layout: ImageLayout,
    pub(crate) new_layout: ImageLayout,
    pub(crate) subresource_range: ImageSubresourceRange,
}
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct CopyImageToImageInfo {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *const c_void,
    pub(crate) flags: HostImageCopyFlags,
    pub(crate) src_image: *const Image,
    pub(crate) src_image_layout: ImageLayout,
    pub(crate) dst_image: *const Image,
    pub(crate) dst_image_layout: ImageLayout,
    pub(crate) region_count: u32,
    pub(crate) p_regions: *const ImageCopy2,
}
pub type ImageCopy2 = crate::ImageCopy;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(C)]
pub struct Extent3D {
    pub width: u32,
    pub height: u32,
    pub depth: u32,
}
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct Offset3D {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct ImageSubresourceLayers {
    pub(crate) aspect_mask: ImageAspectFlags,
    pub(crate) mip_level: u32,
    pub(crate) base_array_layer: u32,
    pub(crate) layer_count: u32,
}
flags!(HostImageCopyBit, HostImageCopyFlags, u32);
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[repr(i32)]
pub enum HostImageCopyBit {
    Memcpy = 1 << 0,
}

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct CopyImageToMemoryInfo {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *const c_void,
    pub(crate) flags: HostImageCopyFlags,
    pub(crate) src_image: *const Image,
    pub(crate) src_image_layout: ImageLayout,
    pub(crate) region_count: u32,
    pub(crate) p_regions: *const ImageToMemoryCopy,
}
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct ImageToMemoryCopy {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *const c_void,
    pub(crate) p_host_pointer: *mut c_void,
    pub(crate) memory_row_length: u32,
    pub(crate) memory_image_height: u32,
    pub(crate) image_subresource: ImageSubresourceLayers,
    pub(crate) image_offset: Offset3D,
    pub(crate) image_extent: Extent3D,
}
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct CopyMemoryToImageInfo {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *const c_void,
    pub(crate) flags: HostImageCopyFlags,
    pub(crate) dst_image: *const Image,
    pub(crate) dst_image_layout: ImageLayout,
    pub(crate) region_count: u32,
    pub(crate) p_regions: *const MemoryToImageCopy,
}
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct MemoryToImageCopy {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *const c_void,
    pub(crate) p_host_pointer: *const c_void,
    pub(crate) memory_row_length: u32,
    pub(crate) memory_image_height: u32,
    pub(crate) image_subresource: ImageSubresourceLayers,
    pub(crate) image_offset: Offset3D,
    pub(crate) image_extent: Extent3D,
}
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct PhysicalDeviceHostImageCopyProperties {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *mut c_void,
    pub(crate) copy_src_layout_count: u32,
    pub(crate) p_copy_src_layouts: *mut ImageLayout,
    pub(crate) copy_dst_layout_count: u32,
    pub(crate) p_copy_dst_layouts: *mut ImageLayout,
    pub(crate) optimal_tiling_layout_uuid: [u8; VK_UUID_SIZE as usize],
    pub(crate) identical_memory_type_requirements: Bool32,
}
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct PhysicalDeviceHostImageCopyFeatures {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *mut c_void,
    pub(crate) host_image_copy: Bool32,
}
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct BindMemoryStatus {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *const c_void,
    pub(crate) p_result: *mut Result,
}
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct PhysicalDeviceMaintenance6Properties {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *mut c_void,
    pub(crate) block_texel_view_compatible_multiple_layers: Bool32,
    pub(crate) max_combined_image_sampler_descriptor_count: u32,
    pub(crate) fragment_shading_rate_clamp_combiner_inputs: Bool32,
}
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct PhysicalDeviceMaintenance6Features {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *mut c_void,
    pub(crate) maintenance_6: Bool32,
}
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct BufferUsageFlags2CreateInfo {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *const c_void,
    pub(crate) usage: BufferUsageFlags2,
}
flags!(BufferUsageBit2, BufferUsageFlags2, u64);
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[repr(i64)]
pub enum BufferUsageBit2 {
    TransferSrc = 1 << 0,
    TransferDst = 1 << 1,
    UniformTexelBuffer = 1 << 2,
    StorageTexelBuffer = 1 << 3,
    UniformBuffer = 1 << 4,
    StorageBuffer = 1 << 5,
    IndexBuffer = 1 << 6,
    VertexBuffer = 1 << 7,
    IndirectBuffer = 1 << 8,
    ShaderDeviceAddress = 1 << 17,
    DescriptorHeap = 1 << 28,
}

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct DeviceImageSubresourceInfo {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *const c_void,
    pub(crate) p_create_info: *const ImageCreateInfo,
    pub(crate) p_subresource: *const ImageSubresource2,
}
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct ImageSubresource2 {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *mut c_void,
    pub(crate) image_subresource: ImageSubresource,
}
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct ImageSubresource {
    pub(crate) aspect_mask: ImageAspectFlags,
    pub(crate) mip_level: u32,
    pub(crate) array_layer: u32,
}
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct ImageCreateInfo {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *const c_void,
    pub(crate) flags: ImageCreateFlags,
    pub(crate) image_type: ImageType,
    pub(crate) format: Format,
    pub(crate) extent: Extent3D,
    pub(crate) mip_levels: u32,
    pub(crate) array_layers: u32,
    pub(crate) samples: SampleCountBit,
    pub(crate) tiling: ImageTiling,
    pub(crate) usage: ImageUsageFlags,
    pub(crate) sharing_mode: SharingMode,
    pub(crate) queue_family_index_count: u32,
    pub(crate) p_queue_family_indices: *const u32,
    pub(crate) initial_layout: ImageLayout,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[repr(i32)]
pub enum ImageTiling {
    Optimal = 0,
    Linear = 1,
    DrmFormatModifier = 1000158000,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[repr(i32)]
pub enum SampleCountBit {
    _1 = 1 << 0,
    _2 = 1 << 1,
    _4 = 1 << 2,
    _8 = 1 << 3,
    _16 = 1 << 4,
    _32 = 1 << 5,
    _64 = 1 << 6,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[repr(i32)]
pub enum ImageType {
    _1D = 0,
    _2D = 1,
    _3D = 2,
}

flags!(ImageCreateBit, ImageCreateFlags, u32);
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[repr(i32)]
pub enum ImageCreateBit {
    SparseBinding = 1 << 0,
    SparseResidency = 1 << 1,
    SparseAliased = 1 << 2,
    MutableFormat = 1 << 3,
    CubeCompatible = 1 << 4,
    Alias = 1 << 10,
    SplitInstanceBindRegions = 1 << 6,
    _2DArrayCompatible = 1 << 5,
    BlockTexelViewCompatible = 1 << 7,
    ExtendedUsage = 1 << 8,
    Protected = 1 << 11,
    Disjoint = 1 << 9,
    DescriptorHeapCaptureReplay = 1 << 16,
}

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct SubresourceLayout2 {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *mut c_void,
    pub(crate) subresource_layout: SubresourceLayout,
}
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct SubresourceLayout {
    pub(crate) offset: DeviceSize,
    pub(crate) size: DeviceSize,
    pub(crate) row_pitch: DeviceSize,
    pub(crate) array_pitch: DeviceSize,
    pub(crate) depth_pitch: DeviceSize,
}
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct PhysicalDeviceMaintenance5Properties {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *mut c_void,
    pub(crate) early_fragment_multisample_coverage_after_sample_counting: Bool32,
    pub(crate) early_fragment_sample_mask_test_before_sample_counting: Bool32,
    pub(crate) depth_stencil_swizzle_one_support: Bool32,
    pub(crate) polygon_mode_point_size: Bool32,
    pub(crate) non_strict_single_pixel_wide_lines_use_parallelogram: Bool32,
    pub(crate) non_strict_wide_lines_use_parallelogram: Bool32,
}
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct PhysicalDeviceMaintenance5Features {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *mut c_void,
    pub(crate) maintenance_5: Bool32,
}
flags!(MemoryUnmapFlags, u32);
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct MemoryUnmapInfo {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *const c_void,
    pub(crate) flags: MemoryUnmapFlags,
    pub(crate) memory: *const DeviceMemory,
}
#[repr(transparent)]
pub struct DeviceMemory([u8; 0]);
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct MemoryMapInfo {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *const c_void,
    pub(crate) flags: MemoryMapFlags,
    pub(crate) memory: *const DeviceMemory,
    pub(crate) offset: DeviceSize,
    pub(crate) size: DeviceSize,
}
flags!(MemoryMapFlags, u32);
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct PhysicalDeviceIndexTypeUint8Features {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *mut c_void,
    pub(crate) index_type_uint_8: Bool32,
}
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct QueueFamilyGlobalPriorityProperties {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *mut c_void,
    pub(crate) priority_count: u32,
    pub(crate) priorities: [QueueGlobalPriority; VK_MAX_GLOBAL_PRIORITY_SIZE as usize],
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[repr(i32)]
pub enum QueueGlobalPriority {
    Low = 128,
    Medium = 256,
    High = 512,
    Realtime = 1024,
}

pub const VK_MAX_GLOBAL_PRIORITY_SIZE: u32 = 16;
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct PhysicalDeviceGlobalPriorityQueryFeatures {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *mut c_void,
    pub(crate) global_priority_query: Bool32,
}
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct DeviceQueueGlobalPriorityCreateInfo {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *const c_void,
    pub(crate) global_priority: QueueGlobalPriority,
}
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct PhysicalDeviceVulkan14Properties {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *mut c_void,
    pub(crate) line_sub_pixel_precision_bits: u32,
    pub(crate) max_vertex_attrib_divisor: u32,
    pub(crate) supports_non_zero_first_instance: Bool32,
    pub(crate) max_push_descriptors: u32,
    pub(crate) dynamic_rendering_local_read_depth_stencil_attachments: Bool32,
    pub(crate) dynamic_rendering_local_read_multisampled_attachments: Bool32,
    pub(crate) early_fragment_multisample_coverage_after_sample_counting: Bool32,
    pub(crate) early_fragment_sample_mask_test_before_sample_counting: Bool32,
    pub(crate) depth_stencil_swizzle_one_support: Bool32,
    pub(crate) polygon_mode_point_size: Bool32,
    pub(crate) non_strict_single_pixel_wide_lines_use_parallelogram: Bool32,
    pub(crate) non_strict_wide_lines_use_parallelogram: Bool32,
    pub(crate) block_texel_view_compatible_multiple_layers: Bool32,
    pub(crate) max_combined_image_sampler_descriptor_count: u32,
    pub(crate) fragment_shading_rate_clamp_combiner_inputs: Bool32,
    pub(crate) default_robustness_storage_buffers: PipelineRobustnessBufferBehavior,
    pub(crate) default_robustness_uniform_buffers: PipelineRobustnessBufferBehavior,
    pub(crate) default_robustness_vertex_inputs: PipelineRobustnessBufferBehavior,
    pub(crate) default_robustness_images: PipelineRobustnessImageBehavior,
    pub(crate) copy_src_layout_count: u32,
    pub(crate) p_copy_src_layouts: *mut ImageLayout,
    pub(crate) copy_dst_layout_count: u32,
    pub(crate) p_copy_dst_layouts: *mut ImageLayout,
    pub(crate) optimal_tiling_layout_uuid: [u8; VK_UUID_SIZE as usize],
    pub(crate) identical_memory_type_requirements: Bool32,
}
#[derive(Clone, Copy, Debug, Default)]
#[repr(C)]
pub struct PhysicalDeviceVulkan14Features {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *mut c_void,
    pub(crate) global_priority_query: Bool32,
    pub(crate) shader_subgroup_rotate: Bool32,
    pub(crate) shader_subgroup_rotate_clustered: Bool32,
    pub(crate) shader_float_controls_2: Bool32,
    pub(crate) shader_expect_assume: Bool32,
    pub(crate) rectangular_lines: Bool32,
    pub(crate) bresenham_lines: Bool32,
    pub(crate) smooth_lines: Bool32,
    pub(crate) stippled_rectangular_lines: Bool32,
    pub(crate) stippled_bresenham_lines: Bool32,
    pub(crate) stippled_smooth_lines: Bool32,
    pub(crate) vertex_attribute_instance_rate_divisor: Bool32,
    pub(crate) vertex_attribute_instance_rate_zero_divisor: Bool32,
    pub(crate) index_type_uint_8: Bool32,
    pub(crate) dynamic_rendering_local_read: Bool32,
    pub(crate) maintenance_5: Bool32,
    pub(crate) maintenance_6: Bool32,
    pub(crate) pipeline_protected_access: Bool32,
    pub(crate) pipeline_robustness: Bool32,
    pub(crate) host_image_copy: Bool32,
    pub(crate) push_descriptor: Bool32,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[repr(i32)]
pub enum RenderingBit {
    ContentsSecondaryCommandBuffers = 1 << 0,
    Suspending = 1 << 1,
    Resuming = 1 << 2,
}

flags!(RenderingBit, RenderingFlags, u32);
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct CommandBufferInheritanceRenderingInfo {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *const c_void,
    pub(crate) flags: RenderingFlags,
    pub(crate) view_mask: u32,
    pub(crate) color_attachment_count: u32,
    pub(crate) p_color_attachment_formats: *const Format,
    pub(crate) depth_attachment_format: Format,
    pub(crate) stencil_attachment_format: Format,
    pub(crate) rasterization_samples: SampleCountBit,
}
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct PhysicalDeviceDynamicRenderingFeatures {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *mut c_void,
    pub(crate) dynamic_rendering: Bool32,
}
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct PipelineRenderingCreateInfo {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *const c_void,
    pub(crate) view_mask: u32,
    pub(crate) color_attachment_count: u32,
    pub(crate) p_color_attachment_formats: *const Format,
    pub(crate) depth_attachment_format: Format,
    pub(crate) stencil_attachment_format: Format,
}
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct RenderingInfo {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *const c_void,
    pub(crate) flags: RenderingFlags,
    pub(crate) render_area: Rect2D,
    pub(crate) layer_count: u32,
    pub(crate) view_mask: u32,
    pub(crate) color_attachment_count: u32,
    pub(crate) p_color_attachments: *const RenderingAttachmentInfo,
    pub(crate) p_depth_attachment: *const RenderingAttachmentInfo,
    pub(crate) p_stencil_attachment: *const RenderingAttachmentInfo,
}
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct RenderingAttachmentInfo {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *const c_void,
    pub(crate) image_view: *const ImageView,
    pub(crate) image_layout: ImageLayout,
    pub(crate) resolve_mode: ResolveModeFlags,
    pub(crate) resolve_image_view: *const ImageView,
    pub(crate) resolve_image_layout: ImageLayout,
    pub(crate) load_op: AttachmentLoadOp,
    pub(crate) store_op: AttachmentStoreOp,
    pub(crate) clear_value: ClearValue,
}
impl std::fmt::Debug for ClearValue {
    fn fmt(&self, _writer: &mut std::fmt::Formatter) -> std::result::Result<(), std::fmt::Error> {
        Ok(())
    }
}
#[derive(Clone, Copy)]
#[repr(C)]
pub union ClearValue {
    pub(crate) color: ClearColorValue,
    pub(crate) depth_stencil: ClearDepthStencilValue,
}
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct ClearDepthStencilValue {
    pub(crate) depth: f32,
    pub(crate) stencil: u32,
}
impl std::fmt::Debug for ClearColorValue {
    fn fmt(&self, _writer: &mut std::fmt::Formatter) -> std::result::Result<(), std::fmt::Error> {
        Ok(())
    }
}
#[derive(Clone, Copy)]
#[repr(C)]
pub union ClearColorValue {
    pub(crate) float_32: [f32; 4 as usize],
    pub(crate) int_32: [i32; 4 as usize],
    pub(crate) uint_32: [u32; 4 as usize],
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[repr(i32)]
pub enum AttachmentStoreOp {
    Store = 0,
    DontCare = 1,
    None = 1000301000,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[repr(i32)]
pub enum AttachmentLoadOp {
    Load = 0,
    Clear = 1,
    DontCare = 2,
    None = 1000400000,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[repr(i32)]
pub enum ResolveModeBit {
    SampleZero = 1 << 0,
    Average = 1 << 1,
    Min = 1 << 2,
    Max = 1 << 3,
}

impl ResolveModeFlags {
    pub const NONE: Self = Self(0);
}
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct Rect2D {
    pub offset: Offset2D,
    pub extent: Extent2D,
}
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct Offset2D {
    pub x: i32,
    pub y: i32,
}
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct ResolveImageInfo2 {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *const c_void,
    pub(crate) src_image: *const Image,
    pub(crate) src_image_layout: ImageLayout,
    pub(crate) dst_image: *const Image,
    pub(crate) dst_image_layout: ImageLayout,
    pub(crate) region_count: u32,
    pub(crate) p_regions: *const ImageResolve2,
}
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct ImageResolve2 {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *const c_void,
    pub(crate) src_subresource: ImageSubresourceLayers,
    pub(crate) src_offset: Offset3D,
    pub(crate) dst_subresource: ImageSubresourceLayers,
    pub(crate) dst_offset: Offset3D,
    pub(crate) extent: Extent3D,
}
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct BlitImageInfo2 {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *const c_void,
    pub(crate) src_image: *const Image,
    pub(crate) src_image_layout: ImageLayout,
    pub(crate) dst_image: *const Image,
    pub(crate) dst_image_layout: ImageLayout,
    pub(crate) region_count: u32,
    pub(crate) p_regions: *const ImageBlit2,
    pub(crate) filter: Filter,
}
pub type ImageBlit2 = crate::ImageBlit;
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct PhysicalDeviceTexelBufferAlignmentProperties {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *mut c_void,
    pub(crate) storage_texel_buffer_offset_alignment_bytes: DeviceSize,
    pub(crate) storage_texel_buffer_offset_single_texel_alignment: Bool32,
    pub(crate) uniform_texel_buffer_offset_alignment_bytes: DeviceSize,
    pub(crate) uniform_texel_buffer_offset_single_texel_alignment: Bool32,
}
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct PhysicalDeviceShaderIntegerDotProductProperties {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *mut c_void,
    pub(crate) integer_dot_product_8_bit_unsigned_accelerated: Bool32,
    pub(crate) integer_dot_product_8_bit_signed_accelerated: Bool32,
    pub(crate) integer_dot_product_8_bit_mixed_signedness_accelerated: Bool32,
    pub(crate) integer_dot_product_4_x_8_bit_packed_unsigned_accelerated: Bool32,
    pub(crate) integer_dot_product_4_x_8_bit_packed_signed_accelerated: Bool32,
    pub(crate) integer_dot_product_4_x_8_bit_packed_mixed_signedness_accelerated: Bool32,
    pub(crate) integer_dot_product_16_bit_unsigned_accelerated: Bool32,
    pub(crate) integer_dot_product_16_bit_signed_accelerated: Bool32,
    pub(crate) integer_dot_product_16_bit_mixed_signedness_accelerated: Bool32,
    pub(crate) integer_dot_product_32_bit_unsigned_accelerated: Bool32,
    pub(crate) integer_dot_product_32_bit_signed_accelerated: Bool32,
    pub(crate) integer_dot_product_32_bit_mixed_signedness_accelerated: Bool32,
    pub(crate) integer_dot_product_64_bit_unsigned_accelerated: Bool32,
    pub(crate) integer_dot_product_64_bit_signed_accelerated: Bool32,
    pub(crate) integer_dot_product_64_bit_mixed_signedness_accelerated: Bool32,
    pub(crate) integer_dot_product_accumulating_saturating_8_bit_unsigned_accelerated: Bool32,
    pub(crate) integer_dot_product_accumulating_saturating_8_bit_signed_accelerated: Bool32,
    pub(crate) integer_dot_product_accumulating_saturating_8_bit_mixed_signedness_accelerated:
        Bool32,
    pub(crate) integer_dot_product_accumulating_saturating_4_x_8_bit_packed_unsigned_accelerated:
        Bool32,
    pub(crate) integer_dot_product_accumulating_saturating_4_x_8_bit_packed_signed_accelerated:
        Bool32,
    pub(crate) integer_dot_product_accumulating_saturating_4_x_8_bit_packed_mixed_signedness_accelerated:
        Bool32,
    pub(crate) integer_dot_product_accumulating_saturating_16_bit_unsigned_accelerated: Bool32,
    pub(crate) integer_dot_product_accumulating_saturating_16_bit_signed_accelerated: Bool32,
    pub(crate) integer_dot_product_accumulating_saturating_16_bit_mixed_signedness_accelerated:
        Bool32,
    pub(crate) integer_dot_product_accumulating_saturating_32_bit_unsigned_accelerated: Bool32,
    pub(crate) integer_dot_product_accumulating_saturating_32_bit_signed_accelerated: Bool32,
    pub(crate) integer_dot_product_accumulating_saturating_32_bit_mixed_signedness_accelerated:
        Bool32,
    pub(crate) integer_dot_product_accumulating_saturating_64_bit_unsigned_accelerated: Bool32,
    pub(crate) integer_dot_product_accumulating_saturating_64_bit_signed_accelerated: Bool32,
    pub(crate) integer_dot_product_accumulating_saturating_64_bit_mixed_signedness_accelerated:
        Bool32,
}
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct PhysicalDeviceShaderIntegerDotProductFeatures {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *mut c_void,
    pub(crate) shader_integer_dot_product: Bool32,
}
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct DescriptorPoolInlineUniformBlockCreateInfo {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *const c_void,
    pub(crate) max_inline_uniform_block_bindings: u32,
}
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct WriteDescriptorSetInlineUniformBlock {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *const c_void,
    pub(crate) data_size: u32,
    pub(crate) p_data: *const c_void,
}
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct PhysicalDeviceInlineUniformBlockProperties {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *mut c_void,
    pub(crate) max_inline_uniform_block_size: u32,
    pub(crate) max_per_stage_descriptor_inline_uniform_blocks: u32,
    pub(crate) max_per_stage_descriptor_update_after_bind_inline_uniform_blocks: u32,
    pub(crate) max_descriptor_set_inline_uniform_blocks: u32,
    pub(crate) max_descriptor_set_update_after_bind_inline_uniform_blocks: u32,
}
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct PhysicalDeviceInlineUniformBlockFeatures {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *mut c_void,
    pub(crate) inline_uniform_block: Bool32,
    pub(crate) descriptor_binding_inline_uniform_block_update_after_bind: Bool32,
}
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct PhysicalDeviceSubgroupSizeControlProperties {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *mut c_void,
    pub(crate) min_subgroup_size: u32,
    pub(crate) max_subgroup_size: u32,
    pub(crate) max_compute_workgroup_subgroups: u32,
    pub(crate) required_subgroup_size_stages: ShaderStageFlags,
}
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct PhysicalDeviceSubgroupSizeControlFeatures {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *mut c_void,
    pub(crate) subgroup_size_control: Bool32,
    pub(crate) compute_full_subgroups: Bool32,
}
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct PhysicalDeviceImageRobustnessFeatures {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *mut c_void,
    pub(crate) robust_image_access: Bool32,
}
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct PhysicalDeviceZeroInitializeWorkgroupMemoryFeatures {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *mut c_void,
    pub(crate) shader_zero_initialize_workgroup_memory: Bool32,
}
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct PhysicalDevicePipelineCreationCacheControlFeatures {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *mut c_void,
    pub(crate) pipeline_creation_cache_control: Bool32,
}
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct PhysicalDeviceShaderDemoteToHelperInvocationFeatures {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *mut c_void,
    pub(crate) shader_demote_to_helper_invocation: Bool32,
}
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct PhysicalDeviceShaderTerminateInvocationFeatures {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *mut c_void,
    pub(crate) shader_terminate_invocation: Bool32,
}
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct PipelineCreationFeedbackCreateInfo {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *const c_void,
    pub(crate) p_pipeline_creation_feedback: *mut PipelineCreationFeedback,
    pub(crate) pipeline_stage_creation_feedback_count: u32,
    pub(crate) p_pipeline_stage_creation_feedbacks: *mut PipelineCreationFeedback,
}
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct PipelineCreationFeedback {
    pub(crate) flags: PipelineCreationFeedbackFlags,
    pub(crate) duration: u64,
}
flags!(
    PipelineCreationFeedbackBit,
    PipelineCreationFeedbackFlags,
    u32
);
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[repr(i32)]
pub enum PipelineCreationFeedbackBit {
    Valid = 1 << 0,
    ApplicationPipelineCacheHit = 1 << 1,
    BasePipelineAcceleration = 1 << 2,
}

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct DeviceImageMemoryRequirements {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *const c_void,
    pub(crate) p_create_info: *const ImageCreateInfo,
    pub(crate) plane_aspect: ImageAspectBit,
}
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct DeviceBufferMemoryRequirements {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *const c_void,
    pub(crate) p_create_info: *const BufferCreateInfo,
}
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct BufferCreateInfo {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *const c_void,
    pub(crate) flags: BufferCreateFlags,
    pub(crate) size: DeviceSize,
    pub(crate) usage: BufferUsageFlags,
    pub(crate) sharing_mode: SharingMode,
    pub(crate) queue_family_index_count: u32,
    pub(crate) p_queue_family_indices: *const u32,
}
flags!(BufferUsageBit, BufferUsageFlags, u32);
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[repr(i32)]
pub enum BufferUsageBit {
    TransferSrc = 1 << 0,
    TransferDst = 1 << 1,
    UniformTexelBuffer = 1 << 2,
    StorageTexelBuffer = 1 << 3,
    UniformBuffer = 1 << 4,
    StorageBuffer = 1 << 5,
    IndexBuffer = 1 << 6,
    VertexBuffer = 1 << 7,
    IndirectBuffer = 1 << 8,
    ShaderDeviceAddress = 1 << 17,
    DescriptorHeap = 1 << 28,
}

flags!(BufferCreateBit, BufferCreateFlags, u32);
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[repr(i32)]
pub enum BufferCreateBit {
    SparseBinding = 1 << 0,
    SparseResidency = 1 << 1,
    SparseAliased = 1 << 2,
    Protected = 1 << 3,
    DeviceAddressCaptureReplay = 1 << 4,
}

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct PhysicalDeviceMaintenance4Properties {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *mut c_void,
    pub(crate) max_buffer_size: DeviceSize,
}
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct PhysicalDeviceMaintenance4Features {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *mut c_void,
    pub(crate) maintenance_4: Bool32,
}
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct FormatProperties3 {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *mut c_void,
    pub(crate) linear_tiling_features: FormatFeatureFlags2,
    pub(crate) optimal_tiling_features: FormatFeatureFlags2,
    pub(crate) buffer_features: FormatFeatureFlags2,
}
flags!(FormatFeatureBit2, FormatFeatureFlags2, u64);
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[repr(i64)]
pub enum FormatFeatureBit2 {
    SampledImage = 1 << 0,
    StorageImage = 1 << 1,
    StorageImageAtomic = 1 << 2,
    UniformTexelBuffer = 1 << 3,
    StorageTexelBuffer = 1 << 4,
    StorageTexelBufferAtomic = 1 << 5,
    VertexBuffer = 1 << 6,
    ColorAttachment = 1 << 7,
    ColorAttachmentBlend = 1 << 8,
    DepthStencilAttachment = 1 << 9,
    BlitSrc = 1 << 10,
    BlitDst = 1 << 11,
    SampledImageFilterLinear = 1 << 12,
    TransferSrc = 1 << 14,
    TransferDst = 1 << 15,
    SampledImageFilterMinmax = 1 << 16,
    MidpointChromaSamples = 1 << 17,
    SampledImageYcbcrConversionLinearFilter = 1 << 18,
    SampledImageYcbcrConversionSeparateReconstructionFilter = 1 << 19,
    SampledImageYcbcrConversionChromaReconstructionExplicit = 1 << 20,
    SampledImageYcbcrConversionChromaReconstructionExplicitForceable = 1 << 21,
    Disjoint = 1 << 22,
    CositedChromaSamples = 1 << 23,
    StorageReadWithoutFormat = 1 << 31,
    StorageWriteWithoutFormat = 1 << 32,
    SampledImageDepthComparison = 1 << 33,
    SampledImageFilterCubic = 1 << 13,
    HostImageTransfer = 1 << 46,
}

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct PhysicalDeviceTextureCompressionASTCHDRFeatures {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *mut c_void,
    pub(crate) texture_compression_astc_hdr: Bool32,
}
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct CopyImageToBufferInfo2 {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *const c_void,
    pub(crate) src_image: *const Image,
    pub(crate) src_image_layout: ImageLayout,
    pub(crate) dst_buffer: *const Buffer,
    pub(crate) region_count: u32,
    pub(crate) p_regions: *const BufferImageCopy2,
}

type BufferImageCopy2 = crate::BufferImageCopy;

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct CopyBufferToImageInfo2 {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *const c_void,
    pub(crate) src_buffer: *const Buffer,
    pub(crate) dst_image: *const Image,
    pub(crate) dst_image_layout: ImageLayout,
    pub(crate) region_count: u32,
    pub(crate) p_regions: *const BufferImageCopy2,
}
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct CopyImageInfo2 {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *const c_void,
    pub(crate) src_image: *const Image,
    pub(crate) src_image_layout: ImageLayout,
    pub(crate) dst_image: *const Image,
    pub(crate) dst_image_layout: ImageLayout,
    pub(crate) region_count: u32,
    pub(crate) p_regions: *const ImageCopy2,
}
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct CopyBufferInfo2 {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *const c_void,
    pub(crate) src_buffer: *const Buffer,
    pub(crate) dst_buffer: *const Buffer,
    pub(crate) region_count: u32,
    pub(crate) p_regions: *const BufferCopy2,
}

pub type BufferCopy2 = crate::BufferCopy;

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct PhysicalDeviceSynchronization2Features {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *mut c_void,
    pub(crate) synchronization_2: Bool32,
}

pub type SubmitInfo2 = crate::SubmitInfo;

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct SemaphoreSubmitInfo {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *const c_void,
    pub(crate) semaphore: *const Semaphore,
    pub(crate) value: u64,
    pub(crate) stage_mask: PipelineStageFlags2,
    pub(crate) device_index: u32,
}

pub type PipelineStageFlags2 = crate::PipelineStageFlags;
pub type PipelineStageBit2 = crate::PipelineStageBit;

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct CommandBufferSubmitInfo {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *const c_void,
    pub(crate) command_buffer: *const CommandBuffer,
    pub(crate) device_mask: u32,
}
#[repr(transparent)]
pub struct CommandBuffer([u8; 0]);
flags!(SubmitBit, SubmitFlags, u32);
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[repr(i32)]
pub enum SubmitBit {
    Protected = 1 << 0,
}

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct DependencyInfo {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *const c_void,
    pub(crate) dependency_flags: DependencyFlags,
    pub(crate) memory_barrier_count: u32,
    pub(crate) p_memory_barriers: *const MemoryBarrier2,
    pub(crate) buffer_memory_barrier_count: u32,
    pub(crate) p_buffer_memory_barriers: *const BufferMemoryBarrier2,
    pub(crate) image_memory_barrier_count: u32,
    pub(crate) p_image_memory_barriers: *const ImageMemoryBarrier2,
}

pub type ImageMemoryBarrier2 = crate::ImageMemoryBarrier;

pub type AccessFlags2 = crate::AccessFlags;
pub type AccessBit2 = crate::AccessBit;

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct BufferMemoryBarrier2 {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *const c_void,
    pub(crate) src_stage_mask: PipelineStageFlags2,
    pub(crate) src_access_mask: AccessFlags2,
    pub(crate) dst_stage_mask: PipelineStageFlags2,
    pub(crate) dst_access_mask: AccessFlags2,
    pub(crate) src_queue_family_index: u32,
    pub(crate) dst_queue_family_index: u32,
    pub(crate) buffer: *const Buffer,
    pub(crate) offset: DeviceSize,
    pub(crate) size: DeviceSize,
}

pub type MemoryBarrier2 = crate::MemoryBarrier;

flags!(DependencyBit, DependencyFlags, u32);
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[repr(i32)]
pub enum DependencyBit {
    ByRegion = 1 << 0,
    DeviceGroup = 1 << 2,
    ViewLocal = 1 << 1,
}

flags!(PrivateDataSlotCreateFlags, u32);
#[repr(transparent)]
pub struct PrivateDataSlot([u8; 0]);
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct PrivateDataSlotCreateInfo {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *const c_void,
    pub(crate) flags: PrivateDataSlotCreateFlags,
}
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct DevicePrivateDataCreateInfo {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *const c_void,
    pub(crate) private_data_slot_request_count: u32,
}
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct PhysicalDevicePrivateDataFeatures {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *mut c_void,
    pub(crate) private_data: Bool32,
}
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct PhysicalDeviceToolProperties {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *mut c_void,
    pub(crate) name: [c_char; VK_MAX_EXTENSION_NAME_SIZE as usize],
    pub(crate) version: [c_char; VK_MAX_EXTENSION_NAME_SIZE as usize],
    pub(crate) purposes: ToolPurposeFlags,
    pub(crate) description: [c_char; VK_MAX_DESCRIPTION_SIZE as usize],
    pub(crate) layer: [c_char; VK_MAX_EXTENSION_NAME_SIZE as usize],
}
flags!(ToolPurposeBit, ToolPurposeFlags, u32);
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[repr(i32)]
pub enum ToolPurposeBit {
    Validation = 1 << 0,
    Profiling = 1 << 1,
    Tracing = 1 << 2,
    AdditionalFeatures = 1 << 3,
    ModifyingFeatures = 1 << 4,
}

pub const VK_MAX_EXTENSION_NAME_SIZE: u32 = 256;
pub const VK_MAX_DESCRIPTION_SIZE: u32 = 256;
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct PhysicalDeviceVulkan13Properties {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *mut c_void,
    pub(crate) min_subgroup_size: u32,
    pub(crate) max_subgroup_size: u32,
    pub(crate) max_compute_workgroup_subgroups: u32,
    pub(crate) required_subgroup_size_stages: ShaderStageFlags,
    pub(crate) max_inline_uniform_block_size: u32,
    pub(crate) max_per_stage_descriptor_inline_uniform_blocks: u32,
    pub(crate) max_per_stage_descriptor_update_after_bind_inline_uniform_blocks: u32,
    pub(crate) max_descriptor_set_inline_uniform_blocks: u32,
    pub(crate) max_descriptor_set_update_after_bind_inline_uniform_blocks: u32,
    pub(crate) max_inline_uniform_total_size: u32,
    pub(crate) integer_dot_product_8_bit_unsigned_accelerated: Bool32,
    pub(crate) integer_dot_product_8_bit_signed_accelerated: Bool32,
    pub(crate) integer_dot_product_8_bit_mixed_signedness_accelerated: Bool32,
    pub(crate) integer_dot_product_4_x_8_bit_packed_unsigned_accelerated: Bool32,
    pub(crate) integer_dot_product_4_x_8_bit_packed_signed_accelerated: Bool32,
    pub(crate) integer_dot_product_4_x_8_bit_packed_mixed_signedness_accelerated: Bool32,
    pub(crate) integer_dot_product_16_bit_unsigned_accelerated: Bool32,
    pub(crate) integer_dot_product_16_bit_signed_accelerated: Bool32,
    pub(crate) integer_dot_product_16_bit_mixed_signedness_accelerated: Bool32,
    pub(crate) integer_dot_product_32_bit_unsigned_accelerated: Bool32,
    pub(crate) integer_dot_product_32_bit_signed_accelerated: Bool32,
    pub(crate) integer_dot_product_32_bit_mixed_signedness_accelerated: Bool32,
    pub(crate) integer_dot_product_64_bit_unsigned_accelerated: Bool32,
    pub(crate) integer_dot_product_64_bit_signed_accelerated: Bool32,
    pub(crate) integer_dot_product_64_bit_mixed_signedness_accelerated: Bool32,
    pub(crate) integer_dot_product_accumulating_saturating_8_bit_unsigned_accelerated: Bool32,
    pub(crate) integer_dot_product_accumulating_saturating_8_bit_signed_accelerated: Bool32,
    pub(crate) integer_dot_product_accumulating_saturating_8_bit_mixed_signedness_accelerated:
        Bool32,
    pub(crate) integer_dot_product_accumulating_saturating_4_x_8_bit_packed_unsigned_accelerated:
        Bool32,
    pub(crate) integer_dot_product_accumulating_saturating_4_x_8_bit_packed_signed_accelerated:
        Bool32,
    pub(crate) integer_dot_product_accumulating_saturating_4_x_8_bit_packed_mixed_signedness_accelerated:
        Bool32,
    pub(crate) integer_dot_product_accumulating_saturating_16_bit_unsigned_accelerated: Bool32,
    pub(crate) integer_dot_product_accumulating_saturating_16_bit_signed_accelerated: Bool32,
    pub(crate) integer_dot_product_accumulating_saturating_16_bit_mixed_signedness_accelerated:
        Bool32,
    pub(crate) integer_dot_product_accumulating_saturating_32_bit_unsigned_accelerated: Bool32,
    pub(crate) integer_dot_product_accumulating_saturating_32_bit_signed_accelerated: Bool32,
    pub(crate) integer_dot_product_accumulating_saturating_32_bit_mixed_signedness_accelerated:
        Bool32,
    pub(crate) integer_dot_product_accumulating_saturating_64_bit_unsigned_accelerated: Bool32,
    pub(crate) integer_dot_product_accumulating_saturating_64_bit_signed_accelerated: Bool32,
    pub(crate) integer_dot_product_accumulating_saturating_64_bit_mixed_signedness_accelerated:
        Bool32,
    pub(crate) storage_texel_buffer_offset_alignment_bytes: DeviceSize,
    pub(crate) storage_texel_buffer_offset_single_texel_alignment: Bool32,
    pub(crate) uniform_texel_buffer_offset_alignment_bytes: DeviceSize,
    pub(crate) uniform_texel_buffer_offset_single_texel_alignment: Bool32,
    pub(crate) max_buffer_size: DeviceSize,
}
#[derive(Clone, Copy, Debug, Default)]
#[repr(C)]
pub struct PhysicalDeviceVulkan13Features {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *mut c_void,
    pub(crate) robust_image_access: Bool32,
    pub(crate) inline_uniform_block: Bool32,
    pub(crate) descriptor_binding_inline_uniform_block_update_after_bind: Bool32,
    pub(crate) pipeline_creation_cache_control: Bool32,
    pub(crate) private_data: Bool32,
    pub(crate) shader_demote_to_helper_invocation: Bool32,
    pub(crate) shader_terminate_invocation: Bool32,
    pub(crate) subgroup_size_control: Bool32,
    pub(crate) compute_full_subgroups: Bool32,
    pub(crate) synchronization_2: Bool32,
    pub(crate) texture_compression_astc_hdr: Bool32,
    pub(crate) shader_zero_initialize_workgroup_memory: Bool32,
    pub(crate) dynamic_rendering: Bool32,
    pub(crate) shader_integer_dot_product: Bool32,
    pub(crate) maintenance_4: Bool32,
}
pub type Flags64 = u64;
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct AttachmentDescriptionStencilLayout {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *mut c_void,
    pub(crate) stencil_initial_layout: ImageLayout,
    pub(crate) stencil_final_layout: ImageLayout,
}
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct AttachmentReferenceStencilLayout {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *mut c_void,
    pub(crate) stencil_layout: ImageLayout,
}
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct PhysicalDeviceSeparateDepthStencilLayoutsFeatures {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *mut c_void,
    pub(crate) separate_depth_stencil_layouts: Bool32,
}
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct FramebufferAttachmentsCreateInfo {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *const c_void,
    pub(crate) attachment_image_info_count: u32,
    pub(crate) p_attachment_image_infos: *const FramebufferAttachmentImageInfo,
}
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct FramebufferAttachmentImageInfo {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *const c_void,
    pub(crate) flags: ImageCreateFlags,
    pub(crate) usage: ImageUsageFlags,
    pub(crate) width: u32,
    pub(crate) height: u32,
    pub(crate) layer_count: u32,
    pub(crate) view_format_count: u32,
    pub(crate) p_view_formats: *const Format,
}
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct RenderPassAttachmentBeginInfo {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *const c_void,
    pub(crate) attachment_count: u32,
    pub(crate) p_attachments: *const *const ImageView,
}
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct PhysicalDeviceImagelessFramebufferFeatures {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *mut c_void,
    pub(crate) imageless_framebuffer: Bool32,
}
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct ImageStencilUsageCreateInfo {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *const c_void,
    pub(crate) stencil_usage: ImageUsageFlags,
}
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct PhysicalDeviceDepthStencilResolveProperties {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *mut c_void,
    pub(crate) supported_depth_resolve_modes: ResolveModeFlags,
    pub(crate) supported_stencil_resolve_modes: ResolveModeFlags,
    pub(crate) independent_resolve_none: Bool32,
    pub(crate) independent_resolve: Bool32,
}
flags!(ResolveModeBit, ResolveModeFlags, u32);
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct SubpassDescriptionDepthStencilResolve {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *const c_void,
    pub(crate) depth_resolve_mode: ResolveModeBit,
    pub(crate) stencil_resolve_mode: ResolveModeBit,
    pub(crate) p_depth_stencil_resolve_attachment: *const AttachmentReference2,
}
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct AttachmentReference2 {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *const c_void,
    pub(crate) attachment: u32,
    pub(crate) layout: ImageLayout,
    pub(crate) aspect_mask: ImageAspectFlags,
}
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct RenderPassCreateInfo2 {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *const c_void,
    pub(crate) flags: RenderPassCreateFlags,
    pub(crate) attachment_count: u32,
    pub(crate) p_attachments: *const AttachmentDescription2,
    pub(crate) subpass_count: u32,
    pub(crate) p_subpasses: *const SubpassDescription2,
    pub(crate) dependency_count: u32,
    pub(crate) p_dependencies: *const SubpassDependency2,
    pub(crate) correlated_view_mask_count: u32,
    pub(crate) p_correlated_view_masks: *const u32,
}
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct SubpassDependency2 {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *const c_void,
    pub(crate) src_subpass: u32,
    pub(crate) dst_subpass: u32,
    pub(crate) src_stage_mask: PipelineStageFlags,
    pub(crate) dst_stage_mask: PipelineStageFlags,
    pub(crate) src_access_mask: AccessFlags,
    pub(crate) dst_access_mask: AccessFlags,
    pub(crate) dependency_flags: DependencyFlags,
    pub(crate) view_offset: i32,
}
flags!(AccessBit, AccessFlags, u32);
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[repr(i32)]
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
}

impl AccessFlags {
    pub const NONE: Self = Self(0);
}
flags!(PipelineStageBit, PipelineStageFlags, u32);
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[repr(i32)]
pub enum PipelineStageBit {
    TopOfPipe = 1 << 0,
    DrawIndirect = 1 << 1,
    VertexInput = 1 << 2,
    VertexShader = 1 << 3,
    TessellationControlShader = 1 << 4,
    TessellationEvaluationShader = 1 << 5,
    GeometryShader = 1 << 6,
    FragmentShader = 1 << 7,
    EarlyFragmentTests = 1 << 8,
    LateFragmentTests = 1 << 9,
    ColorAttachmentOutput = 1 << 10,
    ComputeShader = 1 << 11,
    Transfer = 1 << 12,
    BottomOfPipe = 1 << 13,
    Host = 1 << 14,
    AllGraphics = 1 << 15,
    AllCommands = 1 << 16,
}

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct SubpassDescription2 {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *const c_void,
    pub(crate) flags: SubpassDescriptionFlags,
    pub(crate) pipeline_bind_point: PipelineBindPoint,
    pub(crate) view_mask: u32,
    pub(crate) input_attachment_count: u32,
    pub(crate) p_input_attachments: *const AttachmentReference2,
    pub(crate) color_attachment_count: u32,
    pub(crate) p_color_attachments: *const AttachmentReference2,
    pub(crate) p_resolve_attachments: *const AttachmentReference2,
    pub(crate) p_depth_stencil_attachment: *const AttachmentReference2,
    pub(crate) preserve_attachment_count: u32,
    pub(crate) p_preserve_attachments: *const u32,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[repr(i32)]
pub enum PipelineBindPoint {
    Graphics = 0,
    Compute = 1,
}

flags!(SubpassDescriptionFlags, u32);
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct AttachmentDescription2 {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *const c_void,
    pub(crate) flags: AttachmentDescriptionFlags,
    pub(crate) format: Format,
    pub(crate) samples: SampleCountBit,
    pub(crate) load_op: AttachmentLoadOp,
    pub(crate) store_op: AttachmentStoreOp,
    pub(crate) stencil_load_op: AttachmentLoadOp,
    pub(crate) stencil_store_op: AttachmentStoreOp,
    pub(crate) initial_layout: ImageLayout,
    pub(crate) final_layout: ImageLayout,
}
flags!(AttachmentDescriptionBit, AttachmentDescriptionFlags, u32);
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[repr(i32)]
pub enum AttachmentDescriptionBit {
    MayAlias = 1 << 0,
}

flags!(RenderPassCreateFlags, u32);
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct SubpassEndInfo {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *const c_void,
}
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct SubpassBeginInfo {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *const c_void,
    pub(crate) contents: SubpassContents,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[repr(i32)]
pub enum SubpassContents {
    Inline = 0,
    SecondaryCommandBuffers = 1,
}

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct PhysicalDeviceShaderSubgroupExtendedTypesFeatures {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *mut c_void,
    pub(crate) shader_subgroup_extended_types: Bool32,
}
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct PhysicalDeviceUniformBufferStandardLayoutFeatures {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *mut c_void,
    pub(crate) uniform_buffer_standard_layout: Bool32,
}
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct PhysicalDeviceSamplerFilterMinmaxProperties {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *mut c_void,
    pub(crate) filter_minmax_single_component_formats: Bool32,
    pub(crate) filter_minmax_image_component_mapping: Bool32,
}
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct SamplerReductionModeCreateInfo {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *const c_void,
    pub(crate) reduction_mode: SamplerReductionMode,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[repr(i32)]
pub enum SamplerReductionMode {
    WeightedAverage = 0,
    Min = 1,
    Max = 2,
}

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct PhysicalDeviceScalarBlockLayoutFeatures {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *mut c_void,
    pub(crate) scalar_block_layout: Bool32,
}
flags!(DescriptorBindingBit, DescriptorBindingFlags, u32);
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[repr(i32)]
pub enum DescriptorBindingBit {
    UpdateAfterBind = 1 << 0,
    UpdateUnusedWhilePending = 1 << 1,
    PartiallyBound = 1 << 2,
    VariableDescriptorCount = 1 << 3,
}

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct DescriptorSetVariableDescriptorCountLayoutSupport {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *mut c_void,
    pub(crate) max_variable_descriptor_count: u32,
}
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct DescriptorSetVariableDescriptorCountAllocateInfo {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *const c_void,
    pub(crate) descriptor_set_count: u32,
    pub(crate) p_descriptor_counts: *const u32,
}
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct PhysicalDeviceDescriptorIndexingProperties {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *mut c_void,
    pub(crate) max_update_after_bind_descriptors_in_all_pools: u32,
    pub(crate) shader_uniform_buffer_array_non_uniform_indexing_native: Bool32,
    pub(crate) shader_sampled_image_array_non_uniform_indexing_native: Bool32,
    pub(crate) shader_storage_buffer_array_non_uniform_indexing_native: Bool32,
    pub(crate) shader_storage_image_array_non_uniform_indexing_native: Bool32,
    pub(crate) shader_input_attachment_array_non_uniform_indexing_native: Bool32,
    pub(crate) robust_buffer_access_update_after_bind: Bool32,
    pub(crate) quad_divergent_implicit_lod: Bool32,
    pub(crate) max_per_stage_descriptor_update_after_bind_samplers: u32,
    pub(crate) max_per_stage_descriptor_update_after_bind_uniform_buffers: u32,
    pub(crate) max_per_stage_descriptor_update_after_bind_storage_buffers: u32,
    pub(crate) max_per_stage_descriptor_update_after_bind_sampled_images: u32,
    pub(crate) max_per_stage_descriptor_update_after_bind_storage_images: u32,
    pub(crate) max_per_stage_descriptor_update_after_bind_input_attachments: u32,
    pub(crate) max_per_stage_update_after_bind_resources: u32,
    pub(crate) max_descriptor_set_update_after_bind_samplers: u32,
    pub(crate) max_descriptor_set_update_after_bind_uniform_buffers: u32,
    pub(crate) max_descriptor_set_update_after_bind_uniform_buffers_dynamic: u32,
    pub(crate) max_descriptor_set_update_after_bind_storage_buffers: u32,
    pub(crate) max_descriptor_set_update_after_bind_storage_buffers_dynamic: u32,
    pub(crate) max_descriptor_set_update_after_bind_sampled_images: u32,
    pub(crate) max_descriptor_set_update_after_bind_storage_images: u32,
    pub(crate) max_descriptor_set_update_after_bind_input_attachments: u32,
}
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct PhysicalDeviceDescriptorIndexingFeatures {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *mut c_void,
    pub(crate) shader_input_attachment_array_dynamic_indexing: Bool32,
    pub(crate) shader_uniform_texel_buffer_array_dynamic_indexing: Bool32,
    pub(crate) shader_storage_texel_buffer_array_dynamic_indexing: Bool32,
    pub(crate) shader_uniform_buffer_array_non_uniform_indexing: Bool32,
    pub(crate) shader_sampled_image_array_non_uniform_indexing: Bool32,
    pub(crate) shader_storage_buffer_array_non_uniform_indexing: Bool32,
    pub(crate) shader_storage_image_array_non_uniform_indexing: Bool32,
    pub(crate) shader_input_attachment_array_non_uniform_indexing: Bool32,
    pub(crate) shader_uniform_texel_buffer_array_non_uniform_indexing: Bool32,
    pub(crate) shader_storage_texel_buffer_array_non_uniform_indexing: Bool32,
    pub(crate) descriptor_binding_uniform_buffer_update_after_bind: Bool32,
    pub(crate) descriptor_binding_sampled_image_update_after_bind: Bool32,
    pub(crate) descriptor_binding_storage_image_update_after_bind: Bool32,
    pub(crate) descriptor_binding_storage_buffer_update_after_bind: Bool32,
    pub(crate) descriptor_binding_uniform_texel_buffer_update_after_bind: Bool32,
    pub(crate) descriptor_binding_storage_texel_buffer_update_after_bind: Bool32,
    pub(crate) descriptor_binding_update_unused_while_pending: Bool32,
    pub(crate) descriptor_binding_partially_bound: Bool32,
    pub(crate) descriptor_binding_variable_descriptor_count: Bool32,
    pub(crate) runtime_descriptor_array: Bool32,
}
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct DescriptorSetLayoutBindingFlagsCreateInfo {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *const c_void,
    pub(crate) binding_count: u32,
    pub(crate) p_binding_flags: *const DescriptorBindingFlags,
}
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct PhysicalDeviceFloatControlsProperties {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *mut c_void,
    pub(crate) denorm_behavior_independence: ShaderFloatControlsIndependence,
    pub(crate) rounding_mode_independence: ShaderFloatControlsIndependence,
    pub(crate) shader_signed_zero_inf_nan_preserve_float_16: Bool32,
    pub(crate) shader_signed_zero_inf_nan_preserve_float_32: Bool32,
    pub(crate) shader_signed_zero_inf_nan_preserve_float_64: Bool32,
    pub(crate) shader_denorm_preserve_float_16: Bool32,
    pub(crate) shader_denorm_preserve_float_32: Bool32,
    pub(crate) shader_denorm_preserve_float_64: Bool32,
    pub(crate) shader_denorm_flush_to_zero_float_16: Bool32,
    pub(crate) shader_denorm_flush_to_zero_float_32: Bool32,
    pub(crate) shader_denorm_flush_to_zero_float_64: Bool32,
    pub(crate) shader_rounding_mode_rte_float_16: Bool32,
    pub(crate) shader_rounding_mode_rte_float_32: Bool32,
    pub(crate) shader_rounding_mode_rte_float_64: Bool32,
    pub(crate) shader_rounding_mode_rtz_float_16: Bool32,
    pub(crate) shader_rounding_mode_rtz_float_32: Bool32,
    pub(crate) shader_rounding_mode_rtz_float_64: Bool32,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[repr(i32)]
pub enum ShaderFloatControlsIndependence {
    _32BitOnly = 0,
    All = 1,
    None = 2,
}

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct PhysicalDeviceShaderFloat16Int8Features {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *mut c_void,
    pub(crate) shader_float_16: Bool32,
    pub(crate) shader_int_8: Bool32,
}
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct PhysicalDeviceShaderAtomicInt64Features {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *mut c_void,
    pub(crate) shader_buffer_int_64_atomics: Bool32,
    pub(crate) shader_shared_int_64_atomics: Bool32,
}
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct PhysicalDevice8BitStorageFeatures {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *mut c_void,
    pub(crate) storage_buffer_8_bit_access: Bool32,
    pub(crate) uniform_and_storage_buffer_8_bit_access: Bool32,
    pub(crate) storage_push_constant_8: Bool32,
}
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct DeviceMemoryOpaqueCaptureAddressInfo {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *const c_void,
    pub(crate) memory: *const DeviceMemory,
}
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct MemoryOpaqueCaptureAddressAllocateInfo {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *const c_void,
    pub(crate) opaque_capture_address: u64,
}
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct BufferOpaqueCaptureAddressCreateInfo {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *const c_void,
    pub(crate) opaque_capture_address: u64,
}
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct BufferDeviceAddressInfo {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *const c_void,
    pub(crate) buffer: *const Buffer,
}
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct PhysicalDeviceBufferDeviceAddressFeatures {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *mut c_void,
    pub(crate) buffer_device_address: Bool32,
    pub(crate) buffer_device_address_capture_replay: Bool32,
    pub(crate) buffer_device_address_multi_device: Bool32,
}
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct SemaphoreSignalInfo {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *const c_void,
    pub(crate) semaphore: *const Semaphore,
    pub(crate) value: u64,
}
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct SemaphoreWaitInfo {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *const c_void,
    pub(crate) flags: SemaphoreWaitFlags,
    pub(crate) semaphore_count: u32,
    pub(crate) p_semaphores: *const *const Semaphore,
    pub(crate) p_values: *const u64,
}
flags!(SemaphoreWaitBit, SemaphoreWaitFlags, u32);
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[repr(i32)]
pub enum SemaphoreWaitBit {
    Any = 1 << 0,
}

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct TimelineSemaphoreSubmitInfo {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *const c_void,
    pub(crate) wait_semaphore_value_count: u32,
    pub(crate) p_wait_semaphore_values: *const u64,
    pub(crate) signal_semaphore_value_count: u32,
    pub(crate) p_signal_semaphore_values: *const u64,
}
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct SemaphoreTypeCreateInfo {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *const c_void,
    pub(crate) semaphore_type: SemaphoreType,
    pub(crate) initial_value: u64,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[repr(i32)]
pub enum SemaphoreType {
    Binary = 0,
    Timeline = 1,
}

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct PhysicalDeviceTimelineSemaphoreProperties {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *mut c_void,
    pub(crate) max_timeline_semaphore_value_difference: u64,
}
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct PhysicalDeviceTimelineSemaphoreFeatures {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *mut c_void,
    pub(crate) timeline_semaphore: Bool32,
}
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct PhysicalDeviceHostQueryResetFeatures {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *mut c_void,
    pub(crate) host_query_reset: Bool32,
}
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct PhysicalDeviceVulkanMemoryModelFeatures {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *mut c_void,
    pub(crate) vulkan_memory_model: Bool32,
    pub(crate) vulkan_memory_model_device_scope: Bool32,
    pub(crate) vulkan_memory_model_availability_visibility_chains: Bool32,
}
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct ImageFormatListCreateInfo {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *const c_void,
    pub(crate) view_format_count: u32,
    pub(crate) p_view_formats: *const Format,
}
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct PhysicalDeviceVulkan12Properties {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *mut c_void,
    pub(crate) driver_id: DriverId,
    pub(crate) driver_name: [c_char; VK_MAX_DRIVER_NAME_SIZE as usize],
    pub(crate) driver_info: [c_char; VK_MAX_DRIVER_INFO_SIZE as usize],
    pub(crate) conformance_version: ConformanceVersion,
    pub(crate) denorm_behavior_independence: ShaderFloatControlsIndependence,
    pub(crate) rounding_mode_independence: ShaderFloatControlsIndependence,
    pub(crate) shader_signed_zero_inf_nan_preserve_float_16: Bool32,
    pub(crate) shader_signed_zero_inf_nan_preserve_float_32: Bool32,
    pub(crate) shader_signed_zero_inf_nan_preserve_float_64: Bool32,
    pub(crate) shader_denorm_preserve_float_16: Bool32,
    pub(crate) shader_denorm_preserve_float_32: Bool32,
    pub(crate) shader_denorm_preserve_float_64: Bool32,
    pub(crate) shader_denorm_flush_to_zero_float_16: Bool32,
    pub(crate) shader_denorm_flush_to_zero_float_32: Bool32,
    pub(crate) shader_denorm_flush_to_zero_float_64: Bool32,
    pub(crate) shader_rounding_mode_rte_float_16: Bool32,
    pub(crate) shader_rounding_mode_rte_float_32: Bool32,
    pub(crate) shader_rounding_mode_rte_float_64: Bool32,
    pub(crate) shader_rounding_mode_rtz_float_16: Bool32,
    pub(crate) shader_rounding_mode_rtz_float_32: Bool32,
    pub(crate) shader_rounding_mode_rtz_float_64: Bool32,
    pub(crate) max_update_after_bind_descriptors_in_all_pools: u32,
    pub(crate) shader_uniform_buffer_array_non_uniform_indexing_native: Bool32,
    pub(crate) shader_sampled_image_array_non_uniform_indexing_native: Bool32,
    pub(crate) shader_storage_buffer_array_non_uniform_indexing_native: Bool32,
    pub(crate) shader_storage_image_array_non_uniform_indexing_native: Bool32,
    pub(crate) shader_input_attachment_array_non_uniform_indexing_native: Bool32,
    pub(crate) robust_buffer_access_update_after_bind: Bool32,
    pub(crate) quad_divergent_implicit_lod: Bool32,
    pub(crate) max_per_stage_descriptor_update_after_bind_samplers: u32,
    pub(crate) max_per_stage_descriptor_update_after_bind_uniform_buffers: u32,
    pub(crate) max_per_stage_descriptor_update_after_bind_storage_buffers: u32,
    pub(crate) max_per_stage_descriptor_update_after_bind_sampled_images: u32,
    pub(crate) max_per_stage_descriptor_update_after_bind_storage_images: u32,
    pub(crate) max_per_stage_descriptor_update_after_bind_input_attachments: u32,
    pub(crate) max_per_stage_update_after_bind_resources: u32,
    pub(crate) max_descriptor_set_update_after_bind_samplers: u32,
    pub(crate) max_descriptor_set_update_after_bind_uniform_buffers: u32,
    pub(crate) max_descriptor_set_update_after_bind_uniform_buffers_dynamic: u32,
    pub(crate) max_descriptor_set_update_after_bind_storage_buffers: u32,
    pub(crate) max_descriptor_set_update_after_bind_storage_buffers_dynamic: u32,
    pub(crate) max_descriptor_set_update_after_bind_sampled_images: u32,
    pub(crate) max_descriptor_set_update_after_bind_storage_images: u32,
    pub(crate) max_descriptor_set_update_after_bind_input_attachments: u32,
    pub(crate) supported_depth_resolve_modes: ResolveModeFlags,
    pub(crate) supported_stencil_resolve_modes: ResolveModeFlags,
    pub(crate) independent_resolve_none: Bool32,
    pub(crate) independent_resolve: Bool32,
    pub(crate) filter_minmax_single_component_formats: Bool32,
    pub(crate) filter_minmax_image_component_mapping: Bool32,
    pub(crate) max_timeline_semaphore_value_difference: u64,
    pub(crate) framebuffer_integer_color_sample_counts: SampleCountFlags,
}
flags!(SampleCountBit, SampleCountFlags, u32);
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct ConformanceVersion {
    pub(crate) major: u8,
    pub(crate) minor: u8,
    pub(crate) subminor: u8,
    pub(crate) patch: u8,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[repr(i32)]
pub enum DriverId {
    AmdProprietary = 1,
    AmdOpenSource = 2,
    MesaRadv = 3,
    NvidiaProprietary = 4,
    IntelProprietaryWindows = 5,
    IntelOpenSourceMesa = 6,
    ImaginationProprietary = 7,
    QualcommProprietary = 8,
    ArmProprietary = 9,
    GoogleSwiftshader = 10,
    GgpProprietary = 11,
    BroadcomProprietary = 12,
    MesaLlvmpipe = 13,
    Moltenvk = 14,
    CoreaviProprietary = 15,
    JuiceProprietary = 16,
    VerisiliconProprietary = 17,
    MesaTurnip = 18,
    MesaV3Dv = 19,
    MesaPanvk = 20,
    SamsungProprietary = 21,
    MesaVenus = 22,
    MesaDozen = 23,
    MesaNvk = 24,
    ImaginationOpenSourceMesa = 25,
    MesaHoneykrisp = 26,
    VulkanScEmulationOnVulkan = 27,
    MesaKosmickrisp = 28,
}

pub const VK_MAX_DRIVER_INFO_SIZE: u32 = 256;
pub const VK_MAX_DRIVER_NAME_SIZE: u32 = 256;
#[derive(Clone, Copy, Debug, Default)]
#[repr(C)]
pub struct PhysicalDeviceVulkan12Features {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *mut c_void,
    pub(crate) sampler_mirror_clamp_to_edge: Bool32,
    pub(crate) draw_indirect_count: Bool32,
    pub(crate) storage_buffer_8_bit_access: Bool32,
    pub(crate) uniform_and_storage_buffer_8_bit_access: Bool32,
    pub(crate) storage_push_constant_8: Bool32,
    pub(crate) shader_buffer_int_64_atomics: Bool32,
    pub(crate) shader_shared_int_64_atomics: Bool32,
    pub(crate) shader_float_16: Bool32,
    pub(crate) shader_int_8: Bool32,
    pub(crate) descriptor_indexing: Bool32,
    pub(crate) shader_input_attachment_array_dynamic_indexing: Bool32,
    pub(crate) shader_uniform_texel_buffer_array_dynamic_indexing: Bool32,
    pub(crate) shader_storage_texel_buffer_array_dynamic_indexing: Bool32,
    pub(crate) shader_uniform_buffer_array_non_uniform_indexing: Bool32,
    pub(crate) shader_sampled_image_array_non_uniform_indexing: Bool32,
    pub(crate) shader_storage_buffer_array_non_uniform_indexing: Bool32,
    pub(crate) shader_storage_image_array_non_uniform_indexing: Bool32,
    pub(crate) shader_input_attachment_array_non_uniform_indexing: Bool32,
    pub(crate) shader_uniform_texel_buffer_array_non_uniform_indexing: Bool32,
    pub(crate) shader_storage_texel_buffer_array_non_uniform_indexing: Bool32,
    pub(crate) descriptor_binding_uniform_buffer_update_after_bind: Bool32,
    pub(crate) descriptor_binding_sampled_image_update_after_bind: Bool32,
    pub(crate) descriptor_binding_storage_image_update_after_bind: Bool32,
    pub(crate) descriptor_binding_storage_buffer_update_after_bind: Bool32,
    pub(crate) descriptor_binding_uniform_texel_buffer_update_after_bind: Bool32,
    pub(crate) descriptor_binding_storage_texel_buffer_update_after_bind: Bool32,
    pub(crate) descriptor_binding_update_unused_while_pending: Bool32,
    pub(crate) descriptor_binding_partially_bound: Bool32,
    pub(crate) descriptor_binding_variable_descriptor_count: Bool32,
    pub(crate) runtime_descriptor_array: Bool32,
    pub(crate) sampler_filter_minmax: Bool32,
    pub(crate) scalar_block_layout: Bool32,
    pub(crate) imageless_framebuffer: Bool32,
    pub(crate) uniform_buffer_standard_layout: Bool32,
    pub(crate) shader_subgroup_extended_types: Bool32,
    pub(crate) separate_depth_stencil_layouts: Bool32,
    pub(crate) host_query_reset: Bool32,
    pub(crate) timeline_semaphore: Bool32,
    pub(crate) buffer_device_address: Bool32,
    pub(crate) buffer_device_address_capture_replay: Bool32,
    pub(crate) buffer_device_address_multi_device: Bool32,
    pub(crate) vulkan_memory_model: Bool32,
    pub(crate) vulkan_memory_model_device_scope: Bool32,
    pub(crate) vulkan_memory_model_availability_visibility_chains: Bool32,
    pub(crate) shader_output_viewport_index: Bool32,
    pub(crate) shader_output_layer: Bool32,
    pub(crate) subgroup_broadcast_dynamic_id: Bool32,
}
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct PhysicalDeviceVulkan11Properties {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *mut c_void,
    pub(crate) device_uuid: [u8; VK_UUID_SIZE as usize],
    pub(crate) driver_uuid: [u8; VK_UUID_SIZE as usize],
    pub(crate) device_luid: [u8; VK_LUID_SIZE as usize],
    pub(crate) device_node_mask: u32,
    pub(crate) device_luid_valid: Bool32,
    pub(crate) subgroup_size: u32,
    pub(crate) subgroup_supported_stages: ShaderStageFlags,
    pub(crate) subgroup_supported_operations: SubgroupFeatureFlags,
    pub(crate) subgroup_quad_operations_in_all_stages: Bool32,
    pub(crate) point_clipping_behavior: PointClippingBehavior,
    pub(crate) max_multiview_view_count: u32,
    pub(crate) max_multiview_instance_index: u32,
    pub(crate) protected_no_fault: Bool32,
    pub(crate) max_per_set_descriptors: u32,
    pub(crate) max_memory_allocation_size: DeviceSize,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[repr(i32)]
pub enum PointClippingBehavior {
    AllClipPlanes = 0,
    UserClipPlanesOnly = 1,
}

flags!(SubgroupFeatureBit, SubgroupFeatureFlags, u32);
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[repr(i32)]
pub enum SubgroupFeatureBit {
    Basic = 1 << 0,
    Vote = 1 << 1,
    Arithmetic = 1 << 2,
    Ballot = 1 << 3,
    Shuffle = 1 << 4,
    ShuffleRelative = 1 << 5,
    Clustered = 1 << 6,
    Quad = 1 << 7,
    Rotate = 1 << 9,
    RotateClustered = 1 << 10,
}

pub const VK_LUID_SIZE: u32 = 8;
#[derive(Clone, Copy, Debug, Default)]
#[repr(C)]
pub struct PhysicalDeviceVulkan11Features {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *mut c_void,
    pub(crate) storage_buffer_16_bit_access: Bool32,
    pub(crate) uniform_and_storage_buffer_16_bit_access: Bool32,
    pub(crate) storage_push_constant_16: Bool32,
    pub(crate) storage_input_output_16: Bool32,
    pub(crate) multiview: Bool32,
    pub(crate) multiview_geometry_shader: Bool32,
    pub(crate) multiview_tessellation_shader: Bool32,
    pub(crate) variable_pointers_storage_buffer: Bool32,
    pub(crate) variable_pointers: Bool32,
    pub(crate) protected_memory: Bool32,
    pub(crate) sampler_ycbcr_conversion: Bool32,
    pub(crate) shader_draw_parameters: Bool32,
}
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct PhysicalDeviceDriverProperties {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *mut c_void,
    pub(crate) driver_id: DriverId,
    pub(crate) driver_name: [c_char; VK_MAX_DRIVER_NAME_SIZE as usize],
    pub(crate) driver_info: [c_char; VK_MAX_DRIVER_INFO_SIZE as usize],
    pub(crate) conformance_version: ConformanceVersion,
}
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct PhysicalDeviceShaderDrawParametersFeatures {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *mut c_void,
    pub(crate) shader_draw_parameters: Bool32,
}
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct PhysicalDeviceMultiviewProperties {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *mut c_void,
    pub(crate) max_multiview_view_count: u32,
    pub(crate) max_multiview_instance_index: u32,
}
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct PhysicalDeviceMultiviewFeatures {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *mut c_void,
    pub(crate) multiview: Bool32,
    pub(crate) multiview_geometry_shader: Bool32,
    pub(crate) multiview_tessellation_shader: Bool32,
}
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct RenderPassMultiviewCreateInfo {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *const c_void,
    pub(crate) subpass_count: u32,
    pub(crate) p_view_masks: *const u32,
    pub(crate) dependency_count: u32,
    pub(crate) p_view_offsets: *const i32,
    pub(crate) correlation_mask_count: u32,
    pub(crate) p_correlation_masks: *const u32,
}
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct PipelineTessellationDomainOriginStateCreateInfo {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *const c_void,
    pub(crate) domain_origin: TessellationDomainOrigin,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[repr(i32)]
pub enum TessellationDomainOrigin {
    UpperLeft = 0,
    LowerLeft = 1,
}

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct RenderPassInputAttachmentAspectCreateInfo {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *const c_void,
    pub(crate) aspect_reference_count: u32,
    pub(crate) p_aspect_references: *const InputAttachmentAspectReference,
}
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct InputAttachmentAspectReference {
    pub(crate) subpass: u32,
    pub(crate) input_attachment_index: u32,
    pub(crate) aspect_mask: ImageAspectFlags,
}
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct PhysicalDevicePointClippingProperties {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *mut c_void,
    pub(crate) point_clipping_behavior: PointClippingBehavior,
}
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct DeviceGroupRenderPassBeginInfo {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *const c_void,
    pub(crate) device_mask: u32,
    pub(crate) device_render_area_count: u32,
    pub(crate) p_device_render_areas: *const Rect2D,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[repr(i32)]
pub enum ChromaLocation {
    CositedEven = 0,
    Midpoint = 1,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[repr(i32)]
pub enum SamplerYcbcrRange {
    ItuFull = 0,
    ItuNarrow = 1,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[repr(i32)]
pub enum SamplerYcbcrModelConversion {
    RgbIdentity = 0,
    YcbcrIdentity = 1,
    Ycbcr709 = 2,
    Ycbcr601 = 3,
    Ycbcr2020 = 4,
}

#[repr(transparent)]
pub struct SamplerYcbcrConversion([u8; 0]);
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct SamplerYcbcrConversionImageFormatProperties {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *mut c_void,
    pub(crate) combined_image_sampler_descriptor_count: u32,
}
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct PhysicalDeviceSamplerYcbcrConversionFeatures {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *mut c_void,
    pub(crate) sampler_ycbcr_conversion: Bool32,
}
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct SamplerYcbcrConversionInfo {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *const c_void,
    pub(crate) conversion: *const SamplerYcbcrConversion,
}
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct SamplerYcbcrConversionCreateInfo {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *const c_void,
    pub(crate) format: Format,
    pub(crate) ycbcr_model: SamplerYcbcrModelConversion,
    pub(crate) ycbcr_range: SamplerYcbcrRange,
    pub(crate) components: ComponentMapping,
    pub(crate) x_chroma_offset: ChromaLocation,
    pub(crate) y_chroma_offset: ChromaLocation,
    pub(crate) chroma_filter: Filter,
    pub(crate) force_explicit_reconstruction: Bool32,
}
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct DescriptorSetLayoutSupport {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *mut c_void,
    pub(crate) supported: Bool32,
}
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct PhysicalDeviceMaintenance3Properties {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *mut c_void,
    pub(crate) max_per_set_descriptors: u32,
    pub(crate) max_memory_allocation_size: DeviceSize,
}
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct DescriptorUpdateTemplateCreateInfo {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *const c_void,
    pub(crate) flags: DescriptorUpdateTemplateCreateFlags,
    pub(crate) descriptor_update_entry_count: u32,
    pub(crate) p_descriptor_update_entries: *const DescriptorUpdateTemplateEntry,
    pub(crate) template_type: DescriptorUpdateTemplateType,
    pub(crate) descriptor_set_layout: *const DescriptorSetLayout,
    pub(crate) pipeline_bind_point: PipelineBindPoint,
    pub(crate) pipeline_layout: *const PipelineLayout,
    pub(crate) set: u32,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[repr(i32)]
pub enum DescriptorUpdateTemplateType {
    DescriptorSet = 0,
    PushDescriptors = 1,
}

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct DescriptorUpdateTemplateEntry {
    pub(crate) dst_binding: u32,
    pub(crate) dst_array_element: u32,
    pub(crate) descriptor_count: u32,
    pub(crate) descriptor_type: DescriptorType,
    pub(crate) offset: usize,
    pub(crate) stride: usize,
}
flags!(DescriptorUpdateTemplateCreateFlags, u32);
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct PhysicalDeviceVariablePointersFeatures {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *mut c_void,
    pub(crate) variable_pointers_storage_buffer: Bool32,
    pub(crate) variable_pointers: Bool32,
}
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct PhysicalDevice16BitStorageFeatures {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *mut c_void,
    pub(crate) storage_buffer_16_bit_access: Bool32,
    pub(crate) uniform_and_storage_buffer_16_bit_access: Bool32,
    pub(crate) storage_push_constant_16: Bool32,
    pub(crate) storage_input_output_16: Bool32,
}
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct PhysicalDeviceSubgroupProperties {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *mut c_void,
    pub(crate) subgroup_size: u32,
    pub(crate) supported_stages: ShaderStageFlags,
    pub(crate) supported_operations: SubgroupFeatureFlags,
    pub(crate) quad_operations_in_all_stages: Bool32,
}
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct ExternalSemaphoreProperties {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *mut c_void,
    pub(crate) export_from_imported_handle_types: ExternalSemaphoreHandleTypeFlags,
    pub(crate) compatible_handle_types: ExternalSemaphoreHandleTypeFlags,
    pub(crate) external_semaphore_features: ExternalSemaphoreFeatureFlags,
}
flags!(
    ExternalSemaphoreFeatureBit,
    ExternalSemaphoreFeatureFlags,
    u32
);
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[repr(i32)]
pub enum ExternalSemaphoreFeatureBit {
    Exportable = 1 << 0,
    Importable = 1 << 1,
}

flags!(
    ExternalSemaphoreHandleTypeBit,
    ExternalSemaphoreHandleTypeFlags,
    u32
);
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[repr(i32)]
pub enum ExternalSemaphoreHandleTypeBit {
    OpaqueFd = 1 << 0,
    OpaqueWin32 = 1 << 1,
    OpaqueWin32Kmt = 1 << 2,
    D3D12Fence = 1 << 3,
    SyncFd = 1 << 4,
}

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct PhysicalDeviceExternalSemaphoreInfo {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *const c_void,
    pub(crate) handle_type: ExternalSemaphoreHandleTypeBit,
}
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct ExportSemaphoreCreateInfo {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *const c_void,
    pub(crate) handle_types: ExternalSemaphoreHandleTypeFlags,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[repr(i32)]
pub enum SemaphoreImportBit {
    Temporary = 1 << 0,
}

flags!(SemaphoreImportBit, SemaphoreImportFlags, u32);
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct ExportFenceCreateInfo {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *const c_void,
    pub(crate) handle_types: ExternalFenceHandleTypeFlags,
}
flags!(
    ExternalFenceHandleTypeBit,
    ExternalFenceHandleTypeFlags,
    u32
);
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[repr(i32)]
pub enum ExternalFenceHandleTypeBit {
    OpaqueFd = 1 << 0,
    OpaqueWin32 = 1 << 1,
    OpaqueWin32Kmt = 1 << 2,
    SyncFd = 1 << 3,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[repr(i32)]
pub enum FenceImportBit {
    Temporary = 1 << 0,
}

flags!(FenceImportBit, FenceImportFlags, u32);
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct ExternalFenceProperties {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *mut c_void,
    pub(crate) export_from_imported_handle_types: ExternalFenceHandleTypeFlags,
    pub(crate) compatible_handle_types: ExternalFenceHandleTypeFlags,
    pub(crate) external_fence_features: ExternalFenceFeatureFlags,
}
flags!(ExternalFenceFeatureBit, ExternalFenceFeatureFlags, u32);
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[repr(i32)]
pub enum ExternalFenceFeatureBit {
    Exportable = 1 << 0,
    Importable = 1 << 1,
}

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct PhysicalDeviceExternalFenceInfo {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *const c_void,
    pub(crate) handle_type: ExternalFenceHandleTypeBit,
}
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct ExportMemoryAllocateInfo {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *const c_void,
    pub(crate) handle_types: ExternalMemoryHandleTypeFlags,
}
flags!(
    ExternalMemoryHandleTypeBit,
    ExternalMemoryHandleTypeFlags,
    u32
);
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[repr(i32)]
pub enum ExternalMemoryHandleTypeBit {
    OpaqueFd = 1 << 0,
    OpaqueWin32 = 1 << 1,
    OpaqueWin32Kmt = 1 << 2,
    D3D11Texture = 1 << 3,
    D3D11TextureKmt = 1 << 4,
    D3D12Heap = 1 << 5,
    D3D12Resource = 1 << 6,
    DmaBuf = 1 << 9,
}

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct ExternalMemoryBufferCreateInfo {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *const c_void,
    pub(crate) handle_types: ExternalMemoryHandleTypeFlags,
}
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct ExternalMemoryImageCreateInfo {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *const c_void,
    pub(crate) handle_types: ExternalMemoryHandleTypeFlags,
}
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct ImageDrmFormatModifierListCreateInfo {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *const c_void,
    pub(crate) drm_format_modifier_count: u32,
    pub(crate) p_drm_format_modifiers: *const u64,
}
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct ImageDrmFormatModifierProperties {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *const c_void,
    pub(crate) drm_format_modifier: u64,
}
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct DrmFormatModifierPropertiesList {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *const c_void,
    pub(crate) drm_format_modifier_count: u32,
    pub(crate) p_drm_format_modifier_properties: *mut DrmFormatModifierProperties,
}
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct DrmFormatModifierProperties {
    pub(crate) drm_format_modifier: u64,
    pub(crate) drm_format_modifier_plane_count: u32,
    pub(crate) drm_format_modifier_tiling_features: FormatFeatureFlags,
}
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct MemoryGetFdInfo {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *const c_void,
    pub(crate) memory: *const DeviceMemory,
    pub(crate) handle_type: ExternalMemoryHandleTypeBit,
}
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct ImportSemaphoreFdInfo {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *const c_void,
    pub(crate) semaphore: *const Semaphore,
    pub(crate) flags: SemaphoreImportFlags,
    pub(crate) handle_type: ExternalSemaphoreHandleTypeBit,
    pub(crate) fd: i32,
}
// #[derive(Clone, Copy, Debug)]
// #[repr(C)]
// pub struct ImportSemaphoreWin32HandleInfo {
//     pub(crate) s_type: StructureType,
//     pub(crate) p_next: *const c_void,
//     pub(crate) semaphore: *const Semaphore,
//     pub(crate) flags: SemaphoreImportFlags,
//     pub(crate) handle_type: ExternalSemaphoreHandleTypeBit,
//     pub(crate) handle: HANDLE,
//     pub(crate) name: PCWSTR,
// }
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct SemaphoreGetFdInfo {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *const c_void,
    pub(crate) semaphore: *const Semaphore,
    pub(crate) handle_type: ExternalSemaphoreHandleTypeBit,
}
// #[derive(Clone, Copy, Debug)]
// #[repr(C)]
// pub struct ImportMemoryWin32HandleInfo {
//     pub(crate) s_type: StructureType,
//     pub(crate) p_next: *const c_void,
//     pub(crate) handle_type: ExternalMemoryHandleTypeBit,
//     pub(crate) handle: HANDLE,
//     pub(crate) name: PCWSTR,
// }
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct PhysicalDeviceDrmProperties {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *mut c_void,
    pub(crate) has_primary: Bool32,
    pub(crate) has_render: Bool32,
    pub(crate) primary_major: i64,
    pub(crate) primary_minor: i64,
    pub(crate) render_major: i64,
    pub(crate) render_minor: i64,
}
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct PhysicalDeviceIDProperties {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *mut c_void,
    pub(crate) device_uuid: [u8; VK_UUID_SIZE as usize],
    pub(crate) driver_uuid: [u8; VK_UUID_SIZE as usize],
    pub(crate) device_luid: [u8; VK_LUID_SIZE as usize],
    pub(crate) device_node_mask: u32,
    pub(crate) device_luid_valid: Bool32,
}
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct ExternalBufferProperties {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *mut c_void,
    pub(crate) external_memory_properties: ExternalMemoryProperties,
}
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct ExternalMemoryProperties {
    pub(crate) external_memory_features: ExternalMemoryFeatureFlags,
    pub(crate) export_from_imported_handle_types: ExternalMemoryHandleTypeFlags,
    pub(crate) compatible_handle_types: ExternalMemoryHandleTypeFlags,
}
flags!(ExternalMemoryFeatureBit, ExternalMemoryFeatureFlags, u32);
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[repr(i32)]
pub enum ExternalMemoryFeatureBit {
    DedicatedOnly = 1 << 0,
    Exportable = 1 << 1,
    Importable = 1 << 2,
}

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct PhysicalDeviceExternalBufferInfo {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *const c_void,
    pub(crate) flags: BufferCreateFlags,
    pub(crate) usage: BufferUsageFlags,
    pub(crate) handle_type: ExternalMemoryHandleTypeBit,
}
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct ExternalImageFormatProperties {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *mut c_void,
    pub(crate) external_memory_properties: ExternalMemoryProperties,
}
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct PhysicalDeviceExternalImageFormatInfo {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *const c_void,
    pub(crate) handle_type: ExternalMemoryHandleTypeBit,
}
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct ImagePlaneMemoryRequirementsInfo {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *const c_void,
    pub(crate) plane_aspect: ImageAspectBit,
}
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct BindImagePlaneMemoryInfo {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *const c_void,
    pub(crate) plane_aspect: ImageAspectBit,
}
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct ProtectedSubmitInfo {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *const c_void,
    pub(crate) protected_submit: Bool32,
}
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct DeviceQueueInfo2 {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *const c_void,
    pub(crate) flags: DeviceQueueCreateFlags,
    pub(crate) queue_family_index: u32,
    pub(crate) queue_index: u32,
}
flags!(DeviceQueueCreateBit, DeviceQueueCreateFlags, u32);
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[repr(i32)]
pub enum DeviceQueueCreateBit {
    Protected = 1 << 0,
}

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct PhysicalDeviceProtectedMemoryProperties {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *mut c_void,
    pub(crate) protected_no_fault: Bool32,
}
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct PhysicalDeviceProtectedMemoryFeatures {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *mut c_void,
    pub(crate) protected_memory: Bool32,
}
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct ImageViewUsageCreateInfo {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *const c_void,
    pub(crate) usage: ImageUsageFlags,
}
flags!(CommandPoolTrimFlags, u32);
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct PhysicalDeviceSparseImageFormatInfo2 {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *const c_void,
    pub(crate) format: Format,
    pub(crate) r#type: ImageType,
    pub(crate) samples: SampleCountBit,
    pub(crate) usage: ImageUsageFlags,
    pub(crate) tiling: ImageTiling,
}
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct SparseImageFormatProperties2 {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *mut c_void,
    pub(crate) properties: SparseImageFormatProperties,
}
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct SparseImageFormatProperties {
    pub(crate) aspect_mask: ImageAspectFlags,
    pub(crate) image_granularity: Extent3D,
    pub(crate) flags: SparseImageFormatFlags,
}
flags!(SparseImageFormatBit, SparseImageFormatFlags, u32);
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[repr(i32)]
pub enum SparseImageFormatBit {
    SingleMiptail = 1 << 0,
    AlignedMipSize = 1 << 1,
    NonstandardBlockSize = 1 << 2,
}

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct PhysicalDeviceMemoryProperties2 {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *mut c_void,
    pub(crate) memory_properties: PhysicalDeviceMemoryProperties,
}
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct PhysicalDeviceMemoryProperties {
    pub(crate) memory_type_count: u32,
    pub(crate) memory_types: [MemoryType; VK_MAX_MEMORY_TYPES as usize],
    pub(crate) memory_heap_count: u32,
    pub(crate) memory_heaps: [MemoryHeap; VK_MAX_MEMORY_HEAPS as usize],
}
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct MemoryHeap {
    pub(crate) size: DeviceSize,
    pub(crate) flags: MemoryHeapFlags,
}
flags!(MemoryHeapBit, MemoryHeapFlags, u32);
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[repr(i32)]
pub enum MemoryHeapBit {
    DeviceLocal = 1 << 0,
    MultiInstance = 1 << 1,
}

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct MemoryType {
    pub(crate) property_flags: MemoryPropertyFlags,
    pub(crate) heap_index: u32,
}
flags!(MemoryPropertyBit, MemoryPropertyFlags, u32);
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[repr(i32)]
pub enum MemoryPropertyBit {
    DeviceLocal = 1 << 0,
    HostVisible = 1 << 1,
    HostCoherent = 1 << 2,
    HostCached = 1 << 3,
    LazilyAllocated = 1 << 4,
    Protected = 1 << 5,
}

pub const VK_MAX_MEMORY_HEAPS: u32 = 16;
pub const VK_MAX_MEMORY_TYPES: u32 = 32;
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct QueueFamilyProperties2 {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *mut c_void,
    pub(crate) queue_family_properties: QueueFamilyProperties,
}
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct QueueFamilyProperties {
    pub(crate) queue_flags: QueueFlags,
    pub(crate) queue_count: u32,
    pub(crate) timestamp_valid_bits: u32,
    pub(crate) min_image_transfer_granularity: Extent3D,
}
flags!(QueueBit, QueueFlags, u32);
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[repr(i32)]
pub enum QueueBit {
    Graphics = 1 << 0,
    Compute = 1 << 1,
    Transfer = 1 << 2,
    SparseBinding = 1 << 3,
    Protected = 1 << 4,
}

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct PhysicalDeviceImageFormatInfo2 {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *const c_void,
    pub(crate) format: Format,
    pub(crate) r#type: ImageType,
    pub(crate) tiling: ImageTiling,
    pub(crate) usage: ImageUsageFlags,
    pub(crate) flags: ImageCreateFlags,
}
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct ImageFormatProperties2 {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *mut c_void,
    pub(crate) image_format_properties: ImageFormatProperties,
}
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct ImageFormatProperties {
    pub(crate) max_extent: Extent3D,
    pub(crate) max_mip_levels: u32,
    pub(crate) max_array_layers: u32,
    pub(crate) sample_counts: SampleCountFlags,
    pub(crate) max_resource_size: DeviceSize,
}
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct FormatProperties2 {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *mut c_void,
    pub(crate) format_properties: FormatProperties,
}
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct FormatProperties {
    pub(crate) linear_tiling_features: FormatFeatureFlags,
    pub(crate) optimal_tiling_features: FormatFeatureFlags,
    pub(crate) buffer_features: FormatFeatureFlags,
}
flags!(FormatFeatureBit, FormatFeatureFlags, u32);
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[repr(i32)]
pub enum FormatFeatureBit {
    SampledImage = 1 << 0,
    StorageImage = 1 << 1,
    StorageImageAtomic = 1 << 2,
    UniformTexelBuffer = 1 << 3,
    StorageTexelBuffer = 1 << 4,
    StorageTexelBufferAtomic = 1 << 5,
    VertexBuffer = 1 << 6,
    ColorAttachment = 1 << 7,
    ColorAttachmentBlend = 1 << 8,
    DepthStencilAttachment = 1 << 9,
    BlitSrc = 1 << 10,
    BlitDst = 1 << 11,
    SampledImageFilterLinear = 1 << 12,
    TransferSrc = 1 << 14,
    TransferDst = 1 << 15,
    MidpointChromaSamples = 1 << 17,
    SampledImageYcbcrConversionLinearFilter = 1 << 18,
    SampledImageYcbcrConversionSeparateReconstructionFilter = 1 << 19,
    SampledImageYcbcrConversionChromaReconstructionExplicit = 1 << 20,
    SampledImageYcbcrConversionChromaReconstructionExplicitForceable = 1 << 21,
    Disjoint = 1 << 22,
    CositedChromaSamples = 1 << 23,
    SampledImageFilterMinmax = 1 << 16,
}

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct PhysicalDeviceProperties2 {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *mut c_void,
    pub(crate) properties: PhysicalDeviceProperties,
}
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct PhysicalDeviceProperties {
    pub(crate) api_version: u32,
    pub(crate) driver_version: u32,
    pub(crate) vendor_id: u32,
    pub(crate) device_id: u32,
    pub(crate) device_type: PhysicalDeviceType,
    pub(crate) device_name: [c_char; VK_MAX_PHYSICAL_DEVICE_NAME_SIZE as usize],
    pub(crate) pipeline_cache_uuid: [u8; VK_UUID_SIZE as usize],
    pub(crate) limits: PhysicalDeviceLimits,
    pub(crate) sparse_properties: PhysicalDeviceSparseProperties,
}
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct PhysicalDeviceSparseProperties {
    pub(crate) residency_standard_2_d_block_shape: Bool32,
    pub(crate) residency_standard_2_d_multisample_block_shape: Bool32,
    pub(crate) residency_standard_3_d_block_shape: Bool32,
    pub(crate) residency_aligned_mip_size: Bool32,
    pub(crate) residency_non_resident_strict: Bool32,
}
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct PhysicalDeviceLimits {
    pub(crate) max_image_dimension_1_d: u32,
    pub(crate) max_image_dimension_2_d: u32,
    pub(crate) max_image_dimension_3_d: u32,
    pub(crate) max_image_dimension_cube: u32,
    pub(crate) max_image_array_layers: u32,
    pub(crate) max_texel_buffer_elements: u32,
    pub(crate) max_uniform_buffer_range: u32,
    pub(crate) max_storage_buffer_range: u32,
    pub(crate) max_push_constants_size: u32,
    pub(crate) max_memory_allocation_count: u32,
    pub(crate) max_sampler_allocation_count: u32,
    pub(crate) buffer_image_granularity: DeviceSize,
    pub(crate) sparse_address_space_size: DeviceSize,
    pub(crate) max_bound_descriptor_sets: u32,
    pub(crate) max_per_stage_descriptor_samplers: u32,
    pub(crate) max_per_stage_descriptor_uniform_buffers: u32,
    pub(crate) max_per_stage_descriptor_storage_buffers: u32,
    pub(crate) max_per_stage_descriptor_sampled_images: u32,
    pub(crate) max_per_stage_descriptor_storage_images: u32,
    pub(crate) max_per_stage_descriptor_input_attachments: u32,
    pub(crate) max_per_stage_resources: u32,
    pub(crate) max_descriptor_set_samplers: u32,
    pub(crate) max_descriptor_set_uniform_buffers: u32,
    pub(crate) max_descriptor_set_uniform_buffers_dynamic: u32,
    pub(crate) max_descriptor_set_storage_buffers: u32,
    pub(crate) max_descriptor_set_storage_buffers_dynamic: u32,
    pub(crate) max_descriptor_set_sampled_images: u32,
    pub(crate) max_descriptor_set_storage_images: u32,
    pub(crate) max_descriptor_set_input_attachments: u32,
    pub(crate) max_vertex_input_attributes: u32,
    pub(crate) max_vertex_input_bindings: u32,
    pub(crate) max_vertex_input_attribute_offset: u32,
    pub(crate) max_vertex_input_binding_stride: u32,
    pub(crate) max_vertex_output_components: u32,
    pub(crate) max_tessellation_generation_level: u32,
    pub(crate) max_tessellation_patch_size: u32,
    pub(crate) max_tessellation_control_per_vertex_input_components: u32,
    pub(crate) max_tessellation_control_per_vertex_output_components: u32,
    pub(crate) max_tessellation_control_per_patch_output_components: u32,
    pub(crate) max_tessellation_control_total_output_components: u32,
    pub(crate) max_tessellation_evaluation_input_components: u32,
    pub(crate) max_tessellation_evaluation_output_components: u32,
    pub(crate) max_geometry_shader_invocations: u32,
    pub(crate) max_geometry_input_components: u32,
    pub(crate) max_geometry_output_components: u32,
    pub(crate) max_geometry_output_vertices: u32,
    pub(crate) max_geometry_total_output_components: u32,
    pub(crate) max_fragment_input_components: u32,
    pub(crate) max_fragment_output_attachments: u32,
    pub(crate) max_fragment_dual_src_attachments: u32,
    pub(crate) max_fragment_combined_output_resources: u32,
    pub(crate) max_compute_shared_memory_size: u32,
    pub(crate) max_compute_work_group_count: [u32; 3 as usize],
    pub(crate) max_compute_work_group_invocations: u32,
    pub(crate) max_compute_work_group_size: [u32; 3 as usize],
    pub(crate) sub_pixel_precision_bits: u32,
    pub(crate) sub_texel_precision_bits: u32,
    pub(crate) mipmap_precision_bits: u32,
    pub(crate) max_draw_indexed_index_value: u32,
    pub(crate) max_draw_indirect_count: u32,
    pub(crate) max_sampler_lod_bias: f32,
    pub(crate) max_sampler_anisotropy: f32,
    pub(crate) max_viewports: u32,
    pub(crate) max_viewport_dimensions: [u32; 2 as usize],
    pub(crate) viewport_bounds_range: [f32; 2 as usize],
    pub(crate) viewport_sub_pixel_bits: u32,
    pub(crate) min_memory_map_alignment: usize,
    pub(crate) min_texel_buffer_offset_alignment: DeviceSize,
    pub(crate) min_uniform_buffer_offset_alignment: DeviceSize,
    pub(crate) min_storage_buffer_offset_alignment: DeviceSize,
    pub(crate) min_texel_offset: i32,
    pub(crate) max_texel_offset: u32,
    pub(crate) min_texel_gather_offset: i32,
    pub(crate) max_texel_gather_offset: u32,
    pub(crate) min_interpolation_offset: f32,
    pub(crate) max_interpolation_offset: f32,
    pub(crate) sub_pixel_interpolation_offset_bits: u32,
    pub(crate) max_framebuffer_width: u32,
    pub(crate) max_framebuffer_height: u32,
    pub(crate) max_framebuffer_layers: u32,
    pub(crate) framebuffer_color_sample_counts: SampleCountFlags,
    pub(crate) framebuffer_depth_sample_counts: SampleCountFlags,
    pub(crate) framebuffer_stencil_sample_counts: SampleCountFlags,
    pub(crate) framebuffer_no_attachments_sample_counts: SampleCountFlags,
    pub(crate) max_color_attachments: u32,
    pub(crate) sampled_image_color_sample_counts: SampleCountFlags,
    pub(crate) sampled_image_integer_sample_counts: SampleCountFlags,
    pub(crate) sampled_image_depth_sample_counts: SampleCountFlags,
    pub(crate) sampled_image_stencil_sample_counts: SampleCountFlags,
    pub(crate) storage_image_sample_counts: SampleCountFlags,
    pub(crate) max_sample_mask_words: u32,
    pub(crate) timestamp_compute_and_graphics: Bool32,
    pub(crate) timestamp_period: f32,
    pub(crate) max_clip_distances: u32,
    pub(crate) max_cull_distances: u32,
    pub(crate) max_combined_clip_and_cull_distances: u32,
    pub(crate) discrete_queue_priorities: u32,
    pub(crate) point_size_range: [f32; 2 as usize],
    pub(crate) line_width_range: [f32; 2 as usize],
    pub(crate) point_size_granularity: f32,
    pub(crate) line_width_granularity: f32,
    pub(crate) strict_lines: Bool32,
    pub(crate) standard_sample_locations: Bool32,
    pub(crate) optimal_buffer_copy_offset_alignment: DeviceSize,
    pub(crate) optimal_buffer_copy_row_pitch_alignment: DeviceSize,
    pub(crate) non_coherent_atom_size: DeviceSize,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[repr(transparent)]
pub struct PhysicalDeviceType(core::ffi::c_int);

impl PhysicalDeviceType {
    pub const OTHER: Self = Self(0);
    pub const INTEGRATED_GPU: Self = Self(1);
    pub const DISCRETE_GPU: Self = Self(2);
    pub const VIRTUAL_GPU: Self = Self(3);
    pub const CPU: Self = Self(4);
}

pub const VK_MAX_PHYSICAL_DEVICE_NAME_SIZE: u32 = 256;
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct PhysicalDeviceFeatures2 {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *mut c_void,
    pub(crate) features: PhysicalDeviceFeatures,
}
#[derive(Clone, Copy, Debug, Default)]
#[repr(C)]
pub struct PhysicalDeviceFeatures {
    pub(crate) robust_buffer_access: Bool32,
    pub(crate) full_draw_index_uint_32: Bool32,
    pub(crate) image_cube_array: Bool32,
    pub(crate) independent_blend: Bool32,
    pub(crate) geometry_shader: Bool32,
    pub(crate) tessellation_shader: Bool32,
    pub(crate) sample_rate_shading: Bool32,
    pub(crate) dual_src_blend: Bool32,
    pub(crate) logic_op: Bool32,
    pub(crate) multi_draw_indirect: Bool32,
    pub(crate) draw_indirect_first_instance: Bool32,
    pub(crate) depth_clamp: Bool32,
    pub(crate) depth_bias_clamp: Bool32,
    pub(crate) fill_mode_non_solid: Bool32,
    pub(crate) depth_bounds: Bool32,
    pub(crate) wide_lines: Bool32,
    pub(crate) large_points: Bool32,
    pub(crate) alpha_to_one: Bool32,
    pub(crate) multi_viewport: Bool32,
    pub(crate) sampler_anisotropy: Bool32,
    pub(crate) texture_compression_etc_2: Bool32,
    pub(crate) texture_compression_astc_ldr: Bool32,
    pub(crate) texture_compression_bc: Bool32,
    pub(crate) occlusion_query_precise: Bool32,
    pub(crate) pipeline_statistics_query: Bool32,
    pub(crate) vertex_pipeline_stores_and_atomics: Bool32,
    pub(crate) fragment_stores_and_atomics: Bool32,
    pub(crate) shader_tessellation_and_geometry_point_size: Bool32,
    pub(crate) shader_image_gather_extended: Bool32,
    pub(crate) shader_storage_image_extended_formats: Bool32,
    pub(crate) shader_storage_image_multisample: Bool32,
    pub(crate) shader_storage_image_read_without_format: Bool32,
    pub(crate) shader_storage_image_write_without_format: Bool32,
    pub(crate) shader_uniform_buffer_array_dynamic_indexing: Bool32,
    pub(crate) shader_sampled_image_array_dynamic_indexing: Bool32,
    pub(crate) shader_storage_buffer_array_dynamic_indexing: Bool32,
    pub(crate) shader_storage_image_array_dynamic_indexing: Bool32,
    pub(crate) shader_clip_distance: Bool32,
    pub(crate) shader_cull_distance: Bool32,
    pub(crate) shader_float_64: Bool32,
    pub(crate) shader_int_64: Bool32,
    pub(crate) shader_int_16: Bool32,
    pub(crate) shader_resource_residency: Bool32,
    pub(crate) shader_resource_min_lod: Bool32,
    pub(crate) sparse_binding: Bool32,
    pub(crate) sparse_residency_buffer: Bool32,
    pub(crate) sparse_residency_image_2_d: Bool32,
    pub(crate) sparse_residency_image_3_d: Bool32,
    pub(crate) sparse_residency_2_samples: Bool32,
    pub(crate) sparse_residency_4_samples: Bool32,
    pub(crate) sparse_residency_8_samples: Bool32,
    pub(crate) sparse_residency_16_samples: Bool32,
    pub(crate) sparse_residency_aliased: Bool32,
    pub(crate) variable_multisample_rate: Bool32,
    pub(crate) inherited_queries: Bool32,
}
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct SparseImageMemoryRequirements2 {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *mut c_void,
    pub(crate) memory_requirements: SparseImageMemoryRequirements,
}
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct SparseImageMemoryRequirements {
    pub(crate) format_properties: SparseImageFormatProperties,
    pub(crate) image_mip_tail_first_lod: u32,
    pub(crate) image_mip_tail_size: DeviceSize,
    pub(crate) image_mip_tail_offset: DeviceSize,
    pub(crate) image_mip_tail_stride: DeviceSize,
}
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct MemoryRequirements2 {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *mut c_void,
    pub(crate) memory_requirements: MemoryRequirements,
}
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct MemoryRequirements {
    pub(crate) size: DeviceSize,
    pub(crate) alignment: DeviceSize,
    pub(crate) memory_type_bits: u32,
}
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct ImageSparseMemoryRequirementsInfo2 {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *const c_void,
    pub(crate) image: *const Image,
}
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct ImageMemoryRequirementsInfo2 {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *const c_void,
    pub(crate) image: *const Image,
}
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct BufferMemoryRequirementsInfo2 {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *const c_void,
    pub(crate) buffer: *const Buffer,
}
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct DeviceGroupDeviceCreateInfo {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *const c_void,
    pub(crate) physical_device_count: u32,
    pub(crate) p_physical_devices: *const *const PhysicalDevice,
}
#[repr(transparent)]
pub struct PhysicalDevice([u8; 0]);
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct PhysicalDeviceGroupProperties {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *mut c_void,
    pub(crate) physical_device_count: u32,
    pub(crate) physical_devices: [*const PhysicalDevice; VK_MAX_DEVICE_GROUP_SIZE as usize],
    pub(crate) subset_allocation: Bool32,
}
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct BindImageMemoryDeviceGroupInfo {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *const c_void,
    pub(crate) device_index_count: u32,
    pub(crate) p_device_indices: *const u32,
    pub(crate) split_instance_bind_region_count: u32,
    pub(crate) p_split_instance_bind_regions: *const Rect2D,
}
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct BindBufferMemoryDeviceGroupInfo {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *const c_void,
    pub(crate) device_index_count: u32,
    pub(crate) p_device_indices: *const u32,
}
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct DeviceGroupBindSparseInfo {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *const c_void,
    pub(crate) resource_device_index: u32,
    pub(crate) memory_device_index: u32,
}
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct DeviceGroupSubmitInfo {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *const c_void,
    pub(crate) wait_semaphore_count: u32,
    pub(crate) p_wait_semaphore_device_indices: *const u32,
    pub(crate) command_buffer_count: u32,
    pub(crate) p_command_buffer_device_masks: *const u32,
    pub(crate) signal_semaphore_count: u32,
    pub(crate) p_signal_semaphore_device_indices: *const u32,
}
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct DeviceGroupCommandBufferBeginInfo {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *const c_void,
    pub(crate) device_mask: u32,
}
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct MemoryAllocateFlagsInfo {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *const c_void,
    pub(crate) flags: MemoryAllocateFlags,
    pub(crate) device_mask: u32,
}
flags!(MemoryAllocateBit, MemoryAllocateFlags, u32);
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[repr(i32)]
pub enum MemoryAllocateBit {
    DeviceMask = 1 << 0,
    DeviceAddress = 1 << 1,
    DeviceAddressCaptureReplay = 1 << 2,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[repr(i32)]
pub enum PeerMemoryFeatureBit {
    CopySrc = 1 << 0,
    CopyDst = 1 << 1,
    GenericSrc = 1 << 2,
    GenericDst = 1 << 3,
}

flags!(PeerMemoryFeatureBit, PeerMemoryFeatureFlags, u32);
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct MemoryDedicatedAllocateInfo {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *const c_void,
    pub(crate) image: *const Image,
    pub(crate) buffer: *const Buffer,
}
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct MemoryDedicatedRequirements {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *mut c_void,
    pub(crate) prefers_dedicated_allocation: Bool32,
    pub(crate) requires_dedicated_allocation: Bool32,
}
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct BindImageMemoryInfo {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *const c_void,
    pub(crate) image: *const Image,
    pub(crate) memory: *const DeviceMemory,
    pub(crate) memory_offset: DeviceSize,
}
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct BindBufferMemoryInfo {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *const c_void,
    pub(crate) buffer: *const Buffer,
    pub(crate) memory: *const DeviceMemory,
    pub(crate) memory_offset: DeviceSize,
}
#[repr(transparent)]
pub struct Device([u8; 0]);
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct DeviceCreateInfo {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *const c_void,
    pub(crate) flags: DeviceCreateFlags,
    pub(crate) queue_create_info_count: u32,
    pub(crate) p_queue_create_infos: *const DeviceQueueCreateInfo,
    pub(crate) enabled_layer_count: u32,
    pub(crate) pp_enabled_layer_names: *const *const c_char,
    pub(crate) enabled_extension_count: u32,
    pub(crate) pp_enabled_extension_names: *const *const c_char,
    pub(crate) p_enabled_features: *const PhysicalDeviceFeatures,
}
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct DeviceQueueCreateInfo {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *const c_void,
    pub(crate) flags: DeviceQueueCreateFlags,
    pub(crate) queue_family_index: u32,
    pub(crate) queue_count: u32,
    pub(crate) p_queue_priorities: *const f32,
}
flags!(DeviceCreateFlags, u32);
#[repr(transparent)]
pub struct Queue([u8; 0]);
#[repr(transparent)]
pub struct CommandPool([u8; 0]);
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct CommandPoolCreateInfo {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *const c_void,
    pub(crate) flags: CommandPoolCreateFlags,
    pub(crate) queue_family_index: u32,
}
flags!(CommandPoolCreateBit, CommandPoolCreateFlags, u32);
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[repr(i32)]
pub enum CommandPoolCreateBit {
    Transient = 1 << 0,
    ResetCommandBuffer = 1 << 1,
    Protected = 1 << 2,
}

flags!(CommandPoolResetBit, CommandPoolResetFlags, u32);
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[repr(i32)]
pub enum CommandPoolResetBit {
    ReleaseResources = 1 << 0,
}

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct CommandBufferAllocateInfo {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *const c_void,
    pub(crate) command_pool: *const CommandPool,
    pub(crate) level: CommandBufferLevel,
    pub(crate) command_buffer_count: u32,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[repr(i32)]
pub enum CommandBufferLevel {
    Primary = 0,
    Secondary = 1,
}

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct MemoryAllocateInfo {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *const c_void,
    pub(crate) allocation_size: DeviceSize,
    pub(crate) memory_type_index: u32,
}
flags!(CullModeBit, CullModeFlags, u32);
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[repr(i32)]
pub enum CullModeBit {
    Front = 1 << 0,
    Back = 1 << 1,
}

impl CullModeFlags {
    pub const NONE: Self = Self(0);
    pub const FRONT_AND_BACK: Self = Self(3);
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[repr(i32)]
pub enum PolygonMode {
    Fill = 0,
    Line = 1,
    Point = 2,
}

pub type SampleMask = u32;
flags!(ColorComponentBit, ColorComponentFlags, u32);
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[repr(i32)]
pub enum ColorComponentBit {
    R = 1 << 0,
    G = 1 << 1,
    B = 1 << 2,
    A = 1 << 3,
}

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct Viewport {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub min_depth: f32,
    pub max_depth: f32,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[repr(i32)]
pub enum PrimitiveTopology {
    PointList = 0,
    LineList = 1,
    LineStrip = 2,
    TriangleList = 3,
    TriangleStrip = 4,
    TriangleFan = 5,
    LineListWithAdjacency = 6,
    LineStripWithAdjacency = 7,
    TriangleListWithAdjacency = 8,
    TriangleStripWithAdjacency = 9,
    PatchList = 10,
}

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct FenceCreateInfo {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *const c_void,
    pub(crate) flags: FenceCreateFlags,
}
flags!(FenceCreateBit, FenceCreateFlags, u32);
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[repr(i32)]
pub enum FenceCreateBit {
    Signaled = 1 << 0,
}

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct SemaphoreCreateInfo {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *const c_void,
    pub(crate) flags: SemaphoreCreateFlags,
}
flags!(SemaphoreCreateFlags, u32);
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct SubmitInfo {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *const c_void,
    pub(crate) wait_semaphore_count: u32,
    pub(crate) p_wait_semaphores: *const *const Semaphore,
    pub(crate) p_wait_dst_stage_mask: *const PipelineStageFlags,
    pub(crate) command_buffer_count: u32,
    pub(crate) p_command_buffers: *const *const CommandBuffer,
    pub(crate) signal_semaphore_count: u32,
    pub(crate) p_signal_semaphores: *const *const Semaphore,
}
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct CommandBufferBeginInfo {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *const c_void,
    pub(crate) flags: CommandBufferUsageFlags,
    pub(crate) p_inheritance_info: *const CommandBufferInheritanceInfo,
}
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct CommandBufferInheritanceInfo {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *const c_void,
    pub(crate) render_pass: *const RenderPass,
    pub(crate) subpass: u32,
    pub(crate) framebuffer: *const Framebuffer,
    pub(crate) occlusion_query_enable: Bool32,
    pub(crate) query_flags: QueryControlFlags,
    pub(crate) pipeline_statistics: QueryPipelineStatisticFlags,
}
flags!(QueryPipelineStatisticBit, QueryPipelineStatisticFlags, u32);
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[repr(i32)]
pub enum QueryPipelineStatisticBit {
    InputAssemblyVertices = 1 << 0,
    InputAssemblyPrimitives = 1 << 1,
    VertexShaderInvocations = 1 << 2,
    GeometryShaderInvocations = 1 << 3,
    GeometryShaderPrimitives = 1 << 4,
    ClippingInvocations = 1 << 5,
    ClippingPrimitives = 1 << 6,
    FragmentShaderInvocations = 1 << 7,
    TessellationControlShaderPatches = 1 << 8,
    TessellationEvaluationShaderInvocations = 1 << 9,
    ComputeShaderInvocations = 1 << 10,
}

flags!(QueryControlBit, QueryControlFlags, u32);
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[repr(i32)]
pub enum QueryControlBit {
    Precise = 1 << 0,
}

#[repr(transparent)]
pub struct Framebuffer([u8; 0]);
#[repr(transparent)]
pub struct RenderPass([u8; 0]);
flags!(CommandBufferUsageBit, CommandBufferUsageFlags, u32);
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[repr(i32)]
pub enum CommandBufferUsageBit {
    OneTimeSubmit = 1 << 0,
    RenderPassContinue = 1 << 1,
    SimultaneousUse = 1 << 2,
}

#[repr(transparent)]
pub struct AccelerationStructure([u8; 0]);
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct AccelerationStructureCreateInfo {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *const c_void,
    pub(crate) create_flags: AccelerationStructureCreateFlags,
    pub(crate) buffer: *const Buffer,
    pub(crate) offset: DeviceSize,
    pub(crate) size: DeviceSize,
    pub(crate) r#type: AccelerationStructureType,
    pub(crate) device_address: DeviceAddress,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[repr(i32)]
pub enum AccelerationStructureType {
    TopLevel = 0,
    BottomLevel = 1,
    Generic = 2,
}

flags!(
    AccelerationStructureCreateBit,
    AccelerationStructureCreateFlags,
    u32
);
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[repr(i32)]
pub enum AccelerationStructureCreateBit {
    DeviceAddressCaptureReplay = 1 << 0,
}

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct AccelerationStructureBuildRangeInfo {
    pub(crate) primitive_count: u32,
    pub(crate) primitive_offset: u32,
    pub(crate) first_vertex: u32,
    pub(crate) transform_offset: u32,
}
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct AccelerationStructureBuildGeometryInfo {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *const c_void,
    pub(crate) r#type: AccelerationStructureType,
    pub(crate) flags: BuildAccelerationStructureFlags,
    pub(crate) mode: BuildAccelerationStructureMode,
    pub(crate) src_acceleration_structure: *const AccelerationStructure,
    pub(crate) dst_acceleration_structure: *const AccelerationStructure,
    pub(crate) geometry_count: u32,
    pub(crate) p_geometries: *const AccelerationStructureGeometry,
    pub(crate) pp_geometries: *const *const AccelerationStructureGeometry,
    pub(crate) scratch_data: DeviceOrHostAddress,
}
impl std::fmt::Debug for DeviceOrHostAddress {
    fn fmt(&self, _writer: &mut std::fmt::Formatter) -> std::result::Result<(), std::fmt::Error> {
        Ok(())
    }
}
#[derive(Clone, Copy)]
#[repr(C)]
pub union DeviceOrHostAddress {
    pub(crate) device_address: DeviceAddress,
    pub(crate) host_address: *mut c_void,
}
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct AccelerationStructureGeometry {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *const c_void,
    pub(crate) geometry_type: GeometryType,
    pub(crate) geometry: AccelerationStructureGeometryData,
    pub(crate) flags: GeometryFlags,
}
flags!(GeometryBit, GeometryFlags, u32);
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[repr(i32)]
pub enum GeometryBit {
    Opaque = 1 << 0,
    NoDuplicateAnyHitInvocation = 1 << 1,
}

impl std::fmt::Debug for AccelerationStructureGeometryData {
    fn fmt(&self, _writer: &mut std::fmt::Formatter) -> std::result::Result<(), std::fmt::Error> {
        Ok(())
    }
}
#[derive(Clone, Copy)]
#[repr(C)]
pub union AccelerationStructureGeometryData {
    pub(crate) triangles: AccelerationStructureGeometryTrianglesData,
    pub(crate) aabbs: AccelerationStructureGeometryAabbsData,
    pub(crate) instances: AccelerationStructureGeometryInstancesData,
}
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct AccelerationStructureGeometryInstancesData {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *const c_void,
    pub(crate) array_of_pointers: Bool32,
    pub(crate) data: DeviceOrHostAddressConst,
}
impl std::fmt::Debug for DeviceOrHostAddressConst {
    fn fmt(&self, _writer: &mut std::fmt::Formatter) -> std::result::Result<(), std::fmt::Error> {
        Ok(())
    }
}
#[derive(Clone, Copy)]
#[repr(C)]
pub union DeviceOrHostAddressConst {
    pub(crate) device_address: DeviceAddress,
    pub(crate) host_address: *const c_void,
}
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct AccelerationStructureGeometryAabbsData {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *const c_void,
    pub(crate) data: DeviceOrHostAddressConst,
    pub(crate) stride: DeviceSize,
}
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct AccelerationStructureGeometryTrianglesData {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *const c_void,
    pub(crate) vertex_format: Format,
    pub(crate) vertex_data: DeviceOrHostAddressConst,
    pub(crate) vertex_stride: DeviceSize,
    pub(crate) max_vertex: u32,
    pub(crate) index_type: IndexType,
    pub(crate) index_data: DeviceOrHostAddressConst,
    pub(crate) transform_data: DeviceOrHostAddressConst,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[repr(i32)]
pub enum IndexType {
    Uint16 = 0,
    Uint32 = 1,
    Uint8 = 1000265000,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[repr(i32)]
pub enum GeometryType {
    Triangles = 0,
    Aabbs = 1,
    Instances = 2,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[repr(i32)]
pub enum BuildAccelerationStructureMode {
    Build = 0,
    Update = 1,
}

flags!(
    BuildAccelerationStructureBit,
    BuildAccelerationStructureFlags,
    u32
);
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[repr(i32)]
pub enum BuildAccelerationStructureBit {
    AllowUpdate = 1 << 0,
    AllowCompaction = 1 << 1,
    PreferFastTrace = 1 << 2,
    PreferFastBuild = 1 << 3,
    LowMemory = 1 << 4,
}

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct AccelerationStructureBuildSizesInfo {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *mut c_void,
    pub(crate) acceleration_structure_size: DeviceSize,
    pub(crate) update_scratch_size: DeviceSize,
    pub(crate) build_scratch_size: DeviceSize,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[repr(i32)]
pub enum AccelerationStructureBuildType {
    Host = 0,
    Device = 1,
    HostOrDevice = 2,
}

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct AccelerationStructureDeviceAddressInfo {
    pub(crate) s_type: StructureType,
    pub(crate) p_next: *const c_void,
    pub(crate) acceleration_structure: *const AccelerationStructure,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[repr(i32)]
pub enum QueryType {
    Occlusion = 0,
    PipelineStatistics = 1,
    Timestamp = 2,
}

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct StridedDeviceAddressRegion {
    pub(crate) device_address: DeviceAddress,
    pub(crate) stride: DeviceSize,
    pub(crate) size: DeviceSize,
}
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct DrawIndirectCommand {
    pub vertex_count: u32,
    pub instance_count: u32,
    pub first_vertex: u32,
    pub first_instance: u32,
}
pub struct LibraryFns {
    pub create_instance: unsafe extern "system" fn(
        p_create_info: *const InstanceCreateInfo,
        p_allocator: *const AllocationCallbacks,
        p_instance: *mut *const Instance,
    ) -> Result,
}
impl LibraryFns {
    pub unsafe fn load_with(mut load_sym: impl FnMut(&CStr) -> *const u8) -> Self {
        unsafe {
            Self {
                create_instance: mem::transmute(load_sym(c"vkCreateInstance")),
            }
        }
    }
}
pub struct InstanceFns {
    pub enumerate_physical_devices: unsafe extern "system" fn(
        instance: *const Instance,
        p_physical_device_count: *mut u32,
        p_physical_devices: *mut *const PhysicalDevice,
    ) -> Result,
    pub get_physical_device_properties2: unsafe extern "system" fn(
        physical_device: *const PhysicalDevice,
        p_properties: *mut PhysicalDeviceProperties2,
    ),
    pub get_physical_device_format_properties2: unsafe extern "system" fn(
        physical_device: *const PhysicalDevice,
        format: Format,
        p_format_properties: *mut FormatProperties2,
    ),
    pub get_physical_device_queue_family_properties: unsafe extern "system" fn(
        physical_device: *const PhysicalDevice,
        p_queue_family_property_count: *mut u32,
        p_queue_family_properties: *mut QueueFamilyProperties,
    ),
    pub get_physical_device_memory_properties: unsafe extern "system" fn(
        physical_device: *const PhysicalDevice,
        p_memory_properties: *mut PhysicalDeviceMemoryProperties,
    ),
    pub create_device: unsafe extern "system" fn(
        physical_device: *const PhysicalDevice,
        p_create_info: *const DeviceCreateInfo,
        p_allocator: *const AllocationCallbacks,
        p_device: *mut *const Device,
    ) -> Result,
    pub destroy_instance: unsafe extern "system" fn(
        instance: *const Instance,
        p_allocator: *const AllocationCallbacks,
    ),
}
impl InstanceFns {
    pub unsafe fn load_with(mut load_sym: impl FnMut(&CStr) -> *const u8) -> Self {
        unsafe {
            Self {
                enumerate_physical_devices: mem::transmute(load_sym(c"vkEnumeratePhysicalDevices")),
                get_physical_device_properties2: mem::transmute(load_sym(
                    c"vkGetPhysicalDeviceProperties2",
                )),
                get_physical_device_format_properties2: mem::transmute(load_sym(
                    c"vkGetPhysicalDeviceFormatProperties2",
                )),
                get_physical_device_queue_family_properties: mem::transmute(load_sym(
                    c"vkGetPhysicalDeviceQueueFamilyProperties",
                )),
                get_physical_device_memory_properties: mem::transmute(load_sym(
                    c"vkGetPhysicalDeviceMemoryProperties",
                )),
                create_device: mem::transmute(load_sym(c"vkCreateDevice")),
                destroy_instance: mem::transmute(load_sym(c"vkDestroyInstance")),
            }
        }
    }
}
pub struct DeviceFns {
    pub get_image_drm_format_modifier_properties: unsafe extern "system" fn(
        device: *const Device,
        image: *const Image,
        p_properties: *mut ImageDrmFormatModifierProperties,
    ) -> Result,
    pub get_memory_fd: unsafe extern "system" fn(
        device: *const Device,
        p_get_fd_info: *const MemoryGetFdInfo,
        p_fd: *mut i32,
    ) -> Result,
    pub get_semaphore_fd: unsafe extern "system" fn(
        device: *const Device,
        p_get_fd_info: *const SemaphoreGetFdInfo,
        p_fd: *mut i32,
    ) -> Result,
    pub import_semaphore_fd: unsafe extern "system" fn(
        device: *const Device,
        p_import_semaphore_fd_info: *const ImportSemaphoreFdInfo,
    ) -> Result,
    // pub import_semaphore_win32_handle: unsafe extern "system" fn(
    //     device: *const Device,
    //     p_import_semaphore_win32_handle_info: *const ImportSemaphoreWin32HandleInfo,
    // ) -> Result,
    #[cfg(feature = "ray-tracing")]
    rtx: DeviceRtxFns,
    pub get_device_queue: unsafe extern "system" fn(
        device: *const Device,
        queue_family_index: u32,
        queue_index: u32,
        p_queue: *mut *const Queue,
    ),
    pub create_command_pool: unsafe extern "system" fn(
        device: *const Device,
        p_create_info: *const CommandPoolCreateInfo,
        p_allocator: *const AllocationCallbacks,
        p_command_pool: *mut *const CommandPool,
    ) -> Result,
    pub reset_command_pool: unsafe extern "system" fn(
        device: *const Device,
        command_pool: *const CommandPool,
        flags: CommandPoolResetFlags,
    ) -> Result,
    pub allocate_command_buffers: unsafe extern "system" fn(
        device: *const Device,
        p_allocate_info: *const CommandBufferAllocateInfo,
        p_command_buffers: *mut *const CommandBuffer,
    ) -> Result,
    pub create_image: unsafe extern "system" fn(
        device: *const Device,
        p_create_info: *const ImageCreateInfo,
        p_allocator: *const AllocationCallbacks,
        p_image: *mut *const Image,
    ) -> Result,
    pub create_buffer: unsafe extern "system" fn(
        device: *const Device,
        p_create_info: *const BufferCreateInfo,
        p_allocator: *const AllocationCallbacks,
        p_buffer: *mut *const Buffer,
    ) -> Result,
    pub bind_buffer_memory: unsafe extern "system" fn(
        device: *const Device,
        buffer: *const Buffer,
        memory: *const DeviceMemory,
        memory_offset: DeviceSize,
    ) -> Result,
    pub get_buffer_memory_requirements: unsafe extern "system" fn(
        device: *const Device,
        buffer: *const Buffer,
        p_memory_requirements: *mut MemoryRequirements,
    ),
    pub bind_image_memory: unsafe extern "system" fn(
        device: *const Device,
        image: *const Image,
        memory: *const DeviceMemory,
        memory_offset: DeviceSize,
    ) -> Result,
    pub get_image_memory_requirements2: unsafe extern "system" fn(
        device: *const Device,
        p_info: *const ImageMemoryRequirementsInfo2,
        p_memory_requirements: *mut MemoryRequirements2,
    ),
    pub get_image_subresource_layout: unsafe extern "system" fn(
        device: *const Device,
        image: *const Image,
        p_subresource: *const ImageSubresource,
        p_layout: *mut SubresourceLayout,
    ),
    pub allocate_memory: unsafe extern "system" fn(
        device: *const Device,
        p_allocate_info: *const MemoryAllocateInfo,
        p_allocator: *const AllocationCallbacks,
        p_memory: *mut *const DeviceMemory,
    ) -> Result,
    pub map_memory: unsafe extern "system" fn(
        device: *const Device,
        memory: *const DeviceMemory,
        offset: DeviceSize,
        size: DeviceSize,
        flags: MemoryMapFlags,
        pp_data: *mut *mut c_void,
    ) -> Result,
    pub create_image_view: unsafe extern "system" fn(
        device: *const Device,
        p_create_info: *const ImageViewCreateInfo,
        p_allocator: *const AllocationCallbacks,
        p_view: *mut *const ImageView,
    ) -> Result,
    pub cmd_begin_rendering: unsafe extern "system" fn(
        command_buffer: *const CommandBuffer,
        p_rendering_info: *const RenderingInfo,
    ),
    pub cmd_end_rendering: unsafe extern "system" fn(command_buffer: *const CommandBuffer),
    pub cmd_set_rasterizer_discard_enable: unsafe extern "system" fn(
        command_buffer: *const CommandBuffer,
        rasterizer_discard_enable: Bool32,
    ),
    pub cmd_set_cull_mode:
        unsafe extern "system" fn(command_buffer: *const CommandBuffer, cull_mode: CullModeFlags),
    pub cmd_set_depth_test_enable:
        unsafe extern "system" fn(command_buffer: *const CommandBuffer, depth_test_enable: Bool32),
    pub cmd_set_stencil_test_enable: unsafe extern "system" fn(
        command_buffer: *const CommandBuffer,
        stencil_test_enable: Bool32,
    ),
    pub cmd_set_depth_bias_enable:
        unsafe extern "system" fn(command_buffer: *const CommandBuffer, depth_bias_enable: Bool32),
    pub cmd_set_polygon_mode_ext:
        unsafe extern "system" fn(command_buffer: *const CommandBuffer, polygon_mode: PolygonMode),
    pub cmd_set_line_width:
        unsafe extern "system" fn(command_buffer: *const CommandBuffer, line_width: f32),
    pub cmd_set_rasterization_samples_ext: unsafe extern "system" fn(
        command_buffer: *const CommandBuffer,
        rasterization_samples: SampleCountBit,
    ),
    pub cmd_set_sample_mask_ext: unsafe extern "system" fn(
        command_buffer: *const CommandBuffer,
        samples: SampleCountBit,
        p_sample_mask: *const SampleMask,
    ),
    pub cmd_set_alpha_to_coverage_enable_ext: unsafe extern "system" fn(
        command_buffer: *const CommandBuffer,
        alpha_to_coverage_enable: Bool32,
    ),
    pub cmd_set_color_blend_enable_ext: unsafe extern "system" fn(
        command_buffer: *const CommandBuffer,
        first_attachment: u32,
        attachment_count: u32,
        p_color_blend_enables: *const Bool32,
    ),
    pub cmd_set_color_write_mask_ext: unsafe extern "system" fn(
        command_buffer: *const CommandBuffer,
        first_attachment: u32,
        attachment_count: u32,
        p_color_write_masks: *const ColorComponentFlags,
    ),
    pub cmd_set_viewport_with_count: unsafe extern "system" fn(
        command_buffer: *const CommandBuffer,
        viewport_count: u32,
        p_viewports: *const Viewport,
    ),
    pub cmd_set_scissor_with_count: unsafe extern "system" fn(
        command_buffer: *const CommandBuffer,
        scissor_count: u32,
        p_scissors: *const Rect2D,
    ),
    pub cmd_set_primitive_topology: unsafe extern "system" fn(
        command_buffer: *const CommandBuffer,
        primitive_topology: PrimitiveTopology,
    ),
    pub cmd_set_primitive_restart_enable: unsafe extern "system" fn(
        command_buffer: *const CommandBuffer,
        primitive_restart_enable: Bool32,
    ),
    pub cmd_set_vertex_input_ext: unsafe extern "system" fn(
        command_buffer: *const CommandBuffer,
        vertex_binding_description_count: u32,
        p_vertex_binding_descriptions: *const VertexInputBindingDescription2,
        vertex_attribute_description_count: u32,
        p_vertex_attribute_descriptions: *const VertexInputAttributeDescription2,
    ),
    pub cmd_set_color_blend_equation_ext: unsafe extern "system" fn(
        command_buffer: *const CommandBuffer,
        first_attachment: u32,
        attachment_count: u32,
        p_color_blend_equations: *const ColorBlendEquation,
    ),
    pub cmd_draw: unsafe extern "system" fn(
        command_buffer: *const CommandBuffer,
        vertex_count: u32,
        instance_count: u32,
        first_vertex: u32,
        first_instance: u32,
    ),
    pub cmd_draw_indexed: unsafe extern "system" fn(
        command_buffer: *const CommandBuffer,
        index_count: u32,
        instance_count: u32,
        first_index: u32,
        vertex_offset: i32,
        first_instance: u32,
    ),
    pub cmd_draw_indexed_indirect: unsafe extern "system" fn(
        command_buffer: *const CommandBuffer,
        buffer: *const Buffer,
        offset: DeviceSize,
        draw_count: u32,
        stride: u32,
    ),
    pub cmd_draw_indexed_indirect_count: unsafe extern "system" fn(
        command_buffer: *const CommandBuffer,
        buffer: *const Buffer,
        offset: DeviceSize,
        count_buffer: *const Buffer,
        count_buffer_offset: DeviceSize,
        max_draw_count: u32,
        stride: u32,
    ),
    pub cmd_draw_indirect: unsafe extern "system" fn(
        command_buffer: *const CommandBuffer,
        buffer: *const Buffer,
        offset: DeviceSize,
        draw_count: u32,
        stride: u32,
    ),
    pub cmd_draw_indirect_count: unsafe extern "system" fn(
        command_buffer: *const CommandBuffer,
        buffer: *const Buffer,
        offset: DeviceSize,
        count_buffer: *const Buffer,
        count_buffer_offset: DeviceSize,
        max_draw_count: u32,
        stride: u32,
    ),
    // pub cmd_draw_mesh_tasks_ext: unsafe extern "system" fn(
    //     command_buffer: *const CommandBuffer,
    //     group_count_x: u32,
    //     group_count_y: u32,
    //     group_count_z: u32,
    // ),
    // pub cmd_draw_mesh_tasks_indirect_ext: unsafe extern "system" fn(
    //     command_buffer: *const CommandBuffer,
    //     buffer: *const Buffer,
    //     offset: DeviceSize,
    //     draw_count: u32,
    //     stride: u32,
    // ),
    // pub cmd_draw_mesh_tasks_indirect_count_ext: unsafe extern "system" fn(
    //     command_buffer: *const CommandBuffer,
    //     buffer: *const Buffer,
    //     offset: DeviceSize,
    //     count_buffer: *const Buffer,
    //     count_buffer_offset: DeviceSize,
    //     max_draw_count: u32,
    //     stride: u32,
    // ),
    pub create_fence: unsafe extern "system" fn(
        device: *const Device,
        p_create_info: *const FenceCreateInfo,
        p_allocator: *const AllocationCallbacks,
        p_fence: *mut *const Fence,
    ) -> Result,
    pub create_semaphore: unsafe extern "system" fn(
        device: *const Device,
        p_create_info: *const SemaphoreCreateInfo,
        p_allocator: *const AllocationCallbacks,
        p_semaphore: *mut *const Semaphore,
    ) -> Result,
    pub reset_fences: unsafe extern "system" fn(
        device: *const Device,
        fence_count: u32,
        p_fences: *const *const Fence,
    ) -> Result,
    pub queue_submit_2: unsafe extern "system" fn(
        queue: *const Queue,
        submit_count: u32,
        p_submits: *const SubmitInfo2,
        fence: *const Fence,
    ) -> Result,
    pub queue_submit: unsafe extern "system" fn(
        queue: *const Queue,
        submit_count: u32,
        p_submits: *const SubmitInfo,
        fence: *const Fence,
    ) -> Result,
    pub device_wait_idle: unsafe extern "system" fn(device: *const Device) -> Result,
    pub begin_command_buffer: unsafe extern "system" fn(
        command_buffer: *const CommandBuffer,
        p_begin_info: *const CommandBufferBeginInfo,
    ) -> Result,
    pub end_command_buffer:
        unsafe extern "system" fn(command_buffer: *const CommandBuffer) -> Result,
    pub wait_for_fences: unsafe extern "system" fn(
        device: *const Device,
        fence_count: u32,
        p_fences: *const *const Fence,
        wait_all: Bool32,
        timeout: u64,
    ) -> Result,
    pub wait_semaphores: unsafe extern "system" fn(
        device: *const Device,
        p_wait_info: *const SemaphoreWaitInfo,
        timeout: u64,
    ) -> Result,
    pub cmd_blit_image_2: unsafe extern "system" fn(
        command_buffer: *const CommandBuffer,
        p_blit_image_info: *const BlitImageInfo2,
    ),
    pub cmd_copy_image_to_buffer_2: unsafe extern "system" fn(
        command_buffer: *const CommandBuffer,
        p_copy_image_to_buffer_info: *const CopyImageToBufferInfo2,
    ),
    pub cmd_copy_buffer_2: unsafe extern "system" fn(
        command_buffer: *const CommandBuffer,
        p_copy_buffer_info: *const CopyBufferInfo2,
    ),
    pub cmd_copy_image_2: unsafe extern "system" fn(
        command_buffer: *const CommandBuffer,
        p_copy_image_info: *const CopyImageInfo2,
    ),
    pub cmd_copy_buffer_to_image_2: unsafe extern "system" fn(
        command_buffer: *const CommandBuffer,
        p_copy_buffer_to_image_info: *const CopyBufferToImageInfo2,
    ),
    pub cmd_resolve_image_2: unsafe extern "system" fn(
        command_buffer: *const CommandBuffer,
        p_resolve_image_info: *const ResolveImageInfo2,
    ),
    pub cmd_pipeline_barrier_2: unsafe extern "system" fn(
        command_buffer: *const CommandBuffer,
        p_dependency_info: *const DependencyInfo,
    ),
    pub get_buffer_device_address: unsafe extern "system" fn(
        device: *const Device,
        p_info: *const BufferDeviceAddressInfo,
    ) -> DeviceAddress,
    pub get_semaphore_counter_value: unsafe extern "system" fn(
        device: *const Device,
        semaphore: *const Semaphore,
        p_value: *mut u64,
    ) -> Result,
    pub signal_semaphore: unsafe extern "system" fn(
        device: *const Device,
        p_signal_info: *const SemaphoreSignalInfo,
    ) -> Result,
    pub create_shaders_ext: unsafe extern "system" fn(
        device: *const Device,
        create_info_count: u32,
        p_create_infos: *const ShaderCreateInfo,
        p_allocator: *const AllocationCallbacks,
        p_shaders: *mut *const Shader,
    ) -> Result,
    pub cmd_bind_shaders_ext: unsafe extern "system" fn(
        command_buffer: *const CommandBuffer,
        stage_count: u32,
        p_stages: *const ShaderStageBit,
        p_shaders: *const *const Shader,
    ),
    pub cmd_dispatch: unsafe extern "system" fn(
        command_buffer: *const CommandBuffer,
        group_count_x: u32,
        group_count_y: u32,
        group_count_z: u32,
    ),
    pub cmd_dispatch_base: unsafe extern "system" fn(
        command_buffer: *const CommandBuffer,
        base_group_x: u32,
        base_group_y: u32,
        base_group_z: u32,
        group_count_x: u32,
        group_count_y: u32,
        group_count_z: u32,
    ),
    pub cmd_dispatch_indirect: unsafe extern "system" fn(
        command_buffer: *const CommandBuffer,
        buffer: *const Buffer,
        offset: DeviceSize,
    ),
    pub destroy_image: unsafe extern "system" fn(
        device: *const Device,
        image: *const Image,
        p_allocator: *const AllocationCallbacks,
    ),
    pub destroy_image_view: unsafe extern "system" fn(
        device: *const Device,
        image_view: *const ImageView,
        p_allocator: *const AllocationCallbacks,
    ),
    pub destroy_buffer: unsafe extern "system" fn(
        device: *const Device,
        buffer: *const Buffer,
        p_allocator: *const AllocationCallbacks,
    ),
    pub destroy_device:
        unsafe extern "system" fn(device: *const Device, p_allocator: *const AllocationCallbacks),
    pub destroy_shader_ext: unsafe extern "system" fn(
        device: *const Device,
        shader: *const Shader,
        p_allocator: *const AllocationCallbacks,
    ),
    pub free_memory: unsafe extern "system" fn(
        device: *const Device,
        memory: *const DeviceMemory,
        p_allocator: *const AllocationCallbacks,
    ),
    pub free_command_buffers: unsafe extern "system" fn(
        device: *const Device,
        command_pool: *const CommandPool,
        command_buffer_count: u32,
        p_command_buffers: *const *const CommandBuffer,
    ),
    pub destroy_fence: unsafe extern "system" fn(
        device: *const Device,
        fence: *const Fence,
        p_allocator: *const AllocationCallbacks,
    ),
    pub destroy_semaphore: unsafe extern "system" fn(
        device: *const Device,
        semaphore: *const Semaphore,
        p_allocator: *const AllocationCallbacks,
    ),
    pub destroy_command_pool: unsafe extern "system" fn(
        device: *const Device,
        command_pool: *const CommandPool,
        p_allocator: *const AllocationCallbacks,
    ),
    pub cmd_push_data_ext: unsafe extern "system" fn(
        command_buffer: *const CommandBuffer,
        p_push_data_info: *const PushDataInfo,
    ),
    pub cmd_bind_resource_heap_ext: unsafe extern "system" fn(
        command_buffer: *const CommandBuffer,
        p_bind_info: *const BindHeapInfo,
    ),
    pub cmd_bind_sampler_heap_ext: unsafe extern "system" fn(
        command_buffer: *const CommandBuffer,
        p_bind_info: *const BindHeapInfo,
    ),
    pub write_resource_descriptors_ext: unsafe extern "system" fn(
        device: *const Device,
        resource_count: u32,
        p_resources: *const ResourceDescriptorInfo,
        p_descriptors: *const HostAddressRange,
    ) -> Result,
    pub write_sampler_descriptors_ext: unsafe extern "system" fn(
        device: *const Device,
        sampler_count: u32,
        p_samplers: *const SamplerCreateInfo,
        p_descriptors: *const HostAddressRange,
    ) -> Result,
}

impl DeviceFns {
    pub unsafe fn load_with(mut load_sym: impl FnMut(&CStr) -> *const u8) -> Self {
        unsafe {
            Self {
                get_image_drm_format_modifier_properties: mem::transmute(load_sym(
                    c"vkGetImageDrmFormatModifierPropertiesEXT",
                )),
                get_memory_fd: mem::transmute(load_sym(c"vkGetMemoryFdKHR")),
                get_semaphore_fd: mem::transmute(load_sym(c"vkGetSemaphoreFdKHR")),
                import_semaphore_fd: mem::transmute(load_sym(c"vkImportSemaphoreFdKHR")),
                // import_semaphore_win32_handle: mem::transmute(load_sym(
                //     c"vkImportSemaphoreWin32HandleKHR",
                // )),
                get_device_queue: mem::transmute(load_sym(c"vkGetDeviceQueue")),
                create_command_pool: mem::transmute(load_sym(c"vkCreateCommandPool")),
                reset_command_pool: mem::transmute(load_sym(c"vkResetCommandPool")),
                allocate_command_buffers: mem::transmute(load_sym(c"vkAllocateCommandBuffers")),
                create_image: mem::transmute(load_sym(c"vkCreateImage")),
                create_buffer: mem::transmute(load_sym(c"vkCreateBuffer")),
                bind_buffer_memory: mem::transmute(load_sym(c"vkBindBufferMemory")),
                get_buffer_memory_requirements: mem::transmute(load_sym(
                    c"vkGetBufferMemoryRequirements",
                )),
                bind_image_memory: mem::transmute(load_sym(c"vkBindImageMemory")),
                get_image_memory_requirements2: mem::transmute(load_sym(
                    c"vkGetImageMemoryRequirements2",
                )),
                get_image_subresource_layout: mem::transmute(load_sym(
                    c"vkGetImageSubresourceLayout",
                )),
                allocate_memory: mem::transmute(load_sym(c"vkAllocateMemory")),
                map_memory: mem::transmute(load_sym(c"vkMapMemory")),
                create_image_view: mem::transmute(load_sym(c"vkCreateImageView")),
                cmd_begin_rendering: mem::transmute(load_sym(c"vkCmdBeginRendering")),
                cmd_end_rendering: mem::transmute(load_sym(c"vkCmdEndRendering")),
                cmd_set_rasterizer_discard_enable: mem::transmute(load_sym(
                    c"vkCmdSetRasterizerDiscardEnable",
                )),
                cmd_set_cull_mode: mem::transmute(load_sym(c"vkCmdSetCullMode")),
                cmd_set_depth_test_enable: mem::transmute(load_sym(c"vkCmdSetDepthTestEnable")),
                cmd_set_stencil_test_enable: mem::transmute(load_sym(c"vkCmdSetStencilTestEnable")),
                cmd_set_depth_bias_enable: mem::transmute(load_sym(c"vkCmdSetDepthBiasEnable")),
                cmd_set_polygon_mode_ext: mem::transmute(load_sym(c"vkCmdSetPolygonModeEXT")),
                cmd_set_line_width: mem::transmute(load_sym(c"vkCmdSetLineWidth")),
                cmd_set_rasterization_samples_ext: mem::transmute(load_sym(
                    c"vkCmdSetRasterizationSamplesEXT",
                )),
                cmd_set_sample_mask_ext: mem::transmute(load_sym(c"vkCmdSetSampleMaskEXT")),
                cmd_set_alpha_to_coverage_enable_ext: mem::transmute(load_sym(
                    c"vkCmdSetAlphaToCoverageEnableEXT",
                )),
                cmd_set_color_blend_enable_ext: mem::transmute(load_sym(
                    c"vkCmdSetColorBlendEnableEXT",
                )),
                cmd_set_color_write_mask_ext: mem::transmute(load_sym(
                    c"vkCmdSetColorWriteMaskEXT",
                )),
                cmd_set_viewport_with_count: mem::transmute(load_sym(c"vkCmdSetViewportWithCount")),
                cmd_set_scissor_with_count: mem::transmute(load_sym(c"vkCmdSetScissorWithCount")),
                cmd_set_primitive_topology: mem::transmute(load_sym(c"vkCmdSetPrimitiveTopology")),
                cmd_set_primitive_restart_enable: mem::transmute(load_sym(
                    c"vkCmdSetPrimitiveRestartEnable",
                )),
                cmd_set_vertex_input_ext: mem::transmute(load_sym(c"vkCmdSetVertexInputEXT")),
                cmd_set_color_blend_equation_ext: mem::transmute(load_sym(
                    c"vkCmdSetColorBlendEquationEXT",
                )),
                cmd_draw: mem::transmute(load_sym(c"vkCmdDraw")),
                cmd_draw_indexed: mem::transmute(load_sym(c"vkCmdDrawIndexed")),
                cmd_draw_indexed_indirect: mem::transmute(load_sym(c"vkCmdDrawIndexedIndirect")),
                cmd_draw_indexed_indirect_count: mem::transmute(load_sym(
                    c"vkCmdDrawIndexedIndirectCount",
                )),
                cmd_draw_indirect: mem::transmute(load_sym(c"vkCmdDrawIndirect")),
                cmd_draw_indirect_count: mem::transmute(load_sym(c"vkCmdDrawIndirectCount")),
                // cmd_draw_mesh_tasks_ext: mem::transmute(load_sym(c"vkCmdDrawMeshTasksEXT")),
                // cmd_draw_mesh_tasks_indirect_ext: mem::transmute(load_sym(
                //     c"vkCmdDrawMeshTasksIndirectEXT",
                // )),
                // cmd_draw_mesh_tasks_indirect_count_ext: mem::transmute(load_sym(
                //     c"vkCmdDrawMeshTasksIndirectCountEXT",
                // )),
                create_fence: mem::transmute(load_sym(c"vkCreateFence")),
                create_semaphore: mem::transmute(load_sym(c"vkCreateSemaphore")),
                reset_fences: mem::transmute(load_sym(c"vkResetFences")),
                queue_submit_2: mem::transmute(load_sym(c"vkQueueSubmit2")),
                queue_submit: mem::transmute(load_sym(c"vkQueueSubmit")),
                device_wait_idle: mem::transmute(load_sym(c"vkDeviceWaitIdle")),
                begin_command_buffer: mem::transmute(load_sym(c"vkBeginCommandBuffer")),
                end_command_buffer: mem::transmute(load_sym(c"vkEndCommandBuffer")),
                wait_for_fences: mem::transmute(load_sym(c"vkWaitForFences")),
                wait_semaphores: mem::transmute(load_sym(c"vkWaitSemaphores")),
                cmd_blit_image_2: mem::transmute(load_sym(c"vkCmdBlitImage2")),
                cmd_copy_image_to_buffer_2: mem::transmute(load_sym(c"vkCmdCopyImageToBuffer2")),
                cmd_copy_buffer_2: mem::transmute(load_sym(c"vkCmdCopyBuffer2")),
                cmd_copy_image_2: mem::transmute(load_sym(c"vkCmdCopyImage2")),
                cmd_copy_buffer_to_image_2: mem::transmute(load_sym(c"vkCmdCopyBufferToImage2")),
                cmd_resolve_image_2: mem::transmute(load_sym(c"vkCmdResolveImage2")),
                cmd_pipeline_barrier_2: mem::transmute(load_sym(c"vkCmdPipelineBarrier2")),
                get_buffer_device_address: mem::transmute(load_sym(c"vkGetBufferDeviceAddress")),
                get_semaphore_counter_value: mem::transmute(load_sym(
                    c"vkGetSemaphoreCounterValue",
                )),
                signal_semaphore: mem::transmute(load_sym(c"vkSignalSemaphore")),
                create_shaders_ext: mem::transmute(load_sym(c"vkCreateShadersEXT")),
                cmd_bind_shaders_ext: mem::transmute(load_sym(c"vkCmdBindShadersEXT")),
                cmd_dispatch: mem::transmute(load_sym(c"vkCmdDispatch")),
                cmd_dispatch_base: mem::transmute(load_sym(c"vkCmdDispatchBase")),
                cmd_dispatch_indirect: mem::transmute(load_sym(c"vkCmdDispatchIndirect")),
                destroy_image: mem::transmute(load_sym(c"vkDestroyImage")),
                destroy_image_view: mem::transmute(load_sym(c"vkDestroyImageView")),
                destroy_buffer: mem::transmute(load_sym(c"vkDestroyBuffer")),
                destroy_device: mem::transmute(load_sym(c"vkDestroyDevice")),
                destroy_shader_ext: mem::transmute(load_sym(c"vkDestroyShaderEXT")),
                free_memory: mem::transmute(load_sym(c"vkFreeMemory")),
                free_command_buffers: mem::transmute(load_sym(c"vkFreeCommandBuffers")),
                destroy_fence: mem::transmute(load_sym(c"vkDestroyFence")),
                destroy_semaphore: mem::transmute(load_sym(c"vkDestroySemaphore")),
                destroy_command_pool: mem::transmute(load_sym(c"vkDestroyCommandPool")),
                cmd_push_data_ext: mem::transmute(load_sym(c"vkCmdPushDataEXT")),
                cmd_bind_resource_heap_ext: mem::transmute(load_sym(c"vkCmdBindResourceHeapEXT")),
                cmd_bind_sampler_heap_ext: mem::transmute(load_sym(c"vkCmdBindSamplerHeapEXT")),
                write_resource_descriptors_ext: mem::transmute(load_sym(
                    c"vkWriteResourceDescriptorsEXT",
                )),
                write_sampler_descriptors_ext: mem::transmute(load_sym(
                    c"vkWriteSamplerDescriptorsEXT",
                )),
            }
        }
    }
}

struct DeviceRtxFns {
    pub create_acceleration_structure_khr: unsafe extern "system" fn(
        device: *const Device,
        p_create_info: *const AccelerationStructureCreateInfo,
        p_allocator: *const AllocationCallbacks,
        p_acceleration_structure: *mut *const AccelerationStructure,
    ) -> Result,
    pub cmd_build_acceleration_structures_khr: unsafe extern "system" fn(
        command_buffer: *const CommandBuffer,
        info_count: u32,
        p_infos: *const AccelerationStructureBuildGeometryInfo,
        pp_build_range_infos: *const *const AccelerationStructureBuildRangeInfo,
    ),
    pub cmd_build_acceleration_structures_indirect_khr: unsafe extern "system" fn(
        command_buffer: *const CommandBuffer,
        info_count: u32,
        p_infos: *const AccelerationStructureBuildGeometryInfo,
        p_indirect_device_addresses: *const DeviceAddress,
        p_indirect_strides: *const u32,
        pp_max_primitive_counts: *const *const u32,
    ),
    pub get_acceleration_structure_build_sizes_khr: unsafe extern "system" fn(
        device: *const Device,
        build_type: AccelerationStructureBuildType,
        p_build_info: *const AccelerationStructureBuildGeometryInfo,
        p_max_primitive_counts: *const u32,
        p_size_info: *mut AccelerationStructureBuildSizesInfo,
    ),
    pub get_acceleration_structure_device_address_khr: unsafe extern "system" fn(
        device: *const Device,
        p_info: *const AccelerationStructureDeviceAddressInfo,
    )
        -> DeviceAddress,
    pub write_acceleration_structures_properties_khr: unsafe extern "system" fn(
        device: *const Device,
        acceleration_structure_count: u32,
        p_acceleration_structures: *const *const AccelerationStructure,
        query_type: QueryType,
        data_size: usize,
        p_data: *mut c_void,
        stride: usize,
    ) -> Result,
    pub cmd_trace_rays_khr: unsafe extern "system" fn(
        command_buffer: *const CommandBuffer,
        p_raygen_shader_binding_table: *const StridedDeviceAddressRegion,
        p_miss_shader_binding_table: *const StridedDeviceAddressRegion,
        p_hit_shader_binding_table: *const StridedDeviceAddressRegion,
        p_callable_shader_binding_table: *const StridedDeviceAddressRegion,
        width: u32,
        height: u32,
        depth: u32,
    ),
    pub cmd_trace_rays_indirect_khr: unsafe extern "system" fn(
        command_buffer: *const CommandBuffer,
        p_raygen_shader_binding_table: *const StridedDeviceAddressRegion,
        p_miss_shader_binding_table: *const StridedDeviceAddressRegion,
        p_hit_shader_binding_table: *const StridedDeviceAddressRegion,
        p_callable_shader_binding_table: *const StridedDeviceAddressRegion,
        indirect_device_address: DeviceAddress,
    ),
    pub cmd_trace_rays_indirect_2_khr: unsafe extern "system" fn(
        command_buffer: *const CommandBuffer,
        indirect_device_address: DeviceAddress,
    ),
    pub destroy_acceleration_structure_khr: unsafe extern "system" fn(
        device: *const Device,
        acceleration_structure: *const AccelerationStructure,
        p_allocator: *const AllocationCallbacks,
    ),
}

impl DeviceRtxFns {
    pub unsafe fn load_with(mut load_sym: impl FnMut(&CStr) -> *const u8) -> Self {
        unsafe {
            Self {
                create_acceleration_structure_khr: mem::transmute(load_sym(
                    c"vkCreateAccelerationStructureKHR",
                )),
                cmd_build_acceleration_structures_khr: mem::transmute(load_sym(
                    c"vkCmdBuildAccelerationStructuresKHR",
                )),
                cmd_build_acceleration_structures_indirect_khr: mem::transmute(load_sym(
                    c"vkCmdBuildAccelerationStructuresIndirectKHR",
                )),
                get_acceleration_structure_build_sizes_khr: mem::transmute(load_sym(
                    c"vkGetAccelerationStructureBuildSizesKHR",
                )),
                get_acceleration_structure_device_address_khr: mem::transmute(load_sym(
                    c"vkGetAccelerationStructureDeviceAddressKHR",
                )),
                write_acceleration_structures_properties_khr: mem::transmute(load_sym(
                    c"vkWriteAccelerationStructuresPropertiesKHR",
                )),
                cmd_trace_rays_khr: mem::transmute(load_sym(c"vkCmdTraceRaysKHR")),
                cmd_trace_rays_indirect_khr: mem::transmute(load_sym(c"vkCmdTraceRaysIndirectKHR")),
                cmd_trace_rays_indirect_2_khr: mem::transmute(load_sym(
                    c"vkCmdTraceRaysIndirect2KHR",
                )),
                destroy_acceleration_structure_khr: mem::transmute(load_sym(
                    c"vkDestroyAccelerationStructureKHR",
                )),
            }
        }
    }
}
