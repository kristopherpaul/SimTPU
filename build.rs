use anyhow::{Context, Result};

#[path = "src/config.rs"]
mod config;

fn main() -> Result<()> {
    const CONFIG_PATH: &str = "config/hardware.yaml";
    println!("cargo:rerun-if-changed={CONFIG_PATH}");

    let config = config::TpuConfig::from_file(CONFIG_PATH).context(format!("failed to load TPU config from {CONFIG_PATH}"))?;
    
    let out_dir = std::env::var("OUT_DIR").context("failed to get OUT_DIR")?;
    let out_path = std::path::Path::new(&out_dir).join("types.rs");

    let act_type = match config.mmu.pe.act_bitw {
        8 => "i8",
        16 => "i16",
        32 => "i32",
        64 => "i64",
        _ => unreachable!("validated by PeConfig::validate")
    };

    let psum_type = match config.mmu.pe.psum_bitw {
        8 => "i8",
        16 => "i16",
        32 => "i32",
        64 => "i64",
        _ => unreachable!("validated by PeConfig::validate")
    };

    let addr_type = match config.vmem.addr_bitw {
        8 => "u8",
        16 => "u16",
        32 => "u32",
        _ => unreachable!("validated by VmemConfig::validate")
    };

    let generated_code = format!(
        "pub const MMU_ROWS: usize = {};\n\
         pub const MMU_COLS: usize = {};\n\
         pub const VMEM_SIZE: usize = {};\n\
         \n\
         pub type PeAct = {};\n\
         pub type PePsum = {};\n\
         pub type VAddr = {};\n",
        config.mmu.num_rows,
        config.mmu.num_cols,
        config.vmem.size,
        act_type,
        psum_type,
        addr_type
    );

    std::fs::write(out_path, generated_code).context("failed to write generated types.rs")?;

    Ok(())
}