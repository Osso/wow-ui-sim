//! API-level coverage for the bag cleanup preferences and container search
//! filter (ContainerDocumentation.lua:97/:328/:364/:405/:440/:450/:483/:493/:513,
//! Retail and Forever). The original ContainerFrame.lua cleanup menu reaches
//! the backpack setters only through local menu callbacks.
#![cfg(any(feature = "client-retail", feature = "client-wowforever"))]

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
