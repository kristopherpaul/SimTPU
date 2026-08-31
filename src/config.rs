use std::path::Path;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct TpuConfig {
    pub mmu: MmuConfig,
    pub vmem: VmemConfig
}

impl TpuConfig {
    pub fn from_file(path: impl AsRef<Path>) -> Result<Self, TpuConfigError> {
        let contents = std::fs::read_to_string(path)?;
        let config: TpuConfig = yaml_serde::from_str(&contents)?;

        config.validate()?;

        Ok(config)
    }

    pub fn validate(&self) -> Result<(), TpuConfigError> {
        self.mmu.validate()?;
        self.vmem.validate()?;

        Ok(())
    }
}

#[derive(Debug, Deserialize)]
pub struct MmuConfig {
    pub num_rows: usize,
    pub num_cols: usize,
    pub pe: PeConfig,
}

impl MmuConfig {
    pub fn validate(&self) -> Result<(), MmuConfigError> {
        if self.num_rows == 0 || self.num_cols == 0 {
            return Err(MmuConfigError::InvalidDimensions {
                rows: self.num_rows,
                cols: self.num_cols,
            });
        }

        self.pe.validate()?;

        Ok(())
    }
}

#[derive(Debug, Deserialize)]
pub struct PeConfig {
    pub act_bitw: u8,
    pub psum_bitw: u8
}

impl PeConfig {
    pub fn validate(&self) -> Result<(), PeConfigError> {
        match self.act_bitw {
            8 | 16 | 32 | 64 => {},
            _ => return Err(PeConfigError::InvalidActBitw(self.act_bitw)),
        }

        match self.psum_bitw {
            8 | 16 | 32 | 64 => {},
            _ => return Err(PeConfigError::InvalidPsumBitw(self.psum_bitw)),
        }

        if self.psum_bitw < self.act_bitw {
            return Err(PeConfigError::IncompatibleBitw {
                act_bitw: self.act_bitw,
                psum_bitw: self.psum_bitw,
            });
        }

        Ok(())
    }
}

#[derive(Debug, Deserialize)]
pub struct VmemConfig {
    pub size: usize
}

impl VmemConfig {
    pub fn validate(&self) -> Result<(), VmemConfigError> {
        if self.size == 0 {
            return Err(VmemConfigError::InvalidSize(self.size));
        }

        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum TpuConfigError {
    #[error("failed to parse config: {0}")]
    Parse(#[from] yaml_serde::Error),

    #[error("failed to read config file: {0}")]
    Io(#[from] std::io::Error),

    #[error("mmu config error: {0}")]
    MmuConfig(#[from] MmuConfigError),

    #[error("vmem config error: {0}")]
    VmemConfig(#[from] VmemConfigError)
}

#[derive(Debug, thiserror::Error)]
pub enum MmuConfigError {
    #[error("invalid systolic array dimensions: {rows}x{cols}")]
    InvalidDimensions {rows: usize, cols: usize},

    #[error("pe config error: {0}")]
    PeConfig(#[from] PeConfigError),
}

#[derive(Debug, thiserror::Error)]
pub enum PeConfigError {
    #[error("unsupported activation bit width: {0}")]
    InvalidActBitw(u8),

    #[error("unsupported partial sum bit width: {0}")]
    InvalidPsumBitw(u8),

    #[error("incompatible bit widths: activation {act_bitw} > partial sum {psum_bitw}")]
    IncompatibleBitw {act_bitw: u8, psum_bitw: u8},
}

#[derive(Debug, thiserror::Error)]
pub enum VmemConfigError {
    #[error("invalid vmem size: {0}")]
    InvalidSize(usize),
}