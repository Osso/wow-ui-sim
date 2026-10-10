use std::borrow::Cow;
use std::sync::OnceLock;
use std::time::Duration;

use rilua::vm::closure::Closure;
use rilua::vm::gc::arena::GcRef;
use rilua::vm::state::LuaState;

const BUILTIN_ADDON_NAME: &str = "__BuiltIn";
const HANDLER_TIMING_ENV: &str = "WOW_SIM_LOG_HANDLER_TIMINGS";

pub(crate) fn is_enabled() -> bool {
    min_duration_ms().is_some()
}

pub(crate) fn should_log(duration: Duration) -> bool {
    let Some(min_duration_ms) = min_duration_ms() else {
        return false;
    };
    duration.as_secs_f64() * 1000.0 >= min_duration_ms
}

pub(crate) fn log_with_source(
    addon_name: Option<&str>,
    handler_name: &str,
    frame_name: Option<&str>,
    widget_id: u64,
    duration: Duration,
    source: Option<&str>,
) {
    if !should_log(duration) {
        return;
    }
    eprintln!(
        "{}",
        format_log(
            addon_name,
            handler_name,
            frame_name,
            widget_id,
            duration,
            source,
        )
    );
}

pub(crate) fn lua_closure_source_label(
    state: &LuaState,
    func_ref: GcRef<Closure>,
) -> Option<String> {
    let closure = state.gc.closures.get(func_ref)?;
    let lua_closure = closure.as_lua()?;
    let proto = lua_closure.proto.as_ref();
    if proto.source.is_empty() {
        return None;
    }

    Some(format!("{}:{}", proto.source, proto.line_defined))
}

fn min_duration_ms() -> Option<f64> {
    static MIN_DURATION_MS: OnceLock<Option<f64>> = OnceLock::new();
    *MIN_DURATION_MS
        .get_or_init(|| parse_min_duration_ms(std::env::var(HANDLER_TIMING_ENV).ok().as_deref()))
}

fn parse_min_duration_ms(raw: Option<&str>) -> Option<f64> {
    let raw = raw?;
    let value = raw.trim();
    if value.is_empty() {
        return Some(0.0);
    }
    Some(value.parse::<f64>().ok().unwrap_or(0.0).max(0.0))
}

fn format_log(
    addon_name: Option<&str>,
    handler_name: &str,
    frame_name: Option<&str>,
    widget_id: u64,
    duration: Duration,
    source: Option<&str>,
) -> String {
    let addon = addon_name.unwrap_or(BUILTIN_ADDON_NAME);
    let frame = frame_label(frame_name, widget_id);
    let mut line = format!(
        "[handler] addon={addon} handler={handler_name} frame={frame} duration_ms={:.3}",
        duration.as_secs_f64() * 1000.0
    );
    if let Some(source) = source.filter(|source| !source.is_empty()) {
        line.push_str(" source=");
        line.push_str(source);
    }
    line
}

fn frame_label<'a>(frame_name: Option<&'a str>, widget_id: u64) -> Cow<'a, str> {
    match frame_name {
        Some(name) if !name.is_empty() => Cow::Borrowed(name),
        _ => Cow::Owned(format!("#{widget_id}")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_min_duration_ms_disables_logging_when_unset() {
        assert_eq!(parse_min_duration_ms(None), None);
    }

    #[test]
    fn parse_min_duration_ms_reads_threshold() {
        assert_eq!(parse_min_duration_ms(Some("10")), Some(10.0));
        assert_eq!(parse_min_duration_ms(Some(" 2.5 ")), Some(2.5));
    }

    #[test]
    fn parse_min_duration_ms_falls_back_to_zero_for_invalid_values() {
        assert_eq!(parse_min_duration_ms(Some("")), Some(0.0));
        assert_eq!(parse_min_duration_ms(Some("abc")), Some(0.0));
        assert_eq!(parse_min_duration_ms(Some("-3")), Some(0.0));
    }

    #[test]
    fn duration_filter_is_inclusive() {
        assert!(!should_log_with_threshold(
            Duration::from_micros(9_999),
            Some(10.0)
        ));
        assert!(should_log_with_threshold(
            Duration::from_millis(10),
            Some(10.0)
        ));
        assert!(should_log_with_threshold(
            Duration::from_millis(11),
            Some(10.0)
        ));
    }

    #[test]
    fn format_log_uses_builtin_fallbacks() {
        let line = format_log(
            None,
            "OnUpdate",
            None,
            42,
            Duration::from_micros(1250),
            None,
        );
        assert_eq!(
            line,
            "[handler] addon=__BuiltIn handler=OnUpdate frame=#42 duration_ms=1.250"
        );
    }

    #[test]
    fn format_log_preserves_named_frame_and_addon() {
        let line = format_log(
            Some("Blizzard_UIParent"),
            "OnShow",
            Some("GameMenuFrame"),
            7,
            Duration::from_micros(500),
            None,
        );
        assert_eq!(
            line,
            "[handler] addon=Blizzard_UIParent handler=OnShow frame=GameMenuFrame duration_ms=0.500"
        );
    }

    #[test]
    fn format_log_includes_source_when_known() {
        let line = format_log(
            Some("MacroToolkit"),
            "OnUpdate",
            None,
            56392,
            Duration::from_millis(51),
            Some("@Interface/AddOns/MacroToolkit/modules/showtooltipMock.lua:42"),
        );
        assert_eq!(
            line,
            "[handler] addon=MacroToolkit handler=OnUpdate frame=#56392 duration_ms=51.000 source=@Interface/AddOns/MacroToolkit/modules/showtooltipMock.lua:42"
        );
    }

    fn should_log_with_threshold(duration: Duration, threshold_ms: Option<f64>) -> bool {
        let Some(threshold_ms) = threshold_ms else {
            return false;
        };
        duration.as_secs_f64() * 1000.0 >= threshold_ms
    }
}
