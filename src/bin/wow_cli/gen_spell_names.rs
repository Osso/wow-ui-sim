//! Generate the small Retail name-only spell supplement from a pinned DB2 export.

use super::csv_util::{escape_str, parse_csv_line, read_csv_records};
use std::fs::File;
use std::io::{BufReader, Write};
use std::path::Path;

const INPUT: &str = "data/db2/retail-12.1.5.70783/SpellNames.csv";
const OUTPUT: &str = "crates/sim-data/data/spell_name_overrides_retail.rs";

struct SpellName {
    id: u32,
    name: String,
}

struct Columns {
    id: usize,
    name: usize,
}

pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    let rows = load_names(Path::new(INPUT))?;
    let mut output = File::create(OUTPUT)?;
    write_generated(&mut output, &rows)?;
    println!("Generated {} Retail spell-name overrides", rows.len());
    println!("Output: {OUTPUT}");
    Ok(())
}

fn load_names(path: &Path) -> Result<Vec<SpellName>, Box<dyn std::error::Error>> {
    let records = read_csv_records(BufReader::new(File::open(path)?))?;
    let header = records.first().ok_or("SpellNames.csv has no header")?;
    let columns = find_columns(header)?;
    let mut rows = records
        .iter()
        .skip(1)
        .map(|row| parse_row(&columns, row))
        .collect::<Result<Vec<_>, _>>()?;
    rows.sort_unstable_by_key(|row| row.id);
    if rows.windows(2).any(|pair| pair[0].id == pair[1].id) {
        return Err("SpellNames.csv has duplicate IDs".into());
    }
    Ok(rows)
}

fn find_columns(header: &str) -> Result<Columns, String> {
    let fields = parse_csv_line(header);
    let find = |name: &str| {
        fields
            .iter()
            .position(|field| field == name)
            .ok_or_else(|| format!("SpellNames.csv has no {name} column"))
    };
    Ok(Columns {
        id: find("ID")?,
        name: find("Name_lang")?,
    })
}

fn parse_row(columns: &Columns, record: &str) -> Result<SpellName, String> {
    let fields = parse_csv_line(record);
    let id = fields
        .get(columns.id)
        .ok_or("SpellNames.csv row has no ID")?
        .parse()
        .map_err(|_| "SpellNames.csv row has an invalid ID")?;
    let name = fields
        .get(columns.name)
        .ok_or("SpellNames.csv row has no Name_lang")?;
    if name.is_empty() {
        return Err(format!("SpellNames.csv has an empty name for ID {id}"));
    }
    Ok(SpellName {
        id,
        name: name.clone(),
    })
}

fn write_generated(output: &mut File, rows: &[SpellName]) -> std::io::Result<()> {
    writeln!(
        output,
        "//! Generated from pinned SpellName DB2 rows; see SpellName.provenance.json."
    )?;
    writeln!(output, "//! VERIFIED (DB2 12.1.5.70783).")?;
    writeln!(
        output,
        "pub static SPELL_NAME_OVERRIDES_RETAIL: &[(u32, &str)] = &["
    )?;
    for row in rows {
        writeln!(output, "    ({}, \"{}\"),", row.id, escape_str(&row.name))?;
    }
    writeln!(output, "];\n")
}
