//! Generate a small profile cache from an authenticated UIDungeonScoreRarity DB2.
use serde::Serialize;
use std::ops::Range;

#[derive(Serialize)]
struct Cache {
    schema: u32,
    profile: String,
    build: String,
    layout_hash: String,
    rows: Vec<std::collections::BTreeMap<String, serde_json::Value>>,
}

pub fn generate(
    profile: &str,
    build: &str,
    bytes: &[u8],
    zero: Vec<Range<usize>>,
) -> Result<Vec<u8>, String> {
    let table = crate::forever_db2::decode_all("UIDungeonScoreRarity", bytes, zero)?;
    if table.unreadable_records != 0 || table.rows.is_empty() {
        return Err("UIDungeonScoreRarity has no complete readable tier data".into());
    }
    let cache = Cache {
        schema: 1,
        profile: profile.to_owned(),
        build: build.to_owned(),
        layout_hash: format!("{:08X}", table.layout_hash),
        rows: table.rows.into_iter().map(|row| row.columns).collect(),
    };
    serde_json::to_vec(&cache)
        .map_err(|error| format!("encode UIDungeonScoreRarity cache: {error}"))
}
