use std::io::Write;

use anyhow::Result;
use kvm_bindings::kvm_userspace_memory_region;
use kvm_ioctls::{Kvm, VcpuExit, VcpuFd, VmFd};
use memmap2::MmapMut;

#[allow(dead_code)]
pub struct VirtualMachine {
    kvm_fd: Kvm,
    vm_fd: VmFd,
    vcpu_fd: VcpuFd,

    guest_mem: MmapMut,

    console_buffer: Vec<u8>,
}

impl VirtualMachine {
    pub fn new(mem_size: usize, guest_phys_addr: u64, code: &[u8]) -> Result<Self> {
        let kvm_fd = Kvm::new()?;
        let vm_fd = kvm_fd.create_vm()?;

        let mut guest_mem = MmapMut::map_anon(mem_size)?;
        guest_mem[..code.len()].copy_from_slice(code);

        let mem_region = kvm_userspace_memory_region {
            slot: 0,
            flags: Default::default(),
            guest_phys_addr,
            memory_size: mem_size as u64,
            userspace_addr: guest_mem.as_mut_ptr() as u64,
        };
        unsafe { vm_fd.set_user_memory_region(mem_region)? };

        let vcpu_fd = vm_fd.create_vcpu(0)?;
        setup_regs(&vcpu_fd, guest_phys_addr)?;

        Ok(Self {
            kvm_fd,
            vm_fd,
            vcpu_fd,
            guest_mem,
            console_buffer: Vec::new(),
        })
    }

    pub fn run_to_completion(mut self) -> Result<()> {
        while !self.run_once()? {}
        Ok(())
    }

    fn run_once(&mut self) -> Result<bool> {
        let vm_exit = self.vcpu_fd.run()?;
        match vm_exit {
            VcpuExit::IoOut(0x0042, data) => {
                for &byte in data {
                    self.console_buffer.push(byte);
                    if byte == b'\n' {
                        std::io::stdout().write(&self.console_buffer)?;
                        self.console_buffer.clear();
                    }
                }
            }
            VcpuExit::IoIn(addr, data) => println!("io in {addr:x} {data:02x?}"),
            VcpuExit::IoOut(addr, data) => println!("io out {addr:x} {data:02x?}"),
            VcpuExit::Hlt => return Ok(true),
            r => anyhow::bail!("Unexpected exit: {r:?}"),
        }
        Ok(false)
    }
}

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
