use clap::Parser;

use chip8::cpu::CPU;

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
    // Argument parsing
    let args = Args::parse();

    let mut cpu = CPU::default();
    cpu.load_rom(&args.file);

    
}
