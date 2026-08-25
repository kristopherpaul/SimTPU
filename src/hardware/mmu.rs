use super::pe::{Pe, PeError, PeInputs, PeStrobes};
use crate::types::{PeAct, PePsum, MMU_ROWS, MMU_COLS};
use num_traits::{WrappingAdd, WrappingMul};

pub struct Mmu<
    W = PeAct,
    A = PeAct,
    P = PePsum,
    const ROWS: usize = MMU_ROWS,
    const COLS: usize = MMU_COLS
> {
    pes: [[Pe<W, A, P>; COLS]; ROWS],
    out_acc: [P; COLS]
}

struct MmuStrobes<
    const COLS: usize = MMU_COLS
> {
    load_weight: [bool; COLS],
    reset: bool
}

struct MmuInputs<
    W = PeAct,
    A = PeAct,
    const ROWS: usize = MMU_ROWS,
    const COLS: usize = MMU_COLS
> {
    weights: [W; COLS],
    acts: [A; ROWS],
    strobes: MmuStrobes<COLS>
}

impl<W, A, P, const ROWS: usize, const COLS: usize> Mmu<W, A, P, ROWS, COLS>
where
    W: Default + Into<P> + Copy,
    A: Default + Into<P> + Copy,
    P: Default + WrappingMul + WrappingAdd + Copy
{
    pub fn new() -> Result<Self, MmuError> {
        Ok(Self {
            pes: [[Pe::new()?; COLS]; ROWS],
            out_acc: [P::default(); COLS]
        })
    }

    pub fn tick(&mut self, inputs: MmuInputs<W, A>) -> Result<(), MmuError> {
        for i in (0..self.pes.len()).rev() {
            for j in (0..self.pes[i].len()).rev() {
                let weight: W = match i {
                    0 => inputs.weights[j],
                    _ => self.pes[i-1][j].weight()
                };
                let psum: P = match i {
                    0 => P::default(),
                    _ => self.pes[i-1][j].out_psum()
                };
                let act = match j {
                    0 => inputs.acts[i],
                    _ => self.pes[i][j-1].out_act()
                };
                
                self.pes[i][j].tick(PeInputs {
                    weight,
                    act,
                    psum,
                    strobes: PeStrobes {
                        load_weight: inputs.strobes.load_weight[j],
                        reset: inputs.strobes.reset
                    }
                })?;
            }
        }
        
        self.out_acc = std::array::from_fn(|j| {
            self.pes[ROWS-1][j].out_psum()
        });

        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum MmuError {
    #[error("pe error: {0}")]
    Pe(#[from] PeError),
}


#[cfg(test)]
mod tests {
    use super::*;

    type TestMmu = Mmu<u8, u8, u16, 2, 2>;

    fn inputs(
        weights: [u8; 2],
        acts: [u8; 2],
        load_weight: [bool; 2],
        reset: bool,
    ) -> MmuInputs<u8, u8, 2, 2> {
        MmuInputs {
            weights,
            acts,
            strobes: MmuStrobes {
                load_weight,
                reset,
            },
        }
    }

    #[test]
    fn new_initializes_pes_and_outputs_to_default() {
        let mmu = TestMmu::new().unwrap();

        assert_eq!(mmu.pes[0][0].weight(), 0);
        assert_eq!(mmu.pes[1][1].out_psum(), 0);
        assert_eq!(mmu.out_acc, [0, 0]);
    }

    #[test]
    fn staggered_activations_and_weights_produce_expected_outputs() {
        let mut mmu = TestMmu::new().unwrap();

        mmu.tick(inputs([2, 0], [0, 0], [true, false], false)).unwrap();
        assert_eq!(mmu.pes[0][0].weight(), 2);  assert_eq!(mmu.pes[0][1].weight(), 0);
        assert_eq!(mmu.pes[1][0].weight(), 0);  assert_eq!(mmu.pes[1][1].weight(), 0);
        mmu.tick(inputs([0, 7], [3, 0], [false, true], false)).unwrap();
        assert_eq!(mmu.pes[0][0].weight(), 2);  assert_eq!(mmu.pes[0][1].weight(), 7);
        assert_eq!(mmu.pes[1][0].weight(), 0);  assert_eq!(mmu.pes[1][1].weight(), 0);
        mmu.tick(inputs([0, 0], [5, 0], [false, false], false)).unwrap();
        assert_eq!(mmu.pes[0][0].weight(), 2);  assert_eq!(mmu.pes[0][1].weight(), 7);
        assert_eq!(mmu.pes[1][0].weight(), 0);  assert_eq!(mmu.pes[1][1].weight(), 0);
        assert_eq!(mmu.out_acc, [6, 0]);
        mmu.tick(inputs([0, 0], [0, 0], [false, false], false)).unwrap();
        assert_eq!(mmu.out_acc, [10, 21]);
        mmu.tick(inputs([0, 0], [0, 0], [false, false], false)).unwrap();
        assert_eq!(mmu.out_acc, [0, 35]);
    }

    #[test]
    fn reset_clears_pes_and_outputs() {
        let mut mmu = TestMmu::new().unwrap();
        mmu.tick(inputs([2, 3], [0, 0], [true, true], false)).unwrap();
        mmu.tick(inputs([0, 0], [4, 5], [false, false], false)).unwrap();
        mmu.tick(inputs([0, 0], [0, 0], [false, false], true)).unwrap();

        assert_eq!(mmu.pes[0][0].weight(), 0);
        assert_eq!(mmu.pes[1][1].out_psum(), 0);
        assert_eq!(mmu.out_acc, [0, 0]);
    }
}