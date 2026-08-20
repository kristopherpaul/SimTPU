use anyhow::{Context, Result};
use simtpu::Tpu;

fn main() -> Result<()> {
    let mut tpu = Tpu::new().context("failed to initialize TPU simulator")?;
    tpu.run().context("TPU execution failed unexpectedly")?;

    Ok(())
}