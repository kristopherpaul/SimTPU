mod codec;
mod define;
pub use define::InstructionError;
use codec::impl_param_codec;
use define::isa;

impl_param_codec!(u8, u16, u32, u64, i8, i16, i32, i64);

isa! {
    LoadW {
        mnemonic: "LOAD_W",
        opcode: 0x10,
        params: {
            src_addr: u32,
            rows: u16,
            cols: u16
        }
    }

    LoadA {
        mnemonic: "LOAD_A",
        opcode: 0x11,
        params: {
            src_addr: u32,
            len: u16
        }
    }

    Matmul {
        mnemonic: "MATMUL",
        opcode: 0x12,
        params: {
            cycles: u32
        }
    }

    Bias {
        mnemonic: "BIAS",
        opcode: 0x13,
        params: {
            bias_addr: u32,
            len: u16
        }
    }

    Act {
        mnemonic: "ACT",
        opcode: 0x14,
        params: {
            fn_id: u8,
            param_addr: u32
        }
    }

    Store {
        mnemonic: "STORE",
        opcode: 0x15,
        params: {
            dst_addr: u32,
            len: u16
        }
    }

    Sync {
        mnemonic: "SYNC",
        opcode: 0x16,
        params: {}
    }
}