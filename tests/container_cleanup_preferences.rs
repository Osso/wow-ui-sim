//! API-level coverage for the bag cleanup preferences and container search
//! filter (ContainerDocumentation.lua:97/:328/:364/:405/:440/:450/:483/:493/:513,
//! Retail and Forever). The original ContainerFrame.lua cleanup menu reaches
//! the backpack setters only through local menu callbacks.
#![cfg(any(feature = "client-retail", feature = "client-wowforever"))]

use crate::common;
use wow_ui_sim::lua_api::WowLuaEnv;

const PREFERENCE_PAIRS: &str = r#"
    local preferencePairs = {
        { C_Container.GetSortBagsRightToLeft, C_Container.SetSortBagsRightToLeft },
        { C_Container.GetInsertItemsLeftToRight, C_Container.SetInsertItemsLeftToRight },
        { C_Container.GetBankAutosortDisabled, C_Container.SetBankAutosortDisabled },
    }
    for index, pair in ipairs(preferencePairs) do
        local get, set = pair[1], pair[2]
        assert(get() == false, index)
        set(true)
        assert(get() == true, index)
        for other, otherPair in ipairs(preferencePairs) do
            if other ~= index then assert(otherPair[1]() == false, other) end
        end
        set(false)
        assert(get() == false, index)
    end
"#;

const BACKPACK_FLAGS: &str = r#"
    C_Container.SetBackpackAutosortDisabled(true)
    assert(C_Container.GetBackpackAutosortDisabled() == true)
    assert(C_Container.GetBackpackSellJunkDisabled() == false)
    C_Container.SetBackpackSellJunkDisabled(true)
    assert(C_Container.GetBackpackSellJunkDisabled() == true)
    C_Container.SetBackpackAutosortDisabled(false)
    assert(C_Container.GetBackpackAutosortDisabled() == false)
    assert(C_Container.GetBackpackSellJunkDisabled() == true)
    C_Container.SetBackpackSellJunkDisabled(false)
    assert(C_Container.GetBackpackSellJunkDisabled() == false)
"#;

const SEARCH_FILTER: &str = r#"
    A_Admin.ClearBags()
    A_Admin.AddBagItem(0, 1, 6948, 1)
    assert(C_Container.IsContainerFiltered(0) == false)
    C_Container.SetItemSearch("hearth")
    assert(C_Container.IsContainerFiltered(0) == false)
    C_Container.SetItemSearch("water")
    assert(C_Container.IsContainerFiltered(0) == true)
    A_Admin.AddBagItem(0, 2, 159, 1)
    assert(C_Container.IsContainerFiltered(0) == false)
    C_Container.SetItemSearch("")
    assert(C_Container.IsContainerFiltered(0) == false)
"#;

const BACKPACK_CLEANUP_MENU: &str = r#"
    local frame = ContainerFrameUtil_GetShownFrameForID(Enum.BagIndex.Backpack)
    assert(frame and frame:IsShown(), "backpack frame must be open")
    local function pick_checkbox(index)
        frame.PortraitButton:Click()
        local root = frame.PortraitButton:GetMenuDescription()
        local checkboxIndex, selected = 0, nil
        for _, entry in root:EnumerateElementDescriptions() do
            if entry:IsCheckbox() then
                checkboxIndex = checkboxIndex + 1
                if checkboxIndex == index then selected = entry end
            end
        end
        if not selected then return false end
        assert(selected:Pick(MenuInputContext.MouseButton, "LeftButton"))
        return true
    end
    local function select_preference(getter)
        for index = 1, 16 do
            if not pick_checkbox(index) then break end
            if getter() then return index end
        end
        error("backpack cleanup menu did not enable the preference")
    end
    assert(C_Container.GetBackpackAutosortDisabled() == false)
    local autosortIndex = select_preference(C_Container.GetBackpackAutosortDisabled)
    assert(C_Container.GetBackpackAutosortDisabled() == true)
    assert(pick_checkbox(autosortIndex))
    assert(C_Container.GetBackpackAutosortDisabled() == false)
    assert(C_Container.GetBackpackSellJunkDisabled() == false)
    select_preference(C_Container.GetBackpackSellJunkDisabled)
    assert(C_Container.GetBackpackSellJunkDisabled() == true)
"#;

#[test]
fn bag_slot_filter_overlay_tracks_the_container_search() {
    common::with_timeout(120, || {
        let env = WowLuaEnv::new().expect("Lua environment");
        common::load_game_startup_inventory(&env);
        env.sync_addon_names_to_lua();
        env.apply_post_load_workarounds();
        wow_ui_sim::startup::fire_startup_events_for_screen(
            &env,
            wow_ui_sim::screen::ScreenKind::Game,
        );
        env.exec(
            r#"
            A_Admin.ClearBags()
            A_Admin.AddBagItem(0, 1, 6948, 1)
            OpenAllBags()
            local backpack = MainMenuBarBackpackButton
            for _, case in ipairs({ { "water", true }, { "hearth", false } }) do
                C_Container.SetItemSearch(case[1])
                A_Admin.FireEvent("INVENTORY_SEARCH_UPDATE")
                local filtered = C_Container.IsContainerFiltered(0)
                assert(filtered == case[2], case[1])
                assert(backpack.ItemContextOverlay:IsShown() == filtered, case[1])
            end
            C_Container.SetItemSearch("")
            A_Admin.FireEvent("INVENTORY_SEARCH_UPDATE")
            "#,
        )
        .expect("bag button overlay follows each search result");
    });
}

#[test]
fn backpack_cleanup_menu_updates_container_preferences() {
    common::with_timeout(120, || {
        let env = WowLuaEnv::new().expect("Lua environment");
        common::load_game_startup_inventory(&env);
        env.sync_addon_names_to_lua();
        env.apply_post_load_workarounds();
        wow_ui_sim::startup::fire_startup_events_for_screen(
            &env,
            wow_ui_sim::screen::ScreenKind::Game,
        );
        env.exec(
            r#"
            SetCVar("combinedBags", "0")
            ToggleBag(Enum.BagIndex.Backpack)
            "#,
        )
        .expect("open backpack through ContainerFrame");
        env.exec(BACKPACK_CLEANUP_MENU)
            .expect("menu checkbox picks update the supplied preferences");
    });
}

#[test]
fn container_cleanup_preferences_and_search_filter_follow_inputs() {
    let env = WowLuaEnv::new().expect("Lua environment");
    for (label, chunk) in [
        ("preference pairs", PREFERENCE_PAIRS),
        ("backpack flags", BACKPACK_FLAGS),
        ("search filter", SEARCH_FILTER),
    ] {
        env.exec(chunk).unwrap_or_else(|error| panic!("{label}: {error}"));
    }
}
