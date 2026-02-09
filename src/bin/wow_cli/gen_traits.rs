//! Generator for traits.rs from WoW CSV exports.
//!
//! Reads 21 Trait* CSV files from ~/Projects/wow/data/ and generates data/traits.rs
//! with PHF maps for the talent tree system (Dragonflight+).
//!
//! Generates: data/traits.rs
//!
//! Split across modules:
//!   - gen_traits_load: CSV parsing and raw data structures
//!   - gen_traits_emit: Code generation and PHF map writing

use std::fs::File;
use std::path::Path;

use super::gen_traits_emit;
use super::gen_traits_load;

pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    let data = gen_traits_load::load_all()?;
    let (tree_infos, node_infos) = gen_traits_emit::denormalize(&data);

    std::fs::create_dir_all("data")?;
    let output_path = Path::new("data/traits.rs");
    let mut out = File::create(output_path)?;

    gen_traits_emit::write_header(&mut out)?;
    gen_traits_emit::write_struct_defs(&mut out)?;
    gen_traits_emit::write_tree_db(&mut out, &tree_infos)?;
    gen_traits_emit::write_node_db(&mut out, &node_infos)?;
    gen_traits_emit::write_entry_db(&mut out, &data.entries)?;
    gen_traits_emit::write_definition_db(&mut out, &data.definitions)?;
    gen_traits_emit::write_cond_db(&mut out, &data.conds)?;
    gen_traits_emit::write_sub_tree_db(&mut out, &data.sub_trees)?;
    gen_traits_emit::write_currency_db(&mut out, &data.currencies)?;
    gen_traits_emit::write_tests(&mut out)?;

    println!("Output: {}", output_path.display());
    Ok(())
}
