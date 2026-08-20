use crate::types::{PeAct, PePsum};
use num_traits::{WrappingAdd, WrappingMul};

pub struct Pe<W = PeAct, A = PeAct, P = PePsum> {
    weight: W,
    out_act: A,
    out_psum: P
}

pub struct PeStrobes {
    pub load_weight: bool,
    pub reset: bool
}

pub struct PeInputs<W = PeAct, A = PeAct, P = PePsum> {
    pub weight: W,
    pub act: A,
    pub psum: P,
    pub strobes: PeStrobes
}

impl<W, A, P> Pe<W, A, P>
where
    W: Default + Into<P> + Copy,
    A: Default + Into<P> + Copy,
    P: Default + WrappingMul + WrappingAdd + Copy
{
    pub fn new() -> Result<Self, PeError> {
        Ok(Self {
            weight: W::default(),
            out_act: A::default(),
            out_psum: P::default()
        })
    }

    pub fn tick(&mut self, inputs: PeInputs<W, A, P>) -> Result<(), PeError> {
        if inputs.strobes.reset {
            self.weight = W::default();
            self.out_act = A::default();
            self.out_psum = P::default();
        } else if inputs.strobes.load_weight {
            self.weight = inputs.weight;
        } else {
            self.out_act = inputs.act;
            self.out_psum = inputs.act.into().wrapping_mul(&self.weight.into()).wrapping_add(&inputs.psum);
        }

        Ok(())
    }

    pub fn weight(&self) -> W {
        self.weight
    }

    pub fn out_act(&self) -> A {
        self.out_act
    }

    pub fn out_psum(&self) -> P {
        self.out_psum
    }
}

#[derive(Debug, thiserror::Error)]
pub enum PeError {
    
}