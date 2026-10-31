//! Cached UIDungeonScoreRarity tiers and native row selection.
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
    #[serde(rename = "DungeonScoreMax")]
    overall_max: i64,
    #[serde(rename = "KeystoneLevelMin")]
    level: i64,
    #[serde(rename = "KeystoneLevelMax")]
    level_max: i64,
    #[serde(rename = "Field_9_2_5_43630_007")]
    run: i64,
    #[serde(rename = "Field_9_2_5_43630_008")]
    run_max: i64,
    #[serde(rename = "Field_9_2_5_43630_009")]
    dungeon: i64,
    #[serde(rename = "Field_9_2_5_43630_010")]
    dungeon_max: i64,
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
    Ok(cache.rows)
}
pub fn color(
    score: f64,
    family: Family,
    passed: &std::collections::HashSet<u32>,
) -> Result<Option<(f64, f64, f64)>, String> {
    let rows = CATALOG.get_or_init(load).as_ref().map_err(Clone::clone)?;
    for row in rows {
        let gate = match row.group {
            0 => true,
            96877 => passed.contains(&248),
            96879 => passed.contains(&121) && !passed.contains(&248),
            id => return Err(format!("unsupported dungeon rarity PlayerCondition {id}")),
        };
        let (min, max) = match family {
            Family::Overall => (row.overall, row.overall_max),
            Family::Level => (row.level, row.level_max),
            Family::Run => (row.run, row.run_max),
            Family::Dungeon => (row.dungeon, row.dungeon_max),
        };
        if gate && min as f64 <= score && (max == 0 || score < max as f64) {
            let color = row.color as u32;
            return Ok(Some((
                ((color >> 16) & 255) as f64 / 255.0,
                ((color >> 8) & 255) as f64 / 255.0,
                (color & 255) as f64 / 255.0,
            )));
        }
    }
    Ok(None)
}
