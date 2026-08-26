use crate::types::PePsum;
use std::collections::VecDeque;

pub struct Fifo<W = PePsum> {
    queue: VecDeque<W>,
    out_weight: W,
    out_valid: bool
}

pub struct FifoStrobes {
    pub load_weight: bool,
    pub release_weight: bool,
    pub reset: bool
}

pub struct FifoInputs<W = PePsum> {
    pub weight: W,
    pub strobes: FifoStrobes
}

impl<W> Fifo<W>
where
    W: Default + Copy
{
    pub fn new() -> Result<Self, FifoError> {
        Ok(Self {
            queue: VecDeque::new(),
            out_weight: W::default(),
            out_valid: false
        })
    }

    pub fn tick(&mut self, inputs: FifoInputs<W>) -> Result<(), FifoError> {
        if inputs.strobes.reset {
            self.queue.clear();
            self.out_weight = W::default();
            self.out_valid = false;
        } else {
            if inputs.strobes.load_weight {
                self.queue.push_back(inputs.weight);
                self.out_valid = false;
            } else if inputs.strobes.release_weight {
                if let Some(weight) = self.queue.pop_front() {
                    self.out_weight = weight;
                    self.out_valid = true;
                } else {
                    return Err(FifoError::PopEmptyQueue);
                }
            } else {
                self.out_valid = false;
            }
        }
        
        Ok(())
    }

    pub fn out_weight(&self) -> W {
        self.out_weight
    }

    pub fn out_valid(&self) -> bool {
        self.out_valid
    }
}

#[derive(Debug, thiserror::Error)]
pub enum FifoError {
    #[error("tried to pop from empty FIFO queue")]
    PopEmptyQueue
}


#[cfg(test)]
mod tests {
    use super::*;

    type TestFifo = Fifo<u8>;

    fn inputs(weight: u8, load_weight: bool, release_weight: bool, reset: bool) -> FifoInputs<u8> {
        FifoInputs {
            weight,
            strobes: FifoStrobes {
                load_weight,
                release_weight,
                reset,
            },
        }
    }

    #[test]
    fn new_initializes_state_to_default() {
        let fifo = TestFifo::new().unwrap();
        assert!(fifo.queue.is_empty());
        assert_eq!(fifo.out_weight, 0);
        assert!(!fifo.out_valid);
    }

    #[test]
    fn releases_weights_in_fifo_order() {
        let mut fifo = TestFifo::new().unwrap();

        fifo.tick(inputs(3, true, false, false)).unwrap();
        fifo.tick(inputs(7, true, false, false)).unwrap();
        fifo.tick(inputs(0, false, true, false)).unwrap();
        assert_eq!(fifo.out_weight(), 3);
        assert!(fifo.out_valid());

        fifo.tick(inputs(0, false, true, false)).unwrap();
        assert_eq!(fifo.out_weight(), 7);
        assert!(fifo.out_valid());
    }

    #[test]
    fn load_deasserts_valid_output_and_idle_cycle_keeps_weight() {
        let mut fifo = TestFifo::new().unwrap();
        fifo.tick(inputs(5, true, false, false)).unwrap();
        fifo.tick(inputs(0, false, true, false)).unwrap();
        assert!(fifo.out_valid());

        fifo.tick(inputs(0, false, false, false)).unwrap();
        assert_eq!(fifo.out_weight(), 5);
        assert!(!fifo.out_valid());
    }

    #[test]
    fn load_has_priority_over_release() {
        let mut fifo = TestFifo::new().unwrap();
        fifo.tick(inputs(4, true, false, false)).unwrap();
        fifo.tick(inputs(9, true, true, false)).unwrap();
        assert!(!fifo.out_valid());

        fifo.tick(inputs(0, false, true, false)).unwrap();
        assert_eq!(fifo.out_weight(), 4);
        fifo.tick(inputs(0, false, true, false)).unwrap();
        assert_eq!(fifo.out_weight(), 9);
    }

    #[test]
    fn reset_clears_queue_and_output_state() {
        let mut fifo = TestFifo::new().unwrap();
        fifo.tick(inputs(6, true, false, false)).unwrap();
        fifo.tick(inputs(8, true, false, false)).unwrap();
        fifo.tick(inputs(0, false, true, false)).unwrap();

        fifo.tick(inputs(99, true, true, true)).unwrap();
        assert!(fifo.queue.is_empty());
        assert_eq!(fifo.out_weight(), 0);
        assert!(!fifo.out_valid());
    }

    #[test]
    fn releasing_empty_fifo_returns_error() {
        let mut fifo = TestFifo::new().unwrap();
        assert!(matches!(
            fifo.tick(inputs(0, false, true, false)),
            Err(FifoError::PopEmptyQueue)
        ));
    }
}