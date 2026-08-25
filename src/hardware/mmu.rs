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