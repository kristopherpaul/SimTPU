use crate::types::{PePsum, MMU_COLS};
use super::fifo::{Fifo, FifoStrobes, FifoInputs, FifoError};

pub struct WeightFifo<
    W = PePsum,
    const CNT: usize = MMU_COLS
> {
    fifos: Vec<Fifo<W>>,
    out_weights: [W; CNT],
    out_valids: [bool; CNT]
}

pub struct WeightFifoStrobes<const CNT: usize = MMU_COLS> {
    pub load_weight: [bool; CNT],
    pub release_weight: [bool; CNT],
    pub reset: bool
}

pub struct WeightFifoInputs<
    W = PePsum,
    const CNT: usize = MMU_COLS
> {
    pub weights: [W; CNT],
    pub strobes: WeightFifoStrobes<CNT>
}

impl<W, const CNT: usize> WeightFifo<W, CNT>
where
    W: Default + Copy
{
    pub fn new() -> Result<Self, WeightFifoError> {
        let mut fifos = Vec::with_capacity(CNT);
        for _ in 0..CNT {
            fifos.push(Fifo::new()?);
        }

        Ok(Self {
            fifos,
            out_weights: [W::default(); CNT],
            out_valids: [false; CNT]
        })
    }

    pub fn tick(&mut self, inputs: WeightFifoInputs<W, CNT>) -> Result<(), WeightFifoError> {
        for i in 0..CNT {
            self.fifos[i].tick(FifoInputs {
                weight: inputs.weights[i],
                strobes: FifoStrobes {
                    load_weight: inputs.strobes.load_weight[i],
                    release_weight: inputs.strobes.release_weight[i],
                    reset: inputs.strobes.reset
                }
            })?;
            
            self.out_weights[i] = self.fifos[i].out_weight();
            self.out_valids[i] = self.fifos[i].out_valid();
        }
        
        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum WeightFifoError {
    #[error("fifo error: {0}")]
    Fifo(#[from] FifoError),
}