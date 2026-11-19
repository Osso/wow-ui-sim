//! Bounded `bit.band` cost at Questie login scale.
//!
//! Questie's `Townsfolk` build calls `bit.band(npcFlags, mask)` for every NPC
//! and townsfolk type during its login coroutine. With the former Lua
//! fallback that loop took about 15 s of Questie's Mists login; a native
//! library keeps 200,000 calls far inside the budget.

use std::time::{Duration, Instant};
use wow_ui_sim::lua_api::WowLuaEnv;

const CALLS: u64 = 200_000;
/// Generous: the native calls take a small fraction of this in a debug test build.
const BUDGET: Duration = Duration::from_secs(2);
const MASKS: [u64; 8] = [1, 2, 16, 128, 4096, 65536, 1 << 20, 1 << 31];

const RUN_LUA: &str = r#"
local masks = {1, 2, 16, 128, 4096, 65536, 2^20, 2^31}
local band, matches = bit.band, 0
for i = 1, CALLS do
    local mask = masks[i % 8 + 1]
    if band(i * 2654435761 % 4294967296, mask) == mask then matches = matches + 1 end
end
return matches
"#;

fn expected_matches() -> i64 {
    (1..=CALLS)
        .filter(|i| {
            let mask = MASKS[(i % 8) as usize];
            (i * 2_654_435_761 % 4_294_967_296) & mask == mask
        })
        .count() as i64
}

#[test]
fn questie_scale_bit_band_calls_stay_under_budget() {
    perf_test_timeout! {
        let env = WowLuaEnv::new().unwrap();
        let script = RUN_LUA.replace("CALLS", &CALLS.to_string());
        let started = Instant::now();
        let matches: i64 = env.eval(&script).unwrap();
        let elapsed = started.elapsed();
        eprintln!("{CALLS} bit.band calls: {elapsed:.2?} (budget {BUDGET:.2?})");
        assert_eq!(matches, expected_matches());
        assert!(
            elapsed < BUDGET,
            "{CALLS} bit.band calls took {elapsed:.2?}, exceeding budget {BUDGET:.2?}"
        );
    }
}
