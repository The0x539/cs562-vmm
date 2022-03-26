use std::collections::VecDeque;
use std::io::{Read, Write};
use std::sync::mpsc::{sync_channel, Receiver, SyncSender};

use anyhow::Result;
use itertools::Itertools;
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

    keyboard_buffer: VecDeque<u8>,
    keyboard_rx: Receiver<u8>,
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

        let (keyboard_tx, keyboard_rx) = sync_channel(512);
        std::thread::spawn(move || handle_stdin(keyboard_tx));

        Ok(Self {
            kvm_fd,
            vm_fd,
            vcpu_fd,
            guest_mem,
            console_buffer: Vec::new(),
            keyboard_buffer: VecDeque::new(),
            keyboard_rx,
        })
    }

    pub fn run_to_completion(mut self) -> Result<()> {
        while !self.run_once()? {}
        Ok(())
    }

    fn run_once(&mut self) -> Result<bool> {
        self.process_keyboard_input();
        let vm_exit = self.vcpu_fd.run()?;
        use VcpuExit::*;
        match vm_exit {
            IoOut(0x0042, data) => {
                for &byte in data {
                    self.console_buffer.push(byte);
                    if byte == b'\n' {
                        std::io::stdout().write(&self.console_buffer)?;
                        self.console_buffer.clear();
                    }
                }
            }

            IoIn(0x0044, data) => data[0] = *self.keyboard_buffer.front().unwrap_or(&0),

            IoIn(0x0045, data) => data[0] = !self.keyboard_buffer.is_empty() as u8,
            IoOut(0x0045, [0, ..]) => drop(self.keyboard_buffer.pop_front()),

            IoIn(addr, data) => println!("io in {addr:x} {data:02x?}"),
            IoOut(addr, data) => println!("io out {addr:x} {data:02x?}"),
            Hlt => return Ok(true),
            r => anyhow::bail!("Unexpected exit: {r:?}"),
        }
        Ok(false)
    }

    fn process_keyboard_input(&mut self) {
        while let Ok(byte) = self.keyboard_rx.try_recv() {
            if self.keyboard_buffer.len() < 64 {
                self.keyboard_buffer.push_back(byte);
            } else {
                break;
            }
        }
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

fn handle_stdin(tx: SyncSender<u8>) {
    std::io::stdin()
        .lock()
        .bytes()
        .map_while(Result::ok)
        .map_while(move |byte| tx.send(byte).ok())
        .collect()
}
