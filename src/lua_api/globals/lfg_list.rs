//! `C_LFGList` namespace — Group Finder search, category, and activity APIs.
//!
//! Backed by `SimState::lfg_category_info`, `lfg_activity_groups`,
//! `lfg_activities`, and `world.premade_listings`.

mod catalog;
mod counts;

pub use counts::{admin_set_applicant_counts, admin_set_application_counts};

use crate::event::Event;
use crate::lua_api::env::WowLuaAppData;
use crate::lua_api::methods::{
    borrow_state, borrow_state_mut, call_function_state, create_string, create_table, frame_ref,
    table_set,
};
use crate::lua_api::script_helpers::{get_dispatch_script, get_event_listeners};
use crate::lua_api::state_types::{LfgApplication, PendingTimer, PremadeListing};
use crate::lua_api::{next_timer_id, timer_layout};
use crate::lua_bridge::{FromStack, table_set_rust_fn_static};
use rilua::vm::closure::{Closure, RustClosure, RustFn};
use rilua::vm::gc::arena::GcRef;
use rilua::vm::state::LuaState;
use rilua::vm::table::Table;
use rilua::{LuaResult, Val, runtime_error};
use std::collections::HashMap;
use std::time::Instant;

fn fire_event_with_args(state: &mut LuaState, event_name: &str, args: Vec<Val>) -> LuaResult<()> {
    borrow_state_mut(state)?.events.push(Event {
        name: event_name.to_string(),
        args: Vec::new(),
    });
    for widget_id in get_event_listeners(state, event_name) {
        let Some(handler) = get_dispatch_script(state, widget_id, "OnEvent") else {
            continue;
        };
        let Ok(frame) = frame_ref(state, widget_id) else {
            continue;
        };
        let event_name_val = create_string(state, event_name);
        let mut call_args = Vec::with_capacity(2 + args.len());
        call_args.push(frame);
        call_args.push(event_name_val);
        call_args.extend(args.iter().cloned());
        let _ = call_function_state(state, handler, &call_args);
    }
    Ok(())
}

fn defer_lfg_search_results_event(state: &mut LuaState) -> LuaResult<()> {
    let callback = Val::Function(state.gc.alloc_closure(Closure::Rust(RustClosure::new(
        dispatch_lfg_search_results_received,
        "C_LFGList.SearchResultsReady",
    ))));
    let id = next_timer_id();
    timer_layout::store_timer_callback(state, id, callback);

    let app = state
        .app_data::<WowLuaAppData>()
        .ok_or_else(|| runtime_error("missing WowLuaAppData"))?;
    let owner_addon = {
        let sim = app.sim_state.borrow();
        sim.loading_addon_index.or(sim.executing_addon_index)
    };
    app.sim_state
        .borrow_mut()
        .rilua_timers
        .push_back(PendingTimer {
            id,
            fire_at: Instant::now(),
            interval: None,
            remaining: None,
            cancelled: false,
            owner_addon,
            callback_receives_timer: false,
            callback_arg: None,
        });
    Ok(())
}

fn dispatch_lfg_search_results_received(state: &mut LuaState) -> LuaResult<u32> {
    fire_event_with_args(state, "LFG_LIST_SEARCH_RESULTS_RECEIVED", Vec::new())?;
    Ok(0)
}

fn premade_listing(state: &LuaState, search_result_id: u32) -> Option<PremadeListing> {
    borrow_state(state)
        .ok()?
        .world
        .premade_listings
        .iter()
        .find(|listing| listing.search_result_id == search_result_id)
        .cloned()
}

fn build_search_result_info(state: &mut LuaState, listing: &PremadeListing) -> Val {
    let info = create_table(state);
    set_search_result_identity_fields(state, info, listing);
    set_search_result_activity_fields(state, info, listing);
    set_search_result_size_fields(state, info, listing);
    set_search_result_social_fields(state, info, listing);
    set_search_result_requirement_fields(state, info);
    info
}

fn set_search_result_identity_fields(state: &mut LuaState, info: Val, listing: &PremadeListing) {
    let name = create_string(state, &listing.name);
    let comment = create_string(state, &listing.comment);
    let leader_name = create_string(state, &listing.leader_name);
    let voice_chat = create_string(state, &listing.voice_chat);
    let party_guid = create_string(state, &listing.party_guid);
    table_set(
        state,
        info,
        "searchResultID",
        Val::Num(listing.search_result_id as f64),
    );
    table_set(state, info, "name", name);
    table_set(state, info, "comment", comment);
    table_set(state, info, "leaderName", leader_name);
    table_set(state, info, "voiceChat", voice_chat);
    table_set(state, info, "autoAccept", Val::Bool(listing.auto_accept));
    table_set(state, info, "isDelisted", Val::Bool(listing.is_delisted));
    table_set(state, info, "partyGUID", party_guid);
}

fn set_search_result_activity_fields(state: &mut LuaState, info: Val, listing: &PremadeListing) {
    // `activityID` is the legacy single-value form; `activityIDs` is the
    // current shape addons read (`searchResultInfo.activityIDs[1]`).
    table_set(
        state,
        info,
        "activityID",
        Val::Num(listing.activity_id as f64),
    );
    let activity_ids = activity_ids_table(state, listing);
    table_set(state, info, "activityIDs", activity_ids);
    table_set(
        state,
        info,
        "generalPlaystyle",
        Val::Num(listing.general_playstyle as f64),
    );
    table_set(
        state,
        info,
        "crossFactionListing",
        Val::Bool(listing.cross_faction_listing),
    );
    table_set(
        state,
        info,
        "leaderFactionGroup",
        Val::Num(listing.leader_faction_group as f64),
    );
}

fn activity_ids_table(state: &mut LuaState, listing: &PremadeListing) -> Val {
    let activity_ids = create_table(state);
    let Val::Table(activity_ids_ref) = activity_ids else {
        return activity_ids;
    };
    if let Some(t) = state.gc.tables.get_mut(activity_ids_ref) {
        let _ = t.raw_set(
            Val::Num(1.0),
            Val::Num(listing.activity_id as f64),
            &state.gc.string_arena,
        );
    }
    state.gc.barrier_back(activity_ids_ref);
    activity_ids
}

fn set_search_result_size_fields(state: &mut LuaState, info: Val, listing: &PremadeListing) {
    table_set(
        state,
        info,
        "numMembers",
        Val::Num(listing.num_members as f64),
    );
    table_set(
        state,
        info,
        "maxMembers",
        Val::Num(listing.max_members as f64),
    );
}

fn set_search_result_social_fields(state: &mut LuaState, info: Val, listing: &PremadeListing) {
    table_set(
        state,
        info,
        "numBNetFriends",
        Val::Num(listing.num_bnet_friends as f64),
    );
    table_set(
        state,
        info,
        "numCharFriends",
        Val::Num(listing.num_char_friends as f64),
    );
    table_set(
        state,
        info,
        "numGuildMates",
        Val::Num(listing.num_guild_mates as f64),
    );
}

fn set_search_result_requirement_fields(state: &mut LuaState, info: Val) {
    table_set(state, info, "requiredItemLevel", Val::Num(0.0));
    table_set(state, info, "requiredHonorLevel", Val::Num(0.0));
    table_set(state, info, "requiredDungeonScore", Val::Num(0.0));
    table_set(state, info, "requiredPvpRating", Val::Num(0.0));
    table_set(state, info, "questID", Val::Num(0.0));
    table_set(state, info, "age", Val::Num(0.0));
    table_set(state, info, "isWarMode", Val::Bool(false));
    set_patch_12_1_search_result_fields(state, info);
}

#[cfg(feature = "retail-12-1-0")]
fn set_patch_12_1_search_result_fields(state: &mut LuaState, info: Val) {
    table_set(state, info, "censored", Val::Bool(false));
}

#[cfg(not(feature = "retail-12-1-0"))]
fn set_patch_12_1_search_result_fields(_state: &mut LuaState, _info: Val) {}

fn get_search_result_info(state: &mut LuaState) -> LuaResult<u32> {
    let search_result_id = Option::<f64>::from_stack(state, 1)?.unwrap_or(0.0) as u32;
    let Some(listing) = premade_listing(state, search_result_id) else {
        return Ok(0);
    };
    let info = build_search_result_info(state, &listing);
    state.push(info);
    Ok(1)
}

fn has_search_result_info(state: &mut LuaState) -> LuaResult<u32> {
    let search_result_id = Option::<f64>::from_stack(state, 1)?.unwrap_or(0.0) as u32;
    let found = premade_listing(state, search_result_id).is_some();
    state.push(Val::Bool(found));
    Ok(1)
}

fn get_search_results(state: &mut LuaState) -> LuaResult<u32> {
    let search_result_ids = {
        let sim = borrow_state(state)?;
        sim.world
            .premade_listings
            .iter()
            .map(|listing| listing.search_result_id)
            .collect::<Vec<_>>()
    };
    let results = create_table(state);
    if let Val::Table(table_ref) = results {
        for (index, search_result_id) in search_result_ids.iter().enumerate() {
            if let Some(table) = state.gc.tables.get_mut(table_ref) {
                let _ = table.raw_set(
                    Val::Num(index as f64 + 1.0),
                    Val::Num(*search_result_id as f64),
                    &state.gc.string_arena,
                );
            }
        }
        state.gc.barrier_back(table_ref);
    }
    state.push(Val::Num(search_result_ids.len() as f64));
    state.push(results);
    Ok(2)
}

fn search(state: &mut LuaState) -> LuaResult<u32> {
    defer_lfg_search_results_event(state)?;
    Ok(0)
}

/// `GetFilteredSearchResults()` → `(totalResults, results)`. Mirrors
/// `GetSearchResults` once the user has applied filters in the panel —
/// the sim does not model server-side filter state, so the panel sees
/// every seeded listing.
fn get_filtered_search_results(state: &mut LuaState) -> LuaResult<u32> {
    get_search_results(state)
}

/// `GetSearchResultMemberCounts(resultID)` → display data table.
///
/// Shape consumed by `LFGListGroupDataDisplay_Update`:
///   `{ TANK = n, HEALER = n, DAMAGER = n, NOROLE = n,
///      classesByRole = { TANK = { WARRIOR = n, ... }, ... },
///      leaversByClass = { WARRIOR = n, ... } }`
///
/// `LFGListGroupDataDisplayEnumerate_Update` reads `displayData.NOROLE`
/// without a nil-guard (`numPlayers = TANK + HEALER + DAMAGER + NOROLE`),
/// so every key must be a number, not nil.
fn get_search_result_member_counts(state: &mut LuaState) -> LuaResult<u32> {
    let search_result_id = Option::<f64>::from_stack(state, 1)?.unwrap_or(0.0) as u32;
    let Some(listing) = premade_listing(state, search_result_id) else {
        state.push(Val::Nil);
        return Ok(1);
    };
    let display = create_table(state);
    set_search_result_role_counts(state, display, &listing);
    let classes_by_role = classes_by_role_table(state, &listing.classes_by_role);
    table_set(state, display, "classesByRole", classes_by_role);
    let leavers = create_table(state);
    table_set(state, display, "leaversByClass", leavers);
    state.push(display);
    Ok(1)
}

fn set_search_result_role_counts(state: &mut LuaState, display: Val, listing: &PremadeListing) {
    table_set(state, display, "TANK", Val::Num(listing.tanks as f64));
    table_set(state, display, "HEALER", Val::Num(listing.healers as f64));
    table_set(state, display, "DAMAGER", Val::Num(listing.damagers as f64));
    table_set(state, display, "NOROLE", Val::Num(listing.no_role as f64));
}

fn classes_by_role_table(
    state: &mut LuaState,
    classes_by_role: &HashMap<String, HashMap<String, i32>>,
) -> Val {
    let table = create_table(state);
    let Val::Table(table_ref) = table else {
        return table;
    };
    for (role, class_counts) in classes_by_role {
        let inner = class_counts_table(state, class_counts);
        raw_set_string_key(state, table_ref, role, inner);
    }
    state.gc.barrier_back(table_ref);
    table
}

fn class_counts_table(state: &mut LuaState, class_counts: &HashMap<String, i32>) -> Val {
    let table = create_table(state);
    let Val::Table(table_ref) = table else {
        return table;
    };
    for (class_name, count) in class_counts {
        raw_set_string_key(state, table_ref, class_name, Val::Num(*count as f64));
    }
    state.gc.barrier_back(table_ref);
    table
}

fn raw_set_string_key(state: &mut LuaState, table_ref: GcRef<Table>, key: &str, value: Val) {
    let Val::Str(key_ref) = create_string(state, key) else {
        return;
    };
    if let Some(table) = state.gc.tables.get_mut(table_ref) {
        let _ = table.raw_set(Val::Str(key_ref), value, &state.gc.string_arena);
    }
}

/// `GetGroupLeaverCountsByRole()` → `(tank, healer, damager)`. Sim has
/// no group-history model, so report zero leavers in every role.
fn get_group_leaver_counts_by_role(state: &mut LuaState) -> LuaResult<u32> {
    state.push(Val::Num(0.0));
    state.push(Val::Num(0.0));
    state.push(Val::Num(0.0));
    Ok(3)
}

/// `GetApplications()` → array of `searchResultID` for every pending
/// application the player has submitted.
fn get_applications(state: &mut LuaState) -> LuaResult<u32> {
    let result_ids = {
        let sim = borrow_state(state)?;
        sim.lfg_applications
            .iter()
            .map(|app| app.search_result_id)
            .collect::<Vec<_>>()
    };
    let result = create_table(state);
    if let Val::Table(table_ref) = result {
        for (index, rid) in result_ids.iter().enumerate() {
            if let Some(t) = state.gc.tables.get_mut(table_ref) {
                let _ = t.raw_set(
                    Val::Num(index as f64 + 1.0),
                    Val::Num(*rid as f64),
                    &state.gc.string_arena,
                );
            }
        }
        state.gc.barrier_back(table_ref);
    }
    state.push(result);
    Ok(1)
}

/// `GetPremadeGroupFinderStyle()` → style selector.
fn get_premade_group_finder_style(_state: &mut LuaState) -> LuaResult<u32> {
    state_push_num_zero(_state)
}

fn state_push_num_zero(state: &mut LuaState) -> LuaResult<u32> {
    state.push(rilua::Val::Num(0.0));
    Ok(1)
}

fn can_create_quest_group(_state: &mut LuaState) -> LuaResult<u32> {
    _state.push(rilua::Val::Bool(false));
    Ok(1)
}

fn can_create_scenario_group(_state: &mut LuaState) -> LuaResult<u32> {
    _state.push(rilua::Val::Bool(false));
    Ok(1)
}

fn is_premade_group_finder_enabled(_state: &mut LuaState) -> LuaResult<u32> {
    _state.push(rilua::Val::Bool(false));
    Ok(1)
}

fn remove_listing(_state: &mut LuaState) -> LuaResult<u32> {
    Ok(0)
}

/// `GetApplicationInfo(searchResultID)` →
///   `(applicationID, applicationStatus, pendingStatus, applicationDuration, role)`.
///
/// Returns the `"none"` status (not nil) when no application exists for
/// `resultID` — the panel's search-entry render does
/// `isApplication = (appStatus ~= "none" or pendingStatus)`, so a nil
/// status would mark every browsed result as an active application.
fn get_application_info(state: &mut LuaState) -> LuaResult<u32> {
    let search_result_id = Option::<f64>::from_stack(state, 1)?.unwrap_or(0.0) as u32;
    let app_opt = borrow_state(state)?
        .lfg_applications
        .iter()
        .find(|a| a.search_result_id == search_result_id)
        .cloned();
    let Some(app) = app_opt else {
        state.push(Val::Num(0.0));
        let none_str = create_string(state, "none");
        state.push(none_str);
        state.push(Val::Nil);
        state.push(Val::Num(0.0));
        let empty = create_string(state, "");
        state.push(empty);
        return Ok(5);
    };
    state.push(Val::Num(app.application_id as f64));
    let status_val = create_string(state, &app.status);
    state.push(status_val);
    match &app.pending_status {
        Some(s) => {
            let v = create_string(state, s);
            state.push(v);
        }
        None => state.push(Val::Nil),
    }
    state.push(Val::Num(app.duration));
    let role_val = create_string(state, &app.role);
    state.push(role_val);
    Ok(5)
}

/// `ApplyToGroup(searchResultID, tank, healer, damager)` — submit an
/// application. Idempotent: re-applying to a result that already has a
/// pending application is a no-op (matches retail's "you have a pending
/// application" guard, exposed as the gray Sign Up button).
fn apply_to_group(state: &mut LuaState) -> LuaResult<u32> {
    let search_result_id = Option::<f64>::from_stack(state, 1)?.unwrap_or(0.0) as u32;
    let tank = Option::<bool>::from_stack(state, 2)?.unwrap_or(false);
    let healer = Option::<bool>::from_stack(state, 3)?.unwrap_or(false);
    let damager = Option::<bool>::from_stack(state, 4)?.unwrap_or(false);
    let role = selected_application_role(tank, healer, damager);
    let listing_name = apply_to_group_listing_name(state, search_result_id)?;
    let Some(listing_name) = listing_name else {
        return Ok(0);
    };
    create_lfg_application(state, search_result_id, role)?;
    fire_lfg_application_events(state, search_result_id, &listing_name)?;
    Ok(0)
}

fn selected_application_role(tank: bool, healer: bool, damager: bool) -> String {
    if tank {
        "TANK"
    } else if healer {
        "HEALER"
    } else if damager {
        "DAMAGER"
    } else {
        ""
    }
    .to_string()
}

fn apply_to_group_listing_name(
    state: &mut LuaState,
    search_result_id: u32,
) -> LuaResult<Option<String>> {
    let sim = borrow_state(state)?;
    if sim
        .lfg_applications
        .iter()
        .any(|a| a.search_result_id == search_result_id)
    {
        return Ok(None);
    }
    Ok(sim
        .world
        .premade_listings
        .iter()
        .find(|l| l.search_result_id == search_result_id)
        .map(|l| l.name.clone()))
}

fn create_lfg_application(
    state: &mut LuaState,
    search_result_id: u32,
    role: String,
) -> LuaResult<()> {
    let now = borrow_state(state)?.start_time.elapsed().as_secs_f64();
    let mut sim = borrow_state_mut(state)?;
    let app_id = sim.lfg_next_application_id;
    sim.lfg_next_application_id += 1;
    sim.lfg_applications.push(LfgApplication {
        application_id: app_id,
        search_result_id,
        status: "applied".to_string(),
        pending_status: None,
        start_time: now,
        duration: 120.0,
        role,
    });
    Ok(())
}

fn fire_lfg_application_events(
    state: &mut LuaState,
    search_result_id: u32,
    listing_name: &str,
) -> LuaResult<()> {
    let new_status = create_string(state, "applied");
    let old_status = create_string(state, "none");
    let group_name = create_string(state, listing_name);
    fire_event_with_args(
        state,
        "LFG_LIST_APPLICATION_STATUS_UPDATED",
        vec![
            Val::Num(search_result_id as f64),
            new_status,
            old_status,
            group_name,
        ],
    )?;
    fire_event_with_args(
        state,
        "LFG_LIST_SEARCH_RESULT_UPDATED",
        vec![Val::Num(search_result_id as f64)],
    )?;
    Ok(())
}

/// `CancelApplication(searchResultID)` — cancel a pending application.
/// Removes the entry from `lfg_applications` and fires the status-update
/// event so the panel transitions the row back to its non-application
/// rendering branch.
fn cancel_application(state: &mut LuaState) -> LuaResult<u32> {
    let search_result_id = Option::<f64>::from_stack(state, 1)?.unwrap_or(0.0) as u32;
    let listing_name = {
        let mut sim = borrow_state_mut(state)?;
        let pos = sim
            .lfg_applications
            .iter()
            .position(|a| a.search_result_id == search_result_id);
        let Some(idx) = pos else {
            return Ok(0);
        };
        sim.lfg_applications.remove(idx);
        sim.world
            .premade_listings
            .iter()
            .find(|l| l.search_result_id == search_result_id)
            .map(|l| l.name.clone())
            .unwrap_or_default()
    };
    let new_status = create_string(state, "cancelled");
    let old_status = create_string(state, "applied");
    let group_name = create_string(state, &listing_name);
    fire_event_with_args(
        state,
        "LFG_LIST_APPLICATION_STATUS_UPDATED",
        vec![
            Val::Num(search_result_id as f64),
            new_status,
            old_status,
            group_name,
        ],
    )?;
    Ok(0)
}

fn ensure_c_lfg_list_table(state: &mut LuaState) -> GcRef<Table> {
    let key = state.gc.intern_string_static(b"C_LFGList");
    let global = state.global;
    let existing = state
        .gc
        .tables
        .get(global)
        .map(|t| t.get_str(key, &state.gc.string_arena));
    if let Some(Val::Table(r)) = existing {
        return r;
    }
    let new_val = create_table(state);
    let Val::Table(new_ref) = new_val else {
        unreachable!("create_table must return a table");
    };
    if let Some(global_table) = state.gc.tables.get_mut(global) {
        let _ = global_table.raw_set(Val::Str(key), new_val, &state.gc.string_arena);
    }
    state.gc.barrier_back(global);
    new_ref
}

fn register_lfg_list_methods(
    state: &mut LuaState,
    table_ref: GcRef<Table>,
    methods: &[(&'static str, RustFn)],
) -> LuaResult<()> {
    for (name, method) in methods {
        table_set_rust_fn_static(state, table_ref, name, *method)?;
    }
    Ok(())
}

fn register_search_methods(state: &mut LuaState, table_ref: GcRef<Table>) -> LuaResult<()> {
    register_lfg_list_methods(
        state,
        table_ref,
        &[
            ("GetSearchResultInfo", get_search_result_info),
            ("HasSearchResultInfo", has_search_result_info),
            ("GetSearchResults", get_search_results),
            ("GetFilteredSearchResults", get_filtered_search_results),
            (
                "GetSearchResultMemberCounts",
                get_search_result_member_counts,
            ),
            (
                "GetGroupLeaverCountsByRole",
                get_group_leaver_counts_by_role,
            ),
            ("Search", search),
        ],
    )
}

#[cfg(feature = "retail-12-1-0")]
fn confirm_censored_active_entry(_state: &mut LuaState) -> LuaResult<u32> {
    Ok(0)
}

#[cfg(feature = "retail-12-1-0")]
fn does_censored_text_match(state: &mut LuaState) -> LuaResult<u32> {
    state.push(Val::Bool(false));
    Ok(1)
}

#[cfg(feature = "retail-12-1-0")]
fn is_censored_active_entry_unresolved(state: &mut LuaState) -> LuaResult<u32> {
    state.push(Val::Bool(false));
    Ok(1)
}

#[cfg(feature = "retail-12-1-0")]
fn reveal_censored_active_entry(_state: &mut LuaState) -> LuaResult<u32> {
    Ok(0)
}

#[cfg(feature = "retail-12-1-0")]
fn reveal_censored_search_result(_state: &mut LuaState) -> LuaResult<u32> {
    Ok(0)
}

fn register_listing_methods(state: &mut LuaState, table_ref: GcRef<Table>) -> LuaResult<()> {
    register_lfg_list_methods(
        state,
        table_ref,
        &[
            #[cfg(feature = "retail-12-1-0")]
            ("ConfirmCensoredActiveEntry", confirm_censored_active_entry),
            #[cfg(feature = "retail-12-1-0")]
            ("DoesCensoredTextMatch", does_censored_text_match),
            #[cfg(feature = "retail-12-1-0")]
            (
                "IsCensoredActiveEntryUnresolved",
                is_censored_active_entry_unresolved,
            ),
            #[cfg(feature = "retail-12-1-0")]
            ("RevealCensoredActiveEntry", reveal_censored_active_entry),
            #[cfg(feature = "retail-12-1-0")]
            ("RevealCensoredSearchResult", reveal_censored_search_result),
            ("GetNumApplications", counts::get_num_applications),
            ("GetNumApplicants", counts::get_num_applicants),
            ("RemoveListing", remove_listing),
            ("GetApplications", get_applications),
            ("GetApplicationInfo", get_application_info),
            ("ApplyToGroup", apply_to_group),
            ("CancelApplication", cancel_application),
        ],
    )
}

fn register_capability_methods(state: &mut LuaState, table_ref: GcRef<Table>) -> LuaResult<()> {
    register_lfg_list_methods(
        state,
        table_ref,
        &[
            ("GetPremadeGroupFinderStyle", get_premade_group_finder_style),
            ("CanCreateQuestGroup", can_create_quest_group),
            ("CanCreateScenarioGroup", can_create_scenario_group),
            (
                "IsPremadeGroupFinderEnabled",
                is_premade_group_finder_enabled,
            ),
            ("HasActivityList", catalog::has_activity_list),
            ("HasActiveEntryInfo", catalog::has_active_entry_info),
            ("GetActiveEntryInfo", catalog::get_active_entry_info),
            ("GetAvailableRoles", catalog::get_available_roles),
        ],
    )
}

fn register_activity_methods(state: &mut LuaState, table_ref: GcRef<Table>) -> LuaResult<()> {
    register_lfg_list_methods(
        state,
        table_ref,
        &[
            ("GetActivityInfoTable", catalog::get_activity_info_table),
            ("GetAvailableCategories", catalog::get_available_categories),
            ("GetLfgCategoryInfo", catalog::get_lfg_category_info),
            (
                "GetAvailableActivityGroups",
                catalog::get_available_activity_groups,
            ),
            ("GetAvailableActivities", catalog::get_available_activities),
            ("GetActivityGroupInfo", catalog::get_activity_group_info),
            ("GetActivityFullName", catalog::get_activity_full_name),
            ("GetPlaystyleString", catalog::get_playstyle_string),
        ],
    )
}

fn register_filter_methods(state: &mut LuaState, table_ref: GcRef<Table>) -> LuaResult<()> {
    register_lfg_list_methods(
        state,
        table_ref,
        &[
            (
                "GetAvailableLanguageSearchFilter",
                catalog::get_available_language_search_filter,
            ),
            (
                "GetLanguageSearchFilter",
                catalog::get_language_search_filter,
            ),
            (
                "GetDefaultLanguageSearchFilter",
                catalog::get_default_language_search_filter,
            ),
            ("GetAdvancedFilter", catalog::get_advanced_filter),
        ],
    )
}

pub fn register_all(lua: &mut rilua::Lua) -> LuaResult<()> {
    use rilua::LuaApiMut;
    let state = lua.state_mut();
    let table_ref = ensure_c_lfg_list_table(state);
    register_search_methods(state, table_ref)?;
    register_listing_methods(state, table_ref)?;
    register_capability_methods(state, table_ref)?;
    register_activity_methods(state, table_ref)?;
    register_filter_methods(state, table_ref)?;
    Ok(())
}
