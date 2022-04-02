use std::num::NonZeroU32;

use bytemuck::Zeroable;
use enum_iterator::IntoEnumIterator;
use rendering_util::RenderingContext;
use tracing::info;
use wgpu::BindGroup;
use wgpu::BindGroupDescriptor;
use wgpu::BindGroupEntry;
use wgpu::BindGroupLayout;
use wgpu::BindGroupLayoutDescriptor;
use wgpu::BindGroupLayoutEntry;
use wgpu::BindingResource;
use wgpu::BindingType;
use wgpu::BlendState;
use wgpu::Buffer;
use wgpu::BufferBinding;
use wgpu::BufferBindingType;
use wgpu::BufferDescriptor;
use wgpu::BufferSize;
use wgpu::BufferUsages;
use wgpu::ColorTargetState;
use wgpu::ColorWrites;
use wgpu::CommandEncoderDescriptor;
use wgpu::Extent3d;
use wgpu::FragmentState;
use wgpu::FrontFace;
use wgpu::LoadOp;
use wgpu::MultisampleState;
use wgpu::Operations;
use wgpu::PipelineLayout;
use wgpu::PipelineLayoutDescriptor;
use wgpu::PolygonMode;
use wgpu::PrimitiveState;
use wgpu::PrimitiveTopology;
use wgpu::RenderPassColorAttachment;
use wgpu::RenderPassDescriptor;
use wgpu::RenderPipeline;
use wgpu::RenderPipelineDescriptor;
use wgpu::ShaderModule;
use wgpu::ShaderStages;
use wgpu::Texture;
use wgpu::TextureAspect;
use wgpu::TextureDescriptor;
use wgpu::TextureDimension;
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
use crate::ecs::Resolution;
use crate::error::Error;
use crate::material::Material;
use crate::tile::TILE_SIZE;
use crate::tile::Tile;
use crate::tile::TileData;

use super::Globals;

#[repr(C, align(256))]
#[derive(Clone, Copy, Debug, Zeroable)]
struct Locals {
    pub chunk_position: [i32; 2],
}

#[allow(dead_code)]
pub struct ChunkRenderer {
    shader: ShaderModule,
    bind_group_layout: BindGroupLayout,
    pipeline_layout: PipelineLayout,
    pipeline: RenderPipeline,
    locals: Buffer,
    chunks: Buffer,
    tiles: Buffer,
    materials: Texture,
    materials_view: TextureView,
    bind_group: BindGroup,
}

impl ChunkRenderer {
    pub fn new(rc: &RenderingContext, globals: &Buffer, resolution: Resolution) -> Result<Self, Error> {
        // Load this now to test for compilation errors
        let shader = rc.device.create_shader_module(&include_wgsl!("shaders/chunk.wgsl"));

        let chunks_per_row = (resolution.width as f32 / (CHUNK_SIZE * TILE_SIZE) as f32).ceil() as u64 + 1;
        let chunks_per_column = (resolution.height as f32 / (CHUNK_SIZE * TILE_SIZE) as f32).ceil() as u64 + 1;
        let chunk_data_size = std::mem::size_of::<Chunk>() as u64 * (chunks_per_row + 2) * (chunks_per_column + 2);

        let bind_group_layout = rc.device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("chunk_renderer::bind_group_layout"),
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::VERTEX_FRAGMENT,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: BufferSize::new(std::mem::size_of::<Globals>() as _),
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: true,
                        min_binding_size: BufferSize::new(std::mem::size_of::<Locals>() as _),
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
                        view_dimension: TextureViewDimension::D2Array,
                        sample_type: TextureSampleType::Float { filterable: false },
                    },
                    count: None,
                },
            ],
        });

        let pipeline_layout = rc.device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("chunk_renderer::pipeline_layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = rc.device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("chunk_renderer::pipeline"),
            layout: Some(&pipeline_layout),
            vertex: VertexState {
                module: &shader,
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
                module: &shader,
                entry_point: "fs_main",
                targets: &[ColorTargetState {
                    format: rc.surface_format(),
                    blend: Some(BlendState::ALPHA_BLENDING),
                    write_mask: ColorWrites::ALL,
                }],
            }),
            multiview: None,
        });

        let locals = rc.device.create_buffer(&BufferDescriptor {
            label: Some("chunk_renderer::locals"),
            size: std::mem::size_of::<Locals>() as _,
            usage: BufferUsages::COPY_DST | BufferUsages::UNIFORM,
            mapped_at_creation: false,
        });

        let chunks = rc.device.create_buffer(&BufferDescriptor {
            label: Some("chunk_renderer::chunks"),
            size: chunk_data_size,
            usage: BufferUsages::COPY_DST | BufferUsages::STORAGE,
            mapped_at_creation: false,
        });

        let tiles = rc.device.create_buffer_init(&BufferInitDescriptor {
            label: Some("chunk_renderer::tiles"),
            contents: bytemuck::cast_slice(&Tile::tile_data()),
            usage: BufferUsages::STORAGE,
        });

        let (material_data, material_size) = material_data()?;
        let materials = rc.device.create_texture(&TextureDescriptor {
            label: Some("tile_atlas"),
            size: material_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: rc.surface_format(),
            usage: TextureUsages::COPY_DST | TextureUsages::TEXTURE_BINDING,
        });

        rc.queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &materials,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &material_data,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: NonZeroU32::new(material_size.width * rc.surface_format().describe().block_size as u32),
                rows_per_image: NonZeroU32::new(material_size.height),
            },
            material_size,
        );

        let materials_view = materials.create_view(&TextureViewDescriptor {
            label: Some("tile_atlas_view"),
            format: Some(rc.surface_format()),
            dimension: Some(TextureViewDimension::D2Array),
            aspect: TextureAspect::All,
            ..Default::default()
        });

        let bind_group = rc.device.create_bind_group(&BindGroupDescriptor {
            label: Some("chunk_bind_group"),
            layout: &bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: globals.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::Buffer(BufferBinding {
                        buffer: &locals,
                        offset: 0,
                        size: BufferSize::new(std::mem::size_of::<Locals>() as _),
                    }),
                },
                BindGroupEntry {
                    binding: 2,
                    resource: chunks.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 3,
                    resource: tiles.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 4,
                    resource: BindingResource::TextureView(&materials_view),
                },
            ],
        });

        Ok(Self {
            shader,
            bind_group_layout,
            pipeline_layout,
            pipeline,
            locals,
            chunks,
            tiles,
            materials,
            materials_view,
            bind_group,
        })
    }

    pub fn write_chunks(&self, rc: &RenderingContext, chunks: &[Chunk]) {
        rc.queue.write_buffer(&self.chunks, 0, bytemuck::cast_slice(chunks));
    }

    pub fn render(
        &self,
        rc: &RenderingContext,
        surface_view: &TextureView,
    ) {
        // Build our command encoder
        let mut command_encoder = rc.device.create_command_encoder(&CommandEncoderDescriptor {
            label: Some("command_encoder"),
        });

        // Render chunks!
        {
            let mut render_pass = command_encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some("render_game"),
                color_attachments: &[RenderPassColorAttachment {
                    view: surface_view,
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Clear(CHUNK_CLEAR_COLOR),
                        store: true,
                    },
                }],
                depth_stencil_attachment: None,
            });
    
            render_pass.set_pipeline(&self.pipeline);
            render_pass.set_bind_group(0, &self.bind_group, &[0]);
            render_pass.draw(0..4, 0..1);
        }

        // Submit our work
        rc.queue.submit([command_encoder.finish()]);
    }
}

fn material_data() -> Result<(Vec<u8>, Extent3d), Error> {
    let mut bytes = vec![];
    let mut i = 0;
    for material in Material::into_enum_iter() {
        let data = std::fs::read(format!("./materials/{}.png", material.texture_path()))?;
        let image = image::load_from_memory(&data)?;
        bytes.append(&mut image.into_bytes());
        i += 1;
    }

    Ok((bytes, Extent3d { width: TILE_SIZE * 4, height: TILE_SIZE * 4, depth_or_array_layers: i }))
}
