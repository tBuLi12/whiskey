use guillotiere::euclid;

use vk_binds_test::{
    AccessBit, App, AttachmentStoreOp, BlendFactor, BlendOp, ColorBlendEquation, ColorComponentBit,
    CommandPool, Copies, DescriptorHeap, DescriptorType, Filter, ImageAspectBit, ImageLayout,
    ImageUsageBit, MemoryBarrier, Offset2D, PipelineStageBit, PolygonMode, PrimitiveTopology,
    Rect2D, Render, RenderingAttachmentInfo, RenderingInfo, SampleCountBit, SamplerAddressMode,
    SamplerMipmapMode, Shader, ShaderCreateBit, ShaderCreateInfo, ShaderStageBit,
    StagingMemoryPool, SubmitInfo, TimelineSemaphore, Viewport, Window, formats::ClearColorF32,
};

use crate::text::{Paragraph, TextStuff};

mod text;

fn main() {
    env_logger::init();

    App::run(|app| {
        app.new_window()
            .image_usage(ImageUsageBit::ColorAttachment)
            .open(|app, _| {
                #[repr(C)]
                struct ShaderData<Bytes> {
                    _align: [u32; 0],
                    bytes: Bytes,
                }

                let shader_data = ShaderData {
                    _align: [],
                    bytes: *include_bytes!("../../shader.spv"),
                };

                let mut shaders = app
                    .create_shaders(&[
                        ShaderCreateInfo::new_spirv(
                            ShaderStageBit::Vertex,
                            &shader_data.bytes,
                            c"textVert",
                        )
                        .flags(ShaderCreateBit::LinkStage)
                        .next_stage(ShaderStageBit::Fragment),
                        ShaderCreateInfo::new_spirv(
                            ShaderStageBit::Fragment,
                            &shader_data.bytes,
                            c"textFrag",
                        )
                        .flags(ShaderCreateBit::LinkStage),
                    ])
                    .into_iter();

                let text_vert_shader = shaders.next().unwrap();
                let text_frag_shader = shaders.next().unwrap();

                let cx = &**app;
                let text = TextStuff::new(*cx);

                let heap = cx.create_descriptor_heap();
                let sampler_heap = cx.create_sampler_heap();

                Hello {
                    text_vert_shader,
                    text_frag_shader,
                    heap,
                    sampler_heap,
                    text,
                    paragraph: None,
                    atlas_descriptor: None,
                    sampler_descriptor: None,
                    n: 0,
                    frame_done: cx.create_timeline_semaphore(0),
                }
            })
            .unwrap();
    });
}

struct Hello {
    text_vert_shader: Shader,
    text_frag_shader: Shader,
    heap: DescriptorHeap,
    sampler_heap: DescriptorHeap,
    text: TextStuff,
    paragraph: Option<Paragraph>,
    atlas_descriptor: Option<u32>,
    sampler_descriptor: Option<u32>,
    n: usize,
    frame_done: TimelineSemaphore,
}

struct Frame {
    pool: CommandPool,
    staging_mem_pool: StagingMemoryPool,
    copies: Copies,
}

impl Render for Hello {
    type Frame = Frame;

    fn new_frame(app: &mut App) -> Self::Frame {
        let mut pool = app.graphics_queue().create_command_pool();
        pool.allocate_command_buffers(1);

        Frame {
            pool,
            staging_mem_pool: app.create_staging_memory_pool(),
            copies: app.create_copies(),
        }
    }

    fn render(&mut self, app: &mut App, window: &mut Window, frame: &mut Self::Frame) {
        self.n += 1;
        self.frame_done.wait(self.n.saturating_sub(2) as u64);
        frame.pool.reset();
        frame.staging_mem_pool.reset();

        if self.paragraph.is_none() {
            self.paragraph = Some(self.text.layout_text(
                include_str!("../../lorem.txt"),
                Some(window.physical_size().width as f32),
                &mut frame.staging_mem_pool,
                &mut self.heap,
            ));
        }

        let buffer = frame.pool.get_buffer();
        buffer.begin();

        if self.atlas_descriptor.is_none() {
            self.atlas_descriptor = Some(self.heap.view_descriptor(
                DescriptorType::StorageImage,
                ImageLayout::General,
                self.text.atlas.image(),
                &mut frame.staging_mem_pool,
            ));
        }

        if self.sampler_descriptor.is_none() {
            self.sampler_descriptor = Some(self.sampler_heap.sampler_descriptor(
                Filter::Linear,
                Filter::Linear,
                SamplerMipmapMode::Linear,
                SamplerAddressMode::ClampToBorder,
                SamplerAddressMode::ClampToBorder,
                SamplerAddressMode::ClampToBorder,
                0.0,
                true,
                16.0,
                0.0,
                1000.0,
                &mut frame.staging_mem_pool,
            ));
        }

        self.text.record_uploads(&mut frame.copies);
        self.text.atlas.record_uploads(&mut frame.copies);
        self.text.curves.record_uploads(&mut frame.copies);
        self.heap.record_uploads(&mut frame.copies);
        self.sampler_heap.record_uploads(&mut frame.copies);

        frame.copies.record().barrier(
            window
                .image()
                .barrier(ImageAspectBit::Color)
                .old_layout(ImageLayout::Undefined)
                .new_layout(ImageLayout::General)
                .dst_access(PipelineStageBit::AllCommands, AccessBit::MemoryWrite)
                .src_queue(!2)
                .dst_queue(app.graphics_queue().family_idx()),
        );

        frame.copies.flush(&buffer);

        buffer.barriers(
            None,
            &[MemoryBarrier::new()
                .src_access(PipelineStageBit::AllTransfer, AccessBit::TransferWrite)
                .dst_access(PipelineStageBit::AllCommands, AccessBit::MemoryRead)],
            &[],
            &[],
        );

        buffer.begin_rendering(
            RenderingInfo::new(Rect2D {
                offset: Offset2D { x: 0, y: 0 },
                extent: window.physical_size(),
            })
            .layer_count(1)
            .color_attachments(&[RenderingAttachmentInfo::new_cleared(
                window.image_view(),
                ImageLayout::General,
                ClearColorF32 {
                    r: 0.0,
                    g: 0.0,
                    b: 0.0,
                    a: 1.0,
                },
            )
            .store_op(AttachmentStoreOp::Store)]),
        );

        buffer.set_rasterizer_discard_enable(false);
        buffer.set_cull_mode(None);
        buffer.set_depth_test_enable(false);
        buffer.set_stencil_test_enable(false);
        buffer.set_depth_bias_enable(false);
        buffer.set_polygon_mode(PolygonMode::Fill);
        buffer.set_rasterization_samples(SampleCountBit::_1);
        buffer.set_sample_mask(SampleCountBit::_1, &[1]);
        buffer.set_alpha_to_coverage_enable(false);
        buffer.set_color_blend_enable(0, &[1]);
        buffer.set_color_blend_equation(
            0,
            &[ColorBlendEquation {
                src_color_blend_factor: BlendFactor::SrcAlpha,
                dst_color_blend_factor: BlendFactor::OneMinusSrcAlpha,
                color_blend_op: BlendOp::Add,
                src_alpha_blend_factor: BlendFactor::One,
                dst_alpha_blend_factor: BlendFactor::One,
                alpha_blend_op: BlendOp::Add,
            }],
        );
        buffer.set_color_write_mask(
            0,
            &[ColorComponentBit::B
                | ColorComponentBit::G
                | ColorComponentBit::R
                | ColorComponentBit::A],
        );
        buffer.set_viewport_with_count(&[Viewport {
            x: 0.0,
            y: 0.0,
            width: window.physical_size().width as f32,
            height: window.physical_size().height as f32,
            min_depth: 0.0,
            max_depth: 1.0,
        }]);
        buffer.set_scissor_with_count(&[Rect2D {
            offset: Offset2D { x: 0, y: 0 },
            extent: window.physical_size(),
        }]);
        buffer.set_primitive_topology(PrimitiveTopology::TriangleList);
        buffer.set_primitive_restart_enable(false);
        buffer.set_vertex_input();

        buffer.bind_vertex_fragment_shaders(self.text_vert_shader, self.text_frag_shader);

        #[repr(C)]
        struct TextPushData {
            transform: euclid::default::Transform3D<f32>,
            glyphs: u64,
            curves: u64,
            atlas: u32,
            sampler: u32,
        }

        buffer.push_data(&TextPushData {
            transform: euclid::default::Transform3D::ortho(
                0.0,
                window.physical_size().width as f32,
                0.0,
                window.physical_size().height as f32,
                1.0,
                -1.0,
            ),
            glyphs: self.paragraph.as_ref().unwrap().glyphs.device_address(),
            curves: self.text.curves.device_address(),
            atlas: self.atlas_descriptor.unwrap(),
            sampler: self.sampler_descriptor.unwrap(),
        });

        buffer.bind_resource_heap(&self.heap);
        buffer.bind_sampler_heap(&self.sampler_heap);

        buffer.draw(
            0..6,
            0..self.paragraph.as_ref().unwrap().glyphs.len() as u32,
        );

        buffer.end_rendering();

        buffer.barriers(
            None,
            &[],
            &[],
            &[window
                .image()
                .barrier(ImageAspectBit::Color)
                .old_layout(ImageLayout::General)
                .new_layout(ImageLayout::General)
                .src_access(PipelineStageBit::AllCommands, AccessBit::MemoryWrite)
                .src_queue(app.graphics_queue().family_idx())
                .dst_queue(!2)],
        );

        buffer.end();

        app.graphics_queue()
            .submit(&[SubmitInfo::new(&[buffer.submit_info()])
                .signal_semaphores(&[
                    window
                        .signal_semaphore()
                        .submit_info(PipelineStageBit::AllCommands),
                    self.frame_done
                        .submit_info(PipelineStageBit::AllCommands, self.n as u64),
                ])
                .wait_semaphores(&[window
                    .wait_semaphore()
                    .submit_info(PipelineStageBit::AllCommands)])]);
    }
}
