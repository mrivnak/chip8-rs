use clap::Parser;
use std::io::Read;
use tracing::{error, info, warn};
use winit::dpi::LogicalSize;
use winit::event::Event;
use winit::event_loop::EventLoop;
use winit::window::WindowBuilder;

use crate::renderer::DEFAULT_PIXEL_SIZE;
use chip8::cpu::CPU;
use chip8::gpu::{Pixel, DISPLAY_HEIGHT, DISPLAY_WIDTH};

mod data;
mod renderer;

#[derive(Parser, Clone, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Display verbose output
    #[arg(short, long)]
    verbose: bool,

    /// Display debug info
    #[arg(short, long)]
    debug: bool,

    /// ROM file to run
    #[arg(value_parser, required = true)]
    file: String,
}

#[tokio::main]
pub async fn main() {
    cfg_if::cfg_if! {
        if #[cfg(debug_assertions)] {
            tracing_subscriber::fmt::init();
        }
    }

    // Argument parsing
    let args = Args::parse();
    dbg!(args.clone());

    // Read ROM file
    let file = std::fs::File::open(&args.file).expect("Could not open file");
    let mut reader = std::io::BufReader::new(file);
    let mut rom_buffer = Vec::new();

    reader
        .read_to_end(&mut rom_buffer)
        .expect("Could not read file");
    let rom = rom_buffer.as_slice();

    // Initialize CPU
    let mut cpu = CPU::init();
    cpu.load_rom(rom);

    // Create window
    let event_loop = EventLoop::new().expect("Could not create event loop");
    let window = WindowBuilder::new()
        .with_title("Chip8")
        .with_inner_size(LogicalSize::new(
            DISPLAY_WIDTH as f32 * DEFAULT_PIXEL_SIZE,
            DISPLAY_HEIGHT as f32 * DEFAULT_PIXEL_SIZE,
        ))
        .build(&event_loop)
        .expect("Could not create window");

    let mut renderer = renderer::PixelRenderer::new(&window, DISPLAY_HEIGHT, DISPLAY_WIDTH).await;

    // Run CPU
    cpu.run();

    // Main loop
    let _ = event_loop.run(|event, event_target| {
        let pixels = cpu.pixels();

        // TODO: remove, just for testing rendering
        let pixels = pixels
            .iter()
            .map(|pixel| match rand::random() {
                true => Pixel::On,
                false => Pixel::Off,
            })
            .collect::<Vec<_>>();
        let pixels = pixels.as_slice();

        // TODO: maybe set a flag when pixels change to avoid redraws

        match event {
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == window.id() => {
                if !renderer.input(event) {
                    use winit::event::WindowEvent;

                    match event {
                        WindowEvent::KeyboardInput {
                            device_id: _,
                            event: key_event,
                            is_synthetic: _,
                        } => {
                            use winit::event::ElementState;

                            // match key_event.state {
                            //     ElementState::Pressed => todo!(),
                            //     ElementState::Released => todo!(),
                            // }
                        }
                        WindowEvent::CloseRequested => event_target.exit(),
                        WindowEvent::Resized(physical_size) => renderer.resize(*physical_size),
                        WindowEvent::RedrawRequested => {
                            renderer.update();

                            match renderer.render(pixels) {
                                Ok(_) => {}
                                // Reconfigure the surface if lost
                                Err(wgpu::SurfaceError::Lost) => renderer.resize(renderer.size),
                                // The system is out of memory, we should probably quit
                                Err(wgpu::SurfaceError::OutOfMemory) => event_target.exit(),
                                // All other errors (Outdated, Timeout) should be resolved by the next frame
                                Err(e) => error!("{:?}", e),
                            }
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    });
}
