use super::*;
use crate::lua_bridge::table_set_rust_fn_static;
use rilua::vm::gc::arena::GcRef;
use rilua::vm::table::Table;
use rilua::{LuaApiMut, LuaResult, RustFn, Val};

type ActionBarMethod = (&'static str, RustFn);

const GENERAL_METHODS: &[ActionBarMethod] = &[
    ("GetBonusBarIndexForSlot", get_bonus_bar_index_for_slot),
    ("IsOnBarOrSpecialBar", is_on_bar_or_special_bar),
    ("FindSpellActionButtons", find_spell_action_buttons),
    (
        "GetCurrentActionBarByClass",
        get_current_action_bar_by_class,
    ),
    ("HasFlyoutActionButtons", has_flyout_action_buttons),
    ("EnableActionRangeCheck", enable_action_range_check),
    ("IsAssistedCombatAction", is_assisted_combat_action),
    (
        "HasAssistedCombatActionButtons",
        has_assisted_combat_action_buttons,
    ),
];

const BASIC_SLOT_VALUE_METHODS: &[ActionBarMethod] = &[
    ("GetActionText", get_action_text),
    ("GetActionCount", get_action_count),
    ("GetActionDisplayCount", get_action_display_count),
    ("GetActionUseCount", get_action_use_count),
];

const BASIC_SLOT_PREDICATE_METHODS: &[ActionBarMethod] = &[
    ("IsConsumableAction", is_consumable_action),
    ("IsStackableAction", is_stackable_action),
    ("IsItemAction", is_item_action),
    ("IsAttackAction", is_attack_action),
    ("IsAutoRepeatAction", is_auto_repeat_action),
    ("IsEquippedAction", is_equipped_action),
    ("IsEquippedGearOutfitAction", is_equipped_gear_outfit_action),
    ("IsHelpfulAction", is_helpful_action),
    ("IsHarmfulAction", is_harmful_action),
    ("IsPressHoldReleaseSpell", is_press_hold_release_spell),
];

const COOLDOWN_SLOT_METHODS: &[ActionBarMethod] = &[
    (
        "GetActionLossOfControlCooldown",
        get_action_loss_of_control_cooldown,
    ),
    (
        "GetActionLossOfControlCooldownInfo",
        get_action_loss_of_control_cooldown_info,
    ),
    ("UsesActionText", uses_action_text),
    ("GetActionChargeDuration", get_action_charge_duration),
    ("GetActionCooldownDuration", get_action_cooldown_duration),
    (
        "GetActionLossOfControlCooldownDuration",
        get_action_loss_of_control_cooldown_duration,
    ),
    ("GetSpell", get_spell),
    (
        "GetItemActionOnEquipSpellID",
        get_item_action_on_equip_spell_id,
    ),
];

const SLOT_MUTATION_METHODS: &[ActionBarMethod] = &[
    ("PutActionInSlot", put_action_in_slot),
    ("ForceUpdateAction", force_update_action),
    ("GetProfessionQualityInfo", get_profession_quality_info),
];

pub fn register_all(lua: &mut rilua::Lua) -> crate::Result<()> {
    let state = lua.state_mut();
    let table_ref = ensure_namespace_table(state, C_ACTION_BAR);
    register_general_methods(state, table_ref)?;
    register_page_methods(state, table_ref)?;
    register_state_queries(state, table_ref)?;
    register_basic_slot_methods(state, table_ref)?;
    register_cooldown_slot_methods(state, table_ref)?;
    register_pet_slot_methods(state, table_ref)?;
    register_stateful_methods(state, table_ref)?;
    register_slot_mutation_methods(state, table_ref)?;
    crate::c_api::action_macros::register(state, table_ref)?;
    Ok(())
}

fn register_general_methods(state: &mut LuaState, table_ref: GcRef<Table>) -> LuaResult<()> {
    install_action_bar_methods(state, table_ref, GENERAL_METHODS)
}

fn install_action_bar_methods(
    state: &mut LuaState,
    table_ref: GcRef<Table>,
    methods: &[ActionBarMethod],
) -> LuaResult<()> {
    for &(name, func) in methods {
        table_set_rust_fn_static(state, table_ref, name, func)?;
    }
    Ok(())
}

fn register_page_methods(state: &mut LuaState, table_ref: GcRef<Table>) -> LuaResult<()> {
    let methods: [(&'static str, RustFn); 10] = [
        ("GetActionBarPage", get_action_bar_page),
        ("SetActionBarPage", set_action_bar_page),
        ("GetExtraBarIndex", get_extra_bar_index),
        ("GetMultiCastBarIndex", get_multicast_bar_index),
        ("GetVehicleBarIndex", get_vehicle_bar_index),
        ("GetOverrideBarIndex", get_override_bar_index),
        ("GetTempShapeshiftBarIndex", get_temp_shapeshift_bar_index),
        ("GetBonusBarIndex", get_bonus_bar_index),
        ("GetBonusBarOffset", get_bonus_bar_offset),
        ("GetOverrideBarSkin", get_override_bar_skin),
    ];
    install_action_bar_methods(state, table_ref, &methods)
}

fn register_state_queries(state: &mut LuaState, table_ref: GcRef<Table>) -> LuaResult<()> {
    let methods: [(&'static str, RustFn); 6] = [
        ("HasVehicleActionBar", has_vehicle_action_bar),
        ("HasOverrideActionBar", has_override_action_bar),
        ("HasBonusActionBar", has_bonus_action_bar),
        ("HasTempShapeshiftActionBar", has_temp_shapeshift_action_bar),
        ("HasExtraActionBar", has_extra_action_bar),
        ("IsPossessBarVisible", is_possess_bar_visible),
    ];
    install_action_bar_methods(state, table_ref, &methods)
}

fn register_basic_slot_methods(state: &mut LuaState, table_ref: GcRef<Table>) -> LuaResult<()> {
    install_action_bar_methods(state, table_ref, BASIC_SLOT_VALUE_METHODS)?;
    install_action_bar_methods(state, table_ref, BASIC_SLOT_PREDICATE_METHODS)
}

fn register_cooldown_slot_methods(state: &mut LuaState, table_ref: GcRef<Table>) -> LuaResult<()> {
    install_action_bar_methods(state, table_ref, COOLDOWN_SLOT_METHODS)
}

fn register_pet_slot_methods(state: &mut LuaState, table_ref: GcRef<Table>) -> LuaResult<()> {
    let methods: [(&'static str, RustFn); 7] = [
        ("FindFlyoutActionButtons", find_flyout_action_buttons),
        ("FindPetActionButtons", find_pet_action_buttons),
        ("GetPetActionPetBarIndices", get_pet_action_pet_bar_indices),
        ("RegisterActionUIButton", register_action_ui_button),
        ("IsAutoCastPetAction", is_auto_cast_pet_action),
        (
            "IsEnabledAutoCastPetAction",
            is_enabled_auto_cast_pet_action,
        ),
        ("ToggleAutoCastPetAction", toggle_auto_cast_pet_action),
    ];
    install_action_bar_methods(state, table_ref, &methods)
}

fn register_stateful_methods(state: &mut LuaState, table_ref: GcRef<Table>) -> LuaResult<()> {
    let methods: [(&'static str, RustFn); 6] = [
        ("HasAction", has_action),
        ("GetActionTexture", get_action_texture),
        ("IsUsableAction", is_usable_action),
        ("IsCurrentAction", is_current_action),
        ("GetActionCooldown", get_action_cooldown),
        ("GetActionCharges", get_action_charges),
    ];
    install_action_bar_methods(state, table_ref, &methods)
}

fn register_slot_mutation_methods(state: &mut LuaState, table_ref: GcRef<Table>) -> LuaResult<()> {
    install_action_bar_methods(state, table_ref, SLOT_MUTATION_METHODS)
}

fn ensure_namespace_table(state: &mut LuaState, namespace: &'static str) -> GcRef<Table> {
    let key = state.gc.intern_string_static(namespace.as_bytes());
    let global = state.global;
    let existing = state
        .gc
        .tables
        .get(global)
        .map(|table| table.get_str(key, &state.gc.string_arena));
    if let Some(Val::Table(table_ref)) = existing {
        return table_ref;
    }

    let table = create_table(state);
    let Val::Table(table_ref) = table else {
        unreachable!("create_table must return a table");
    };
    if let Some(global_table) = state.gc.tables.get_mut(global) {
        let _ = global_table.raw_set(Val::Str(key), table, &state.gc.string_arena);
    }
    state.gc.barrier_back(global);
    table_ref
}
