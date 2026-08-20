use crate::hardware::mmu::{Mmu, MmuError};

pub struct Tpu {
    mmu: Mmu,
}

impl Tpu {
    pub fn new() -> Result<Self, TpuError> {
        Ok(Self {
            mmu: Mmu::new()?,
        })
    }

    pub fn run(&mut self) -> Result<(), TpuError> {
        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum TpuError {
    #[error("mmu error: {0}")]
    Mmu(#[from] MmuError),
}