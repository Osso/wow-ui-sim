//! Runtime access to the selected Forever spell cache.
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use serde::Deserialize;
use serde_json::Value;
use sim_data::spell_power::SpellPowerCost;
use sim_data::spells::SpellInfo;

static CATALOG: OnceLock<Result<Catalog, String>> = OnceLock::new();

#[derive(Deserialize)]
struct FileData {
    schema: u32,
    build: String,
    tables: BTreeMap<String, Table>,
}
#[derive(Deserialize)]
struct Table {
    layout_hash: String,
    unreadable_records: usize,
    rows: Vec<BTreeMap<String, Value>>,
}
struct Catalog {
    spells: BTreeMap<u32, SpellInfo>,
    descriptions: BTreeMap<u32, &'static str>,
    power: BTreeMap<u32, Vec<SpellPowerCost>>,
}

fn data_path() -> Result<PathBuf, String> {
    let exe = std::env::current_exe().map_err(|e| format!("locate executable: {e}"))?;
    let bundled = exe
        .parent()
        .ok_or("executable has no parent")?
        .join("spells/wowforever/spells.json");
    if bundled.is_file() {
        return Ok(bundled);
    }
    dirs::cache_dir()
        .map(|path| path.join("wow-ui-sim/spells/wowforever/spells.json"))
        .ok_or_else(|| "cannot determine Forever spell cache directory".into())
}
fn load() -> Result<Catalog, String> {
    let path = data_path()?;
    load_from(&path)
}
fn load_from(path: &Path) -> Result<Catalog, String> {
    let contents = std::fs::read(path).map_err(|e| {
        format!(
            "cannot read Forever spell data {}: {e}; run `wow-cli casc sync-forever-spells`",
            path.display()
        )
    })?;
    let data: FileData = serde_json::from_slice(&contents)
        .map_err(|e| format!("invalid Forever spell data {}: {e}", path.display()))?;
    parse(data)
}
fn table<'a>(
    data: &'a FileData,
    name: &str,
    hash: &str,
) -> Result<&'a [BTreeMap<String, Value>], String> {
    let table = data
        .tables
        .get(name)
        .ok_or_else(|| format!("Forever spell cache missing {name}"))?;
    if table.layout_hash != hash {
        return Err(format!(
            "Forever spell cache {name} has incorrect layout {}",
            table.layout_hash
        ));
    }
    let _unreadable = table.unreadable_records;
    Ok(&table.rows)
}
fn number(row: &BTreeMap<String, Value>, key: &str) -> Result<i64, String> {
    row.get(key)
        .and_then(Value::as_i64)
        .ok_or_else(|| format!("Forever spell row missing integer {key}"))
}
fn integer(row: &BTreeMap<String, Value>, key: &str) -> Result<u32, String> {
    u32::try_from(number(row, key)?).map_err(|_| format!("Forever spell row has invalid {key}"))
}
fn float(row: &BTreeMap<String, Value>, key: &str) -> Result<f32, String> {
    row.get(key)
        .and_then(Value::as_f64)
        .map(|v| v as f32)
        .ok_or_else(|| format!("Forever spell row missing number {key}"))
}
fn text(row: &BTreeMap<String, Value>, key: &str) -> Result<&'static str, String> {
    let value = row
        .get(key)
        .and_then(Value::as_str)
        .ok_or_else(|| format!("Forever spell row missing string {key}"))?;
    Ok(Box::leak(value.to_owned().into_boxed_str()))
}
fn keyed_rows<'a>(
    rows: &'a [BTreeMap<String, Value>],
    field: &str,
    selected: &BTreeSet<u32>,
) -> Result<BTreeMap<u32, Vec<&'a BTreeMap<String, Value>>>, String> {
    let mut result: BTreeMap<u32, Vec<_>> = BTreeMap::new();
    for row in rows {
        let id = integer(row, field)?;
        if !selected.contains(&id) {
            return Err(format!("Forever spell cache contains unselected ID {id}"));
        }
        result.entry(id).or_default().push(row);
    }
    if result.keys().copied().collect::<BTreeSet<_>>() != *selected {
        return Err(format!("Forever spell cache is missing {field} records"));
    }
    Ok(result)
}
fn one<'a>(
    rows: &'a BTreeMap<u32, Vec<&'a BTreeMap<String, Value>>>,
    id: u32,
    label: &str,
) -> Result<&'a BTreeMap<String, Value>, String> {
    let entries = &rows[&id];
    if entries.len() != 1 {
        return Err(format!("Forever spell {id}: expected one {label} row"));
    }
    Ok(entries[0])
}
fn parse(data: FileData) -> Result<Catalog, String> {
    if data.schema != 1 || data.build != "1.60.1.70685" || data.tables.len() != 5 {
        return Err("unsupported Forever spell cache schema or build".into());
    }
    let selected = crate::forever_spell_data::selected_ids()?;
    let names = keyed_rows(table(&data, "SpellName", "782EE721")?, "ID", &selected)?;
    let texts = keyed_rows(table(&data, "Spell", "E3D134FB")?, "ID", &selected)?;
    let misc = keyed_rows(table(&data, "SpellMisc", "434B3607")?, "SpellID", &selected)?;
    let effects = keyed_rows(
        table(&data, "SpellEffect", "5362E3D4")?,
        "SpellID",
        &selected,
    )?;
    let costs = keyed_rows(
        table(&data, "SpellPower", "61AD223F")?,
        "SpellID",
        &selected,
    )?;
    let mut spells = BTreeMap::new();
    let mut descriptions = BTreeMap::new();
    let mut power = BTreeMap::new();
    for id in selected {
        let name = one(&names, id, "name")?;
        let texts = one(&texts, id, "text")?;
        let misc = misc[&id]
            .iter()
            .find(|r| number(r, "DifficultyID") == Ok(0))
            .ok_or_else(|| format!("Forever spell {id} lacks default misc"))?;
        let effect = effects[&id]
            .iter()
            .find(|r| number(r, "DifficultyID") == Ok(0) && number(r, "EffectIndex") == Ok(0))
            .ok_or_else(|| format!("Forever spell {id} lacks default effect"))?;
        spells.insert(
            id,
            SpellInfo {
                name: text(name, "Name_lang")?,
                subtext: text(texts, "NameSubtext_lang")?,
                icon_file_data_id: integer(misc, "SpellIconFileDataID")?,
                school_mask: integer(misc, "SchoolMask")?,
                implicit_target: u8::try_from(integer(effect, "ImplicitTarget_0")?)
                    .map_err(|_| format!("Forever spell {id} has invalid target"))?,
            },
        );
        descriptions.insert(id, text(texts, "Description_lang")?);
        let mut ordered = costs[&id]
            .iter()
            .map(|row| number(row, "OrderIndex").map(|order| (order, *row)))
            .collect::<Result<Vec<_>, _>>()?;
        ordered.sort_by_key(|(order, _)| *order);
        let entries = ordered
            .iter()
            .map(|(_, row)| {
                Ok(SpellPowerCost {
                    power_type: i8::try_from(number(row, "PowerType")?)
                        .map_err(|_| format!("Forever spell {id} has invalid power type"))?,
                    mana_cost: i32::try_from(number(row, "ManaCost")?)
                        .map_err(|_| format!("Forever spell {id} has invalid mana cost"))?,
                    cost_pct: float(row, "PowerCostPct")?,
                    cost_max_pct: float(row, "PowerCostMaxPct")?,
                    cost_per_sec: number(row, "ManaPerSecond")? as f32,
                    required_aura_id: integer(row, "RequiredAuraSpellID")?,
                    optional_cost: i32::try_from(number(row, "OptionalCost")?)
                        .map_err(|_| format!("Forever spell {id} has invalid optional cost"))?,
                })
            })
            .collect::<Result<Vec<_>, String>>()?;
        power.insert(id, entries);
    }
    Ok(Catalog {
        spells,
        descriptions,
        power,
    })
}
fn catalog() -> Result<&'static Catalog, String> {
    CATALOG.get_or_init(load).as_ref().map_err(Clone::clone)
}
pub fn initialize() -> Result<(), String> {
    catalog().map(|_| ())
}
pub fn spell(id: u32) -> Option<&'static SpellInfo> {
    catalog().unwrap_or_else(|e| panic!("{e}")).spells.get(&id)
}
pub fn description(id: u32) -> Option<&'static str> {
    catalog()
        .unwrap_or_else(|e| panic!("{e}"))
        .descriptions
        .get(&id)
        .copied()
}
pub fn power(id: u32) -> Option<&'static [SpellPowerCost]> {
    catalog()
        .unwrap_or_else(|e| panic!("{e}"))
        .power
        .get(&id)
        .map(Vec::as_slice)
}
pub fn spell_id_by_name(name: &str) -> Option<u32> {
    catalog()
        .unwrap_or_else(|e| panic!("{e}"))
        .spells
        .iter()
        .find_map(|(id, spell)| spell.name.eq_ignore_ascii_case(name).then_some(*id))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn missing_cache_reports_the_generation_command() {
        let dir = tempfile::tempdir().unwrap();
        let error = load_from(&dir.path().join("missing.json")).err().unwrap();
        assert!(error.contains("wow-cli casc sync-forever-spells"));
    }
}
