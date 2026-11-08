#![cfg(feature = "client-mists")]

#[path = "common/lua_error_output.rs"]
mod lua_error_output;

use std::process::Command;

#[path = "common/bounded_process.rs"]
mod bounded_process;

#[path = "common/wow_sim_binary.rs"]
mod wow_sim_binary;

#[test]
fn mists_legacy_service_panels_support_state_changing_interactions() {
    let output = bounded_process::output(
        90,
        Command::new(wow_sim_binary::path()).args([
            "--no-addons",
            "--no-saved-vars",
            "--exec-lua",
            LEGACY_SERVICE_INTERACTIONS_LUA,
            "lua-errors",
        ]),
    )
    .expect("failed to run wow-sim");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        output.status.success(),
        "legacy service panel flow failed\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );
    lua_error_output::assert_no_lua_errors("legacy service panel flow", &stdout, &stderr);
}

const LEGACY_SERVICE_INTERACTIONS_LUA: &str = r#"
    local function fail(message)
        error(message, 0)
    end

    local function assertShown(frame, label)
        if not frame or not frame:IsShown() then
            fail(label .. " did not show")
        end
    end

    ArchaeologyFrame_LoadUI()
    ShowUIPanel(ArchaeologyFrame)
    assertShown(ArchaeologyFrame, "ArchaeologyFrame")
    local archaeologyHelpTab = {
        GetParent = function() return ArchaeologyFrame end,
        GetID = function() return ARCHAEOLOGY_HELP_TAB end,
    }
    if ArchaeologyFrame.selectedTab ~= ARCHAEOLOGY_HELP_TAB
        or not ArchaeologyFrame.helpPage:IsShown() then
        fail("ArchaeologyFrame did not start on the help page")
    end
    ArchaeologyFrame_OnTabClick(archaeologyHelpTab)
    if ArchaeologyFrame.selectedTab ~= ARCHAEOLOGY_SUMMARY_TAB
        or ArchaeologyFrame.currentFrame ~= ArchaeologyFrame.summaryPage then
        fail("Archaeology help-tab toggle did not restore selected summary state")
    end

    CraftFrame_LoadUI()
    ShowUIPanel(CraftFrame)
    assertShown(CraftFrame, "CraftFrame")
    CraftFrame_SetSelection(1)
    if GetCraftSelectionIndex() ~= 1 or CraftName:GetText() ~= "Rough Sharpening Stone" then
        fail("CraftFrame selection did not populate the selected craft detail")
    end

    TradeSkillFrame_LoadUI()
    ShowUIPanel(TradeSkillFrame)
    assertShown(TradeSkillFrame, "TradeSkillFrame")
    TradeSkillSkillButton_OnClick(TradeSkillSkill1, "LeftButton")
    if GetTradeSkillSelectionIndex() ~= 1 or TradeSkillSkillName:GetText() ~= "Rough Sharpening Stone" then
        fail("TradeSkillFrame row click did not select and render the first recipe")
    end
    TradeSkillFrameIncrement_OnClick()
    if TradeSkillInputBox:GetNumber() ~= 2 then
        fail("TradeSkillFrame increment button did not update the repeat count")
    end
    TradeSkillFrameDecrement_OnClick()
    if TradeSkillInputBox:GetNumber() ~= 1 then
        fail("TradeSkillFrame decrement button did not update the repeat count")
    end

    ItemSocketingFrame_LoadUI()
    C_ItemSocketInfo._state.numSockets = 1
    C_ItemSocketInfo._state.socketTypes = { [1] = "Red" }
    C_ItemSocketInfo._state.clickProposals = {
        [1] = {
            name = "Parity Ruby",
            icon = 111,
            gemMatchesSocket = true,
            link = "item:111",
        },
    }
    ShowUIPanel(ItemSocketingFrame)
    assertShown(ItemSocketingFrame, "ItemSocketingFrame")
    ItemSocketingFrame.SocketingContainer:Update()
    ItemSocketingFrame.SocketingContainer.SocketFrames[1]:Click()
    local newGemName = C_ItemSocketInfo.GetNewSocketInfo(1)
    if C_ItemSocketInfo._state.selectedSocketIndex ~= 1 or newGemName ~= "Parity Ruby" then
        fail("ItemSocketingFrame socket click did not propose a new socket gem")
    end
    ItemSocketingFrame.SocketingContainer.ApplySocketsButton:Click()
    local existingGemName = C_ItemSocketInfo.GetExistingSocketInfo(1)
    if existingGemName ~= "Parity Ruby" or C_ItemSocketInfo._state.acceptCount ~= 1 then
        fail("ItemSocketingFrame apply click did not accept proposed sockets")
    end

    Reforging_LoadUI()
    ReforgingFrame_Show()
    assertShown(ReforgingFrame, "ReforgingFrame")
    ReforgingFrame_AddItemClick(ReforgingFrame)
    ReforgingFrameRestoreButton:Click()
    if not ReforgingFrame:IsShown() then
        fail("ReforgingFrame action controls unexpectedly hid the panel")
    end

    ItemUpgrade_LoadUI()
    ItemUpgradeFrame_Show()
    assertShown(ItemUpgradeFrame, "ItemUpgradeFrame")
    C_ItemUpgrade.SetItemUpgradeFromLocation({ bagID = 0, slotIndex = 1 })
    if not __item_upgrade_state or not __item_upgrade_state.location then
        fail("ItemUpgradeFrame setup did not retain the selected item location")
    end
    C_ItemUpgrade.ClearItemUpgrade()
    if __item_upgrade_state.location ~= nil then
        fail("ItemUpgradeFrame clear path did not clear the selected item location")
    end

    LoadAddOn("Blizzard_BarbershopUI")
    ShowUIPanel(BarberShopFrame)
    assertShown(BarberShopFrame, "BarberShopFrame")
    BarberShop_SetViewingAlteredForm(true)
    if not C_BarberShop.IsViewingAlteredForm() then
        fail("BarberShop altered-form control did not update backing state")
    end
    BarberShop_SetViewingAlteredForm(false)
    if C_BarberShop.IsViewingAlteredForm() then
        fail("BarberShop altered-form reset did not update backing state")
    end

    BlackMarket_LoadUI()
    BlackMarketFrame_Show()
    assertShown(BlackMarketFrame, "BlackMarketFrame")
    A_Admin.SetMoney(5000000)
    local row = {
        marketID = 77,
        minNextBid = 12345,
        itemLink = "item:19019",
        Selection = {
            selected = false,
            SetShown = function(self, shown)
                self.selected = shown
            end,
        },
    }
    for key, value in pairs(BlackMarketItemMixin) do
        row[key] = value
    end
    BlackMarketFrame.ScrollBox.ForEachFrame = function(_, callback)
        callback(row, {})
    end
    row:OnClick("LeftButton")
    if BlackMarketFrame.selectedMarketID ~= 77 then
        fail("BlackMarket row click did not select the auction row")
    end
    if row.Selection.selected ~= true then
        fail("BlackMarket row click did not update the visible row selection")
    end
    if not BlackMarketFrame:IsShown() then
        fail("BlackMarket selected row update unexpectedly hid the frame")
    end
"#;
