use std::num::NonZeroU32;

use bytemuck::Pod;
use bytemuck::Zeroable;
use image::ImageDecoder;
use image::gif::GifDecoder;
use log::warn;
use wgpu::Adapter;
use wgpu::Backends;
use wgpu::BindGroup;
use wgpu::BindGroupDescriptor;
use wgpu::BindGroupEntry;
use wgpu::BindGroupLayoutDescriptor;
use wgpu::BindGroupLayoutEntry;
use wgpu::BindingResource;
use wgpu::BindingType;
use wgpu::BlendState;
use wgpu::Buffer;
use wgpu::BufferBindingType;
use wgpu::BufferUsages;
use wgpu::Color;
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

use crate::error::Error;
use crate::map::Map;
use crate::tile::Tile;

const TEXTURE_FORMAT: TextureFormat = TextureFormat::Bgra8UnormSrgb;
const SPRITE_SIZE: u32 = 16;

const SQUARE_VERTICES: &[Vertex] = &[
    Vertex { position: [-1.0,  1.0] },
    Vertex { position: [ 1.0,  1.0] },
    Vertex { position: [-1.0, -1.0] },
    Vertex { position: [ 1.0, -1.0] },
];

const SQUARE_INDICES: &[u16] = &[
    0, 1, 2, 2, 1, 3,
];

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct Vertex {
    position: [f32; 2],
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
                    offset:0,
                    shader_location: 0,
                    format: VertexFormat::Float32x2,
                },
            ],
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Pod, Zeroable)]
struct Uniforms {
    resolution: [f32; 2],
    map_width: i32,
    map_height: i32,
    sprite_size: i32,
}

impl Uniforms {
    fn new(resolution: [f32; 2]) -> Self {
        Self {
            resolution,
            map_width: 7,
            map_height: 8,
            sprite_size: SPRITE_SIZE as i32,
        }
    }
}

pub struct Renderer {
    _instance: Instance,
    surface: Surface,
    _adapter: Adapter,
    device: Device,
    queue: Queue,
    surface_configuration: SurfaceConfiguration,
    uniform_buffer: Buffer,
    map_view: TextureView,
    drawing_bind_group: BindGroup,
    drawing_pipeline: RenderPipeline,
    lighting_bind_group: BindGroup,
    lighting_pipeline: RenderPipeline,
    vertex_buffer: Buffer,
    index_buffer: Buffer,
}

impl Renderer {
    pub async fn new(window: &winit::window::Window) -> Result<Self, Error> {
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

        let uniform_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("uniform_buffer"),
            contents: bytemuck::bytes_of(&Uniforms::new([resolution.width as f32, resolution.height as f32])),
            usage: BufferUsages::COPY_DST | BufferUsages::UNIFORM,
        });

        let (sprite_atlas_data, sprite_atlas_size) = sprite_atlas();
        let sprite_atlas = device.create_texture(&TextureDescriptor {
            label: Some("sprite_atlas"),
            size: sprite_atlas_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TEXTURE_FORMAT,
            usage: TextureUsages::COPY_DST | TextureUsages::TEXTURE_BINDING,
        });

        queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &sprite_atlas,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &sprite_atlas_data,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: NonZeroU32::new(sprite_atlas_size.width * TEXTURE_FORMAT.describe().block_size as u32),
                rows_per_image: None,
            },
            sprite_atlas_size,
        );

        let sprite_atlas_view = sprite_atlas.create_view(&TextureViewDescriptor {
            label: Some("sprite_atlas_view"),
            format: Some(TEXTURE_FORMAT),
            dimension: Some(TextureViewDimension::D2),
            aspect: TextureAspect::All,
            ..Default::default()
        });

        let tile_atlas = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("tile_atlas"),
            contents: bytemuck::cast_slice(&Tile::render_atlas()),
            usage: BufferUsages::STORAGE,
        });

        let map = {
            use Tile::*;

            Map::new(
                vec![
                    [Stone, Stone, Stone, Stone, Stone, Stone, Stone],
                    [Brick, Brick, Brick, Brick, Stone, Stone, Stone],
                    [Stone,  Void,  Void, Stone,  Void,  Void, Stone],
                    [Stone,  Void,  Void,  Void,  Void,  Void, Stone],
                    [Stone, Stone,  Void,  Player,  Void,  Void, Stone],
                    [Stone, Stone,  Void,  Void,  Void,  Void, Stone],
                    [Stone, Stone, Stone,  Void,  Void, Stone, Stone],
                    [Stone, Stone, Stone, Stone, Stone, Stone, Stone],
                ],
                vec![],
            )
        };

        let scene_layout = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("scene_layout"),
            contents: bytemuck::cast_slice(&map.layout()),
            usage: BufferUsages::STORAGE,
        });

        let game_shader = device.create_shader_module(&ShaderModuleDescriptor {
            label: Some("shader"),
            source: ShaderSource::Wgsl(include_str!("shaders/game.wgsl").into()),
        });

        let map_texture = device.create_texture(&TextureDescriptor {
            label: Some("map_texture"),
            size: Extent3d { width: 7 * SPRITE_SIZE, height: 8 * SPRITE_SIZE, depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TEXTURE_FORMAT,
            usage: TextureUsages::TEXTURE_BINDING | TextureUsages::RENDER_ATTACHMENT,
        });

        let map_view = map_texture.create_view(&TextureViewDescriptor::default());

        let drawing_bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("drawing_bind_group_layout"),
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
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
                BindGroupLayoutEntry {
                    binding: 2,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Storage {
                            read_only: true,
                        },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 3,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Storage {
                            read_only: true,
                        },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

        let drawing_bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("drawing_bind_group"),
            layout: &drawing_bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: uniform_buffer.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::TextureView(&sprite_atlas_view),
                },
                BindGroupEntry {
                    binding: 2,
                    resource: tile_atlas.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 3,
                    resource: scene_layout.as_entire_binding(),
                },
            ],
        });

        let drawing_pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("drawing_pipeline_layout"),
            bind_group_layouts: &[&drawing_bind_group_layout],
            push_constant_ranges: &[],
        });

        let drawing_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("drawing_pipeline"),
            layout: Some(&drawing_pipeline_layout),
            vertex: VertexState {
                module: &game_shader,
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
                module: &game_shader,
                entry_point: "draw_map",
                targets: &[ColorTargetState {
                    format: TEXTURE_FORMAT,
                    blend: Some(BlendState::ALPHA_BLENDING),
                    write_mask: ColorWrites::ALL,
                }],
            }),
        });

        let lighting_bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("lighting_bind_group_layout"),
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
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

        let lighting_bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("lighting_bind_group"),
            layout: &lighting_bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::TextureView(&map_view),
                },
            ],
        });

        let lighting_pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("lighting_pipeline_layout"),
            bind_group_layouts: &[&lighting_bind_group_layout],
            push_constant_ranges: &[],
        });

        let lighting_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("lighting_pipeline"),
            layout: Some(&lighting_pipeline_layout),
            vertex: VertexState {
                module: &game_shader,
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
                module: &game_shader,
                entry_point: "draw_light",
                targets: &[ColorTargetState {
                    format: TEXTURE_FORMAT,
                    blend: Some(BlendState::REPLACE),
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

        Ok(Self {
            _instance: instance,
            surface,
            _adapter: adapter,
            device,
            queue,
            surface_configuration,
            uniform_buffer,
            map_view,
            drawing_bind_group,
            drawing_pipeline,
            lighting_bind_group,
            lighting_pipeline,
            vertex_buffer,
            index_buffer,
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
            let mut drawing_pass = command_encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some("render_pass"),
                color_attachments: &[RenderPassColorAttachment {
                    view: &self.map_view,
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Clear(Color { r: 0.01, g: 0.01, b: 0.01, a: 1.0 }),
                        store: true,
                    },
                }],
                depth_stencil_attachment: None,
            });

            drawing_pass.set_pipeline(&self.drawing_pipeline);
            drawing_pass.set_bind_group(0, &self.drawing_bind_group, &[]);
            drawing_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            drawing_pass.set_index_buffer(self.index_buffer.slice(..), IndexFormat::Uint16);
            drawing_pass.draw_indexed(0..SQUARE_INDICES.len() as u32, 0, 0..1);
        }

        {
            let mut lighting_pass = command_encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some("render_pass"),
                color_attachments: &[RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Clear(Color { r: 0.01, g: 0.01, b: 0.01, a: 1.0 }),
                        store: true,
                    },
                }],
                depth_stencil_attachment: None,
            });

            lighting_pass.set_pipeline(&self.lighting_pipeline);
            lighting_pass.set_bind_group(0, &self.lighting_bind_group, &[]);
            lighting_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            lighting_pass.set_index_buffer(self.index_buffer.slice(..), IndexFormat::Uint16);
            lighting_pass.draw_indexed(0..SQUARE_INDICES.len() as u32, 0, 0..1);
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

        self.queue.write_buffer(
            &self.uniform_buffer,
            0,
            bytemuck::bytes_of(&Uniforms::new([resolution.width as f32, resolution.height as f32])),
        );
    }
}

fn sprite_atlas() -> (Vec<u8>, Extent3d) {
    let bytes = include_bytes!("textures/tiles.gif");
    let decoder = GifDecoder::new(&bytes[..]).unwrap();
    let dimensions = decoder.dimensions();

    let mut buf = vec![0; decoder.total_bytes() as usize];
    decoder.read_image(&mut buf).unwrap();

    (buf, Extent3d { width: dimensions.0, height: dimensions.1, depth_or_array_layers: 1 })
}
