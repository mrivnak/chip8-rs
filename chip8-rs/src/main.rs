use clap::Parser;
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

#[derive(Parser)]
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

pub fn main() {
    env_logger::init();

    // Argument parsing
    // let args = Args::parse();

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();
    window.set_title("Chip8.rs");

    event_loop.run(move |event, _, control_flow| match event {
        Event::WindowEvent {
            ref event,
            window_id,
        } if window_id == window.id() => match event {
            WindowEvent::CloseRequested
            | WindowEvent::KeyboardInput {
                input:
                    KeyboardInput {
                        state: ElementState::Pressed,
                        virtual_keycode: Some(VirtualKeyCode::Escape),
                        ..
                    },
                ..
            } => *control_flow = ControlFlow::Exit,
            _ => {}
        },
        _ => {
            // Main loop code goes here
        }
    });
}
