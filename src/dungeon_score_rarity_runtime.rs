//! Cached UIDungeonScoreRarity tiers; native selection semantics remain unknown.
use serde::Deserialize;
use std::{path::PathBuf, sync::OnceLock};

#[derive(Deserialize)]
struct Cache {
    schema: u32,
    profile: String,
    build: String,
    layout_hash: String,
    rows: Vec<Row>,
}
#[derive(Deserialize)]
pub struct Row {
    #[serde(rename = "ScoreColor")]
    color: i64,
    #[serde(rename = "DungeonScoreMin")]
    overall: i64,
    #[serde(rename = "KeystoneLevelMin")]
    level: i64,
    #[serde(rename = "Field_9_2_5_43630_007")]
    run: i64,
    #[serde(rename = "Field_9_2_5_43630_009")]
    dungeon: i64,
    #[serde(rename = "Field_9_2_5_43630_011")]
    group: i64,
}
#[derive(Clone, Copy)]
pub enum Family {
    Overall,
    Level,
    Run,
    Dungeon,
}
static CATALOG: OnceLock<Result<Vec<Row>, String>> = OnceLock::new();

fn path(profile: &str) -> Result<PathBuf, String> {
    let exe = std::env::current_exe().map_err(|e| format!("locate executable: {e}"))?;
    let bundled = exe
        .parent()
        .ok_or("executable has no parent")?
        .join("dungeon-score-rarity")
        .join(profile)
        .join("rarity.json");
    if bundled.is_file() {
        return Ok(bundled);
    }
    dirs::cache_dir()
        .map(|p| {
            p.join("wow-ui-sim/dungeon-score-rarity")
                .join(profile)
                .join("rarity.json")
        })
        .ok_or_else(|| "cannot locate dungeon score rarity cache".into())
}
fn load() -> Result<Vec<Row>, String> {
    let profile = crate::client_profile::ACTIVE.cache_subdir();
    let filename = path(profile)?;
    let bytes = std::fs::read(&filename).map_err(|e| format!("cannot read dungeon score rarity cache {}: {e}; run wow-cli casc sync-dungeon-score-rarity", filename.display()))?;
    let cache: Cache = serde_json::from_slice(&bytes)
        .map_err(|e| format!("invalid dungeon score rarity cache: {e}"))?;
    let build = crate::dungeon_score_rarity_pin::active()?.build;
    if cache.schema != 1
        || cache.profile != profile
        || cache.layout_hash != "6242874E"
        || cache.build != build
        || cache.rows.is_empty()
    {
        return Err("invalid dungeon score rarity cache identity or empty rows".into());
    }
    let max = cache
        .rows
        .iter()
        .map(|row| row.group)
        .max()
        .ok_or("empty rarity table")?;
    Ok(cache
        .rows
        .into_iter()
        .filter(|row| row.group == max)
        .collect())
}
pub fn color(score: f64, family: Family) -> Result<(f64, f64, f64), String> {
    let rows = CATALOG.get_or_init(load).as_ref().map_err(Clone::clone)?;
    let threshold = |row: &Row| match family {
        Family::Overall => row.overall,
        Family::Level => row.level,
        Family::Run => row.run,
        Family::Dungeon => row.dungeon,
    };
    let row = rows
        .iter()
        .filter(|row| threshold(row) as f64 <= score)
        .max_by_key(|row| threshold(row))
        .or_else(|| rows.iter().min_by_key(|row| threshold(row)))
        .ok_or("empty rarity set")?;
    let color = row.color as u32;
    Ok((
        ((color >> 16) & 255) as f64 / 255.0,
        ((color >> 8) & 255) as f64 / 255.0,
        (color & 255) as f64 / 255.0,
    ))
}
