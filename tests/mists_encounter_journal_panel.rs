#![cfg(feature = "client-mists")]

#[path = "common/lua_error_output.rs"]
mod lua_error_output;

use std::process::Command;

#[path = "common/bounded_process.rs"]
mod bounded_process;

#[path = "common/wow_sim_binary.rs"]
mod wow_sim_binary;

#[test]
fn mists_encounter_journal_opens_and_displays_instances() {
    let output = bounded_process::output(
        90,
        Command::new(wow_sim_binary::path()).args([
            "--no-addons",
            "--no-saved-vars",
            "--exec-lua",
            r#"
            if ToggleEncounterJournal() ~= true then
                error("ToggleEncounterJournal did not load the journal")
            end
            if EncounterJournal == nil or EncounterJournal:IsShown() ~= true then
                error("EncounterJournal did not open")
            end
            if EncounterJournal.instanceSelect == nil or EncounterJournal.encounter == nil then
                error("EncounterJournal core frames missing")
            end
            if EJ_GetNumTiers() < 5 then
                error("Mists Encounter Journal tiers missing")
            end

            EJ_SelectTier(5)
            EncounterJournal_ListInstances()
            local instanceID, name = EJ_GetInstanceByIndex(1, false)
            if type(instanceID) ~= "number" or type(name) ~= "string" then
                error("Mists dungeon instance list is empty")
            end

            EncounterJournal_DisplayInstance(instanceID)
            if EncounterJournal.encounter:IsShown() ~= true then
                error("Encounter Journal instance panel did not show")
            end
            if EncounterJournal.instanceID ~= instanceID then
                error("Encounter Journal did not select requested instance")
            end
            if EncounterJournal.encounter.info.instanceTitle:GetText() ~= name then
                error("Encounter Journal instance title mismatch")
            end
            if EncounterJournal.encounter.info.tab ~= 1 then
                error("Encounter Journal did not default to overview tab")
            end

            EncounterJournal_SetTab(2)
            if EncounterJournal.encounter.info.LootContainer:IsShown() ~= true then
                error("Encounter Journal loot tab did not show")
            end
            EncounterJournal_SetTab(1)
            local tabOneContentShown =
                EncounterJournal.encounter.info.overviewScroll:IsShown() or
                EncounterJournal.encounter.info.detailsScroll:IsShown()
            if EncounterJournal.encounter.info.tab ~= 1 or not tabOneContentShown then
                error("Encounter Journal overview/abilities tab did not restore")
            end
            "#,
            "lua-errors",
        ]),
    )
    .expect("failed to run wow-sim");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        output.status.success(),
        "wow-sim failed\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );
    lua_error_output::assert_no_lua_errors("Encounter Journal panel", &stdout, &stderr);
}
