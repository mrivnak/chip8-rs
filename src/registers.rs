pub struct Registers {
    pub v: [u8; 16],
    pub i: u16,
    pub pc: u16,
}

impl Default for Registers {
    fn default() -> Registers {
        Registers {
            v: [0; 16],
            i: 0,
            pc: 0,
        }
    }
}