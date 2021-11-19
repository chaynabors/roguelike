use std::num::NonZeroU32;

use log::warn;
use wgpu::Adapter;
use wgpu::Backends;
use wgpu::BindGroup;
use wgpu::BindGroupDescriptor;
use wgpu::BindGroupEntry;
use wgpu::BindGroupLayoutDescriptor;
use wgpu::BindGroupLayoutEntry;
use wgpu::BindingType;
use wgpu::Buffer;
use wgpu::BufferBindingType;
use wgpu::BufferDescriptor;
use wgpu::BufferUsages;
use wgpu::CommandEncoderDescriptor;
use wgpu::ComputePassDescriptor;
use wgpu::ComputePipeline;
use wgpu::ComputePipelineDescriptor;
use wgpu::Device;
use wgpu::DeviceDescriptor;
use wgpu::Extent3d;
use wgpu::Features;
use wgpu::ImageCopyBuffer;
use wgpu::ImageCopyTexture;
use wgpu::ImageDataLayout;
use wgpu::Instance;
use wgpu::Limits;
use wgpu::Origin3d;
use wgpu::PipelineLayoutDescriptor;
use wgpu::PresentMode;
use wgpu::Queue;
use wgpu::RequestAdapterOptions;
use wgpu::ShaderModuleDescriptor;
use wgpu::ShaderSource;
use wgpu::ShaderStages;
use wgpu::Surface;
use wgpu::SurfaceConfiguration;
use wgpu::SurfaceError;
use wgpu::TextureAspect;
use wgpu::TextureUsages;
use wgpu::util::BufferInitDescriptor;
use wgpu::util::DeviceExt;
use winit::dpi::PhysicalSize;

use crate::error::Error;

const TILE_SIZE: u32 = 16;

pub struct Renderer {
    _instance: Instance,
    surface: Surface,
    _adapter: Adapter,
    device: Device,
    queue: Queue,
    size: PhysicalSize<u32>,
    virtual_size: PhysicalSize<u32>,
    surface_configuration: SurfaceConfiguration,
    display_buffer: Buffer,
    bind_group: BindGroup,
    pipeline: ComputePipeline,
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

        let size = window.inner_size();
        let virtual_size = virtual_size(size);
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

        let (display_buffer, bind_group, pipeline) = create_pipeline(&device, virtual_size);

        Ok(Self {
            _instance: instance,
            surface,
            _adapter: adapter,
            device,
            queue,
            size,
            virtual_size,
            surface_configuration,
            display_buffer,
            bind_group,
            pipeline,
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

        let mut command_encoder = self.device.create_command_encoder(&CommandEncoderDescriptor {
            label: Some("command_encoder")
        });

        {
            let mut pass = command_encoder.begin_compute_pass(&ComputePassDescriptor {
                label: Some("pass"),
            });

            pass.set_pipeline(&self.pipeline);
            pass.set_bind_group(0, &self.bind_group, &[]);
            pass.dispatch(self.virtual_size.width / TILE_SIZE, self.virtual_size.height / TILE_SIZE, 1);
        }

        command_encoder.copy_buffer_to_texture(
            ImageCopyBuffer {
                buffer: &self.display_buffer,
                layout: ImageDataLayout {
                    offset: 0,
                    bytes_per_row: NonZeroU32::new(self.virtual_size.width * 4),
                    rows_per_image: None,
                },
            },
            ImageCopyTexture {
                texture: &current_texture.texture,
                mip_level: 0,
                origin: Origin3d { x: 0, y: 0, z: 0 },
                aspect: TextureAspect::All,
            },
            Extent3d { width: self.size.width, height: self.size.height, depth_or_array_layers: 1 },
        );

        self.queue.submit([command_encoder.finish()]);
        current_texture.present();

        Ok(())
    }

    pub fn resize(&mut self, size: PhysicalSize<u32>) {
        if size.width == 0 || size.height == 0 { return };

        self.size = size;
        self.virtual_size = virtual_size(size);
        self.surface_configuration.width = size.width;
        self.surface_configuration.height = size.height;
        self.surface.configure(&self.device, &self.surface_configuration);

        let (display_buffer, bind_group, pipeline) = create_pipeline(&self.device, self.virtual_size);
        self.display_buffer = display_buffer;
        self.bind_group = bind_group;
        self.pipeline = pipeline;
    }
}

fn virtual_size(size: PhysicalSize<u32>) -> PhysicalSize<u32> {
    PhysicalSize::new(
        size.width + (TILE_SIZE * 4) - (size.width % (TILE_SIZE * 4)),
        size.height + TILE_SIZE - (size.height % TILE_SIZE),
    )
}

fn create_pipeline(device: &Device, virtual_size: PhysicalSize<u32>) -> (Buffer, BindGroup, ComputePipeline) {
    let bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
        label: Some("bind_group_layout"),
        entries: &[
            BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStages::COMPUTE,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            BindGroupLayoutEntry {
                binding: 1,
                visibility: ShaderStages::COMPUTE,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Storage {
                        read_only: false,
                    },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
        ],
    });

    let uniform_buffer = device.create_buffer_init(&BufferInitDescriptor {
        label: Some("uniform_buffer"),
        contents: bytemuck::cast_slice(&[
            TILE_SIZE,
            virtual_size.width,
            virtual_size.height,
        ]),
        usage: BufferUsages::UNIFORM,
    });

    let display_buffer = device.create_buffer(&BufferDescriptor {
        label: Some("display_buffer"),
        size: (virtual_size.width * virtual_size.height * 4) as u64,
        usage: BufferUsages::COPY_SRC | BufferUsages::STORAGE,
        mapped_at_creation: false,
    });

    let bind_group = device.create_bind_group(&BindGroupDescriptor {
        label: Some("bind_group"),
        layout: &bind_group_layout,
        entries: &[
            BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            },
            BindGroupEntry {
                binding: 1,
                resource: display_buffer.as_entire_binding(),
            },
        ],
    });

    let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
        label: Some("pipeline_layout"),
        bind_group_layouts: &[&bind_group_layout],
        push_constant_ranges: &[],
    });

    let shader = device.create_shader_module(&ShaderModuleDescriptor {
        label: Some("shader"),
        source: ShaderSource::Wgsl(include_str!("shaders/game_view.wgsl").into()),
    });

    let pipeline = device.create_compute_pipeline(&ComputePipelineDescriptor {
        label: Some("pipeline"),
        layout: Some(&pipeline_layout),
        module: &shader,
        entry_point: "main",
    });

    (display_buffer, bind_group, pipeline)
}
