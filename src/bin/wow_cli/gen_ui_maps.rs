//! Generator for ui_maps.rs from the WoW UiMap CSV export.
//!
//! Reads from ~/Projects/wow/data/:
//!   - UiMap.csv (Name_lang, ID, ParentUiMapID, Flags, System, Type, ...)
//!
//! Generates: crates/sim-data/data/ui_maps.rs
//!
//! `C_Map.GetMapInfo` answers from this table for every uiMapID the hand
//! seeded `SimState.maps` does not carry, so an addon that names the
//! destination of a teleport by its map gets "Dornogal", not "Map 2339".

use super::csv_util::{escape_str, parse_csv_line, read_csv_records, wow_data_dir};
use std::collections::BTreeMap;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;

const RETAIL_ASSIGNMENT_CSV: &str = "data/db2/retail-12.1.5.70783/UiMapAssignment.csv";
const RETAIL_ASSIGNMENT_OUTPUT: &str = "crates/sim-data-profile/data/ui_map_assignments_retail.rs";

pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    let wow_data = wow_data_dir();
    let csv_path = wow_data.join("UiMap.csv");
    println!("Loading UiMap from {}...", csv_path.display());

    let file = File::open(&csv_path)?;
    let reader = BufReader::new(file);

    let output_path = Path::new("crates/sim-data/data/ui_maps.rs");
    let mut out = File::create(output_path)?;

    write_header(&mut out)?;
    let (count, skipped) = build_ui_map_table(&mut out, reader)?;
    write_lookup_fn(&mut out)?;
    write_tests(&mut out)?;

    println!("Generated {} ui map entries ({} skipped)", count, skipped);
    println!("Output: {}", output_path.display());
    run_assignments()?;
    Ok(())
}

pub fn run_assignments() -> Result<(), Box<dyn std::error::Error>> {
    let csv_path = Path::new(RETAIL_ASSIGNMENT_CSV);
    println!(
        "Loading Retail UiMapAssignment from {}...",
        csv_path.display()
    );
    let (assignments, row_count) = load_assignment_rows(csv_path)?;
    let output_path = Path::new(RETAIL_ASSIGNMENT_OUTPUT);
    let mut out = File::create(output_path)?;
    write_assignment_file(&mut out, &assignments)?;
    println!("Generated {} UiMapAssignment rows", row_count);
    println!("Output: {}", output_path.display());
    Ok(())
}

struct Columns {
    name: usize,
    id: usize,
    parent: usize,
    flags: usize,
    map_type: usize,
}

fn find_columns(header: &str) -> Result<Columns, Box<dyn std::error::Error>> {
    let fields = parse_csv_line(header);
    let index = |name: &str| {
        fields
            .iter()
            .position(|f| f == name)
            .ok_or_else(|| format!("UiMap.csv has no {name} column"))
    };
    Ok(Columns {
        name: index("Name_lang")?,
        id: index("ID")?,
        parent: index("ParentUiMapID")?,
        flags: index("Flags")?,
        map_type: index("Type")?,
    })
}

fn build_ui_map_table(
    out: &mut File,
    reader: BufReader<File>,
) -> Result<(u32, u32), Box<dyn std::error::Error>> {
    let mut builder = phf_codegen::Map::new();
    let mut count = 0u32;
    let mut skipped = 0u32;
    let mut columns: Option<Columns> = None;

    for (index, line) in reader.lines().enumerate() {
        let line = line?;
        let Some(cols) = columns.as_ref() else {
            columns = Some(find_columns(&line)?);
            continue;
        };
        match parse_ui_map_row(cols, &line, index + 1)? {
            Some((id, value)) => {
                builder.entry(id, &value);
                count += 1;
            }
            None => {
                skipped += 1;
            }
        }
    }

    writeln!(
        out,
        "pub static UI_MAP_DB: phf::Map<u32, UiMapInfo> = {};",
        builder.build()
    )?;
    writeln!(out)?;
    Ok((count, skipped))
}

fn parse_ui_map_row(
    cols: &Columns,
    line: &str,
    row: usize,
) -> Result<Option<(u32, String)>, String> {
    let fields = parse_csv_line(line);
    let field = |index: usize, name: &str| {
        fields
            .get(index)
            .map(String::as_str)
            .ok_or_else(|| format!("UiMap.csv row {row}: missing field {name} (short row)"))
    };
    let number = |index: usize, name: &str| -> Result<u32, String> {
        let value = field(index, name)?;
        value
            .parse()
            .map_err(|_| format!("UiMap.csv row {row}: invalid {name}: {value:?}"))
    };
    let name = field(cols.name, "Name_lang")?;
    let id = number(cols.id, "ID")?;
    let parent_map_id = number(cols.parent, "ParentUiMapID")?;
    let flags = number(cols.flags, "Flags")?;
    let map_type = number(cols.map_type, "Type")?;
    if name.is_empty() {
        return Ok(None);
    }

    let value = format!(
        "UiMapInfo {{ name: \"{}\", map_type: {}, parent_map_id: {}, flags: {} }}",
        escape_str(name),
        map_type,
        parent_map_id,
        flags
    );
    Ok(Some((id, value)))
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct UiMapAssignmentRow {
    map_id: i32,
    order_index: u32,
    ui_min: [f32; 2],
    ui_max: [f32; 2],
    region_min: [f32; 2],
    region_max: [f32; 2],
}

struct AssignmentColumns {
    ui_map_id: usize,
    order_index: usize,
    map_id: usize,
    ui_min: [usize; 2],
    ui_max: [usize; 2],
    region_min: [usize; 2],
    region_max: [usize; 2],
}

fn find_assignment_columns(header: &str) -> Result<AssignmentColumns, String> {
    let fields = parse_csv_line(header);
    let index = |name: &str| {
        fields
            .iter()
            .position(|field| field == name)
            .ok_or_else(|| format!("UiMapAssignment.csv has no {name} column"))
    };
    Ok(AssignmentColumns {
        ui_map_id: index("UiMapID")?,
        order_index: index("OrderIndex")?,
        map_id: index("MapID")?,
        ui_min: [index("UiMin_0")?, index("UiMin_1")?],
        ui_max: [index("UiMax_0")?, index("UiMax_1")?],
        region_min: [index("Region_0")?, index("Region_1")?],
        region_max: [index("Region_3")?, index("Region_4")?],
    })
}

fn parse_assignment_row(
    columns: &AssignmentColumns,
    line: &str,
    row: usize,
) -> Result<(u32, UiMapAssignmentRow), String> {
    let fields = parse_csv_line(line);
    let field = |index: usize, name: &str| {
        fields
            .get(index)
            .map(String::as_str)
            .ok_or_else(|| format!("UiMapAssignment.csv row {row}: missing {name} (short row)"))
    };
    let integer = |index: usize, name: &str| -> Result<u32, String> {
        let value = field(index, name)?;
        value
            .parse()
            .map_err(|_| format!("UiMapAssignment.csv row {row}: invalid {name}: {value:?}"))
    };
    let signed_integer = |index: usize, name: &str| -> Result<i32, String> {
        let value = field(index, name)?;
        value
            .parse()
            .map_err(|_| format!("UiMapAssignment.csv row {row}: invalid {name}: {value:?}"))
    };
    let float = |index: usize, name: &str| -> Result<f32, String> {
        let value = field(index, name)?;
        let number = value
            .parse::<f32>()
            .map_err(|_| format!("UiMapAssignment.csv row {row}: invalid {name}: {value:?}"))?;
        if !number.is_finite() {
            return Err(format!(
                "UiMapAssignment.csv row {row}: nonfinite {name}: {value:?}"
            ));
        }
        Ok(number)
    };

    let ui_map_id = integer(columns.ui_map_id, "UiMapID")?;
    let assignment = UiMapAssignmentRow {
        map_id: signed_integer(columns.map_id, "MapID")?,
        order_index: integer(columns.order_index, "OrderIndex")?,
        ui_min: [
            float(columns.ui_min[0], "UiMin_0")?,
            float(columns.ui_min[1], "UiMin_1")?,
        ],
        ui_max: [
            float(columns.ui_max[0], "UiMax_0")?,
            float(columns.ui_max[1], "UiMax_1")?,
        ],
        region_min: [
            float(columns.region_min[0], "Region_0")?,
            float(columns.region_min[1], "Region_1")?,
        ],
        region_max: [
            float(columns.region_max[0], "Region_3")?,
            float(columns.region_max[1], "Region_4")?,
        ],
    };
    Ok((ui_map_id, assignment))
}

fn load_assignment_rows(
    path: &Path,
) -> Result<(BTreeMap<u32, Vec<UiMapAssignmentRow>>, usize), Box<dyn std::error::Error>> {
    let file = File::open(path)?;
    let records = read_csv_records(BufReader::new(file))?;
    let header = records
        .first()
        .ok_or_else(|| format!("{}: missing CSV header", path.display()))?;
    let columns =
        find_assignment_columns(header).map_err(|error| format!("{}: {error}", path.display()))?;
    let mut assignments = BTreeMap::<u32, Vec<UiMapAssignmentRow>>::new();

    for (index, line) in records.iter().enumerate().skip(1) {
        let row_number = index + 1;
        let (ui_map_id, assignment) = parse_assignment_row(&columns, line, row_number)
            .map_err(|error| format!("{}: {error}", path.display()))?;
        assignments.entry(ui_map_id).or_default().push(assignment);
    }
    if assignments.is_empty() {
        return Err(format!("{}: no UiMapAssignment rows", path.display()).into());
    }
    for rows in assignments.values_mut() {
        rows.sort_by_key(|assignment| assignment.order_index);
    }
    let row_count = assignments.values().map(Vec::len).sum();
    Ok((assignments, row_count))
}

fn write_assignment_file(
    out: &mut File,
    assignments: &BTreeMap<u32, Vec<UiMapAssignmentRow>>,
) -> std::io::Result<()> {
    writeln!(
        out,
        "//! Auto-generated Retail UiMapAssignment vector2 fields from a CASC DB2 export."
    )?;
    writeln!(
        out,
        "//! Do not edit manually - regenerate with: wow-cli generate ui-map-assignments"
    )?;
    writeln!(
        out,
        "//! VERIFIED source: data/db2/retail-12.1.5.70783/provenance.json."
    )?;
    writeln!(
        out,
        "//! INFERRED applicability: 70783 assignments match the selected 70575 Retail pin; exact parity is unverified."
    )?;
    writeln!(out)?;
    writeln!(out, "#[derive(Debug, Clone, Copy, PartialEq)]")?;
    writeln!(out, "pub struct UiMapAssignment {{")?;
    writeln!(out, "    pub map_id: i32,")?;
    writeln!(out, "    pub order_index: u32,")?;
    writeln!(out, "    pub ui_min: [f32; 2],")?;
    writeln!(out, "    pub ui_max: [f32; 2],")?;
    writeln!(out, "    pub region_min: [f32; 2],")?;
    writeln!(out, "    pub region_max: [f32; 2],")?;
    writeln!(out, "}}")?;
    writeln!(out)?;
    writeln!(
        out,
        "pub static UI_MAP_ASSIGNMENTS: &[(u32, &'static [UiMapAssignment])] = &["
    )?;
    for (ui_map_id, rows) in assignments {
        writeln!(out, "    ({ui_map_id}, &[")?;
        for row in rows {
            let ui_min = row.ui_min.map(|value| format!("{value:?}f32"));
            let ui_max = row.ui_max.map(|value| format!("{value:?}f32"));
            let region_min = row.region_min.map(|value| format!("{value:?}f32"));
            let region_max = row.region_max.map(|value| format!("{value:?}f32"));
            writeln!(
                out,
                "        UiMapAssignment {{ map_id: {}, order_index: {}, ui_min: [{}, {}], ui_max: [{}, {}], region_min: [{}, {}], region_max: [{}, {}] }},",
                row.map_id,
                row.order_index,
                ui_min[0],
                ui_min[1],
                ui_max[0],
                ui_max[1],
                region_min[0],
                region_min[1],
                region_max[0],
                region_max[1],
            )?;
        }
        writeln!(out, "    ]),")?;
    }
    writeln!(out, "];")?;
    writeln!(out)?;
    writeln!(
        out,
        "pub fn get_ui_map_assignments(ui_map_id: u32) -> Option<&'static [UiMapAssignment]> {{"
    )?;
    writeln!(
        out,
        "    let index = UI_MAP_ASSIGNMENTS.binary_search_by_key(&ui_map_id, |(id, _)| *id).ok()?;"
    )?;
    writeln!(out, "    Some(UI_MAP_ASSIGNMENTS[index].1)")?;
    writeln!(out, "}}")?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn assignment_parser_uses_named_columns_and_preserves_db2_values() {
        let header = "MapID,UiMapID,OrderIndex,UiMin_0,UiMin_1,UiMax_0,UiMax_1,Region_0,Region_1,Region_2,Region_3,Region_4,Region_5";
        let columns = find_assignment_columns(header).unwrap();
        let row =
            "1,7,0,0,0,1,1,-3802.08008,-3245.83008,unused-z-min,-168.75,2204.16992,unused-z-max";
        let (ui_map_id, assignment) = parse_assignment_row(&columns, row, 2).unwrap();

        assert_eq!(ui_map_id, 7);
        assert_eq!(assignment.map_id, 1);
        assert_eq!(assignment.order_index, 0);
        assert_eq!(assignment.ui_min, [0.0, 0.0]);
        assert_eq!(assignment.ui_max, [1.0, 1.0]);
        assert_eq!(assignment.region_min, [-3802.08008, -3245.83008]);
        assert_eq!(assignment.region_max, [-168.75, 2204.16992]);
    }

    #[test]
    fn assignment_parser_rejects_nonfinite_geometry() {
        let header = "UiMapID,OrderIndex,MapID,UiMin_0,UiMin_1,UiMax_0,UiMax_1,Region_0,Region_1,Region_2,Region_3,Region_4,Region_5";
        let columns = find_assignment_columns(header).unwrap();
        let error =
            parse_assignment_row(&columns, "7,0,1,0,0,1,1,NaN,0,-1000000,10,20,1000000", 42)
                .unwrap_err();
        assert!(error.contains("row 42"), "{error}");
        assert!(error.contains("nonfinite Region_0"), "{error}");
    }

    #[test]
    fn assignment_parser_preserves_negative_map_ids() {
        let header = "UiMapID,OrderIndex,MapID,UiMin_0,UiMin_1,UiMax_0,UiMax_1,Region_0,Region_1,Region_2,Region_3,Region_4,Region_5";
        let columns = find_assignment_columns(header).unwrap();
        let row = "1148,2,-1,0,0,1,1,-1,-2,-1000000,3,4,1000000";
        let (_, assignment) = parse_assignment_row(&columns, row, 9).unwrap();
        assert_eq!(assignment.map_id, -1);
    }

    #[test]
    fn malformed_numeric_fields_are_rejected() {
        let columns = find_columns("Name_lang,ID,ParentUiMapID,Flags,Type").unwrap();
        for (line, field) in [
            ("Dornogal,no,2248,0,3", "ID"),
            ("Dornogal,2339,no,0,3", "ParentUiMapID"),
            ("Dornogal,2339,2248,no,3", "Flags"),
            ("Dornogal,2339,2248,0,no", "Type"),
        ] {
            let error = parse_ui_map_row(&columns, line, 42).unwrap_err();
            assert!(error.contains("row 42"), "{error}");
            assert!(error.contains(field), "{error}");
        }
    }

    #[test]
    fn short_row_identifies_missing_field() {
        let columns = find_columns("Name_lang,ID,ParentUiMapID,Flags,Type").unwrap();
        let error = parse_ui_map_row(&columns, "Dornogal,2339,2248", 7).unwrap_err();
        assert!(error.contains("row 7"), "{error}");
        assert!(error.contains("Flags"), "{error}");
        assert!(error.contains("short row"), "{error}");
    }

    #[test]
    fn unnamed_row_is_skipped() {
        let columns = find_columns("Name_lang,ID,ParentUiMapID,Flags,Type").unwrap();
        assert!(
            parse_ui_map_row(&columns, ",1500,1355,0,5", 2)
                .unwrap()
                .is_none()
        );
    }
}

fn write_header(out: &mut File) -> std::io::Result<()> {
    writeln!(out, "//! Auto-generated UiMap data from the WoW UiMap CSV.")?;
    writeln!(
        out,
        "//! Do not edit manually - regenerate with: wow-cli generate ui-maps"
    )?;
    writeln!(out)?;
    writeln!(out, "#[derive(Debug, Clone)]")?;
    writeln!(out, "pub struct UiMapInfo {{")?;
    writeln!(out, "    pub name: &'static str,")?;
    writeln!(
        out,
        "    /// `Enum.UIMapType`: 0 cosmic, 1 world, 2 continent, 3 zone, 4 dungeon, 5 micro, 6 orphan"
    )?;
    writeln!(out, "    pub map_type: u32,")?;
    writeln!(out, "    pub parent_map_id: u32,")?;
    writeln!(out, "    pub flags: u32,")?;
    writeln!(out, "}}")?;
    writeln!(out)?;
    Ok(())
}

fn write_lookup_fn(out: &mut File) -> std::io::Result<()> {
    writeln!(
        out,
        "pub fn get_ui_map(id: u32) -> Option<&'static UiMapInfo> {{"
    )?;
    writeln!(out, "    UI_MAP_DB.get(&id)")?;
    writeln!(out, "}}")?;
    Ok(())
}

fn write_tests(out: &mut File) -> std::io::Result<()> {
    writeln!(out)?;
    writeln!(out, "#[cfg(test)]")?;
    writeln!(out, "mod tests {{")?;
    writeln!(out, "    use super::*;")?;
    writeln!(out)?;
    writeln!(out, "    #[test]")?;
    writeln!(out, "    fn test_ui_map_count() {{")?;
    writeln!(out, "        assert!(UI_MAP_DB.len() > 1500);")?;
    writeln!(out, "    }}")?;
    writeln!(out)?;
    writeln!(out, "    #[test]")?;
    writeln!(
        out,
        "    fn test_dornogal_is_a_zone_of_the_isle_of_dorn() {{"
    )?;
    writeln!(
        out,
        "        let map = get_ui_map(2339).expect(\"Dornogal (2339) should exist\");"
    )?;
    writeln!(out, "        assert_eq!(map.name, \"Dornogal\");")?;
    writeln!(out, "        assert_eq!(map.map_type, 3);")?;
    writeln!(
        out,
        "        assert_eq!(map.parent_map_id, 2248, \"Isle of Dorn, not Khaz Algar 2274\");"
    )?;
    writeln!(out, "    }}")?;
    writeln!(out)?;
    writeln!(out, "    #[test]")?;
    writeln!(out, "    fn test_cosmic_and_azeroth() {{")?;
    writeln!(
        out,
        "        assert_eq!(get_ui_map(946).map(|m| m.name), Some(\"Cosmic\"));"
    )?;
    writeln!(
        out,
        "        assert_eq!(get_ui_map(947).map(|m| m.name), Some(\"Azeroth\"));"
    )?;
    writeln!(
        out,
        "        assert_eq!(get_ui_map(947).map(|m| m.parent_map_id), Some(946));"
    )?;
    writeln!(out, "    }}")?;
    writeln!(out)?;
    writeln!(out, "    #[test]")?;
    writeln!(out, "    fn test_nonexistent_ui_map() {{")?;
    writeln!(out, "        assert!(get_ui_map(999_999_999).is_none());")?;
    writeln!(out, "    }}")?;
    writeln!(out, "}}")?;
    Ok(())
}
