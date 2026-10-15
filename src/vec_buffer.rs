use std::{
    marker::PhantomData,
    mem,
    ops::{self, RangeBounds},
    ptr,
};

use crate::{Buffer, BufferRange, BufferUsageFlags, Context, MemoryPropertyBit, vk};

pub struct VecBuffer<T> {
    inner: Buffer<T>,
    usage: BufferUsageFlags,
    len: usize,
    sharing: bool,
}

impl<T: std::fmt::Debug> VecBuffer<T> {
    pub fn clear(&mut self) {
        self.len = 0;
    }

    pub fn push(&mut self, value: T) {
        self.extend_from_slice(&[value]);
    }

    pub fn extend_from_slice(&mut self, values: &[T]) {
        if self.len + values.len() > self.capacity() {
            let cx = Context(());
            let mut builder = cx
                .new_vec_buffer::<T>()
                .capacity((self.len + values.len()).next_power_of_two())
                .usage(self.usage);

            if self.sharing {
                builder = builder.concurrent_sharing();
            }

            let new_buffer = builder.create();

            for i in 0..self.len {
                unsafe {
                    let val = self.host_ptr().add(i).read();
                    // eprintln!("COPYING: {val:?}");
                    new_buffer.host_ptr().add(i).write(val);
                    // ptr::copy_nonoverlapping(
                    //     dbg!(self.host_ptr()),
                    //     dbg!(new_buffer.host_ptr()),
                    //     dbg!(self.len),
                    // );
                }
            }

            self.inner = new_buffer.inner;
        }

        unsafe {
            ptr::copy_nonoverlapping(values.as_ptr(), self.host_ptr().add(self.len), values.len())
        };

        self.len += values.len();

        // eprintln!("AFTER PUSH");
        // for i in 0..self.len {
        //     unsafe {
        //         let val = self.host_ptr().add(i).read();
        //         eprintln!("VALUE: {val:?}");
        //         // ptr::copy_nonoverlapping(
        //         //     dbg!(self.host_ptr()),
        //         //     dbg!(new_buffer.host_ptr()),
        //         //     dbg!(self.len),
        //         // );
        //     }
        // }
    }

    // pub fn write_descriptor<Fmt>(&mut self, image: &Image2D<Fmt>) {
    //     let cx = Context(());

    //     if self.capacity()
    //         < cx.heap_properties.image_descriptor_size as usize
    //             + cx.heap_properties.min_resource_heap_reserved_range as usize
    //     {
    //         let mut builder = cx
    //             .new_vec_buffer()
    //             .capacity(
    //                 (cx.heap_properties.image_descriptor_size
    //                     + cx.heap_properties.min_resource_heap_reserved_range)
    //                     .next_power_of_two() as usize,
    //             )
    //             .usage(self.usage);

    //         if self.sharing {
    //             builder = builder.concurrent_sharing();
    //         }

    //         let new_buffer = builder.create();

    //         *self = new_buffer;
    //     }

    //     unsafe {
    //         (cx.device_fns.write_resource_descriptors_ext)(
    //             cx.device,
    //             1,
    //             &vk::ResourceDescriptorInfo {
    //                 s_type: vk::StructureType::ResourceDescriptorInfo,
    //                 p_next: ptr::null(),
    //                 r#type: vk::DescriptorType::StorageImage,
    //                 data: vk::ResourceDescriptorData {
    //                     p_image: &vk::ImageDescriptorInfo {
    //                         s_type: vk::StructureType::ImageDescriptorInfo,
    //                         p_next: ptr::null(),
    //                         p_view: &vk::ImageViewCreateInfo {
    //                             s_type: vk::StructureType::ImageViewCreateInfo,
    //                             p_next: ptr::null(),
    //                             flags: vk::ImageViewCreateFlags(0),
    //                             image: image.inner.vk,
    //                             view_type: vk::ImageViewType::_2D,
    //                             format: vk::Format::R16G16B16A16Uint,
    //                             components: vk::ComponentMapping {
    //                                 r: vk::ComponentSwizzle::Identity,
    //                                 g: vk::ComponentSwizzle::Identity,
    //                                 b: vk::ComponentSwizzle::Identity,
    //                                 a: vk::ComponentSwizzle::Identity,
    //                             },
    //                             subresource_range: vk::ImageSubresourceRange {
    //                                 aspect_mask: vk::ImageAspectBit::Color.into(),
    //                                 base_mip_level: 0,
    //                                 level_count: 1,
    //                                 base_array_layer: 0,
    //                                 layer_count: 1,
    //                             },
    //                         },
    //                         layout: vk::ImageLayout::General,
    //                     },
    //                 },
    //             },
    //             &vk::HostAddressRange {
    //                 address: self.host_ptr().cast(),
    //                 size: cx.heap_properties.image_descriptor_size as usize,
    //             },
    //         )
    //         .must_succeed();
    //     };
    // }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn capacity(&self) -> usize {
        self.inner.len()
    }

    pub fn device_address(&self) -> u64 {
        self.inner.device_address()
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
            ops::Bound::Unbounded => self.len,
        };

        assert!(start <= end && end <= self.len);

        BufferRange {
            buffer: &self.inner,
            offset: start,
            len: end - start,
        }
    }

    fn host_ptr(&self) -> *mut T {
        self.inner.handle.block.host_ptr.cast()
    }
}

pub struct VecBufferBuilder<'cx, T> {
    marker: PhantomData<T>,
    cx: &'cx Context,
    create_info: vk::BufferCreateInfo,
}

impl<T> VecBufferBuilder<'_, T> {
    pub fn create(self) -> VecBuffer<T> {
        let buffer = self.cx.create_buffer(
            &self.create_info,
            MemoryPropertyBit::DeviceLocal
                | MemoryPropertyBit::HostVisible
                | MemoryPropertyBit::HostCoherent,
        );
        assert!(buffer.handle.block.host_ptr.cast::<T>().is_aligned());

        VecBuffer {
            inner: buffer,
            usage: self.create_info.usage,
            len: 0,
            sharing: !self.create_info.p_queue_family_indices.is_null(),
        }
    }

    pub fn usage(mut self, usage: impl Into<BufferUsageFlags>) -> Self {
        self.create_info.usage = usage.into();
        self
    }

    pub fn concurrent_sharing(mut self) -> Self {
        self.create_info.sharing_mode = vk::SharingMode::Concurrent;
        self.create_info.queue_family_index_count = self.cx.user_queue_indices.len() as u32;
        self.create_info.p_queue_family_indices = self.cx.user_queue_indices.as_ptr();
        self
    }

    pub fn capacity(mut self, capacity: usize) -> Self {
        assert!(capacity > 0);
        self.create_info.size = (capacity * mem::size_of::<T>()) as u64;
        self
    }
}

impl Context {
    pub fn new_vec_buffer<T>(&self) -> VecBufferBuilder<'_, T> {
        assert!(mem::size_of::<T>() > 0);

        VecBufferBuilder {
            marker: PhantomData,
            cx: self,
            create_info: vk::BufferCreateInfo {
                s_type: vk::StructureType::BufferCreateInfo,
                p_next: ptr::null(),
                flags: vk::BufferCreateFlags(0),
                size: (mem::size_of::<T>() * 4) as u64,
                usage: BufferUsageFlags(0),
                sharing_mode: vk::SharingMode::Exclusive,
                queue_family_index_count: 0,
                p_queue_family_indices: ptr::null(),
            },
        }
    }
}
