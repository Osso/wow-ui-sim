//! Supplied PTR GameRules inputs and finite 70493 consumer declarations.
use crate::client_profile::{ACTIVE, ClientProfile};
use crate::lua_api::methods::{borrow_state, borrow_state_mut, create_table, table_set};
use crate::lua_bridge::{FromStack, table_set_rust_fn};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

pub(crate) fn register(state: &mut LuaState) -> LuaResult<()> {
    if ACTIVE != ClientProfile::Ptr {
        return Ok(());
    }
    let namespace = super::helpers::ensure_namespace(state, "C_GameRules")?;
    let functions: &[(&str, rilua::RustFn)] = &[
        ("IsHardcoreActive", is_hardcore_active),
        ("IsSelfFoundAllowed", is_self_found_allowed),
        ("IsSDHDToggleEnabled", is_sd_hd_toggle_enabled),
        ("AccountHasSDEnabled", account_has_sd_enabled),
        ("SetSDHDToggleValue", set_sd_hd_toggle_value),
    ];
    for &(name, function) in functions {
        table_set_rust_fn(state, namespace, name, function)?;
    }
    let enums = super::helpers::ensure_global_table(state, "Enum");
    register_usage_locations(state, enums);
    register_recent_allies_categories(state, enums);
    Ok(())
}

fn is_hardcore_active(state: &mut LuaState) -> LuaResult<u32> {
    let active = borrow_state(state)?.game_rules.hardcore_active;
    state.push(Val::Bool(active));
    Ok(1)
}

fn is_self_found_allowed(state: &mut LuaState) -> LuaResult<u32> {
    let allowed = borrow_state(state)?.game_rules.self_found_allowed;
    state.push(Val::Bool(allowed));
    Ok(1)
}

fn is_sd_hd_toggle_enabled(state: &mut LuaState) -> LuaResult<u32> {
    let enabled = borrow_state(state)?.game_rules.sd_hd_toggle_enabled;
    state.push(Val::Bool(enabled));
    Ok(1)
}

fn account_has_sd_enabled(state: &mut LuaState) -> LuaResult<u32> {
    let enabled = borrow_state(state)?.game_rules.account_has_sd_enabled;
    state.push(Val::Bool(enabled));
    Ok(1)
}

fn set_sd_hd_toggle_value(state: &mut LuaState) -> LuaResult<u32> {
    let set_to_sd = bool::from_stack(state, 1)?;
    let usage_location = i32::from_stack(state, 2)?;
    let mut sim = borrow_state_mut(state)?;
    sim.game_rules.account_has_sd_enabled = set_to_sd;
    sim.game_rules.sd_toggle_usage_location = Some(usage_location);
    Ok(0)
}

fn register_usage_locations(state: &mut LuaState, enums: Val) {
    // 70493 GameRulesDocumentation.lua:467–477.
    publish(
        state,
        enums,
        "SdToggleUsageLocationUI",
        &[
            ("BeforeLogin", 0),
            ("CharacterCreation", 1),
            ("BarberShop", 2),
            ("OptionsMenu", 3),
        ],
    );
    publish(
        state,
        enums,
        "SdToggleUsageLocationUIMeta",
        &[("MinValue", 0), ("MaxValue", 3), ("NumValues", 4)],
    );
}

fn register_recent_allies_categories(state: &mut LuaState, enums: Val) {
    // 70493 RecentAlliesConstantsDocumentation.lua:6–18.
    publish(
        state,
        enums,
        "RecentAlliesInteractionCategoryFilter",
        &[
            ("Professions", 0),
            ("PvP", 1),
            ("Raiding", 2),
            ("Dungeons", 3),
            ("Delves", 4),
            ("Questing", 5),
        ],
    );
    publish(
        state,
        enums,
        "RecentAlliesInteractionCategoryFilterMeta",
        &[("MinValue", 0), ("MaxValue", 5), ("NumValues", 6)],
    );
}

fn publish(state: &mut LuaState, enums: Val, name: &str, fields: &[(&str, i32)]) {
    let table = create_table(state);
    for &(field, value) in fields {
        table_set(state, table, field, Val::Num(f64::from(value)));
    }
    table_set(state, enums, name, table);
}
