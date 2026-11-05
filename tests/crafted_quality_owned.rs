//! Private positive-contract authorship; not executed RED/GREEN or native evidence.
#![cfg(any(feature = "client-retail", feature = "client-ptr", feature = "client-wowforever"))]

use wow_ui_sim::c_api::c_trade_skill_quality::CraftingQualityInfo;
use wow_ui_sim::lua_api::WowLuaEnv;

const ITEM_ID: u32 = 900001;
const OTHER_ITEM_ID: u32 = 900002;

fn supplied_record() -> CraftingQualityInfo {
    CraftingQualityInfo {
        quality: 3,
        icon: "crafted-icon".into(),
        icon_small: "crafted-small".into(),
        icon_inventory: "crafted-inventory".into(),
        icon_mixed: "crafted-mixed".into(),
        icon_appear: "crafted-appear".into(),
        icon_dissolve: "crafted-dissolve".into(),
        bar_fill: "crafted-fill".into(),
        bar_background: "crafted-background".into(),
        bar_background_cap: "crafted-cap".into(),
        bar_highlight: "crafted-highlight".into(),
        icon_chat: "crafted-chat".into(),
        icon_quest_objective: "crafted-quest".into(),
    }
}

fn new_contract_env() -> WowLuaEnv {
    let env = WowLuaEnv::new().expect("fresh crafted-item contract environment");
    env.exec(include_str!("crafted_quality_owned/assertions.lua"))
        .expect("install test-owned expected values and snapshot assertions");
    env
}

#[test]
fn crafted_quality_supplied_record_projects_exact_thirteen_fields_and_one_result() {
    let env = new_contract_env();
    env.state()
        .borrow_mut()
        .crafted_item_quality
        .insert(ITEM_ID, supplied_record());
    env.exec(
        r#"
        local value = C_TradeSkillUI.GetItemCraftedQualityInfo(900001)
        assertCraftedSnapshot(value, craftedExpected)
        assert(select('#', C_TradeSkillUI.GetItemCraftedQualityInfo(900001)) == 1,
            'present crafted result must have one-value arity')
        "#,
    )
    .expect("public supplied-record projection, not recipe recorder mechanics");
}

#[test]
fn crafted_quality_existing_item_selector_forms_share_the_supplied_record() {
    let env = new_contract_env();
    env.state()
        .borrow_mut()
        .crafted_item_quality
        .insert(ITEM_ID, supplied_record());
    env.exec(
        r#"
        -- Bounded existing parser choices; no name lookup or new numeric grammar.
        for _, selector in ipairs({900001, '900001', 'item:900001:0:0',
            '|cff0070dd|Hitem:900001:0:0|h[Supplied crafted input]|h|r'}) do
            local value = C_TradeSkillUI.GetItemCraftedQualityInfo(selector)
            assertCraftedSnapshot(value, craftedExpected)
            assert(select('#', C_TradeSkillUI.GetItemCraftedQualityInfo(selector)) == 1)
        end
        "#,
    )
    .expect("numeric, decimal string, item string and hyperlink resolve the same host item");
}

#[test]
fn crafted_quality_detached_snapshots_survive_host_replacement_and_removal() {
    let env = new_contract_env();
    env.state()
        .borrow_mut()
        .crafted_item_quality
        .insert(ITEM_ID, supplied_record());
    env.state()
        .borrow_mut()
        .crafted_item_quality
        .insert(OTHER_ITEM_ID, supplied_record());
    env.exec(
        r#"
        craftedRetained = C_TradeSkillUI.GetItemCraftedQualityInfo(900001)
        local changed = C_TradeSkillUI.GetItemCraftedQualityInfo(900001)
        assertCraftedSnapshot(craftedRetained, craftedExpected)
        assert(not rawequal(changed, craftedRetained), 'each query returns a detached table')
        for key in pairs(changed) do changed[key] = 'client mutation' end
        changed.extra = true
        assertCraftedSnapshot(craftedRetained, craftedExpected)
        assertCraftedSnapshot(C_TradeSkillUI.GetItemCraftedQualityInfo(900001), craftedExpected)
        "#,
    )
    .expect("Lua mutation does not change host record or retained result");
    let mut replacement = supplied_record();
    replacement.quality = 4;
    replacement.icon_inventory = "crafted-replaced-inventory".into();
    env.state()
        .borrow_mut()
        .crafted_item_quality
        .insert(ITEM_ID, replacement);
    env.exec(
        r#"
        local expected = {}
        for key, value in pairs(craftedExpected) do expected[key] = value end
        expected.quality = 4
        expected.iconInventory = 'crafted-replaced-inventory'
        craftedReplacementExpected = expected
        craftedReplacement = C_TradeSkillUI.GetItemCraftedQualityInfo(900001)
        assertCraftedSnapshot(craftedReplacement, expected)
        assertCraftedSnapshot(craftedRetained, craftedExpected)
        assertCraftedSnapshot(C_TradeSkillUI.GetItemCraftedQualityInfo(900002), craftedExpected)
        "#,
    )
    .expect("replacement changes only future snapshots for the mapped item");
    env.state()
        .borrow_mut()
        .crafted_item_quality
        .remove(&ITEM_ID);
    env.exec(
        r#"
        assert(C_TradeSkillUI.GetItemCraftedQualityInfo(900001) == nil)
        assert(select('#', C_TradeSkillUI.GetItemCraftedQualityInfo(900001)) == 1)
        assertCraftedSnapshot(craftedRetained, craftedExpected)
        assertCraftedSnapshot(craftedReplacement, craftedReplacementExpected)
        assertCraftedSnapshot(C_TradeSkillUI.GetItemCraftedQualityInfo(900002), craftedExpected)
        "#,
    )
    .expect("removal leaves prior results and the other item unchanged");
}

#[test]
fn crafted_quality_simulator_unknown_is_one_nil_and_inputs_are_environment_owned() {
    let first = new_contract_env();
    let second = new_contract_env();
    first
        .state()
        .borrow_mut()
        .crafted_item_quality
        .insert(ITEM_ID, supplied_record());
    // Recipe input is deliberately independent, not an alias or seed for crafted lookup.
    second
        .state()
        .borrow_mut()
        .recipe_quality_inputs
        .item_quality
        .insert((164, ITEM_ID), supplied_record());
    second
        .exec(
            r#"
        -- Explicit simulator unknown-item policy; no native catalog classification claim.
        for _, item in ipairs({900001, 900002, 900003}) do
            assert(C_TradeSkillUI.GetItemCraftedQualityInfo(item) == nil)
            assert(select('#', C_TradeSkillUI.GetItemCraftedQualityInfo(item)) == 1)
        end
        "#,
        )
        .expect("empty independent environment stays unknown despite recipe record");
    first
        .exec(
            r#"
        assertCraftedSnapshot(C_TradeSkillUI.GetItemCraftedQualityInfo(900001), craftedExpected)
        assert(C_TradeSkillUI.GetItemCraftedQualityInfo(900003) == nil)
        assert(select('#', C_TradeSkillUI.GetItemCraftedQualityInfo(900003)) == 1)
        "#,
        )
        .expect("positive host input distinguishes this case from an auto-nil-only test");
    let mut second_record = supplied_record();
    second_record.quality = 5;
    second
        .state()
        .borrow_mut()
        .crafted_item_quality
        .insert(ITEM_ID, second_record);
    second
        .exec(
            r#"
        local expected = {}
        for key, value in pairs(craftedExpected) do expected[key] = value end
        expected.quality = 5
        assertCraftedSnapshot(C_TradeSkillUI.GetItemCraftedQualityInfo(900001), expected)
        "#,
        )
        .expect("same item can hold independent records in two environments");
    first.exec(
        "assertCraftedSnapshot(C_TradeSkillUI.GetItemCraftedQualityInfo(900001), craftedExpected)",
    )
    .expect("second environment does not overwrite first environment");
}

#[test]
fn crafted_quality_real_vm_secret_selectors_guard_lookup_and_recover_wrappers() {
    let env = new_contract_env();
    env.state()
        .borrow_mut()
        .crafted_item_quality
        .insert(ITEM_ID, supplied_record());
    env.exec(
        r#"
        debug.settaintmode(true)
        assert(debug.getstacktaint() == nil, 'untainted setup required')
        craftedSecretSelectors = {}
        local payloads = {900001, '900001', 'item:900001:0:0',
            '|Hitem:900001:0:0|h[Supplied crafted input]|h',
            900003, '900003', 'item:900003:0', '|Hitem:900003:0|h[Unknown input]|h'}
        for index, payload in ipairs(payloads) do
            local selector = secretwrap(payload)
            craftedSecretSelectors[index] = selector
            assert(issecretvalue(selector), 'genuine VM wrapper required')
            local value = C_TradeSkillUI.GetItemCraftedQualityInfo(selector)
            if index <= 4 then
                assertCraftedSnapshot(value, craftedExpected)
            else
                assert(value == nil, 'untainted absent secret selector must remain unknown')
            end
            assert(select('#', C_TradeSkillUI.GetItemCraftedQualityInfo(selector)) == 1)
            local function addon()
                assert(debug.getstacktaint() == 'CraftedQualityPositiveTest')
                local ok, message = pcall(C_TradeSkillUI.GetItemCraftedQualityInfo, selector)
                assert(not ok, 'tainted present/absent secret denied before lookup')
                assert(type(message) == 'string')
                assert(message:find('requires an untainted caller', 1, true), message)
                assert(issecretvalue(selector), 'denial must preserve wrapper')
                assert(not pcall(secretunwrap, selector), 'tainted access still denied')
                assert(debug.getstacktaint() == 'CraftedQualityPositiveTest')
            end
            debug.setobjecttaint(addon, 'CraftedQualityPositiveTest')
            addon()
            assert(debug.getstacktaint() == nil, 'outer taint restored after denial')
            assert(issecretvalue(selector), 'original wrapper retained')
            assert(secretunwrap(selector) == payload, 'original payload retained')
            local recovered = C_TradeSkillUI.GetItemCraftedQualityInfo(selector)
            if index <= 4 then assertCraftedSnapshot(recovered, craftedExpected)
            else assert(recovered == nil) end
        end
        -- No duplicate ordinary nil/wrong-kind test: only authentication-before-type here.
        local wrongKind = secretwrap(true)
        local function deniedBeforeType()
            local ok, message = pcall(C_TradeSkillUI.GetItemCraftedQualityInfo, wrongKind)
            assert(not ok)
            assert(type(message) == 'string')
            assert(message:find('requires an untainted caller', 1, true), message)
            assert(issecretvalue(wrongKind))
            assert(debug.getstacktaint() == 'CraftedQualityPositiveTest')
        end
        debug.setobjecttaint(deniedBeforeType, 'CraftedQualityPositiveTest')
        deniedBeforeType()
        assert(debug.getstacktaint() == nil)
        assert(issecretvalue(wrongKind) and secretunwrap(wrongKind) == true)
        "#,
    )
    .expect("authentic present/absent selectors and type-order denial preserve security state");
    env.state()
        .borrow_mut()
        .crafted_item_quality
        .remove(&ITEM_ID);
    env.exec(
        r#"
        for _, selector in ipairs(craftedSecretSelectors) do
            assert(issecretvalue(selector))
            assert(C_TradeSkillUI.GetItemCraftedQualityInfo(selector) == nil)
            assert(select('#', C_TradeSkillUI.GetItemCraftedQualityInfo(selector)) == 1)
        end
        assert(debug.getstacktaint() == nil)
        "#,
    )
    .expect("retained authentic wrappers recover untainted absent lookup after host removal");
}
