use super::codec::CodecError;

macro_rules! isa {
    (
        $(
            $variant:ident {
                mnemonic: $mnemonic:literal,
                opcode: $opcode:expr,
                params: {
                    $( $param:ident : $ty:ty ),* $(,)?
                }
            }
        )*
    ) => {
        #[derive(Debug, Clone, PartialEq, Eq)]
        pub enum Instruction {
            $(
                $variant {
                    $( $param: $ty ),*
                }
            )*
        }

        impl Instruction {
            pub fn mnemonic(&self) -> &'static str {
                match self {
                    $(
                        Self::$variant { .. } => $mnemonic,
                    )*
                }
            }

            pub fn opcode(&self) -> u8 {
                match self {
                    $(
                        Self::$variant { .. } => $opcode,
                    )*
                }
            }

            pub fn encode(&self, out: &mut Vec<u8>) {
                out.push(self.opcode());

                match self {
                    $(
                        Self::$variant {
                            $( $param ),*
                        } => {
                            $(
                                $crate::isa::codec::ParamCodec::encode(*$param, out);
                            )*
                        }
                    )*
                }
            }

            pub fn decode(input: &mut &[u8]) -> Result<Self, $crate::isa::define::InstructionError> {
                let opcode = *input
                    .first()
                    .ok_or(
                        $crate::isa::define::InstructionError::UnexpectedEof
                    )?;

                *input = &input[1..];

                match opcode {
                    $(
                        $opcode => {
                            Ok(Self::$variant {
                                $(
                                    $param: 
                                        <$ty as $crate::isa::codec::ParamCodec>::decode(input)
                                        .map_err(|source| {
                                            $crate::isa::define::InstructionError::DecodeOperand {
                                                mnemonic: $mnemonic,
                                                source,
                                            }
                                        })?,
                                )*
                            })
                        }
                    )*

                    opcode => Err(
                        $crate::isa::define::InstructionError::UnknownOpcode(opcode)
                    ),
                }
            }

            pub fn parse_assembly(line: &str) -> Result<Self, $crate::isa::define::InstructionError> {
                let mut parts = line.split_whitespace();

                let mnemonic = parts
                    .next()
                    .ok_or(
                        $crate::isa::define::InstructionError::UnexpectedEof
                    )?;

                match mnemonic {
                    $(
                        $mnemonic => {
                            $(
                                let value = parts
                                    .next()
                                    .ok_or(
                                        $crate::isa::define::InstructionError::MissingOperand {
                                            mnemonic: $mnemonic,
                                            parameter: stringify!($param),
                                        }
                                    )?;

                                let $param =
                                    <$ty as $crate::isa::codec::ParamCodec>::parse(value)
                                    .map_err(|source| {
                                        $crate::isa::define::InstructionError::InvalidOperand {
                                            mnemonic: $mnemonic,
                                            source,
                                        }
                                    })?;
                            )*

                            if parts.next().is_some() {
                                return Err(
                                    $crate::isa::define::InstructionError::TooManyOperands {
                                        mnemonic: $mnemonic,
                                    }
                                );
                            }

                            Ok(Self::$variant {
                                $( $param ),*
                            })
                        }
                    )*

                    _ => Err(
                        $crate::isa::define::InstructionError::UnknownMnemonic(
                            mnemonic.to_owned()
                        )
                    ),
                }
            }
        }
    };
}

pub(crate) use isa;

#[derive(Debug, thiserror::Error)]
pub enum InstructionError {
    #[error("unexpected end of input while decoding instruction")]
    UnexpectedEof,

    #[error("unknown opcode: 0x{0:02x}")]
    UnknownOpcode(u8),

    #[error("unknown instruction `{0}`")]
    UnknownMnemonic(String),

    #[error("{mnemonic}: missing operand `{parameter}`")]
    MissingOperand {
        mnemonic: &'static str,
        parameter: &'static str,
    },

    #[error("{mnemonic}: too many operands")]
    TooManyOperands {
        mnemonic: &'static str,
    },

    #[error("{mnemonic}: invalid operand")]
    InvalidOperand {
        mnemonic: &'static str,
        #[source]
        source: CodecError,
    },

    #[error("{mnemonic}: failed to decode operand")]
    DecodeOperand {
        mnemonic: &'static str,
        #[source]
        source: CodecError,
    },
}