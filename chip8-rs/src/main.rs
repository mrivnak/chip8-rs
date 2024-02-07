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

use chip8::display::{DISPLAY_HEIGHT, DISPLAY_WIDTH};
use chip8::system::SystemBuilder;

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
    let system_builder = SystemBuilder::new(rom);

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

    let system = system_builder.run();

    // Main loop
    let _ = event_loop.run(|event, event_target| {
        // Window event handling
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
                        use winit::keyboard::{KeyCode, PhysicalKey};
                        // Keymap:
                        // on a US layout keyboard, a 4x4 grid on the left side of the keyboard
                        // 0-3: 1, 2, 3, 4
                        // 4-7: q, w, e, r
                        // 8-11: a, s, d, f
                        // 12-15: z, x, c, v

                        if let Some(key) = match key_event.physical_key {
                            PhysicalKey::Code(code) => match code {
                                KeyCode::Digit1 => Some(0x0),
                                KeyCode::Digit2 => Some(0x1),
                                KeyCode::Digit3 => Some(0x2),
                                KeyCode::Digit4 => Some(0x3),
                                KeyCode::KeyQ => Some(0x4),
                                KeyCode::KeyW => Some(0x5),
                                KeyCode::KeyE => Some(0x6),
                                KeyCode::KeyR => Some(0x7),
                                KeyCode::KeyA => Some(0x8),
                                KeyCode::KeyS => Some(0x9),
                                KeyCode::KeyD => Some(0xA),
                                KeyCode::KeyF => Some(0xB),
                                KeyCode::KeyZ => Some(0xC),
                                KeyCode::KeyX => Some(0xD),
                                KeyCode::KeyC => Some(0xE),
                                KeyCode::KeyV => Some(0xF),
                                _ => None,
                            },
                            _ => None,
                        } {
                            match key_event.state {
                                ElementState::Pressed => system.key_down(key as u8),
                                ElementState::Released => system.key_up(key as u8),
                            }
                        }
                    }
                    WindowEvent::CloseRequested => event_target.exit(),
                    WindowEvent::Resized(physical_size) => renderer.resize(*physical_size),
                    WindowEvent::RedrawRequested => {
                        // Pixel rendering
                        let pixels = system
                            .pixels()
                            .into_iter()
                            .map(|p| match p {
                                chip8::display::Pixel::On => renderer::Pixel::On,
                                chip8::display::Pixel::Off => renderer::Pixel::Off,
                            })
                            .collect::<Vec<_>>();

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
            Event::AboutToWait => window.request_redraw(),
            _ => {}
        }
    });
}
