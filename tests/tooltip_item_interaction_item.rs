//! R88n.1 through the selected, unmodified ItemInteraction UI and TooltipDataHandler.
#![cfg(all(feature = "client-retail", feature = "optional-tests"))]

use crate::common::blizzard_addon_harness::blizzard_ui_dir;

#[path = "tooltip_full_env_helpers.rs"]
mod tooltip_full_env_helpers;
use wow_ui_sim::c_api::ItemInteractionTooltipInput;
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::toc::TocFile;

fn supply_object(env: &WowLuaEnv, kind: Option<i32>) {
    env.state().borrow_mut().item_interaction_tooltip_input =
        kind.map(|kind| ItemInteractionTooltipInput {
            object_kind: Some(kind),
            ..Default::default()
        });
}

fn original_item_interaction() -> WowLuaEnv {
    let root = blizzard_ui_dir();
    let toc_path = root.join("Blizzard_ItemInteractionUI");
    let toc_path =
        wow_ui_sim::loader::find_toc_file(&toc_path).expect("selected ItemInteraction TOC");
    let toc = TocFile::from_file(&toc_path).expect("read selected ItemInteraction TOC");
    assert!(toc.is_load_on_demand());
    let env = tooltip_full_env_helpers::load_full_env_without_post_load();
    env.exec("assert(not C_AddOns.IsAddOnLoaded('Blizzard_ItemInteractionUI'))")
        .unwrap();
    wow_ui_sim::loader::load_addon(&env.loader_env(), &toc_path)
        .expect("load original ItemInteraction on-demand TOC");
    env.exec(
        r#"
        assert(C_AddOns.IsAddOnLoaded("Blizzard_ItemInteractionUI"), "ItemInteraction LoD")
        local function assertSource(method, file)
            local source = debug.getinfo(method, "S").source
            assert(source:find(file, 1, true), source)
        end
        assertSource(ItemInteractionFrame.SetInputItemSlotTooltip, "Blizzard_ItemInteractionUI.lua")
        assertSource(GameTooltip.SetItemInteractionItem, "TooltipDataHandler.lua")
        assertSource(GameTooltip.GetPrimaryTooltipData, "TooltipDataHandler.lua")
        ItemInteractionFrame.interactionType = Enum.UIItemInteractionType.CleanseCorruption
        "#,
    )
    .expect("runtime-selected original tooltip caller and accessor");
    env
}

fn trigger_original_slot(env: &WowLuaEnv) {
    env.exec(
        r#"
        GameTooltip:Hide()
        local frame = ItemInteractionFrame
        local location = ItemLocation:CreateFromBagAndSlot(0, 1)
        frame:SetInputItemSlotTooltip(assert(frame.ItemSlot), location)
        "#,
    )
    .expect("original interaction slot tooltip");
}

// SUPPLEMENTARY original-consumer boundary test; not one of the four payload selectors.
#[test]
fn original_item_interaction_slot_distinguishes_absent_failed_and_empty_lines() {
    let env = original_item_interaction();
    trigger_original_slot(&env);
    env.exec("assert(GameTooltip:GetPrimaryTooltipData() == nil, 'absent object')")
        .unwrap();

    // Run misses before showing the empty-lines payload: a physically hidden
    // tooltip can retain its last handler data without firing OnHide again.
    for kind in [2, 4] {
        supply_object(&env, Some(kind));
        trigger_original_slot(&env);
        env.exec("assert(GameTooltip:GetPrimaryTooltipData() == nil, 'no producer payload')")
            .unwrap();
    }

    supply_object(&env, Some(3));
    trigger_original_slot(&env);
    env.exec(
        r#"
        local info = GameTooltip:GetPrimaryTooltipInfo()
        local data = assert(GameTooltip:GetPrimaryTooltipData(), "selected object must publish")
        assert(info.getterName == "GetItemInteractionItem", "wrong original accessor")
        assert(type(data.lines) == "table" and next(data.lines) == nil, "empty lines expected")
        assert(next(data) == "lines" and next(data, "lines") == nil, "only lines expected")
        "#,
    )
    .unwrap();
}

// SUPPLEMENTARY direct API contract test; its assertions remain unchanged.
#[test]
fn item_interaction_item_api_distinguishes_zero_results_and_empty_lines() {
    let env = WowLuaEnv::new().unwrap();
    let check_zero =
        "assert(select('#', C_TooltipInfo.GetItemInteractionItem()) == 0, 'expected zero results')";
    env.exec(check_zero).unwrap();
    for kind in [2, 4] {
        supply_object(&env, Some(kind));
        env.exec(check_zero).unwrap();
    }
    for kind in [1, 3] {
        supply_object(&env, Some(kind));
        env.exec(
            r#"
            local getter = C_TooltipInfo.GetItemInteractionItem
            assert(select('#', getter()) == 1, "expected one result")
            local data = getter()
            assert(type(data) == "table" and type(data.lines) == "table")
            assert(next(data.lines) == nil, "expected an empty lines table")
            assert(next(data) == "lines" and next(data, "lines") == nil, "only lines expected")
            "#,
        )
        .unwrap();
    }
}
