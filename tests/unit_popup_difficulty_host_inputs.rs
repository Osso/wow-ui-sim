#![cfg(feature = "client-retail")]

use crate::common::blizzard_addon_harness::with_blizzard_addon_closure;

const DIFFICULTY_CASES: &str = r##"
local observedForce
CanChangePlayerDifficulty = function() return true end
SetRaidDifficultyID = function(_, force) observedForce = force end
A_Admin.SetInstanceInfo("Legacy Raid", "raid", DifficultyUtil.ID.Raid10Normal, 10, nil, true)
SetRaidDifficulties(true, DifficultyUtil.ID.PrimaryRaidMythic)
assert(select(7, GetInstanceInfo()), "dynamic instance state is returned")
assert(observedForce == true, "dynamic legacy difficulty forces the primary setting")

A_Admin.SetInstanceInfo("Legacy Raid", "raid", DifficultyUtil.ID.Raid10Normal, 10)
SetRaidDifficulties(true, DifficultyUtil.ID.PrimaryRaidMythic)
assert(not select(7, GetInstanceInfo()), "omitted dynamic state defaults to false")
assert(observedForce == nil, "omitted state preserves the static difficulty behavior")
"##;

#[test]
fn unit_popup_difficulty_tracks_supplied_dynamic_instance_state() {
    with_blizzard_addon_closure(&["Blizzard_UnitPopup"], &[], |env, loaded| {
        assert!(loaded.iter().any(|addon| addon == "Blizzard_UnitPopup"));
        env.exec(DIFFICULTY_CASES)
            .expect("original UnitPopup difficulty consumer should use supplied instance state");
        let invalid = env.exec(
            "A_Admin.SetInstanceInfo('Legacy Raid', 'raid', 3, 10, nil, 1)",
        );
        assert!(invalid.is_err(), "dynamic instance state must be a boolean");
    });
}
