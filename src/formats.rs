use crate::vk;

pub trait Format {
    type ClearValue: Into<vk::ClearValue>;

    fn vk() -> vk::Format;
    fn all_aspects() -> vk::ImageAspectFlags;
}

#[derive(Clone, Copy, Debug)]
pub struct ClearColorF32 {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Into<vk::ClearValue> for ClearColorF32 {
    fn into(self) -> vk::ClearValue {
        vk::ClearValue {
            color: vk::ClearColorValue {
                float_32: [self.r, self.g, self.b, self.a],
            },
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct ClearColorU32 {
    pub r: u32,
    pub g: u32,
    pub b: u32,
    pub a: u32,
}

impl Into<vk::ClearValue> for ClearColorU32 {
    fn into(self) -> vk::ClearValue {
        vk::ClearValue {
            color: vk::ClearColorValue {
                uint_32: [self.r, self.g, self.b, self.a],
            },
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[repr(C)]
pub struct B8G8R8A8Unorm {
    pub b: u8,
    pub g: u8,
    pub r: u8,
    pub a: u8,
}

impl Format for B8G8R8A8Unorm {
    type ClearValue = ClearColorF32;

    fn all_aspects() -> vk::ImageAspectFlags {
        vk::ImageAspectBit::Color.into()
    }

    fn vk() -> vk::Format {
        vk::Format::B8G8R8A8Unorm
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[repr(C)]
pub struct R8G8B8A8Uint {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Format for R8G8B8A8Uint {
    type ClearValue = ClearColorU32;

    fn all_aspects() -> vk::ImageAspectFlags {
        vk::ImageAspectBit::Color.into()
    }

    fn vk() -> vk::Format {
        vk::Format::R8G8B8A8Uint
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[repr(C)]
pub struct R16G16B16A16Uint {
    pub r: u16,
    pub g: u16,
    pub b: u16,
    pub a: u16,
}

impl Format for R16G16B16A16Uint {
    type ClearValue = ClearColorU32;

    fn all_aspects() -> vk::ImageAspectFlags {
        vk::ImageAspectBit::Color.into()
    }

    fn vk() -> vk::Format {
        vk::Format::R16G16B16A16Uint
    }
}
