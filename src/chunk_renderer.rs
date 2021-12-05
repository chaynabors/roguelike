use std::num::NonZeroU32;

use image::ImageDecoder;
use image::gif::GifDecoder;
use wgpu::BindGroup;
use wgpu::BindGroupDescriptor;
use wgpu::BindGroupEntry;
use wgpu::BindGroupLayoutDescriptor;
use wgpu::BindGroupLayoutEntry;
use wgpu::BindingResource;
use wgpu::BindingType;
use wgpu::BlendState;
use wgpu::BufferBinding;
use wgpu::BufferBindingType;
use wgpu::BufferDescriptor;
use wgpu::BufferSize;
use wgpu::BufferUsages;
use wgpu::ColorTargetState;
use wgpu::ColorWrites;
use wgpu::CommandEncoder;
use wgpu::Device;
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
use wgpu::Queue;
use wgpu::RenderPassColorAttachment;
use wgpu::RenderPassDescriptor;
use wgpu::RenderPipeline;
use wgpu::RenderPipelineDescriptor;
use wgpu::ShaderModuleDescriptor;
use wgpu::ShaderSource;
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
use wgpu::util::BufferInitDescriptor;
use wgpu::util::DeviceExt;

use crate::chunk::CHUNK_CLEAR_COLOR;
use crate::chunk::Chunk;
use crate::chunk::ChunkLayout;
use crate::tile::Tile;

const TEXTURE_FORMAT: TextureFormat = TextureFormat::Bgra8UnormSrgb;

static UNIFORMS_SIZE: u64 = std::mem::size_of::<ChunkLayout>() as u64 * 5;

struct ChunkRenderer {
    pipeline: RenderPipeline,
    bind_group: BindGroup,
}

impl ChunkRenderer {
    fn new(device: &Device, queue: &Queue) -> Self {
        let shader = device.create_shader_module(&ShaderModuleDescriptor {
            label: Some("chunk_shader"),
            source: ShaderSource::Wgsl(include_str!("shaders/chunk.wgsl").into()),
        });

        let bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("chunk_bind_group_layout"),
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: BufferSize::new(UNIFORMS_SIZE),
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
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

        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("chunk_pipeline_layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("chunk_pipeline"),
            layout: Some(&pipeline_layout),
            vertex: VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[],
            },
            primitive: PrimitiveState {
                topology: PrimitiveTopology::TriangleStrip,
                strip_index_format: None,
                front_face: FrontFace::Cw,
                cull_mode: None,
                clamp_depth: false,
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
                    format: TEXTURE_FORMAT,
                    blend: Some(BlendState::ALPHA_BLENDING),
                    write_mask: ColorWrites::ALL,
                }],
            }),
        });

        let uniforms = device.create_buffer(&BufferDescriptor {
            label: Some("chunk_uniforms"),
            size: UNIFORMS_SIZE,
            usage: BufferUsages::COPY_DST | BufferUsages::UNIFORM,
            mapped_at_creation: false,
        });

        let tile_data = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("tile_data"),
            contents: bytemuck::cast_slice(&Tile::render_atlas()),
            usage: BufferUsages::STORAGE,
        });

        let (tile_atlas_data, tile_atlas_size) = tile_atlas();
        let tile_atlas = device.create_texture(&TextureDescriptor {
            label: Some("tile_atlas"),
            size: tile_atlas_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TEXTURE_FORMAT,
            usage: TextureUsages::COPY_DST | TextureUsages::TEXTURE_BINDING,
        });

        queue.write_texture(
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

        let bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("chunk_bind_group"),
            layout: &bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::Buffer(BufferBinding {
                        buffer: &uniforms,
                        offset: 0,
                        size: BufferSize::new(UNIFORMS_SIZE),
                    }),
                },
                BindGroupEntry {
                    binding: 2,
                    resource: tile_data.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 3,
                    resource: BindingResource::TextureView(&tile_atlas_view),
                },
            ],
        });

        Self {
            pipeline,
            bind_group,
        }
    }

    fn render(
        &self,
        command_encoder: &mut CommandEncoder,
        view: &TextureView,
        chunk: &Chunk,
        neighbors: [&Chunk; 4],
    ) {
        let mut render_pass = command_encoder.begin_render_pass(&RenderPassDescriptor {
            label: Some("render_chunk"),
            color_attachments: &[RenderPassColorAttachment {
                view,
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
}

fn tile_atlas() -> (Vec<u8>, Extent3d) {
    let bytes = include_bytes!("textures/tiles.gif");
    let decoder = GifDecoder::new(&bytes[..]).unwrap();
    let dimensions = decoder.dimensions();

    let mut buf = vec![0; decoder.total_bytes() as usize];
    decoder.read_image(&mut buf).unwrap();

    (buf, Extent3d { width: dimensions.0, height: dimensions.1, depth_or_array_layers: 1 })
}
