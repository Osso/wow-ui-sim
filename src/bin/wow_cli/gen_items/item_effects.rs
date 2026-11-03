//! Full profile-pinned item effects, sparse metadata, and source provenance.

use super::super::csv_util::{escape_str, parse_csv_line, read_csv_records};
use super::{field, header_index};
use clap::ValueEnum;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::fs::File;
use std::io::{BufReader, Write};
use std::path::Path;

const RETAIL_BASELINE_COMMIT: &str = "d1196bd7dd0e25de561d342055f92c6f51ab0dfb";
const RETAIL_BASELINE_BLOB: &str = "edd4b70f0002179d463438df7ee1a60799fc37fc";
const RETAIL_BASELINE_PATH: &str = "crates/sim-data/data/item_effects.rs";
const RETAIL_BASELINE_SHA256: &str =
    "0c11a5ef78c3864c9adeb19000878c7ce7cc419279138fc47040cc94bb0ff1c2";
const RETAIL_BASELINE_URL: &str = "https://raw.githubusercontent.com/Osso/wow-ui-sim/d1196bd7dd0e25de561d342055f92c6f51ab0dfb/crates/sim-data/data/item_effects.rs";

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
type EffectRow = (u32, u32, i32, i32, i32, u32, u32);
type MetadataRow = (u32, u32, u32);

#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
pub(super) enum Profile {
    Retail,
    Ptr,
    Wrath,
    Mists,
    Era,
    Anniversary,
    #[value(name = "wowforever")]
    WowForever,
}

impl Profile {
    fn name(self) -> &'static str {
        match self {
            Self::Retail => "retail",
            Self::Ptr => "ptr",
            Self::Wrath => "wrath",
            Self::Mists => "mists",
            Self::Era => "era",
            Self::Anniversary => "anniversary",
            Self::WowForever => "wowforever",
        }
    }

    fn has_inline_item_effects(self) -> bool {
        matches!(
            self,
            Self::Wrath | Self::Mists | Self::Era | Self::Anniversary
        )
    }
}

struct InputTable {
    records: Vec<String>,
    source: Value,
}

#[derive(Debug)]
struct Catalog {
    rows: Vec<EffectRow>,
    names: BTreeMap<u32, String>,
    unresolved: Vec<(u32, u32)>,
    source_associations: usize,
    has_specialization_ids: bool,
    has_player_condition_ids: bool,
}

pub(super) fn run(
    profile: Profile,
    csv_dir: Option<&Path>,
    build_override: Option<&str>,
    retail_baseline: Option<&Path>,
) -> Result<()> {
    if retail_baseline.is_some() && profile != Profile::Retail {
        return Err("--retail-baseline is only valid with --profile retail".into());
    }
    let profile_name = profile.name();
    let build = resolve_build(profile, build_override)?;
    let effects = read_table("ItemEffect", &build, csv_dir)?;
    let relations = if profile.has_inline_item_effects() {
        None
    } else {
        Some(read_table("ItemXItemEffect", &build, csv_dir)?)
    };
    let names = read_table("SpellName", &build, csv_dir)?;
    let mut catalog = build_catalog(
        &effects.records,
        relations.as_ref().map(|table| table.records.as_slice()),
        &names.records,
    )?;
    let source_effect_rows = catalog.rows.len();
    let mut sources = vec![effects.source];
    if let Some(relations) = relations {
        sources.push(relations.source);
    }
    sources.push(names.source);
    let baseline = if profile == Profile::Retail {
        let (preserved, source) = read_retail_baseline(retail_baseline)?;
        let receipt = merge_retail_baseline(&mut catalog, preserved)?;
        Some(json!({"source": source, "merge": receipt}))
    } else {
        None
    };
    catalog.rows.sort_unstable();
    catalog.unresolved.sort_unstable();
    let used_spells: BTreeSet<u32> = catalog
        .rows
        .iter()
        .filter_map(|row| u32::try_from(row.2).ok())
        .collect();
    catalog.names.retain(|id, _| used_spells.contains(id));
    let metadata = sparse_metadata(&catalog.rows)?;
    let missing_names: BTreeSet<u32> = used_spells
        .iter()
        .copied()
        .filter(|id| !catalog.names.contains_key(id))
        .collect();
    let missing_effects: BTreeSet<u32> = catalog.unresolved.iter().map(|row| row.1).collect();
    let output = format!("crates/sim-data-profile/data/item_effects_{profile_name}.rs");
    let command = format!("wow-cli generate items --profile {profile_name} --build {build}");
    let mut provenance = json!({
        "profile": profile_name,
        "build": build,
        "sources": sources,
        "association_source": if profile.has_inline_item_effects() {
            "ItemEffect.ParentItemID"
        } else {
            "ItemXItemEffect"
        },
        "source_association_rows": catalog.source_associations,
        "resolved_source_effect_rows": source_effect_rows,
        "output_effect_rows": catalog.rows.len(),
        "output_metadata_rows": metadata.len(),
        "output_spell_name_rows": catalog.names.len(),
        "unresolved_associations": catalog.unresolved,
        "missing_item_effect_ids": missing_effects,
        "missing_spell_name_ids": missing_names,
        "metadata_columns": {
            "ChrSpecializationID": catalog.has_specialization_ids,
            "PlayerConditionID": catalog.has_player_condition_ids,
        },
        "sparse_metadata": {
            "key": "ItemEffect.ID",
            "columns": ["ChrSpecializationID", "PlayerConditionID"],
            "omitted_available_values": 0,
            "unavailable_columns": "See metadata_columns; zero placeholders are not captured values.",
        },
        "output": output,
        "generator": command,
    });
    if let Some(baseline) = baseline {
        provenance["retail_preservation"] = baseline;
    }
    std::fs::create_dir_all("crates/sim-data-profile/data")?;
    write_catalog(
        &mut File::create(&output)?,
        profile_name,
        &build,
        &catalog,
        &metadata,
    )?;
    std::fs::create_dir_all("data/item-effects")?;
    std::fs::write(
        format!("data/item-effects/{profile_name}-provenance.json"),
        serde_json::to_string_pretty(&provenance)? + "\n",
    )?;
    println!(
        "{profile_name} {build}: {} effect rows, {} sparse metadata rows, {} spell names; {} unresolved associations",
        catalog.rows.len(),
        metadata.len(),
        catalog.names.len(),
        catalog.unresolved.len(),
    );
    println!("Output: {output}");
    Ok(())
}

fn resolve_build(profile: Profile, build_override: Option<&str>) -> Result<String> {
    match build_override {
        Some(build) => Ok(build.to_owned()),
        None if profile.has_inline_item_effects() => Err(format!(
            "{} has no pinned item-data build; pass --build",
            profile.name()
        )
        .into()),
        None => {
            let pin: Value = serde_json::from_slice(&std::fs::read(format!(
                "data/blizzard-ui-builds/{}.json",
                profile.name()
            ))?)?;
            Ok(pin["version"]
                .as_str()
                .ok_or("profile pin has no version")?
                .to_owned())
        }
    }
}

fn download(url: &str) -> Result<Vec<u8>> {
    Ok(reqwest::blocking::Client::builder()
        .user_agent("wow-ui-sim item-data generator")
        .build()?
        .get(url)
        .send()?
        .error_for_status()?
        .bytes()?
        .to_vec())
}

fn read_table(table: &str, build: &str, csv_dir: Option<&Path>) -> Result<InputTable> {
    let url = format!("https://wago.tools/db2/{table}/csv?build={build}&locale=enUS");
    let raw = match csv_dir {
        Some(directory) => std::fs::read(directory.join(format!("{table}.csv")))?,
        None => download(&url)?,
    };
    parse_table(table, &url, &raw)
}

fn parse_table(table: &str, url: &str, raw: &[u8]) -> Result<InputTable> {
    let csv = raw.strip_prefix(b"\xef\xbb\xbf").unwrap_or(raw);
    let mut records = read_csv_records(BufReader::new(csv))?;
    records.retain(|record| !record.is_empty());
    if records.is_empty() {
        return Err(format!("empty {table}.csv").into());
    }
    let source = json!({
        "table": table,
        "url": url,
        "sha256": sha256(raw),
        "bytes": raw.len(),
        "rows": records.len() - 1,
        "columns": parse_csv_line(&records[0]),
    });
    Ok(InputTable { records, source })
}

fn sha256(raw: &[u8]) -> String {
    format!("{:x}", Sha256::digest(raw))
}

fn require_columns(table: &str, index: &HashMap<String, usize>, columns: &[&str]) -> Result<()> {
    for column in columns {
        if !index.contains_key(*column) {
            return Err(format!("{table}.csv is missing {column}").into());
        }
    }
    Ok(())
}

fn build_catalog(
    effect_records: &[String],
    relation_records: Option<&[String]>,
    name_records: &[String],
) -> Result<Catalog> {
    let index = header_index(effect_records.first().ok_or("empty ItemEffect.csv")?);
    require_columns(
        "ItemEffect",
        &index,
        &["ID", "SpellID", "TriggerType", "LegacySlotIndex"],
    )?;
    let has_specialization_ids = index.contains_key("ChrSpecializationID");
    let has_player_condition_ids = index.contains_key("PlayerConditionID");
    let mut effects = HashMap::new();
    for record in &effect_records[1..] {
        let fields = parse_csv_line(record);
        let id = field(&fields, &index, "ID").parse::<u32>()?;
        let specialization = if has_specialization_ids {
            field(&fields, &index, "ChrSpecializationID").parse::<u32>()?
        } else {
            0
        };
        let condition = if has_player_condition_ids {
            field(&fields, &index, "PlayerConditionID").parse::<u32>()?
        } else {
            0
        };
        let effect = (
            field(&fields, &index, "SpellID").parse::<i32>()?,
            field(&fields, &index, "TriggerType").parse::<i32>()?,
            field(&fields, &index, "LegacySlotIndex").parse::<i32>()?,
            specialization,
            condition,
        );
        if effects.insert(id, effect).is_some() {
            return Err(format!("duplicate ItemEffect.ID {id}").into());
        }
    }
    let index = header_index(name_records.first().ok_or("empty SpellName.csv")?);
    require_columns("SpellName", &index, &["ID", "Name_lang"])?;
    let mut names = BTreeMap::new();
    for record in &name_records[1..] {
        let fields = parse_csv_line(record);
        let id = field(&fields, &index, "ID").parse::<u32>()?;
        let name = field(&fields, &index, "Name_lang").to_owned();
        if names.insert(id, name).is_some() {
            return Err(format!("duplicate SpellName.ID {id}").into());
        }
    }

    let (associations, effect_column, item_column, table) = match relation_records {
        Some(records) => (records, "ItemEffectID", "ItemID", "ItemXItemEffect"),
        None => (effect_records, "ID", "ParentItemID", "ItemEffect"),
    };
    let index = header_index(
        associations
            .first()
            .ok_or_else(|| format!("empty {table}.csv"))?,
    );
    require_columns(table, &index, &[item_column, effect_column])?;
    let mut rows = Vec::new();
    let mut unresolved = Vec::new();
    let mut source_associations = 0;
    for record in &associations[1..] {
        let fields = parse_csv_line(record);
        let item = field(&fields, &index, item_column).parse::<u32>()?;
        if relation_records.is_none() && item == 0 {
            continue;
        }
        let effect = field(&fields, &index, effect_column).parse::<u32>()?;
        source_associations += 1;
        match effects.get(&effect) {
            Some(&(spell, trigger, slot, specialization, condition)) => {
                rows.push((
                    item,
                    effect,
                    spell,
                    trigger,
                    slot,
                    specialization,
                    condition,
                ));
            }
            None => unresolved.push((item, effect)),
        }
    }
    rows.sort_unstable();
    Ok(Catalog {
        rows,
        names,
        unresolved,
        source_associations,
        has_specialization_ids,
        has_player_condition_ids,
    })
}

fn sparse_metadata(rows: &[EffectRow]) -> Result<Vec<MetadataRow>> {
    let mut metadata = BTreeMap::new();
    for &(_, effect, _, _, _, specialization, condition) in rows {
        let value = (specialization, condition);
        if let Some(previous) = metadata.insert(effect, value)
            && previous != value
        {
            return Err(format!("conflicting metadata for ItemEffect.ID {effect}").into());
        }
    }
    Ok(metadata
        .into_iter()
        .filter_map(|(effect, (specialization, condition))| {
            (specialization != 0 || condition != 0).then_some((effect, specialization, condition))
        })
        .collect())
}

fn read_retail_baseline(path: Option<&Path>) -> Result<(Catalog, Value)> {
    let raw = match path {
        Some(path) => std::fs::read(path)?,
        None => download(RETAIL_BASELINE_URL)?,
    };
    // Git checkouts may use CRLF; the immutable source and its hash use LF.
    let normalized = std::str::from_utf8(&raw)?.replace("\r\n", "\n");
    if sha256(normalized.as_bytes()) != RETAIL_BASELINE_SHA256 {
        return Err("Retail baseline does not match the pinned historical mapping".into());
    }
    let catalog = parse_retail_baseline(&normalized)?;
    let source = json!({
        "url": RETAIL_BASELINE_URL,
        "commit": RETAIL_BASELINE_COMMIT,
        "git_blob": RETAIL_BASELINE_BLOB,
        "path": RETAIL_BASELINE_PATH,
        "sha256": RETAIL_BASELINE_SHA256,
        "bytes": normalized.len(),
        "normalization": "CRLF to LF",
        "effect_rows": catalog.rows.len(),
        "spell_name_rows": catalog.names.len(),
    });
    Ok((catalog, source))
}

fn parse_retail_baseline(source: &str) -> Result<Catalog> {
    // The pinned generator emitted JSON-compatible numeric/string tuple contents.
    // Parse that exact historical format as data; never execute the source.
    let mut array = "";
    let mut rows = Vec::new();
    let mut names = BTreeMap::new();
    for line in source.lines() {
        if line.starts_with("pub static ITEM_EFFECTS:") {
            array = "effects";
        } else if line.starts_with("pub static ITEM_SPELL_NAMES:") {
            array = "names";
        } else if let Some(tuple) = line
            .trim()
            .strip_prefix('(')
            .and_then(|line| line.strip_suffix("),"))
        {
            let json = format!("[{tuple}]");
            match array {
                "effects" => {
                    let (item, effect, spell, trigger, slot, specialization, condition): (
                        u32,
                        u32,
                        u32,
                        i32,
                        i32,
                        u32,
                        u32,
                    ) = serde_json::from_str(&json)?;
                    rows.push((
                        item,
                        effect,
                        i32::try_from(spell)?,
                        trigger,
                        slot,
                        specialization,
                        condition,
                    ));
                }
                "names" => {
                    let (spell, name): (u32, String) = serde_json::from_str(&json)?;
                    if names.insert(spell, name).is_some() {
                        return Err(format!("duplicate historical spell name {spell}").into());
                    }
                }
                _ => return Err("tuple outside the historical item arrays".into()),
            }
        }
    }
    if rows.is_empty() || names.is_empty() {
        return Err("historical Retail mapping has missing arrays".into());
    }
    let source_associations = rows.len();
    Ok(Catalog {
        rows,
        names,
        unresolved: Vec::new(),
        source_associations,
        has_specialization_ids: true,
        has_player_condition_ids: true,
    })
}

fn merge_retail_baseline(catalog: &mut Catalog, baseline: Catalog) -> Result<Value> {
    for (column, source_available, baseline_available) in [
        (
            "ChrSpecializationID",
            catalog.has_specialization_ids,
            baseline.has_specialization_ids,
        ),
        (
            "PlayerConditionID",
            catalog.has_player_condition_ids,
            baseline.has_player_condition_ids,
        ),
    ] {
        if source_available != baseline_available {
            return Err(format!(
                "Retail ItemEffect.{column} availability does not match the historical baseline"
            )
            .into());
        }
    }
    let mut preserved = BTreeMap::new();
    for row in baseline.rows {
        if preserved.insert((row.0, row.1), row).is_some() {
            return Err("duplicate historical Retail item/effect association".into());
        }
    }
    let source_keys: HashSet<_> = catalog.rows.iter().map(|row| (row.0, row.1)).collect();
    let mut restored_rows = 0;
    for row in &mut catalog.rows {
        if let Some(old) = preserved.get(&(row.0, row.1))
            && *row != *old
        {
            *row = *old;
            restored_rows += 1;
        }
    }
    let added_rows = preserved
        .iter()
        .filter(|(key, _)| !source_keys.contains(*key))
        .map(|(_, row)| *row)
        .collect::<Vec<_>>();
    let added_count = added_rows.len();
    catalog.rows.extend(added_rows);
    catalog
        .unresolved
        .retain(|association| !preserved.contains_key(association));
    let mut restored_names = 0;
    let mut added_names = 0;
    for (spell, name) in baseline.names {
        match catalog.names.insert(spell, name.clone()) {
            Some(current) if current != name => restored_names += 1,
            None => added_names += 1,
            _ => {}
        }
    }
    Ok(json!({
        "policy": "Preserve historical association values and spell-name spelling; retain all new source associations.",
        "historical_associations_added": added_count,
        "source_associations_restored": restored_rows,
        "historical_names_added": added_names,
        "source_names_restored": restored_names,
    }))
}

fn write_catalog(
    out: &mut impl Write,
    profile: &str,
    build: &str,
    catalog: &Catalog,
    metadata: &[MetadataRow],
) -> std::io::Result<()> {
    writeln!(
        out,
        "//! Generated from wago.tools DB2 CSV exports for {profile}, build {build} (enUS)."
    )?;
    writeln!(
        out,
        "//! Regenerate with `wow-cli generate items --profile {profile} --build {build}`."
    )?;
    if profile == "retail" {
        writeln!(
            out,
            "//! Includes the hash-pinned historical Retail preservation input."
        )?;
    }
    writeln!(
        out,
        "pub const HAS_SPECIALIZATION_IDS: bool = {};",
        catalog.has_specialization_ids
    )?;
    writeln!(
        out,
        "pub const HAS_PLAYER_CONDITION_IDS: bool = {};",
        catalog.has_player_condition_ids
    )?;
    writeln!(out, "\n#[rustfmt::skip]")?;
    writeln!(
        out,
        "pub static ITEM_EFFECTS: &[(u32, u32, i32, i32, i32)] = &["
    )?;
    for &(item, effect, spell, trigger, slot, _, _) in &catalog.rows {
        writeln!(out, "    ({item}, {effect}, {spell}, {trigger}, {slot}),")?;
    }
    writeln!(out, "];\n\n#[rustfmt::skip]")?;
    writeln!(
        out,
        "pub static ITEM_EFFECT_METADATA: &[(u32, u32, u32)] = &["
    )?;
    for &(effect, specialization, condition) in metadata {
        writeln!(out, "    ({effect}, {specialization}, {condition}),")?;
    }
    writeln!(out, "];\n\n#[rustfmt::skip]")?;
    writeln!(out, "pub static ITEM_SPELL_NAMES: &[(u32, &str)] = &[")?;
    for (spell, name) in &catalog.names {
        writeln!(out, "    ({spell}, \"{}\"),", escape_str(name))?;
    }
    writeln!(out, "];")
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    fn records(source: &str) -> Vec<String> {
        read_csv_records(BufReader::new(source.as_bytes())).unwrap()
    }

    #[test]
    fn all_effect_rows_survive_with_signed_spells_missing_names_and_sparse_metadata() {
        let effects = records(
            "ID,SpellID,TriggerType,LegacySlotIndex,ChrSpecializationID,PlayerConditionID\n1,100,1,0,0,0\n2,200,0,2,71,0\n3,-1,0,1,0,9\n4,0,0,9,0,0\n",
        );
        let relations = records("ItemID,ItemEffectID\n10,1\n10,2\n10,2\n20,3\n30,4\n40,99\n");
        let names = records("ID,Name_lang\n100,Passive\n200,\n");
        let catalog = build_catalog(&effects, Some(&relations), &names).unwrap();
        assert_eq!(
            catalog.rows,
            [
                (10, 1, 100, 1, 0, 0, 0),
                (10, 2, 200, 0, 2, 71, 0),
                (10, 2, 200, 0, 2, 71, 0),
                (20, 3, -1, 0, 1, 0, 9),
                (30, 4, 0, 0, 9, 0, 0),
            ]
        );
        assert_eq!(catalog.names[&200], "");
        assert_eq!(catalog.unresolved, [(40, 99)]);
        assert_eq!(catalog.source_associations, 6);
        assert_eq!(
            sparse_metadata(&catalog.rows).unwrap(),
            [(2, 71, 0), (3, 0, 9)]
        );
        assert!(catalog.has_specialization_ids && catalog.has_player_condition_ids);
    }

    #[test]
    fn classic_inline_rows_preserve_passive_and_negative_effects_and_absent_columns() {
        let effects = records(
            "ID,SpellID,TriggerType,LegacySlotIndex,ChrSpecializationID,ParentItemID\n1,100,1,0,0,10\n2,-1,0,2,0,10\n3,300,0,1,0,0\n",
        );
        let names = records("ID,Name_lang\n100,Passive\n300,Unassociated\n");
        let catalog = build_catalog(&effects, None, &names).unwrap();
        assert_eq!(
            catalog.rows,
            [(10, 1, 100, 1, 0, 0, 0), (10, 2, -1, 0, 2, 0, 0)]
        );
        assert!(catalog.has_specialization_ids);
        assert!(!catalog.has_player_condition_ids);
        assert!(sparse_metadata(&catalog.rows).unwrap().is_empty());
        assert_eq!(catalog.source_associations, 2);
    }

    #[test]
    fn duplicate_effect_records_and_conflicting_sparse_values_are_rejected() {
        let effects = records("ID,SpellID,TriggerType,LegacySlotIndex\n1,100,0,0\n1,200,0,1\n");
        let names = records("ID,Name_lang\n100,Name\n");
        let relations = records("ItemID,ItemEffectID\n10,1\n");
        assert_eq!(
            build_catalog(&effects, Some(&relations), &names)
                .unwrap_err()
                .to_string(),
            "duplicate ItemEffect.ID 1"
        );
        assert_eq!(
            sparse_metadata(&[(10, 1, 100, 0, 0, 0, 0), (20, 1, 100, 0, 0, 71, 0)])
                .unwrap_err()
                .to_string(),
            "conflicting metadata for ItemEffect.ID 1"
        );
    }

    #[test]
    fn historical_retail_preservation_keeps_old_rows_names_and_all_new_associations() {
        let old = "pub static ITEM_EFFECTS: &[(u32,u32,u32,i32,i32,u32,u32)] = &[\n    (10, 1, 100, 0, 0, 71, 9),\n    (30, 3, 300, 1, 0, 0, 0),\n];\npub static ITEM_SPELL_NAMES: &[(u32,&str)] = &[\n    (100, \"Old\\nspelling\"),\n    (300, \"Preserved\"),\n];\n";
        let effects = records(
            "ID,SpellID,TriggerType,LegacySlotIndex,ChrSpecializationID,PlayerConditionID\n1,101,0,0,0,0\n2,200,0,0,0,0\n",
        );
        let relations = records("ItemID,ItemEffectID\n10,1\n20,2\n20,2\n");
        let names = records("ID,Name_lang\n100,Changed\n101,New\n200,New association\n");
        let mut catalog = build_catalog(&effects, Some(&relations), &names).unwrap();
        let receipt =
            merge_retail_baseline(&mut catalog, parse_retail_baseline(old).unwrap()).unwrap();
        catalog.rows.sort_unstable();
        assert_eq!(
            catalog.rows,
            [
                (10, 1, 100, 0, 0, 71, 9),
                (20, 2, 200, 0, 0, 0, 0),
                (20, 2, 200, 0, 0, 0, 0),
                (30, 3, 300, 1, 0, 0, 0),
            ]
        );
        assert_eq!(catalog.names[&100], "Old\nspelling");
        assert_eq!(receipt["historical_associations_added"], 1);
        assert_eq!(receipt["source_associations_restored"], 1);
        assert_eq!(receipt["source_names_restored"], 1);
    }

    #[test]
    fn retail_preservation_rejects_unknown_metadata_on_new_associations() {
        let names = records("ID,Name_lang\n100,Preserved\n200,New association\n");
        let relations = records("ItemID,ItemEffectID\n20,2\n");
        let baseline_effects = records(
            "ID,SpellID,TriggerType,LegacySlotIndex,ChrSpecializationID,PlayerConditionID\n1,100,0,0,71,9\n",
        );
        let baseline_relations = records("ItemID,ItemEffectID\n10,1\n");
        for (column, source) in [
            (
                "ChrSpecializationID",
                "ID,SpellID,TriggerType,LegacySlotIndex,PlayerConditionID\n2,200,0,0,0\n",
            ),
            (
                "PlayerConditionID",
                "ID,SpellID,TriggerType,LegacySlotIndex,ChrSpecializationID\n2,200,0,0,0\n",
            ),
        ] {
            let mut catalog = build_catalog(&records(source), Some(&relations), &names).unwrap();
            let original_rows = catalog.rows.clone();
            let baseline =
                build_catalog(&baseline_effects, Some(&baseline_relations), &names).unwrap();
            assert_eq!(
                merge_retail_baseline(&mut catalog, baseline)
                    .unwrap_err()
                    .to_string(),
                format!(
                    "Retail ItemEffect.{column} availability does not match the historical baseline"
                )
            );
            assert_eq!(catalog.rows, original_rows);
        }
    }

    #[test]
    fn provenance_and_generated_text_preserve_quoted_source_bytes() {
        let raw = b"\xef\xbb\xbfID,Name_lang\r\n1,\"First\r\nSecond\"\r\n";
        let table = parse_table("SpellName", "https://example.invalid/pinned", raw).unwrap();
        assert_eq!(table.source["bytes"], raw.len());
        assert_eq!(table.source["rows"], 1);
        assert_eq!(
            table.source["sha256"],
            "62b198479d90fef7c4b57d1b6e611bf422ff4a39fc4cc6e7b9e02244e18c3eae"
        );
        assert_eq!(table.source["columns"], json!(["ID", "Name_lang"]));
        let catalog = Catalog {
            rows: vec![(10, 1, 100, 1, 0, 0, 0)],
            names: BTreeMap::from([(100, "First\r\n\"Second\"".to_owned())]),
            unresolved: vec![],
            source_associations: 1,
            has_specialization_ids: false,
            has_player_condition_ids: false,
        };
        let mut output = Vec::new();
        write_catalog(&mut output, "wrath", "3.4.3.54261", &catalog, &[]).unwrap();
        let text = String::from_utf8(output).unwrap();
        assert!(text.contains("HAS_PLAYER_CONDITION_IDS: bool = false"));
        assert!(text.contains("(10, 1, 100, 1, 0),"));
        assert!(text.contains(r#"(100, "First\r\n\"Second\""),"#));
    }

    #[test]
    fn missing_required_name_and_classic_association_columns_are_rejected() {
        let effects = records("ID,SpellID,TriggerType,LegacySlotIndex\n1,100,0,0\n");
        let names = records("ID,Name\n100,Unrecognized\n");
        assert_eq!(
            build_catalog(&effects, None, &names)
                .unwrap_err()
                .to_string(),
            "SpellName.csv is missing Name_lang"
        );
        let names = records("ID,Name_lang\n100,Named\n");
        assert_eq!(
            build_catalog(&effects, None, &names)
                .unwrap_err()
                .to_string(),
            "ItemEffect.csv is missing ParentItemID"
        );
    }

    #[test]
    fn classic_sources_require_explicit_data_builds() {
        for profile in [
            Profile::Wrath,
            Profile::Mists,
            Profile::Era,
            Profile::Anniversary,
        ] {
            assert_eq!(
                resolve_build(profile, None).unwrap_err().to_string(),
                format!(
                    "{} has no pinned item-data build; pass --build",
                    profile.name()
                )
            );
            assert_eq!(
                resolve_build(profile, Some("1.2.3.45678")).unwrap(),
                "1.2.3.45678"
            );
        }
    }

    #[test]
    fn profile_source_options_require_an_explicit_profile() {
        #[derive(Parser)]
        struct Cli {
            #[command(flatten)]
            options: super::super::Options,
        }
        assert!(
            Cli::try_parse_from(["items"])
                .unwrap()
                .options
                .profile
                .is_none()
        );
        assert!(Cli::try_parse_from(["items", "--csv-dir", "cached"]).is_err());
        assert!(Cli::try_parse_from(["items", "--build", "1.60.1.70532"]).is_err());
        assert!(Cli::try_parse_from(["items", "--retail-baseline", "old.rs"]).is_err());
        let parsed = Cli::try_parse_from([
            "items",
            "--profile",
            "retail",
            "--build",
            "12.1.5.70575",
            "--retail-baseline",
            "old.rs",
        ])
        .unwrap();
        assert_eq!(parsed.options.profile, Some(Profile::Retail));
        assert_eq!(
            parsed.options.retail_baseline.as_deref(),
            Some(Path::new("old.rs"))
        );
    }
}
