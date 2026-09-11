use super::{layout, read_id};
use crate::lua_api::methods::{borrow_state, create_table_with_fields, table_set_num};
use crate::lua_bridge::{stack_val, table_set_rust_fn_static};
use rilua::vm::{gc::arena::GcRef, state::LuaState, table::Table};
use rilua::{LuaResult, Val, runtime_error};

pub(super) fn register(state: &mut LuaState, namespace: GcRef<Table>) -> LuaResult<()> {
    for (name, function) in [
        ("GetTrackList", list as rilua::RustFn),
        ("GetTrackInfo", info),
        ("GetTrackType", kind),
        ("GetTrackMaxEventDuration", maximum),
        ("GetEventTrack", event_track),
        ("HasVisibleEvents", has_visible),
        ("GetEventHighlightTime", highlight_time),
    ] {
        table_set_rust_fn_static(state, namespace, name, function)?;
    }
    Ok(())
}

fn read_track(state: &LuaState) -> LuaResult<u8> {
    match stack_val(state, 1) {
        Val::Num(value) if (0.0..=4.0).contains(&value) && value.fract() == 0.0 => Ok(value as u8),
        _ => Err(runtime_error(
            "timeline track must be an integer from 0 to 4",
        )),
    }
}

fn track_type(track: u8) -> u8 {
    match track {
        0 | 3 => 1,
        1 | 2 => 2,
        _ => 0,
    }
}

fn bounds(track: u8) -> (f64, f64) {
    match track {
        0 => (0.0, 0.0),
        1 => (0.0, 15.0),
        2 => (15.0, 60.0),
        3 => (60.0, f64::INFINITY),
        _ => (f64::INFINITY, f64::INFINITY),
    }
}

fn track_table(state: &mut LuaState, track: u8) -> Val {
    let (minimum, maximum) = bounds(track);
    let sorted = matches!(track, 0 | 3);
    create_table_with_fields(
        state,
        &[
            ("id", Val::Num(f64::from(track))),
            ("type", Val::Num(f64::from(track_type(track)))),
            ("minimumDuration", Val::Num(minimum)),
            ("maximumDuration", Val::Num(maximum)),
            ("minimumEventIntroDuration", Val::Num(0.0)),
            ("minimumEventGapDuration", Val::Num(0.0)),
            (
                "maximumEventCount",
                if sorted {
                    Val::Num(layout::SORTED_CAPACITY as f64)
                } else {
                    Val::Nil
                },
            ),
            (
                "sortDirection",
                if sorted { Val::Num(1.0) } else { Val::Nil },
            ),
        ],
    )
}

fn list(state: &mut LuaState) -> LuaResult<u32> {
    let result = state.gc.alloc_table(Table::with_sizes(5, 0));
    state.push(Val::Table(result));
    for track in 0..5 {
        let info = track_table(state, track);
        table_set_num(state, result, f64::from(track + 1), info);
    }
    Ok(1)
}

fn info(state: &mut LuaState) -> LuaResult<u32> {
    let track = read_track(state)?;
    let result = track_table(state, track);
    state.push(result);
    Ok(1)
}

fn kind(state: &mut LuaState) -> LuaResult<u32> {
    let track = read_track(state)?;
    state.push(Val::Num(f64::from(track_type(track))));
    Ok(1)
}

fn maximum(state: &mut LuaState) -> LuaResult<u32> {
    let track = read_track(state)?;
    state.push(Val::Num(bounds(track).1));
    Ok(1)
}

fn event_track(state: &mut LuaState) -> LuaResult<u32> {
    let id = read_id(state)?;
    let position = borrow_state(state)?
        .encounter_timeline
        .events
        .get(&id)
        .map(|event| (event.track, event.track_index));
    let Some((track, index)) = position else {
        return Ok(0);
    };
    state.push(Val::Num(f64::from(track)));
    state.push(index.map_or(Val::Nil, |index| Val::Num(index as f64)));
    Ok(2)
}

fn has_visible(state: &mut LuaState) -> LuaResult<u32> {
    let visible = borrow_state(state)?
        .encounter_timeline
        .events
        .values()
        .any(layout::visible);
    state.push(Val::Bool(visible));
    Ok(1)
}

fn highlight_time(state: &mut LuaState) -> LuaResult<u32> {
    state.push(Val::Num(layout::HIGHLIGHT_TIME));
    Ok(1)
}
