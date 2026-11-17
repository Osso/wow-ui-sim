//! Bounded `GetAddOnMetadata` cost for a TOC with QuestieDB-scale metadata.
//!
//! QuestieDB's Mists TOC has 408,495 metadata keys. A per-call scan of every
//! key makes 10,000 lookups take tens of seconds; an index keeps them in
//! milliseconds.

use std::fmt::Write as _;
use std::time::{Duration, Instant};
use wow_ui_sim::loader::{find_toc_file, load_addon};
use wow_ui_sim::lua_api::WowLuaEnv;

const METADATA_KEYS: i64 = 400_000;
const LOOKUPS: i64 = 10_000;
/// Generous: the indexed lookups take a few milliseconds in a debug test build.
const LOOKUP_BUDGET: Duration = Duration::from_secs(2);

fn large_toc() -> String {
    let mut toc = String::from("## Title: Large Metadata Fixture\n");
    for key in 0..METADATA_KEYS {
        writeln!(toc, "## X-Key-{key}: value-{key}").unwrap();
    }
    toc
}

fn load_large_addon(env: &WowLuaEnv, root: &std::path::Path) {
    let addon = root.join("LargeMetadata");
    std::fs::create_dir_all(&addon).unwrap();
    std::fs::write(addon.join("LargeMetadata.toc"), large_toc()).unwrap();
    let toc_path = find_toc_file(&addon).unwrap();
    let result = load_addon(&env.loader_env(), &toc_path).unwrap();
    assert!(result.warnings.is_empty(), "{:?}", result.warnings);
}

/// Every fourth field misses; the hits alternate exact and lower-case spelling.
const PREPARE_LOOKUPS_LUA: &str = r#"
MetadataLookupFields = {}
for i = 1, LOOKUPS do
    local key = (i * 7919) % KEYS
    local field
    if i % 4 == 0 then
        field = "X-Missing-" .. key
    elseif i % 2 == 0 then
        field = "x-key-" .. key
    else
        field = "X-Key-" .. key
    end
    MetadataLookupFields[i] = field
end
"#;

const RUN_LOOKUPS_LUA: &str = r#"
local hits, misses = 0, 0
for i = 1, #MetadataLookupFields do
    local value = GetAddOnMetadata("LargeMetadata", MetadataLookupFields[i])
    if value == nil then misses = misses + 1 else hits = hits + 1 end
end
return hits, misses
"#;

fn prepare_lookups(env: &WowLuaEnv) {
    let script = PREPARE_LOOKUPS_LUA
        .replace("LOOKUPS", &LOOKUPS.to_string())
        .replace("KEYS", &METADATA_KEYS.to_string());
    env.exec(&script).unwrap();
}

#[test]
fn metadata_lookups_on_400k_key_toc_stay_under_budget() {
    perf_test_timeout! {
        let root = tempfile::tempdir().unwrap();
        let env = WowLuaEnv::new().unwrap();
        load_large_addon(&env, root.path());
        prepare_lookups(&env);
        let started = Instant::now();
        let (hits, misses): (i64, i64) = env.eval(RUN_LOOKUPS_LUA).unwrap();
        let elapsed = started.elapsed();
        eprintln!("{LOOKUPS} metadata lookups on {METADATA_KEYS} keys: {elapsed:.2?} (budget {LOOKUP_BUDGET:.2?})");
        assert_eq!((hits, misses), (LOOKUPS - LOOKUPS / 4, LOOKUPS / 4));
        assert!(
            elapsed < LOOKUP_BUDGET,
            "{LOOKUPS} metadata lookups took {elapsed:.2?}, exceeding budget {LOOKUP_BUDGET:.2?}"
        );
    }
}
