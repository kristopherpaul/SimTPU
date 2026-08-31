use crate::types::{PePsum, MMU_COLS};
use std::collections::VecDeque;

pub struct Acc<
    P = PePsum,
    const SIZE: usize = MMU_COLS
> {
    buffer: [VecDeque<P>; SIZE],
    out_data: [P; SIZE],
    out_valid: bool
}

pub struct AccStrobes {
    pub load: bool,
    pub reset: bool
}

pub struct AccInputs<
    P = PePsum,
    const SIZE: usize = MMU_COLS
> {
    pub in_data: [P; SIZE],
    pub strobes: AccStrobes
}

impl<P, const SIZE: usize> Acc<P, SIZE>
where
    P: Default + Copy
{
    pub fn new() -> Result<Self, AccError> {
        Ok(Self {
            buffer: std::array::from_fn(|index| VecDeque::from(vec![P::default(); SIZE-index-1])),
            out_data: [P::default(); SIZE],
            out_valid: false
        })
    }

    pub fn tick(&mut self, inputs: AccInputs<P, SIZE>) -> Result<(), AccError> {
        if inputs.strobes.reset {
            self.buffer = std::array::from_fn(|index| VecDeque::from(vec![P::default(); SIZE-index-1]));
            self.out_data = [P::default(); SIZE];
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

    pub fn out_data(&self) -> [P; SIZE] {
        self.out_data
    }

    pub fn out_valid(&self) -> bool {
        self.out_valid
    }
}

#[derive(Debug, thiserror::Error)]
pub enum AccError {

}