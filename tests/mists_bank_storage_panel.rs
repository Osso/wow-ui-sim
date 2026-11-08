#![cfg(feature = "client-mists")]

use std::path::PathBuf;
use std::process::Command;

use crate::common::blizzard_addon_harness::{
    blizzard_ui_dir, load_blizzard_addon_closure_into_env, new_blizzard_addon_env,
};

#[test]
fn original_guild_bank_tabs_follow_empty_supplied_empty_catalog() {
    let ui = blizzard_ui_dir();
    let env = new_blizzard_addon_env(&ui);
    env.state().borrow_mut().world.guild_name = None;
    for (name, toc) in wow_ui_sim::loader::discover_blizzard_addons_for_screen(
        &ui,
        wow_ui_sim::screen::ScreenKind::Game,
    ) {
        wow_ui_sim::loader::load_addon(&env.loader_env(), &toc)
            .unwrap_or_else(|error| panic!("load {name}: {error}"));
    }
    let loaded = load_blizzard_addon_closure_into_env(&env, &ui, &["Blizzard_GuildBankUI"], &[]);
    assert!(loaded.iter().any(|name| name == "Blizzard_GuildBankUI"));
    env.exec(r##"
        local source = debug.getinfo(GuildBankFrame.UpdateTabs, "S").source:gsub("\\", "/")
        assert(source:find("Blizzard_GuildBankUI/TBC/Blizzard_GuildBankUI.lua", 1, true), source)
        assert(not IsGuildLeader(), "exclude the buy-new-tab branch")
        GuildBankFrame.mode = "bank"
        GuildBankFrame:UpdateTabs()
        assert(not GuildBankFrame.BankTabs[1]:IsShown(), "empty catalog must hide tab 1")
        assert(GetNumGuildBankTabs() == 0)
        A_Admin.AddGuildBankTab("Supplied materials", "Interface\\Icons\\INV_Misc_Bag_10", true, false, 3, 2, true)
        GuildBankFrame:UpdateTabs()
        assert(GuildBankFrame.BankTabs[1]:IsShown(), "supplied catalog must show tab 1")
        assert(GuildBankFrame.BankTabs[1].Button.tooltip == "Supplied materials")
        assert(not GuildBankFrame.BankTabs[2]:IsShown(), "only supplied tabs exist")
        local name, icon, view, deposit, withdrawals, remaining, filtered = GetGuildBankTabInfo(1)
        assert(name == "Supplied materials" and icon == "Interface\\Icons\\INV_Misc_Bag_10")
        assert(view == true and deposit == false and withdrawals == 3 and remaining == 2 and filtered == true)
        A_Admin.ClearGuildBankTabs()
        GuildBankFrame:UpdateTabs()
        assert(not GuildBankFrame.BankTabs[1]:IsShown(), "cleared catalog must hide tab 1")
        assert(select("#", GetGuildBankTabInfo(1)) == 0)
    "##).expect("original Mists guild-bank tab setup must project the supplied catalog");
}

fn wow_sim_binary() -> PathBuf {
    std::env::var_os("CARGO_BIN_EXE_wow-sim")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("target")
                .join("debug")
                .join("wow-sim")
        })
}

#[test]
fn mists_bank_and_guild_bank_support_storage_flow() {
    let output = Command::new("timeout")
        .arg("90")
        .arg(wow_sim_binary())
        .args([
            "--no-addons",
            "--no-saved-vars",
            "--exec-lua",
            r#"
            A_Admin.ClearBags()
            A_Admin.AddBagItem(0, 1, 6948, 1)
            A_Admin.AddBagItem(-1, 1, 6948, 1)
            A_Admin.ClearGuildBank()
            A_Admin.AddGuildBankTab("General", [[Interface\Icons\INV_Misc_Bag_10]], true, true, -1, -1, false)
            A_Admin.AddGuildBankItem(1, 1, 6948, 1)

            FireEvent("BANKFRAME_OPENED")
            if not (BankFrame and BankFrame:IsShown()) then
                error("BankFrame did not open")
            end

            OpenAllBags()
            local hoveredBagItem = false
            for index = 1, 36 do
                local bagButton = _G["ContainerFrame1Item" .. index]
                if bagButton and bagButton:IsShown() then
                    ContainerFrameItemButton_OnEnter(bagButton)
                    if GameTooltip:IsShown() then
                        hoveredBagItem = true
                        break
                    end
                end
            end
            if not hoveredBagItem then
                error("bag item hover did not show GameTooltip")
            end

            local bankSlots, full = GetNumBankSlots()
            if bankSlots ~= 0 or full ~= false then
                error("bank slot state=" .. tostring(bankSlots) .. "/" .. tostring(full))
            end

            if C_Container.GetContainerNumSlots(BANK_CONTAINER) <= 0 then
                error("bank container has no slots")
            end

            PurchaseSlot()
            bankSlots, full = GetNumBankSlots()
            if bankSlots ~= 1 or full ~= false then
                error("bank slot state after purchase=" .. tostring(bankSlots) .. "/" .. tostring(full))
            end

            local bankInfo = C_Container.GetContainerItemInfo(BANK_CONTAINER, 1)
            if not bankInfo or bankInfo.itemID ~= 6948 then
                error("bank item missing")
            end

            local ok, reason = LoadAddOn("Blizzard_GuildBankUI")
            if ok == false then
                error("Blizzard_GuildBankUI failed to load: " .. tostring(reason))
            end

            if not C_GuildBank.IsGuildBankEnabled() then
                error("guild bank disabled")
            end

            FireEvent("GUILDBANKFRAME_OPENED")
            if not (GuildBankFrame and GuildBankFrame:IsShown()) then
                error("GuildBankFrame did not open")
            end

            if GetNumGuildBankTabs() < 1 then
                error("expected at least one guild bank tab")
            end

            local name, icon, isViewable = GetGuildBankTabInfo(1)
            if name ~= "General" or not isViewable then
                error("guild bank tab=" .. tostring(name) .. "/" .. tostring(isViewable))
            end

            local texture, itemCount, locked, isFiltered, quality = GetGuildBankItemInfo(1, 1)
            if itemCount ~= 1 or texture == nil or quality == nil then
                error("guild bank item=" .. tostring(texture) .. "/" .. tostring(itemCount) .. "/" .. tostring(quality))
            end

            if GetGuildBankMoney() <= 0 then
                error("guild bank money missing")
            end
            "#,
            "lua-errors",
        ])
        .output()
        .expect("failed to run wow-sim");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        output.status.success(),
        "wow-sim failed\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );
    assert_no_lua_errors(&stdout, &stderr);
}

fn assert_no_lua_errors(stdout: &str, stderr: &str) {
    assert!(
        !stdout.contains("Lua error:") && !stderr.contains("Lua error:"),
        "bank storage flow emitted Lua errors\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );
}
