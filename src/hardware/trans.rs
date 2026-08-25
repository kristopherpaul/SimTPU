use crate::types::PePsum;
use std::cmp::PartialOrd;
use num_traits::WrappingMul;

pub enum TransFn<P = PePsum> {
    ReLU,
    LeakyReLU(P)
}

impl<P> Default for TransFn<P> {
    fn default() -> Self {
        TransFn::<P>::ReLU
    }
}

pub struct Trans<P = PePsum> {
    trans: TransFn<P>,
    out_act: P,
    out_valid: bool
}

pub struct TransStrobes {
    pub valid: bool,
    pub switch_transfn: bool,
    pub reset: bool
}

pub struct TransInputs<P = PePsum> {
    pub trans: TransFn<P>,
    pub preact: P,
    pub strobes: TransStrobes
}

impl<P> Trans<P>
where
    P: Default + PartialOrd<i32> + WrappingMul + Copy
{
    pub fn new() -> Result<Self, TransError> {
        Ok(Self {
            trans: TransFn::default(),
            out_act: P::default(),
            out_valid: false
        })
    }

    pub fn tick(&mut self, inputs: TransInputs<P>) -> Result<(), TransError> {
        if inputs.strobes.reset {
            self.out_act = P::default();
            self.out_valid = false;
        } else {
            if inputs.strobes.valid {
                self.out_act = self.transform(inputs.preact)?;
                self.out_valid = true;
            } else {
                self.out_valid = false;                
            }
            if inputs.strobes.switch_transfn {
                self.trans = inputs.trans;
            }
        }
        
        Ok(())
    }

    fn transform(&self, input: P) -> Result<P, TransError> {
        match self.trans {
            TransFn::ReLU => {
                if input > 0 {
                    Ok(input)
                } else {
                    Ok(P::default())
                }
            },
            TransFn::LeakyReLU(leak_factor) => {
                if input >= 0 {
                    Ok(input)
                } else {
                    Ok(input.wrapping_mul(&leak_factor))
                }
            }
        }
    }

    pub fn out_act(&self) -> P {
        self.out_act
    }

    pub fn out_valid(&self) -> bool {
        self.out_valid
    }
}

#[derive(Debug, thiserror::Error)]
pub enum TransError {

}