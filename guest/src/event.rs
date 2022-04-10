pub enum Event {
    Keyboard(u8),
    Tick,
}

impl Event {
    fn next() -> Self {
        loop {
            if let Some(c) = crate::keyboard::poll() {
                break Self::Keyboard(c);
            } else if crate::timer::poll() {
                break Self::Tick;
            }
        }
    }
}

pub fn run_loop(mut f: impl FnMut(Event)) -> ! {
    loop {
        f(Event::next());
    }
}
