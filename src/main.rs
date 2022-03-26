mod timer;
mod vm;

use anyhow::Result;

use vm::VirtualMachine;

fn main() -> Result<()> {
    #[rustfmt::skip]
    let asm_code = [
        0xba, 0x42, 0x00, // mov $0x0042, %dx
        0x00, 0xd8,       // add %bl, %al
        0x04, b'0',       // add $'0', %al
        0xee,             // out %al, (%dx)
        0xb0, b'\n',      // mov $'\n', %al
        0xee,             // out %al, (%dx)
        0xf4,             // hlt
    ];

    let vm = VirtualMachine::new(0x4000, 0x1000, &asm_code)?;
    vm.run_to_completion()?;

    Ok(())
}
