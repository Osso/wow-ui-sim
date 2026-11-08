#![cfg(feature = "client-mists")]

use std::path::PathBuf;
use std::process::Command;

#[path = "common/bounded_process.rs"]
mod bounded_process;

/// Cargo sets `CARGO_BIN_EXE_wow-sim` when it builds this test target, so the
/// path is fixed at compile time. Reading the variable at run time falls back
/// to `<manifest>/target/debug/wow-sim`, which does not exist when the test
/// binary is executed from a shared target directory.
fn wow_sim_binary() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_wow-sim"))
}

#[test]
fn mists_collections_tabs_render_without_wardrobe() {
    let output = bounded_process::output(
        90,
        Command::new(wow_sim_binary()).args([
            "--no-addons",
            "--no-saved-vars",
            "--exec-lua",
            r#"
            ToggleCollectionsJournal()

            local requiredFrames = {
                { "CollectionsJournal", CollectionsJournal },
                { "MountJournal", MountJournal },
                { "PetJournal", PetJournal },
                { "ToyBox", ToyBox },
                { "HeirloomsJournal", HeirloomsJournal },
            }
            for _, entry in ipairs(requiredFrames) do
                if entry[2] == nil then
                    error(entry[1] .. " missing")
                end
            end

            if WardrobeCollectionFrame ~= nil or CollectionsJournalTab5 ~= nil then
                error("Mists Collections should not expose Wardrobe")
            end
            if CollectionsJournal.numTabs ~= 4 then
                error("Mists Collections tab count mismatch")
            end

            local panelChecks = {
                { 1, MountJournal },
                { 2, PetJournal },
                { 3, ToyBox },
                { 4, HeirloomsJournal },
            }
            for _, check in ipairs(panelChecks) do
                CollectionsJournal_SetTab(CollectionsJournal, check[1])
                CollectionsJournal_UpdateSelectedTab(CollectionsJournal)
                if check[2]:IsShown() ~= true then
                    error("Collections tab " .. check[1] .. " did not show its panel")
                end
            end

            if C_MountJournal.GetNumDisplayedMounts() < 1 then
                error("mount collection data missing")
            end
            local totalPets, ownedPets = C_PetJournal.GetNumPets()
            if totalPets < 1 or ownedPets < 1 then
                error("pet collection data missing")
            end
            if C_ToyBox.GetNumTotalDisplayedToys() < 1 then
                error("toy collection data missing")
            end
            if C_Heirloom.GetNumHeirlooms() < 1 then
                error("heirloom collection data missing")
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
    assert_no_lua_errors(&stdout, &stderr);
}

#[test]
fn mists_collections_rows_drive_mount_toy_and_heirloom_actions() {
    let output = bounded_process::output(
        90,
        Command::new(wow_sim_binary())
        .args([
            "--no-addons",
            "--no-saved-vars",
            "--exec-lua",
            r#"
            ToggleCollectionsJournal()
            if not CollectionsJournal or not CollectionsJournal:IsShown() then
                error("CollectionsJournal did not open")
            end

            CollectionsJournal_SetTab(CollectionsJournal, 1)
            CollectionsJournal_UpdateSelectedTab(CollectionsJournal)
            MountJournal_UpdateMountList()
            local mountID = C_MountJournal.GetDisplayedMountID(1)
            if not mountID or mountID == 0 then
                error("no displayed mount available")
            end

            local mountButton = { index = 1, spellID = select(2, C_MountJournal.GetDisplayedMountInfo(1)) }
            MountListItem_OnClick(mountButton, "LeftButton")
            if MountJournal.selectedMountID ~= mountID then
                error("mount row click did not select displayed mount")
            end

            MountJournalMountButton_OnClick(MountJournal.MountButton)
            local _, _, _, active = C_MountJournal.GetMountInfoByID(mountID)
            if not active then
                error("mount action did not summon selected mount")
            end
            MountJournalMountButton_OnClick(MountJournal.MountButton)
            local _, _, _, activeAfterDismiss = C_MountJournal.GetMountInfoByID(mountID)
            if activeAfterDismiss then
                error("mount action did not dismiss active mount")
            end

            C_MountJournal.SetIsFavorite(1, true)
            local isFavorite = C_MountJournal.GetIsFavorite(1)
            if not isFavorite then
                error("mount favorite action did not persist")
            end

            CollectionsJournal_SetTab(CollectionsJournal, 3)
            CollectionsJournal_UpdateSelectedTab(CollectionsJournal)
            local toyID = C_ToyBox.GetToyFromIndex(1)
            if not toyID or toyID <= 0 then
                error("no displayed toy available")
            end

            ToySpellButton_OnClick({ itemID = toyID }, "LeftButton")

            C_ToyBox.SetIsFavorite(toyID, true)
            if not C_ToyBox.GetIsFavorite(toyID) then
                error("toy favorite action did not persist")
            end

            CollectionsJournal_SetTab(CollectionsJournal, 4)
            CollectionsJournal_UpdateSelectedTab(CollectionsJournal)
            local heirloomID = C_Heirloom.GetHeirloomItemIDFromDisplayedIndex(1)
            if not heirloomID or heirloomID == 0 then
                error("no displayed heirloom available")
            end

            local beforeCount = GetItemCount(heirloomID)
            HeirloomsJournalSpellButton_OnClick({ itemID = heirloomID }, "LeftButton")
            if GetItemCount(heirloomID) <= beforeCount then
                error("heirloom action did not create an heirloom copy")
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
    assert_no_lua_errors(&stdout, &stderr);
}

#[test]
fn mists_collections_pet_row_selects_and_shows_tooltip() {
    let output = bounded_process::output(
        90,
        Command::new(wow_sim_binary()).args([
            "--no-addons",
            "--no-saved-vars",
            "--exec-lua",
            r#"
            ToggleCollectionsJournal()
            CollectionsJournal_SetTab(CollectionsJournal, 2)
            CollectionsJournal_UpdateSelectedTab(CollectionsJournal)
            PetJournal_UpdatePetList()

            local row = PetJournal.ScrollBox:FindFrameByPredicate(function(frame)
                return frame.index == 2
            end)
            if not row or not row:IsShown() or not row.petID then
                error("second pet row is not visible")
            end
            if type(row:GetScript("OnClick")) ~= "function" then
                error("pet row has no OnClick handler")
            end

            row:GetScript("OnClick")(row, "LeftButton")
            if PetJournalPetCard.petID ~= row.petID or PetJournalPetCard.petIndex ~= row.index then
                error("pet row click did not select its pet card")
            end

            local dragButton = row.dragButton
            if not dragButton or type(dragButton:GetScript("OnEnter")) ~= "function" then
                error("pet row drag button has no tooltip handler")
            end
            dragButton:GetScript("OnEnter")(dragButton)
            if not GameTooltip:IsShown() or GameTooltip:NumLines() == 0 then
                error("pet row tooltip did not populate")
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
    assert_no_lua_errors(&stdout, &stderr);
}

/// The `lua-errors` command writes the authoritative JSON array on stdout and
/// always writes a summary banner to stderr, whose `Lua errors: N unique` line
/// contains the `Lua error` substring. Judge the array and the exit status
/// instead; a real exec-lua failure also prints `[exec-lua] error`.
fn assert_no_lua_errors(stdout: &str, stderr: &str) {
    assert!(
        stdout.trim() == "[]" && !stderr.contains("[exec-lua] error"),
        "Collections panel flow emitted Lua errors\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );
}
