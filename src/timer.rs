use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use std::time::Duration;

use parking_lot::{Condvar, Mutex};

#[derive(Default)]
struct TimerFlags(u8);

impl TimerFlags {
    fn enabled(&self) -> bool {
        self.0 & 1 != 0
    }

    fn fire(&mut self) {
        self.0 |= 2;
    }
}

#[derive(Default)]
pub struct Timer {
    interval: AtomicU32,
    cv: Condvar,
    flags: Mutex<TimerFlags>,
}

impl Timer {
    fn interval(&self) -> Duration {
        let millis = self.interval.load(Ordering::Relaxed);
        Duration::from_millis(millis as u64)
    }

    pub fn set_interval(&self, millis: u32) {
        self.interval.store(millis, Ordering::Relaxed);
    }

    pub fn flags(&self) -> u8 {
        self.flags.lock().0
    }

    pub fn set_flags(&self, val: u8) {
        self.flags.lock().0 = val;
        self.cv.notify_all();
    }

    fn run(&self) {
        loop {
            std::thread::sleep(self.interval());

            let mut flags = self.flags.lock();
            if flags.enabled() {
                flags.fire();
            } else {
                while !flags.enabled() {
                    self.cv.wait(&mut flags);
                }
            }
        }
    }

    pub fn launch(self: &Arc<Self>) {
        let this = self.clone();
        std::thread::spawn(move || this.run());
    }
}
