#![cfg(feature = "client-retail")]

use crate::common::blizzard_addon_harness::with_blizzard_addon_closure;

const DIFFICULTY_CASES: &str = r##"
local observedForce
CanChangePlayerDifficulty = function() return true end
SetRaidDifficultyID = function(_, force) observedForce = force end
A_Admin.SetInstanceInfo("Legacy Raid", "raid", DifficultyUtil.ID.Raid10Normal, 10, nil, false)
SetRaidDifficulties(true, DifficultyUtil.ID.PrimaryRaidMythic)
assert(not select(7, GetInstanceInfo()), "static instance state is returned")
assert(observedForce == nil, "static legacy difficulty does not force the primary setting")

A_Admin.SetInstanceInfo("Legacy Raid", "raid", DifficultyUtil.ID.Raid10Normal, 10, nil, true)
SetRaidDifficulties(true, DifficultyUtil.ID.PrimaryRaidMythic)
assert(select(7, GetInstanceInfo()), "dynamic instance state is returned")
assert(observedForce == true, "dynamic legacy difficulty forces the primary setting")
"##;

#[test]
fn unit_popup_difficulty_tracks_supplied_dynamic_instance_state() {
    with_blizzard_addon_closure(&["Blizzard_UnitPopup"], &[], |env, loaded| {
        assert!(loaded.iter().any(|addon| addon == "Blizzard_UnitPopup"));
        env.exec(DIFFICULTY_CASES)
            .expect("original UnitPopup difficulty consumer should use supplied instance state");
    });
}
