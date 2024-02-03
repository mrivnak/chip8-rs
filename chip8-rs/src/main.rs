use clap::Parser;
use pixels_wgpu::data::Color;
use pixels_wgpu::renderer;
use pixels_wgpu::renderer::PixelRenderer;
use std::io::Read;
use tracing::error;
use winit::dpi::LogicalSize;
use winit::event::Event;
use winit::event_loop::EventLoop;
use winit::window::WindowBuilder;
use chip8::cpu;

use chip8::cpu::CPU;
use chip8::gpu::{DISPLAY_HEIGHT, DISPLAY_WIDTH};

const DEFAULT_PIXEL_SIZE: f32 = 20.0;
const PIXEL_ON_COLOR: Color = Color {
    r: 1.0,
    g: 1.0,
    b: 1.0,
    a: 1.0,
};
const PIXEL_OFF_COLOR: Color = Color {
    r: 0.0,
    g: 0.0,
    b: 0.0,
    a: 1.0,
};

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

    let mut renderer = PixelRenderer::new(
        &window,
        DISPLAY_HEIGHT,
        DISPLAY_WIDTH,
        DEFAULT_PIXEL_SIZE,
        PIXEL_ON_COLOR,
        PIXEL_OFF_COLOR,
    )
    .await;

    // Run CPU
    cpu.run();

    // Main loop
    let mut last_frame = std::time::Instant::now();
    let mut pixels = vec![renderer::Pixel::Off; DISPLAY_HEIGHT * DISPLAY_WIDTH];
    let _ = event_loop.run(|event, event_target| {
        let now = std::time::Instant::now();
        if now.duration_since(last_frame) > std::time::Duration::from_millis((1.0 / cpu::FREQUENCY as f32 * 1000.0) as u64) {
            pixels = cpu
                .pixels()
                .into_iter()
                .map(|&p| match p {
                    chip8::gpu::Pixel::On => renderer::Pixel::On,
                    chip8::gpu::Pixel::Off => renderer::Pixel::Off,
                })
                .collect::<Vec<_>>();
            window.request_redraw();
        }

        match event {
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == window.id() => {
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
                        match renderer.render(pixels.as_slice()) {
                            Ok(_) => {}
                            // Reconfigure the surface if lost
                            Err(pixels_wgpu::wgpu::SurfaceError::Lost) => {
                                renderer.resize(renderer.size)
                            }
                            // The system is out of memory, we should probably quit
                            Err(pixels_wgpu::wgpu::SurfaceError::OutOfMemory) => {
                                event_target.exit()
                            }
                            // All other errors (Outdated, Timeout) should be resolved by the next frame
                            Err(e) => error!("{:?}", e),
                        }
                    }
                    _ => {}
                }
            }
            _ => {}
        }

        last_frame = std::time::Instant::now();
    });
}
