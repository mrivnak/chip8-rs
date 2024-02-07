use rand::{Rng, SeedableRng};
use rand_pcg::Pcg64Mcg;

use crate::data::OpCode;
use crate::display::{Display, Pixel, DISPLAY_SIZE};
use crate::keyboard::Keyboard;
use crate::memory::PROGRAM_START;
use crate::timer::Timer;
use crate::{data::Address, memory::MemoryBus, registers::Registers};

const STACK_SIZE: usize = 16;
pub const FREQUENCY: u32 = 500; // 500 Hz
const FONT_START: Address = 0x050; // Arbitrary, but it's convention to start at 0x50

pub type Stack = [Address; STACK_SIZE];

#[derive(Default, Debug, PartialEq)]
pub enum Interrupt {
    #[default]
    None,
    KeyPress(u8),
}

pub struct Cpu {
    pub registers: Registers,
    memory: MemoryBus,
    stack: Stack,
    keyboard: Keyboard,
    display: Display,
    pub drawing: bool,
    rng: Pcg64Mcg,
    interrupt: Interrupt,
    delay: Timer,
    sound: Timer,
}

impl Default for Cpu {
    fn default() -> Cpu {
        Cpu {
            registers: Registers::default(),
            memory: MemoryBus::default(),
            stack: [0; STACK_SIZE],
            keyboard: Keyboard::default(),
            display: Display::default(),
            drawing: false,
            rng: Pcg64Mcg::from_entropy(),
            interrupt: Interrupt::None,
            delay: Timer::new(),
            sound: Timer::new(),
        }
    }
}

impl Cpu {
    pub fn init(rom: &[u8]) -> Cpu {
        let mut cpu = Cpu::default();

        // Load font into memory
        let font_rom = include_bytes!("../../res/font.bin");
        cpu.memory.write_bytes(FONT_START, font_rom);

        // Load ROM into memory
        cpu.memory.write_bytes(PROGRAM_START, rom);
        cpu.registers.pc = PROGRAM_START;

        cpu
    }

    pub fn pixels(&mut self) -> &[Pixel; DISPLAY_SIZE] {
        self.drawing = false;
        &self.display.pixels
    }

    pub fn tick(&mut self) {
        match self.interrupt {
            Interrupt::KeyPress(_) => (),
            Interrupt::None => {
                let instr = self.fetch();
                self.execute(instr);
            }
        }
    }

    pub fn tick_timers(&mut self) {
        self.delay.tick();
        self.sound.tick();
    }

    pub fn key_down(&mut self, key: u8) {
        self.keyboard[key as usize] = true;

        if let Interrupt::KeyPress(x) = self.interrupt {
            self.registers.v[x as usize] = key;
            self.interrupt = Interrupt::None;
        }
    }

    pub fn key_up(&mut self, key: u8) {
        self.keyboard[key as usize] = false;
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
                0xE0 => self.display.clear(), // 00E0; CLS
                0xEE => self.ret(),           // 00EE; RET
                _ => self.sys_addr(instr),    // 0NNN; SYS addr
            },
            0x1 => self.jump_addr(instr & 0x0FFF), // 1NNN; JMP addr
            0x2 => self.call_addr(instr & 0x0FFF), // 2NNN; CALL addr
            0x3 => {
                // 3XNN; SE Vx, byte
                let vx = ((instr as u16 & 0x0F00) >> 8) as usize;
                self.registers.pc += if self.registers.v[vx] == (instr & 0x00FF) as u8 {
                    2
                } else {
                    0
                }
            }
            0x4 => {
                // 4XNN; SNE Vx, byte
                let vx = ((instr as u16 & 0x0F00) >> 8) as usize;
                self.registers.pc += if self.registers.v[vx] != (instr & 0x00FF) as u8 {
                    2
                } else {
                    0
                }
            }
            0x5 => {
                // 5XY0; SE Vx, Vy
                let vx = ((instr as u16 & 0x0F00) >> 8) as usize;
                let vy = ((instr as u16 & 0x00F0) >> 4) as usize;
                self.registers.pc += if self.registers.v[vx] == self.registers.v[vy] {
                    2
                } else {
                    0
                }
            }
            0x6 => {
                // 6XNN; LD Vx, byte
                let vx = ((instr as u16 & 0x0F00) >> 8) as usize;
                self.registers.v[vx] = (instr & 0x00FF) as u8;
            }
            0x7 => {
                // 7XNN; ADD Vx, byte
                let vx = ((instr as u16 & 0x0F00) >> 8) as usize;
                self.registers.v[vx] = self.registers.v[vx].wrapping_add((instr & 0x00FF) as u8);
            }
            0x8 => {
                let vy = ((instr as u16 & 0x00F0) >> 4) as usize;
                let vx = ((instr as u16 & 0x0F00) >> 8) as usize;
                match instr & 0x000F {
                    0x0 => self.registers.v[vx] = self.registers.v[vy], // 8XY0; LD Vx, Vy
                    0x1 => self.registers.v[vx] |= self.registers.v[vy], // 8XY1; OR Vx, Vy
                    0x2 => self.registers.v[vx] &= self.registers.v[vy], // 8XY2; AND Vx, Vy
                    0x3 => self.registers.v[vx] ^= self.registers.v[vy], // 8XY3; XOR Vx, Vy
                    0x4 => {
                        // 8XY4; ADD Vx, Vy
                        let (result, overflow) =
                            self.registers.v[vx].overflowing_add(self.registers.v[vy]);
                        self.registers.v[0xF] = if overflow { 1 } else { 0 };
                        self.registers.v[vx] = result;
                    }
                    0x5 => {
                        // 8XY5; SUB Vx, Vy
                        let (result, overflow) =
                            self.registers.v[vx].overflowing_sub(self.registers.v[vy]);
                        self.registers.v[0xF] = if overflow { 0 } else { 1 };
                        self.registers.v[vx] = result;
                    }
                    0x6 => {
                        // 8XY6; SHR Vx {, Vy}
                        self.registers.v[0xF] = self.registers.v[vx] & 0b0000_0001;
                        self.registers.v[vx] >>= 1;
                    }
                    0x7 => {
                        // 8XY7; SUBN Vx, Vy
                        let (result, overflow) =
                            self.registers.v[vy].overflowing_sub(self.registers.v[vx]);
                        self.registers.v[0xF] = if overflow { 0 } else { 1 };
                        self.registers.v[vx] = result;
                    }
                    0xE => {
                        // 8XYE; SHL Vx {, Vy}
                        self.registers.v[0xF] = (self.registers.v[vx] & 0b1000_0000) >> 7;
                        self.registers.v[vx] <<= 1;
                    }
                    _ => unreachable!("Instruction 0x{:04X} is not valid", instr),
                }
            }
            0x9 => {
                // 9XY0; SNE Vx, Vy
                let vx = ((instr as u16 & 0x0F00) >> 8) as usize;
                let vy = ((instr as u16 & 0x00F0) >> 4) as usize;
                self.registers.pc += if self.registers.v[vx] != self.registers.v[vy] {
                    2
                } else {
                    0
                }
            }
            0xA => self.registers.i = instr & 0x0FFF, // ANNN; LD I, addr
            0xB => self.jump_addr((instr & 0x0FFF) + self.registers.v[0] as u16), // BNNN; JP V0, addr
            0xC => {
                // CXNN; RND Vx, byte
                let vx = ((instr as u16 & 0x0F00) >> 8) as usize;
                self.registers.v[vx] = self.rng.gen::<u8>() & (instr & 0x00FF) as u8;
            }
            0xD => {
                // DXYN; DRW Vx, Vy, nibble
                let vx = ((instr as u16 & 0x0F00) >> 8) as usize;
                let vy = ((instr as u16 & 0x00F0) >> 4) as usize;
                let n = instr & 0x000F;

                let sprite = self.memory.read_bytes(self.registers.i, n as usize);
                let x = self.registers.v[vx] as usize;
                let y = self.registers.v[vy] as usize;

                let collision = self.display.draw(x, y, &sprite);
                self.drawing = true;
                self.registers.v[0xF] = if collision { 1 } else { 0 };
            }
            0xE => match instr & 0x00FF {
                0x9E => {
                    // EX9E; SKP Vx
                    let vx = ((instr as u16 & 0x0F00) >> 8) as usize;
                    self.registers.pc += if self.keyboard[self.registers.v[vx] as usize] {
                        2
                    } else {
                        0
                    }
                }
                0xA1 => {
                    // EXA1; SKNP Vx
                    let vx = ((instr as u16 & 0x0F00) >> 8) as usize;
                    self.registers.pc += if !self.keyboard[self.registers.v[vx] as usize] {
                        2
                    } else {
                        0
                    }
                }
                _ => unimplemented!("Instruction 0x{:04X} not implemented", instr),
            },
            0xF => match instr & 0x00FF {
                0x07 => {
                    // FX07; LD Vx, DT
                    let vx = ((instr as u16 & 0x0F00) >> 8) as usize;
                    self.registers.v[vx] = self.delay.get();
                }
                0x0A => {
                    // FX0A; LD Vx, K
                    let vx = ((instr as u16 & 0x0F00) >> 8) as usize;
                    self.interrupt = Interrupt::KeyPress(vx as u8);
                }
                0x15 => {
                    // FX15; LD DT, Vx
                    let vx = ((instr as u16 & 0x0F00) >> 8) as usize;
                    self.delay.set(self.registers.v[vx]);
                }
                0x18 => {
                    // FX18; LD ST, Vx
                    let vx = ((instr as u16 & 0x0F00) >> 8) as usize;
                    self.sound.set(self.registers.v[vx]);
                }
                0x1E => {
                    // FX1E; ADD I, Vx
                    let vx = ((instr as u16 & 0x0F00) >> 8) as usize;
                    self.registers.i += self.registers.v[vx] as u16;
                }
                0x29 => {
                    // FX29; LD F, Vx
                    let vx = ((instr as u16 & 0x0F00) >> 8) as usize;
                    let digit = self.registers.v[vx];
                    debug_assert!(digit <= 0xF, "Invalid digit: 0x{:X}", digit);
                    self.registers.i = (digit as u16 * 5) + FONT_START;
                }
                0x33 => {
                    // FX33; LD B, Vx
                    let vx = ((instr as u16 & 0x0F00) >> 8) as usize;
                    let value = self.registers.v[vx];
                    self.memory.write(self.registers.i, value / 100);
                    self.memory.write(self.registers.i + 1, (value / 10) % 10);
                    self.memory.write(self.registers.i + 2, value % 10);
                }
                0x55 => {
                    // FX55; LD [I], Vx
                    let x = ((instr as u16 & 0x0F00) >> 8) as usize;
                    for i in 0..=x {
                        self.memory
                            .write(self.registers.i + i as u16, self.registers.v[i]);
                    }
                }
                0x65 => {
                    // FX65; LD Vx, [I]
                    let x = ((instr as u16 & 0x0F00) >> 8) as usize;
                    for i in 0..=x {
                        self.registers.v[i] = self.memory.read(self.registers.i + i as u16);
                    }
                }
                _ => unimplemented!("Instruction 0x{:04X} not implemented", instr),
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
        self.registers.sp = self.registers.sp.saturating_add(1);
        self.stack[self.registers.sp] = self.registers.pc;
        self.registers.pc = addr & 0x0FFF;
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod tests {
    use super::*;
    use crate::display::DISPLAY_WIDTH;

    #[test]
    fn test_tick_timers() {
        let mut cpu = Cpu::default();
        cpu.delay.set(0x12);
        cpu.sound.set(0x34);

        cpu.tick_timers();
        assert_eq!(cpu.delay.get(), 0x11);
        assert_eq!(cpu.sound.get(), 0x33);
    }

    #[test]
    fn test_fetch() {
        let mut cpu = Cpu::default();
        cpu.registers.pc = 0x200;
        cpu.memory.write(0x200, 0x12);
        cpu.memory.write(0x201, 0x34);

        assert_eq!(cpu.fetch(), 0x1234); // check that the instruction is read correctly
        assert_eq!(cpu.registers.pc, 0x202); // check that the program counter is incremented
    }

    #[test]
    fn test_RET() {
        const RET: OpCode = 0x00EE;

        let mut cpu = Cpu::default();
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

        let mut cpu = Cpu::default();
        assert_eq!(cpu.registers.pc, 0);
        cpu.execute(JMP);
        assert_eq!(cpu.registers.pc, 0x234);
    }

    #[test]
    fn test_CALL_addr() {
        const CALL: OpCode = 0x2345;

        let mut cpu = Cpu::default();
        cpu.registers.pc = 0x200;
        cpu.registers.sp = 0;

        cpu.execute(CALL);

        assert_eq!(cpu.registers.pc, 0x345);
        assert_eq!(cpu.stack[cpu.registers.sp], 0x200);
    }

    #[test]
    fn test_SE_Vx_byte() {
        let mut cpu = Cpu::default();
        cpu.registers.v[0] = 0x12;
        cpu.registers.pc = 0x200;

        cpu.execute(0x3012);
        assert_eq!(cpu.registers.pc, 0x202);

        cpu.execute(0x3013);
        assert_eq!(cpu.registers.pc, 0x202);
    }

    #[test]
    fn test_SNE_Vx_byte() {
        let mut cpu = Cpu::default();
        cpu.registers.v[0] = 0x12;
        cpu.registers.pc = 0x200;

        cpu.execute(0x4013);
        assert_eq!(cpu.registers.pc, 0x202);

        cpu.execute(0x4012);
        assert_eq!(cpu.registers.pc, 0x202);
    }

    #[test]
    fn test_SE_Vx_Vy() {
        let mut cpu = Cpu::default();
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
        let mut cpu = Cpu::default();
        cpu.registers.v[0] = 0x00;

        cpu.execute(0x6012);
        assert_eq!(cpu.registers.v[0], 0x12);
    }

    #[test]
    fn test_ADD_Vx_byte() {
        let mut cpu = Cpu::default();
        cpu.registers.v[0] = 0x12;

        cpu.execute(0x7012);
        assert_eq!(cpu.registers.v[0], 0x24);

        cpu.registers.v[0] = 0xFF;

        cpu.execute(0x7001);
        assert_eq!(cpu.registers.v[0], 0x00);
    }

    #[test]
    fn test_LD_Vx_Vy() {
        let mut cpu = Cpu::default();
        cpu.registers.v[0] = 0x12;
        cpu.registers.v[1] = 0x34;

        cpu.execute(0x8010);
        assert_eq!(cpu.registers.v[0], 0x34);
    }

    #[test]
    fn test_OR_Vx_Vy() {
        let mut cpu = Cpu::default();
        cpu.registers.v[0] = 0b1100;
        cpu.registers.v[1] = 0b1010;

        cpu.execute(0x8011);
        assert_eq!(cpu.registers.v[0], 0b1110);
    }

    #[test]
    fn test_AND_Vx_Vy() {
        let mut cpu = Cpu::default();
        cpu.registers.v[0] = 0b1100;
        cpu.registers.v[1] = 0b1010;

        cpu.execute(0x8012);
        assert_eq!(cpu.registers.v[0], 0b1000);
    }

    #[test]
    fn test_XOR_Vx_Vy() {
        let mut cpu = Cpu::default();
        cpu.registers.v[0] = 0b1100;
        cpu.registers.v[1] = 0b1010;

        cpu.execute(0x8013);
        assert_eq!(cpu.registers.v[0], 0b0110);
    }

    #[test]
    fn test_ADD_Vx_Vy() {
        let mut cpu = Cpu::default();
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
        let mut cpu = Cpu::default();
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
        let mut cpu = Cpu::default();
        cpu.registers.v[0] = 0b0000_1100;

        cpu.execute(0x8006);
        assert_eq!(cpu.registers.v[0], 0b0000_0110);
        assert_eq!(cpu.registers.v[0xF], 0);

        cpu.registers.v[0] = 0b0000_1101;

        cpu.execute(0x8006);
        assert_eq!(cpu.registers.v[0], 0b0000_0110);
        assert_eq!(cpu.registers.v[0xF], 1);
    }

    #[test]
    fn test_SUBN_Vx_Vy() {
        let mut cpu = Cpu::default();
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

    #[test]
    fn test_SHL_Vx() {
        let mut cpu = Cpu::default();
        cpu.registers.v[0] = 0b1100_0000;

        cpu.execute(0x800E);
        assert_eq!(cpu.registers.v[0], 0b1000_0000);
        assert_eq!(cpu.registers.v[0xF], 1);

        cpu.registers.v[0] = 0b0100_0000;

        cpu.execute(0x800E);
        assert_eq!(cpu.registers.v[0], 0b1000_0000);
        assert_eq!(cpu.registers.v[0xF], 0);
    }

    #[test]
    fn test_SNE_Vx_Vy() {
        let mut cpu = Cpu::default();
        cpu.registers.v[0] = 0x12;
        cpu.registers.v[1] = 0x12;
        cpu.registers.v[2] = 0x13;
        cpu.registers.pc = 0x200;

        cpu.execute(0x9010);
        assert_eq!(cpu.registers.pc, 0x200);

        cpu.execute(0x9020);
        assert_eq!(cpu.registers.pc, 0x202);
    }

    #[test]
    fn test_LD_I_addr() {
        let mut cpu = Cpu::default();
        cpu.registers.i = 0x1234;

        cpu.execute(0xA432);
        assert_eq!(cpu.registers.i, 0x0432);
    }

    #[test]
    fn test_JP_V0_addr() {
        let mut cpu = Cpu::default();
        cpu.registers.v[0] = 0x12;

        cpu.execute(0xB234);
        assert_eq!(cpu.registers.pc, 0x0246);
    }

    #[test]
    fn test_RND_Vx_byte() {
        let mut cpu = Cpu::default();
        cpu.rng = Pcg64Mcg::seed_from_u64(0);
        cpu.execute(0xC012);
        assert_eq!(cpu.registers.v[0], 0x02);
    }

    #[test]
    fn test_DRW_Vx_Vy_nibble() {
        let mut cpu = Cpu::default();
        cpu.registers.i = 0x200;
        cpu.registers.v[0] = 0;
        cpu.registers.v[1] = 0;
        cpu.memory.write(0x200, 0b11110000);
        cpu.memory.write(0x201, 0b00001111);

        cpu.execute(0xD012);
        assert_eq!(cpu.registers.v[0xF], 0);
        assert!(cpu.drawing);

        let on_pixels = [
            (0, 0),
            (1, 0),
            (2, 0),
            (3, 0),
            (4, 1),
            (5, 1),
            (6, 1),
            (7, 1),
        ];
        for &(x, y) in on_pixels.iter() {
            assert_eq!(cpu.display.pixels[y * DISPLAY_WIDTH + x], Pixel::On);
        }
    }

    #[test]
    fn test_SKP_Vx() {
        let mut cpu = Cpu::default();
        cpu.keyboard[0xB] = true;
        cpu.registers.v[0] = 0xB;
        cpu.registers.pc = 0x200;

        cpu.execute(0xE09E);
        assert_eq!(cpu.registers.pc, 0x202);

        cpu.execute(0xE19E);
        assert_eq!(cpu.registers.pc, 0x202);
    }

    #[test]
    fn test_SKNP_Vx() {
        let mut cpu = Cpu::default();
        cpu.keyboard[0xB] = true;
        cpu.registers.v[0] = 0xB;
        cpu.registers.pc = 0x200;

        cpu.execute(0xE0A1);
        assert_eq!(cpu.registers.pc, 0x200);

        cpu.execute(0xE1A1);
        assert_eq!(cpu.registers.pc, 0x202);
    }

    #[test]
    fn test_LD_Vx_DT() {
        let mut cpu = Cpu::default();
        cpu.delay.set(0x12);
        cpu.registers.v[0] = 0x00;

        cpu.execute(0xF007);
        assert_eq!(cpu.registers.v[0], 0x12);
    }

    #[test]
    fn test_LD_Vx_K() {
        let mut cpu = Cpu::default();
        cpu.registers.v[0] = 0xFF;

        cpu.execute(0xF00A);
        assert_eq!(cpu.interrupt, Interrupt::KeyPress(0));

        cpu.key_down(0xB);
        assert_eq!(cpu.interrupt, Interrupt::None);
        assert_eq!(cpu.registers.v[0], 0xB);
    }

    #[test]
    fn test_LD_DT_Vx() {
        let mut cpu = Cpu::default();
        cpu.registers.v[0] = 0x12;
        cpu.delay.set(0x00);

        cpu.execute(0xF015);
        assert_eq!(cpu.delay.get(), 0x12);
    }

    #[test]
    fn test_LD_ST_Vx() {
        let mut cpu = Cpu::default();
        cpu.registers.v[0] = 0x12;
        cpu.sound.set(0x00);

        cpu.execute(0xF018);
        assert_eq!(cpu.sound.get(), 0x12);
    }

    #[test]
    fn test_ADD_I_Vx() {
        let mut cpu = Cpu::default();
        cpu.registers.i = 0x1234;
        cpu.registers.v[0] = 0x12;

        cpu.execute(0xF01E);
        assert_eq!(cpu.registers.i, 0x1246);
    }

    #[test]
    fn test_LD_F_Vx() {
        let mut cpu = Cpu::default();
        for i in 0x0..=0xF_u16 {
            cpu.registers.v[0] = i as u8;
            cpu.registers.i = 0x000;

            cpu.execute(0xF029);
            assert_eq!(cpu.registers.i, (i * 5) + FONT_START);
        }
    }

    #[test]
    fn test_LD_B_Vx() {
        let mut cpu = Cpu::default();
        cpu.registers.i = 0x200;
        cpu.registers.v[0] = 123;

        cpu.execute(0xF033);
        assert_eq!(cpu.memory.read(0x200), 1);
        assert_eq!(cpu.memory.read(0x201), 2);
        assert_eq!(cpu.memory.read(0x202), 3);
    }

    #[test]
    fn test_LD_I_Vx() {
        let mut cpu = Cpu::default();
        for i in 0x0..=0xF {
            cpu.registers.v[i as usize] = i;
        }

        cpu.registers.i = 0x200;
        cpu.execute(0xF755);

        for i in 0x0..=0xF_u8 {
            if i <= 0x7 {
                assert_eq!(cpu.memory.read(0x200 + i as Address), i);
            } else {
                assert_eq!(cpu.memory.read(0x200 + i as Address), 0);
            }
        }

        cpu.execute(0xFF55);

        for i in 0x0..=0xF_u8 {
            assert_eq!(cpu.memory.read(0x200 + i as Address), i);
        }
    }

    #[test]
    fn test_LD_Vx_I() {
        let mut cpu = Cpu::default();
        for i in 0x0..=0xF {
            cpu.memory.write(0x200 + i as Address, i);
        }

        cpu.registers.i = 0x200;
        cpu.execute(0xF765);

        for i in 0x0..=0xF_u8 {
            if i <= 0x7 {
                assert_eq!(cpu.registers.v[i as usize], i);
            } else {
                assert_eq!(cpu.registers.v[i as usize], 0);
            }
        }

        cpu.execute(0xFF65);

        for i in 0x0..=0xF_u8 {
            assert_eq!(cpu.registers.v[i as usize], i);
        }
    }
}
