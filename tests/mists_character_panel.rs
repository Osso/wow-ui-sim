#![cfg(feature = "client-mists")]

use std::process::Command;

#[path = "common/bounded_process.rs"]
mod bounded_process;

#[path = "common/wow_sim_binary.rs"]
mod wow_sim_binary;

#[test]
fn mists_character_panel_populates_gear_and_reputation() {
    let output = bounded_process::output(
        90,
        Command::new(wow_sim_binary::path()).args([
            "--no-addons",
            "--no-saved-vars",
            "--exec-lua",
            r#"
            ToggleCharacter("PaperDollFrame")
            if not CharacterFrame or not CharacterFrame:IsShown() then
                error("CharacterFrame did not open")
            end

            local gearSlots = {
                "CharacterHeadSlot",
                "CharacterChestSlot",
                "CharacterLegsSlot",
                "CharacterFeetSlot",
                "CharacterMainHandSlot",
            }
            for _, slotName in ipairs(gearSlots) do
                local slot = _G[slotName]
                local icon = _G[slotName .. "IconTexture"]
                if not slot or not slot:IsShown() then
                    error(slotName .. " is not shown")
                end
                if not icon or not icon:GetTexture() then
                    error(slotName .. " has no icon texture")
                end
            end

            ToggleCharacter("ReputationFrame")
            if not ReputationFrame or not ReputationFrame:IsShown() then
                error("ReputationFrame did not open")
            end
            if type(ReputationFrame_Update) == "function" then
                ReputationFrame_Update()
            end
            if GetNumFactions() <= 0 then
                error("no factions are exposed")
            end

            local populatedRows = 0
            for i = 1, 15 do
                local row = _G["ReputationBar" .. i]
                local name = _G["ReputationBar" .. i .. "FactionName"]
                local bar = _G["ReputationBar" .. i .. "ReputationBar"]
                if row and row:IsShown() and name and name:GetText() and bar then
                    populatedRows = populatedRows + 1
                end
            end
            if populatedRows == 0 then
                error("ReputationFrame has no populated rows")
            end
            "#,
            "dump-tree",
            "--filter-key",
            "CharacterFrame",
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
fn mists_character_bottom_tabs_size_their_text() {
    let output = bounded_process::output(
        90,
        Command::new(wow_sim_binary::path()).args([
            "--no-addons",
            "--no-saved-vars",
            "--exec-lua",
            r#"
            ToggleCharacter("ReputationFrame")
            local tabTextNames = {
                "CharacterFrameTab1Text",
                "CharacterFrameTab2Text",
                "CharacterFrameTab3Text",
                "CharacterFrameTab4Text",
            }
            for _, name in ipairs(tabTextNames) do
                local fontString = _G[name]
                if not fontString then
                    error(name .. " is missing")
                end
                if type(fontString:GetText()) ~= "string" or fontString:GetText() == "" then
                    error(name .. " has no text")
                end
                if fontString:GetStringWidth() <= 0 then
                    error(name .. " has no measurable string width")
                end
                if fontString:GetWidth() <= 0 then
                    error(name .. " has zero frame width")
                end
            end

            local visibleTabs = {
                CharacterFrameTab1,
                CharacterFrameTab3,
                CharacterFrameTab4,
            }
            local function assertVisibleTabChrome(tab)
                local name = tab:GetName()
                local normalVisible = _G[name .. "Left"]:IsVisible()
                    and _G[name .. "Middle"]:IsVisible()
                    and _G[name .. "Right"]:IsVisible()
                local activeVisible = _G[name .. "LeftDisabled"]:IsVisible()
                    and _G[name .. "MiddleDisabled"]:IsVisible()
                    and _G[name .. "RightDisabled"]:IsVisible()
                if not normalVisible and not activeVisible then
                    error(name .. " has no visible tab texture set")
                end
                for _, suffix in ipairs({
                    "Left",
                    "Middle",
                    "Right",
                    "LeftDisabled",
                    "MiddleDisabled",
                    "RightDisabled",
                }) do
                    local texture = _G[name .. suffix]
                    if not texture or type(texture:GetTexture()) ~= "string" then
                        error(name .. suffix .. " has no texture")
                    end
                end
            end

            for _, tab in ipairs(visibleTabs) do
                local name = tab:GetName()
                if not tab:IsShown() then
                    error(name .. " is unexpectedly hidden")
                end
                if type(tab:GetText()) ~= "string" or tab:GetText() == "" then
                    error(name .. " has no tab label")
                end
                assertVisibleTabChrome(tab)
            end

            if CharacterFrameTab2:IsShown() then
                error("pet tab is visible despite absent pet UI")
            end
            for _, suffix in ipairs({
                "Left",
                "Middle",
                "Right",
                "LeftDisabled",
                "MiddleDisabled",
                "RightDisabled",
                "Text",
            }) do
                local region = _G["CharacterFrameTab2" .. suffix]
                if region and region:IsVisible() then
                    error("hidden pet tab leaked visible " .. suffix)
                end
            end
            "#,
            "dump-tree",
            "--filter",
            "CharacterFrameTab",
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
fn mists_character_pet_tab_stays_visible_when_pet_ui_is_seeded() {
    let output = bounded_process::output(
        90,
        Command::new(wow_sim_binary::path()).args([
            "--no-addons",
            "--no-saved-vars",
            "--exec-lua",
            r#"
            A_Admin.SetPetActionSlot(
                1,
                "Claw",
                "Interface\\Icons\\Ability_Druid_Rake",
                16827
            )

            ToggleCharacter("PetPaperDollFrame")
            if not CharacterFrame or not CharacterFrame:IsShown() then
                error("CharacterFrame did not open")
            end
            if not CharacterFrameTab2 or not CharacterFrameTab2:IsShown() then
                error("pet tab is hidden despite seeded pet UI")
            end
            if not PetPaperDollFrame or not PetPaperDollFrame:IsShown() then
                error("PetPaperDollFrame did not open")
            end
            "#,
            "dump-tree",
            "--filter-key",
            "CharacterFrame",
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
    assert!(
        stdout.contains("PetPaperDollFrame") && stdout.contains("CharacterFrameTab2"),
        "character dump did not include the visible pet panel/tab\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );
}

#[test]
fn mists_character_pet_tab_hides_when_pet_ui_is_absent() {
    let output = bounded_process::output(
        90,
        Command::new(wow_sim_binary::path()).args([
            "--no-addons",
            "--no-saved-vars",
            "--exec-lua",
            r#"
            local hasPetUI = HasPetUI()
            if hasPetUI then
                error("default simulator state unexpectedly has pet UI")
            end

            ToggleCharacter("PaperDollFrame")
            if not CharacterFrame or not CharacterFrame:IsShown() then
                error("CharacterFrame did not open")
            end
            if CharacterFrameTab2 and CharacterFrameTab2:IsShown() then
                error("pet tab is visible despite absent pet UI")
            end
            "#,
            "dump-tree",
            "--filter-key",
            "CharacterFrame",
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
    assert!(
        !stdout.contains("CharacterFrameTab2Text [FontString] (14x10) visible"),
        "hidden pet tab leaked its text region as visible\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );
}

#[test]
fn mists_character_pet_tab_rehides_stale_tab_when_panel_opens() {
    let output = bounded_process::output(
        90,
        Command::new(wow_sim_binary::path()).args([
            "--no-addons",
            "--no-saved-vars",
            "--exec-lua",
            r#"
            FireEvent("PET_UI_UPDATE")
            CharacterFrameTab2:Show()

            ToggleCharacter("PaperDollFrame")
            if not CharacterFrame or not CharacterFrame:IsShown() then
                error("CharacterFrame did not open")
            end
            if CharacterFrameTab2:IsShown() then
                error("stale pet tab stayed visible despite absent pet UI")
            end
            "#,
            "dump-tree",
            "--filter-key",
            "CharacterFrame",
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
fn mists_character_subpanels_drive_titles_and_equipment_sets() {
    let output = bounded_process::output(
        90,
        Command::new(wow_sim_binary::path()).args([
            "--no-addons",
            "--no-saved-vars",
            "--exec-lua",
            r#"
            ToggleCharacter("PaperDollFrame")
            if not CharacterFrame or not CharacterFrame:IsShown() then
                error("CharacterFrame did not open")
            end

            PaperDollFrame_SetSidebar(PaperDollFrame, 2)
            local titlePane = PaperDollFrame.TitleManagerPane
            if not titlePane or not titlePane:IsShown() then
                error("TitleManagerPane did not open")
            end

            PaperDollTitlesPane_Update()
            if not titlePane.titles or #titlePane.titles < 2 then
                error("TitleManagerPane has no selectable title rows")
            end

            local selectedTitle = titlePane.titles[2]
            PlayerTitleButton_OnClick({ titleId = selectedTitle.id })
            PaperDollTitlesPane_Update()
            if GetCurrentTitle() ~= selectedTitle.id then
                error("title click did not update current title")
            end
            if titlePane.selected ~= selectedTitle.id then
                error("title pane did not retain the selected title")
            end

            SetCurrentTitle(-1)
            if not SetTitleByName(string.sub(selectedTitle.name, 1, 4)) then
                error("SetTitleByName did not find the selected title")
            end
            if GetCurrentTitle() ~= selectedTitle.id then
                error("SetTitleByName did not update current title")
            end

            local setName = "Codex Mists Set"
            C_EquipmentSet.CreateEquipmentSet(setName, "Interface\\Icons\\INV_Misc_QuestionMark")
            local setID = C_EquipmentSet.GetEquipmentSetID(setName)
            if not setID then
                error("equipment set was not created")
            end

            PaperDollFrame_SetSidebar(PaperDollFrame, 3)
            local equipmentPane = PaperDollFrame.EquipmentManagerPane
            if not equipmentPane or not equipmentPane:IsShown() then
                error("EquipmentManagerPane did not open")
            end

            PaperDollEquipmentManagerPane_Update(true)
            if not equipmentPane.equipmentSetIDs or #equipmentPane.equipmentSetIDs == 0 then
                error("EquipmentManagerPane has no equipment-set rows")
            end

            GearSetButton_OnClick({ setID = setID }, "LeftButton", false)
            if equipmentPane.selectedSetID ~= setID then
                error("equipment set row click did not select the set")
            end

            PaperDollEquipmentManagerPaneEquipSet_OnClick(equipmentPane.EquipSet)
            local _, _, _, isEquipped = C_EquipmentSet.GetEquipmentSetInfo(setID)
            if not isEquipped then
                error("equipment set equip button did not mark the set equipped")
            end

            C_EquipmentSet.IgnoreSlotForSave(1)
            if not C_EquipmentSet.IsSlotIgnoredForSave(1) then
                error("equipment manager did not persist ignored slot state")
            end
            "#,
            "dump-tree",
            "--filter-key",
            "CharacterFrame",
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
    assert!(
        stdout.contains("EquipmentManagerPane") && stdout.contains("Codex Mists Set"),
        "character dump did not include the selected equipment-manager row\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );
}

fn assert_no_lua_errors(stdout: &str, stderr: &str) {
    assert!(
        !stdout.contains("Lua error") && !stderr.contains("Lua error"),
        "character panel opened with Lua errors\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );
}
