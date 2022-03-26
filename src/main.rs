use anyhow::Result;
use kvm_bindings::kvm_userspace_memory_region;
use kvm_ioctls::VcpuExit;
use kvm_ioctls::{Kvm, VcpuFd};
use memmap2::MmapMut;

fn main() -> Result<()> {
    #[rustfmt::skip]
    let asm_code = [
        0xba, 0xf8, 0x03,             // mov $0x3f8, %dx
        0x00, 0xd8,                   // add %bl, %al
        0x04, b'0',                   // add $'0', %al
        0xee,                         // out %al, %dx
        0xec,                         // in %dx, %al
        0xc6, 0x06, 0x00, 0x80, 0x00, // movl $0, (0x8000); This generates a MMIO Write.
        0x8a, 0x16, 0x00, 0x80,       // movl (0x8000), %dl; This generates a MMIO Read.
        0xf4,                         // hlt
    ];

    let kvm = Kvm::new()?;
    let vm = kvm.create_vm()?;

    let mem_size = 0x4000;
    let guest_phys_addr = 0x1000;
    let slot = 0;

    let mut guest_mem = MmapMut::map_anon(mem_size)?;
    guest_mem[..asm_code.len()].copy_from_slice(&asm_code);

    let mem_region = kvm_userspace_memory_region {
        slot,
        flags: kvm_bindings::KVM_MEM_LOG_DIRTY_PAGES,
        guest_phys_addr,
        memory_size: mem_size as u64,
        userspace_addr: guest_mem.as_ptr() as u64,
    };
    unsafe { vm.set_user_memory_region(mem_region)? };

    let mut vcpu = vm.create_vcpu(0)?;

    setup_regs(&mut vcpu, guest_phys_addr)?;

    loop {
        match vcpu.run().expect("run failed") {
            VcpuExit::IoIn(addr, data) => {
                println!("io in {addr:x} {data:02x?}");
            }
            VcpuExit::IoOut(addr, data) => {
                println!("io out {addr:x} {data:02x?}");
            }
            VcpuExit::MmioRead(addr, data) => {
                println!("mmio read from {addr:x}: {data:02x?}");
            }
            VcpuExit::MmioWrite(addr, data) => {
                println!("mmio write to {addr:x}: {data:02x?}");

                let num_dirty_pages = vm
                    .get_dirty_log(slot, mem_size)?
                    .into_iter()
                    .map(u64::count_ones)
                    .sum::<u32>();

                assert_eq!(num_dirty_pages, 1);
            }
            VcpuExit::Hlt => break,
            r => anyhow::bail!("Unexpected exit: {r:?}"),
        }
    }

    Ok(())
}

fn setup_regs(vcpu: &VcpuFd, rip: u64) -> Result<()> {
    let mut sregs = vcpu.get_sregs()?;
    sregs.cs.base = 0;
    sregs.cs.selector = 0;
    vcpu.set_sregs(&sregs)?;

    let mut regs = vcpu.get_regs()?;
    regs.rip = rip;
    regs.rax = 2;
    regs.rbx = 3;
    regs.rflags = 2;
    vcpu.set_regs(&regs)?;

    Ok(())
}
