use wgpu::Adapter;
use wgpu::Backends;
use wgpu::Device;
use wgpu::DeviceDescriptor;
use wgpu::Features;
use wgpu::Instance;
use wgpu::Limits;
use wgpu::Queue;
use wgpu::RequestAdapterOptions;
use wgpu::Surface;

use crate::error::Error;

pub struct Renderer {
    instance: Instance,
    surface: Surface,
    adapter: Adapter,
    device: Device,
    queue: Queue,
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

        Ok(Self {
            instance,
            surface,
            adapter,
            device,
            queue,
        })
    }

    fn render(&self) {

    }
}
