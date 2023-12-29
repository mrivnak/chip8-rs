use crate::data::OpCode;
use crate::gpu::{Display, Pixel, DISPLAY_SIZE};
use crate::memory::PROGRAM_START;
use crate::{data::Address, memory::MemoryBus, registers::Registers};

const STACK_SIZE: usize = 16;

pub type Stack = [Address; STACK_SIZE];

pub struct CPU {
    pub registers: Registers,
    pub memory: MemoryBus,
    pub stack: Stack,
    display: Display,
}

impl Default for CPU {
    fn default() -> CPU {
        CPU {
            registers: Registers::default(),
            memory: MemoryBus::default(),
            stack: [0; STACK_SIZE],
            display: Display::default(),
        }
    }
}

impl CPU {
    pub fn init() -> CPU {
        let mut cpu = CPU::default();

        // Load font into memory
        let font_rom = include_bytes!("../../res/font.bin");
        const FONT_START: Address = 0x50; // Arbitrary, but it's convention to start at 0x50
        cpu.memory.write_bytes(FONT_START, font_rom);

        cpu
    }

    pub fn load_rom(&mut self, rom: &[u8]) {
        self.memory.write_bytes(PROGRAM_START, rom);
    }

    pub fn pixels(&self) -> &[Pixel; DISPLAY_SIZE] {
        &self.display.pixels
    }

    pub fn run(&mut self) {
        // TODO: tick in a separate thread at 1 MHz
        // loop {
        //     self.tick();
        // }
    }

    fn tick(&mut self) {
        // TODO: Implement tick

        let instr = self.fetch();
        self.execute(instr);
    }

    fn fetch(&mut self) -> OpCode {
        let left = self.memory.read(self.registers.pc);
        let right = self.memory.read(self.registers.pc + 1);
        let instruction = (left as u16) << 8 | right as u16;
        self.registers.pc += 2;
        instruction
    }

    fn execute(&mut self, instr: OpCode) {
        match (instr & 0xF000) >> 12 {
            0 => match instr & 0x00FF {
                0xE0 => self.display.clear(),
                0xEE => self.ret(),
                _ => self.sys_addr(instr),
            },
            1 => self.jump_addr(instr & 0x0FFF),
            2 => self.call_addr(instr & 0x0FFF),
            _ => unimplemented!("Instruction {:04X} not implemented", instr),
        }
    }

    fn ret(&mut self) {
        self.registers.pc = self.stack[self.registers.sp];
        self.registers.sp = self.registers.sp.saturating_sub(1);
    }

    fn sys_addr(&mut self, _addr: Address) {
        // unimplemented!("System instructions are not implemented");
        // Ignore system instructions
    }

    fn jump_addr(&mut self, addr: Address) {
        self.registers.pc = addr;
    }

    fn call_addr(&mut self, addr: Address) {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fetch() {
        let mut cpu = CPU::default();
        cpu.registers.pc = 0x200;
        cpu.memory.write(0x200, 0x12);
        cpu.memory.write(0x201, 0x34);

        assert_eq!(cpu.fetch(), 0x1234); // check that the instruction is read correctly
        assert_eq!(cpu.registers.pc, 0x202); // check that the program counter is incremented
    }

    #[test]
    fn test_ret() {
        let mut cpu = CPU::default();
        cpu.registers.sp = 1;
        cpu.stack[0] = 0x1234;
        cpu.stack[1] = 0x5678;

        cpu.ret();
        assert_eq!(cpu.registers.pc, 0x5678);
        assert_eq!(cpu.registers.sp, 0);

        cpu.ret();
        assert_eq!(cpu.registers.pc, 0x1234);
        assert_eq!(cpu.registers.sp, 0); // saturating_sub should prevent underflow
    }

    #[test]
    fn test_jump_addr() {
        let mut cpu = CPU::default();
        assert_eq!(cpu.registers.pc, 0);
        cpu.jump_addr(0x1234 & 0x0FFF);
        assert_eq!(cpu.registers.pc, 0x234);
    }
}
