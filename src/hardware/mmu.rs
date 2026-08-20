use super::pe::{Pe, PeError};
use crate::types::{MMU_ROWS, MMU_COLS};

pub struct Mmu {
    num_rows: usize,
    num_cols: usize,
    pe: Pe,
}

impl Mmu {
    pub fn new() -> Result<Self, MmuError> {
        Ok(Self {
            num_rows: MMU_ROWS,
            num_cols: MMU_COLS,
            pe: Pe::new()?,
        })
    }
}

#[derive(Debug, thiserror::Error)]
pub enum MmuError {
    #[error("pe error: {0}")]
    Pe(#[from] PeError),
}