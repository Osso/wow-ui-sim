//! Generate the Retail SpellName binary index from the pinned DB2 export.

use super::csv_util::{parse_csv_line, read_csv_records};
use std::fs::File;
use std::io::{BufReader, Write};
use std::path::Path;

const INPUT: &str = "data/db2/retail-12.1.5.70783/SpellName.csv";
const RUST_OUTPUT: &str = "crates/sim-data/data/spell_name_overrides_retail.rs";
const BINARY_OUTPUT: &str = "crates/sim-data/data/spell_names_retail.bin";
const MODULE_SOURCE: &str = r#"//! Generated Retail SpellName index; see `spell_names_retail.provenance.json`.
//! VERIFIED (DB2 12.1.5.70783); the data file stores all readable non-empty names.

const SPELL_NAME_DATA: &[u8] = include_bytes!("spell_names_retail.bin");
const HEADER_BYTES: usize = 4;
const ENTRY_BYTES: usize = 8;

pub fn get_spell_name(id: u32) -> Option<&'static str> {
    let count = read_u32(0)? as usize;
    let names_start = HEADER_BYTES.checked_add(count.checked_mul(ENTRY_BYTES)?)?;
    if names_start > SPELL_NAME_DATA.len() {
        return None;
    }
    let mut low = 0;
    let mut high = count;
    while low < high {
        let middle = low + (high - low) / 2;
        let entry = HEADER_BYTES + middle * ENTRY_BYTES;
        if read_u32(entry)? < id {
            low = middle + 1;
        } else {
            high = middle;
        }
    }
    if low == count {
        return None;
    }
    let entry = HEADER_BYTES + low * ENTRY_BYTES;
    if read_u32(entry)? != id {
        return None;
    }
    let name_offset = read_u32(entry + 4)? as usize;
    let name_start = names_start.checked_add(name_offset)?;
    let name_data = SPELL_NAME_DATA.get(name_start..)?;
    let name_end = name_data.iter().position(|byte| *byte == 0)?;
    std::str::from_utf8(&name_data[..name_end]).ok()
}

fn read_u32(offset: usize) -> Option<u32> {
    let end = offset.checked_add(4)?;
    let bytes = SPELL_NAME_DATA.get(offset..end)?;
    Some(u32::from_le_bytes(bytes.try_into().ok()?))
}

#[cfg(test)]
mod tests {
    use super::get_spell_name;
    use std::time::Instant;

    #[test]
    fn binary_name_lookup_matches_distinct_db2_rows() {
        let started = Instant::now();
        let heart = get_spell_name(275825);
        eprintln!("first_spell_name_lookup_ns={}", started.elapsed().as_nanos());
        assert_eq!(heart, Some("Heart of Azeroth"));
        assert_eq!(get_spell_name(133137), Some("Active"));
        assert_eq!(get_spell_name(u32::MAX), None);
    }
}
"#;

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
    let data = encode_name_index(&rows)?;
    std::fs::write(BINARY_OUTPUT, &data)?;
    File::create(RUST_OUTPUT)?.write_all(MODULE_SOURCE.as_bytes())?;
    println!("Generated {} names in {} bytes", rows.len(), data.len());
    println!("Outputs: {RUST_OUTPUT}, {BINARY_OUTPUT}");
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

fn encode_name_index(rows: &[SpellName]) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let count = u32::try_from(rows.len())?;
    let mut index = Vec::with_capacity(rows.len() * 8);
    let mut names = Vec::new();
    for row in rows {
        let offset = u32::try_from(names.len())?;
        index.extend_from_slice(&row.id.to_le_bytes());
        index.extend_from_slice(&offset.to_le_bytes());
        names.extend_from_slice(row.name.as_bytes());
        names.push(0);
    }
    let mut data = Vec::with_capacity(4 + index.len() + names.len());
    data.extend_from_slice(&count.to_le_bytes());
    data.extend(index);
    data.extend(names);
    Ok(data)
}
