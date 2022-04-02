mod chunk_renderer;

use bytemuck::Pod;
use bytemuck::Zeroable;
use rendering_util::RenderingContext;
use wgpu::Buffer;
use wgpu::BufferUsages;
use wgpu::Extent3d;
use wgpu::TextureDescriptor;
use wgpu::TextureDimension;
use wgpu::TextureUsages;
use wgpu::TextureView;
use wgpu::TextureViewDescriptor;
use wgpu::util::BufferInitDescriptor;
use wgpu::util::DeviceExt;
use winit::window::Window;

use crate::chunk::Chunk;
use crate::ecs::Resolution;
use crate::error::Error;

use self::chunk_renderer::ChunkRenderer;

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
struct Globals {
    resolution: [u32; 2],
}

impl Globals {
    fn new(resolution: Resolution) -> Self {
        Self { resolution: [resolution.width, resolution.height] }
    }
}

pub struct Graphics {
    rendering_context: RenderingContext,
    globals: Buffer,
    chunk_renderer: ChunkRenderer,
}

impl Graphics {
    pub async fn new(
        window: &Window,
        resolution: Resolution,
    ) -> Result<Self, Error> {
        let width = resolution.width;
        let height = resolution.height;

        let rc = RenderingContext::new(&window, width, height).await?;

        let globals = rc.device.create_buffer_init(&BufferInitDescriptor {
            label: Some("globals"),
            contents: bytemuck::bytes_of(
                &Globals::new(resolution)
            ),
            usage: BufferUsages::COPY_DST | BufferUsages::UNIFORM,
        });

        let chunk_renderer = ChunkRenderer::new(&rc, &globals, resolution)?;

        Ok(Self {
            rendering_context: rc,
            globals,
            chunk_renderer,
        })
    }

    pub fn write_chunks(&self, chunks: &[Chunk]) {
        self.chunk_renderer.write_chunks(&self.rendering_context, chunks);
    }

    pub fn render(&mut self, resolution: Resolution) -> Result<(), Error> {
        let rc = &self.rendering_context;
        let width = resolution.width;
        let height = resolution.height;

        // Write our globals
        rc.queue.write_buffer(
            &self.globals,
            0,
            bytemuck::bytes_of(&Globals::new(resolution),
        ));

        // Do our rendering
        self.rendering_context.render(width, height, |rc, surface_view| {
            self.chunk_renderer.render(rc, surface_view);
        })?;

        Ok(())
    }
}

fn unlit_view(rc: &RenderingContext, resolution: Resolution) -> TextureView {
    let unlit_texture = rc.device.create_texture(&TextureDescriptor {
        label: Some("graphics::unlit_texture"),
        size: Extent3d {
            width: resolution.width,
            height: resolution.height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: TextureDimension::D2,
        format: rc.surface_format(),
        usage: TextureUsages::TEXTURE_BINDING | TextureUsages::RENDER_ATTACHMENT,
    });

    unlit_texture.create_view(&TextureViewDescriptor::default())
}
