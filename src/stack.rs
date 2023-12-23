pub struct Stack {
    stack: [u8; 16],
    sp: u8,
}

impl Default for Stack {
    fn default() -> Stack {
        Stack {
            stack: [0; 16],
            sp: 0,
        }
    }
}