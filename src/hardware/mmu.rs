use super::pe::{Pe, PeError};
use crate::config::MmuConfig;

pub struct Mmu {
    num_rows: usize,
    num_cols: usize,
    pe: Pe,
}

impl Mmu {
    pub fn new(config: MmuConfig) -> Result<Self, MmuError> {
        Ok(Self {
            num_rows: config.num_rows,
            num_cols: config.num_cols,
            pe: Pe::new(config.pe)?,
        })
    }
}

#[derive(Debug, thiserror::Error)]
pub enum MmuError {
    #[error("invalid systolic array dimensions: {rows}x{cols}")]
    InvalidDimensions {rows: usize, cols: usize},

    #[error("pe error: {0}")]
    Pe(#[from] PeError),
}