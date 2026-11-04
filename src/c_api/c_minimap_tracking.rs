//! Minimap tracking defaults and Forever's static tracking filters.

use super::helpers::{ensure_namespace, global_val};
use crate::client_profile::{ACTIVE, ClientProfile};
use crate::lua_api::methods::{
    borrow_state, borrow_state_mut, create_string, create_string_static, create_table_with_fields,
    val_to_string,
};
use crate::lua_api::script_helpers::fire_named_event_state;
use crate::lua_api::workarounds::RETAIL_TRACKING_DEFAULT_MASK;
use crate::lua_bridge::{stack_val, table_set_rust_fn_static};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val, runtime_error};

const FOREVER_FILTER_MASK: i32 = 0x1ff_ffff;
const FOREVER_DEFAULT_MASK: u32 = 2_162_728;

struct TrackingRow {
    key: &'static str,
    filter: u32,
    texture: u32,
    subtype: i32,
    classes: u32,
}
const ROWS: &[TrackingRow] = &[
    TrackingRow {
        key: "AUCTIONEER",
        filter: 1,
        texture: 136452,
        subtype: 2,
        classes: 0,
    },
    TrackingRow {
        key: "BANKER",
        filter: 2,
        texture: 136453,
        subtype: 2,
        classes: 0,
    },
    TrackingRow {
        key: "BANKER",
        filter: 4194304,
        texture: 136453,
        subtype: 2,
        classes: 0,
    },
    TrackingRow {
        key: "BARBER",
        filter: 262144,
        texture: 3852099,
        subtype: 2,
        classes: 0,
    },
    TrackingRow {
        key: "BATTLEMASTER",
        filter: 4,
        texture: 136454,
        subtype: 2,
        classes: 0,
    },
    TrackingRow {
        key: "FLIGHTMASTER",
        filter: 8,
        texture: 136456,
        subtype: -1,
        classes: 0,
    },
    TrackingRow {
        key: "VENDOR_FOOD",
        filter: 16,
        texture: 136457,
        subtype: 2,
        classes: 0,
    },
    TrackingRow {
        key: "VENDOR_POISON",
        filter: 1048576,
        texture: 136462,
        subtype: 2,
        classes: 16,
    },
    TrackingRow {
        key: "VENDOR_AMMO",
        filter: 16777216,
        texture: 136451,
        subtype: 2,
        classes: 26,
    },
    TrackingRow {
        key: "INNKEEPER",
        filter: 32,
        texture: 136458,
        subtype: 2,
        classes: 0,
    },
    TrackingRow {
        key: "ITEM_UPGRADE_MASTER",
        filter: 524288,
        texture: 4025144,
        subtype: 2,
        classes: 0,
    },
    TrackingRow {
        key: "MAILBOX",
        filter: 64,
        texture: 136459,
        subtype: 2,
        classes: 0,
    },
    TrackingRow {
        key: "TRAINER_CLASS",
        filter: 8388608,
        texture: 136455,
        subtype: 2,
        classes: 0,
    },
    TrackingRow {
        key: "TRAINER_PROFESSION",
        filter: 128,
        texture: 136463,
        subtype: 2,
        classes: 0,
    },
    TrackingRow {
        key: "VENDOR_REAGENT",
        filter: 256,
        texture: 136464,
        subtype: 2,
        classes: 0,
    },
    TrackingRow {
        key: "REPAIR",
        filter: 512,
        texture: 136465,
        subtype: 2,
        classes: 0,
    },
    TrackingRow {
        key: "STABLEMASTER",
        filter: 2048,
        texture: 136466,
        subtype: 2,
        classes: 0,
    },
    TrackingRow {
        key: "TRANSMOGRIFIER",
        filter: 4096,
        texture: 1598183,
        subtype: 2,
        classes: 0,
    },
    TrackingRow {
        key: "TRIVIAL_QUESTS",
        filter: 1024,
        texture: 237607,
        subtype: -1,
        classes: 0,
    },
    TrackingRow {
        key: "ACCOUNT_COMPLETED_QUESTS",
        filter: 2097152,
        texture: 4675649,
        subtype: -1,
        classes: 0,
    },
    TrackingRow {
        key: "POI",
        filter: 8192,
        texture: 457292,
        subtype: -1,
        classes: 0,
    },
    TrackingRow {
        key: "TARGET",
        filter: 16384,
        texture: 524052,
        subtype: -1,
        classes: 0,
    },
    TrackingRow {
        key: "FOCUS",
        filter: 32768,
        texture: 524051,
        subtype: -1,
        classes: 0,
    },
    TrackingRow {
        key: "QUEST_POIS",
        filter: 65536,
        texture: 535616,
        subtype: -1,
        classes: 0,
    },
    TrackingRow {
        key: "DIGSITES",
        filter: 131072,
        texture: 535615,
        subtype: -1,
        classes: 0,
    },
];

pub(super) fn register(state: &mut LuaState) -> LuaResult<()> {
    if !matches!(ACTIVE, ClientProfile::Retail | ClientProfile::WowForever) {
        return Ok(());
    }
    let namespace = ensure_namespace(state, "C_Minimap")?;
    // Forever MinimapDocumentation.lua:28, 61, 88, 103, 136, 234.
    table_set_rust_fn_static(state, namespace, "GetDefaultTrackingValue", query_default)?;
    if ACTIVE == ClientProfile::WowForever {
        table_set_rust_fn_static(state, namespace, "IsFilteredOut", is_filtered_out)?;
        table_set_rust_fn_static(state, namespace, "GetNumTrackingTypes", |state| {
            let count = eligible_rows(state)?.count();
            state.push(Val::Num(count as f64));
            Ok(1)
        })?;
        table_set_rust_fn_static(state, namespace, "GetTrackingFilter", get_tracking_filter)?;
        table_set_rust_fn_static(state, namespace, "GetTrackingInfo", get_tracking_info)?;
        table_set_rust_fn_static(state, namespace, "SetTracking", set_tracking)?;
        table_set_rust_fn_static(state, namespace, "ClearAllTracking", clear_tracking)?;
    }
    Ok(())
}

fn eligible_rows(state: &mut LuaState) -> LuaResult<impl Iterator<Item = &'static TrackingRow>> {
    let sim = borrow_state(state)?;
    let class_mask = if sim.player.present {
        u32::try_from(sim.player.class_index - 1)
            .ok()
            .and_then(|shift| 1u32.checked_shl(shift))
            .unwrap_or(0)
    } else {
        0
    };
    Ok(ROWS
        .iter()
        .filter(move |row| row.classes == 0 || row.classes & class_mask != 0))
}

fn tracking_row(state: &mut LuaState) -> LuaResult<Option<&'static TrackingRow>> {
    let value = rilua::table_security::unwrap_secret(state, stack_val(state, 1))?;
    let Val::Num(index) = value else {
        return Ok(None);
    };
    if index < 1.0 || index.fract() != 0.0 {
        return Ok(None);
    }
    Ok(eligible_rows(state)?.nth(index as usize - 1))
}

fn tracking_mask(state: &mut LuaState) -> LuaResult<u32> {
    let eligible = eligible_rows(state)?.fold(0, |mask, row| mask | row.filter);
    let mut sim = borrow_state_mut(state)?;
    let initial = sim
        .cvars
        .get("minimapTrackedInfov4")
        .and_then(|v| v.parse::<u32>().ok())
        .unwrap_or(FOREVER_DEFAULT_MASK)
        & eligible;
    Ok(*sim.minimap_tracking_mask.get_or_insert(initial))
}

fn query_default(state: &mut LuaState) -> LuaResult<u32> {
    let value = rilua::table_security::unwrap_secret(state, stack_val(state, 1))?;
    let number = match value {
        Val::Nil => return Err(runtime_error("GetDefaultTrackingValue requires filterType")),
        Val::Num(number) => number,
        Val::Str(_) => val_to_string(state, value)
            .and_then(|s| s.trim().parse::<f64>().ok())
            .unwrap_or(0.0),
        _ => 0.0,
    };
    if !number.is_finite() || number < f64::from(i32::MIN) || number > f64::from(i32::MAX) {
        return Err(runtime_error(
            "GetDefaultTrackingValue filterType integer overflow",
        ));
    }
    let filter = number as i32;
    let (max_filter, default_mask) = if ACTIVE == ClientProfile::WowForever {
        (FOREVER_FILTER_MASK, FOREVER_DEFAULT_MASK)
    } else {
        (0x7fffff, RETAIL_TRACKING_DEFAULT_MASK)
    };
    if !(0..=max_filter).contains(&filter) {
        return Err(runtime_error(
            "GetDefaultTrackingValue filterType out of range",
        ));
    }
    let filter = filter as u32;
    state.push(Val::Bool(default_mask & filter == filter));
    Ok(1)
}

fn is_filtered_out(state: &mut LuaState) -> LuaResult<u32> {
    let value = rilua::table_security::unwrap_secret(state, stack_val(state, 1))?;
    let Val::Num(filter) = value else {
        return Err(runtime_error("IsFilteredOut requires a numeric filterType"));
    };
    if !(0.0..=f64::from(FOREVER_FILTER_MASK)).contains(&filter) || filter.fract() != 0.0 {
        return Err(runtime_error("IsFilteredOut filterType out of range"));
    }
    let filter = filter as u32;
    let mask = tracking_mask(state)?;
    state.push(Val::Bool(filter != 0 && filter & mask == 0));
    Ok(1)
}

fn get_tracking_filter(state: &mut LuaState) -> LuaResult<u32> {
    let row = tracking_row(state)?;
    let table = if let Some(row) = row {
        create_table_with_fields(state, &[("filterID", Val::Num(f64::from(row.filter)))])
    } else {
        create_table_with_fields(state, &[])
    };
    state.push(table);
    Ok(1)
}

fn get_tracking_info(state: &mut LuaState) -> LuaResult<u32> {
    let Some(row) = tracking_row(state)? else {
        state.push(Val::Nil);
        return Ok(1);
    };
    let active = tracking_mask(state)? & row.filter != 0;
    let key = format!("MINIMAP_TRACKING_{}", row.key);
    let name = match global_val(state, &key) {
        name @ Val::Str(_) => name,
        _ => create_string(state, &key),
    };
    let tracking_type = create_string_static(state, "other");
    let info = create_table_with_fields(
        state,
        &[
            ("name", name),
            ("texture", Val::Num(f64::from(row.texture))),
            ("active", Val::Bool(active)),
            ("type", tracking_type),
            ("subType", Val::Num(f64::from(row.subtype))),
        ],
    );
    state.push(info);
    Ok(1)
}

fn set_tracking(state: &mut LuaState) -> LuaResult<u32> {
    let Some(row) = tracking_row(state)? else {
        return Ok(0);
    };
    let value = rilua::table_security::unwrap_secret(state, stack_val(state, 2))?;
    let Val::Bool(enabled) = value else {
        return Err(runtime_error("SetTracking requires a boolean"));
    };
    if !borrow_state(state)?.player.present {
        return Ok(0);
    }
    let mask = tracking_mask(state)?;
    let new_mask = if enabled {
        mask | row.filter
    } else {
        mask & !row.filter
    };
    if mask != new_mask {
        borrow_state_mut(state)?.minimap_tracking_mask = Some(new_mask);
        fire_named_event_state(state, "MINIMAP_UPDATE_TRACKING", &[]);
    }
    Ok(0)
}

fn clear_tracking(state: &mut LuaState) -> LuaResult<u32> {
    let mask = tracking_mask(state)?;
    let new_mask = mask & !(FOREVER_FILTER_MASK as u32);
    if mask != new_mask {
        borrow_state_mut(state)?.minimap_tracking_mask = Some(new_mask);
        fire_named_event_state(state, "MINIMAP_UPDATE_TRACKING", &[]);
    }
    Ok(0)
}
