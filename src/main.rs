use anyhow::{Context, Result};
use simtpu::{Tpu, Program};

fn main() -> Result<()> {
    let program_path = std::env::args().nth(1).context("usage: simtpu <program.asm|program.bin>")?;
    let mut program = Program::new(None);
    program.load_file(program_path).context("failed to load program")?;

    let mut tpu = Tpu::new().context("failed to initialize TPU simulator")?;
    tpu.run(&program).context("TPU execution failed unexpectedly")?;

    Ok(())
}