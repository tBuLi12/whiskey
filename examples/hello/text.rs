use std::{
    collections::HashMap,
    mem,
    process::{self},
    sync::Arc,
};

use parley::{
    FontContext, FontFamily, FontFamilyName, LayoutContext, StyleProperty, fontique::Blob,
};
use skrifa::{
    FontRef, GlyphId, MetadataProvider, OutlineGlyphCollection,
    outline::OutlinePen,
    prelude::{LocationRef, Size},
};
use vk_binds_test::{
    AccessBit, Arena, Atlas, Buffer, BufferImageCopy, BufferUsageBit, CommandBuffer, Context,
    Copies, DescriptorHeap, Extent2D, Extent3D, Filter, Image2DArray, ImageAspectBit, ImageBlit,
    ImageData, ImageLayout, ImageMemoryBarrier, ImageUsageBit, Offset3D, PipelineStageBit,
    StagingMemoryPool,
    formats::{B8G8R8A8Unorm, R16G16B16A16Uint},
    vec_buffer::VecBuffer,
};

pub struct TextStuff {
    cx: Context,
    font_cx: FontContext,
    layout_cx: LayoutContext<()>,
    outlines: HashMap<u32, Option<Glyph>>,
    pub atlas: Atlas<R16G16B16A16Uint>,
    // pub raw_atlas: Atlas<R16G16B16A16Uint>,
    pub rasterized_glyphs: HashMap<u16, RasterizedGlyphArray>,
    pub curves: Arena<[f32; 6]>,
}

struct RasterizedGlyphArray {
    next: u32,
    array: Image2DArray<B8G8R8A8Unorm>,
    uploads: Vec<(Buffer<B8G8R8A8Unorm>, usize, u32)>,
    image_idx: u32,
}

impl RasterizedGlyphArray {
    pub fn upload(
        &mut self,
        data: ImageData<'_, B8G8R8A8Unorm>,
        staging_pool: &mut StagingMemoryPool,
    ) -> u32 {
        let idx = self.next;
        if idx == 32 {
            panic!();
        }
        self.next += 1;

        let staging_range = staging_pool.insert(data.as_slice());

        self.uploads
            .push((staging_range.buffer().clone(), staging_range.offset(), idx));
        idx
    }
}

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct Glyph {
    base_idx: u32,
    n: u32,
    base_offset: [u32; 2],
    image_idx: u32,
    layer: u32,
    min_x: f32,
    min_y: f32,
    cell_width: f32,
    cell_height: f32,
}

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct PositionedGlyph {
    position: [f32; 2],
    base_idx: u32,
    n: u32,
    base_offset: [u32; 2],
    image_idx: u32,
    layer: u32,
    min_x: f32,
    min_y: f32,
    max_x: f32,
    max_y: f32,
    cell_width: f32,
    cell_height: f32,
}

impl TextStuff {
    pub fn record_uploads(&mut self, copies: &mut Copies) {
        for (&depth, array) in &mut self.rasterized_glyphs {
            for (buffer, offset, layer) in array.uploads.drain(..) {
                let mut copies = copies.record();
                let image = array.array.clone();
                copies
                    .barrier(
                        array
                            .array
                            .barrier(ImageAspectBit::Color)
                            .layer_range(layer..layer + 1)
                            .mip_range(0..1)
                            .new_layout(ImageLayout::General)
                            .dst_access(PipelineStageBit::AllTransfer, AccessBit::TransferWrite),
                    )
                    .next_stage(move |cmd_buffer| {
                        cmd_buffer.copy_buffer_to_image(
                            &buffer,
                            &image,
                            ImageLayout::General,
                            &[BufferImageCopy::new::<B8G8R8A8Unorm>(
                                offset,
                                Offset3D { x: 0, y: 0, z: 0 },
                                layer..(layer + 1),
                                0,
                                Extent3D {
                                    width: 4 << depth,
                                    height: 4 << depth,
                                    depth: 1,
                                },
                            )],
                        );
                    });

                for i in 1..depth + 3 {
                    let image = array.array.clone();
                    copies
                        .barrier(
                            array
                                .array
                                .barrier(ImageAspectBit::Color)
                                .layer_range(layer..layer + 1)
                                .mip_range(i as u32..i as u32 + 1)
                                .new_layout(ImageLayout::General)
                                .dst_access(
                                    PipelineStageBit::AllTransfer,
                                    AccessBit::TransferWrite,
                                ),
                        )
                        .next_stage(move |cmd_buffer| {
                            cmd_buffer.blit_image(
                                &image,
                                ImageLayout::General,
                                &image,
                                ImageLayout::General,
                                Filter::Linear,
                                &[ImageBlit::new::<B8G8R8A8Unorm>(
                                    [
                                        Offset3D { x: 0, y: 0, z: 0 },
                                        Offset3D {
                                            x: (4 << depth) >> (i - 1),
                                            y: (4 << depth) >> (i - 1),
                                            z: 1,
                                        },
                                    ],
                                    layer..layer + 1,
                                    i as u32 - 1,
                                    [
                                        Offset3D { x: 0, y: 0, z: 0 },
                                        Offset3D {
                                            x: (4 << depth) >> i,
                                            y: (4 << depth) >> i,
                                            z: 1,
                                        },
                                    ],
                                    layer..layer + 1,
                                    i as u32,
                                )],
                            );
                        });
                }
            }
        }
    }

    pub fn new(cx: Context) -> Self {
        let mut font_cx = FontContext::new();

        font_cx.collection.register_fonts(
            Blob::new(Arc::new(include_bytes!("../../Ballet_28pt-Regular.ttf"))),
            None,
        );

        // for name in font_cx.collection.family_names() {
        //     eprintln!("{name}");
        // }

        let atlas = cx.new_atlas().usage(ImageUsageBit::Storage).create();
        // let raw_atlas = cx
        //     .new_atlas()
        //     .usage(ImageUsageBit::Storage | ImageUsageBit::Sampled)
        //     .create();

        Self {
            atlas,
            rasterized_glyphs: HashMap::new(),
            curves: cx
                .new_arena()
                .usage(BufferUsageBit::StorageBuffer)
                // .capacity(1024 * 1024)
                .create(),
            cx,
            font_cx,
            layout_cx: LayoutContext::<()>::new(),
            outlines: HashMap::new(),
        }
    }

    pub fn layout_text(
        &mut self,
        text: &str,
        max_width: Option<f32>,
        staging_mem_pool: &mut StagingMemoryPool,
        descriptor_heap: &mut DescriptorHeap,
    ) -> Paragraph {
        let mut builder = self
            .layout_cx
            .ranged_builder(&mut self.font_cx, text, 1.0, false);

        builder.push_default(StyleProperty::FontFamily(FontFamily::Single(
            // FontFamilyName::Named(std::borrow::Cow::Borrowed("Ballet 28pt")),
            FontFamilyName::Generic(parley::GenericFamily::SystemUi),
        )));
        builder.push_default(StyleProperty::FontSize(30.0));

        let mut layout = builder.build(text);
        // layout.break_all_lines(None);
        layout.break_all_lines(max_width);
        layout.align(
            parley::Alignment::Left,
            parley::AlignmentOptions {
                align_when_overflowing: true,
            },
        );

        let mut paragraph = Paragraph {
            glyphs: self
                .cx
                .new_vec_buffer()
                .usage(BufferUsageBit::ShaderDeviceAddress | BufferUsageBit::StorageBuffer)
                .create(),
        };

        for line in layout.lines() {
            for item in line.items() {
                match item {
                    parley::PositionedLayoutItem::GlyphRun(glyph_run) => {
                        eprintln!("GLYPH RUN");
                        let run = glyph_run.run();
                        let font = run.font();
                        let font_ref = FontRef::from_index(font.data.as_ref(), font.index).unwrap();
                        let scale = run.font_size()
                            / font_ref
                                .metrics(Size::unscaled(), LocationRef::new(&[]))
                                .units_per_em as f32;
                        let glyph_collection = OutlineGlyphCollection::new(&font_ref);

                        for glyph in glyph_run.positioned_glyphs() {
                            let glyph_data = self.outlines.entry(glyph.id).or_insert_with(|| {
                                let outline = glyph_collection.get(GlyphId::new(glyph.id)).unwrap();
                                let mut pen = Pen {
                                    points: vec![],
                                    start: None,
                                    anchor: None,
                                };
                                let _ = outline.draw(Size::unscaled(), &mut pen).unwrap();

                                let mut max_x = f32::MIN;
                                let mut min_x = f32::MAX;
                                let mut max_y = f32::MIN;
                                let mut min_y = f32::MAX;

                                for i in 0..pen.points.len() {
                                    let Bezier { x, y } = pen.points[i];

                                    max_y = max_y.max(y.0).max(y.1).max(y.2);
                                    min_y = min_y.min(y.0).min(y.1).min(y.2);
                                    max_x = max_x.max(x.0).max(x.1).max(x.2);
                                    min_x = min_x.min(x.0).min(x.1).min(x.2);
                                }

                                #[derive(Debug)]
                                enum Command {
                                    Assign(Vec<Bezier>),
                                    Next,
                                    Subdivide,
                                }

                                if pen.points.len() > 0 {
                                    fn compute_split_depth(
                                        curves: &mut [Bezier],
                                        commands: &mut Vec<Command>,
                                        max_x: f32,
                                        min_x: f32,
                                        max_y: f32,
                                        min_y: f32,
                                        depth: u32,
                                    ) -> u32 {
                                        // if depth > 1 {
                                        if curves.len() <= 4 {
                                            commands.push(Command::Assign(curves.to_vec()));
                                            commands.push(Command::Next);
                                            return 0;
                                        }

                                        commands.push(Command::Subdivide);

                                        if depth > 20 {
                                            dbg!(curves);
                                            process::exit(0);
                                        }

                                        let mid_x = (max_x + min_x) / 2.0;
                                        let mid_y = (max_y + min_y) / 2.0;

                                        let d = {
                                            let (in_half, _) =
                                                partition::partition(curves, |curve| {
                                                    curve.in_bounds(max_x, min_x, mid_y, min_y)
                                                });

                                            let (in_quarter, _) =
                                                partition::partition(in_half, |curve| {
                                                    curve.in_bounds(mid_x, min_x, mid_y, min_y)
                                                });

                                            let d = compute_split_depth(
                                                in_quarter,
                                                commands,
                                                mid_x,
                                                min_x,
                                                mid_y,
                                                min_y,
                                                depth + 1,
                                            );

                                            let (in_quarter, _) =
                                                partition::partition(in_half, |curve| {
                                                    curve.in_bounds(max_x, mid_x, mid_y, min_y)
                                                });

                                            compute_split_depth(
                                                in_quarter,
                                                commands,
                                                max_x,
                                                mid_x,
                                                mid_y,
                                                min_y,
                                                depth + 1,
                                            )
                                            .max(d)
                                        };

                                        let d = {
                                            let (in_half, _) =
                                                partition::partition(curves, |curve| {
                                                    curve.in_bounds(max_x, min_x, max_y, mid_y)
                                                });

                                            let (in_quarter, _) =
                                                partition::partition(in_half, |curve| {
                                                    curve.in_bounds(mid_x, min_x, max_y, mid_y)
                                                });

                                            let d = compute_split_depth(
                                                in_quarter,
                                                commands,
                                                mid_x,
                                                min_x,
                                                max_y,
                                                mid_y,
                                                depth + 1,
                                            );

                                            let (in_quarter, _) =
                                                partition::partition(in_half, |curve| {
                                                    curve.in_bounds(max_x, mid_x, max_y, mid_y)
                                                });

                                            compute_split_depth(
                                                in_quarter,
                                                commands,
                                                max_x,
                                                mid_x,
                                                max_y,
                                                mid_y,
                                                depth + 1,
                                            )
                                            .max(d)
                                        }
                                        .max(d);

                                        d + 1
                                    }

                                    let mut commands = vec![];

                                    // eprintln!("for {c}");
                                    let depth = compute_split_depth(
                                        &mut pen.points,
                                        &mut commands,
                                        max_x,
                                        min_x,
                                        max_y,
                                        min_y,
                                        0,
                                    );

                                    // dbg!(max_y);
                                    // dbg!(max_x);
                                    // dbg!(min_y);
                                    // dbg!(min_x);
                                    // dbg!(min_triple_dist);

                                    // if depth > 4 {
                                    // for (x, y) in &pen.points {
                                    //     eprintln!("({x},{y})");
                                    // }
                                    // let depth = 0;
                                    let tiles = 4usize.pow(depth);

                                    let mut curves = vec![];
                                    let mut image = vec![
                                        R16G16B16A16Uint {
                                            r: 0,
                                            g: 0,
                                            b: 0,
                                            a: 0
                                        };
                                        tiles
                                    ];
                                    let mut raw_image = vec![
                                        B8G8R8A8Unorm {
                                            r: 0,
                                            g: 0,
                                            b: 0,
                                            a: 0
                                        };
                                        tiles * 16
                                    ];

                                    if true {
                                        let size = 1 << depth;

                                        let mut x = 0;
                                        let mut y = 0;
                                        let mut scale = size;
                                        let mut n = 0;
                                        for command in commands {
                                            match command {
                                                Command::Assign(bezier) => {
                                                    let start = curves.len();
                                                    curves.extend_from_slice(&bezier);
                                                    let end = curves.len();
                                                    for j in y..y + scale {
                                                        for i in x..x + scale {
                                                            // image[j * size + i][n] = bezier;
                                                            if start > u16::MAX as usize
                                                                || end > u16::MAX as usize
                                                            {
                                                                panic!()
                                                            }
                                                            image[j * size + i] =
                                                                R16G16B16A16Uint {
                                                                    r: 0,
                                                                    g: start as u16,
                                                                    b: end as u16,
                                                                    a: 0,
                                                                };
                                                        }
                                                    }
                                                    n += 1;
                                                }
                                                Command::Next => {
                                                    // eprintln!("Next");
                                                    n = 0;
                                                    loop {
                                                        if x & scale == 0 {
                                                            x += scale;
                                                            break;
                                                        } else if y & scale == 0 {
                                                            x -= scale;
                                                            y += scale;
                                                            break;
                                                        } else {
                                                            x -= scale;
                                                            y -= scale;
                                                            scale *= 2;
                                                        }
                                                    }
                                                }
                                                Command::Subdivide => {
                                                    // eprintln!("Subdivide");
                                                    scale /= 2;
                                                }
                                            }
                                        }

                                        let cell_width = (max_x - min_x) / (1 << depth) as f32;
                                        let cell_height = (max_y - min_y) / (1 << depth) as f32;
                                        for j in 0..size {
                                            for i in 0..size {
                                                let cell_start = min_x + i as f32 * cell_width;
                                                let cell_end = cell_start + cell_width;
                                                let cell_y_start = min_y + j as f32 * cell_height;

                                                let x = (cell_start + cell_end) / 2.0;
                                                let y = cell_y_start;

                                                let cell = &mut image[j * size + i];
                                                let beziers =
                                                    &curves[cell.g as usize..cell.b as usize];
                                                let mut bias = pen
                                                    .points
                                                    .iter()
                                                    .filter(|&curve| {
                                                        !beziers.iter().any(|c| c == curve)
                                                    })
                                                    .map(|curve| curve.intersect_x(x, y))
                                                    .sum::<i32>();

                                                for curve in beziers {
                                                    if curve.x.0 >= cell_start
                                                        && curve.x.0 < cell_end
                                                        && curve.y.0 < cell_y_start
                                                    {
                                                        if x < curve.x.0 {
                                                            bias += 1;
                                                        }
                                                    }
                                                    if curve.x.2 >= cell_start
                                                        && curve.x.2 < cell_end
                                                        && curve.y.2 < cell_y_start
                                                    {
                                                        if x < curve.x.2 {
                                                            bias -= 1;
                                                        }
                                                    }
                                                }

                                                cell.r = (128 + bias) as u16;
                                            }
                                        }
                                        let cell_width = (max_x - min_x) / (4 << depth) as f32;
                                        let cell_height = (max_y - min_y) / (4 << depth) as f32;
                                        for j in 0..size * 4 {
                                            for i in 0..size * 4 {
                                                let cell_start = min_x + i as f32 * cell_width;
                                                let cell_y_start = min_y + j as f32 * cell_height;

                                                let n = 8;
                                                let x = cell_start;
                                                let y = cell_y_start;

                                                let cell = &mut raw_image[j * size * 4 + i];
                                                let mut color = 0;
                                                for j in 0..n {
                                                    for i in 0..n {
                                                        let wn = pen
                                                            .points
                                                            .iter()
                                                            .map(|curve| {
                                                                curve.intersect_x(
                                                                    x + cell_width
                                                                        * (i as f32 + 0.5)
                                                                        / (n as f32),
                                                                    y + cell_height
                                                                        * (j as f32 + 0.5)
                                                                        / (n as f32),
                                                                )
                                                            })
                                                            .sum::<i32>();
                                                        if wn != 0 {
                                                            color += 1;
                                                        }
                                                    }
                                                }
                                                cell.r = (((color as f32) / (n * n) as f32)
                                                    * u8::MAX as f32)
                                                    as u8;
                                                cell.g = (((color as f32) / (n * n) as f32)
                                                    * u8::MAX as f32)
                                                    as u8;
                                                cell.b = (((color as f32) / (n * n) as f32)
                                                    * u8::MAX as f32)
                                                    as u8;
                                                cell.a = u8::MAX;
                                                // eprintln!("SETTING");
                                            }
                                        }
                                    } else {
                                        curves.extend_from_slice(&pen.points);
                                        for tile in &mut image {
                                            tile.b = curves.len() as u16;
                                            tile.r = 128;
                                        }
                                    }

                                    println!("{}x{}", max_x - min_x, max_y - min_y);

                                    // if depth > 4 {
                                    // }
                                    // }

                                    // for curve in &pen.points {
                                    //     eprintln!("({},{})", curve.x.0, curve.y.0);
                                    //     eprintln!("({},{})", curve.x.2, curve.y.2);
                                    // }
                                    // for curve in &image[0] {
                                    //     eprintln!("({},{})", curve.x.0, curve.y.0);
                                    //     eprintln!("({},{})", curve.x.2, curve.y.2);
                                    // }
                                    // let curves = mem::take(&mut pen.points);
                                    let mut render_scale = 1.0;
                                    let cell_width = (max_x - min_x) / (1 << depth) as f32;
                                    let cell_height = (max_y - min_y) / (1 << depth) as f32;
                                    // dbg!((cell_height, cell_width));

                                    let base_idx = self.curves.alloc(
                                        &curves
                                            .into_iter()
                                            .map(|c| {
                                                [
                                                    c.x.0 - min_x,
                                                    c.y.0 - min_y,
                                                    c.x.1 - min_x,
                                                    c.y.1 - min_y,
                                                    c.x.2 - min_x,
                                                    c.y.2 - min_y,
                                                ]
                                            })
                                            .collect::<Vec<_>>(),
                                        staging_mem_pool,
                                    );

                                    let (base_offset_x, base_offset_y) = self.atlas.upload(
                                        ImageData::from_slice(
                                            &image,
                                            Extent3D {
                                                width: 1 << depth,
                                                height: 1 << depth,
                                                depth: 1,
                                            },
                                        ),
                                        staging_mem_pool,
                                    );

                                    let base_offset = [base_offset_x, base_offset_y];

                                    let array = self
                                        .rasterized_glyphs
                                        .entry(depth as u16)
                                        .or_insert_with(|| {
                                            let image = self
                                                .cx
                                                .new_image_array(
                                                    Extent2D {
                                                        width: 4 << depth,
                                                        height: 4 << depth,
                                                    },
                                                    32,
                                                )
                                                .mip_levels(3 + depth)
                                                .usage(
                                                    ImageUsageBit::Sampled
                                                        | ImageUsageBit::TransferDst
                                                        | ImageUsageBit::TransferSrc,
                                                )
                                                .create();

                                            RasterizedGlyphArray {
                                                image_idx: descriptor_heap.view_descriptor(
                                                    vk_binds_test::DescriptorType::SampledImage,
                                                    ImageLayout::General,
                                                    &image,
                                                    staging_mem_pool,
                                                ),
                                                next: 0,
                                                array: image,
                                                uploads: vec![],
                                            }
                                        });

                                    // array

                                    let layer = array.upload(
                                        ImageData::from_slice(
                                            &raw_image,
                                            Extent3D {
                                                width: 4 << depth,
                                                height: 4 << depth,
                                                depth: 1,
                                            },
                                        ),
                                        staging_mem_pool,
                                    );

                                    // let raw_base_offset = [raw_base_offset_x, raw_base_offset_y];

                                    let cell_width = (max_x - min_x) / (1 << depth) as f32;
                                    let cell_height = (max_y - min_y) / (1 << depth) as f32;

                                    Some(Glyph {
                                        base_idx: base_idx as u32,
                                        base_offset,
                                        image_idx: array.image_idx,
                                        layer,
                                        min_x,
                                        min_y,
                                        cell_width,
                                        cell_height,
                                        n: (1 << depth),
                                    })
                                } else {
                                    None
                                }
                            });

                            if let Some(glyph_data) = glyph_data {
                                paragraph.glyphs.push(PositionedGlyph {
                                    position: [glyph.x, glyph.y],
                                    base_idx: glyph_data.base_idx,
                                    n: glyph_data.n,
                                    base_offset: glyph_data.base_offset,
                                    image_idx: glyph_data.image_idx,
                                    layer: glyph_data.layer,
                                    min_x: glyph_data.min_x * scale,
                                    min_y: glyph_data.min_y * scale,
                                    max_x: (glyph_data.min_x
                                        + glyph_data.cell_width * glyph_data.n as f32)
                                        * scale,
                                    max_y: (glyph_data.min_y
                                        + glyph_data.cell_height * glyph_data.n as f32)
                                        * scale,
                                    cell_width: glyph_data.cell_width,
                                    cell_height: glyph_data.cell_height,
                                });
                            }

                            // let start_x_f = glyph.x + *min_x * scale;
                            // let start_x = start_x_f.floor() as usize;
                            // let end_x_f = glyph.x + *max_x * scale;
                            // let end_x = end_x_f.ceil() as usize;
                            // let start_y_f = glyph.y - *max_y * scale;
                            // let start_y = start_y_f.floor() as usize;
                            // let end_y_f = glyph.y - *min_y * scale;
                            // let end_y = end_y_f.ceil() as usize;
                            // for y in start_y..end_y {
                            //     for x in start_x..end_x {
                            //         let pixel = &mut buffer[y * width + x];
                            //         let mut c = 0;
                            //         for (xo, yo) in [
                            //             (0.16666, 0.16666),
                            //             (0.50, 0.16666),
                            //             (0.833333, 0.16666),
                            //             (0.16666, 0.50),
                            //             (0.50, 0.50),
                            //             (0.833333, 0.50),
                            //             (0.16666, 0.833333),
                            //             (0.50, 0.833333),
                            //             (0.833333, 0.833333),
                            //         ] {
                            //             let paint = 0
                            //                 != outline
                            //                     .iter()
                            //                     .map(|curve| {
                            //                         curve.intersect_x(
                            //                             ((x as f32 + xo) - glyph.x) / scale,
                            //                             (glyph.y - (y as f32 + yo)) / scale,
                            //                         )
                            //                     })
                            //                     .sum::<i32>();
                            //             if paint {
                            //                 c += 0x001c1c1c;
                            //             }
                            //         }
                            //         *pixel = c;
                            //     }
                            // }
                        }
                    }
                    parley::PositionedLayoutItem::InlineBox(positioned_inline_box) => {
                        unreachable!()
                    }
                }
            }
        }

        paragraph
    }
}

pub struct Paragraph {
    pub glyphs: VecBuffer<PositionedGlyph>,
}

struct Pen {
    points: Vec<Bezier>,
    start: Option<(f32, f32)>,
    anchor: Option<(f32, f32)>,
}

impl Pen {
    pub fn push(&mut self, cx: f32, cy: f32, x: f32, y: f32) {
        let anchor = self.anchor.unwrap();
        self.points.push(Bezier {
            x: (anchor.0, cx, x),
            y: (anchor.1, cy, y),
        });

        self.anchor = Some((x, y));

        // self.points.push(Point {
        //     x,
        //     y,
        //     i: self.points.len() as u32,
        //     next: self.points.len() as u32 + 1,
        //     prev: u32::MAX,
        // });
    }

    pub fn push_line(&mut self, x: f32, y: f32) {
        let anchor = self.anchor.unwrap();
        // self.points.push(Bezier {
        //     x: (anchor.0, anchor.0, x),
        //     y: (anchor.1, anchor.1, y),
        // });
        // self.points.push(Bezier {
        //     x: (anchor.0, (anchor.0 + x) / 2.0, x),
        //     y: (anchor.1, (anchor.1 + y) / 2.0, y),
        // });
        self.points.push(Bezier {
            x: (anchor.0, x, x),
            y: (anchor.1, y, y),
        });

        self.anchor = Some((x, y));
    }

    pub fn close(&mut self) {
        let start = self.start.take().unwrap();
        let anchor = self.anchor.take().unwrap();

        if start != anchor {
            self.points.push(Bezier {
                x: (anchor.0, (anchor.0 + start.0) / 2.0, start.0),
                y: (anchor.1, (anchor.1 + start.1) / 2.0, start.1),
            });
        }
    }
}

impl OutlinePen for Pen {
    fn move_to(&mut self, x: f32, y: f32) {
        self.start = Some((x, y));
        self.anchor = Some((x, y));
        // eprintln!("({},{})", x, y);
    }

    fn line_to(&mut self, x: f32, y: f32) {
        // eprintln!("({},{})", x, y);
        self.push_line(x, y);
    }

    fn quad_to(&mut self, cx0: f32, cy0: f32, x: f32, y: f32) {
        // eprintln!("({},{})", x, y);
        self.push(cx0, cy0, x, y);
    }

    fn curve_to(&mut self, cx0: f32, cy0: f32, cx1: f32, cy1: f32, x: f32, y: f32) {
        panic!()
        // self.push(x, y);
    }

    fn close(&mut self) {
        // eprintln!("close");
        self.close();
    }
}

#[derive(Clone, Copy, Debug)]
struct Point {
    x: f32,
    y: f32,
    i: u32,
    next: u32,
    prev: u32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct Bezier {
    x: (f32, f32, f32),
    y: (f32, f32, f32),
}

impl Bezier {
    fn in_bounds(&self, max_x: f32, min_x: f32, max_y: f32, min_y: f32) -> bool {
        let in_bounds = self._in_bounds(max_x, min_x, max_y, min_y);
        // if !in_bounds {
        //     assert!(
        //         self.x.0 <= min_x || self.x.0 >= max_x || self.y.0 <= min_y || self.y.0 >= max_y,
        //         "{self:?} in {min_x} {max_x} {min_y} {max_y}"
        //     );
        //     assert!(
        //         self.x.2 <= min_x || self.x.2 >= max_x || self.y.2 <= min_y || self.y.2 >= max_y,
        //         "{self:?} in {min_x} {max_x} {min_y} {max_y}"
        //     );
        // }
        in_bounds
    }

    fn intersect_x(&self, value: f32, max: f32) -> i32 {
        let i = Self::intersect(&self.x, &self.y, value, (f32::MIN, max));
        // eprintln!(
        //     "({},{})->({},{}) {i}",
        //     self.x.0, self.y.0, self.x.2, self.y.2
        // );
        i
    }

    fn intersect(
        params: &(f32, f32, f32),
        other_params: &(f32, f32, f32),
        value: f32,
        bounds: (f32, f32),
    ) -> i32 {
        // eprintln!("{params:?} {value} {bounds:?}");
        fn eval(params: &(f32, f32, f32), t: f32) -> f32 {
            let one_sub_t = 1.0 - t;
            params.0 * one_sub_t * one_sub_t + 2.0 * params.1 * t * one_sub_t + params.2 * t * t
        }

        let denom = params.0 - 2.0 * params.1 + params.2;
        let (root1, root2) = if denom.abs() < 0.00001 {
            let root = (value - params.0) * 0.5 / (params.1 - params.0);
            (root, root)
        } else {
            let delta_l = params.1 * params.1 - params.0 * params.2;
            let delta = delta_l + value * denom;
            if delta < 0.0 {
                // eprintln!("no roots {delta}");
                return 0;
            }
            let delta_sqrt = delta.sqrt();
            let rool_l = params.0 - params.1;
            let root1 = (rool_l - delta_sqrt) / denom;
            let root2 = (rool_l + delta_sqrt) / denom;

            (root1, root2)
        };

        let mut c = 0;

        let (use1, use2) = match (params.2 > value, params.1 > value, params.0 > value) {
            (true, true, true) | (false, false, false) => (false, false),
            (true, true, false) | (true, false, false) => (false, true),
            (false, _, true) => (true, false),
            (true, false, true) | (false, true, false) => (true, true),
        };

        if use1 {
            // if root1 >= 0.0 && root1 < 1.0 {
            // if !use1 {
            //     eprintln!("root 1 {use1} {root1}");
            // }
            let val = eval(other_params, root1);

            if val >= bounds.0 && val <= bounds.1 {
                c += 1;
            }
        }

        if use2 {
            // if root2 >= 0.0 && root2 < 1.0 {
            // if !use2 {
            //     eprintln!(
            //         "root 2 {use2} {root2}, {} {} {}, {value}",
            //         params.0, params.1, params.2
            //     );
            // }
            let val = eval(other_params, root2);

            if val >= bounds.0 && val <= bounds.1 {
                c -= 1;
            }
        }

        // eprintln!("roots out of bounds {root1} {root2}");
        c
    }

    fn _in_bounds(&self, max_x: f32, min_x: f32, max_y: f32, min_y: f32) -> bool {
        Self::intersect(&self.x, &self.y, max_x, (min_y, max_y)) != 0
            || Self::intersect(&self.x, &self.y, min_x, (min_y, max_y)) != 0
            || Self::intersect(&self.y, &self.x, max_y, (min_x, max_x)) != 0
            || Self::intersect(&self.y, &self.x, min_y, (min_x, max_x)) != 0
            || self.x.0 <= max_x && self.x.0 >= min_x && self.y.0 <= max_y && self.y.0 >= min_y
    }
}
// 160
