//! Selected Forever DB2 rows and their source-byte provenance.

use std::collections::{BTreeMap, BTreeSet};
use std::ops::Range;

use serde::Serialize;

pub const SELECTED_IDS: &str = include_str!("../data/forever-spell-ids.txt");
const TABLES: [&str; 5] = [
    "Spell",
    "SpellEffect",
    "SpellMisc",
    "SpellName",
    "SpellPower",
];

/// One DB2 record and all byte ranges used to decode its columns.
pub struct SourceRow {
    pub spell_id: u32,
    pub record_id: u32,
    pub columns: BTreeMap<String, serde_json::Value>,
    pub source_ranges: Vec<Range<usize>>,
}

/// A decoded DB2 table. Ranges use offsets in the decompressed DB2 payload.
pub struct SourceTable {
    pub layout_hash: u32,
    pub unreadable_records: usize,
    pub zero_filled: Vec<Range<usize>>,
    pub rows: Vec<SourceRow>,
}

#[derive(Serialize)]
struct OutputTable {
    layout_hash: String,
    unreadable_records: usize,
    rows: Vec<BTreeMap<String, serde_json::Value>>,
}

#[derive(Serialize)]
struct Output {
    schema: u32,
    build: &'static str,
    tables: BTreeMap<String, OutputTable>,
}

/// Generate a deterministic cache containing only selected spells.
pub fn generate(
    selected: &BTreeSet<u32>,
    mut tables: BTreeMap<String, SourceTable>,
) -> Result<Vec<u8>, String> {
    if selected.is_empty() {
        return Err("Forever spell ID list is empty".into());
    }
    let mut result = BTreeMap::new();
    for name in TABLES {
        let mut table = tables
            .remove(name)
            .ok_or_else(|| format!("missing {name} table"))?;
        table.rows.retain(|row| selected.contains(&row.spell_id));
        table.rows.sort_by_key(|row| (row.spell_id, row.record_id));
        let found: BTreeSet<_> = table.rows.iter().map(|row| row.spell_id).collect();
        if found != *selected {
            let missing: Vec<_> = selected.difference(&found).collect();
            return Err(format!("{name}: missing selected spell IDs {missing:?}"));
        }
        for row in &table.rows {
            if row.source_ranges.is_empty() || row.columns.is_empty() {
                return Err(format!(
                    "{name}: spell {} has no column provenance",
                    row.spell_id
                ));
            }
            for range in &row.source_ranges {
                if range.start >= range.end
                    || table
                        .zero_filled
                        .iter()
                        .any(|zero| range.start < zero.end && zero.start < range.end)
                {
                    return Err(format!(
                        "{name}: spell {} uses invalid or zero-filled bytes {range:?}",
                        row.spell_id
                    ));
                }
            }
        }
        result.insert(
            name.to_owned(),
            OutputTable {
                layout_hash: format!("{:08X}", table.layout_hash),
                unreadable_records: table.unreadable_records,
                rows: table.rows.into_iter().map(|row| row.columns).collect(),
            },
        );
    }
    if !tables.is_empty() {
        return Err(format!(
            "unexpected tables: {:?}",
            tables.keys().collect::<Vec<_>>()
        ));
    }
    serde_json::to_vec(&Output {
        schema: 1,
        build: "1.60.1.70685",
        tables: result,
    })
    .map_err(|error| format!("serialize Forever spell cache: {error}"))
}

pub fn selected_ids() -> Result<BTreeSet<u32>, String> {
    let mut ids = BTreeSet::new();
    for line in SELECTED_IDS.lines() {
        let id = line
            .parse::<u32>()
            .map_err(|error| format!("invalid Forever spell ID {line:?}: {error}"))?;
        if !ids.insert(id) {
            return Err(format!("duplicate Forever spell ID {id}"));
        }
    }
    Ok(ids)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn tables() -> BTreeMap<String, SourceTable> {
        TABLES
            .into_iter()
            .map(|name| {
                (
                    name.to_owned(),
                    SourceTable {
                        layout_hash: 0x1234abcd,
                        unreadable_records: 7,
                        zero_filled: vec![100..200],
                        rows: vec![
                            SourceRow {
                                spell_id: 9,
                                record_id: 9,
                                columns: BTreeMap::from([("ID".into(), json!(9))]),
                                source_ranges: vec![50..60],
                            },
                            SourceRow {
                                spell_id: 3,
                                record_id: 3,
                                columns: BTreeMap::from([
                                    ("ID".into(), json!(3)),
                                    ("Name".into(), json!("test")),
                                ]),
                                source_ranges: vec![30..40, 80..90],
                            },
                            SourceRow {
                                spell_id: 8,
                                record_id: 8,
                                columns: BTreeMap::from([("ID".into(), json!(8))]),
                                source_ranges: vec![110..120],
                            },
                        ],
                    },
                )
            })
            .collect()
    }

    #[test]
    fn selected_rows_are_deterministic_and_zero_filled_columns_are_rejected() {
        let selected = BTreeSet::from([3, 9]);
        let bytes = generate(&selected, tables()).unwrap();
        assert_eq!(bytes, generate(&selected, tables()).unwrap());
        let output: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(output["tables"]["SpellName"]["layout_hash"], "1234ABCD");
        assert_eq!(output["tables"]["SpellName"]["unreadable_records"], 7);
        assert_eq!(
            output["tables"]["SpellName"]["rows"],
            json!([{"ID": 3, "Name": "test"}, {"ID": 9}])
        );
        let mut corrupted = tables();
        corrupted.get_mut("Spell").unwrap().rows[1]
            .source_ranges
            .push(99..101);
        assert!(
            generate(&selected, corrupted)
                .unwrap_err()
                .contains("zero-filled")
        );
        let mut missing = tables();
        missing
            .get_mut("SpellPower")
            .unwrap()
            .rows
            .retain(|row| row.spell_id != 9);
        assert!(
            generate(&selected, missing)
                .unwrap_err()
                .contains("missing selected")
        );
    }
}
