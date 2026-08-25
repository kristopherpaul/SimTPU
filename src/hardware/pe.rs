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


#[cfg(test)]
mod tests {
    use super::*;
    type TestPe = Pe<u8, u8, u8>;

    fn inputs(weight: u8, act: u8, psum: u8, load_weight: bool, reset: bool) -> PeInputs<u8, u8, u8> {
        PeInputs {
            weight,
            act,
            psum,
            strobes: PeStrobes { load_weight, reset },
        }
    }

    #[test]
    fn new_initializes_to_default() {
        let pe = TestPe::new().unwrap();
        assert_eq!(pe.weight, 0);
        assert_eq!(pe.out_act, 0);
        assert_eq!(pe.out_psum, 0);
    }

    #[test]
    fn compute_updates_activation_and_psum() {
        let mut pe = TestPe::new().unwrap();
        pe.tick(inputs(5, 0, 0, true, false)).unwrap();
        pe.tick(inputs(0, 3, 7, false, false)).unwrap();

        assert_eq!(pe.weight, 5);
        assert_eq!(pe.out_act, 3);
        assert_eq!(pe.out_psum, 22); // 3 * 5 + 7
    }

    #[test]
    fn multiple_compute_cycles_use_loaded_weight() {
        let mut pe = TestPe::new().unwrap();
        pe.tick(inputs(4, 0, 0, true, false)).unwrap();
        pe.tick(inputs(0, 2, 1, false, false)).unwrap();
        assert_eq!(pe.out_psum, 9);
        pe.tick(inputs(0, 3, 10, false, false)).unwrap();
        assert_eq!(pe.out_psum, 22);
        assert_eq!(pe.weight, 4);
    }

    #[test]
    fn reset_clears_all_state() {
        let mut pe = TestPe::new().unwrap();
        pe.tick(inputs(5, 2, 3, true, false)).unwrap();
        pe.tick(inputs(0, 4, 6, false, false)).unwrap();
        pe.tick(inputs(0, 0, 0, false, true)).unwrap();

        assert_eq!(pe.weight, 0);
        assert_eq!(pe.out_act, 0);
        assert_eq!(pe.out_psum, 0);
    }

    #[test]
    fn reset_has_priority_over_load_weight() {
        let mut pe = TestPe::new().unwrap();
        pe.tick(inputs(7, 0, 0, true, true)).unwrap();

        assert_eq!(pe.weight, 0);
        assert_eq!(pe.out_act, 0);
        assert_eq!(pe.out_psum, 0);
    }

    #[test]
    fn compute_uses_wrapping_arithmetic() {
        let mut pe = TestPe::new().unwrap();
        pe.tick(inputs(250, 0, 0, true, false)).unwrap();
        pe.tick(inputs(0, 2, 20, false, false)).unwrap();
        // 250 * 2 + 20 = 520 ≡ 8 (mod 256)
        assert_eq!(pe.out_psum, 8);
    }
}