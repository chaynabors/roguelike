use bytemuck::Pod;
use bytemuck::Zeroable;
use log::warn;
use wgpu::Adapter;
use wgpu::AddressMode;
use wgpu::Backends;
use wgpu::BindGroup;
use wgpu::BindGroupDescriptor;
use wgpu::BindGroupEntry;
use wgpu::BindGroupLayoutDescriptor;
use wgpu::BindGroupLayoutEntry;
use wgpu::BindingResource;
use wgpu::BlendState;
use wgpu::Buffer;
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
use wgpu::FilterMode;
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
use wgpu::SamplerDescriptor;
use wgpu::ShaderModuleDescriptor;
use wgpu::ShaderSource;
use wgpu::Surface;
use wgpu::SurfaceConfiguration;
use wgpu::SurfaceError;
use wgpu::TextureAspect;
use wgpu::TextureDescriptor;
use wgpu::TextureDimension;
use wgpu::TextureFormat;
use wgpu::TextureUsages;
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

const TEXTURE_FORMAT: TextureFormat = TextureFormat::Bgra8UnormSrgb;

const SCALE: f32 = 0.5;

const SQUARE_VERTICES: &[Vertex] = &[
    Vertex { position: [-1.0 * SCALE,  1.0 * SCALE, 0.0], tex_coords: [0.0, 0.0] },
    Vertex { position: [ 1.0 * SCALE,  1.0 * SCALE, 0.0], tex_coords: [1.0, 0.0] },
    Vertex { position: [-1.0 * SCALE, -1.0 * SCALE, 0.0], tex_coords: [0.0, 1.0] },
    Vertex { position: [ 1.0 * SCALE, -1.0 * SCALE, 0.0], tex_coords: [1.0, 1.0] },
];

const SQUARE_INDICES: &[u16] = &[
    2, 1, 0,
    1, 2, 3,
];

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct Vertex {
    position: [f32; 3],
    tex_coords: [f32; 2],
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
                    format: VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2,
                }
            ],
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
    bind_group: BindGroup,
    render_pipeline: RenderPipeline,
    vertex_buffer: Buffer,
    index_buffer: Buffer,
}

impl Renderer {
    pub async fn new(window: &winit::window::Window) -> Result<Self, Error> {
        let instance = Instance::new(Backends::all());
        
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

        let size = window.inner_size();
        let surface_configuration = SurfaceConfiguration {
            usage: TextureUsages::COPY_DST | TextureUsages::RENDER_ATTACHMENT,
            format: match surface.get_preferred_format(&adapter) {
                Some(format) => format,
                None => return Err(Error::IncompatibleSurface),
            },
            width: size.width,
            height: size.height,
            present_mode: PresentMode::Fifo,
        };
        surface.configure(&device, &surface_configuration);

        let bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("bind_group_layout"),
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: wgpu::TextureSampleType::Float { filterable: false },
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler {
                        comparison: false,
                        filtering: false,
                    },
                    count: None,
                },
            ],
        });

        let texture_size = Extent3d { width: 2, height: 2, depth_or_array_layers: 1 };
        let render_texture = device.create_texture(&TextureDescriptor {
            label: Some("render_texture"),
            size: Extent3d { width: texture_size.width, height: texture_size.height, depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TEXTURE_FORMAT,
            usage: TextureUsages::COPY_DST | TextureUsages::TEXTURE_BINDING,
        });

        queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &render_texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            bytemuck::cast_slice(&[0, u32::MAX >> 16, u32::MAX >> 24, u32::MAX]),
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: std::num::NonZeroU32::new(4 * 2),
                rows_per_image: std::num::NonZeroU32::new(2),
            },
            texture_size,
        );

        let render_view = render_texture.create_view(&TextureViewDescriptor {
            label: Some("render_view"),
            format: Some(TEXTURE_FORMAT),
            dimension: Some(TextureViewDimension::D2),
            aspect: TextureAspect::All,
            ..Default::default()
        });

        let render_sampler = device.create_sampler(&SamplerDescriptor {
            label: Some("render_sampler"),
            address_mode_u: AddressMode::ClampToEdge,
            address_mode_v: AddressMode::ClampToEdge,
            address_mode_w: AddressMode::ClampToEdge,
            mag_filter: FilterMode::Nearest,
            min_filter: FilterMode::Nearest,
            mipmap_filter: FilterMode::Nearest,
            ..Default::default()
        });

        let bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("bind_group"),
            layout: &bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::TextureView(&render_view),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::Sampler(&render_sampler),
                },
            ],
        });

        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("pipeline_layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let render_view_shader = device.create_shader_module(&ShaderModuleDescriptor {
            label: Some("shader"),
            source: ShaderSource::Wgsl(include_str!("shaders/game_view.wgsl").into()),
        });

        let render_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("render_pipeline"),
            layout: Some(&pipeline_layout),
            vertex: VertexState {
                module: &render_view_shader,
                entry_point: "vs_main",
                buffers: &[Vertex::desc()],
            },
            primitive: PrimitiveState {
                topology: PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: FrontFace::Ccw,
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
                module: &render_view_shader,
                entry_point: "fs_main",
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
            bind_group,
            render_pipeline,
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
            let mut render_pass = command_encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some("render_pass"),
                color_attachments: &[RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Clear(Color { r: 0.1, g: 0., b: 0.1, a: 1. }),
                        store: true,
                    },
                }],
                depth_stencil_attachment: None,
            });

            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, &self.bind_group, &[]);
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.set_index_buffer(self.index_buffer.slice(..), IndexFormat::Uint16);
            render_pass.draw_indexed(0..SQUARE_INDICES.len() as u32, 0, 0..1);
        }

        self.queue.submit([command_encoder.finish()]);
        current_texture.present();

        Ok(())
    }

    pub fn resize(&mut self, size: PhysicalSize<u32>) {
        if size.width == 0 || size.height == 0 { return };
        self.surface_configuration.width = size.width;
        self.surface_configuration.height = size.height;
        self.surface.configure(&self.device, &self.surface_configuration);
    }
}
