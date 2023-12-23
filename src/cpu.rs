use std::io::Read;

use crate::{data::Address, memory::MemoryBus, registers::Registers, stack::Stack};

pub struct CPU {
    pub registers: Registers,
    pub memory: MemoryBus,
    pub stack: Stack,
}

impl Default for CPU {
    fn default() -> CPU {
        CPU {
            registers: Registers::default(),
            memory: MemoryBus::default(),
            stack: Stack::default(),
        }
    }
}

impl CPU {
    pub fn load_rom(&mut self, filename: &str) {
        let file = std::fs::File::open(filename).expect("Could not open file");
        let mut reader = std::io::BufReader::new(file);
        let mut buffer = Vec::new();

        reader
            .read_to_end(&mut buffer)
            .expect("Could not read file");

        for (i, byte) in buffer.iter().enumerate() {
            self.memory.write(i as Address, *byte);
        }
    }

    pub fn tick(&mut self) {
        // TODO: Implement tick
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_rom() {
        let mut romPath = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        romPath.push("res/test/test_opcode.ch8");
        println!("{:?}", romPath);

        let file = std::fs::File::open(romPath.clone()).expect("Could not open file");
        let mut reader = std::io::BufReader::new(file);
        let mut buffer = Vec::new();

        reader
            .read_to_end(&mut buffer)
            .expect("Could not read file");

        let mut cpu = CPU::default();
        cpu.load_rom(romPath.to_str().unwrap());
    }
}
