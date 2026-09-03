use std::fs;
use std::path::Path;
use crate::assembler::{assemble, AssemblerError};
use crate::isa::{Instruction, InstructionError};
use crate::types::VAddr;

#[derive(Debug)]
pub struct Program {
    instructions: Vec<Instruction>,
    data: Vec<(VAddr, Vec<u8>)>
}

impl Program {
    pub fn new(instructions: impl Into<Option<Vec<Instruction>>>) -> Self {
        Self {
            instructions: instructions.into().unwrap_or_default(),
            data: Vec::new()
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

    fn decode_binary(&mut self, binary: &[u8]) -> Result<(), ProgramError> {
        if binary.len() < 8 {
            return Err(ProgramError::InvalidBinaryFormat);
        }

        if &binary[0..4] != b"STPU" {
            return Err(ProgramError::InvalidBinaryFormat);
        }

        let instr_byte_count = u32::from_le_bytes([
            binary[4], binary[5], binary[6], binary[7]
        ]) as usize;

        let total_expected = 8 + instr_byte_count;
        if binary.len() < total_expected {
            return Err(ProgramError::InvalidBinaryFormat);
        }

        let mut instr_slice = &binary[8..(8+instr_byte_count)];
        while !instr_slice.is_empty() {
            self.instructions.push(Instruction::decode(&mut instr_slice)?);
        }

        let mut offset = 8 + instr_byte_count;
        while offset < binary.len() {
            if offset + 8 > binary.len() {
                return Err(ProgramError::InvalidBinaryFormat);
            }

            let addr = u32::from_le_bytes([
                binary[offset], binary[offset + 1], binary[offset + 2], binary[offset + 3]
            ]) as VAddr;
            offset += 4;

            let len = u32::from_le_bytes([
                binary[offset], binary[offset + 1], binary[offset + 2], binary[offset + 3]
            ]) as usize;
            offset += 4;

            if offset + len > binary.len() {
                return Err(ProgramError::InvalidBinaryFormat);
            }

            let bytes = binary[offset..offset + len].to_vec();
            offset += len;

            self.data.push((addr, bytes));
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

    #[error("invalid binary format")]
    InvalidBinaryFormat,

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