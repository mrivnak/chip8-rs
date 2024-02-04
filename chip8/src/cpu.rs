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
        let right = self.memory.read(self.registers.pc);
        let left = self.memory.read(self.registers.pc + 1);
        let instruction = (left as u16) << 8 | right as u16;
        self.registers.pc += 2;
        instruction
    }

    fn execute(&mut self, instr: OpCode) {
        match (instr & 0xF000) >> 12 {
            0x0 => match instr & 0x00FF {
                0xE0 => self.display.clear(), // 00E0; CLS
                0xEE => self.ret(),           // 00EE; RET
                _ => self.sys_addr(instr),    // 0NNN; SYS addr
            },
            0x1 => self.jump_addr(instr & 0x0FFF), // 1NNN; JMP addr
            0x2 => self.call_addr(instr & 0x0FFF), // 2NNN; CALL addr
            0x3 => {
                // 3XNN; SE Vx, byte
                let dst = ((instr as u16 & 0x0F00) >> 8) as usize;
                self.registers.pc += if self.registers.v[dst] == (instr & 0x00FF) as u8 {
                    2
                } else {
                    0
                }
            }
            0x4 => {
                // 4XNN; SNE Vx, byte
                let dst = ((instr as u16 & 0x0F00) >> 8) as usize;
                self.registers.pc += if self.registers.v[dst] != (instr & 0x00FF) as u8 {
                    2
                } else {
                    0
                }
            }
            0x5 => {
                // 5XY0; SE Vx, Vy
                let src = ((instr as u16 & 0x00F0) >> 4) as usize;
                let dst = ((instr as u16 & 0x0F00) >> 8) as usize;
                self.registers.pc += if self.registers.v[dst] == self.registers.v[src] {
                    2
                } else {
                    0
                }
            }
            0x6 => {
                // 6XNN; LD Vx, byte
                let dst = ((instr as u16 & 0x0F00) >> 8) as usize;
                self.registers.v[dst] = (instr & 0x00FF) as u8;
            }
            0x7 => {
                // 7XNN; ADD Vx, byte
                let dst = ((instr as u16 & 0x0F00) >> 8) as usize;
                self.registers.v[dst] = self.registers.v[dst].wrapping_add((instr & 0x00FF) as u8);
            }
            0x8 => {
                let src = ((instr as u16 & 0x00F0) >> 4) as usize;
                let dst = ((instr as u16 & 0x0F00) >> 8) as usize;
                match instr & 0x000F {
                    0x0 => self.registers.v[dst] = self.registers.v[src], // 8XY0; LD Vx, Vy
                    0x1 => self.registers.v[dst] |= self.registers.v[src], // 8XY1; OR Vx, Vy
                    0x2 => self.registers.v[dst] &= self.registers.v[src], // 8XY2; AND Vx, Vy
                    0x3 => self.registers.v[dst] ^= self.registers.v[src], // 8XY3; XOR Vx, Vy
                    0x4 => {
                        // 8XY4; ADD Vx, Vy
                        let (result, overflow) =
                            self.registers.v[dst].overflowing_add(self.registers.v[src]);
                        self.registers.v[0xF] = if overflow { 1 } else { 0 };
                        self.registers.v[dst] = result;
                    }
                    0x5 => {
                        // 8XY5; SUB Vx, Vy
                        let (result, overflow) =
                            self.registers.v[dst].overflowing_sub(self.registers.v[src]);
                        self.registers.v[0xF] = if overflow { 0 } else { 1 };
                        self.registers.v[dst] = result;
                    }
                    0x6 => {
                        // 8XY6; SHR Vx {, Vy}
                        self.registers.v[0xF] = self.registers.v[dst] & 0b1;
                        self.registers.v[dst] >>= 1;
                    }
                    0x7 => {
                        // 8XY7; SUBN Vx, Vy
                        let (result, overflow) =
                            self.registers.v[src].overflowing_sub(self.registers.v[dst]);
                        self.registers.v[0xF] = if overflow { 0 } else { 1 };
                        self.registers.v[dst] = result;
                    }
                    _ => unreachable!("Instruction 0x{:04X} is not valid", instr),
                }
            }
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
        self.registers.sp = self.registers.sp.saturating_add(1);
        self.stack[self.registers.sp] = self.registers.pc;
        self.registers.pc = addr & 0x0FFF;
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod tests {
    use super::*;

    #[test]
    fn test_fetch() {
        let mut cpu = CPU::default();
        cpu.registers.pc = 0x200;
        cpu.memory.write(0x200, 0x34);
        cpu.memory.write(0x201, 0x12);

        assert_eq!(cpu.fetch(), 0x1234); // check that the instruction is read correctly
        assert_eq!(cpu.registers.pc, 0x202); // check that the program counter is incremented
    }

    #[test]
    fn test_RET() {
        const RET: OpCode = 0x00EE;

        let mut cpu = CPU::default();
        cpu.registers.sp = 1;
        cpu.stack[0] = 0x1234;
        cpu.stack[1] = 0x5678;

        cpu.execute(RET);
        assert_eq!(cpu.registers.pc, 0x5678);
        assert_eq!(cpu.registers.sp, 0);

        cpu.execute(RET);
        assert_eq!(cpu.registers.pc, 0x1234);
        assert_eq!(cpu.registers.sp, 0); // saturating_sub should prevent underflow
    }

    #[test]
    fn test_JMP_addr() {
        const JMP: OpCode = 0x1234;

        let mut cpu = CPU::default();
        assert_eq!(cpu.registers.pc, 0);
        cpu.execute(JMP);
        assert_eq!(cpu.registers.pc, 0x234);
    }

    #[test]
    fn test_CALL_addr() {
        const CALL: OpCode = 0x2345;

        let mut cpu = CPU::default();
        cpu.registers.pc = 0x200;
        cpu.registers.sp = 0;

        cpu.execute(CALL);

        assert_eq!(cpu.registers.pc, 0x345);
        assert_eq!(cpu.stack[cpu.registers.sp], 0x200);
    }

    #[test]
    fn test_SE_Vx_byte() {
        let mut cpu = CPU::default();
        cpu.registers.v[0] = 0x12;
        cpu.registers.pc = 0x200;

        cpu.execute(0x3012);
        assert_eq!(cpu.registers.pc, 0x202);

        cpu.execute(0x3013);
        assert_eq!(cpu.registers.pc, 0x202);
    }

    #[test]
    fn test_SNE_Vx_byte() {
        let mut cpu = CPU::default();
        cpu.registers.v[0] = 0x12;
        cpu.registers.pc = 0x200;

        cpu.execute(0x4013);
        assert_eq!(cpu.registers.pc, 0x202);

        cpu.execute(0x4012);
        assert_eq!(cpu.registers.pc, 0x202);
    }

    #[test]
    fn test_SE_Vx_Vy() {
        let mut cpu = CPU::default();
        cpu.registers.v[0] = 0x12;
        cpu.registers.v[1] = 0x12;
        cpu.registers.v[2] = 0x13;
        cpu.registers.pc = 0x200;

        cpu.execute(0x5010);
        assert_eq!(cpu.registers.pc, 0x202);

        cpu.execute(0x5020);
        assert_eq!(cpu.registers.pc, 0x202);
    }

    #[test]
    fn test_LD_Vx_byte() {
        let mut cpu = CPU::default();
        cpu.registers.v[0] = 0x00;

        cpu.execute(0x6012);
        assert_eq!(cpu.registers.v[0], 0x12);
    }

    #[test]
    fn test_ADD_Vx_byte() {
        let mut cpu = CPU::default();
        cpu.registers.v[0] = 0x12;

        cpu.execute(0x7012);
        assert_eq!(cpu.registers.v[0], 0x24);

        cpu.registers.v[0] = 0xFF;

        cpu.execute(0x7001);
        assert_eq!(cpu.registers.v[0], 0x00);
    }

    #[test]
    fn test_LD_Vx_Vy() {
        let mut cpu = CPU::default();
        cpu.registers.v[0] = 0x12;
        cpu.registers.v[1] = 0x34;

        cpu.execute(0x8010);
        assert_eq!(cpu.registers.v[0], 0x34);
    }

    #[test]
    fn test_OR_Vx_Vy() {
        let mut cpu = CPU::default();
        cpu.registers.v[0] = 0b1100;
        cpu.registers.v[1] = 0b1010;

        cpu.execute(0x8011);
        assert_eq!(cpu.registers.v[0], 0b1110);
    }

    #[test]
    fn test_AND_Vx_Vy() {
        let mut cpu = CPU::default();
        cpu.registers.v[0] = 0b1100;
        cpu.registers.v[1] = 0b1010;

        cpu.execute(0x8012);
        assert_eq!(cpu.registers.v[0], 0b1000);
    }

    #[test]
    fn test_XOR_Vx_Vy() {
        let mut cpu = CPU::default();
        cpu.registers.v[0] = 0b1100;
        cpu.registers.v[1] = 0b1010;

        cpu.execute(0x8013);
        assert_eq!(cpu.registers.v[0], 0b0110);
    }

    #[test]
    fn test_ADD_Vx_Vy() {
        let mut cpu = CPU::default();
        cpu.registers.v[0] = 0x12;
        cpu.registers.v[1] = 0x34;

        cpu.execute(0x8014);
        assert_eq!(cpu.registers.v[0], 0x46);
        assert_eq!(cpu.registers.v[0xF], 0);

        cpu.registers.v[0] = 0xFF;
        cpu.registers.v[1] = 0x01;

        cpu.execute(0x8014);
        assert_eq!(cpu.registers.v[0], 0x00);
        assert_eq!(cpu.registers.v[0xF], 1);
    }

    #[test]
    fn test_SUB_Vx_Vy() {
        let mut cpu = CPU::default();
        cpu.registers.v[0] = 0x34;
        cpu.registers.v[1] = 0x12;

        cpu.execute(0x8015);
        assert_eq!(cpu.registers.v[0], 0x22);
        assert_eq!(cpu.registers.v[0xF], 1);

        cpu.registers.v[0] = 0x12;
        cpu.registers.v[1] = 0x34;

        cpu.execute(0x8015);
        assert_eq!(cpu.registers.v[0], 0xDE);
        assert_eq!(cpu.registers.v[0xF], 0);
    }

    #[test]
    fn test_SHR_Vx() {
        let mut cpu = CPU::default();
        cpu.registers.v[0] = 0b1100;

        cpu.execute(0x8006);
        assert_eq!(cpu.registers.v[0], 0b0110);
        assert_eq!(cpu.registers.v[0xF], 0);

        cpu.registers.v[0] = 0b1101;

        cpu.execute(0x8006);
        assert_eq!(cpu.registers.v[0], 0b0110);
        assert_eq!(cpu.registers.v[0xF], 1);
    }

    #[test]
    fn test_SUBN_Vx_Vy() {
        let mut cpu = CPU::default();
        cpu.registers.v[0] = 0x12;
        cpu.registers.v[1] = 0x34;

        cpu.execute(0x8017);
        assert_eq!(cpu.registers.v[0], 0x22);
        assert_eq!(cpu.registers.v[0xF], 1);

        cpu.registers.v[0] = 0x34;
        cpu.registers.v[1] = 0x12;

        cpu.execute(0x8017);
        assert_eq!(cpu.registers.v[0], 0xDE);
        assert_eq!(cpu.registers.v[0xF], 0);
    }
}
