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


#[cfg(test)]
mod tests {
    use super::*;

    type TestTrans = Trans<i32>;

    fn inputs(trans: TransFn<i32>, preact: i32, valid: bool, switch_transfn: bool, reset: bool) -> TransInputs<i32> {
        TransInputs {
            trans,
            preact,
            strobes: TransStrobes {
                valid,
                switch_transfn,
                reset,
            },
        }
    }

    #[test]
    fn new_initializes_to_relu_with_invalid_output() {
        let trans = TestTrans::new().unwrap();
        assert_eq!(trans.out_act(), 0);
        assert!(!trans.out_valid());
    }

    #[test]
    fn relu_passes_positive_values_and_clamps_negative_values() {
        let mut trans = TestTrans::new().unwrap();
        trans.tick(inputs(TransFn::ReLU, 7, true, false, false)).unwrap();
        assert_eq!(trans.out_act(), 7);
        assert!(trans.out_valid());

        trans.tick(inputs(TransFn::ReLU, -3, true, false, false)).unwrap();
        assert_eq!(trans.out_act(), 0);
        assert!(trans.out_valid());
    }

    #[test]
    fn leaky_relu_passes_nonnegative_values_and_scales_negative_values() {
        let mut trans = TestTrans::new().unwrap();
        trans.tick(inputs(TransFn::LeakyReLU(2), 5, true, true, false)).unwrap();
        trans.tick(inputs(TransFn::LeakyReLU(2), 5, true, false, false)).unwrap();
        assert_eq!(trans.out_act(), 5);
        assert!(trans.out_valid());

        trans.tick(inputs(TransFn::LeakyReLU(2), -4, true, false, false)).unwrap();
        assert_eq!(trans.out_act(), -8);
        assert!(trans.out_valid());
    }

    #[test]
    fn switching_activation_takes_effect_on_the_following_cycle() {
        let mut trans = TestTrans::new().unwrap();
        trans.tick(inputs(TransFn::LeakyReLU(-2), -3, true, true, false)).unwrap();
        assert_eq!(trans.out_act(), 0);
        assert!(trans.out_valid());

        trans.tick(inputs(TransFn::ReLU, -3, true, false, false)).unwrap();
        assert_eq!(trans.out_act(), 6);
        assert!(trans.out_valid());
    }

    #[test]
    fn selected_activation_remains_active_until_switched() {
        let mut trans = TestTrans::new().unwrap();
        trans.tick(inputs(TransFn::LeakyReLU(-2), 0, false, true, false)).unwrap();
        trans.tick(inputs(TransFn::ReLU, -3, true, false, false)).unwrap();
        assert_eq!(trans.out_act(), 6);

        trans.tick(inputs(TransFn::ReLU, -3, true, false, false)).unwrap();
        assert_eq!(trans.out_act(), 6);
    }

    #[test]
    fn invalid_input_deasserts_valid_output_without_changing_value() {
        let mut trans = TestTrans::new().unwrap();
        trans.tick(inputs(TransFn::ReLU, 8, true, false, false)).unwrap();
        trans.tick(inputs(TransFn::ReLU, -99, false, false, false)).unwrap();

        assert_eq!(trans.out_act(), 8);
        assert!(!trans.out_valid());
    }

    #[test]
    fn reset_has_priority_over_switch_and_input() {
        let mut trans = TestTrans::new().unwrap();
        trans.tick(inputs(TransFn::LeakyReLU(-2), -3, true, true, true)).unwrap();
        assert_eq!(trans.out_act(), 0);
        assert!(!trans.out_valid());

        trans.tick(inputs(TransFn::ReLU, -3, true, false, false)).unwrap();
        assert_eq!(trans.out_act(), 0);
    }
}