use crate::cpu;
use crate::cpu::Cpu;
use std::sync::{Arc, Mutex};
use std::thread;

const CPU_CYCLE_TIME: std::time::Duration =
    std::time::Duration::from_millis((1.0 / cpu::FREQUENCY as f32 * 1000.0) as u64);

const TIMER_CYCLE_TIME: std::time::Duration =
    std::time::Duration::from_millis((1.0 / 60.0 * 1000.0) as u64); // 60 Hz

pub struct SystemBuilder<'a> {
    rom: &'a [u8],
}

impl<'a> SystemBuilder<'a> {
    pub fn new(rom: &'a [u8]) -> SystemBuilder<'a> {
        SystemBuilder { rom }
    }

    pub fn run(self) -> System {
        let cpu = Arc::new(Mutex::new(Cpu::init(self.rom)));

        let tick_thread_cpu = Arc::clone(&cpu);
        let cpu_thread = thread::spawn(move || loop {
            tick_thread_cpu.lock().expect("Unable to lock CPU").tick();
            thread::sleep(CPU_CYCLE_TIME); // TODO: Implement proper timing
        });

        let timer_thread_cpu = Arc::clone(&cpu);
        let timer_thread = thread::spawn(move || loop {
            timer_thread_cpu
                .lock()
                .expect("Unable to lock CPU")
                .tick_timers();
            thread::sleep(TIMER_CYCLE_TIME); // TODO: Implement proper timing
        });

        System {
            cpu,
            cpu_thread,
            timer_thread,
        }
    }
}

pub struct System {
    pub cpu: Arc<Mutex<Cpu>>,
    cpu_thread: thread::JoinHandle<()>,
    timer_thread: thread::JoinHandle<()>,
}

impl System {
    pub fn pixels(&self) -> [crate::display::Pixel; crate::display::DISPLAY_SIZE] {
        self.cpu
            .lock()
            .expect("Unable to lock CPU")
            .pixels()
            .to_owned()
    }

    pub fn has_new_frame(&self) -> bool {
        self.cpu.lock().expect("Unable to lock CPU").drawing
    }

    pub fn clear_new_frame(&self) {
        self.cpu.lock().expect("Unable to lock CPU").drawing = false;
    }

    pub fn key_down(&self, key: u8) {
        self.cpu.lock().expect("Unable to lock CPU").key_down(key);
    }

    pub fn key_up(&self, key: u8) {
        self.cpu.lock().expect("Unable to lock CPU").key_up(key);
    }

    pub fn stop(self) {
        let _ = self.cpu_thread.join();
        let _ = self.timer_thread.join();
    }
}
