use crate::isa::{Instruction, InstructionError};

pub fn assemble(source: &str) -> Result<Vec<u8>, AssemblerError> {
    let mut instructions = Vec::new();
    let mut data = Vec::new();
    let mut current_data_addr: Option<u32> = None;

    for (line_no, raw_line) in source.lines().enumerate() {
        let line = raw_line
            .split_once('#')
            .map(|(code, _)| code)
            .unwrap_or(raw_line)
            .trim();
        
        if line.is_empty() {
            continue;
        }

        if line.starts_with(".data") {
            let addr_str = line.strip_prefix(".data").unwrap_or_default().trim();
            current_data_addr = Some(u32::from_str_radix(addr_str, 16)
                .map_err(|_| AssemblerError::InvalidDataAddress { line: line_no + 1 })?);
            continue;
        }

        if line.starts_with(".byte") {
            let addr = current_data_addr
                .ok_or_else(|| AssemblerError::DataWithoutAddress { line: line_no + 1 })?;
            
            let bytes_str = line.strip_prefix(".byte").unwrap_or_default().trim();
            let bytes: Vec<u8> = bytes_str
                .split(',')
                .map(|s| {
                    s.trim().parse::<i32>()
                        .map(|v| v as u8)
                        .map_err(|_| AssemblerError::InvalidByte { line: line_no + 1 })
                })
                .collect::<Result<_, _>>()?;
            
            data.push((addr, bytes));
            current_data_addr = None;
            continue;
        }

        let instruction = Instruction::parse_assembly(line)
            .map_err(|source| AssemblerError::ParseError {line: line_no+1, source})?;
        instruction.encode(&mut instructions);
    }

    // Encode binary format: Magic | InstrCount | Instructions | Data sections
    let mut binary = Vec::new();
    binary.extend_from_slice(b"STPU");
    binary.extend_from_slice(&(instructions.len() as u32).to_le_bytes());
    binary.extend_from_slice(&instructions);
    
    for (addr, bytes) in data {
        binary.extend_from_slice(&addr.to_le_bytes());
        binary.extend_from_slice(&(bytes.len() as u32).to_le_bytes());
        binary.extend_from_slice(&bytes);
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
    #[error("invalid data address at line {line}")]
    InvalidDataAddress { line: usize },
    #[error("data directive without preceding .data address at line {line}")]
    DataWithoutAddress { line: usize },
    #[error("invalid byte value at line {line}")]
    InvalidByte { line: usize },
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn assembles_matmul_with_little_endian_cycles() {
        let binary = assemble("MATMUL 305419896").unwrap();
        let expected = [
            b'S', b'T', b'P', b'U',                // Magic
            5, 0, 0, 0,                            // InstrCount = 5
            0x12, 0x78, 0x56, 0x34, 0x12           // Instructions
        ];
        assert_eq!(binary, expected);
    }

    #[test]
    fn ignores_blank_lines_and_comments() {
        let source = "\n# warm up\n  MATMUL 1  # one cycle\n\nMATMUL 2\n";
        let binary = assemble(source).unwrap();
        let expected = [
            b'S', b'T', b'P', b'U',                // Magic
            10, 0, 0, 0,                           // InstrCount = 10
            0x12, 0x01, 0x00, 0x00, 0x00,         // MATMUL 1
            0x12, 0x02, 0x00, 0x00, 0x00          // MATMUL 2
        ];
        assert_eq!(binary, expected);
    }

    #[test]
    fn reports_missing_operand_and_source_line() {
        let error = assemble("\n\nMATMUL").unwrap_err();
        assert!(matches!(
            error,
            AssemblerError::ParseError {
                line: 3,
                source: InstructionError::MissingOperand {
                    mnemonic: "MATMUL",
                    parameter: "cycles",
                },
            }
        ));
    }

    #[test]
    fn reports_invalid_operand_and_source_line() {
        let error = assemble("MATMUL nope").unwrap_err();
        assert!(matches!(
            error,
            AssemblerError::ParseError {
                line: 1,
                source: InstructionError::InvalidOperand {
                    mnemonic: "MATMUL",
                    ..
                },
            }
        ));
    }
}