mod camera;
mod color;
mod error;
mod light;
mod map;
mod material;
mod player;
mod renderer;
mod state;
mod vector2;

use std::io::Write;

use chrono::Local;
use log::LevelFilter;
use log::info;
use winit::event::Event;
use winit::event::WindowEvent;
use winit::event_loop::ControlFlow;
use winit::event_loop::EventLoop;
use winit::window::WindowBuilder;

use crate::renderer::Renderer;

#[tokio::main]
async fn main() {
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
    let window = WindowBuilder::new().build(&event_loop).unwrap();
    info!("Created event loop and window");

    info!("Creating renderer");
    let renderer = Renderer::new(&window);
    info!("Created renderer");

    info!("Entering event loop");
    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;

        match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                _ => (),
            }
            _ => (),
        }
    });
}
