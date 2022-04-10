use core::arch::asm;

pub fn inb(port: u16) -> u8 {
    let ret: u8;
    unsafe {
        asm!("in al, dx", out("al") ret, in("dx") port);
    }
    ret
}

pub fn outb(port: u16, val: u8) {
    unsafe {
        asm!("out dx, al", in("al") val, in("dx") port);
    }
}

pub fn outs(port: u16, val: u16) {
    unsafe {
        asm!("out dx, ax", in("ax") val, in("dx") port);
    }
}
