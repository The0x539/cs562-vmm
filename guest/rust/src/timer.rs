use crate::io;

pub fn enable(millis: u16) {
    io::outs(0x46, millis);
    io::outb(0x47, 1);
}

pub fn poll() -> bool {
    let reg = io::inb(0x47);
    let tick = reg & 2 != 0;
    if tick {
        io::outb(0x47, reg & !2);
    }
    tick
}
