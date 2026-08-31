use crate::types::{PeAct, MMU_ROWS};
use std::collections::VecDeque;

pub struct SkewBuffer<
    A = PeAct,
    const SIZE: usize = MMU_ROWS
> {
    buffer: [VecDeque<A>; SIZE],
    out_data: [A; SIZE],
    out_valid: bool
}

pub struct SkewBufferStrobes {
    pub load: bool,
    pub reset: bool
}

pub struct SkewBufferInputs<
    A = PeAct,
    const SIZE: usize = MMU_ROWS
> {
    pub in_data: [A; SIZE],
    pub strobes: SkewBufferStrobes
}

impl<A, const SIZE: usize> SkewBuffer<A, SIZE>
where
    A: Default + Copy
{
    pub fn new() -> Result<Self, SkewBufferError> {
        Ok(Self {
            buffer: std::array::from_fn(|index| VecDeque::from(vec![A::default(); index])),
            out_data: [A::default(); SIZE],
            out_valid: false
        })
    }

    pub fn tick(&mut self, inputs: SkewBufferInputs<A>) -> Result<(), SkewBufferError> {
        if inputs.strobes.reset {
            self.buffer = std::array::from_fn(|index| VecDeque::from(vec![A::default(); index]));
            self.out_data = [A::default(); SIZE];
            self.out_valid = false;
        } else if inputs.strobes.load {
            self.buffer.iter_mut()
                       .enumerate()
                       .for_each(|(index, vec)| vec.push_back(inputs.in_data[index]));
            self.out_data.iter_mut()
                         .enumerate()
                         .for_each(|(index, slot)| *slot = self.buffer[index].pop_front().unwrap_or_default());
            self.out_valid = true;
        } else {
            self.out_valid = false;
        }
        
        Ok(())
    }

    pub fn out_data(&self) -> [A; SIZE] {
        self.out_data
    }

    pub fn out_valid(&self) -> bool {
        self.out_valid
    }
}

#[derive(Debug, thiserror::Error)]
pub enum SkewBufferError {

}