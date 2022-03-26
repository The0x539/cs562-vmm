use std::collections::VecDeque;
use std::io::{Read, Write};
use std::sync::mpsc::{sync_channel, Receiver, SyncSender};
use std::sync::Arc;

use anyhow::Result;
use kvm_bindings::kvm_userspace_memory_region;
use kvm_ioctls::{Kvm, VcpuExit, VcpuFd, VmFd};
use memmap2::MmapMut;

use crate::timer::Timer;

#[allow(dead_code)]
pub struct VirtualMachine {
    kvm_fd: Kvm,
    vm_fd: VmFd,
    vcpu_fd: VcpuFd,

    guest_mem: MmapMut,

    console_buffer: Vec<u8>,

    keyboard_buffer: VecDeque<u8>,
    keyboard_rx: Receiver<u8>,

    timer: Arc<Timer>,
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

        let timer = Arc::new(Timer::default());
        timer.launch();

        Ok(Self {
            kvm_fd,
            vm_fd,
            vcpu_fd,
            guest_mem,
            console_buffer: Vec::new(),
            keyboard_buffer: VecDeque::new(),
            keyboard_rx,
            timer,
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

            IoOut(0x0046, data) => self.timer.set_interval(u32_from_le_bytes(data)),

            IoIn(0x0047, data) => data[0] = self.timer.flags(),
            IoOut(0x0047, [val, ..]) => self.timer.set_flags(*val),

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

fn u32_from_le_bytes(b: &[u8]) -> u32 {
    let mut v = [0, 0, 0, 0];
    v[..b.len()].copy_from_slice(b);
    u32::from_le_bytes(v)
}
