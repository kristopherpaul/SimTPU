pub trait ParamCodec: Sized {
    fn encode(self, out: &mut Vec<u8>);
    fn decode(input: &mut &[u8]) -> Result<Self, CodecError>;
    fn parse(s: &str) -> Result<Self, CodecError>;
}

macro_rules! impl_param_codec {
    ($($ty:ty),* $(,)?) => {
        $(
            impl $crate::isa::codec::ParamCodec for $ty {
                fn encode(self, out: &mut Vec<u8>) {
                    out.extend_from_slice(&self.to_le_bytes());
                }

                fn decode(input: &mut &[u8]) -> Result<Self, $crate::isa::codec::CodecError> {
                    const SIZE: usize = std::mem::size_of::<$ty>();

                    if input.len() < SIZE {
                        return Err(
                            $crate::isa::codec::CodecError::UnexpectedEof {
                                ty: stringify!($ty),
                            }
                        );
                    }

                    let (head, tail) = input.split_at(SIZE);
                    *input = tail;

                    Ok(<$ty>::from_le_bytes(
                        head.try_into().expect("slice length was checked"),
                    ))
                }

                fn parse(s: &str) -> Result<Self, $crate::isa::codec::CodecError> {
                    let value = if let Some(hex) = s.strip_prefix("0x") {
                        <$ty>::from_str_radix(hex, 16)
                    } else if let Some(hex) = s.strip_prefix("0X") {
                        <$ty>::from_str_radix(hex, 16)
                    } else {
                        s.parse::<$ty>()
                    };

                    value.map_err(|_| {
                        $crate::isa::codec::CodecError::InvalidValue {
                            ty: stringify!($ty),
                            value: s.to_owned(),
                        }
                    })
                }
            }
        )*
    };
}

pub(crate) use impl_param_codec;

#[derive(Debug, thiserror::Error)]
pub enum CodecError {
    #[error("unexpected end of input while decoding {ty}")]
    UnexpectedEof {
        ty: &'static str,
    },

    #[error("invalid {ty} value `{value}`")]
    InvalidValue {
        ty: &'static str,
        value: String,
    },
}