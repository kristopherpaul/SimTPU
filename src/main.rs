use anyhow::{Context, Result};
use simtpu::{Tpu, TpuConfig};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config_path = std::env::args().nth(1).context("usage: simtpu <config>")?;
    let config = TpuConfig::from_file(&config_path).context(format!("failed to load TPU config from {}", config_path))?;

    let mut tpu = Tpu::new(config).context("failed to initialize TPU simulator")?;
    tpu.run().context("TPU execution failed unexpectedly")?;

    Ok(())
}