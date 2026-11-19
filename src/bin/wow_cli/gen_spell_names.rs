//! Generate the Retail spell-name lookup from a pinned DB2 export.

use super::csv_util::{escape_str, parse_csv_line, read_csv_records};
use std::fs::File;
use std::io::{BufReader, Write};
use std::path::Path;

const INPUT: &str = "data/db2/retail-12.1.5.70783/SpellName.csv";
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
    let header = records.first().ok_or("SpellName.csv has no header")?;
    let columns = find_columns(header)?;
    let mut rows = records
        .iter()
        .skip(1)
        .map(|row| parse_row(&columns, row))
        .collect::<Result<Vec<_>, _>>()?;
    rows.sort_unstable_by_key(|row| row.id);
    if rows.windows(2).any(|pair| pair[0].id == pair[1].id) {
        return Err("SpellName.csv has duplicate IDs".into());
    }
    rows.retain(|row| !row.name.is_empty());
    Ok(rows)
}

fn find_columns(header: &str) -> Result<Columns, String> {
    let fields = parse_csv_line(header);
    let find = |name: &str| {
        fields
            .iter()
            .position(|field| field == name)
            .ok_or_else(|| format!("SpellName.csv has no {name} column"))
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
        .ok_or("SpellName.csv row has no ID")?
        .parse()
        .map_err(|_| "SpellName.csv row has an invalid ID")?;
    let name = fields
        .get(columns.name)
        .ok_or("SpellName.csv row has no Name_lang")?;
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
    writeln!(output, "#[rustfmt::skip]")?;
    writeln!(
        output,
        "pub static SPELL_NAME_OVERRIDES_RETAIL: &[(u32, &str)] = &["
    )?;
    for row in rows {
        writeln!(output, "    ({}, \"{}\"),", row.id, escape_str(&row.name))?;
    }
    writeln!(output, "];")?;
    writeln!(
        output,
        "pub fn get_spell_name(id: u32) -> Option<&'static str> {{"
    )?;
    writeln!(output, "    let index = SPELL_NAME_OVERRIDES_RETAIL")?;
    writeln!(output, "        .binary_search_by_key(&id, |row| row.0)")?;
    writeln!(output, "        .ok()?;")?;
    writeln!(output, "    Some(SPELL_NAME_OVERRIDES_RETAIL[index].1)")?;
    writeln!(output, "}}")
}
