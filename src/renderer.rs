use std::num::NonZeroU32;

use bytemuck::Pod;
use bytemuck::Zeroable;
use image::ImageDecoder;
use image::gif::GifDecoder;
use log::warn;
use wgpu::Backends;
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
use wgpu::BufferSize;
use wgpu::BufferUsages;
use wgpu::ColorTargetState;
use wgpu::ColorWrites;
use wgpu::CommandEncoderDescriptor;
use wgpu::Device;
use wgpu::DeviceDescriptor;
use wgpu::Extent3d;
use wgpu::Face;
use wgpu::Features;
use wgpu::FragmentState;
use wgpu::FrontFace;
use wgpu::IndexFormat;
use wgpu::Instance;
use wgpu::Limits;
use wgpu::LoadOp;
use wgpu::MultisampleState;
use wgpu::Operations;
use wgpu::PipelineLayoutDescriptor;
use wgpu::PolygonMode;
use wgpu::PresentMode;
use wgpu::PrimitiveState;
use wgpu::PrimitiveTopology;
use wgpu::Queue;
use wgpu::RenderPassColorAttachment;
use wgpu::RenderPassDescriptor;
use wgpu::RenderPipeline;
use wgpu::RenderPipelineDescriptor;
use wgpu::RequestAdapterOptions;
use wgpu::ShaderModuleDescriptor;
use wgpu::ShaderSource;
use wgpu::ShaderStages;
use wgpu::Surface;
use wgpu::SurfaceConfiguration;
use wgpu::SurfaceError;
use wgpu::TextureAspect;
use wgpu::TextureDescriptor;
use wgpu::TextureDimension;
use wgpu::TextureFormat;
use wgpu::TextureSampleType;
use wgpu::TextureUsages;
use wgpu::TextureView;
use wgpu::TextureViewDescriptor;
use wgpu::TextureViewDimension;
use wgpu::VertexAttribute;
use wgpu::VertexBufferLayout;
use wgpu::VertexFormat;
use wgpu::VertexState;
use wgpu::util::BufferInitDescriptor;
use wgpu::util::DeviceExt;
use winit::dpi::PhysicalSize;

use crate::chunk::CHUNK_SIZE;
use crate::error::Error;
use crate::tile::Tile;

const TEXTURE_FORMAT: TextureFormat = TextureFormat::Bgra8UnormSrgb;
const TILE_SIZE: u32 = 16;

const SQUARE_VERTICES: &[Vertex] = &[
    Vertex { position: [-1.0, 1.0], tex_coord: [0.0, 0.0] },
    Vertex { position: [1.0, 1.0], tex_coord: [1.0, 0.0] },
    Vertex { position: [-1.0, -1.0], tex_coord: [0.0, 1.0] },
    Vertex { position: [1.0, -1.0], tex_coord: [1.0, 1.0] },
];

const SQUARE_INDICES: &[u16] = &[
    0, 1, 2, 2, 1, 3,
];

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct Vertex {
    position: [f32; 2],
    tex_coord: [f32; 2],
}

impl Vertex {
    fn desc<'a>() -> VertexBufferLayout<'a> {
        VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: VertexFormat::Float32x2,
                },
                VertexAttribute {
                    offset: 4 * 2,
                    shader_location: 1,
                    format: VertexFormat::Float32x2,
                },
            ],
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Pod, Zeroable)]
struct Globals {
    resolution: [u32; 2],
    tile_size: u32,
    chunk_size: u32,
}

impl Globals {
    fn new(resolution: [u32; 2]) -> Self {
        Self {
            resolution,
            tile_size: TILE_SIZE,
            chunk_size: CHUNK_SIZE,
        }
    }
}

#[repr(C, align(2048))]
#[derive(Clone, Copy, Debug, Zeroable)]
struct ChunkLocals {
    position: [u32; 2],
    _pad1: [u8; 8],
    layout: [u128; (CHUNK_SIZE * CHUNK_SIZE / 4) as usize],
}

impl ChunkLocals {
    fn new() -> Self {
        Self {
            position: [0, 0],
            _pad1: [0; 8],
            layout: [
                0x81818181, 0x81818181, 0x81818181, 0x81818181,
                0x81810000, 0, 0, 0x00008181,
                0x81810000, 0x81808180, 0, 0x00008181,
                0x81810000, 0, 0, 0x00008181,
                0x81810000, 0, 0x81808180, 0x00008181,
                0x81810000, 0, 0, 0x00008181,
                0x81810000, 0, 0, 0x00008181,
                0x81810000, 0, 0, 0x00008181,
                0x81810000, 0, 0, 0x00008181,
                0x81810000, 0x81818181, 0, 0x00008181,
                0x81810000, 0, 0, 0x00008181,
                0x81810000, 0, 0, 0x00008181,
                0x81810000, 0, 0, 0x00008181,
                0x81810000, 0, 0, 0x00008181,
                0x81810000, 0, 0, 0x00008181,
                0x81818181, 0x81818181, 0x81818181, 0x81818181,
            ],
        }
    }
}

#[repr(C, align(256))]
#[derive(Clone, Copy, Debug, Zeroable)]
struct EntityLocals {
    position: [u32; 2],
    atlas_position: [u32; 2],
    color: u32,
    detail: u32,
}

impl EntityLocals {
    fn new() -> Self {
        Self {
            position: [0, 0],
            atlas_position: [0, 0],
            color: u32::MAX,
            detail: u32::MAX,
        }
    }
}

#[repr(C, align(256))]
#[derive(Clone, Copy, Debug, Zeroable)]
struct LightingLocals {
    position: [f32; 2],
    color: u32,
    magnitude: f32,
}

impl LightingLocals {
    fn new() -> [Self; 3] {
        [
            Self {
                position: [0.0, 0.0],
                color: 0x000000ff,
                magnitude: 1.0,
            },
            Self {
                position: [1.0, 0.0],
                color: 0x00ff0000,
                magnitude: 1.0,
            },
            Self {
                position: [0.5, 1.0],
                color: 0x0000ff00,
                magnitude: 1.0,
            },
        ]
    }
}

pub struct Renderer {
    surface: Surface,
    device: Device,
    queue: Queue,
    surface_configuration: SurfaceConfiguration,
    lighting_bind_group_layout: BindGroupLayout,
    chunk_pipeline: RenderPipeline,
    entity_pipeline: RenderPipeline,
    lighting_pipeline: RenderPipeline,
    vertex_buffer: Buffer,
    index_buffer: Buffer,
    globals: Buffer,
    chunk_locals: Buffer,
    entity_locals: Buffer,
    lighting_locals: Buffer,
    chunk_bind_group: BindGroup,
    entity_bind_group: BindGroup,
    unlit_view: TextureView,
    lighting_bind_group: BindGroup,
}

impl Renderer {
    pub async fn new(window: &winit::window::Window) -> Result<Self, Error> {
        println!("{}", std::mem::size_of::<ChunkLocals>());

        let instance = Instance::new(Backends::PRIMARY);

        let surface = unsafe { instance.create_surface(window) };

        let adapter = match instance.request_adapter(&RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            force_fallback_adapter: false,
            compatible_surface: Some(&surface),
        }).await {
            Some(adapter) => adapter,
            None => return Err(Error::NoSuitableGraphicsAdapter),
        };

        let (device, queue) = match adapter.request_device(
            &DeviceDescriptor { label: Some("device"), features: Features::empty(), limits: Limits::default() },
            None,
        ).await {
            Ok(dq) => dq,
            Err(_) => return Err(Error::NoSuitableGraphicsDevice),
        };

        let resolution = window.inner_size();
        let surface_configuration = SurfaceConfiguration {
            usage: TextureUsages::COPY_DST | TextureUsages::RENDER_ATTACHMENT,
            format: match surface.get_preferred_format(&adapter) {
                Some(format) => format,
                None => return Err(Error::IncompatibleSurface),
            },
            width: resolution.width,
            height: resolution.height,
            present_mode: PresentMode::Fifo,
        };
        surface.configure(&device, &surface_configuration);

        let chunk_shader = device.create_shader_module(&ShaderModuleDescriptor {
            label: Some("chunk_shader"),
            source: ShaderSource::Wgsl(include_str!("shaders/chunk.wgsl").into()),
        });

        let entity_shader = device.create_shader_module(&ShaderModuleDescriptor {
            label: Some("entity_shader"),
            source: ShaderSource::Wgsl(include_str!("shaders/entity.wgsl").into()),
        });

        let lighting_shader = device.create_shader_module(&ShaderModuleDescriptor {
            label: Some("lighting_shader"),
            source: ShaderSource::Wgsl(include_str!("shaders/lighting.wgsl").into()),
        });

        let globals_bind_group_layout_entry = BindGroupLayoutEntry {
            binding: 0,
            visibility: ShaderStages::VERTEX_FRAGMENT,
            ty: BindingType::Buffer {
                ty: BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: BufferSize::new(std::mem::size_of::<Globals>() as _),
            },
            count: None,
        };

        let chunk_bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("chunk_bind_group_layout"),
            entries: &[
                globals_bind_group_layout_entry,
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::VERTEX_FRAGMENT,
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
                        min_binding_size: None,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 3,
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

        let entity_bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("entity_bind_group_layout"),
            entries: &[
                globals_bind_group_layout_entry,
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::VERTEX_FRAGMENT,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: true,
                        min_binding_size: BufferSize::new(std::mem::size_of::<EntityLocals>() as _),
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

        let lighting_bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("lighting_bind_group_layout"),
            entries: &[
                globals_bind_group_layout_entry,
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: true,
                        min_binding_size: BufferSize::new(std::mem::size_of::<LightingLocals>() as _),
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

        let chunk_pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("chunk_pipeline_layout"),
            bind_group_layouts: &[&chunk_bind_group_layout],
            push_constant_ranges: &[],
        });

        let entity_pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("entity_pipeline_layout"),
            bind_group_layouts: &[&entity_bind_group_layout],
            push_constant_ranges: &[],
        });

        let lighting_pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("lighting_pipeline_layout"),
            bind_group_layouts: &[&lighting_bind_group_layout],
            push_constant_ranges: &[],
        });

        let chunk_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("chunk_pipeline"),
            layout: Some(&chunk_pipeline_layout),
            vertex: VertexState {
                module: &chunk_shader,
                entry_point: "vs_main",
                buffers: &[Vertex::desc()],
            },
            primitive: PrimitiveState {
                topology: PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: FrontFace::Cw,
                cull_mode: Some(Face::Back),
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
                module: &chunk_shader,
                entry_point: "fs_main",
                targets: &[ColorTargetState {
                    format: TEXTURE_FORMAT,
                    blend: Some(BlendState::ALPHA_BLENDING),
                    write_mask: ColorWrites::ALL,
                }],
            }),
        });

        let entity_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("entity_pipeline"),
            layout: Some(&entity_pipeline_layout),
            vertex: VertexState {
                module: &entity_shader,
                entry_point: "vs_main",
                buffers: &[Vertex::desc()],
            },
            primitive: PrimitiveState {
                topology: PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: FrontFace::Cw,
                cull_mode: Some(Face::Back),
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
                module: &entity_shader,
                entry_point: "fs_main",
                targets: &[ColorTargetState {
                    format: TEXTURE_FORMAT,
                    blend: Some(BlendState::ALPHA_BLENDING),
                    write_mask: ColorWrites::ALL,
                }],
            }),
        });

        let lighting_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("lighting_pipeline"),
            layout: Some(&lighting_pipeline_layout),
            vertex: VertexState {
                module: &lighting_shader,
                entry_point: "vs_main",
                buffers: &[Vertex::desc()],
            },
            primitive: PrimitiveState {
                topology: PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: FrontFace::Cw,
                cull_mode: Some(Face::Back),
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
                module: &lighting_shader,
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
        });

        let vertex_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("vertex_buffer"),
            contents: bytemuck::cast_slice(SQUARE_VERTICES),
            usage: BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("index_buffer"),
            contents: bytemuck::cast_slice(SQUARE_INDICES),
            usage: BufferUsages::INDEX,
        });

        let globals = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("globals"),
            contents: bytemuck::bytes_of(&Globals::new([resolution.width, resolution.height])),
            usage: BufferUsages::COPY_DST | BufferUsages::UNIFORM,
        });

        let chunk_locals = vec![ChunkLocals::new()];
        let chunk_locals = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("chunk_locals"),
            contents: unsafe {
                std::slice::from_raw_parts(
                    chunk_locals.as_ptr() as *const u8,
                    chunk_locals.len() * std::mem::size_of::<ChunkLocals>(),
                )
            },
            usage: BufferUsages::COPY_DST | BufferUsages::UNIFORM,
        });

        let entity_locals = vec![EntityLocals::new()];
        let entity_locals = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("entity_locals"),
            contents: unsafe {
                std::slice::from_raw_parts(
                    entity_locals.as_ptr() as *const u8,
                    entity_locals.len() * std::mem::size_of::<EntityLocals>(),
                )
            },
            usage: BufferUsages::COPY_DST | BufferUsages::UNIFORM,
        });

        let lighting_locals = LightingLocals::new();
        let lighting_locals = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("lighting_locals"),
            contents: unsafe {
                std::slice::from_raw_parts(
                    lighting_locals.as_ptr() as *const u8,
                    lighting_locals.len() * std::mem::size_of::<LightingLocals>(),
                )
            },
            usage: BufferUsages::COPY_DST | BufferUsages::UNIFORM,
        });

        let tile_data_atlas = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("tile_atlas"),
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

        let chunk_bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("drawing_bind_group"),
            layout: &chunk_bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: globals.as_entire_binding(),
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
                    resource: tile_data_atlas.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 3,
                    resource: BindingResource::TextureView(&tile_atlas_view),
                },
            ],
        });

        let (entity_atlas_data, entity_atlas_size) = entity_atlas();
        let entity_atlas = device.create_texture(&TextureDescriptor {
            label: Some("entity_atlas"),
            size: entity_atlas_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TEXTURE_FORMAT,
            usage: TextureUsages::COPY_DST | TextureUsages::TEXTURE_BINDING,
        });

        queue.write_texture(
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

        let entity_bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("entity_bind_group"),
            layout: &entity_bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: globals.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::Buffer(BufferBinding {
                        buffer: &entity_locals,
                        offset: 0,
                        size: BufferSize::new(std::mem::size_of::<EntityLocals>() as _),
                    }),
                },
                BindGroupEntry {
                    binding: 2,
                    resource: BindingResource::TextureView(&entity_atlas_view),
                },
            ],
        });

        let unlit_view = unlit_view(&device, resolution);

        let lighting_bind_group = lighting_bind_group(
            &device,
            &lighting_bind_group_layout,
            &globals,
            &lighting_locals,
            &unlit_view
        );

        Ok(Self {
            surface,
            device,
            queue,
            surface_configuration,
            lighting_bind_group_layout,
            chunk_pipeline,
            entity_pipeline,
            lighting_pipeline,
            vertex_buffer,
            index_buffer,
            globals,
            chunk_locals,
            entity_locals,
            lighting_locals,
            chunk_bind_group,
            entity_bind_group,
            unlit_view,
            lighting_bind_group,
        })
    }

    pub fn render(&self) -> Result<(), Error> {
        let current_texture = match self.surface.get_current_texture() {
            Ok(frame) => frame,
            Err(e) => match e {
                SurfaceError::Timeout => {
                    warn!("Timed out while retrieving surface");
                    return Ok(());
                },
                SurfaceError::Outdated => {
                    warn!("Retrieved surface was outdated");
                    return Ok(());
                },
                SurfaceError::Lost => return Err(Error::SurfaceLost),
                SurfaceError::OutOfMemory => return Err(Error::OutOfMemory),
            },
        };

        let view = current_texture.texture.create_view(&TextureViewDescriptor::default());

        let mut command_encoder = self.device.create_command_encoder(&CommandEncoderDescriptor {
            label: Some("command_encoder")
        });

        {
            let mut render_pass = command_encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some("first_pass"),
                color_attachments: &[RenderPassColorAttachment {
                    view: &self.unlit_view,
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Clear(wgpu::Color { r: 0.05, g: 0.05, b: 0.05, a: 1.0 }),
                        store: true,
                    },
                }],
                depth_stencil_attachment: None,
            });

            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.set_index_buffer(self.index_buffer.slice(..), IndexFormat::Uint16);

            render_pass.set_pipeline(&self.chunk_pipeline);
            render_pass.set_bind_group(0, &self.chunk_bind_group, &[0]);
            render_pass.draw_indexed(0..SQUARE_INDICES.len() as u32, 0, 0..1);

            render_pass.set_pipeline(&self.entity_pipeline);
            render_pass.set_bind_group(0, &self.entity_bind_group, &[0]);
            render_pass.draw_indexed(0..SQUARE_INDICES.len() as u32, 0, 0..1);
        }

        {
            let mut render_pass = command_encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some("second_pass"),
                color_attachments: &[RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Clear(wgpu::Color { r: 0.0, g: 0.0, b: 0.0, a: 1.0 }),
                        store: true,
                    },
                }],
                depth_stencil_attachment: None,
            });

            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.set_index_buffer(self.index_buffer.slice(..), IndexFormat::Uint16);

            render_pass.set_pipeline(&self.lighting_pipeline);
            for i in 0..3 {
                render_pass.set_bind_group(0, &self.lighting_bind_group, &[i * std::mem::size_of::<LightingLocals>() as u32]);
                render_pass.draw_indexed(0..SQUARE_INDICES.len() as u32, 0, 0..1);
            }
        }

        self.queue.submit([command_encoder.finish()]);
        current_texture.present();

        Ok(())
    }

    pub fn resize(&mut self, resolution: PhysicalSize<u32>) {
        if resolution.width == 0 || resolution.height == 0 { return };
        self.surface_configuration.width = resolution.width;
        self.surface_configuration.height = resolution.height;
        self.surface.configure(&self.device, &self.surface_configuration);
        self.queue.write_buffer(&self.globals, 0, bytemuck::bytes_of(&Globals::new([resolution.width, resolution.height])));
        self.unlit_view = unlit_view(&self.device, resolution);
        self.lighting_bind_group = lighting_bind_group(
            &self.device,
            &self.lighting_bind_group_layout,
            &self.globals,
            &self.lighting_locals,
            &self.unlit_view
        );
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

fn unlit_view(device: &Device, resolution: PhysicalSize<u32>) -> TextureView {
    let unlit_texture = device.create_texture(&TextureDescriptor {
        label: Some("unlit_texture"),
        size: Extent3d { width: resolution.width, height: resolution.height, depth_or_array_layers: 1 },
        mip_level_count: 1,
        sample_count: 1,
        dimension: TextureDimension::D2,
        format: TEXTURE_FORMAT,
        usage: TextureUsages::TEXTURE_BINDING | TextureUsages::RENDER_ATTACHMENT,
    });

    unlit_texture.create_view(&TextureViewDescriptor::default())
}

fn lighting_bind_group(
    device: &Device,
    layout: &BindGroupLayout,
    globals: &Buffer,
    locals: &Buffer,
    unlit_view: &TextureView,
) -> BindGroup {
    device.create_bind_group(&BindGroupDescriptor {
        label: Some("lighting_bind_group"),
        layout,
        entries: &[
            BindGroupEntry {
                binding: 0,
                resource: globals.as_entire_binding(),
            },
            BindGroupEntry {
                binding: 1,
                resource: BindingResource::Buffer(BufferBinding {
                    buffer: locals,
                    offset: 0,
                    size: BufferSize::new(std::mem::size_of::<LightingLocals>() as _),
                }),
            },
            BindGroupEntry {
                binding: 2,
                resource: BindingResource::TextureView(unlit_view),
            },
        ],
    })
}
