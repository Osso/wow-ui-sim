#![cfg(feature = "client-mists")]

use std::process::Command;

#[path = "common/wow_sim_binary.rs"]
mod wow_sim_binary;

#[test]
fn mists_lazy_loaded_panels_use_profile_blizzard_ui_sources() {
    let output = Command::new(wow_sim_binary::path())
        .env("WOW_SIM_NO_SAVED_VARS", "1")
        .env("WOW_SIM_NO_ADDONS", "1")
        .arg("--exec-lua")
        .arg(
            r#"
            local addons = {
                "Blizzard_AchievementUI",
                "Blizzard_Collections",
                "Blizzard_EncounterJournal",
            }
            for _, name in ipairs(addons) do
                local ok, reason = C_AddOns.LoadAddOn(name)
                if not ok then
                    error(name .. ": " .. tostring(reason))
                end
            end
            "#,
        )
        .arg("dump-tree")
        .output()
        .expect("wow-sim dump-tree should run");

    assert!(
        output.status.success(),
        "wow-sim dump-tree failed with status {:?}\nstderr:\n{}",
        output.status.code(),
        String::from_utf8_lossy(&output.stderr)
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    let forbidden = [
        "Lua error",
        "Blizzard_Collections/Classic",
        "Blizzard_Collections/Shared/Blizzard_Wardrobe_Sets.lua",
        "Blizzard_AchievementUI/Cata/Blizzard_AchievementUI.lua:532",
        "AchievementFrame_UpdateTrackedAchievements",
        "AchievementShield_OnLoad",
        "AchievementFrameStats_OnLoad",
        "WardrobeCollectionFrameMixin",
        "TransformCameraSpaceToModelSpace",
        "SetPortraitToAsset",
    ];
    let matches: Vec<_> = forbidden
        .iter()
        .filter(|needle| stderr.contains(**needle))
        .copied()
        .collect();

    assert!(
        matches.is_empty(),
        "Mists lazy-loaded panels used incomplete or wrong Blizzard UI sources: {matches:?}\nstderr:\n{stderr}"
    );
}
