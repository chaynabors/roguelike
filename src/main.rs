mod camera;
mod chunk;
mod game_renderer;
mod entity;
mod error;
mod light;
mod tile;
mod player;
mod rendering_context;
mod world;

use std::io::Write;
use std::time::Duration;
use std::time::Instant;

use chrono::Local;
use log::LevelFilter;
use log::error;
use log::info;
use log::warn;
use num_traits::ToPrimitive;
use winit::dpi::PhysicalSize;
use winit::event::Event;
use winit::event::WindowEvent;
use winit::event_loop::ControlFlow;
use winit::event_loop::EventLoop;
use winit::window::WindowBuilder;

use crate::entity::Entity;
use crate::game_renderer::GameRenderer;
use crate::error::Error;
use crate::light::Light;
use crate::rendering_context::RenderingContext;
use crate::tile::Tile;
use crate::world::World;

#[tokio::main]
async fn main() -> Result<(), crate::error::Error> {
    env_logger::builder()
        .format(|buf, record| {
            writeln!(
                buf,
                "{} {}: {}",
                record.level(),
                Local::now().format("%H:%M:%S"),
                record.args()
            )
        })
        .filter_level(LevelFilter::Info)
        .init();

    info!("Creating event loop and window");
    let resolution = [256; 2];
    let event_loop = EventLoop::new();
    let window = match WindowBuilder::new()
        .with_inner_size(PhysicalSize::new(resolution[0], resolution[0]))
        .with_min_inner_size(PhysicalSize::new(128, 128))
        .build(&event_loop)
    {
        Ok(window) => window,
        Err(_) => return Err(Error::WindowCreationFailed),
    };
    info!("Created event loop and window");

    info!("Creating rendering context");
    let mut rendering_context = match RenderingContext::new(&window, resolution).await {
        Ok(renderer) => renderer,
        Err(e) => return Err(e),
    };
    info!("Created renderer");

    info!("Creating test world");
    let world = World {
        name: "World".to_string(),
        seed: 0,
        chunks: vec![
            [
                [
                    ToPrimitive::to_u8(&Tile::Wall).unwrap(),
                    ToPrimitive::to_u8(&Tile::Wall).unwrap(),
                    ToPrimitive::to_u8(&Tile::Wall).unwrap(),
                    ToPrimitive::to_u8(&Tile::Wall).unwrap(),
                    ToPrimitive::to_u8(&Tile::Wall).unwrap(),
                    ToPrimitive::to_u8(&Tile::Wall).unwrap(),
                    ToPrimitive::to_u8(&Tile::Wall).unwrap(),
                    ToPrimitive::to_u8(&Tile::Planks).unwrap(),
                    ToPrimitive::to_u8(&Tile::Wall).unwrap(),
                    ToPrimitive::to_u8(&Tile::Wall).unwrap(),
                    ToPrimitive::to_u8(&Tile::Planks).unwrap(),
                    ToPrimitive::to_u8(&Tile::Wall).unwrap(),
                    ToPrimitive::to_u8(&Tile::Wall).unwrap(),
                    ToPrimitive::to_u8(&Tile::Wall).unwrap(),
                    ToPrimitive::to_u8(&Tile::Wall).unwrap(),
                    ToPrimitive::to_u8(&Tile::Wall).unwrap(),
                ],
                [ToPrimitive::to_u8(&Tile::Wall).unwrap(); 16],
                [ToPrimitive::to_u8(&Tile::Wall).unwrap(); 16],
                [ToPrimitive::to_u8(&Tile::Wall).unwrap(); 16],
                [ToPrimitive::to_u8(&Tile::Wall).unwrap(); 16],
                [ToPrimitive::to_u8(&Tile::Wall).unwrap(); 16],
                [ToPrimitive::to_u8(&Tile::Wall).unwrap(); 16],
                [ToPrimitive::to_u8(&Tile::Planks).unwrap(); 16],
                [ToPrimitive::to_u8(&Tile::Planks).unwrap(); 16],
                [ToPrimitive::to_u8(&Tile::Planks).unwrap(); 16],
                [ToPrimitive::to_u8(&Tile::Wall).unwrap(); 16],
                [ToPrimitive::to_u8(&Tile::Planks).unwrap(); 16],
                [ToPrimitive::to_u8(&Tile::Wall).unwrap(); 16],
                [ToPrimitive::to_u8(&Tile::Wall).unwrap(); 16],
                [ToPrimitive::to_u8(&Tile::Wall).unwrap(); 16],
                [ToPrimitive::to_u8(&Tile::Wall).unwrap(); 16],
            ],
            [[ToPrimitive::to_u8(&Tile::Void).unwrap(); 16]; 16],
            [[ToPrimitive::to_u8(&Tile::Wall).unwrap(); 16]; 16],
            [[ToPrimitive::to_u8(&Tile::Wall).unwrap(); 16]; 16],
            [[ToPrimitive::to_u8(&Tile::Void).unwrap(); 16]; 16],
            [[ToPrimitive::to_u8(&Tile::Wall).unwrap(); 16]; 16],
            [[ToPrimitive::to_u8(&Tile::Wall).unwrap(); 16]; 16],
            [[ToPrimitive::to_u8(&Tile::Wall).unwrap(); 16]; 16],
            [[ToPrimitive::to_u8(&Tile::Wall).unwrap(); 16]; 16],
            [[ToPrimitive::to_u8(&Tile::Wall).unwrap(); 16]; 16],
            [
                [
                    ToPrimitive::to_u8(&Tile::Wall).unwrap(),
                    ToPrimitive::to_u8(&Tile::Wall).unwrap(),
                    ToPrimitive::to_u8(&Tile::Wall).unwrap(),
                    ToPrimitive::to_u8(&Tile::Wall).unwrap(),
                    ToPrimitive::to_u8(&Tile::Wall).unwrap(),
                    ToPrimitive::to_u8(&Tile::Wall).unwrap(),
                    ToPrimitive::to_u8(&Tile::Wall).unwrap(),
                    ToPrimitive::to_u8(&Tile::Planks).unwrap(),
                    ToPrimitive::to_u8(&Tile::Wall).unwrap(),
                    ToPrimitive::to_u8(&Tile::Wall).unwrap(),
                    ToPrimitive::to_u8(&Tile::Wall).unwrap(),
                    ToPrimitive::to_u8(&Tile::Wall).unwrap(),
                    ToPrimitive::to_u8(&Tile::Wall).unwrap(),
                    ToPrimitive::to_u8(&Tile::Wall).unwrap(),
                    ToPrimitive::to_u8(&Tile::Wall).unwrap(),
                    ToPrimitive::to_u8(&Tile::Wall).unwrap(),
                ],
                [ToPrimitive::to_u8(&Tile::Wall).unwrap(); 16],
                [ToPrimitive::to_u8(&Tile::Wall).unwrap(); 16],
                [ToPrimitive::to_u8(&Tile::Wall).unwrap(); 16],
                [ToPrimitive::to_u8(&Tile::Wall).unwrap(); 16],
                [ToPrimitive::to_u8(&Tile::Wall).unwrap(); 16],
                [ToPrimitive::to_u8(&Tile::Wall).unwrap(); 16],
                [ToPrimitive::to_u8(&Tile::Planks).unwrap(); 16],
                [ToPrimitive::to_u8(&Tile::Planks).unwrap(); 16],
                [ToPrimitive::to_u8(&Tile::Planks).unwrap(); 16],
                [ToPrimitive::to_u8(&Tile::Wall).unwrap(); 16],
                [ToPrimitive::to_u8(&Tile::Wall).unwrap(); 16],
                [ToPrimitive::to_u8(&Tile::Wall).unwrap(); 16],
                [ToPrimitive::to_u8(&Tile::Wall).unwrap(); 16],
                [ToPrimitive::to_u8(&Tile::Wall).unwrap(); 16],
                [ToPrimitive::to_u8(&Tile::Wall).unwrap(); 16],
            ],
            [[ToPrimitive::to_u8(&Tile::Void).unwrap(); 16]; 16],
            [[ToPrimitive::to_u8(&Tile::Wall).unwrap(); 16]; 16],
            [[ToPrimitive::to_u8(&Tile::Wall).unwrap(); 16]; 16],
            [[ToPrimitive::to_u8(&Tile::Wall).unwrap(); 16]; 16],
            [[ToPrimitive::to_u8(&Tile::Wall).unwrap(); 16]; 16],
        ],
        entities: vec![
            Entity::new([0.0, 0.0], [0, 0], [1, 1], u32::MAX, None),
        ],
        lights: vec![
            Light::new([-0.5, 0.5], [255, 0, 0], 255),
            Light::new([1.0, 0.0], [0, 255, 0], 255),
            Light::new([0.0, -1.0], [0, 0, 255], 255),
            Light::new([0.5, -0.5], [255, 255, 255], 255),
        ],
    };
    info!("Created test world");

    info!("Creating game renderer");
    let mut game_renderer = GameRenderer::new(&rendering_context, resolution);
    game_renderer.write_chunks(&rendering_context, &world.chunks);
    game_renderer.write_entities(&rendering_context, &world.entities);
    game_renderer.write_lights(&rendering_context, &world.lights);
    info!("Created game renderer");

    info!("Entering event loop");
    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::WaitUntil(Instant::now() + Duration::from_millis(8));

        match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::Resized(resolution) => {
                    let resolution = [resolution.width, resolution.height];
                    rendering_context.resize(resolution);
                    game_renderer.resize(&rendering_context, resolution);
                },
                WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                    let resolution = [new_inner_size.width, new_inner_size.height];
                    rendering_context.resize(resolution);
                    game_renderer.resize(&rendering_context, resolution);
                },
                _ => (),
            },
            Event::RedrawRequested(_) => if let Err(e) = rendering_context.render(|surface_view, command_encoder| {
                game_renderer.render(surface_view, command_encoder);
            }) { match e {
                Error::SurfaceLost => {
                    warn!("Surface lost, resizing surface");
                    let resolution = window.inner_size();
                    rendering_context.resize([resolution.width, resolution.height]);
                },
                Error::OutOfMemory => {
                    error!("Application ran out of memory");
                    *control_flow = ControlFlow::Exit;
                },
                _ => panic!("{:?}", e),
            }},
            _ => (),
        }
    });

    // Code here will never be run
}
