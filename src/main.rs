mod timer;
mod vm;

use std::path::PathBuf;

use anyhow::{anyhow, Result};
use clap::Parser;

use vm::VirtualMachine;

#[derive(Parser)]
struct Args {
    code_file: PathBuf,
}

fn main() -> Result<()> {
    let args = Args::parse();

    let data = std::fs::read(&args.code_file)?;
    let obj = object::read::File::parse(&data[..])
        .map_err(|e| anyhow!("failed to parse object file: {e}"))?;

    let vm = VirtualMachine::new(0x4000, obj)?;
    vm.run_to_completion()?;

    Ok(())
}
