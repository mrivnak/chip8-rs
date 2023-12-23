use super::data::Address;

const MEMORY_SIZE: usize = 0x1000;

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

    pub fn read(&self, addr: Address) -> u8 {
        self.memory[addr as usize]
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