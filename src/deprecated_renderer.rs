use std::num::NonZeroU32;

use bytemuck::Pod;
use bytemuck::Zeroable;
use image::ImageDecoder;
use image::gif::GifDecoder;
use itertools::Itertools;
use log::warn;
use wgpu::Adapter;
use wgpu::Backends;
use wgpu::BindGroup;
use wgpu::BindGroupDescriptor;
use wgpu::BindGroupEntry;
use wgpu::BindGroupLayout;
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
use wgpu::PipelineLayout;
use wgpu::PipelineLayoutDescriptor;
use wgpu::PresentMode;
use wgpu::Queue;
use wgpu::RequestAdapterOptions;
use wgpu::ShaderModule;
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
use crate::map::Map;
use crate::material::Material;

const TILE_SIZE: u32 = 16;

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Pod, Zeroable)]
struct Parameters {
    padded_screen_width: u32,
    map_width: u32,
    map_height: u32,
    atlas_byte_width: u32,
    atlas_size: u32,
    tile_size: u32,
}

pub struct Renderer {
    _instance: Instance,
    surface: Surface,
    _adapter: Adapter,
    device: Device,
    queue: Queue,
    size: PhysicalSize<u32>,
    virtual_size: PhysicalSize<u32>,
    surface_configuration: SurfaceConfiguration,
    bind_group_layout: BindGroupLayout,
    map_width: u32,
    map_height: u32,
    atlas_byte_width: u32,
    atlas_size: u32,
    uniform_buffer: Buffer,
    map_buffer: Buffer,
    material_atlas: Buffer,
    texture_atlas: Buffer,
    pipeline_layout: PipelineLayout,
    shader: ShaderModule,
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
                        ty: BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 2,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 3,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 4,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

        let map_width = 0;
        let map_height = 0;
        let (texture_atlas_data, atlas_byte_width, atlas_size) = texture_atlas();

        let uniform_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("uniform_buffer"),
            contents: bytemuck::bytes_of(&Parameters {
                padded_screen_width: virtual_size.width,
                map_width,
                map_height,
                atlas_byte_width,
                atlas_size,
                tile_size: TILE_SIZE,
            }),
            usage: BufferUsages::COPY_DST | BufferUsages::UNIFORM,
        });

        let map_buffer = create_map_buffer(&device, None);

        let material_atlas = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("material_atlas"),
            contents: bytemuck::cast_slice(&Material::material_atlas()),
            usage: BufferUsages::STORAGE,
        });

        let texture_atlas = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("texture_atlas"),
            contents: bytemuck::cast_slice(&texture_atlas_data),
            usage: BufferUsages::STORAGE,
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

        let display_buffer = create_display_buffer(&device, virtual_size);
        let (bind_group, pipeline) = create_pipeline(
            &device,
            &bind_group_layout,
            &uniform_buffer,
            &map_buffer,
            &material_atlas,
            &texture_atlas,
            &display_buffer,
            &pipeline_layout,
            &shader,
        );

        Ok(Self {
            _instance: instance,
            surface,
            _adapter: adapter,
            device,
            queue,
            size,
            virtual_size,
            surface_configuration,
            bind_group_layout,
            map_width,
            map_height,
            atlas_byte_width,
            atlas_size,
            uniform_buffer,
            map_buffer,
            material_atlas,
            texture_atlas,
            pipeline_layout,
            shader,
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
            pass.dispatch(self.virtual_size.width / 16, self.virtual_size.height / 16, 1);
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

        self.display_buffer = create_display_buffer(&self.device, self.virtual_size);
        let (bind_group, pipeline) = create_pipeline(
            &self.device,
            &self.bind_group_layout,
            &self.uniform_buffer,
            &self.map_buffer,
            &self.material_atlas,
            &self.texture_atlas,
            &self.display_buffer,
            &self.pipeline_layout,
            &self.shader,
        );

        self.bind_group = bind_group;
        self.pipeline = pipeline;

        self.queue.write_buffer(&self.uniform_buffer, 0, bytemuck::bytes_of(&Parameters {
            padded_screen_width: self.virtual_size.width,
            map_width: self.map_width,
            map_height: self.map_height,
            atlas_byte_width: self.atlas_byte_width,
            atlas_size: self.atlas_size,
            tile_size: TILE_SIZE,
        }));
    }

    pub fn change_map(&mut self, map: &Map) {
        self.map_width = map.width;
        self.map_height = map.layout().len() as u32 / map.width;
        self.map_buffer = create_map_buffer(&self.device, Some(map));

        let (bind_group, pipeline) = create_pipeline(
            &self.device,
            &self.bind_group_layout,
            &self.uniform_buffer,
            &self.map_buffer,
            &self.material_atlas,
            &self.texture_atlas,
            &self.display_buffer,
            &self.pipeline_layout,
            &self.shader
        );

        self.bind_group = bind_group;
        self.pipeline = pipeline;

        self.queue.write_buffer(&self.uniform_buffer, 0, bytemuck::bytes_of(&Parameters {
            padded_screen_width: self.virtual_size.width,
            map_width: self.map_width,
            map_height: self.map_height,
            atlas_byte_width: self.atlas_byte_width,
            atlas_size: self.atlas_size,
            tile_size: TILE_SIZE,
        }));
    }
}

fn virtual_size(size: PhysicalSize<u32>) -> PhysicalSize<u32> {
    PhysicalSize::new(
        size.width + 64 - (size.width % 64),
        size.height + 16 - (size.height % 16),
    )
}

fn create_map_buffer(device: &Device, map: Option<&Map>) -> Buffer {
    let contents = match map {
        Some(map) => map.layout(),
        None => &[0],
    };

    device.create_buffer_init(&BufferInitDescriptor {
        label: Some("map_buffer"),
        contents: bytemuck::cast_slice(contents),
        usage: BufferUsages::COPY_DST | BufferUsages::STORAGE,
    })
}

fn create_display_buffer(device: &Device, virtual_size: PhysicalSize<u32>) -> Buffer {
    device.create_buffer(&BufferDescriptor {
        label: Some("display_buffer"),
        size: (virtual_size.width * virtual_size.height * 4) as u64,
        usage: BufferUsages::COPY_SRC | BufferUsages::STORAGE,
        mapped_at_creation: false,
    })
}

fn create_pipeline(
    device: &Device,
    bind_group_layout: &BindGroupLayout,
    uniform_buffer: &Buffer,
    map_buffer: &Buffer,
    material_atlas: &Buffer,
    texture_atlas: &Buffer,
    display_buffer: &Buffer,
    pipeline_layout: &PipelineLayout,
    shader: &ShaderModule,
) -> (BindGroup, ComputePipeline) {
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
                resource: map_buffer.as_entire_binding(),
            },
            BindGroupEntry {
                binding: 2,
                resource: material_atlas.as_entire_binding(),
            },
            BindGroupEntry {
                binding: 3,
                resource: texture_atlas.as_entire_binding(),
            },
            BindGroupEntry {
                binding: 4,
                resource: display_buffer.as_entire_binding(),
            },
        ],
    });

    let pipeline = device.create_compute_pipeline(&ComputePipelineDescriptor {
        label: Some("pipeline"),
        layout: Some(&pipeline_layout),
        module: shader,
        entry_point: "main",
    });

    (bind_group, pipeline)
}


fn texture_atlas() -> (Vec<u32>, u32, u32) {
    let bytes = include_bytes!("textures/tiles.gif");
    let decoder = GifDecoder::new(&bytes[..]).unwrap();

    let mut bytes = vec![0; decoder.total_bytes() as usize];
    let dimensions = decoder.dimensions();
    decoder.read_image(&mut bytes).unwrap();

    let mut buf = vec![];
    for chunk in &bytes.iter().step_by(4).chunks(TILE_SIZE as usize) {
        let mut row = 0;
        for (i, byte) in chunk.into_iter().enumerate() {
            let color = match byte {
                x if x > &0 => 0x3u32,
                _ => 0,
            };
            row |= color << (TILE_SIZE - 1 - i as u32) * 2
        }

        buf.push(row);
    }

    println!("{:?}", buf);

    let len = buf.len().try_into().unwrap();
    (buf, dimensions.0 / 16, len)
}
