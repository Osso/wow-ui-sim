#![cfg(all(feature = "client-wowforever", feature = "optional-tests"))]

use crate::common::blizzard_addon_harness::{
    blizzard_ui_dir, load_blizzard_addon_closure_into_env, new_blizzard_addon_env,
};
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::lua_api::state::MerchantCostRecord;

const PVP_TALENT_USAGE: &str = "Usage: local data = C_TooltipInfo.GetPvpTalent(talentID [, isInspect, groupIndex, talentIndex])";
const APPEARANCE_USAGE: &str =
    "Usage: local data = C_TooltipInfo.GetItemByItemModifiedAppearanceID(itemModifiedAppearanceID)";
const MERCHANT_COST_USAGE: &str =
    "Usage: local data = C_TooltipInfo.GetMerchantCostItem(slot, costIndex)";
const OUTFIT_USAGE: &str = "Usage: local data = C_TooltipInfo.GetOutfit(outfitID)";

fn assert_usage_error(env: &WowLuaEnv, call: &str, usage: &str) {
    env.exec(&format!(
        "local ok, err = pcall(function() {call} end); \
         assert(not ok, 'invalid arguments must fail'); \
         assert(string.find(tostring(err), {usage:?}, 1, true), 'Usage text must be present')"
    ))
    .unwrap();
}

#[test]
fn forever_pvp_talent_uses_supplied_spell_tooltips_and_misses_empty_records() {
    let env = WowLuaEnv::new().unwrap();
    env.state()
        .borrow_mut()
        .forever_pvp_talent_spell_ids
        .extend([(91_001, 116), (91_002, 133)]);
    env.exec(
        r#"
        local getter = C_TooltipInfo.GetPvpTalent
        assert(type(getter) == 'function', 'GetPvpTalent must be registered')
        for _, case in ipairs({{91001, 116}, {91002, 133}}) do
            local data = getter(case[1])
            local expected = C_TooltipInfo.GetSpellByID(case[2])
            assert(data and data.type == 1 and data.id == case[2],
                'each supplied talent must publish its mapped spell identity')
            assert(#data.lines == #expected.lines,
                'supplied talent uses the shared spell tooltip lines')
            for index, line in ipairs(expected.lines) do
                assert(data.lines[index].leftText == line.leftText,
                    'supplied talent preserves shared spell line text')
            end
        end
        assert(select('#', getter('2147483646.9', 0, '1.8', '2.9')) == 0,
            'a missing talent record must return zero values')
        "#,
    )
    .unwrap();
    assert_usage_error(&env, "C_TooltipInfo.GetPvpTalent()", PVP_TALENT_USAGE);
}

#[test]
fn forever_appearance_uses_supplied_item_tooltips_and_misses_empty_records() {
    let env = WowLuaEnv::new().unwrap();
    env.state()
        .borrow_mut()
        .forever_appearance_item_ids
        .extend([(90_001, 6948), (90_002, 159)]);
    env.exec(
        r#"
        local getter = C_TooltipInfo.GetItemByItemModifiedAppearanceID
        assert(type(getter) == 'function',
            'GetItemByItemModifiedAppearanceID must be registered')
        for _, case in ipairs({{90001, 6948}, {90002, 159}}) do
            local data = getter(case[1])
            local expected = C_TooltipInfo.GetItemByID(case[2])
            assert(data and data.type == 0 and data.id == case[2],
                'each supplied appearance must publish its mapped item identity')
            assert(#data.lines == #expected.lines and #data.lines > 0,
                'supplied appearance uses the shared item tooltip lines')
            for index, line in ipairs(expected.lines) do
                assert(data.lines[index].leftText == line.leftText,
                    'supplied appearance preserves shared item line text')
            end
        end
        assert(select('#', getter('2147483646.9')) == 0,
            'a missing ItemModifiedAppearance record must return zero values')
        "#,
    )
    .unwrap();
    assert_usage_error(
        &env,
        "C_TooltipInfo.GetItemByItemModifiedAppearanceID()",
        APPEARANCE_USAGE,
    );
}

#[test]
fn forever_merchant_cost_getter_returns_no_values_without_an_offer() {
    let env = WowLuaEnv::new().unwrap();
    {
        let mut state = env.state().borrow_mut();
        state.merchant_items = vec![6948, 159];
        state
            .merchant_offer_costs
            .insert(1, MerchantCostRecord::default());
    }
    env.exec(
        r#"
        local getter = C_TooltipInfo.GetMerchantCostItem
        assert(type(getter) == 'function', 'GetMerchantCostItem must be registered')
        assert(select('#', getter('1.8', '1.8')) == 0,
            'an offer without an extended-cost record must return zero values')
        assert(select('#', getter(2, 1)) == 0,
            'an extended-cost record with no selected cost must return zero values')
        assert(select('#', getter(3, 1)) == 0,
            'a missing merchant offer must return zero values')
        assert(select('#', getter(0, 1)) == 0,
            'zero selects no offer or cost')
        "#,
    )
    .unwrap();
    assert_usage_error(
        &env,
        "C_TooltipInfo.GetMerchantCostItem(nil, 1)",
        MERCHANT_COST_USAGE,
    );
}

#[test]
fn forever_outfit_getter_handles_zero_and_missing_saved_names() {
    let env = WowLuaEnv::new().unwrap();
    env.state()
        .borrow_mut()
        .transmog_outfit_catalog
        .outfits_info_supplied = true;
    env.exec(
        r#"
        local getter = C_TooltipInfo.GetOutfit
        assert(type(getter) == 'function', 'GetOutfit must be registered')
        assert(select('#', getter(0)) == 0, 'outfit ID zero must return zero values')
        local count = select('#', getter('42.8'))
        local data = getter('42.8')
        assert(count == 1 and type(data) == 'table',
            'a nonzero ID without a saved name must return one TooltipData table')
        assert(data.lines and data.lines[1] and data.lines[1].leftText == '',
            'a missing saved name must build an empty-name tooltip')
        "#,
    )
    .unwrap();
    assert_usage_error(&env, "C_TooltipInfo.GetOutfit(nil)", OUTFIT_USAGE);
}

fn load_spellbook_outfit_consumer() -> WowLuaEnv {
    let ui = blizzard_ui_dir();
    let env = new_blizzard_addon_env(&ui);
    crate::common::supply_empty_forever_spellbook_outfits(&env);
    let tooltip_and_panel_addons = load_blizzard_addon_closure_into_env(
        &env,
        &ui,
        &["Blizzard_GameTooltip", "Blizzard_UIParentPanelManager"],
        &[],
    );
    assert!(
        tooltip_and_panel_addons
            .iter()
            .any(|addon| addon == "Blizzard_GameTooltip")
    );
    assert!(
        tooltip_and_panel_addons
            .iter()
            .any(|addon| addon == "Blizzard_UIParentPanelManager")
    );
    env.exec("assert(type(RegisterUIPanel) == 'function')")
        .expect("panel manager must load before the selected SpellBook consumer");

    let spellbook_addons =
        load_blizzard_addon_closure_into_env(&env, &ui, &["Blizzard_PlayerSpells"], &[]);
    assert!(
        spellbook_addons
            .iter()
            .any(|addon| addon == "Blizzard_PlayerSpells")
    );
    env
}

fn assert_selected_spellbook_outfit_source(env: &WowLuaEnv) {
    env.exec(
        r#"
        assert(type(SpellBookOutfitItemMixin) == 'table'
            and type(SpellBookOutfitItemMixin.OnIconEnter) == 'function',
            'the selected SpellBook outfit consumer must load')
        local source = debug.getinfo(SpellBookOutfitItemMixin.OnIconEnter, 'S')
        local path = source.source:gsub('\\', '/')
        local suffix = 'SpellBook/Blizzard_SpellBookItem.lua'
        assert(source.what == 'Lua' and path:sub(-#suffix) == suffix
            and source.linedefined == 877,
            'the consumer must come from the selected Blizzard Lua source')
        "#,
    )
    .unwrap();
}

fn invoke_spellbook_outfit_consumer(env: &WowLuaEnv) {
    env.exec(
        r#"
        local button = {
            Button = CreateFrame('Button', nil, UIParent),
            GetOutfitID = function() return 42 end,
            GamepadAdjustTooltipAnchor = function() end,
        }
        SpellBookOutfitItemMixin.OnIconEnter(button)
        "#,
    )
    .unwrap();
}

fn assert_spellbook_outfit_consumer_result(env: &WowLuaEnv) {
    env.exec(
        r#"
        local info = GameTooltip:GetPrimaryTooltipInfo()
        local data = GameTooltip:GetPrimaryTooltipData()
        assert(info and info.getterName == 'GetOutfit',
            'the original consumer must dispatch GetOutfit')
        assert(data and data.lines[1].leftText == '',
            'the missing saved name must reach the original tooltip consumer')
        "#,
    )
    .unwrap();
}

#[test]
fn forever_spellbook_outfit_consumer_uses_the_selected_tooltip_getter() {
    let env = load_spellbook_outfit_consumer();
    assert_selected_spellbook_outfit_source(&env);
    invoke_spellbook_outfit_consumer(&env);
    assert_spellbook_outfit_consumer_result(&env);
}
