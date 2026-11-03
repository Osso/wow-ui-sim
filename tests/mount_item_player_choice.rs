//! Whole cached consumer baseline; not mount-branch or native dressability proof.
#![cfg(any(feature = "client-retail", feature = "client-ptr"))]

use std::path::Path;

use wow_ui_sim::loader::{
    BlizzardAddonOverride, discover_blizzard_addon_closure_for_screen_with_overrides, load_addon,
};
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::screen::ScreenKind;

const ROOTS: &[&str] = &[
    "Blizzard_PlayerChoice",
    "Blizzard_ItemButton",
    "Blizzard_SharedXML",
];

const ORIGINAL_ITEM_CONSUMER: &str = r#"
    local observations = {}
    local function observe(...)
        local fields = {}
        for index = 1, select('#', ...) do
            local value = select(index, ...)
            fields[index] = tostring(value)
        end
        observations[#observations + 1] = table.concat(fields, '\t')
    end
    for _, addon in ipairs({
        'Blizzard_PlayerChoice', 'Blizzard_ItemButton', 'Blizzard_SharedXML'
    }) do
        assert(C_AddOns.IsAddOnLoaded(addon), 'whole addon not loaded: ' .. addon)
    end
    assert(DoesTemplateExist('PlayerChoiceBaseOptionItemRewardTemplate'),
        'original reward XML template missing')
    local reward = CreateFrame('Button', nil, UIParent,
        'PlayerChoiceBaseOptionItemRewardTemplate')
    assert(reward:GetObjectType() == 'Button', 'original XML reward Button')
    assert(reward.itemButton, 'original XML intrinsic ItemButton child missing')

    local originalRewardMethod = PlayerChoiceBaseOptionItemRewardMixin.IsDressupReward
    local originalSetItem = ItemButtonMixin.SetItem
    assert(reward.IsDressupReward == originalRewardMethod,
        'reward must retain original vendor method identity')
    assert(reward.itemButton.SetItem == originalSetItem,
        'intrinsic ItemButton must retain original vendor SetItem identity')
    assert(reward.itemButton.SetItemInternal == ItemButtonMixin.SetItemInternal,
        'original SetItemInternal identity')
    assert(reward.itemButton.GetItemInfo == ItemButtonMixin.GetItemInfo,
        'original vendor GetItemInfo identity, not a replacement getter')

    local function provenance(fn, suffix)
        local info = debug.getinfo(fn, 'S')
        local source = info.source:gsub('\\', '/')
        observe('[mount-item consumer provenance]', info.what, source,
            info.linedefined, info.lastlinedefined)
        assert(info.what == 'Lua', 'expected cached vendor Lua method')
        assert(source:sub(-#suffix) == suffix, 'wrong vendor source: ' .. source)
    end
    local rewardSource = 'Blizzard_PlayerChoice/Blizzard_PlayerChoiceOptionBase.lua'
    local itemSource = 'Blizzard_ItemButton/Shared/ItemButtonTemplate.lua'
    provenance(reward.IsDressupReward, rewardSource)
    provenance(reward.itemButton.SetItem, itemSource)
    provenance(reward.itemButton.SetItemInternal, itemSource)
    provenance(reward.itemButton.GetItemInfo, itemSource)

    local name, link = C_Item.GetItemInfo(6948)
    assert(name == 'Hearthstone' and type(link) == 'string' and link ~= '',
        'generated real item metadata/link required')
    observe('[mount-item consumer input]', name, link)
    observe('[mount-item consumer stage]', 'original ItemButton:SetItem(6948)')
    reward.itemButton:SetItem(6948)
    assert(reward.itemButton.itemLink == link, 'real SetItem must store generated item link')
    assert(reward.itemButton:GetItemID() == 6948, 'real ItemButton item identity')
    assert(reward.itemButton.SetItem == originalSetItem,
        'SetItem identity unchanged after real setup')
    assert(reward.IsDressupReward == originalRewardMethod,
        'reward method identity unchanged before actual invocation')

    -- Observations only: no missing-API guard replaces the actual vendor call.
    observe('[mount-item consumer API observations]',
        type(C_Item.IsDressableItemByID),
        type(C_MountJournal and C_MountJournal.GetMountFromItem),
        type(C_PetJournal and C_PetJournal.GetPetInfoByItemID))
    observe('[mount-item consumer stage]', 'original reward:IsDressupReward({itemId=6948})')
    local value = reward:IsDressupReward({itemId=6948})
    assert(type(value) == 'boolean', 'actual original consumer must return a boolean')
    observe('[mount-item consumer result]', value)
    local predicate = C_Item.IsDressableItemByID
    local predicateValue = predicate(reward.itemButton.itemLink)
    observe('[mount-item consumer predicate observation]',
        type(rawget(C_Item, 'IsDressableItemByID')), type(predicateValue), predicateValue)
    local predicateSource = debug.getinfo(predicate, 'S')
    observe('[mount-item consumer predicate source]', predicateSource.what,
        predicateSource.source, predicateSource.linedefined, predicateSource.lastlinedefined)
    return value, table.concat(observations, '\n')
"#;

#[cfg(feature = "client-retail")]
#[test]
fn pet_info_by_item_uses_native_tuple_and_argument_contract() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        local api = C_PetJournal.GetPetInfoByItemID
        for _, value in ipairs({false, {}, 'not-a-number'}) do
            local ok, err = pcall(api, value)
            assert(not ok and err:find('Usage: Script_GetPetInfoByItemID(itemID)', 1, true))
        end
        local ok, err = pcall(api)
        assert(not ok and err:find('Usage: Script_GetPetInfoByItemID(itemID)', 1, true))
        assert(select('#', api(0)) == 0 and select('#', api(6948)) == 0)
        assert(select('#', api(4401)) == 13)
        local name, icon, petType, creature, source, description, wild, battle,
              tradeable, unique, obtainable, display, species = api('4401', 'ignored')
        assert(name == 'Mechanical Squirrel' and icon == 656559 and petType == 10)
        assert(creature == 2671 and source == '|cFFFFD200Profession: |rEngineering')
        assert(description == "A mechanical squirrel's logic center tells it to collect and store both nuts and bolts for the winter.")
        assert(not wild and battle and tradeable and not unique and obtainable)
        assert(display == 7937 and species == 39)
        local _, _, _, _, _, _, _, _, _, _, _, worgDisplay, worgSpecies = api(12264)
        assert(worgDisplay == 9563 and worgSpecies == 89)
        "#,
    )
    .unwrap();
}

#[test]
fn player_choice_recognizes_pet_item_from_real_catalog() {
    let ui = crate::common::blizzard_addon_harness::blizzard_ui_dir();
    let env = crate::common::blizzard_addon_harness::new_blizzard_addon_env(&ui);
    load_player_choice_closure(&env, &ui);
    env.exec(
        r#"
        local reward = CreateFrame('Button', nil, UIParent,
            'PlayerChoiceBaseOptionItemRewardTemplate')
        assert(reward.IsDressupReward == PlayerChoiceBaseOptionItemRewardMixin.IsDressupReward)
        reward.itemButton.itemLink = 'item:4401'
        assert(not C_Item.IsDressableItemByID(reward.itemButton.itemLink))
        assert(not C_MountJournal.GetMountFromItem(4401))
        assert(reward:IsDressupReward({itemId=4401}), 'Mechanical Squirrel Box must be a pet reward')
        reward.itemButton.itemLink = 'item:6948'
        assert(not reward:IsDressupReward({itemId=6948}), 'Hearthstone is not a pet reward')
        "#,
    )
    .expect("original PlayerChoice consumer distinguishes pet and non-pet items");
    assert_no_recorded_errors(&env, "pet item PlayerChoice consumer");
}

#[test]
fn mount_item_player_choice_real_item_consumer_returns_boolean() {
    let ui = crate::common::blizzard_addon_harness::blizzard_ui_dir();
    eprintln!("[mount-item consumer cache] {}", ui.display());
    let item = wow_ui_sim::items::get_item(6948).expect("generated real item 6948");
    assert_eq!(item.name, "Hearthstone");
    let env = crate::common::blizzard_addon_harness::new_blizzard_addon_env(&ui);
    load_player_choice_closure(&env, &ui);
    invoke_original_item_consumer(&env);
}

#[test]
fn mount_item_player_choice_original_consumer_tracks_live_supplied_relation() {
    let ui = crate::common::blizzard_addon_harness::blizzard_ui_dir();
    let env = crate::common::blizzard_addon_harness::new_blizzard_addon_env(&ui);
    load_player_choice_closure(&env, &ui);
    let baseline = invoke_original_item_consumer(&env);
    env.state()
        .borrow_mut()
        .mount_item_relations
        .insert(6948, 401);
    let mapped = invoke_original_item_consumer(&env);
    assert!(mapped, "supplied relation should produce a dress-up reward");
    assert_ne!(
        baseline, mapped,
        "actual consumer must depend on the supplied mount relation"
    );
    env.state().borrow_mut().mount_item_relations.remove(&6948);
    assert_eq!(invoke_original_item_consumer(&env), baseline);
}

fn load_player_choice_closure(env: &WowLuaEnv, ui: &Path) {
    let overrides = [BlizzardAddonOverride {
        addon: "Blizzard_PlayerChoice",
        extra_roots: &["Blizzard_ItemButton", "Blizzard_SharedXML"],
    }];
    let addons = discover_blizzard_addon_closure_for_screen_with_overrides(
        ui,
        ScreenKind::Game,
        ROOTS,
        &overrides,
    );
    eprintln!("[mount-item consumer discovered closure] {addons:?}");
    for root in ROOTS {
        assert!(
            addons.iter().any(|(name, _)| name == root),
            "whole closure root missing: {root}"
        );
    }
    for (name, toc) in addons {
        load_consumer_addon(env, &name, &toc);
    }
    assert_no_recorded_errors(env, "whole closure load");
}

fn load_consumer_addon(env: &WowLuaEnv, name: &str, toc: &Path) {
    eprintln!("[mount-item consumer load] {name}: {}", toc.display());
    let result = load_addon(&env.loader_env(), toc);
    print_recorded_errors(env, name);
    let loaded = result
        .unwrap_or_else(|error| panic!("whole closure load {name} at {}: {error}", toc.display()));
    eprintln!(
        "[mount-item consumer warnings] {name}: {:?}",
        loaded.warnings
    );
    eprintln!(
        "[mount-item consumer nil observations] {name}: {:?}",
        loaded.nil_symbol_observations
    );
    eprintln!(
        "[mount-item consumer missing requirements] {name}: {:?}",
        loaded.missing_requirements
    );
    assert!(
        loaded.warnings.is_empty(),
        "actual whole closure warnings for {name}: {:?}",
        loaded.warnings
    );
    assert_no_recorded_errors(env, name);
}

fn print_recorded_errors(env: &WowLuaEnv, boundary: &str) {
    let state = env.state().borrow();
    eprintln!(
        "[mount-item consumer Lua errors] {boundary}: {:?}",
        state.lua_errors
    );
    eprintln!(
        "[mount-item consumer Lua error records] {boundary}: {:?}",
        state.lua_error_records
    );
    eprintln!(
        "[mount-item consumer Lua error counts] {boundary}: {:?}",
        state.lua_error_counts
    );
}

fn assert_no_recorded_errors(env: &WowLuaEnv, boundary: &str) {
    assert!(
        env.state().borrow().lua_errors.is_empty(),
        "{boundary} recorded Lua errors: {:?}",
        env.state().borrow().lua_errors
    );
}

fn invoke_original_item_consumer(env: &WowLuaEnv) -> bool {
    let result = env.eval::<(bool, String)>(ORIGINAL_ITEM_CONSUMER);
    if let Ok((_, observations)) = &result {
        assert!(
            !observations.is_empty(),
            "consumer observations must be retained"
        );
        for line in observations.lines() {
            eprintln!("[mount-item consumer observation] {line}");
        }
    }
    print_recorded_errors(env, "original SetItem / IsDressupReward invocation");
    if let Err(error) = &result {
        eprintln!("[mount-item consumer original invocation error] {error}");
    }
    let (value, _) =
        result.expect("original real-item consumer call must succeed and return a boolean");
    assert_no_recorded_errors(env, "original SetItem / IsDressupReward invocation");
    value
}

#[test]
fn item_dressability_original_player_choice_tracks_supplied_boolean_and_mount() {
    let ui = crate::common::blizzard_addon_harness::blizzard_ui_dir();
    let env = crate::common::blizzard_addon_harness::new_blizzard_addon_env(&ui);
    load_player_choice_closure(&env, &ui);
    env.state().borrow_mut().mount_item_relations.remove(&6948);
    for expected in [false, true, false] {
        env.state()
            .borrow_mut()
            .item_dressability
            .insert(6948, expected);
        assert_eq!(invoke_original_item_consumer(&env), expected);
    }
    env.state()
        .borrow_mut()
        .mount_item_relations
        .insert(6948, 401);
    assert_real_consumer_predicate(&env, false);
    assert!(invoke_original_item_consumer(&env));
    env.state().borrow_mut().mount_item_relations.remove(&6948);
    assert_real_consumer_predicate(&env, false);
    assert!(!invoke_original_item_consumer(&env));
    env.state().borrow_mut().item_dressability.remove(&6948);
    assert!(!invoke_original_item_consumer(&env));
}

fn assert_real_consumer_predicate(env: &WowLuaEnv, expected: bool) {
    let value = env
        .eval::<bool>(
            r#"
            local _, link = C_Item.GetItemInfo(6948)
            assert(type(link) == 'string' and link:find('|Hitem:6948:', 1, true))
            return C_Item.IsDressableItemByID(link)
            "#,
        )
        .expect("independent supplied predicate through real generated item link");
    assert_eq!(value, expected);
}
