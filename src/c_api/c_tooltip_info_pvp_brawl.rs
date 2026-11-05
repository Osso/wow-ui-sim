//! WM107: `GetPvpBrawl` composes the active or special-event brawl tooltip.
//!
//! Retail/PTR/Forever `TooltipInfoDocumentation.lua:710` declare the getter.
//! Publication on PTR and Forever is **INFERRED** shared-model reuse.
//!
//! Modelled from WM107:
//! - tooltip type 16 (`PvPBrawl`) with no identity id;
//! - rows: name (plain, highlight), ends-in only for the active record
//!   (serialized type 42 because the colour equals `DISABLED_FONT_COLOR`),
//!   blank + map heading + map rows when the map count is nonzero,
//!   blank + rules heading + long description when the description is nonempty;
//! - the name is the record's raw localized name, never a format string;
//! - the all-arena arm emits one `PVP_BRAWL_TOOLTIP_ALL_ARENA_MAPS` row instead
//!   of per-map `BRAWL_TOOLTIP_MAP_ENTRY` rows, and uses the plural heading;
//! - the special-event record never emits ends-in and forces queue permission
//!   true; the active record rejects a false server permission before any row;
//! - no queue row, short-description row or separator is emitted.
//!
//! EMULATED: WM107 leaves the exact localized ends-in text and the loaded
//! GlobalColor values UNKNOWN. The ends-in text reuses the simulator's existing
//! localized duration formatter, and every colour uses the named colour seam
//! with the established simulator fallbacks (`HIGHLIGHT_FONT_COLOR` white,
//! `NORMAL_FONT_COLOR` gold, and the measured 0.5019608 gray for the trivial /
//! disabled colour).

use crate::c_api::c_pvp::PvpBrawlInfo;
use crate::c_api::c_tooltip_info_totem::format_remaining_time;
use crate::c_api::tooltip_getter_args::optional_bool;
use crate::lua_api::globals::missing_surface::tooltip_info::{
    append_line, compose_tooltip, format_global_text, global_text,
};
use crate::lua_api::methods::borrow_state;
use crate::lua_bridge::table_set_rust_fn_static;
use rilua::LuaResult;
use rilua::Val;
use rilua::vm::{gc::arena::GcRef, state::LuaState, table::Table};

/// `Enum.TooltipDataType.PvPBrawl` (TooltipInfoSharedDocumentation).
const TOOLTIP_TYPE_PVP_BRAWL: f64 = 16.0;
const LINE_TYPE_PLAIN: f64 = 0.0;
const LINE_TYPE_BLANK: f64 = 1.0;
/// WM107: the shared line constructor re-types the ends-in row by colour value.
const LINE_TYPE_DISABLED: f64 = 42.0;
const BLANK_TEXT: &str = " ";
/// WM107: `TRIVIAL_DIFFICULTY_COLOR` equals `DISABLED_FONT_COLOR`.
const DISABLED_COLOR: (f64, f64, f64) = (0.5019608, 0.5019608, 0.5019608);

pub(crate) fn register(state: &mut LuaState, namespace: GcRef<Table>) -> LuaResult<()> {
    table_set_rust_fn_static(state, namespace, "GetPvpBrawl", get_pvp_brawl)
}

/// The two records differ only in the forced queue permission and the
/// remaining-time presence that WM107 establishes.
struct BrawlRows {
    record: PvpBrawlInfo,
    remaining: Option<f64>,
    can_queue: bool,
}

fn get_pvp_brawl(state: &mut LuaState) -> LuaResult<u32> {
    let is_special = optional_bool(state, 1)?;
    let Some(rows) = select_rows(state, is_special)? else {
        return crate::c_api::tooltip_data::unknown_getter_failure();
    };
    if !rows.can_queue {
        return crate::c_api::tooltip_data::unknown_getter_failure();
    }
    let tooltip = compose_tooltip(state, TOOLTIP_TYPE_PVP_BRAWL, None);
    // Plain and blank rows pass no colour: the simulator's named-colour surface
    // resolves every unmodelled `*_COLOR` constant to white, so an explicit
    // highlight colour would equal the disabled placeholder and be re-typed by
    // the native value-based remap. The default normal colour publishes white,
    // which matches the measured highlight value.
    append_line(state, tooltip, LINE_TYPE_PLAIN, &rows.record.name, None);
    if let Some(remaining) = rows.remaining {
        let text = ends_in_text(state, remaining)?;
        append_line(
            state,
            tooltip,
            LINE_TYPE_DISABLED,
            &text,
            Some(DISABLED_COLOR),
        );
    }
    if !rows.record.map_names.is_empty() {
        append_blank(state, tooltip);
        let heading = map_heading(state, &rows.record)?;
        append_line(state, tooltip, LINE_TYPE_PLAIN, &heading, None);
        append_map_rows(state, tooltip, &rows.record)?;
    }
    if !rows.record.long_description.is_empty() {
        append_blank(state, tooltip);
        let rules = required_global_text(state, "BRAWL_TOOLTIP_RULES")?;
        append_line(state, tooltip, LINE_TYPE_PLAIN, &rules, None);
        append_line(
            state,
            tooltip,
            LINE_TYPE_PLAIN,
            &rows.record.long_description,
            None,
        );
    }
    state.push(tooltip);
    Ok(1)
}

fn select_rows(state: &mut LuaState, is_special: bool) -> LuaResult<Option<BrawlRows>> {
    if is_special {
        let special = borrow_state(state)?.special_event_brawl.clone();
        return Ok(special.map(|record| BrawlRows {
            record: record.into(),
            remaining: None,
            // WM107: the special-event path forces queue permission true.
            can_queue: true,
        }));
    }
    let active = borrow_state(state)?.active_brawl.clone();
    Ok(active.map(|record| {
        let remaining = record.time_left_until_next_change;
        let can_queue = record.can_queue;
        BrawlRows {
            record,
            remaining,
            can_queue,
        }
    }))
}

fn append_blank(state: &mut LuaState, tooltip: Val) {
    append_line(state, tooltip, LINE_TYPE_BLANK, BLANK_TEXT, None);
}

fn map_heading(state: &mut LuaState, record: &PvpBrawlInfo) -> LuaResult<String> {
    // WM107: the all-arena arm is plural; one explicit map is singular.
    let key = if !record.includes_all_arenas && record.map_names.len() == 1 {
        "BRAWL_TOOLTIP_MAP"
    } else {
        "BRAWL_TOOLTIP_MAPS"
    };
    required_global_text(state, key)
}

fn append_map_rows(state: &mut LuaState, tooltip: Val, record: &PvpBrawlInfo) -> LuaResult<()> {
    if record.includes_all_arenas {
        let text = required_global_text(state, "PVP_BRAWL_TOOLTIP_ALL_ARENA_MAPS")?;
        append_line(state, tooltip, LINE_TYPE_PLAIN, &text, None);
        return Ok(());
    }
    for map in &record.map_names {
        let text = format_global_text(state, "BRAWL_TOOLTIP_MAP_ENTRY", map)?;
        append_line(state, tooltip, LINE_TYPE_PLAIN, &text, None);
    }
    Ok(())
}

/// EMULATED: WM107 leaves the rendered ends-in text UNKNOWN; format the pinned
/// `BRAWL_TOOLTIP_ENDS` global with the simulator's localized duration text.
fn ends_in_text(state: &mut LuaState, remaining: f64) -> LuaResult<String> {
    let milliseconds = if remaining.is_finite() && remaining > 0.0 {
        (remaining * 1000.0).ceil().min(f64::from(u32::MAX)) as u32
    } else {
        0
    };
    let duration = format_remaining_time(state, milliseconds)?;
    format_global_text(state, "BRAWL_TOOLTIP_ENDS", &duration)
}

fn required_global_text(state: &mut LuaState, key: &str) -> LuaResult<String> {
    global_text(state, key)
        .ok_or_else(|| rilua::runtime_error(format!("missing localized global {key}")))
}
