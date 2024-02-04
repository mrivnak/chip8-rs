use crate::data::OpCode;
use crate::gpu::{Display, Pixel, DISPLAY_SIZE};
use crate::memory::PROGRAM_START;
use crate::{data::Address, memory::MemoryBus, registers::Registers};

const STACK_SIZE: usize = 16;
pub const FREQUENCY: u32 = 500; // 500 Hz

pub type Stack = [Address; STACK_SIZE];

pub struct CPU {
    pub registers: Registers,
    pub memory: MemoryBus,
    pub stack: Stack,
    display: Display,
    pub has_new_frame: bool,
}

impl Default for CPU {
    fn default() -> CPU {
        CPU {
            registers: Registers::default(),
            memory: MemoryBus::default(),
            stack: [0; STACK_SIZE],
            display: Display::default(),
            has_new_frame: false,
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

    pub fn pixels(&mut self) -> &[Pixel; DISPLAY_SIZE] {
        self.has_new_frame = false;
        &self.display.pixels
    }

    pub fn tick(&mut self) {
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
            0x0 => match instr & 0x00FF {
                0xE0 => self.display.clear(),
                0xEE => self.ret(),
                _ => self.sys_addr(instr),
            },
            0x1 => self.jump_addr(instr & 0x0FFF),
            0x2 => self.call_addr(instr & 0x0FFF),
            0x3 => {
                self.registers.pc += if self.registers.v[(instr as u16 & 0x0F00 >> 8) as usize]
                    == (instr & 0x00FF) as u8
                {
                    2
                } else {
                    0
                }
            }
            0x4 => {
                self.registers.pc += if self.registers.v[(instr as u16 & 0x0F00 >> 8) as usize]
                    != (instr & 0x00FF) as u8
                {
                    2
                } else {
                    0
                }
            }
            0x5 => {
                self.registers.pc += if self.registers.v[(instr as u16 & 0x0F00 >> 8) as usize]
                    == self.registers.v[(instr as u16 & 0x0F00 >> 4) as usize]
                {
                    2
                } else {
                    0
                }
            }
            0x6 => {
                self.registers.v[(instr as u16 & 0x0F00 >> 8) as usize] = (instr & 0x00FF) as u8;
            }
            0x7 => {
                self.registers.v[(instr as u16 & 0x0F00 >> 8) as usize] += (instr & 0x00FF) as u8;
            }
            0x8 => match instr & 0x000F {
                0x0 => {
                    self.registers.v[(instr as u16 & 0x0F00 >> 8) as usize] =
                        self.registers.v[(instr as u16 & 0x00F0 >> 4) as usize]
                }
                0x1 => {
                    self.registers.v[(instr as u16 & 0x0F00 >> 8) as usize] |=
                        self.registers.v[(instr as u16 & 0x00F0 >> 4) as usize]
                }
                0x2 => {
                    self.registers.v[(instr as u16 & 0x0F00 >> 8) as usize] &=
                        self.registers.v[(instr as u16 & 0x00F0 >> 4) as usize]
                }
                0x3 => {
                    self.registers.v[(instr as u16 & 0x0F00 >> 8) as usize] ^=
                        self.registers.v[(instr as u16 & 0x00F0 >> 4) as usize]
                }
                0x4 => {
                    let (result, overflow) = self.registers.v
                        [(instr as u16 & 0x0F00 >> 8) as usize]
                        .overflowing_add(self.registers.v[(instr as u16 & 0x00F0 >> 4) as usize]);
                    self.registers.v[0xF] = if overflow { 1 } else { 0 };
                    self.registers.v[(instr as u16 & 0x0F00 >> 8) as usize] = result;
                }
                0x5 => {
                    let (result, overflow) = self.registers.v
                        [(instr as u16 & 0x0F00 >> 8) as usize]
                        .overflowing_sub(self.registers.v[(instr as u16 & 0x00F0 >> 4) as usize]);
                    self.registers.v[0xF] = if overflow { 0 } else { 1 };
                    self.registers.v[(instr as u16 & 0x0F00 >> 8) as usize] = result;
                }
                0x6 => {
                    self.registers.v[0xF] =
                        self.registers.v[(instr as u16 & 0x0F00 >> 8) as usize] & 0x1;
                    self.registers.v[(instr as u16 & 0x0F00 >> 8) as usize] >>= 1;
                }
                0x7 => {
                    let (result, overflow) = self.registers.v
                        [(instr as u16 & 0x00F0 >> 4) as usize]
                        .overflowing_sub(self.registers.v[(instr as u16 & 0x0F00 >> 8) as usize]);
                    self.registers.v[0xF] = if overflow { 0 } else { 1 };
                    self.registers.v[(instr as u16 & 0x0F00 >> 8) as usize] = result;
                }
                _ => unreachable!("Instruction 0x{:04X} is not valid", instr),
            },
            _ => unimplemented!("Instruction 0x{:04X} not implemented", instr),
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
