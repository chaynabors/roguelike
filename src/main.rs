mod camera;
mod color;
mod error;
mod light;
mod map;
mod tile;
mod player;
mod renderer;
mod state;
mod vector2;

use std::io::Write;

use chrono::Local;
use log::LevelFilter;
use log::error;
use log::info;
use winit::dpi::PhysicalSize;
use winit::event::Event;
use winit::event::WindowEvent;
use winit::event_loop::ControlFlow;
use winit::event_loop::EventLoop;
use winit::window::WindowBuilder;

use crate::error::Error;
use crate::renderer::Renderer;

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
    let event_loop = EventLoop::new();
    let window = match WindowBuilder::new()
        .with_min_inner_size(PhysicalSize::new(320, 200))
        .build(&event_loop)
    {
        Ok(window) => window,
        Err(_) => return Err(Error::WindowCreationFailed),
    };
    info!("Created event loop and window");

    info!("Creating renderer");
    let mut renderer = match Renderer::new(&window).await {
        Ok(renderer) => renderer,
        Err(e) => return Err(e),
    };
    info!("Created renderer");

    info!("Entering event loop");
    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;

        match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::Resized(size) => renderer.resize(size),
                WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                WindowEvent::ScaleFactorChanged { new_inner_size, .. } => renderer.resize(*new_inner_size), 
                _ => (),
            },
            Event::RedrawRequested(_) => if let Err(e) = renderer.render() { match e {
                Error::SurfaceLost => renderer.resize(window.inner_size()),
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
