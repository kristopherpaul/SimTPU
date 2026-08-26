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