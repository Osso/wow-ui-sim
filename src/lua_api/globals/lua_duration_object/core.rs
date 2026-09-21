//! Duration timing and bounded secret-access policy; native-unverified choices are in the spec.
use super::{install_method, table_get, table_set};
use crate::lua_api::methods::borrow_state;
use crate::lua_bridge::{FromStack, stack_val};
use rilua::table_security::{is_secret_value, unwrap_secret, wrap_secret};
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

fn read_slot(state: &LuaState, object: Val, slot: i64) -> Val {
    let Val::Table(reference) = object else {
        return Val::Nil;
    };
    state
        .gc
        .tables
        .get(reference)
        .map(|table| table.get_int(slot))
        .unwrap_or(Val::Nil)
}

fn read_slots(state: &LuaState, object: Val) -> [Val; 3] {
    [START, BASE_DURATION, RATE].map(|slot| read_slot(state, object, slot))
}

pub(super) fn has_secret_values(state: &LuaState, object: Val) -> bool {
    read_slots(state, object)
        .into_iter()
        .any(|value| is_secret_value(state, value))
}

pub(super) fn require_secret_access(state: &LuaState, object: Val) -> LuaResult<()> {
    for value in read_slots(state, object) {
        unwrap_secret(state, value)?;
    }
    Ok(())
}

fn read_number(state: &LuaState, object: Val, slot: i64) -> LuaResult<f64> {
    match unwrap_secret(state, read_slot(state, object, slot))? {
        Val::Num(value) => Ok(value),
        _ => Ok(0.0),
    }
}

fn read_timing(state: &LuaState, object: Val) -> LuaResult<Timing> {
    Ok(Timing {
        start: read_number(state, object, START)?,
        base: read_number(state, object, BASE_DURATION)?,
        rate: read_number(state, object, RATE)?,
    })
}

fn write_slots(state: &mut LuaState, object: Val, values: [Val; 3]) -> LuaResult<()> {
    let Val::Table(reference) = object else {
        return Err(rilua::runtime_error("invalid duration object"));
    };
    let table = state
        .gc
        .tables
        .get_mut(reference)
        .ok_or_else(|| rilua::runtime_error("duration object has been collected"))?;
    for (slot, value) in [START, BASE_DURATION, RATE].into_iter().zip(values) {
        table.raw_set(Val::Num(slot as f64), value, &state.gc.string_arena)?;
    }
    state.gc.barrier_back(reference);
    Ok(())
}

fn write_protected_slots(
    state: &mut LuaState,
    object: Val,
    mut values: [Val; 3],
    secret: bool,
) -> LuaResult<()> {
    let previous_top = state.top;
    if secret {
        for value in &mut values {
            *value = wrap_secret(state, *value)?;
            // Root each allocation before wrapping the next timing component.
            state.push(*value);
        }
    }
    let result = write_slots(state, object, values);
    state.top = previous_top;
    result
}

fn write_timing(state: &mut LuaState, object: Val, timing: Timing, secret: bool) -> LuaResult<()> {
    let values = [timing.start, timing.base, timing.rate].map(Val::Num);
    write_protected_slots(state, object, values, secret)
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
        false,
    )
    .expect("new duration table accepts its numeric timing slots");
}

pub(super) fn copy_state(state: &mut LuaState, source: Val, target: Val) -> LuaResult<()> {
    let values = read_slots(state, source);
    let clock = table_get(state, source, "clock");
    write_slots(state, target, values)?;
    table_set(state, target, "clock", clock);
    Ok(())
}

pub(super) fn assign_state(state: &mut LuaState, source: Val, target: Val) -> LuaResult<()> {
    require_secret_access(state, source)?;
    require_secret_access(state, target)?;
    let secret = has_secret_values(state, source) || has_secret_values(state, target);
    let values = read_slots(state, source);
    let clock = table_get(state, source, "clock");
    write_protected_slots(state, target, values, secret)?;
    table_set(state, target, "clock", clock);
    Ok(())
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

fn finite_arg(state: &LuaState, index: i32, default: Option<f64>) -> LuaResult<f64> {
    let value = unwrap_secret(state, stack_val(state, index))?;
    let number = match (value, default) {
        (Val::Num(number), _) => number,
        (Val::Nil, Some(default)) => default,
        _ => {
            return Err(rilua::runtime_error(format!(
                "expected number, got {} at argument {index}",
                value.type_name()
            )));
        }
    };
    if !number.is_finite() {
        return Err(rilua::runtime_error("duration argument must be finite"));
    }
    Ok(number)
}

fn configuration_is_secret(state: &LuaState, object: Val, last_argument: i32) -> bool {
    has_secret_values(state, object)
        || (2..=last_argument).any(|index| is_secret_value(state, stack_val(state, index)))
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
    let object = stack_val(state, 1);
    require_secret_access(state, object)?;
    let endpoint = finite_arg(state, 2, None)?;
    let base = finite_arg(state, 3, None)?;
    let rate = finite_arg(state, 4, Some(1.0))?;
    let start = if from_end {
        endpoint - base / rate
    } else {
        endpoint
    };
    let timing = validate(Timing { start, base, rate })?;
    let secret = configuration_is_secret(state, object, 4);
    write_timing(state, object, timing, secret)?;
    Ok(0)
}

fn set_span(state: &mut LuaState) -> LuaResult<u32> {
    let object = stack_val(state, 1);
    require_secret_access(state, object)?;
    let start = finite_arg(state, 2, None)?;
    let end = finite_arg(state, 3, None)?;
    let timing = validate(Timing {
        start,
        base: end - start,
        rate: 1.0,
    })?;
    let secret = configuration_is_secret(state, object, 3);
    write_timing(state, object, timing, secret)?;
    Ok(0)
}

fn reset(state: &mut LuaState, defaults: bool) -> LuaResult<u32> {
    let object = stack_val(state, 1);
    require_secret_access(state, object)?;
    let secret = !defaults && has_secret_values(state, object);
    write_timing(
        state,
        object,
        Timing {
            start: 0.0,
            base: 0.0,
            rate: 1.0,
        },
        secret,
    )?;
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
    let timing = read_timing(state, object)?;
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

fn evaluate_with_curve(state: &mut LuaState, getter: &str) -> LuaResult<u32> {
    use crate::lua_api::methods::{call_function_state, create_string};
    let object = super::require_duration(state, 1)?;
    let curve = stack_val(state, 2);
    let modifier = stack_val(state, 3);
    let key = create_string(state, getter);
    let method = state.gettable(object, key)?;
    let value = call_function_state(state, method, &[object, modifier])?;
    let Val::Num(value) = value else {
        return Err(rilua::runtime_error("duration query must return a number"));
    };
    let result = crate::c_api::c_curve_util::evaluate_curve_value(state, curve, value)?;
    state.push(result);
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
        ("EvaluateElapsedDuration", |s| {
            evaluate_with_curve(s, "GetElapsedDuration")
        }),
        ("EvaluateRemainingDuration", |s| {
            evaluate_with_curve(s, "GetRemainingDuration")
        }),
        ("EvaluateElapsedPercent", |s| {
            evaluate_with_curve(s, "GetElapsedPercent")
        }),
        ("EvaluateRemainingPercent", |s| {
            evaluate_with_curve(s, "GetRemainingPercent")
        }),
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
