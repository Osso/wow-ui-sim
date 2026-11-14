//! Retail `C_UIColor.GetColors` backed by generated GlobalColor DB2 rows.

use crate::c_api::helpers::register_namespace_function_with_flags;
use crate::client_profile::{ACTIVE, ClientProfile};
use crate::lua_api::methods::{table_set_num, table_set_static};
use rilua::vm::gc::arena::GcRef;
use rilua::vm::state::LuaState;
use rilua::vm::table::Table;
use rilua::{LuaResult, Val, runtime_error};

use sim_data_profile::global_colors_retail::{GlobalColorRow, RETAIL_GLOBAL_COLORS};

const COLOR_CHANNEL_RECIPROCAL: f32 = 1.0_f32 / 255.0_f32;
const COLOR_MIXIN_ERROR: &str = "unable to find mixin or metatable (ColorMixin)";

pub(crate) fn register(lua: &mut rilua::Lua) -> LuaResult<()> {
    if ACTIVE != ClientProfile::Retail {
        return Ok(());
    }

    // Flag 3 keeps distinct namespace tables and publishes one shared function value.
    register_namespace_function_with_flags(lua.state_mut(), "C_UIColor", "GetColors", get_colors, 3)
}

fn get_colors(state: &mut LuaState) -> LuaResult<u32> {
    let rows = RETAIL_GLOBAL_COLORS;
    let output_ref = state.gc.alloc_table(Table::with_sizes(rows.len(), 0));
    let output = Val::Table(output_ref);
    let stack_top = state.top;
    state.push(output);

    if !rows.is_empty() {
        let mixin = find_color_mixin(state)?;
        for (index, row) in rows.iter().enumerate() {
            append_color_record(state, output_ref, index, mixin, row)?;
            state.top = stack_top + 1;
        }
    }

    state.top = stack_top;
    state.push(output);
    Ok(1)
}

fn find_color_mixin(state: &mut LuaState) -> LuaResult<GcRef<Table>> {
    match crate::lua_api::global_colors::global_value(state, "ColorMixin") {
        Val::Table(mixin) => Ok(mixin),
        _ => Err(runtime_error(COLOR_MIXIN_ERROR)),
    }
}

fn append_color_record(
    state: &mut LuaState,
    output: GcRef<Table>,
    index: usize,
    mixin: GcRef<Table>,
    row: &GlobalColorRow,
) -> LuaResult<()> {
    let color = build_color(state, mixin, row.packed_color)?;
    state.push(color);

    let record_ref = state.gc.alloc_table(Table::with_sizes(0, 2));
    let record = Val::Table(record_ref);
    state.push(record);
    let tag = Val::Str(state.gc.intern_string_static(row.base_tag.as_bytes()));
    table_set_static(state, record, "baseTag", tag);
    table_set_static(state, record, "color", color);
    table_set_num(state, output, (index + 1) as f64, record);
    Ok(())
}

fn build_color(state: &mut LuaState, mixin: GcRef<Table>, packed_color: u32) -> LuaResult<Val> {
    let color_ref = state.gc.alloc_table(Table::new());
    let color = Val::Table(color_ref);
    let stack_top = state.top;
    state.push(color);
    copy_mixin_members(state, mixin, color_ref)?;

    let [red, green, blue, alpha] = channels_from_packed_color(packed_color);
    table_set_static(state, color, "r", Val::Num(red));
    table_set_static(state, color, "g", Val::Num(green));
    table_set_static(state, color, "b", Val::Num(blue));
    table_set_static(state, color, "a", Val::Num(alpha));
    state.top = stack_top;
    Ok(color)
}

fn copy_mixin_members(
    state: &mut LuaState,
    mixin: GcRef<Table>,
    color: GcRef<Table>,
) -> LuaResult<()> {
    let mut key = Val::Nil;
    loop {
        let next = state
            .gc
            .tables
            .get(mixin)
            .ok_or_else(|| runtime_error("ColorMixin table has been collected"))?
            .next(key, &state.gc.string_arena)?;
        let Some((next_key, value)) = next else {
            return Ok(());
        };
        key = next_key;

        if let Some(table) = state.gc.tables.get_mut(color) {
            table.raw_set(next_key, value, &state.gc.string_arena)?;
        }
        state.gc.barrier_back(color);
    }
}

fn channels_from_packed_color(packed_color: u32) -> [f64; 4] {
    // VERIFIED for Retail 12.1.5.70688; applying this mapping to 70783 data is INFERRED.
    let [alpha, red, green, blue] = packed_color.to_be_bytes();
    [red, green, blue, alpha].map(|byte| f64::from(f32::from(byte) * COLOR_CHANNEL_RECIPROCAL))
}
