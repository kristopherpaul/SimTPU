use std::path::Path;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct TpuConfig {
    pub mmu: MmuConfig,
}

#[derive(Debug, Deserialize)]
pub struct MmuConfig {
    pub num_rows: usize,
    pub num_cols: usize,
    pub pe: PeConfig,
}

#[derive(Debug, Deserialize)]
pub struct PeConfig {

}

impl TpuConfig {
    pub fn from_file(path: impl AsRef<Path>) -> Result<Self, ConfigError> {
        let contents = std::fs::read_to_string(path)?;
        let config: TpuConfig = yaml_serde::from_str(&contents)?;

        config.validate()?;

        Ok(config)
    }

    pub fn validate(&self) -> Result<(), ConfigError> {
        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("failed to parse config: {0}")]
    Parse(#[from] yaml_serde::Error),

    #[error("failed to read config file: {0}")]
    Io(#[from] std::io::Error),
}