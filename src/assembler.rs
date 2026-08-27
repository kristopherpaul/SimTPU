use crate::isa::{Instruction, InstructionError};

pub fn assemble(source: &str) -> Result<Vec<u8>, AssemblerError> {
    let mut binary = Vec::new();

    for (line_no, raw_line) in source.lines().enumerate() {
        let line = raw_line
            .split_once('#')
            .map(|(code, _)| code)
            .unwrap_or(raw_line)
            .trim();
        
        if line.is_empty() {
            continue;
        }

        let instruction = Instruction::parse_assembly(line)
            .map_err(|source| AssemblerError::ParseError {line: line_no+1, source})?;
        instruction.encode(&mut binary);
    }

    Ok(binary)
}

#[derive(Debug, thiserror::Error)]
pub enum AssemblerError {
    #[error("failed to parse assembly at line {line}")]
    ParseError {
        line: usize,
        #[source]
        source: InstructionError
    },
}