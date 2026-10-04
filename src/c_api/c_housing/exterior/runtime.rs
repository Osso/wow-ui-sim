//! Retail exterior action boundary; native coercion and failure mapping remain unproved.

#[path = "core.rs"]
mod core;
#[path = "mutation.rs"]
mod mutation;
#[path = "queries.rs"]
mod queries;

use crate::c_api::c_housing::catalog::HousingCatalogEntryVariantID;
use crate::c_api::helpers::ensure_namespace;
use crate::lua_api::globals::state_backed_queries::dispatch_event_now;
use crate::lua_api::methods::{borrow_state_mut, create_table, table_set_static};
use crate::lua_bridge::{stack_val, table_set_rust_fn_static};
use mutation::{AttachedDecorAction, ExteriorChange};
use rilua::table_security::unwrap_secret;
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val, runtime_error};

pub(super) fn register(state: &mut LuaState) -> LuaResult<()> {
    let namespace = ensure_namespace(state, "C_HouseExterior")?;
    queries::register(state, namespace)?;
    for (name, callback) in [
        (
            "RemoveFixtureFromSelectedPoint",
            remove_fixture_from_selected_point as rilua::RustFn,
        ),
        (
            "SelectCoreFixtureOption",
            select_core_fixture_option as rilua::RustFn,
        ),
        (
            "SelectFixtureOption",
            select_fixture_option as rilua::RustFn,
        ),
        (
            "SetHouseExteriorSize",
            set_house_exterior_size as rilua::RustFn,
        ),
        (
            "SetHouseExteriorType",
            set_house_exterior_type as rilua::RustFn,
        ),
    ] {
        table_set_rust_fn_static(state, namespace, name, callback)?;
    }
    publish_decor_action_enum(state)?;
    Ok(())
}

fn publish_decor_action_enum(state: &mut LuaState) -> LuaResult<()> {
    let enum_namespace = ensure_namespace(state, "Enum")?;
    let action_enum = create_table(state);
    state.push(action_enum);
    table_set_static(
        state,
        Val::Table(enum_namespace),
        "HousingFixtureDecorAction",
        action_enum,
    );
    table_set_static(state, action_enum, "Store", Val::Num(0.0));
    table_set_static(state, action_enum, "Detach", Val::Num(1.0));
    state.top -= 1;
    Ok(())
}

fn remove_fixture_from_selected_point(state: &mut LuaState) -> LuaResult<u32> {
    let api = "C_HouseExterior.RemoveFixtureFromSelectedPoint";
    let original = stack_val(state, 1);
    let action = authenticate(state, original, api, 1)?;
    let action = read_action(action, api, 1)?;
    let (response, stored_variants) = {
        let mut sim = borrow_state_mut(state)?;
        mutation::remove_fixture(&mut sim.housing, action)?
    };
    publish_exterior_response(
        state,
        ExteriorChange::Fixture.event(),
        response,
        stored_variants,
    )
}

fn select_core_fixture_option(state: &mut LuaState) -> LuaResult<u32> {
    let (target, action) = read_arguments(state, core::API)?;
    let (response, stored_variants) = {
        let mut sim = borrow_state_mut(state)?;
        core::update_core_fixture(&mut sim.housing, target, action)?
    };
    // INFERRED: share the existing fixture-response and storage publication policy.
    publish_exterior_response(
        state,
        ExteriorChange::Fixture.event(),
        response,
        stored_variants,
    )
}

fn select_fixture_option(state: &mut LuaState) -> LuaResult<u32> {
    change_exterior(state, ExteriorChange::Fixture)
}

fn set_house_exterior_size(state: &mut LuaState) -> LuaResult<u32> {
    change_exterior(state, ExteriorChange::Size)
}

fn set_house_exterior_type(state: &mut LuaState) -> LuaResult<u32> {
    change_exterior(state, ExteriorChange::Type)
}

fn change_exterior(state: &mut LuaState, change: ExteriorChange) -> LuaResult<u32> {
    let (target, action) = read_arguments(state, change.api())?;
    let (response, stored_variants) = {
        let mut sim = borrow_state_mut(state)?;
        mutation::update_exterior(&mut sim.housing, change, target, action)?
    };
    publish_exterior_response(state, change.event(), response, stored_variants)
}

fn publish_exterior_response(
    state: &mut LuaState,
    event: &str,
    response: i32,
    stored_variants: Vec<HousingCatalogEntryVariantID>,
) -> LuaResult<u32> {
    // No borrow or pending writes survive callbacks; reentry sees committed state.
    for variant in stored_variants {
        super::super::catalog::publish_storage_update(state, &variant)?;
    }
    dispatch_event_now(state, event, &[Val::Num(f64::from(response))])?;
    Ok(0)
}

fn read_arguments(state: &LuaState, api: &str) -> LuaResult<(u32, AttachedDecorAction)> {
    // Authenticate BOTH originals before type checks, defaults, or model access.
    let target = authenticate(state, stack_val(state, 1), api, 1)?;
    let action = authenticate(state, stack_val(state, 2), api, 2)?;
    let target = read_target(target, api)?;
    let action = read_action(action, api, 2)?;
    Ok((target, action))
}

fn read_action(value: Val, api: &str, position: u32) -> LuaResult<AttachedDecorAction> {
    match value {
        Val::Nil | Val::Num(0.0) => Ok(AttachedDecorAction::Store),
        Val::Num(1.0) => Ok(AttachedDecorAction::Detach),
        _ => Err(runtime_error(format!(
            "{api}: argument {position} must be Store0 or Detach1"
        ))),
    }
}

fn authenticate(state: &LuaState, original: Val, api: &str, position: u32) -> LuaResult<Val> {
    unwrap_secret(state, original)
        .map_err(|_| runtime_error(format!("{api}: argument {position} secret access denied")))
}

fn is_positive_integral_u32(number: f64) -> bool {
    let integral = number.is_finite() && number.fract() == 0.0;
    let in_range = number > 0.0 && number <= f64::from(u32::MAX);
    integral && in_range
}

fn read_target(value: Val, api: &str) -> LuaResult<u32> {
    match value {
        Val::Num(number) if is_positive_integral_u32(number) => Ok(number as u32),
        _ => Err(runtime_error(format!(
            "{api}: argument 1 must be a positive integral u32 number"
        ))),
    }
}
