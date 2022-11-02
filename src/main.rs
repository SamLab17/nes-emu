mod mem;
mod error;
mod cpu;

use error::Result;

fn main() -> Result<()>{
    let mut cpu = cpu::cpu::Cpu::new();
    cpu.run_next_instr()?;
    Ok(())
}
