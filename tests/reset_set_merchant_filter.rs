//! Retail consumer coverage through the selected game-startup addon set.
//! MerchantFrame.lua:290 calls the modeled `CanAffordMerchantItem` global.
#![cfg(feature = "client-retail")]

use crate::common::blizzard_addon_harness::{blizzard_ui_dir, new_blizzard_addon_env};
use wow_ui_sim::loader::{
    discover_all_blizzard_addons, discover_blizzard_startup_addons_for_screen, load_startup_addon,
    register_load_with_addons,
};
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::lua_api::state::{BagItem, MerchantCostRecord};
use wow_ui_sim::screen::ScreenKind;

fn retail_game_startup_env() -> WowLuaEnv {
    let ui = blizzard_ui_dir();
    let env = new_blizzard_addon_env(&ui);
    register_load_with_addons(&env.loader_env(), &discover_all_blizzard_addons(&ui));

    let startup = discover_blizzard_startup_addons_for_screen(&ui, ScreenKind::Game);
    for addon in startup {
        let already_loaded = env
            .state()
            .borrow()
            .addons
            .iter()
            .any(|loaded| loaded.folder_name == addon.name && loaded.loaded);
        if !already_loaded {
            load_startup_addon(&env.loader_env(), &addon.toc_path, addon.kind, None)
                .unwrap_or_else(|error| panic!("load startup {}: {error}", addon.name));
        }
        if addon.name == "Blizzard_EnvironmentCleanup" {
            env.restore_post_cleanup_globals();
        }
    }
    env.sync_string_metatable_to_global_string();
    env
}

#[test]
fn merchant_frame_update_uses_affordability_for_populated_offers() {
    let env = retail_game_startup_env();
    {
        let mut state = env.state().borrow_mut();
        state.merchant_items = vec![6948, 2840, 6948, 6948];
        state.merchant_frame_open = true;
        state.player.present = true;
        state.player.money = 100;
        state.bag_items.insert(
            (0, 1),
            BagItem {
                owned_identity: None,
                item_id: 2840,
                stack_count: 2,
                hyperlink: None,
                durability: None,
                applied_transmog: None,
            },
        );
        state.currency_info.remove(&2245);
        state.merchant_offer_costs.insert(
            0,
            MerchantCostRecord {
                price: 100,
                item_costs: [2840, 0, 0, 0, 0],
                item_cost_counts: [2, 0, 0, 0, 0],
                ..MerchantCostRecord::default()
            },
        );
        state.merchant_offer_costs.insert(
            1,
            MerchantCostRecord {
                price: 101,
                ..MerchantCostRecord::default()
            },
        );
        state.merchant_offer_costs.insert(
            2,
            MerchantCostRecord {
                currency_costs: [2245, 0, 0, 0, 0],
                currency_cost_amounts: [1, 0, 0, 0, 0],
                ..MerchantCostRecord::default()
            },
        );
    }

    env.exec(
        r##"
        MerchantFrame.selectedTab = 1
        MerchantFrame.lastTab = 1
        local updated, updateError = pcall(MerchantFrame_Update)
        assert(updated or not tostring(updateError):find("MerchantFrame.lua:290", 1, true),
            tostring(updateError))
        assert(CanAffordMerchantItem(1) == true, "affordable price and item costs must pass")
        assert(CanAffordMerchantItem(1.9) == true, "fractional indices truncate toward zero")
        assert(CanAffordMerchantItem("1") == true, "numeric strings supply the slot index")
        assert(CanAffordMerchantItem(2) == false, "price above current money must fail")
        assert(CanAffordMerchantItem(3) == false, "missing currency must fail")
        assert(select("#", CanAffordMerchantItem(4)) == 0,
            "offer without a cost record must return no values")
        assert(select("#", CanAffordMerchantItem(99)) == 0,
            "out-of-range merchant index must return no values")
        "##,
    )
    .unwrap();
}

#[test]
fn merchant_frame_on_show_resets_the_filter_without_return_values() {
    let env = retail_game_startup_env();
    {
        let mut state = env.state().borrow_mut();
        state.merchant_items = vec![6948, 2840];
        state.merchant_frame_open = true;
    }

    env.exec(
        r##"
        assert(type(MerchantFrame_OnShow) == "function", "Retail merchant consumer missing")
        assert(type(CanAffordMerchantItem) == "function",
            "merchant affordability global must be registered")
        SetMerchantFilter(3)
        local ok, err = pcall(MerchantFrame_OnShow, MerchantFrame)
        if not ok then
            assert(not tostring(err):find("MerchantFrame.lua:290", 1, true), tostring(err))
        end
        assert(GetMerchantFilter() == 2, "merchant filter must reset on show")
        assert(select("#", ResetSetMerchantFilter()) == 0, "reset must return no values")
        assert(GetMerchantNumItems() == 2, "simulator merchant count remains unfiltered")
        "##,
    )
    .expect("original Retail MerchantFrame_OnShow should reach the reset");
}

/// Original consumer invocation with explicit simulator inputs, not
/// evidence for inferred faction field names or rating comparison direction.
#[cfg(feature = "optional-tests")]
#[test]
fn merchant_r88i5_original_consumer_calls_requirement_failure_and_skip() {
    use wow_ui_sim::lua_api::state::MerchantItemRequirement;
    let env = retail_game_startup_env();
    {
        let mut sim = env.state().borrow_mut();
        sim.merchant_items = vec![6948];
        sim.merchant_frame_open = true;
        sim.player.present = true;
        sim.player.money = 100;
        sim.merchant_offer_costs
            .insert(0, MerchantCostRecord::default());
        sim.merchant_offer_categories.insert(0, 1);
        sim.merchant_item_requirements
            .insert(6948, MerchantItemRequirement::default());
    }
    env.exec(
        r#"
        assert(debug.getinfo(MerchantFrame_UpdateMerchantInfo).source:find(
            'Blizzard_UIPanels_Game/Mainline/MerchantFrame.lua', 1, true))
        MerchantFrame.selectedTab = 1
        MerchantFrame.lastTab = 1
        MerchantFrame.page = 1
        local nativeAffordability = CanAffordMerchantItem
        local function pack(...) return { n = select('#', ...), ... } end
        CanAffordMerchantItem = function(...)
            local result = pack(nativeAffordability(...))
            MerchantRequirementObserved = { result = result, caller = debug.getinfo(2).source }
            return unpack(result, 1, result.n)
        end
        function CheckMerchantRequirementResult(expectFailure)
            MerchantRequirementObserved = nil
            local ok, err = pcall(MerchantFrame_Update)
            assert(not ok and tostring(err):find('MerchantFrame.lua:329:', 1, true), tostring(err))
            local observed = MerchantRequirementObserved
            assert(observed and observed.caller:find(
                'Blizzard_UIPanels_Game/Mainline/MerchantFrame.lua', 1, true))
            assert(observed.result.n == 1)
            assert(observed.result[1] == not expectFailure)
        end
        CheckMerchantRequirementResult(false)
    "#,
    )
    .unwrap();
    // INFERRED adapter: an empty faction table represents the missing owner.
    // VERIFIED publication for a missing native owner is exactly one false.
    env.exec("__wow_reputation_state.factions = {}").unwrap();
    env.state()
        .borrow_mut()
        .merchant_item_requirements
        .get_mut(&6948)
        .unwrap()
        .requirement_id = 42;
    env.exec("assert(select('#', CanAffordMerchantItem(1)) == 1); assert(CanAffordMerchantItem(1) == false); CheckMerchantRequirementResult(true)").unwrap();
    // VERIFIED: zero ID skips the owner/test; missing item record returns no values.
    env.state()
        .borrow_mut()
        .merchant_item_requirements
        .get_mut(&6948)
        .unwrap()
        .requirement_id = 0;
    env.exec("assert(CanAffordMerchantItem(1)); CheckMerchantRequirementResult(false)")
        .unwrap();
    env.state()
        .borrow_mut()
        .merchant_item_requirements
        .remove(&6948);
    env.exec("assert(select('#', CanAffordMerchantItem(1)) == 0)")
        .unwrap();
    // INFERRED adapter fixture: absent rating rejects; test claims only the
    // original Lua call to that supplied simulator outcome.
    {
        let mut sim = env.state().borrow_mut();
        sim.merchant_offer_categories.insert(0, 0);
        let cost = sim.merchant_offer_costs.get_mut(&0).unwrap();
        cost.requirement_value = 1;
        cost.requirement_mode = 0;
    }
    env.exec("CheckMerchantRequirementResult(true)").unwrap();
    // VERIFIED cost gating: signed modes outside0..8 skip the requirement.
    env.state()
        .borrow_mut()
        .merchant_offer_costs
        .get_mut(&0)
        .unwrap()
        .requirement_mode = -1;
    env.exec("assert(CanAffordMerchantItem(1)); CheckMerchantRequirementResult(false)")
        .unwrap();
}
