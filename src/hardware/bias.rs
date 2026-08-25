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


#[cfg(test)]
mod tests {
    use super::*;

    type TestBias = Bias<u8, u16>;

    fn inputs(bias: u8, acc: u16, valid_in: bool, load_bias: bool, reset: bool) -> BiasInputs<u8, u16> {
        BiasInputs {
            bias,
            acc,
            strobes: BiasStrobes {
                valid_in,
                load_bias,
                reset,
            },
        }
    }

    #[test]
    fn new_initializes_state_to_default() {
        let bias = TestBias::new().unwrap();

        assert_eq!(bias.bias, 0);
        assert_eq!(bias.out_preact, 0);
        assert!(!bias.valid_out);
    }

    #[test]
    fn loading_bias_takes_effect_on_the_following_cycle() {
        let mut bias = TestBias::new().unwrap();

        bias.tick(inputs(5, 10, true, true, false)).unwrap();
        assert_eq!(bias.out_preact(), 10);
        assert!(bias.valid_out);

        bias.tick(inputs(0, 10, true, false, false)).unwrap();
        assert_eq!(bias.out_preact(), 15);
        assert!(bias.valid_out);
    }

    #[test]
    fn loaded_bias_is_used_for_each_valid_input_until_overwritten() {
        let mut bias = TestBias::new().unwrap();

        bias.tick(inputs(7, 0, false, true, false)).unwrap();
        bias.tick(inputs(0, 3, true, false, false)).unwrap();
        assert_eq!(bias.out_preact(), 10);

        bias.tick(inputs(0, 8, true, false, false)).unwrap();
        assert_eq!(bias.out_preact(), 15);

        bias.tick(inputs(2, 0, false, true, false)).unwrap();
        bias.tick(inputs(0, 8, true, false, false)).unwrap();
        assert_eq!(bias.out_preact(), 10);
    }

    #[test]
    fn invalid_input_deasserts_valid_output() {
        let mut bias = TestBias::new().unwrap();

        bias.tick(inputs(4, 3, false, true, false)).unwrap();
        bias.tick(inputs(0, 8, true, false, false)).unwrap();
        assert!(bias.valid_out);

        bias.tick(inputs(0, 99, false, false, false)).unwrap();
        assert!(!bias.valid_out);
        assert_eq!(bias.out_preact(), 12);
    }

    #[test]
    fn reset_has_priority_over_load_and_input() {
        let mut bias = TestBias::new().unwrap();

        bias.tick(inputs(9, 7, true, true, true)).unwrap();

        assert_eq!(bias.bias, 0);
        assert_eq!(bias.out_preact(), 0);
        assert!(!bias.valid_out);
    }
}