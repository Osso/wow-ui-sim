#![cfg(feature = "client-retail")]

use crate::common::blizzard_addon_harness::with_blizzard_addon_closure;

const DIFFICULTY_CASES: &str = r##"
local primaryNormal = DifficultyUtil.ID.PrimaryRaidNormal
local function primary_raid_normal_checked()
    local button = { GetDifficultyID = function() return primaryNormal end }
    return UnitPopupRaidDifficulty1ButtonMixin:IsChecked(button, {})
end

A_Admin.SetInstanceInfo("Legacy Raid", "raid", DifficultyUtil.ID.Raid10Normal, 10, nil, false)
assert(not select(7, GetInstanceInfo()), "static instance state is returned")
assert(not primary_raid_normal_checked(), "static legacy difficulty does not select primary normal")

A_Admin.SetInstanceInfo("Legacy Raid", "raid", DifficultyUtil.ID.Raid10Normal, 10, nil, true)
assert(select(7, GetInstanceInfo()), "dynamic instance state is returned")
assert(primary_raid_normal_checked(), "dynamic legacy difficulty selects primary normal")
"##;

#[test]
fn unit_popup_difficulty_tracks_supplied_dynamic_instance_state() {
    with_blizzard_addon_closure(&["Blizzard_UnitPopup"], &[], |env, loaded| {
        assert!(loaded.iter().any(|addon| addon == "Blizzard_UnitPopup"));
        env.exec(DIFFICULTY_CASES)
            .expect("original UnitPopup difficulty consumer should use supplied instance state");
    });
}
