pub struct Timer {
    value: u8,
}

impl Timer {
    pub fn new() -> Timer {
        Timer { value: 0 }
    }

    pub fn tick(&mut self) {
        if self.value > 0 {
            self.value -= 1;
        }
    }

    pub fn set(&mut self, value: u8) {
        self.value = value;
    }

    pub fn get(&self) -> u8 {
        self.value
    }
}
