#![cfg(feature = "client-mists")]

#[path = "common/lua_error_output.rs"]
mod lua_error_output;

use std::process::Command;

#[path = "common/bounded_process.rs"]
mod bounded_process;

#[test]
fn mists_world_map_opens_with_zone_art_and_quest_pins() {
    let output = bounded_process::output(
        90,
        Command::new(env!("CARGO_BIN_EXE_wow-sim")).args([
            "--no-addons",
            "--no-saved-vars",
            "--exec-lua",
            r#"
            ToggleWorldMap()
            if not (WorldMapFrame and WorldMapFrame:IsShown()) then
                error("WorldMapFrame did not open")
            end

            if WorldMapFrame.RefreshAllDataProviders then
                WorldMapFrame:RefreshAllDataProviders()
            end

            local mapID = WorldMapFrame.GetMapID and WorldMapFrame:GetMapID()
            if not mapID or mapID == 0 then
                error("WorldMapFrame has no map ID")
            end

            local canvas = WorldMapFrame.ScrollContainer
                and WorldMapFrame.ScrollContainer.Child
            if not canvas or canvas:GetNumChildren() == 0 then
                error("WorldMapFrame has no scroll-canvas children")
            end

            local questID, questLogIndex = QuestPOIGetQuestIDByVisibleIndex(2)
            if not questID or questID == 0 or not questLogIndex or questLogIndex == 0 then
                error("quest POI visible-index lookup returned no quest")
            end

            local questMapID = GetQuestUiMapID(questID)
            if not questMapID or questMapID == 0 then
                error("quest POI has no UiMapID")
            end

            if InActiveBattlefield() then
                error("default simulator state should not be an active battlefield")
            end

            WorldMapFrame:SetMapID(84)
            WorldMapFrame:NavigateToParentMap()
            if WorldMapFrame:GetMapID() ~= 13 then
                error("WorldMapFrame did not navigate from Stormwind to Eastern Kingdoms")
            end

            WorldMapFrame:SetMapID(84)
            WorldMapFrame:ResetZoom()
            local beforeZoom = WorldMapFrame:GetCanvasZoomPercent()
            WorldMapFrame:ZoomIn()
            local afterZoom = WorldMapFrame:GetCanvasZoomPercent()
            if afterZoom <= beforeZoom then
                error("WorldMapFrame zoom-in did not increase canvas zoom")
            end

            WorldMapFrame:SetMapID(questMapID)
            WorldMapFrame:RefreshAllDataProviders()
            local questPin
            for pin in WorldMapFrame:EnumeratePinsByTemplate("QuestPinTemplate") do
                questPin = pin
                break
            end
            if not questPin or not questPin.questID then
                error("WorldMapFrame did not render an interactive quest pin")
            end

            questPin:OnClick("LeftButton")
            if QuestMapFrame_GetDetailQuestID() ~= questPin.questID then
                error("quest pin click did not select quest details")
            end
            local selectedLogIndex = GetQuestLogIndexByID(questPin.questID) or 0
            if selectedLogIndex > 0 and GetQuestLogSelection() ~= selectedLogIndex then
                error("quest pin click did not update quest-log selection")
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
    lua_error_output::assert_no_lua_errors("world map opened", &stdout, &stderr);
}

