use bytemuck::Pod;
use bytemuck::Zeroable;
use log::warn;
use wgpu::Backends;
use wgpu::BindGroupLayoutEntry;
use wgpu::BindingType;
use wgpu::Buffer;
use wgpu::BufferBindingType;
use wgpu::BufferSize;
use wgpu::BufferUsages;
use wgpu::CommandEncoder;
use wgpu::CommandEncoderDescriptor;
use wgpu::Device;
use wgpu::DeviceDescriptor;
use wgpu::Features;
use wgpu::Instance;
use wgpu::Limits;
use wgpu::PowerPreference;
use wgpu::PresentMode;
use wgpu::Queue;
use wgpu::RequestAdapterOptions;
use wgpu::ShaderStages;
use wgpu::Surface;
use wgpu::SurfaceConfiguration;
use wgpu::SurfaceError;
use wgpu::TextureUsages;
use wgpu::TextureView;
use wgpu::TextureViewDescriptor;
use wgpu::util::BufferInitDescriptor;
use wgpu::util::DeviceExt;
use winit::window::Window;

use crate::error::Error;

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
struct Globals {
    resolution: [u32; 2],
}

impl Globals {
    fn new(resolution: [u32; 2]) -> Self {
        Self { resolution }
    }
}

pub struct RenderingContext {
    pub surface: Surface,
    pub device: Device,
    pub queue: Queue,
    surface_configuration: SurfaceConfiguration,
    pub globals: Buffer,
}

impl RenderingContext {
    pub async fn new(window: &Window, resolution: [u32; 2]) -> Result<Self, Error> {
        let instance = Instance::new(Backends::PRIMARY);

        let surface = unsafe { instance.create_surface(window) };

        let adapter = match instance.request_adapter(&RequestAdapterOptions {
            power_preference: PowerPreference::HighPerformance,
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

        let surface_configuration = SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT,
            format: match surface.get_preferred_format(&adapter) {
                Some(format) => format,
                None => return Err(Error::IncompatibleSurface),
            },
            width: resolution[0],
            height: resolution[1],
            present_mode: PresentMode::Fifo,
        };
        surface.configure(&device, &surface_configuration);

        let globals = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("chunk_globals"),
            contents: bytemuck::bytes_of(&Globals::new(resolution)),
            usage: BufferUsages::COPY_DST | BufferUsages::UNIFORM,
        });

        Ok(Self {
            surface,
            device,
            queue,
            surface_configuration,
            globals,
        })
    }

    pub fn resize(&mut self, resolution: [u32; 2]) {
        self.surface_configuration.width = resolution[0];
        self.surface_configuration.height = resolution[1];
        self.surface.configure(&self.device, &self.surface_configuration);
        self.queue.write_buffer(&self.globals, 0, bytemuck::bytes_of(&Globals::new(resolution)));
    }

    pub fn render<F>(&mut self, mut render_function: F) -> Result<(), Error> where F: FnMut(&TextureView, &mut CommandEncoder) {
        let current_surface = match self.surface.get_current_texture() {
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

        let surface_view = current_surface.texture.create_view(&TextureViewDescriptor::default());

        let mut command_encoder = self.device.create_command_encoder(&CommandEncoderDescriptor {
            label: Some("command_encoder")
        });

        render_function(&surface_view, &mut command_encoder);
        self.queue.submit([command_encoder.finish()]);
        current_surface.present();

        Ok(())
    }

    pub fn globals_bind_group_layout_entry(&self) -> BindGroupLayoutEntry {
        BindGroupLayoutEntry {
            binding: 0,
            visibility: ShaderStages::VERTEX_FRAGMENT,
            ty: BindingType::Buffer {
                ty: BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: BufferSize::new(std::mem::size_of::<Globals>() as _),
            },
            count: None,
        }
    }
}
