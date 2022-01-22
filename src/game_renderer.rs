use std::num::NonZeroU32;

use bytemuck::Zeroable;
use image::ImageDecoder;
use image::gif::GifDecoder;
use wgpu::BindGroup;
use wgpu::BindGroupDescriptor;
use wgpu::BindGroupEntry;
use wgpu::BindGroupLayout;
use wgpu::BindGroupLayoutDescriptor;
use wgpu::BindGroupLayoutEntry;
use wgpu::BindingResource;
use wgpu::BindingType;
use wgpu::BlendComponent;
use wgpu::BlendFactor;
use wgpu::BlendOperation;
use wgpu::BlendState;
use wgpu::Buffer;
use wgpu::BufferBinding;
use wgpu::BufferBindingType;
use wgpu::BufferDescriptor;
use wgpu::BufferSize;
use wgpu::BufferUsages;
use wgpu::Color;
use wgpu::ColorTargetState;
use wgpu::ColorWrites;
use wgpu::CommandEncoder;
use wgpu::Extent3d;
use wgpu::FragmentState;
use wgpu::FrontFace;
use wgpu::LoadOp;
use wgpu::MultisampleState;
use wgpu::Operations;
use wgpu::PipelineLayoutDescriptor;
use wgpu::PolygonMode;
use wgpu::PrimitiveState;
use wgpu::PrimitiveTopology;
use wgpu::RenderPassColorAttachment;
use wgpu::RenderPassDescriptor;
use wgpu::RenderPipeline;
use wgpu::RenderPipelineDescriptor;
use wgpu::ShaderStages;
use wgpu::TextureAspect;
use wgpu::TextureDescriptor;
use wgpu::TextureDimension;
use wgpu::TextureFormat;
use wgpu::TextureSampleType;
use wgpu::TextureUsages;
use wgpu::TextureView;
use wgpu::TextureViewDescriptor;
use wgpu::TextureViewDimension;
use wgpu::VertexState;
use wgpu::include_wgsl;
use wgpu::util::BufferInitDescriptor;
use wgpu::util::DeviceExt;

use crate::chunk::CHUNK_CLEAR_COLOR;
use crate::chunk::CHUNK_SIZE;
use crate::chunk::Chunk;
use crate::entity::Entity;
use crate::light::Light;
use crate::rendering_context::RenderingContext;
use crate::tile::TILE_SIZE;
use crate::tile::Tile;
use crate::tile::TileData;

const TEXTURE_FORMAT: TextureFormat = TextureFormat::Bgra8UnormSrgb;
const MIN_ENTITY_COUNT: u64 = 16;
const MIN_LIGHT_COUNT: u64 = 16;

#[repr(C, align(256))]
#[derive(Clone, Copy, Debug, Zeroable)]
struct ChunkLocals {
    pub chunk_position: [i32; 2],
}

pub struct GameRenderer {
    lighting_bind_group_layout: BindGroupLayout,
    chunk_pipeline: RenderPipeline,
    entity_pipeline: RenderPipeline,
    lighting_pipeline: RenderPipeline,
    chunk_locals: Buffer,
    chunk_data: Buffer,
    entity_locals: Buffer,
    lighting_locals: Buffer,
    chunk_bind_group: BindGroup,
    entity_bind_group: BindGroup,
    unlit_view: TextureView,
    lighting_bind_group: BindGroup,

    entity_count: u32,
    light_count: u32,
}

impl GameRenderer {
    pub fn new(rc: &RenderingContext, resolution: [u32; 2]) -> Self {
        // Load these now to test for compilation errors
        let vs_quad = rc.device.create_shader_module(&include_wgsl!("shaders/vs_quad.wgsl"));
        let fs_chunk = rc.device.create_shader_module(&include_wgsl!("shaders/fs_chunk.wgsl"));
        let fs_entity = rc.device.create_shader_module(&include_wgsl!("shaders/fs_entity.wgsl"));
        let light = rc.device.create_shader_module(&include_wgsl!("shaders/light.wgsl"));

        let chunks_per_row = (resolution[0] as f32 / (CHUNK_SIZE * TILE_SIZE) as f32).ceil() as u64 + 1;
        let chunks_per_column = (resolution[1] as f32 / (CHUNK_SIZE * TILE_SIZE) as f32).ceil() as u64 + 1;
        let chunk_data_size = std::mem::size_of::<Chunk>() as u64 * (chunks_per_row + 2) * (chunks_per_column + 2);

        let chunk_bind_group_layout = rc.device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("chunk_bind_group_layout"),
            entries: &[
                rc.globals_bind_group_layout_entry(),
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: true,
                        min_binding_size: BufferSize::new(std::mem::size_of::<ChunkLocals>() as _),
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 2,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: BufferSize::new(chunk_data_size),
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 3,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: BufferSize::new(std::mem::size_of::<[TileData; 256]>() as _),
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 4,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Texture {
                        multisampled: false,
                        view_dimension: TextureViewDimension::D2,
                        sample_type: TextureSampleType::Float { filterable: false },
                    },
                    count: None,
                },
            ],
        });

        let entity_bind_group_layout = rc.device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("entity_bind_group_layout"),
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::VERTEX_FRAGMENT,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: true,
                        min_binding_size: BufferSize::new(0),
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Texture {
                        multisampled: false,
                        view_dimension: TextureViewDimension::D2,
                        sample_type: TextureSampleType::Float { filterable: false },
                    },
                    count: None,
                },
            ],
        });

        let lighting_bind_group_layout = rc.device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("lighting_bind_group_layout"),
            entries: &[
                rc.globals_bind_group_layout_entry(),
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::VERTEX_FRAGMENT,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: true,
                        min_binding_size: BufferSize::new(0),
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 2,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Texture {
                        multisampled: false,
                        view_dimension: TextureViewDimension::D2,
                        sample_type: TextureSampleType::Float { filterable: false },
                    },
                    count: None,
                },
            ],
        });

        let chunk_pipeline_layout = rc.device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("chunk_pipeline_layout"),
            bind_group_layouts: &[&chunk_bind_group_layout],
            push_constant_ranges: &[],
        });

        let entity_pipeline_layout = rc.device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("entity_pipeline_layout"),
            bind_group_layouts: &[&entity_bind_group_layout],
            push_constant_ranges: &[],
        });

        let lighting_pipeline_layout = rc.device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("lighting_pipeline_layout"),
            bind_group_layouts: &[&lighting_bind_group_layout],
            push_constant_ranges: &[],
        });

        let chunk_pipeline = rc.device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("chunk_pipeline"),
            layout: Some(&chunk_pipeline_layout),
            vertex: VertexState {
                module: &vs_quad,
                entry_point: "vs_main",
                buffers: &[],
            },
            primitive: PrimitiveState {
                topology: PrimitiveTopology::TriangleStrip,
                strip_index_format: None,
                front_face: FrontFace::Ccw,
                cull_mode: None,
                unclipped_depth: false,
                polygon_mode: PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: None,
            multisample: MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            fragment: Some(FragmentState {
                module: &fs_chunk,
                entry_point: "fs_main",
                targets: &[ColorTargetState {
                    format: TEXTURE_FORMAT,
                    blend: Some(BlendState::ALPHA_BLENDING),
                    write_mask: ColorWrites::ALL,
                }],
            }),
            multiview: None,
        });

        let entity_pipeline = rc.device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("entity_pipeline"),
            layout: Some(&entity_pipeline_layout),
            vertex: VertexState {
                module: &vs_quad,
                entry_point: "vs_main",
                buffers: &[],
            },
            primitive: PrimitiveState {
                topology: PrimitiveTopology::TriangleStrip,
                strip_index_format: None,
                front_face: FrontFace::Ccw,
                cull_mode: None,
                unclipped_depth: false,
                polygon_mode: PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: None,
            multisample: MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            fragment: Some(FragmentState {
                module: &fs_entity,
                entry_point: "fs_main",
                targets: &[ColorTargetState {
                    format: TEXTURE_FORMAT,
                    blend: Some(BlendState::ALPHA_BLENDING),
                    write_mask: ColorWrites::ALL,
                }],
            }),
            multiview: None,
        });

        let lighting_pipeline = rc.device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("lighting_pipeline"),
            layout: Some(&lighting_pipeline_layout),
            vertex: VertexState {
                module: &light,
                entry_point: "vs_main",
                buffers: &[],
            },
            primitive: PrimitiveState {
                topology: PrimitiveTopology::TriangleStrip,
                strip_index_format: None,
                front_face: FrontFace::Ccw,
                cull_mode: None,
                unclipped_depth: false,
                polygon_mode: PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: None,
            multisample: MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            fragment: Some(FragmentState {
                module: &light,
                entry_point: "fs_main",
                targets: &[ColorTargetState {
                    format: TEXTURE_FORMAT,
                    blend: Some(BlendState {
                        color: BlendComponent {
                            src_factor: BlendFactor::One,
                            dst_factor: BlendFactor::One,
                            operation: BlendOperation::Add,
                        },
                        alpha: BlendComponent::REPLACE,
                    }),
                    write_mask: ColorWrites::ALL,
                }],
            }),
            multiview: None,
        });

        let chunk_locals = rc.device.create_buffer(&BufferDescriptor {
            label: Some("chunk_locals"),
            size: std::mem::size_of::<ChunkLocals>() as _,
            usage: BufferUsages::COPY_DST | BufferUsages::UNIFORM,
            mapped_at_creation: false,
        });

        let chunk_data = rc.device.create_buffer(&BufferDescriptor {
            label: Some("chunk_data"),
            size: chunk_data_size,
            usage: BufferUsages::COPY_DST | BufferUsages::STORAGE,
            mapped_at_creation: false,
        });

        let tile_data = rc.device.create_buffer_init(&BufferInitDescriptor {
            label: Some("tile_data"),
            contents: bytemuck::cast_slice(&Tile::tiles()),
            usage: BufferUsages::STORAGE,
        });

        let (tile_atlas_data, tile_atlas_size) = tile_atlas();
        let tile_atlas = rc.device.create_texture(&TextureDescriptor {
            label: Some("tile_atlas"),
            size: tile_atlas_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TEXTURE_FORMAT,
            usage: TextureUsages::COPY_DST | TextureUsages::TEXTURE_BINDING,
        });

        rc.queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &tile_atlas,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &tile_atlas_data,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: NonZeroU32::new(tile_atlas_size.width * TEXTURE_FORMAT.describe().block_size as u32),
                rows_per_image: None,
            },
            tile_atlas_size,
        );

        let tile_atlas_view = tile_atlas.create_view(&TextureViewDescriptor {
            label: Some("tile_atlas_view"),
            format: Some(TEXTURE_FORMAT),
            dimension: Some(TextureViewDimension::D2),
            aspect: TextureAspect::All,
            ..Default::default()
        });

        let chunk_bind_group = rc.device.create_bind_group(&BindGroupDescriptor {
            label: Some("chunk_bind_group"),
            layout: &chunk_bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: rc.globals.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::Buffer(BufferBinding {
                        buffer: &chunk_locals,
                        offset: 0,
                        size: BufferSize::new(std::mem::size_of::<ChunkLocals>() as _),
                    }),
                },
                BindGroupEntry {
                    binding: 2,
                    resource: chunk_data.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 3,
                    resource: tile_data.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 4,
                    resource: BindingResource::TextureView(&tile_atlas_view),
                },
            ],
        });

        let entity_locals = rc.device.create_buffer(&BufferDescriptor {
            label: Some("entity_locals"),
            size: std::mem::size_of::<Entity>() as u64 * MIN_ENTITY_COUNT,
            usage: BufferUsages::COPY_DST | BufferUsages::UNIFORM,
            mapped_at_creation: false,
        });

        let (entity_atlas_data, entity_atlas_size) = entity_atlas();
        let entity_atlas = rc.device.create_texture(&TextureDescriptor {
            label: Some("entity_atlas"),
            size: entity_atlas_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TEXTURE_FORMAT,
            usage: TextureUsages::COPY_DST | TextureUsages::TEXTURE_BINDING,
        });

        rc.queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &entity_atlas,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &entity_atlas_data,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: NonZeroU32::new(entity_atlas_size.width * TEXTURE_FORMAT.describe().block_size as u32),
                rows_per_image: None,
            },
            entity_atlas_size,
        );

        let entity_atlas_view = entity_atlas.create_view(&TextureViewDescriptor {
            label: Some("entity_atlas_view"),
            format: Some(TEXTURE_FORMAT),
            dimension: Some(TextureViewDimension::D2),
            aspect: TextureAspect::All,
            ..Default::default()
        });

        let entity_bind_group = rc.device.create_bind_group(&BindGroupDescriptor {
            label: Some("entity_bind_group"),
            layout: &entity_bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::Buffer(BufferBinding {
                        buffer: &entity_locals,
                        offset: 0,
                        size: BufferSize::new(std::mem::size_of::<Entity>() as _),
                    }),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::TextureView(&entity_atlas_view),
                },
            ],
        });

        let lighting_locals = rc.device.create_buffer(&BufferDescriptor {
            label: Some("lighting_locals"),
            size: std::mem::size_of::<Light>() as u64 * MIN_LIGHT_COUNT,
            usage: BufferUsages::COPY_DST | BufferUsages::UNIFORM,
            mapped_at_creation: false,
        });

        let unlit_view = unlit_view(rc, resolution);

        let lighting_bind_group = lighting_bind_group(
            &rc,
            &lighting_bind_group_layout,
            &lighting_locals,
            &unlit_view,
        );

        Self {
            lighting_bind_group_layout,
            chunk_pipeline,
            entity_pipeline,
            lighting_pipeline,
            chunk_locals,
            entity_locals,
            lighting_locals,
            chunk_data,
            chunk_bind_group,
            entity_bind_group,
            unlit_view,
            lighting_bind_group,

            entity_count: 0,
            light_count: 0,
        }
    }

    pub fn write_chunks(&self, rc: &RenderingContext, chunks: &[Chunk]) {
        //rc.queue.write_buffer(
        //    &self.chunk_locals,
        //    0,
        //    unsafe {
        //        std::slice::from_raw_parts(
        //            locals.as_ptr() as *const u8,
        //            locals.len() * std::mem::size_of::<ChunkLocals>(),
        //        )
        //    },
        //);

        rc.queue.write_buffer(
            &self.chunk_data,
            0,
            unsafe {
                std::slice::from_raw_parts(
                    chunks.as_ptr() as *const u8,
                    chunks.len() * std::mem::size_of::<Chunk>(),
                )
            },
        );
    }

    pub fn write_entities(&mut self, rc: &RenderingContext, entities: &[Entity]) {
        self.entity_count = entities.len() as _;

        rc.queue.write_buffer(
            &self.entity_locals,
            0,
            unsafe {
                std::slice::from_raw_parts(
                    entities.as_ptr() as *const u8,
                    entities.len() * std::mem::size_of::<Entity>(),
                )
            },
        )
    }

    pub fn write_lights(&mut self, rc: &RenderingContext, lights: &[Light]) {
        self.light_count = lights.len() as _;

        rc.queue.write_buffer(
            &self.lighting_locals,
            0,
            unsafe {
                std::slice::from_raw_parts(
                    lights.as_ptr() as *const u8,
                    lights.len() * std::mem::size_of::<Light>(),
                )
            },
        )
    }

    pub fn resize(&mut self, rc: &RenderingContext, resolution: [u32; 2]) {
        self.unlit_view = unlit_view(rc, resolution);
        self.lighting_bind_group = lighting_bind_group(
            &rc,
            &self.lighting_bind_group_layout,
            &self.lighting_locals,
            &self.unlit_view,
        );
    }

    pub fn render(
        &self,
        view: &TextureView,
        command_encoder: &mut CommandEncoder,
    ) {
        let mut render_pass = command_encoder.begin_render_pass(&RenderPassDescriptor {
            label: Some("render_game"),
            color_attachments: &[RenderPassColorAttachment {
                view: &self.unlit_view,
                resolve_target: None,
                ops: Operations {
                    load: LoadOp::Clear(CHUNK_CLEAR_COLOR),
                    store: true,
                },
            }],
            depth_stencil_attachment: None,
        });

        render_pass.set_pipeline(&self.chunk_pipeline);
        render_pass.set_bind_group(0, &self.chunk_bind_group, &[0]);
        render_pass.draw(0..4, 0..1);

        render_pass.set_pipeline(&self.entity_pipeline);
        for i in 0..self.entity_count {
            render_pass.set_scissor_rect(0, 0, TILE_SIZE, TILE_SIZE);
            render_pass.set_bind_group(0, &self.entity_bind_group, &[i * std::mem::size_of::<Entity>() as u32]);
            render_pass.draw(0..4, 0..1);
        }
        drop(render_pass);

        let mut render_pass = command_encoder.begin_render_pass(&RenderPassDescriptor {
            label: Some("render_lights"),
            color_attachments: &[RenderPassColorAttachment {
                view,
                resolve_target: None,
                ops: Operations {
                    load: LoadOp::Clear(Color::BLACK),
                    store: true,
                },
            }],
            depth_stencil_attachment: None,
        });

        render_pass.set_pipeline(&self.lighting_pipeline);
        for i in 0..self.light_count {
            render_pass.set_bind_group(0, &self.lighting_bind_group, &[i * std::mem::size_of::<Light>() as u32]);
            render_pass.draw(0..4, 0..1);
        }
        drop(render_pass);
    }
}

fn tile_atlas() -> (Vec<u8>, Extent3d) {
    let bytes = include_bytes!("textures/tiles.gif");
    let decoder = GifDecoder::new(&bytes[..]).unwrap();
    let dimensions = decoder.dimensions();

    let mut buf = vec![0; decoder.total_bytes() as usize];
    decoder.read_image(&mut buf).unwrap();

    (buf, Extent3d { width: dimensions.0, height: dimensions.1, depth_or_array_layers: 1 })
}

fn entity_atlas() -> (Vec<u8>, Extent3d) {
    let bytes = include_bytes!("textures/entities.gif");
    let decoder = GifDecoder::new(&bytes[..]).unwrap();
    let dimensions = decoder.dimensions();

    let mut buf = vec![0; decoder.total_bytes() as usize];
    decoder.read_image(&mut buf).unwrap();

    (buf, Extent3d { width: dimensions.0, height: dimensions.1, depth_or_array_layers: 1 })
}

fn unlit_view(rc: &RenderingContext, resolution: [u32; 2]) -> TextureView {
    let unlit_texture = rc.device.create_texture(&TextureDescriptor {
        label: Some("unlit_texture"),
        size: Extent3d { width: resolution[0], height: resolution[1], depth_or_array_layers: 1 },
        mip_level_count: 1,
        sample_count: 1,
        dimension: TextureDimension::D2,
        format: TEXTURE_FORMAT,
        usage: TextureUsages::TEXTURE_BINDING | TextureUsages::RENDER_ATTACHMENT,
    });

    unlit_texture.create_view(&TextureViewDescriptor::default())
}

fn lighting_bind_group(
    rc: &RenderingContext,
    layout: &BindGroupLayout,
    locals: &Buffer,
    unlit_view: &TextureView,
) -> BindGroup {
    rc.device.create_bind_group(&BindGroupDescriptor {
        label: Some("lighting_bind_group"),
        layout,
        entries: &[
            BindGroupEntry {
                binding: 0,
                resource: rc.globals.as_entire_binding(),
            },
            BindGroupEntry {
                binding: 1,
                resource: BindingResource::Buffer(BufferBinding {
                    buffer: locals,
                    offset: 0,
                    size: BufferSize::new(std::mem::size_of::<Light>() as _),
                }),
            },
            BindGroupEntry {
                binding: 2,
                resource: BindingResource::TextureView(unlit_view),
            },
        ],
    })
}
