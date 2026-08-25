use crate::types::PePsum;
use num_traits::WrappingAdd;

pub struct Bias<B = PePsum, P = PePsum> {
    bias: B,
    out_preact: P,
    valid_out: bool
}

pub struct BiasStrobes {
    pub valid_in: bool,
    pub load_bias: bool,
    pub reset: bool
}

pub struct BiasInputs<B = PePsum, P = PePsum> {
    pub bias: B,
    pub acc: P,
    pub strobes: BiasStrobes
}

impl<B, P> Bias<B, P>
where
    B: Default + Into<P> + Copy,
    P: Default + WrappingAdd + Copy
{
    pub fn new() -> Result<Self, BiasError> {
        Ok(Self {
            bias: B::default(),
            out_preact: P::default(),
            valid_out: false
        })
    }

    pub fn tick(&mut self, inputs: BiasInputs<B, P>) -> Result<(), BiasError> {
        if inputs.strobes.reset {
            self.out_preact = P::default();
            self.valid_out = false;
        } else {
            if inputs.strobes.valid_in {
                self.out_preact = inputs.acc.wrapping_add(&self.bias.into());
                self.valid_out = true;
            } else {
                self.valid_out = false;
            }
            if inputs.strobes.load_bias {
                self.bias = inputs.bias;
            }
        }
        
        Ok(())
    }

    pub fn out_preact(&self) -> P {
        self.out_preact
    }
}

#[derive(Debug, thiserror::Error)]
pub enum BiasError {

}