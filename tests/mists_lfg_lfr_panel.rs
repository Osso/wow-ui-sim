#![cfg(feature = "client-mists")]

#[path = "common/lua_error_output.rs"]
mod lua_error_output;

use std::process::Command;

#[path = "common/bounded_process.rs"]
mod bounded_process;

#[path = "common/wow_sim_binary.rs"]
mod wow_sim_binary;

#[test]
fn mists_lfg_lfr_and_raid_finder_panels_render_with_seeded_choices() {
    let output = bounded_process::output(
        90,
        Command::new(wow_sim_binary::path())
        .args([
            "--no-addons",
            "--no-saved-vars",
            "--exec-lua",
            r#"
            if not PVEFrame then
                local ok, reason = LoadAddOn("Blizzard_GroupFinder")
                if ok == false then
                    error("Blizzard_GroupFinder failed to load: " .. tostring(reason))
                end
            end
            if not (PVEFrame and GroupFinderFrame and LFDQueueFrame and RaidFinderQueueFrame and RaidFinderFrame) then
                error("group finder frame set missing")
            end

            PVEFrame:Show()
            GroupFinderFrame:Show()
            GroupFinderFrame_Update()

            if GetNumRandomDungeons() < 1 or type(GetRandomDungeonBestChoice()) ~= "number" then
                error("random dungeon choice missing")
            end
            LFDParentFrame:Show()
            LFDQueueFrame:Show()
            LFDQueueFrame_SetType("random")
            LFDQueueFrame_Update()
            if LFDQueueFrameFindGroupButton:IsEnabled() ~= true then
                error("random dungeon find-group button disabled")
            end

            LFDQueueFrame_SetType("specific")
            LFDQueueFrame_Update()
            if not LFDDungeonList or #LFDDungeonList < 1 then
                error("specific dungeon list missing")
            end

            local bestRaid = GetBestRFChoice()
            if type(bestRaid) ~= "number" or GetNumRFDungeons() < 1 then
                error("raid finder choice missing")
            end
            local firstRaidID, firstRaidName = GetRFDungeonInfo(1)
            if firstRaidID ~= bestRaid or type(firstRaidName) ~= "string" or firstRaidName == "" then
                error("raid finder row invalid")
            end

            RaidFinderFrame:Show()
            RaidFinderFrame_UpdateAvailability()
            RaidFinderQueueFrame_SetRaid(bestRaid)
            RaidFinderQueueFrame_UpdateRoles()
            RaidFinderQueueFrameRewards_UpdateFrame()
            RaidFinderFrameFindRaidButton_Update()
            if RaidFinderFrame.NoRaidsCover:IsShown() then
                error("raid finder no-raids cover still shown")
            end
            if RaidFinderQueueFrame.raid ~= bestRaid then
                error("raid finder selected raid mismatch")
            end
            "#,
            "lua-errors",
        ])
        )
        .expect("failed to run wow-sim");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        output.status.success(),
        "wow-sim failed\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );
    lua_error_output::assert_no_lua_errors("LFG/LFR panel", &stdout, &stderr);
}
