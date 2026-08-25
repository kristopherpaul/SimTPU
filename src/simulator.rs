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
        self.tick()?;
        Ok(())
    }

    fn tick(&mut self) -> Result<(), TpuError> {
        //self.mmu.tick()?;
        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum TpuError {
    #[error("mmu error: {0}")]
    Mmu(#[from] MmuError),
}