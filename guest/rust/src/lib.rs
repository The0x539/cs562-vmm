#![no_std]
#![no_main]

mod console;
mod io;
mod keyboard;
mod timer;

use core::arch::asm;

use tinyvec::ArrayVec;

#[no_mangle]
pub extern "C" fn rsmain() {
    timer::enable(750);

    let mut buf_a = ArrayVec::from_array_empty([0; 64]);
    let mut buf_b = ArrayVec::from_array_empty([0; 64]);
    let (mut in_buf, mut out_buf) = (&mut buf_a, &mut buf_b);

    loop {
        if let Some(c) = keyboard::poll() {
            // if the buffer is full just trample the last byte
            if in_buf.len() == in_buf.capacity() {
                in_buf.pop();
            }
            in_buf.push(c);

            if c == b'\n' {
                core::mem::swap(&mut in_buf, &mut out_buf);
                in_buf.clear();
            }
        } else if timer::poll() {
            console::print(&out_buf);
        }
    }
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    unsafe {
        asm!("hlt");
    }
    loop {}
}
