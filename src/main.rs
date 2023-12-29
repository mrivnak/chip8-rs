#[cfg(not(target_arch = "wasm32"))]
use clap::Parser;
#[cfg(not(target_arch = "wasm32"))]
use std::io::Read;
use tracing::{error, info, warn};
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;
use winit::dpi::LogicalSize;
use winit::event::{Event, WindowEvent};
use winit::event_loop::EventLoop;
use winit::window::WindowBuilder;

use chip8::cpu::CPU;
use chip8::gpu::{DISPLAY_HEIGHT, DISPLAY_WIDTH};
use crate::renderer::PIXEL_SIZE;

mod renderer;

#[cfg(not(target_arch = "wasm32"))]
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

#[cfg(not(target_arch = "wasm32"))]
#[tokio::main]
async fn main() {
    run().await;
}

#[cfg(target_arch = "wasm32")]
fn main() {
    // Ignored by wasm-bindgen
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen(start))]
pub async fn run() {
    cfg_if::cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
            console_error_panic_hook::set_once();
            tracing_wasm::set_as_global_default();
        } else {
            tracing_subscriber::fmt::init()
        }
    }
    info!("info!!!");
    warn!("warning");
    error!("eeeeeek");

    let mut rom: &[u8] = &[];
    cfg_if::cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
            rom = include_bytes!("../res/roms/pong.ch8");
        } else {
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
            rom = rom_buffer.as_slice();
        }
    }

    // Initialize CPU
    let mut cpu = CPU::init();
    cpu.load_rom(rom);

    // Create window
    let event_loop = EventLoop::new().expect("Could not create event loop");
    let window = WindowBuilder::new()
        .with_title("Chip8")
        .with_inner_size(LogicalSize::new(
            (DISPLAY_WIDTH * PIXEL_SIZE) as f64,
            (DISPLAY_HEIGHT * PIXEL_SIZE) as f64,
        ))
        .build(&event_loop)
        .expect("Could not create window");

    #[cfg(target_arch = "wasm32")]
    {
        use winit::platform::web::WindowExtWebSys;
        web_sys::window()
            .and_then(|win| win.document())
            .and_then(|doc| {
                let dst = doc.get_element_by_id("wasm-example")?;
                let canvas = web_sys::Element::from(window.canvas().unwrap());
                dst.append_child(&canvas).ok()?;
                Some(())
            })
            .expect("Couldn't append canvas to div.");
    }

    let mut state = renderer::Renderer::new(&window).await;

    // Main loop
    let _ = event_loop.run(move |event, event_target| {
        // cpu.tick(); // TODO: tick at 1 MHz
        // Does the cpu need to be in a separate thread so the main loop doesn't need to be that fast
        // Can we send signals between threads to synchronize?

        let pixels = cpu.pixels();

        // TODO: maybe set a flag when pixels change to avoid redraws

        match event {
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == window.id() => if !state.input(event) {
                match event {
                    WindowEvent::CloseRequested => event_target.exit(),
                    WindowEvent::Resized(physical_size) => {
                        state.resize(*physical_size);
                    }
                    WindowEvent::RedrawRequested => {
                        state.update();
                        match state.render(pixels) {
                            Ok(_) => {}
                            // Reconfigure the surface if lost
                            Err(wgpu::SurfaceError::Lost) => state.resize(state.size),
                            // The system is out of memory, we should probably quit
                            Err(wgpu::SurfaceError::OutOfMemory) => event_target.exit(),
                            // All other errors (Outdated, Timeout) should be resolved by the next frame
                            Err(e) => eprintln!("{:?}", e),
                        }
                    }
                    _ => {}
                }},
            _ => {}
        }
    });
}
