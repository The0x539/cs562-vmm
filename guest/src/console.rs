use crate::io::outb;

pub fn put_char(c: u8) {
    outb(0x42, c);
}

pub fn print(s: &[u8]) {
    for &c in s {
        put_char(c);
    }
}
