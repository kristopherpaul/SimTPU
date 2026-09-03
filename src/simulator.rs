use crate::hardware::mmu::{Mmu, MmuError};
use crate::isa::Instruction;
use crate::program::Program;

pub struct Tpu {
    mmu: Mmu,
}

impl Tpu {
    pub fn new() -> Result<Self, TpuError> {
        Ok(Self {
            mmu: Mmu::new()?,
        })
    }

    pub fn run(&mut self, program: &Program) -> Result<(), TpuError> {
        for instruction in program.instructions() {
            match instruction {
                Instruction::Matmul { cycles } => {
                    println!("Executing Matmul instruction for {} cycles", cycles);
                }
                _ => println!("Instruction is not implemented")
            }
        }
        
        Ok(())
    }

    fn tick(&mut self) -> Result<(), TpuError> {
        //self.mmu.tick()?;
        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum TpuError {
    #[error("mmu error: {0}")]
    Mmu(#[from] MmuError),
}