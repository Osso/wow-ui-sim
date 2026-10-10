//! Color tables — make_rilua_color_table, named color globals, RAID_CLASS_COLORS,
//! C_ClassColor, and tooltip / item-quality / class-name / icon-list stubs.

use crate::lua_api::globals::strings::string_data::game_enums::ITEM_QUALITY_COLORS_DATA;
use crate::lua_api::methods::{
    create_string, create_table, table_get, table_set, table_set_num, val_to_string,
};
use crate::lua_bridge::table_set_rust_fn_static;
use crate::lua_bridge::{FromStack, IntoStack, stack_val};
use rilua::vm::state::LuaState;
use rilua::{LuaApiMut, LuaResult, Val};

use super::set_global_val;

type ColorTableRef = rilua::vm::gc::arena::GcRef<rilua::vm::table::Table>;

// ── Data ─────────────────────────────────────────────────────────────────────

pub(super) const NAMED_COLOR_GLOBALS: &[(&'static str, (f64, f64, f64, f64))] = &[
    (
        "PLAYER_FACTION_COLOR_HORDE",
        (0.90196, 0.05098, 0.07059, 1.0),
    ),
    (
        "PLAYER_FACTION_COLOR_ALLIANCE",
        (0.29412, 0.33333, 0.91373, 1.0),
    ),
    ("NORMAL_FONT_COLOR", (1.0, 0.82, 0.0, 1.0)),
    ("HIGHLIGHT_FONT_COLOR", (1.0, 1.0, 1.0, 1.0)),
    ("RED_FONT_COLOR", (1.0, 0.1, 0.1, 1.0)),
    ("GREEN_FONT_COLOR", (0.1, 1.0, 0.1, 1.0)),
    ("GRAY_FONT_COLOR", (0.5, 0.5, 0.5, 1.0)),
    ("PASSIVE_SPELL_FONT_COLOR", (0.5, 0.5, 0.5, 1.0)),
    ("BLACK_FONT_COLOR", (0.0, 0.0, 0.0, 1.0)),
    ("YELLOW_FONT_COLOR", (1.0, 1.0, 0.0, 1.0)),
    ("LIGHTYELLOW_FONT_COLOR", (1.0, 1.0, 0.6, 1.0)),
    ("UNIT_LEVEL_NON_ATTACKABLE", (1.0, 1.0, 0.0, 1.0)),
    ("ORANGE_FONT_COLOR", (1.0, 0.5, 0.25, 1.0)),
    ("WHITE_FONT_COLOR", (1.0, 1.0, 1.0, 1.0)),
    ("DISABLED_FONT_COLOR", (0.5, 0.5, 0.5, 1.0)),
    ("HEIRLOOM_BLUE_COLOR", (0.0, 0.8, 1.0, 1.0)),
    ("DIM_RED_FONT_COLOR", (0.8, 0.1, 0.1, 1.0)),
    ("LIGHTBLUE_FONT_COLOR", (0.51176, 0.77255, 1.0, 1.0)),
    ("HIGHLIGHT_LIGHT_BLUE", (0.51176, 0.77255, 1.0, 1.0)),
    ("FRIENDS_BNET_BACKGROUND_COLOR", (0.0, 0.694, 0.941, 0.05)),
    ("FRIENDS_WOW_NAME_COLOR", (0.996, 0.882, 0.361, 1.0)),
    ("FRIENDS_WOW_BACKGROUND_COLOR", (1.0, 0.824, 0.0, 0.05)),
    ("FRIENDS_GRAY_COLOR", (0.486, 0.518, 0.541, 1.0)),
    (
        "FRIENDS_OFFLINE_BACKGROUND_COLOR",
        (0.588, 0.588, 0.588, 0.05),
    ),
    ("ACTIONBAR_HOTKEY_FONT_COLOR", (0.6, 0.6, 0.6, 1.0)),
    ("FACTION_RED_COLOR", (0.8, 0.13, 0.13, 1.0)),
    ("FACTION_ORANGE_COLOR", (0.93, 0.53, 0.13, 1.0)),
    ("FACTION_YELLOW_COLOR", (0.8, 0.73, 0.13, 1.0)),
    ("FACTION_GREEN_COLOR", (0.13, 0.8, 0.13, 1.0)),
    (
        "OBJECTIVE_TRACKER_BLOCK_HEADER_COLOR",
        (1.0, 0.82, 0.0, 1.0),
    ),
    ("PANEL_BACKGROUND_COLOR", (0.15, 0.15, 0.15, 1.0)),
    ("EDIT_MODE_GRID_LINE_COLOR", (1.0, 1.0, 1.0, 0.3)),
    ("EDIT_MODE_GRID_CENTER_LINE_COLOR", (0.0, 0.8, 1.0, 0.6)),
    ("DEBUFF_TYPE_CURSE_COLOR", (0.6, 0.0, 1.0, 1.0)),
    ("DEBUFF_TYPE_DISEASE_COLOR", (0.6, 0.4, 0.0, 1.0)),
    ("DEBUFF_TYPE_MAGIC_COLOR", (0.2, 0.6, 1.0, 1.0)),
    ("DEBUFF_TYPE_POISON_COLOR", (0.0, 0.6, 0.0, 1.0)),
    ("DEBUFF_TYPE_BLEED_COLOR", (1.0, 0.0, 0.0, 1.0)),
    ("DEBUFF_TYPE_NONE_COLOR", (1.0, 1.0, 1.0, 0.0)),
    // Mists 5.5.4.70268 GlobalColor rows 395–398, literal AARRGGBB.
    // Preserve encoded alpha 0; native GetRGBA parity remains unproven.
    #[cfg(feature = "client-mists")]
    ("CASTBAR_CLASSIC_YELLOW", (1.0, 179.0 / 255.0, 0.0, 0.0)), // 0x00ffb300
    #[cfg(feature = "client-mists")]
    ("CASTBAR_CLASSIC_GREEN", (0.0, 1.0, 0.0, 0.0)), // 0x0000ff00
    #[cfg(feature = "client-mists")]
    (
        "CASTBAR_CLASSIC_GRAY",
        (179.0 / 255.0, 179.0 / 255.0, 179.0 / 255.0, 0.0),
    ), // 0x00b3b3b3
    #[cfg(feature = "client-mists")]
    ("CASTBAR_CLASSIC_RED", (1.0, 0.0, 0.0, 0.0)), // 0x00ff0000
];

pub(super) const RAID_CLASS_COLORS_DATA: &[(&'static str, (f64, f64, f64, f64))] = &[
    ("WARRIOR", (0.78, 0.61, 0.43, 1.0)),
    ("PALADIN", (0.96, 0.55, 0.73, 1.0)),
    ("HUNTER", (0.67, 0.83, 0.45, 1.0)),
    ("ROGUE", (1.00, 0.96, 0.41, 1.0)),
    ("PRIEST", (1.00, 1.00, 1.00, 1.0)),
    ("DEATHKNIGHT", (0.77, 0.12, 0.23, 1.0)),
    ("SHAMAN", (0.00, 0.44, 0.87, 1.0)),
    ("MAGE", (0.25, 0.78, 0.92, 1.0)),
    ("WARLOCK", (0.53, 0.53, 0.93, 1.0)),
    ("MONK", (0.00, 1.00, 0.60, 1.0)),
    ("DRUID", (1.00, 0.49, 0.04, 1.0)),
    ("DEMONHUNTER", (0.64, 0.19, 0.79, 1.0)),
    ("EVOKER", (0.20, 0.58, 0.50, 1.0)),
    ("ADVENTURER", (1.00, 1.00, 1.00, 1.0)),
    ("TRAVELER", (1.00, 1.00, 1.00, 1.0)),
];

const DEBUFF_TYPE_COLORS_DATA: &[(&str, (f64, f64, f64, f64))] = &[
    ("Curse", (0.6, 0.0, 1.0, 1.0)),
    ("Disease", (0.6, 0.4, 0.0, 1.0)),
    ("Magic", (0.2, 0.6, 1.0, 1.0)),
    ("Poison", (0.0, 0.6, 0.0, 1.0)),
];

pub fn debuff_type_color(debuff_type: &str) -> Option<(f64, f64, f64, f64)> {
    DEBUFF_TYPE_COLORS_DATA
        .iter()
        .find(|(name, _)| *name == debuff_type)
        .map(|(_, color)| *color)
}

// ── Color table field accessor helpers ───────────────────────────────────────

fn color_channel(state: &mut LuaState, this: Val, key: &str, default: f64) -> f64 {
    match table_get(state, this, key) {
        Val::Num(n) => n,
        _ => default,
    }
}

fn color_rgb(state: &mut LuaState, this: Val) -> (f64, f64, f64) {
    let r = color_channel(state, this, "r", 0.0);
    let g = color_channel(state, this, "g", 0.0);
    let b = color_channel(state, this, "b", 0.0);
    (r, g, b)
}

fn hex_from_rgb(r: f64, g: f64, b: f64) -> String {
    format!(
        "ff{:02x}{:02x}{:02x}",
        (r * 255.0) as u8,
        (g * 255.0) as u8,
        (b * 255.0) as u8
    )
}

fn color_channel_byte(value: f64) -> i64 {
    ((value * 255.0) + 0.5).floor() as i64
}

fn rgb_bytes_from_color(r: f64, g: f64, b: f64) -> (i64, i64, i64) {
    (
        color_channel_byte(r),
        color_channel_byte(g),
        color_channel_byte(b),
    )
}

// ── Color table constructor ───────────────────────────────────────────────────

fn register_color_access_methods(state: &mut LuaState, t_ref: ColorTableRef) -> LuaResult<()> {
    table_set_rust_fn_static(state, t_ref, "GetRGB", |state| {
        let this = stack_val(state, 1);
        let (r, g, b) = color_rgb(state, this);
        (r, g, b).into_stack(state)
    })?;
    table_set_rust_fn_static(state, t_ref, "GetRGBA", |state| {
        let this = stack_val(state, 1);
        let (r, g, b) = color_rgb(state, this);
        let a = color_channel(state, this, "a", 1.0);
        (r, g, b, a).into_stack(state)
    })?;
    table_set_rust_fn_static(state, t_ref, "GetRGBAsBytes", |state| {
        let this = stack_val(state, 1);
        let (r, g, b) = color_rgb(state, this);
        let (r, g, b) = rgb_bytes_from_color(r, g, b);
        (r, g, b).into_stack(state)
    })?;
    table_set_rust_fn_static(state, t_ref, "GetRGBAAsBytes", |state| {
        let this = stack_val(state, 1);
        let (r, g, b) = color_rgb(state, this);
        let a = color_channel(state, this, "a", 1.0);
        let (r, g, b) = rgb_bytes_from_color(r, g, b);
        (r, g, b, color_channel_byte(a)).into_stack(state)
    })?;
    Ok(())
}

fn register_color_format_methods(state: &mut LuaState, t_ref: ColorTableRef) -> LuaResult<()> {
    table_set_rust_fn_static(state, t_ref, "GenerateHexColor", |state| {
        let this = stack_val(state, 1);
        let (r, g, b) = color_rgb(state, this);
        let hex = hex_from_rgb(r, g, b);
        create_string(state, &hex).into_stack(state)
    })?;
    table_set_rust_fn_static(state, t_ref, "GenerateHexColorNoAlpha", |state| {
        let this = stack_val(state, 1);
        let (r, g, b) = color_rgb(state, this);
        let (r, g, b) = rgb_bytes_from_color(r, g, b);
        let hex = format!("{r:02X}{g:02X}{b:02X}");
        create_string(state, &hex).into_stack(state)
    })?;
    table_set_rust_fn_static(state, t_ref, "WrapTextInColorCode", |state| {
        let this = stack_val(state, 1);
        let text = String::from_stack(state, 2)?;
        let (r, g, b) = color_rgb(state, this);
        let wrapped = format!("|c{}{}|r", hex_from_rgb(r, g, b), text);
        create_string(state, &wrapped).into_stack(state)
    })?;
    Ok(())
}

fn register_color_methods(state: &mut LuaState, t_ref: ColorTableRef) -> LuaResult<()> {
    register_color_access_methods(state, t_ref)?;
    register_color_equality_methods(state, t_ref)?;
    register_color_format_methods(state, t_ref)?;
    Ok(())
}

fn register_color_equality_methods(state: &mut LuaState, t_ref: ColorTableRef) -> LuaResult<()> {
    table_set_rust_fn_static(state, t_ref, "IsRGBEqualTo", |state| {
        let this = stack_val(state, 1);
        let other = stack_val(state, 2);
        let (r, g, b) = color_rgb(state, this);
        let (other_r, other_g, other_b) = color_rgb(state, other);
        (r == other_r && g == other_g && b == other_b).into_stack(state)
    })?;
    table_set_rust_fn_static(state, t_ref, "IsEqualTo", |state| {
        let this = stack_val(state, 1);
        let other = stack_val(state, 2);
        let (r, g, b) = color_rgb(state, this);
        let (other_r, other_g, other_b) = color_rgb(state, other);
        let a = color_channel(state, this, "a", 1.0);
        let other_a = color_channel(state, other, "a", 1.0);
        (r == other_r && g == other_g && b == other_b && a == other_a).into_stack(state)
    })?;
    Ok(())
}

/// Build a rilua color table {r, g, b, a} with GetRGB/GetRGBA/GenerateHexColor/WrapTextInColorCode.
pub fn make_rilua_color_table(
    state: &mut LuaState,
    r: f64,
    g: f64,
    b: f64,
    a: f64,
) -> LuaResult<Val> {
    let t = create_table(state);
    table_set(state, t, "r", Val::Num(r));
    table_set(state, t, "g", Val::Num(g));
    table_set(state, t, "b", Val::Num(b));
    table_set(state, t, "a", Val::Num(a));
    let Val::Table(t_ref) = t else { unreachable!() };
    register_color_methods(state, t_ref)?;
    Ok(t)
}

// ── Registration helpers ──────────────────────────────────────────────────────

pub fn register_rilua_tooltip_colors(lua: &mut rilua::Lua) -> LuaResult<()> {
    // TODO: pull exact constants from string_data::TOOLTIP_DEFAULT_COLOR
    let state = lua.state_mut();
    let color = make_rilua_color_table(state, 1.0, 0.9, 0.0, 1.0)?;
    set_global_val(state, "TOOLTIP_DEFAULT_COLOR", color);
    let bg = make_rilua_color_table(state, 0.09, 0.09, 0.19, 1.0)?;
    set_global_val(state, "TOOLTIP_DEFAULT_BACKGROUND_COLOR", bg);
    Ok(())
}

pub fn register_rilua_item_quality_colors(lua: &mut rilua::Lua) -> LuaResult<()> {
    let state = lua.state_mut();
    let t = create_table(state);
    let Val::Table(table_ref) = t else {
        unreachable!("create_table must return table");
    };
    for (quality, r, g, b, hex) in ITEM_QUALITY_COLORS_DATA {
        let entry = make_item_quality_color_entry(state, *r, *g, *b, hex)?;
        table_set_num(state, table_ref, f64::from(*quality), entry);
    }
    set_global_val(state, "ITEM_QUALITY_COLORS", t);
    Ok(())
}

fn make_item_quality_color_entry(
    state: &mut LuaState,
    r: f64,
    g: f64,
    b: f64,
    hex: &str,
) -> LuaResult<Val> {
    let entry = create_table(state);
    table_set(state, entry, "r", Val::Num(r));
    table_set(state, entry, "g", Val::Num(g));
    table_set(state, entry, "b", Val::Num(b));
    let hex_markup = create_string(state, &format!("|c{hex}"));
    table_set(state, entry, "hex", hex_markup);
    let color = make_rilua_color_table(state, r, g, b, 1.0)?;
    table_set(state, entry, "color", color);
    Ok(entry)
}

pub fn register_rilua_class_name_tables(lua: &mut rilua::Lua) -> LuaResult<()> {
    // TODO: iterate CLASS_NAMES_DATA from string_data
    let state = lua.state_mut();
    let male = create_table(state);
    let female = create_table(state);
    set_global_val(state, "LOCALIZED_CLASS_NAMES_MALE", male);
    set_global_val(state, "LOCALIZED_CLASS_NAMES_FEMALE", female);
    Ok(())
}

pub fn register_rilua_icon_list(lua: &mut rilua::Lua) -> LuaResult<()> {
    // TODO: iterate ICON_LIST_DATA from string_data
    let state = lua.state_mut();
    let t = create_table(state);
    set_global_val(state, "ICON_LIST", t);
    Ok(())
}

fn build_raid_class_colors(lua: &mut rilua::Lua) -> LuaResult<Val> {
    let state = lua.state_mut();
    let raid_class_colors = create_table(state);
    for &(class_name, (r, g, b, a)) in RAID_CLASS_COLORS_DATA {
        let color = make_rilua_color_table(state, r, g, b, a)?;
        table_set(state, raid_class_colors, class_name, color);
    }
    Ok(raid_class_colors)
}

fn build_c_class_color(lua: &mut rilua::Lua) -> LuaResult<Val> {
    let state = lua.state_mut();
    let class_color_namespace = create_table(state);
    let Val::Table(class_color_ref) = class_color_namespace else {
        unreachable!("create_table must return a table");
    };
    table_set_rust_fn_static(state, class_color_ref, "GetClassColor", |state| {
        let class_name = val_to_string(state, stack_val(state, 1))
            .unwrap_or_default()
            .to_ascii_uppercase();
        let (r, g, b, a) = RAID_CLASS_COLORS_DATA
            .iter()
            .find(|(name, _)| *name == class_name)
            .map(|(_, color)| *color)
            .unwrap_or((1.0, 1.0, 1.0, 1.0));
        let color = make_rilua_color_table(state, r, g, b, a)?;
        state.push(color);
        Ok(1)
    })?;
    Ok(class_color_namespace)
}

fn build_debuff_type_colors(lua: &mut rilua::Lua) -> LuaResult<Val> {
    let state = lua.state_mut();
    let debuff_type_colors = create_table(state);
    for &(debuff_type, (r, g, b, a)) in DEBUFF_TYPE_COLORS_DATA {
        let color = make_rilua_color_table(state, r, g, b, a)?;
        table_set(state, debuff_type_colors, debuff_type, color);
    }
    Ok(debuff_type_colors)
}

pub fn register_rilua_color_globals(lua: &mut rilua::Lua) -> LuaResult<()> {
    let state = lua.state_mut();
    for &(name, (r, g, b, a)) in NAMED_COLOR_GLOBALS {
        let color = make_rilua_color_table(state, r, g, b, a)?;
        set_global_val(state, name, color);
    }
    let raid_class_colors = build_raid_class_colors(lua)?;
    set_global_val(lua.state_mut(), "RAID_CLASS_COLORS", raid_class_colors);
    let debuff_type_colors = build_debuff_type_colors(lua)?;
    set_global_val(lua.state_mut(), "DebuffTypeColor", debuff_type_colors);
    let c_class_color = build_c_class_color(lua)?;
    set_global_val(lua.state_mut(), "C_ClassColor", c_class_color);
    Ok(())
}

/// Register all color globals and color-related namespaces.
pub fn register_all(lua: &mut rilua::Lua) -> LuaResult<()> {
    register_rilua_tooltip_colors(lua)?;
    register_rilua_item_quality_colors(lua)?;
    register_rilua_class_name_tables(lua)?;
    register_rilua_icon_list(lua)?;
    register_rilua_color_globals(lua)?;
    Ok(())
}
