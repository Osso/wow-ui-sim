use super::{
    model::{EventInfo, EventState},
    read_id, timer,
};
use crate::lua_api::methods::{borrow_state, create_table_with_fields, table_set_num};
use crate::lua_bridge::{stack_val, table_set_rust_fn_static};
use rilua::vm::{gc::arena::GcRef, state::LuaState, table::Table};
use rilua::{LuaResult, Val, runtime_error};

pub(super) fn register(state: &mut LuaState, namespace: GcRef<Table>) -> LuaResult<()> {
    let methods: &[(&str, rilua::RustFn)] = &[
        ("GetCurrentTime", current_time),
        ("GetEventInfo", info),
        ("GetEventState", event_state),
        ("GetEventList", event_list),
        ("GetEventCountBySource", count_by_source),
        ("GetEventTimer", event_timer),
        ("GetEventTimeElapsed", |s| event_time(s, false)),
        ("GetEventTimeRemaining", |s| event_time(s, true)),
        ("HasAnyEvents", |s| has_events(s, None)),
        ("HasActiveEvents", |s| {
            has_events(s, Some(EventState::Active))
        }),
        ("HasPausedEvents", |s| {
            has_events(s, Some(EventState::Paused))
        }),
        ("IsEventBlocked", is_blocked),
        ("IsFeatureAvailable", feature_enabled),
        ("IsFeatureEnabled", feature_enabled),
    ];
    for &(name, function) in methods {
        table_set_rust_fn_static(state, namespace, name, function)?;
    }
    Ok(())
}

pub(super) fn info_table(state: &mut LuaState, info: &EventInfo) -> Val {
    let name = state.gc.intern_string(&info.name);
    create_table_with_fields(
        state,
        &[
            ("id", Val::Num(f64::from(info.id))),
            ("source", Val::Num(1.0)),
            ("spellName", Val::Str(name)),
            ("spellID", Val::Num(f64::from(info.spell_id))),
            ("iconFileID", Val::Num(f64::from(info.icon))),
            ("duration", Val::Num(info.duration)),
            ("maxQueueDuration", Val::Num(info.max_queue_duration)),
            ("icons", Val::Num(f64::from(info.icons))),
            ("severity", Val::Num(f64::from(info.severity))),
            ("isApproximate", Val::Bool(false)),
        ],
    )
}

fn current_time(state: &mut LuaState) -> LuaResult<u32> {
    let now = borrow_state(state)?.encounter_timeline.now.get();
    state.push(Val::Num(now));
    Ok(1)
}

fn info(state: &mut LuaState) -> LuaResult<u32> {
    let id = read_id(state)?;
    let info = borrow_state(state)?
        .encounter_timeline
        .events
        .get(&id)
        .map(|event| event.info.clone());
    let Some(info) = info else { return Ok(0) };
    let value = info_table(state, &info);
    state.push(value);
    Ok(1)
}

fn event_state(state: &mut LuaState) -> LuaResult<u32> {
    let id = read_id(state)?;
    let value = borrow_state(state)?
        .encounter_timeline
        .events
        .get(&id)
        .map(|event| event.state);
    let Some(value) = value else { return Ok(0) };
    state.push(Val::Num(f64::from(value as u8)));
    Ok(1)
}

fn event_time(state: &mut LuaState, remaining: bool) -> LuaResult<u32> {
    let id = read_id(state)?;
    let time = borrow_state(state)?
        .encounter_timeline
        .events
        .get(&id)
        .map(|event| {
            let elapsed = event.clock.borrow().elapsed();
            if remaining {
                event.info.duration - elapsed
            } else {
                elapsed
            }
        });
    let Some(time) = time else { return Ok(0) };
    state.push(Val::Num(time));
    Ok(1)
}

fn event_timer(state: &mut LuaState) -> LuaResult<u32> {
    let id = read_id(state)?;
    let clock = borrow_state(state)?
        .encounter_timeline
        .events
        .get(&id)
        .map(|event| event.clock.clone());
    let Some(clock) = clock else { return Ok(0) };
    let value = timer::create(state, clock)?;
    state.push(value);
    Ok(1)
}

fn event_list(state: &mut LuaState) -> LuaResult<u32> {
    let ids = borrow_state(state)?
        .encounter_timeline
        .events
        .keys()
        .copied()
        .collect::<Vec<_>>();
    let result = state.gc.alloc_table(Table::with_sizes(ids.len(), 0));
    for (index, id) in ids.into_iter().enumerate() {
        table_set_num(state, result, (index + 1) as f64, Val::Num(f64::from(id)));
    }
    state.push(Val::Table(result));
    Ok(1)
}

fn has_events(state: &mut LuaState, selected: Option<EventState>) -> LuaResult<u32> {
    let found = borrow_state(state)?
        .encounter_timeline
        .events
        .values()
        .any(|event| selected.is_none_or(|value| value == event.state));
    state.push(Val::Bool(found));
    Ok(1)
}

fn count_by_source(state: &mut LuaState) -> LuaResult<u32> {
    let count = match stack_val(state, 1) {
        Val::Num(1.0) => borrow_state(state)?.encounter_timeline.events.len(),
        Val::Num(0.0) | Val::Num(2.0) => 0,
        _ => return Err(runtime_error("unknown EncounterTimelineEventSource")),
    };
    state.push(Val::Num(count as f64));
    Ok(1)
}

fn is_blocked(state: &mut LuaState) -> LuaResult<u32> {
    let id = read_id(state)?;
    if !borrow_state(state)?
        .encounter_timeline
        .events
        .contains_key(&id)
    {
        return Ok(0);
    }
    state.push(Val::Bool(false));
    Ok(1)
}

fn feature_enabled(state: &mut LuaState) -> LuaResult<u32> {
    state.push(Val::Bool(true));
    Ok(1)
}
