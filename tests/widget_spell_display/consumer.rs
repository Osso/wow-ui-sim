//! Real cached constructor/manager/events; no provider, Setup, pool, spell or vendor overrides.
use super::discrete_fixtures::assert_lua;
use super::feed::supply_empty_set;
use super::fixtures::{ID, SET_ID, assert_initial_snapshot, entry, seed_record, supplied_record};
use crate::common::blizzard_addon_harness::{
    load_blizzard_addon_closure_into_env, new_blizzard_addon_env,
};
use wow_ui_sim::c_api::ui_widget_manager::{SpellDisplayRecord, WidgetVisualization};
use wow_ui_sim::lua_api::WowLuaEnv;

fn load_and_register_spell_container_env(attached: bool) -> WowLuaEnv {
    let cache = wow_ui_sim::paths::default_blizzard_ui_addons_path().expect("active profile cache");
    let env = new_blizzard_addon_env(&cache);
    seed_record(&env, ID, supplied_record());
    assert_initial_snapshot(&env);
    // No cached construction before supplied getter result boundary.
    supply_empty_set(&env);
    wow_ui_sim::xml::register_intrinsic_templates();
    let loaded = load_blizzard_addon_closure_into_env(
        &env,
        &cache,
        &[
            "Blizzard_SharedXMLGame",
            "Blizzard_Fonts_Shared",
            "Blizzard_FrameEffects",
            "Blizzard_UIWidgets",
        ],
        &[],
    );
    assert!(loaded.iter().any(|name| name == "Blizzard_UIWidgets"));
    assert_lua(
        &env,
        "grounded scalar and cached template",
        r#"
        assert(Enum.UIWidgetVisualizationType.SpellDisplay == 13)
        assert(Enum.UIWidgetVisualizationType.DoubleStateIconRow == 14)
        assert(UIWidgetManager:GetWidgetTypeInfo(13))
        assert(Enum.WidgetShownState.Shown == 1 and Enum.WidgetShownState.Hidden == 0)
        assert(Enum.WidgetEnabledState.White == 3)
        assert(Enum.WidgetIconSizeType.Large == 2)
        assert(Enum.SpellDisplayIconDisplayType.Buff == 0)
        assert(Enum.SpellDisplayTextShownStateType.Shown == 0)
        assert(Enum.SpellDisplayBorderColor.Blue == 8)
        assert(Enum.SpellDisplayTint.None == 0 and Enum.SpellDisplayTint.Red == 1)
        assert(Enum.WidgetShowGlowState.HideGlow == 0)
        assert(Enum.UIWidgetFontType.Normal == 0 and Enum.UIWidgetTextSizeType.Medium16Pt == 1)
        assert(Enum.WidgetTextHorizontalAlignmentType.Left == 0)
        assert(Enum.UIWidgetTooltipLocation.Top == 4 and Enum.WidgetAnimationType.None == 0)
        assert(Enum.UIWidgetScale.OneHundred == 0 and Enum.UIWidgetLayoutDirection.Vertical == 1)
        assert(Enum.UIWidgetModelSceneLayer.None == 0 and Enum.UIWidgetSetLayoutDirection.Vertical == 0)
        local spell = assert(C_Spell.GetSpellInfo(35395), 'existing repository spell catalog prerequisite')
        assert(spell.name == 'Crusader Strike' and spell.iconID == 135891)
        SpellContainer = CreateFrame('Frame', nil, UIParent, 'UIWidgetContainerTemplate')
        SpellContainer:SetPoint('CENTER', UIParent, 'CENTER')
    "#,
    );
    let unit = if attached { "'player'" } else { "nil" };
    assert_lua(
        &env,
        "registered real empty container",
        &format!(
            r#"
        SpellContainer:RegisterForWidgetSet(431, nil, nil, {unit})
        assert(SpellContainer:IsRegisteredForWidgetSet(431))
        assert(UIWidgetManager.registeredWidgetContainers[SpellContainer])
        assert(SpellContainer:IsEventRegistered('UPDATE_UI_WIDGET'))
        assert(SpellContainer:IsEventRegistered('UPDATE_ALL_UI_WIDGETS'))
        assert(next(SpellContainer.widgetFrames) == nil)
    "#
        ),
    );
    flush(&env, 0);
    env
}

fn supply(env: &WowLuaEnv, record: SpellDisplayRecord, attached: bool) {
    let mut membership = entry();
    if attached {
        membership.unit_token = Some("player".into());
    }
    env.update_ui_widget(
        SET_ID,
        membership,
        WidgetVisualization::SpellDisplay(record),
    )
    .expect("typed feed");
}

fn flush(env: &WowLuaEnv, count: usize) {
    env.fire_on_update(0.0).expect("actual cached OnUpdate");
    assert_lua(
        env,
        "real layout and pool counts",
        &format!(
            r#"
        assert(not SpellContainer.dirtyLayout)
        assert(SpellContainer:GetNumWidgetsShowing() == {count})
        assert(SpellContainer.widgetPools:GetNumActive() == {count})
        assert(SpellContainer.numTimers == 0 and SpellContainer.ticker == nil)
    "#
        ),
    );
}

#[test]
fn spell_display_cached_activation_update_hidden_restore_remove() {
    let env = load_and_register_spell_container_env(false);
    supply(&env, supplied_record(), false);
    assert_lua(
        &env,
        "actual event activation",
        r#"
        SpellFrame = assert(SpellContainer.widgetFrames[761301])
        assert(SpellFrame:IsShown() and SpellFrame.hadSpellID)
        assert(SpellFrame.Spell.spellID == 35395)
        assert(SpellFrame.Spell.Text:GetText() == 'Supplied spell')
        assert(SpellFrame.Spell.StackCount:GetText() == '3')
        assert(SpellFrame.Spell.tooltip == 'Supplied spell tooltip')
        assert(SpellFrame:GetWidth() == SpellFrame.Spell:GetWidth() and SpellFrame:GetWidth() > 1)
        assert(SpellFrame:GetHeight() == SpellFrame.Spell:GetHeight() + 2)
        assert(SpellFrame.Spell.Icon:GetWidth() == 36)
        assert(not SpellFrame.Spell.EarnedCheck:IsShown())
        assert(SpellFrame.rarityPipPool:GetNumActive() == 0, 'non-world-loot branch only')
    "#,
    );
    flush(&env, 1);
    let mut row = supplied_record();
    row.spell_info.text = "Updated spell".into();
    row.spell_info.tooltip = "Updated tooltip".into();
    row.spell_info.stack_display = 4.0;
    row.spell_info.tint = 1;
    row.spell_info.show_as_earned = true;
    row.widget_size_setting = 206.25;
    supply(&env, row.clone(), false);
    assert_lua(
        &env,
        "actual incremental nested presentation",
        r#"
        assert(SpellContainer.widgetFrames[761301] == SpellFrame)
        assert(SpellFrame.Spell.Text:GetText() == 'Updated spell')
        assert(SpellFrame.Spell.tooltip == 'Updated tooltip')
        assert(SpellFrame.Spell.StackCount:GetText() == '4')
        assert(SpellFrame.Spell.EarnedCheck:IsShown())
        local r,g,b = SpellFrame.Spell.Icon:GetVertexColor()
        assert(r == 1 and g == 0 and b == 0)
        assert(SpellFrame.Spell.Text:GetWidth() == 206.25 - 36)
        assert(savedSpell.spellInfo.text == 'Supplied spell' and savedSpell.spellInfo.tint == 0)
    "#,
    );
    flush(&env, 1);
    row.spell_info.shown_state = 0;
    supply(&env, row.clone(), false);
    assert_lua(
        &env,
        "nested Hidden releases actual consumer",
        r#"
        assert(SpellContainer.widgetFrames[761301] == nil and not SpellFrame:IsShown())
        assert(C_UIWidgetManager.GetSpellDisplayVisualizationInfo(761301).spellInfo.shownState == 0)
    "#,
    );
    flush(&env, 0);
    row.spell_info.shown_state = 1;
    supply(&env, row, false);
    assert_lua(
        &env,
        "restored from real manager pool",
        r#"
        RestoredSpell = assert(SpellContainer.widgetFrames[761301])
        assert(RestoredSpell:IsShown() and RestoredSpell.Spell.Text:GetText() == 'Updated spell')
    "#,
    );
    flush(&env, 1);
    assert!(
        env.remove_ui_widget(SET_ID, ID)
            .expect("membership removal")
    );
    assert_lua(
        &env,
        "actual full reconciliation retains supplied record",
        r#"
        assert(next(SpellContainer.widgetFrames) == nil and not RestoredSpell:IsShown())
        assert(#C_UIWidgetManager.GetAllWidgetsBySetID(431) == 0)
        assert(C_UIWidgetManager.GetSpellDisplayVisualizationInfo(761301).spellInfo.text == 'Updated spell')
        assert(savedSpell.spellInfo.text == 'Supplied spell')
    "#,
    );
    flush(&env, 0);
}

#[test]
fn spell_display_unit_context_isolates_original_containers() {
    let env = load_and_register_spell_container_env(true);
    assert_lua(
        &env,
        "second unit container",
        r#"
        OtherContainer = CreateFrame('Frame', nil, UIParent, 'UIWidgetContainerTemplate')
        OtherContainer:RegisterForWidgetSet(431, nil, nil, 'target')
    "#,
    );
    let mut player = entry();
    player.unit_token = Some("player".into());
    let mut target = entry();
    target.unit_token = Some("target".into());
    let mut player_record = supplied_record();
    player_record.spell_info.text = "Player widget".into();
    let mut target_record = supplied_record();
    target_record.spell_info.text = "Target widget".into();
    env.update_ui_widget(
        SET_ID,
        player.clone(),
        WidgetVisualization::SpellDisplay(player_record),
    )
    .unwrap();
    env.update_ui_widget(
        SET_ID,
        target.clone(),
        WidgetVisualization::SpellDisplay(target_record),
    )
    .unwrap();
    assert_lua(
        &env,
        "per-unit queries through original consumer",
        r#"
        SpellContainer:ProcessAllWidgets()
        OtherContainer:ProcessAllWidgets()
        assert(SpellContainer.widgetFrames[761301].Spell.Text:GetText() == 'Player widget', 'player widget must remain distinct')
        assert(OtherContainer.widgetFrames[761301].Spell.Text:GetText() == 'Target widget')
        SpellContainer:UnregisterForWidgetSet()
        events = 0
        listener = CreateFrame('Frame')
        listener:RegisterEvent('UPDATE_UI_WIDGET')
        listener:SetScript('OnEvent', function() events = events + 1 end)
    "#,
    );
    let mut changed = supplied_record();
    changed.spell_info.text = "Unregistered player".into();
    env.update_ui_widget(SET_ID, player, WidgetVisualization::SpellDisplay(changed))
        .unwrap();
    assert_lua(
        &env,
        "unregistered update is not delivered",
        "assert(events == 0)",
    );
    let mut changed = supplied_record();
    changed.spell_info.text = "Target updated".into();
    env.update_ui_widget(SET_ID, target, WidgetVisualization::SpellDisplay(changed))
        .unwrap();
    assert_lua(
        &env,
        "registered target update",
        r#"
        assert(events == 1)
        assert(OtherContainer.widgetFrames[761301].Spell.Text:GetText() == 'Target updated')
    "#,
    );
}

#[test]
fn spell_display_remove_one_unit_preserves_other_container() {
    use wow_ui_sim::c_api::ui_widget_manager::ProcessingUnit;
    let env = load_and_register_spell_container_env(true);
    assert_lua(
        &env,
        "second unit",
        r#"
        OtherContainer = CreateFrame('Frame', nil, UIParent, 'UIWidgetContainerTemplate')
        OtherContainer:RegisterForWidgetSet(431, nil, nil, 'target')
    "#,
    );
    for (unit, text) in [("player", "Player widget"), ("target", "Target widget")] {
        let mut membership = entry();
        membership.unit_token = Some(unit.into());
        let mut row = supplied_record();
        row.spell_info.text = text.into();
        env.update_ui_widget(SET_ID, membership, WidgetVisualization::SpellDisplay(row))
            .unwrap();
    }
    let mut membership = entry();
    membership.unit_token = Some("player".into());
    let mut updated = supplied_record();
    updated.spell_info.text = "Player replacement".into();
    env.update_ui_widget(
        SET_ID,
        membership,
        WidgetVisualization::SpellDisplay(updated),
    )
    .unwrap();
    assert_lua(
        &env,
        "unit replacement leaves target container",
        r#"
        assert(SpellContainer.widgetFrames[761301].Spell.Text:GetText() == 'Player replacement')
        assert(OtherContainer.widgetFrames[761301].Spell.Text:GetText() == 'Target widget')
    "#,
    );
    env.remove_ui_widget_for_unit(SET_ID, ID, ProcessingUnit::Token("player".into()))
        .unwrap();
    assert_lua(
        &env,
        "unit removal leaves target container",
        r#"
        assert(SpellContainer.widgetFrames[761301] == nil, 'removed player widget')
        assert(OtherContainer.widgetFrames[761301].Spell.Text:GetText() == 'Target widget', 'target widget remains')
    "#,
    );
}

#[test]
fn spell_display_guid_context_delivers_to_original_container() {
    let env = load_and_register_spell_container_env(false);
    assert_lua(
        &env,
        "GUID container",
        r#"
        GuidContainer = CreateFrame('Frame', nil, UIParent, 'UIWidgetContainerTemplate')
        GuidContainer:RegisterForWidgetSet(431, nil, nil, {unit='Creature-1', isGuid=true})
    "#,
    );
    let mut membership = entry();
    membership.unit_token = Some("Creature-1".into());
    let mut row = supplied_record();
    row.spell_info.text = "GUID widget".into();
    env.update_ui_widget_for_guid(SET_ID, membership, WidgetVisualization::SpellDisplay(row))
        .unwrap();
    assert_lua(
        &env,
        "GUID event and query",
        r#"
        assert(GuidContainer.widgetFrames[761301].Spell.Text:GetText() == 'GUID widget')
        C_UIWidgetManager.SetProcessingUnitGuid('Creature-1')
        assert(C_UIWidgetManager.GetSpellDisplayVisualizationInfo(761301).spellInfo.text == 'GUID widget')
        C_UIWidgetManager.SetProcessingUnit('Creature-1')
        assert(C_UIWidgetManager.GetSpellDisplayVisualizationInfo(761301) == nil)
        C_UIWidgetManager.SetProcessingUnit(nil)
        assert(C_UIWidgetManager.GetSpellDisplayVisualizationInfo(761301) == nil)
    "#,
    );
}

#[test]
fn spell_display_attached_player_context_separate_from_world_loot_policy() {
    let env = load_and_register_spell_container_env(true);
    supply(&env, supplied_record(), true);
    assert_lua(
        &env,
        "container attachedUnit propagation only",
        r#"
        local frame = assert(SpellContainer.widgetFrames[761301])
        assert(not C_WorldLootObject.IsWorldLootObject('player'))
        assert(SpellContainer.attachedUnit == 'player' and frame.attachedUnit == 'player')
        assert(frame.Spell.spellInfo.text == 'Supplied spell')
        assert(C_UIWidgetManager.GetSpellDisplayVisualizationInfo(761301).attachedUnit == nil,
            'attachedUnit is cached context, not a supplied API field')
    "#,
    );
    flush(&env, 1);
}

#[test]
fn spell_display_seeded_world_loot_and_api_queries() {
    use wow_ui_sim::c_api::world_loot_object::WorldLootObject;
    let env = load_and_register_spell_container_env(true);
    assert_lua(
        &env,
        "world loot event listener",
        r#"
        lootEvents = {}
        local listener = CreateFrame('Frame')
        listener:RegisterEvent('WORLD_LOOT_OBJECT_INFO_UPDATED')
        listener:SetScript('OnEvent', function(_, _, guid) table.insert(lootEvents, guid) end)
        assert(not C_WorldLootObject.IsWorldLootObject('player'))
        assert(C_WorldLootObject.GetWorldLootObjectInfo('player') == nil)
    "#,
    );
    let object = WorldLootObject {
        unit_token: "player".into(),
        inventory_type: 1,
        at_max_quality: true,
        is_upgrade: false,
        distance_squared: 9.0,
        in_range: true,
    };
    env.supply_world_loot_object("GameObject-1", object.clone())
        .unwrap();
    env.state()
        .borrow_mut()
        .world_loot_objects
        .slot_matches
        .insert((1, 1), true);
    assert_lua(
        &env,
        "seeded queries and event",
        r#"
        assert(#lootEvents == 1 and lootEvents[1] == 'GameObject-1')
        assert(C_WorldLootObject.IsWorldLootObject('player'))
        assert(C_WorldLootObject.IsWorldLootObjectByGUID('GameObject-1'))
        assert(not C_WorldLootObject.IsWorldLootObjectByGUID('missing'))
        assert(C_WorldLootObject.IsWorldLootObjectInRange('player'))
        assert(not C_WorldLootObject.IsWorldLootObjectInRange('missing'))
        assert(C_WorldLootObject.GetWorldLootObjectDistanceSquared('player') == 9)
        assert(C_WorldLootObject.GetWorldLootObjectDistanceSquared('missing') == nil)
        local a = C_WorldLootObject.GetWorldLootObjectInfo('player')
        local b = C_WorldLootObject.GetWorldLootObjectInfoByGUID('GameObject-1')
        assert(a.inventoryType == 1 and a.atMaxQuality and not a.isUpgrade)
        assert(b.inventoryType == 1 and b.atMaxQuality and not b.isUpgrade)
        assert(C_WorldLootObject.GetWorldLootObjectInfoByGUID('missing') == nil)
        assert(C_WorldLootObject.DoesSlotMatchInventoryType(1, 1))
        assert(not C_WorldLootObject.DoesSlotMatchInventoryType(2, 1))
        assert(not pcall(C_WorldLootObject.OnWorldLootObjectClick, 'player', nil))
        assert(not pcall(C_WorldLootObject.OnWorldLootObjectClick, nil, true))
        C_WorldLootObject.OnWorldLootObjectClick('player', true)
    "#,
    );
    assert_eq!(
        env.state().borrow().world_loot_objects.clicks,
        [("player".into(), true)]
    );
    // The non-equippable path avoids an unrelated unsupported event.
    let mut non_equippable = object.clone();
    non_equippable.inventory_type = 0;
    env.supply_world_loot_object("GameObject-1", non_equippable)
        .unwrap();
    supply(&env, supplied_record(), true);
    assert_lua(
        &env,
        "original spell display world loot branch",
        r#"
        local frame = assert(SpellContainer.widgetFrames[761301])
        assert(frame.displayedWorldLootObject == 'player')
        assert(frame.rarityPipPool:GetNumActive() == 3)
    "#,
    );
    let mut updated = object;
    updated.inventory_type = 0;
    updated.in_range = false;
    updated.is_upgrade = true;
    env.supply_world_loot_object("GameObject-1", updated)
        .unwrap();
    assert_lua(
        &env,
        "updated state",
        r#"
        assert(#lootEvents == 3 and lootEvents[2] == 'GameObject-1' and lootEvents[3] == 'GameObject-1')
        assert(not C_WorldLootObject.IsWorldLootObjectInRange('player'))
        assert(C_WorldLootObject.GetWorldLootObjectInfo('player').isUpgrade)
    "#,
    );
}

#[test]
fn spell_display_zero_spell_polling_uses_real_on_update_without_feed_notification() {
    let env = load_and_register_spell_container_env(false);
    let mut row = supplied_record();
    row.spell_info.spell_id = 0.0;
    supply(&env, row, false);
    assert_lua(
        &env,
        "zero-spell consumer before poll",
        r#"
        PollingSpell = assert(SpellContainer.widgetFrames[761301])
        assert(not PollingSpell.hadSpellID and PollingSpell.Spell.spellID == 0)
    "#,
    );
    // Explicit host map replacement without event isolates polling from event Setup.
    seed_record(&env, ID, supplied_record());
    assert_lua(
        &env,
        "poll has not run",
        "assert(PollingSpell.Spell.spellID == 0 and not PollingSpell.hadSpellID)",
    );
    env.fire_on_update(0.01)
        .expect("actual XML OnUpdate polling");
    assert_lua(
        &env,
        "zero-spell becomes supplied catalog spell",
        r#"
        assert(SpellContainer.widgetFrames[761301] == PollingSpell)
        assert(PollingSpell.hadSpellID and PollingSpell.Spell.spellID == 35395)
        assert(PollingSpell.Spell.Text:GetText() == 'Supplied spell')
    "#,
    );
}
