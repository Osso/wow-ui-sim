//! Ordinary duration state; formulas are simulator assumptions (see duration-core spec).
use super::{install_method, table_get, table_set};
use crate::lua_api::methods::borrow_state;
use crate::lua_bridge::{FromStack, stack_val};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

// Negative slots leave the existing identity at slot zero and custom fields intact.
const START: i64 = -1;
const BASE_DURATION: i64 = -2;
const RATE: i64 = -3;

#[derive(Clone, Copy)]
struct Timing {
    start: f64,
    base: f64,
    rate: f64,
}

impl Timing {
    fn span(self) -> f64 {
        self.base / self.rate
    }

    fn end(self) -> f64 {
        self.start + self.span()
    }
}

fn read_number(state: &LuaState, object: Val, slot: i64) -> f64 {
    let Val::Table(reference) = object else {
        return 0.0;
    };
    match state
        .gc
        .tables
        .get(reference)
        .map(|table| table.get_int(slot))
    {
        Some(Val::Num(value)) => value,
        _ => 0.0,
    }
}

fn read_timing(state: &LuaState, object: Val) -> Timing {
    Timing {
        start: read_number(state, object, START),
        base: read_number(state, object, BASE_DURATION),
        rate: read_number(state, object, RATE),
    }
}

fn write_timing(state: &mut LuaState, object: Val, timing: Timing) {
    if let Val::Table(reference) = object {
        if let Some(table) = state.gc.tables.get_mut(reference) {
            for (slot, value) in [
                (START, timing.start),
                (BASE_DURATION, timing.base),
                (RATE, timing.rate),
            ] {
                let _ = table.raw_set(
                    Val::Num(slot as f64),
                    Val::Num(value),
                    &state.gc.string_arena,
                );
            }
        }
        state.gc.barrier_back(reference);
    }
}

pub(super) fn initialize(state: &mut LuaState, object: Val) {
    write_timing(
        state,
        object,
        Timing {
            start: 0.0,
            base: 0.0,
            rate: 1.0,
        },
    );
}

pub(super) fn copy_state(state: &mut LuaState, source: Val, target: Val) {
    let timing = read_timing(state, source);
    let clock = table_get(state, source, "clock");
    write_timing(state, target, timing);
    table_set(state, target, "clock", clock);
}

pub(super) fn current_time(state: &LuaState) -> LuaResult<f64> {
    Ok(borrow_state(state)?.start_time.elapsed().as_secs_f64())
}

fn clock_time(state: &mut LuaState, object: Val) -> LuaResult<f64> {
    match table_get(state, object, "clock") {
        Val::Nil => current_time(state),
        clock => match read_clock_time(state, clock)? {
            Val::Num(time) if time.is_finite() => Ok(time),
            _ => Err(rilua::runtime_error(
                "duration clock must contain finite time",
            )),
        },
    }
}

fn read_clock_time(state: &mut LuaState, clock: Val) -> LuaResult<Val> {
    let key = state.gc.intern_string(b"time");
    state.gettable(clock, Val::Str(key))
}

fn finite_arg(state: &mut LuaState, index: i32) -> LuaResult<f64> {
    let value = f64::from_stack(state, index)?;
    if !value.is_finite() {
        return Err(rilua::runtime_error("duration argument must be finite"));
    }
    Ok(value)
}

fn validate(timing: Timing) -> LuaResult<Timing> {
    let duration_is_valid = timing.base.is_finite() && timing.base >= 0.0;
    let rate_is_valid = timing.rate.is_finite() && timing.rate > 0.0;
    if !duration_is_valid || !rate_is_valid {
        return Err(rilua::runtime_error(
            "duration must be nonnegative and rate must be positive and finite",
        ));
    }
    let endpoints_are_finite =
        timing.start.is_finite() && timing.span().is_finite() && timing.end().is_finite();
    if !endpoints_are_finite {
        return Err(rilua::runtime_error("duration endpoints must be finite"));
    }
    Ok(timing)
}

fn set_time(state: &mut LuaState, from_end: bool) -> LuaResult<u32> {
    let endpoint = finite_arg(state, 2)?;
    let base = finite_arg(state, 3)?;
    let rate = Option::<f64>::from_stack(state, 4)?.unwrap_or(1.0);
    let start = if from_end {
        endpoint - base / rate
    } else {
        endpoint
    };
    let timing = validate(Timing { start, base, rate })?;
    write_timing(state, stack_val(state, 1), timing);
    Ok(0)
}

fn set_span(state: &mut LuaState) -> LuaResult<u32> {
    let start = finite_arg(state, 2)?;
    let end = finite_arg(state, 3)?;
    let timing = validate(Timing {
        start,
        base: end - start,
        rate: 1.0,
    })?;
    write_timing(state, stack_val(state, 1), timing);
    Ok(0)
}

fn reset(state: &mut LuaState, defaults: bool) -> LuaResult<u32> {
    let object = stack_val(state, 1);
    initialize(state, object);
    if defaults {
        table_set(state, object, "clock", Val::Nil);
    }
    Ok(0)
}

#[derive(Clone, Copy)]
enum Query {
    Start,
    End,
    Total,
    Elapsed,
    Remaining,
    ElapsedPercent,
    RemainingPercent,
    Rate,
    Clock,
    Zero,
    Started,
    Expired,
    Active,
}

fn time_scale(state: &mut LuaState, timing: Timing) -> LuaResult<f64> {
    match Option::<i32>::from_stack(state, 2)?.unwrap_or(0) {
        0 => Ok(1.0),
        1 => Ok(timing.rate),
        _ => Err(rilua::runtime_error("unknown DurationTimeModifier")),
    }
}

fn duration_value(
    state: &mut LuaState,
    object: Val,
    timing: Timing,
    query: Query,
) -> LuaResult<f64> {
    let scale = time_scale(state, timing)?;
    let span = timing.span();
    if matches!(query, Query::Total) {
        return Ok(span * scale);
    }
    let elapsed = (clock_time(state, object)? - timing.start).clamp(0.0, span);
    let duration = if matches!(query, Query::Elapsed | Query::ElapsedPercent) {
        elapsed
    } else {
        span - elapsed
    };
    if matches!(query, Query::ElapsedPercent | Query::RemainingPercent) {
        return Ok(if span > 0.0 { duration / span } else { 0.0 });
    }
    Ok(duration * scale)
}

fn query(state: &mut LuaState, kind: Query) -> LuaResult<u32> {
    let object = stack_val(state, 1);
    let timing = read_timing(state, object);
    let value = match kind {
        Query::Start => Val::Num(timing.start),
        Query::End => Val::Num(timing.end()),
        Query::Rate => Val::Num(timing.rate),
        Query::Clock => Val::Num(clock_time(state, object)?),
        Query::Zero => Val::Bool(timing.base == 0.0),
        Query::Started => {
            Val::Bool(timing.base > 0.0 && clock_time(state, object)? >= timing.start)
        }
        Query::Expired => {
            Val::Bool(timing.base > 0.0 && clock_time(state, object)? >= timing.end())
        }
        Query::Active => {
            let now = clock_time(state, object)?;
            Val::Bool(timing.base > 0.0 && now >= timing.start && now < timing.end())
        }
        _ => Val::Num(duration_value(state, object, timing, kind)?),
    };
    state.push(value);
    Ok(1)
}

pub(super) fn register(state: &mut LuaState, methods: Val) {
    let methods_to_install: &[(&str, rilua::RustFn)] = &[
        ("SetTimeFromStart", |s| set_time(s, false)),
        ("SetTimeFromEnd", |s| set_time(s, true)),
        ("SetTimeSpan", set_span),
        ("Reset", |s| reset(s, false)),
        ("SetToDefaults", |s| reset(s, true)),
        ("GetStartTime", |s| query(s, Query::Start)),
        ("GetEndTime", |s| query(s, Query::End)),
        ("GetTotalDuration", |s| query(s, Query::Total)),
        ("GetElapsedDuration", |s| query(s, Query::Elapsed)),
        ("GetRemainingDuration", |s| query(s, Query::Remaining)),
        ("GetElapsedPercent", |s| query(s, Query::ElapsedPercent)),
        ("GetRemainingPercent", |s| query(s, Query::RemainingPercent)),
        ("GetModRate", |s| query(s, Query::Rate)),
        ("GetClockTime", |s| query(s, Query::Clock)),
        ("IsZero", |s| query(s, Query::Zero)),
        ("HasStarted", |s| query(s, Query::Started)),
        ("HasExpired", |s| query(s, Query::Expired)),
        ("IsActive", |s| query(s, Query::Active)),
    ];
    for &(name, function) in methods_to_install {
        install_method(state, methods, name, name, function);
    }
}
