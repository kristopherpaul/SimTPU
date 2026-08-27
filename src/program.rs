use std::fs;
use std::path::Path;
use crate::assembler::{assemble, AssemblerError};
use crate::isa::{Instruction, InstructionError};

#[derive(Debug)]
pub struct Program {
    instructions: Vec<Instruction>
}

impl Program {
    pub fn new(instructions: impl Into<Option<Vec<Instruction>>>) -> Self {
        Self {
            instructions: instructions.into().unwrap_or_default()
        }
    }

    pub fn load_file(&mut self, path: impl AsRef<Path>) -> Result<(), ProgramError> {
        let path = path.as_ref();

        match path.extension().and_then(|x| x.to_str()) {
            Some("asm") | Some("txt") => Ok(self.load_assembly(path)?),
            Some("bin") => Ok(self.load_binary(path)?),
            _ => Err(ProgramError::UnsupportedFormat)
        }
    }

    fn load_assembly(&mut self, path: &Path) -> Result<(), ProgramError> {
        let source = fs::read_to_string(path).map_err(ProgramError::ReadAsmFailed)?;
        let binary = assemble(&source)?;
        let binary_path = path.with_extension("bin");

        fs::write(&binary_path, &binary).map_err(ProgramError::WriteBinFailed)?;
        self.decode_binary(&binary)
    }

    fn load_binary(&mut self, path: &Path) -> Result<(), ProgramError> {
        let binary = fs::read(path).map_err(ProgramError::ReadBinFailed)?;
        self.decode_binary(&binary)
    }

    fn decode_binary(&mut self, mut binary: &[u8]) -> Result<(), ProgramError> {
        while !binary.is_empty() {
            self.instructions.push(Instruction::decode(&mut binary)?);
        }
        Ok(())
    }

    pub fn instructions(&self) -> &[Instruction] {
        &self.instructions
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ProgramError {
    #[error("unsupported program format")]
    UnsupportedFormat,

    #[error("failed to read program assembly")]
    ReadAsmFailed(#[source] std::io::Error),

    #[error("failed to write program binary")]
    WriteBinFailed(#[source] std::io::Error),

    #[error("failed to read program binary")]
    ReadBinFailed(#[source] std::io::Error),

    #[error("failed to assemble to binary")]
    AssembleFailed(#[from] AssemblerError),

    #[error("failed to decode instruction")]
    DecodeFailed(#[from] InstructionError)
}