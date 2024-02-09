use super::data::Address;

const MEMORY_SIZE: usize = 0x1000;
pub const PROGRAM_START: Address = 0x200;

pub struct MemoryBus {
    memory: [u8; MEMORY_SIZE],
}

impl Default for MemoryBus {
    fn default() -> MemoryBus {
        MemoryBus {
            memory: [0; MEMORY_SIZE],
        }
    }
}

impl MemoryBus {
    pub fn write(&mut self, addr: Address, data: u8) {
        self.memory[addr as usize] = data;
    }

    pub fn write_bytes(&mut self, addr: Address, data: &[u8]) {
        for (i, byte) in data.iter().enumerate() {
            self.write(addr + i as Address, *byte);
        }
    }

    pub fn read(&self, addr: Address) -> u8 {
        self.memory[addr as usize]
    }

    pub fn read_bytes(&self, addr: Address, len: usize) -> Vec<u8> {
        let mut data = Vec::with_capacity(len);
        for i in 0..len {
            data.push(self.read(addr + i as Address));
        }
        data
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_write() {
        let mut mem = MemoryBus::default();

        let addresses = vec![0x000, 0x0FF, 0xFFF];
        let values = vec![0x0F, 0xF0, 0xAA];

        for addr in &addresses {
            for val in &values {
                mem.write(*addr, *val);
                assert_eq!(*val, mem.memory[*addr as usize])
            }
        }
    }

    #[test]
    fn test_write_bytes() {
        let mut mem = MemoryBus::default();

        let addr: Address = 0x000;
        let values = [0x0F, 0xF0, 0xAA];

        mem.write_bytes(addr, &values);
        for (i, val) in values.iter().enumerate() {
            assert_eq!(*val, mem.memory[addr as usize + i])
        }
    }

    #[test]
    fn test_read() {
        let mut mem = MemoryBus::default();

        let addresses = vec![0x000, 0x0FF, 0xFFF];
        let values = vec![0x0F, 0xF0, 0xAA];

        for addr in &addresses {
            for val in &values {
                mem.memory[*addr as usize] = *val;
                let result = mem.read(*addr);
                assert_eq!(result, *val)
            }
        }
    }
}
