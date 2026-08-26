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


#[cfg(test)]
mod tests {
    use super::*;

    type TestWeightFifo = WeightFifo<u8, 3>;

    fn inputs(weights: [u8; 3], load_weight: [bool; 3], release_weight: [bool; 3], reset: bool) -> WeightFifoInputs<u8, 3> {
        WeightFifoInputs {
            weights,
            strobes: WeightFifoStrobes {
                load_weight,
                release_weight,
                reset,
            },
        }
    }

    #[test]
    fn new_initializes_all_fifos_and_outputs_to_default() {
        let weight_fifo = TestWeightFifo::new().unwrap();
        assert_eq!(weight_fifo.fifos.len(), 3);
        assert_eq!(weight_fifo.out_weights, [0, 0, 0]);
        assert_eq!(weight_fifo.out_valids, [false, false, false]);
    }

    #[test]
    fn loads_and_releases_weights_per_lane() {
        let mut weight_fifo = TestWeightFifo::new().unwrap();
        weight_fifo
            .tick(inputs([10, 20, 30], [true, true, true], [false, false, false], false))
            .unwrap();
        weight_fifo
            .tick(inputs([0, 0, 0], [false, false, false], [true, false, true], false))
            .unwrap();

        assert_eq!(weight_fifo.out_weights, [10, 0, 30]);
        assert_eq!(weight_fifo.out_valids, [true, false, true]);
    }

    #[test]
    fn each_lane_preserves_fifo_order_independently() {
        let mut weight_fifo = TestWeightFifo::new().unwrap();

        weight_fifo
            .tick(inputs([1, 10, 100], [true, true, true], [false, false, false], false))
            .unwrap();
        weight_fifo
            .tick(inputs([2, 20, 200], [true, true, true], [false, false, false], false))
            .unwrap();
        weight_fifo
            .tick(inputs([0, 0, 0], [false, false, false], [true, true, true], false))
            .unwrap();
        assert_eq!(weight_fifo.out_weights, [1, 10, 100]);

        weight_fifo
            .tick(inputs([0, 0, 0], [false, false, false], [true, true, true], false))
            .unwrap();
        assert_eq!(weight_fifo.out_weights, [2, 20, 200]);
    }

    #[test]
    fn reset_clears_all_fifos_and_outputs() {
        let mut weight_fifo = TestWeightFifo::new().unwrap();
        weight_fifo
            .tick(inputs([4, 5, 6], [true, true, true], [false, false, false], false))
            .unwrap();
        weight_fifo
            .tick(inputs([0, 0, 0], [false, false, false], [true, true, true], false))
            .unwrap();

        weight_fifo
            .tick(inputs([9, 9, 9], [true, true, true], [true, true, true], true))
            .unwrap();

        assert_eq!(weight_fifo.out_weights, [0, 0, 0]);
        assert_eq!(weight_fifo.out_valids, [false, false, false]);
        assert!(matches!(
            weight_fifo.tick(inputs([0, 0, 0], [false, false, false], [true, false, false], false)),
            Err(WeightFifoError::Fifo(FifoError::PopEmptyQueue))
        ));
    }

    #[test]
    fn releasing_empty_lane_returns_wrapped_fifo_error() {
        let mut weight_fifo = TestWeightFifo::new().unwrap();
        assert!(matches!(
            weight_fifo.tick(inputs([0, 0, 0], [false, false, false], [false, true, false], false)),
            Err(WeightFifoError::Fifo(FifoError::PopEmptyQueue))
        ));
    }
}