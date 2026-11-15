//! Generate the Retail AreaTable ID and unsigned-16 lookup from the pinned DB2 CSV.

use super::csv_util::{parse_csv_line, read_csv_records};
use std::fmt::Display;
use std::fs::File;
use std::io::{BufReader, Write};
use std::path::Path;
use std::str::FromStr;

const AREA_TABLE_CSV: &str = "data/db2/retail-12.1.5.70783/AreaTable.csv";
const AREA_TABLE_OUTPUT: &str = "crates/sim-data-profile/data/area_table_retail.rs";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct AreaTableRow {
    id: u32,
    continent_id: u16,
}

struct Columns {
    id: usize,
    continent_id: usize,
}

pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    let rows = load_rows(Path::new(AREA_TABLE_CSV))?;
    let mut output = File::create(AREA_TABLE_OUTPUT)?;
    write_generated(&mut output, &rows)?;
    println!("Generated {} Retail AreaTable rows", rows.len());
    println!("Output: {AREA_TABLE_OUTPUT}");
    Ok(())
}

fn load_rows(path: &Path) -> Result<Vec<AreaTableRow>, Box<dyn std::error::Error>> {
    let file = File::open(path)?;
    let records = read_csv_records(BufReader::new(file))?;
    let header = records.first().ok_or("AreaTable.csv has no header")?;
    let columns = find_columns(header).map_err(|error| format!("{}: {error}", path.display()))?;
    let mut rows = records
        .iter()
        .enumerate()
        .skip(1)
        .map(|(index, record)| parse_row(&columns, record, index + 1))
        .collect::<Result<Vec<_>, _>>()?;
    rows.sort_unstable_by_key(|row| row.id);
    reject_duplicate_ids(&rows)?;
    Ok(rows)
}

fn find_columns(header: &str) -> Result<Columns, String> {
    let fields = parse_csv_line(header);
    let find = |name: &str| {
        fields
            .iter()
            .position(|field| field == name)
            .ok_or_else(|| format!("AreaTable.csv has no {name} column"))
    };
    Ok(Columns {
        id: find("ID")?,
        continent_id: find("ContinentID")?,
    })
}

fn parse_row(columns: &Columns, record: &str, row: usize) -> Result<AreaTableRow, String> {
    let fields = parse_csv_line(record);
    Ok(AreaTableRow {
        id: parse_cell(&fields, columns.id, "ID", row)?,
        continent_id: parse_cell(&fields, columns.continent_id, "ContinentID", row)?,
    })
}

fn parse_cell<T>(fields: &[String], column: usize, name: &str, row: usize) -> Result<T, String>
where
    T: FromStr,
    T::Err: Display,
{
    let value = fields
        .get(column)
        .ok_or_else(|| format!("AreaTable.csv row {row}: missing {name} (short row)"))?;
    value
        .parse()
        .map_err(|_| format!("AreaTable.csv row {row}: invalid {name}: {value:?}"))
}

fn reject_duplicate_ids(rows: &[AreaTableRow]) -> Result<(), String> {
    if rows.windows(2).any(|pair| pair[0].id == pair[1].id) {
        return Err("AreaTable.csv has duplicate IDs".into());
    }
    Ok(())
}

fn write_generated(out: &mut File, rows: &[AreaTableRow]) -> std::io::Result<()> {
    write_header(out)?;
    write_rows(out, rows)?;
    write_lookup(out)
}

fn write_header(out: &mut File) -> std::io::Result<()> {
    writeln!(
        out,
        "//! Auto-generated Retail AreaTable lookup from the pinned DB2 CSV."
    )?;
    writeln!(
        out,
        "//! VERIFIED source: data/db2/retail-12.1.5.70783/provenance.json."
    )?;
    writeln!(
        out,
        "//! INFERRED: the native AreaTable u16 result is the DBD ContinentID field."
    )?;
    writeln!(
        out,
        "//! Do not edit manually - regenerate with: wow-cli generate area-table"
    )?;
    writeln!(out)?;
    writeln!(out, "#[derive(Debug, Clone, Copy, PartialEq, Eq)]")?;
    writeln!(out, "pub struct AreaTableRow {{")?;
    writeln!(out, "    pub id: u32,")?;
    writeln!(out, "    pub continent_id: u16,")?;
    writeln!(out, "}}")?;
    writeln!(out)?;
    writeln!(out, "#[rustfmt::skip]")?;
    writeln!(out, "pub static AREA_TABLE_RETAIL: &[AreaTableRow] = &[")
}

fn write_rows(out: &mut File, rows: &[AreaTableRow]) -> std::io::Result<()> {
    for row in rows {
        writeln!(out, "    AreaTableRow {{")?;
        writeln!(out, "        id: {},", row.id)?;
        writeln!(out, "        continent_id: {},", row.continent_id)?;
        writeln!(out, "    }},")?;
    }
    writeln!(out, "];\n")
}

fn write_lookup(out: &mut File) -> std::io::Result<()> {
    writeln!(out, "#[rustfmt::skip]")?;
    writeln!(
        out,
        "pub fn get_continent_id(area_id: u32) -> Option<u16> {{"
    )?;
    writeln!(out, "    let index = AREA_TABLE_RETAIL")?;
    writeln!(out, "        .binary_search_by_key(&area_id, |row| row.id)")?;
    writeln!(out, "        .ok()?;")?;
    writeln!(out, "    Some(AREA_TABLE_RETAIL[index].continent_id)")?;
    writeln!(out, "}}")
}

#[cfg(test)]
mod tests {
    use super::{find_columns, parse_row};

    #[test]
    fn parser_accepts_zero_u16_values() {
        let columns = find_columns("ContinentID,ID").unwrap();
        let row = parse_row(&columns, "0,42", 2).unwrap();
        assert_eq!(row.continent_id, 0);
        assert_eq!(row.id, 42);
    }
}
