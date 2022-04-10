#![no_std]
#![no_main]

use core::arch::asm;

fn inb(port: u16) -> u8 {
    let ret: u8;
    unsafe {
        asm!("in al, dx", out("al") ret, in("dx") port);
    }
    ret
}

fn outb(port: u16, val: u8) {
    unsafe {
        asm!("out dx, al", in("al") val, in("dx") port);
    }
}

fn getch() -> u8 {
    while inb(0x45) == 0 {}
    let c = inb(0x44);
    outb(0x45, 0);
    c
}

fn putch(c: u8) {
    outb(0x42, c);
}

#[no_mangle]
pub extern "C" fn rsmain() {
    loop {
        putch(getch());
    }
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    unsafe {
        asm!("hlt");
    }
    loop {}
}
