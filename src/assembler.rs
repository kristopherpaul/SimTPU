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


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn assembles_matmul_with_little_endian_cycles() {
        let binary = assemble("MATMUL 305419896").unwrap();
        assert_eq!(binary, vec![0x01, 0x78, 0x56, 0x34, 0x12]);
    }

    #[test]
    fn ignores_blank_lines_and_comments() {
        let source = "\n# warm up\n  MATMUL 1  # one cycle\n\nMATMUL 2\n";
        let binary = assemble(source).unwrap();
        assert_eq!(binary, vec![0x01, 0x01, 0x00, 0x00, 0x00, 0x01, 0x02, 0x00, 0x00, 0x00]);
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