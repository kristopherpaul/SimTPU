mod codec;
mod define;
pub use define::InstructionError;
use codec::impl_param_codec;
use define::isa;

impl_param_codec!(u8, u16, u32, u64, i8, i16, i32, i64);

isa! {
    Matmul {
        mnemonic: "MATMUL",
        opcode: 0x01,
        params: {
            cycles: u32
        }
    }
}