mod camera;
mod chunk;
mod ecs;
mod graphics;
mod entity;
mod error;
mod light;
mod material;
mod tile;
mod time;
mod player;
mod world;

use num_traits::ToPrimitive;
use tracing::info;
use winit::dpi::PhysicalSize;
use winit::event::Event;
use winit::event::WindowEvent;
use winit::event_loop::ControlFlow;
use winit::event_loop::EventLoop;
use winit::window::WindowBuilder;

use crate::ecs::Resolution;
use crate::entity::Entity;
use crate::error::Error;
use crate::graphics::Graphics;
use crate::light::Light;
use crate::tile::Tile;
use crate::time::Time;
use crate::world::World;

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt::init();

    info!("Creating event loop and window");
    let mut resolution = Resolution { width: 1280, height: 720 };
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_inner_size::<PhysicalSize<u32>>(resolution.into())
        .build(&event_loop)?;
    let mut scale_factor = window.scale_factor();

    info!("Creating graphics instance");
    let mut graphics = Graphics::new(&window, resolution).await?;

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

    graphics.write_chunks(&world.chunks);

    let mut time = Time::new();

    info!("Entering event loop");
    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;

        match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::Resized(new_size) => resolution = new_size.into(),
                WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                WindowEvent::ScaleFactorChanged { scale_factor: sf, new_inner_size } => {
                    scale_factor = sf;
                    resolution = (*new_inner_size).into();
                },
                _ => (),
            },
            Event::MainEventsCleared => {
                if let Err(e) = graphics.render(resolution) {
                    tracing::error!("{e}");
                    *control_flow = ControlFlow::Exit;
                }
            }
            _ => (),
        }
    });

    // Code here will never be run
}
