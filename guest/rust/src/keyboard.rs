use crate::io;

pub fn poll() -> Option<u8> {
    if io::inb(0x45) != 0 {
        let c = io::inb(0x44);
        io::outb(0x45, 0);
        Some(c)
    } else {
        None
    }
}
