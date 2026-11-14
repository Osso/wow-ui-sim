//! Generate Retail GlobalColor rows from the authenticated local DB2 CSV.

use super::csv_util::{escape_str, parse_csv_line, read_csv_records};
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::{BufReader, Write};
use std::path::Path;

const RETAIL_GLOBAL_COLOR_CSV: &str = "data/db2/retail-12.1.5.70783/GlobalColor.csv";
const RETAIL_GLOBAL_COLOR_PROVENANCE: &str =
    "data/db2/retail-12.1.5.70783/GlobalColor.provenance.json";
const RETAIL_GLOBAL_COLOR_OUTPUT: &str = "crates/sim-data-profile/data/global_colors_retail.rs";
const RETAIL_GLOBAL_COLOR_BUILD: &str = "12.1.5.70783";
const RETAIL_GLOBAL_COLOR_FILE_DATA_ID: u32 = 4_328_468;

#[derive(Clone, Debug, Eq, PartialEq)]
struct GlobalColorRow {
    id: u32,
    base_tag: String,
    packed_color: u32,
}

pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    let csv_path = Path::new(RETAIL_GLOBAL_COLOR_CSV);
    let records = read_csv_records(BufReader::new(File::open(csv_path)?))?;
    let rows = parse_global_color_rows(&records)?;
    let (physical_count, copy_pairs) = read_copy_record_pairs()?;
    let (main_index_rows, auxiliary_rows) = split_feed_rows(rows, physical_count, &copy_pairs)?;
    let export_rows = deduplicate_and_sort(&main_index_rows, &auxiliary_rows);

    let output_path = Path::new(RETAIL_GLOBAL_COLOR_OUTPUT);
    let mut output = File::create(output_path)?;
    write_header(&mut output)?;
    write_global_color_rows(&mut output, &export_rows)?;

    println!(
        "Generated {} exported GlobalColor rows from {} main and {} auxiliary rows in {}",
        export_rows.len(),
        main_index_rows.len(),
        auxiliary_rows.len(),
        csv_path.display()
    );
    println!("Output: {}", output_path.display());
    Ok(())
}

fn parse_global_color_rows(
    records: &[String],
) -> Result<Vec<GlobalColorRow>, Box<dyn std::error::Error>> {
    let header = records.first().ok_or("GlobalColor CSV is empty")?;
    let columns = parse_csv_line(header);
    let id_column = find_column(&columns, "ID")?;
    let tag_column = find_column(&columns, "LuaConstantName")?;
    let color_column = find_column(&columns, "Color")?;
    let rows = records
        .iter()
        .enumerate()
        .skip(1)
        .map(|(index, record)| {
            parse_global_color_row(record, id_column, tag_column, color_column, index + 1)
        })
        .collect::<Result<Vec<_>, _>>()?;
    validate_unique_ids(&rows)?;
    Ok(rows)
}

fn parse_global_color_row(
    record: &str,
    id_column: usize,
    tag_column: usize,
    color_column: usize,
    row_number: usize,
) -> Result<GlobalColorRow, Box<dyn std::error::Error>> {
    let fields = parse_csv_line(record);
    let id = parse_field::<u32>(&fields, id_column, "ID", row_number)?;
    let base_tag = field(&fields, tag_column, "LuaConstantName", row_number)?;
    let signed_color = parse_field::<i32>(&fields, color_column, "Color", row_number)?;
    Ok(GlobalColorRow {
        id,
        base_tag: base_tag.to_owned(),
        packed_color: signed_color as u32,
    })
}

fn read_copy_record_pairs() -> Result<(usize, Vec<(u32, u32)>), Box<dyn std::error::Error>> {
    let provenance = read_global_color_provenance()?;
    let db2 = provenance
        .get("db2")
        .ok_or("GlobalColor provenance has no db2 section")?;
    let physical_count = read_db2_count(db2, "physical_records")?;
    let copy_count = read_db2_count(db2, "copy_records")?;
    let copy_pairs = read_copy_pairs(db2)?;
    if copy_pairs.len() != copy_count {
        return Err(format!(
            "GlobalColor provenance declares {copy_count} copy records but lists {} pairs",
            copy_pairs.len()
        )
        .into());
    }
    Ok((physical_count, copy_pairs))
}

fn read_global_color_provenance() -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let file = File::open(RETAIL_GLOBAL_COLOR_PROVENANCE)?;
    Ok(serde_json::from_reader(BufReader::new(file))?)
}

fn read_db2_count(
    db2: &serde_json::Value,
    field: &str,
) -> Result<usize, Box<dyn std::error::Error>> {
    let count = db2
        .get(field)
        .and_then(serde_json::Value::as_u64)
        .ok_or_else(|| format!("GlobalColor provenance has no valid {field} count"))?;
    Ok(usize::try_from(count)?)
}

fn read_copy_pairs(db2: &serde_json::Value) -> Result<Vec<(u32, u32)>, Box<dyn std::error::Error>> {
    let pairs = db2
        .get("copy_record_pairs")
        .and_then(serde_json::Value::as_array)
        .ok_or("GlobalColor provenance has no copy record pairs")?;
    pairs.iter().map(parse_copy_pair).collect()
}

fn parse_copy_pair(pair: &serde_json::Value) -> Result<(u32, u32), Box<dyn std::error::Error>> {
    let new_id = read_copy_id(pair, "new_id")?;
    let source_id = read_copy_id(pair, "source_id")?;
    Ok((new_id, source_id))
}

fn read_copy_id(pair: &serde_json::Value, field: &str) -> Result<u32, Box<dyn std::error::Error>> {
    pair.get(field)
        .and_then(serde_json::Value::as_u64)
        .and_then(|id| u32::try_from(id).ok())
        .ok_or_else(|| format!("GlobalColor copy pair has an invalid {field}").into())
}

fn split_feed_rows(
    rows: Vec<GlobalColorRow>,
    physical_count: usize,
    copy_pairs: &[(u32, u32)],
) -> Result<(Vec<GlobalColorRow>, Vec<GlobalColorRow>), Box<dyn std::error::Error>> {
    let copy_ids = validate_copy_rows(&rows, copy_pairs)?;
    if rows.len() != physical_count + copy_ids.len() {
        return Err(format!(
            "GlobalColor CSV has {} rows, expected {physical_count} physical plus {} copies",
            rows.len(),
            copy_ids.len()
        )
        .into());
    }

    let mut main_index_rows = rows
        .iter()
        .filter(|row| !copy_ids.contains(&row.id))
        .cloned()
        .collect::<Vec<_>>();
    let mut auxiliary_rows = rows
        .into_iter()
        .filter(|row| copy_ids.contains(&row.id))
        .collect::<Vec<_>>();
    main_index_rows.sort_unstable_by_key(|row| row.id);
    auxiliary_rows.sort_unstable_by_key(|row| row.id);
    if main_index_rows.len() != physical_count {
        return Err(format!(
            "GlobalColor has {} main rows, expected {physical_count}",
            main_index_rows.len()
        )
        .into());
    }
    Ok((main_index_rows, auxiliary_rows))
}

fn validate_copy_rows(
    rows: &[GlobalColorRow],
    copy_pairs: &[(u32, u32)],
) -> Result<HashSet<u32>, Box<dyn std::error::Error>> {
    let rows_by_id = rows
        .iter()
        .map(|row| (row.id, row))
        .collect::<HashMap<_, _>>();
    let mut copy_ids = HashSet::with_capacity(copy_pairs.len());
    for &(copy_id, source_id) in copy_pairs {
        if !copy_ids.insert(copy_id) {
            return Err(format!("GlobalColor provenance repeats copy ID {copy_id}").into());
        }
        let copy = rows_by_id
            .get(&copy_id)
            .ok_or_else(|| format!("GlobalColor CSV has no copy row {copy_id}"))?;
        let source = rows_by_id
            .get(&source_id)
            .ok_or_else(|| format!("GlobalColor CSV has no copy source row {source_id}"))?;
        if copy.base_tag != source.base_tag || copy.packed_color != source.packed_color {
            return Err(format!(
                "GlobalColor copy row {copy_id} differs from source row {source_id}"
            )
            .into());
        }
    }
    Ok(copy_ids)
}

fn deduplicate_and_sort(
    main_index_rows: &[GlobalColorRow],
    auxiliary_rows: &[GlobalColorRow],
) -> Vec<GlobalColorRow> {
    let capacity = main_index_rows.len() + auxiliary_rows.len();
    let mut seen = HashSet::with_capacity(capacity);
    let mut rows = Vec::with_capacity(capacity);
    for row in main_index_rows.iter().chain(auxiliary_rows) {
        if seen.insert(row.base_tag.as_str()) {
            rows.push(row.clone());
        }
    }
    rows.sort_unstable_by(|left, right| left.base_tag.as_bytes().cmp(right.base_tag.as_bytes()));
    rows
}

fn validate_unique_ids(rows: &[GlobalColorRow]) -> Result<(), Box<dyn std::error::Error>> {
    let mut ids = HashSet::with_capacity(rows.len());
    for row in rows {
        if !ids.insert(row.id) {
            return Err(format!("GlobalColor CSV repeats ID {}", row.id).into());
        }
    }
    Ok(())
}

fn find_column(columns: &[String], name: &str) -> Result<usize, Box<dyn std::error::Error>> {
    columns
        .iter()
        .position(|column| column == name)
        .ok_or_else(|| format!("GlobalColor CSV has no {name} column").into())
}

fn field<'a>(
    fields: &'a [String],
    column: usize,
    name: &str,
    row: usize,
) -> Result<&'a str, Box<dyn std::error::Error>> {
    fields
        .get(column)
        .map(String::as_str)
        .ok_or_else(|| format!("GlobalColor CSV row {row} has no {name} value").into())
}

fn parse_field<T: std::str::FromStr>(
    fields: &[String],
    column: usize,
    name: &str,
    row: usize,
) -> Result<T, Box<dyn std::error::Error>>
where
    T::Err: std::fmt::Display,
{
    field(fields, column, name, row)?
        .parse::<T>()
        .map_err(|error| format!("GlobalColor CSV row {row} has invalid {name}: {error}").into())
}

fn write_header(output: &mut File) -> std::io::Result<()> {
    writeln!(output, "//! Auto-generated Retail GlobalColor DB2 rows.")?;
    writeln!(
        output,
        "//! Source build: Retail {RETAIL_GLOBAL_COLOR_BUILD}, FileDataID {RETAIL_GLOBAL_COLOR_FILE_DATA_ID}."
    )?;
    writeln!(
        output,
        "//! Native rules are VERIFIED for 12.1.5.70688 and INFERRED for this data build."
    )?;
    writeln!(
        output,
        "//! Feed order: main rows by ID, then auxiliary copies by ID."
    )?;
    writeln!(
        output,
        "//! First case-sensitive tag wins; rows are sorted by unsigned tag bytes."
    )?;
    writeln!(output, "//! No empty-tag or flag filter is applied.")?;
    writeln!(
        output,
        "//! Do not edit manually; regenerate with `wow-cli generate global-colors`."
    )?;
    writeln!(output)?;
    writeln!(output, "#[derive(Clone, Copy, Debug, Eq, PartialEq)]")?;
    writeln!(output, "pub struct GlobalColorRow {{")?;
    writeln!(output, "    pub id: u32,")?;
    writeln!(output, "    pub base_tag: &'static str,")?;
    writeln!(output, "    pub packed_color: u32,")?;
    writeln!(output, "}}")?;
    writeln!(output)?;
    writeln!(output, "#[rustfmt::skip]")?;
    writeln!(
        output,
        "pub static RETAIL_GLOBAL_COLORS: &[GlobalColorRow] = &["
    )?;
    Ok(())
}

fn write_global_color_rows(output: &mut File, rows: &[GlobalColorRow]) -> std::io::Result<()> {
    for row in rows {
        writeln!(
            output,
            "    GlobalColorRow {{ id: {}, base_tag: \"{}\", packed_color: 0x{:08x} }},",
            row.id,
            escape_str(&row.base_tag),
            row.packed_color
        )?;
    }
    writeln!(output, "];")
}

#[cfg(test)]
mod tests {
    use super::{GlobalColorRow, deduplicate_and_sort};

    fn row(id: u32, base_tag: &str, packed_color: u32) -> GlobalColorRow {
        GlobalColorRow {
            id,
            base_tag: base_tag.to_owned(),
            packed_color,
        }
    }

    #[test]
    fn native_feed_keeps_first_tag_and_sorts_unsigned_bytes() {
        let main = [row(90, "z", 1), row(100, "duplicate", 2), row(101, "", 3)];
        let auxiliary = [
            row(1, "duplicate", 4),
            row(2, "A", 5),
            row(3, "\u{00e9}", 6),
        ];

        let rows = deduplicate_and_sort(&main, &auxiliary);

        assert_eq!(
            rows,
            vec![
                row(101, "", 3),
                row(2, "A", 5),
                row(100, "duplicate", 2),
                row(90, "z", 1),
                row(3, "\u{00e9}", 6),
            ]
        );
    }
}
