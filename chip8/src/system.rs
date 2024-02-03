use crate::cpu;
use crate::cpu::CPU;
use std::sync::{Arc, Mutex};
use std::thread;

const TARGET_FRAME_DELTA: std::time::Duration =
    std::time::Duration::from_millis((1.0 / cpu::FREQUENCY as f32 * 1000.0) as u64);

pub struct SystemBuilder<'a> {
    rom: &'a [u8],
}

impl<'a> SystemBuilder<'a> {
    pub fn new(rom: &'a [u8]) -> SystemBuilder<'a> {
        SystemBuilder { rom }
    }

    pub fn run(self) -> System {
        let cpu = Arc::new(Mutex::new(CPU::init()));
        cpu.lock().expect("Unable to lock CPU").load_rom(self.rom);

        let thread_cpu = Arc::clone(&cpu);
        let cpu_thread = Some(thread::spawn(move || loop {
            thread_cpu.lock().expect("Unable to lock CPU").tick();
            thread::sleep(TARGET_FRAME_DELTA);
        }));

        System { cpu, cpu_thread }
    }
}

pub struct System {
    pub cpu: Arc<Mutex<CPU>>,
    cpu_thread: Option<thread::JoinHandle<()>>,
}

impl System {
    pub fn pixels(&self) -> [crate::gpu::Pixel; crate::gpu::DISPLAY_SIZE] {
        self.cpu
            .lock()
            .expect("Unable to lock CPU")
            .pixels()
            .to_owned()
    }

    pub fn has_new_frame(&self) -> bool {
        self.cpu.lock().expect("Unable to lock CPU").has_new_frame
    }

    pub fn stop(mut self) {
        self.cpu_thread.take().map(|t| t.join());
    }
}
