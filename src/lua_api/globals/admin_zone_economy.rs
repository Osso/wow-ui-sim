//! Rilua A_Admin handlers — Zone, Economy.
//!
//! Extracted from rilua_admin_extras.rs per the 750-line file cap and to keep
//! each sub-module focused on a single concern. The parent entry
//! point in admin.rs imports these as pub(super) and weaves
//! them into the A_Admin TableBuilder chain.

use crate::lua_api::methods::{borrow_state_mut, frame_id_from_stack};
use crate::lua_bridge::{FromStack, stack_val};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

// ── Zone ──────────────────────────────────────────────────────────────────────

pub(super) fn set_zone(state: &mut LuaState) -> LuaResult<u32> {
    let name = String::from_stack(state, 1)?;
    let id = i32::from_stack(state, 2)?;
    let mut st = borrow_state_mut(state)?;
    st.world.zone_name = name;
    st.world.zone_id = id;
    Ok(0)
}

pub(super) fn set_sub_zone(state: &mut LuaState) -> LuaResult<u32> {
    let name = String::from_stack(state, 1)?;
    borrow_state_mut(state)?.world.sub_zone_name = name;
    Ok(0)
}

pub(super) fn set_bind_location(state: &mut LuaState) -> LuaResult<u32> {
    let name = String::from_stack(state, 1)?;
    borrow_state_mut(state)?.bind_location = name;
    Ok(0)
}

pub(super) fn set_instance_info(state: &mut LuaState) -> LuaResult<u32> {
    let name = String::from_stack(state, 1)?;
    let inst_type = String::from_stack(state, 2)?;
    let difficulty = i32::from_stack(state, 3)?;
    let max_players = i32::from_stack(state, 4)?;
    let lfg_dungeon_id = Option::<i32>::from_stack(state, 5)?;
    let mut st = borrow_state_mut(state)?;
    st.world.instance_name = name;
    st.world.instance_type = inst_type;
    st.world.instance_difficulty = difficulty;
    st.world.instance_max_players = max_players;
    st.world.instance_lfg_dungeon_id = lfg_dungeon_id;
    st.world.in_instance = true;
    Ok(0)
}

pub(super) fn set_in_instance(state: &mut LuaState) -> LuaResult<u32> {
    let v = bool::from_stack(state, 1)?;
    borrow_state_mut(state)?.world.in_instance = v;
    Ok(0)
}

// ── Economy ───────────────────────────────────────────────────────────────────

pub(super) fn set_money(state: &mut LuaState) -> LuaResult<u32> {
    let copper = i64::from_stack(state, 1)?;
    borrow_state_mut(state)?.player.money = copper;
    Ok(0)
}

pub(super) fn set_item_level(state: &mut LuaState) -> LuaResult<u32> {
    let ilvl = f64::from_stack(state, 1)?;
    borrow_state_mut(state)?.player.item_level = ilvl as f32;
    Ok(0)
}

/// `A_Admin.SetMousePosition(x, y)` — pass nil for either coordinate to clear
/// the simulated cursor. Drives `GetCursorPosition()` and `Region:IsMouseOver`.
pub(super) fn set_mouse_position(state: &mut LuaState) -> LuaResult<u32> {
    let x = Option::<f64>::from_stack(state, 1)?;
    let y = Option::<f64>::from_stack(state, 2)?;
    let pos = match (x, y) {
        (Some(x), Some(y)) => Some((x as f32, y as f32)),
        _ => None,
    };
    borrow_state_mut(state)?.set_mouse_position(pos);
    Ok(0)
}

/// `A_Admin.SetMouseOverFrame(frame)` moves the simulated cursor to the
/// frame's resolved screen-space center and marks that frame as hovered.
/// Passing nil clears both mouse-over fields.
pub(super) fn set_mouse_over_frame(state: &mut LuaState) -> LuaResult<u32> {
    let target = if matches!(stack_val(state, 1), Val::Nil) {
        None
    } else {
        Some(frame_id_from_stack(state, 1)?)
    };
    let (old_hovered, new_hovered) = {
        let mut st = borrow_state_mut(state)?;
        let old_hovered = st.hovered_frame;
        let position = target.and_then(|id| {
            st.resolve_rect_if_dirty(id);
            st.widgets.get(id).and_then(|frame| {
                frame
                    .layout_rect
                    .map(|rect| (rect.x + rect.width / 2.0, rect.y + rect.height / 2.0))
            })
        });
        st.hovered_frame = target.filter(|_| position.is_some());
        st.set_mouse_position(position);
        (old_hovered, st.hovered_frame)
    };
    if old_hovered != new_hovered {
        for id in [old_hovered, new_hovered].into_iter().flatten() {
            crate::lua_api::frame::methods::button_anchor_hierarchy::refresh_button_font(
                state, id,
            )?;
        }
    }
    Ok(0)
}

pub(super) fn set_game_time(state: &mut LuaState) -> LuaResult<u32> {
    let time = match (stack_val(state, 1), stack_val(state, 2)) {
        (Val::Nil, Val::Nil) => None,
        (Val::Num(hour), Val::Num(minute))
            if hour.fract() == 0.0
                && minute.fract() == 0.0
                && (0.0..=23.0).contains(&hour)
                && (0.0..=59.0).contains(&minute) =>
        {
            Some((hour as u8, minute as u8))
        }
        _ => {
            return Err(rilua::runtime_error(
                "A_Admin.SetGameTime expects hour 0-23 and minute 0-59, or nil",
            ));
        }
    };
    borrow_state_mut(state)?.game_clock.time = time;
    Ok(0)
}

// ── Network stats (for GetNetStats) ───────────────────────────────────────────

/// `A_Admin.SetNetStats(bandwidthIn, bandwidthOut, latencyHome, latencyWorld)`.
/// All four arguments are optional; missing values default to 0. Drives the
/// values returned by `GetNetStats` (registered in `real::net_stats`).
pub(super) fn set_net_stats(state: &mut LuaState) -> LuaResult<u32> {
    let bandwidth_in = Option::<f64>::from_stack(state, 1)?.unwrap_or(0.0);
    let bandwidth_out = Option::<f64>::from_stack(state, 2)?.unwrap_or(0.0);
    let latency_home = Option::<f64>::from_stack(state, 3)?.unwrap_or(0.0);
    let latency_world = Option::<f64>::from_stack(state, 4)?.unwrap_or(0.0);
    let mut st = borrow_state_mut(state)?;
    st.net_stats.bandwidth_in_kbps = bandwidth_in;
    st.net_stats.bandwidth_out_kbps = bandwidth_out;
    st.net_stats.latency_home_ms = latency_home;
    st.net_stats.latency_world_ms = latency_world;
    Ok(0)
}

/// `A_Admin.SetStoreFrameShown(shown)`. Missing arg defaults to `true` so
/// `A_Admin.SetStoreFrameShown()` opens the store. Drives `StoreFrame_IsShown`
/// (registered in `store_frame.rs`).
pub(super) fn set_store_frame_shown(state: &mut LuaState) -> LuaResult<u32> {
    let shown = Option::<bool>::from_stack(state, 1)?.unwrap_or(true);
    borrow_state_mut(state)?.store_frame_shown = shown;
    Ok(0)
}

/// `A_Admin.SetTimerunningSeasonID(id)` — pass `nil` or `0` to clear
/// (no active seasonal mode), or a positive id to enable. Drives
/// `PlayerIsTimerunning()` (returns whether id is non-zero) and
/// `PlayerGetTimerunningSeasonID()` (returns the id, or 0 when none).
pub(super) fn set_timerunning_season_id(state: &mut LuaState) -> LuaResult<u32> {
    let id = Option::<f64>::from_stack(state, 1)?.unwrap_or(0.0) as i64;
    let season = if id > 0 { Some(id as u32) } else { None };
    borrow_state_mut(state)?.timerunning_season_id = season;
    Ok(0)
}

/// `A_Admin.SetShiftKeyDown(down?)` — missing arg defaults to `true` so
/// `A_Admin.SetShiftKeyDown()` presses the key. Drives `IsShiftKeyDown()`
/// and contributes to `IsModifierKeyDown()`.
pub(super) fn set_shift_key_down(state: &mut LuaState) -> LuaResult<u32> {
    let down = Option::<bool>::from_stack(state, 1)?.unwrap_or(true);
    borrow_state_mut(state)?.modifier_keys.shift = down;
    Ok(0)
}

/// `A_Admin.SetControlKeyDown(down?)` — see `SetShiftKeyDown`.
pub(super) fn set_control_key_down(state: &mut LuaState) -> LuaResult<u32> {
    let down = Option::<bool>::from_stack(state, 1)?.unwrap_or(true);
    borrow_state_mut(state)?.modifier_keys.control = down;
    Ok(0)
}

/// `A_Admin.SetAltKeyDown(down?)` — see `SetShiftKeyDown`.
pub(super) fn set_alt_key_down(state: &mut LuaState) -> LuaResult<u32> {
    let down = Option::<bool>::from_stack(state, 1)?.unwrap_or(true);
    borrow_state_mut(state)?.modifier_keys.alt = down;
    Ok(0)
}

/// `A_Admin.SetMetaKeyDown(down?)` — see `SetShiftKeyDown`. Does NOT
/// contribute to `IsModifierKeyDown()` (matches WoW semantics).
pub(super) fn set_meta_key_down(state: &mut LuaState) -> LuaResult<u32> {
    let down = Option::<bool>::from_stack(state, 1)?.unwrap_or(true);
    borrow_state_mut(state)?.modifier_keys.meta = down;
    Ok(0)
}

/// `A_Admin.SetGuildRanks({ {name, flags}, ... })` — replaces the guild
/// roster. Each entry is `{ name = string, flags = { bool, bool, ... } }`.
/// Pass no arg or an empty table to clear the roster (no guild).
pub(super) fn set_guild_ranks(state: &mut LuaState) -> LuaResult<u32> {
    use rilua::Val;
    let arg = crate::lua_bridge::stack_val(state, 1);
    let Val::Table(list_ref) = arg else {
        let mut st = borrow_state_mut(state)?;
        st.world.guild_ranks.clear();
        st.world.guild_selected_rank = 0;
        return Ok(0);
    };
    // Snapshot the list's array part before we touch state mutably.
    let entry_refs: Vec<rilua::vm::gc::arena::GcRef<rilua::vm::table::Table>> =
        match state.gc.tables.get(list_ref) {
            Some(list) => list
                .array_slice()
                .iter()
                .filter_map(|v| match v {
                    Val::Table(r) => Some(*r),
                    _ => None,
                })
                .collect(),
            None => Vec::new(),
        };
    let ranks: Vec<_> = entry_refs
        .into_iter()
        .map(|entry_ref| read_rank_entry(state, entry_ref))
        .collect();
    let mut st = borrow_state_mut(state)?;
    st.world.guild_ranks = ranks;
    if st.world.guild_selected_rank as usize > st.world.guild_ranks.len() {
        st.world.guild_selected_rank = 0;
    }
    Ok(0)
}

fn read_rank_entry(
    state: &mut LuaState,
    entry_ref: rilua::vm::gc::arena::GcRef<rilua::vm::table::Table>,
) -> crate::lua_api::state_types::GuildRank {
    use rilua::Val;
    let name_key = state.gc.intern_string_static(b"name");
    let flags_key = state.gc.intern_string_static(b"flags");
    let (name_val, flags_val) = {
        let Some(table) = state.gc.tables.get(entry_ref) else {
            return crate::lua_api::state_types::GuildRank::default();
        };
        (
            table.get_str(name_key, &state.gc.string_arena),
            table.get_str(flags_key, &state.gc.string_arena),
        )
    };
    let name = match name_val {
        Val::Str(s) => state
            .gc
            .string_arena
            .get(s)
            .and_then(|lua_str| std::str::from_utf8(lua_str.data()).ok())
            .map(str::to_owned)
            .unwrap_or_default(),
        _ => String::new(),
    };
    let flags: Vec<bool> = match flags_val {
        Val::Table(flags_ref) => match state.gc.tables.get(flags_ref) {
            Some(flags_table) => flags_table
                .array_slice()
                .iter()
                .map(|v| matches!(v, Val::Bool(true)))
                .collect(),
            None => Vec::new(),
        },
        _ => Vec::new(),
    };
    crate::lua_api::state_types::GuildRank { name, flags }
}

/// `A_Admin.SetGuildEmblem(filename, bkgR, bkgG, bkgB, borderR, borderG,
/// borderB, emblemR, emblemG, emblemB)` — every arg is optional; missing
/// values default to `0.0` (colours) or `""` (filename). Drives
/// `GetGuildLogoInfo()`.
pub(super) fn set_guild_emblem(state: &mut LuaState) -> LuaResult<u32> {
    let filename = Option::<String>::from_stack(state, 1)?.unwrap_or_default();
    let bkg_r = Option::<f64>::from_stack(state, 2)?.unwrap_or(0.0);
    let bkg_g = Option::<f64>::from_stack(state, 3)?.unwrap_or(0.0);
    let bkg_b = Option::<f64>::from_stack(state, 4)?.unwrap_or(0.0);
    let border_r = Option::<f64>::from_stack(state, 5)?.unwrap_or(0.0);
    let border_g = Option::<f64>::from_stack(state, 6)?.unwrap_or(0.0);
    let border_b = Option::<f64>::from_stack(state, 7)?.unwrap_or(0.0);
    let emblem_r = Option::<f64>::from_stack(state, 8)?.unwrap_or(0.0);
    let emblem_g = Option::<f64>::from_stack(state, 9)?.unwrap_or(0.0);
    let emblem_b = Option::<f64>::from_stack(state, 10)?.unwrap_or(0.0);
    let mut st = borrow_state_mut(state)?;
    let logo = &mut st.world.guild_logo;
    logo.tabard_metadata = None;
    logo.emblem_filename = filename;
    logo.background = (bkg_r, bkg_g, bkg_b);
    logo.border = (border_r, border_g, border_b);
    logo.emblem = (emblem_r, emblem_g, emblem_b);
    Ok(0)
}

/// `A_Admin.SetZonePVP(pvpType, isSubZonePvP, factionName)` — drives the
/// three return values of `C_PvP.GetZonePVPInfo()`. `pvpType` defaults to
/// `"contested"`, `isSubZonePvP` defaults to `false`, `factionName` defaults
/// to `nil` (neutral zone). Pass an empty string for `factionName` to clear
/// an earlier faction assignment.
pub(super) fn set_zone_pvp(state: &mut LuaState) -> LuaResult<u32> {
    let pvp_type = Option::<String>::from_stack(state, 1)?
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "contested".into());
    let is_sub_zone = Option::<bool>::from_stack(state, 2)?.unwrap_or(false);
    let faction = Option::<String>::from_stack(state, 3)?.filter(|s| !s.is_empty());
    let mut st = borrow_state_mut(state)?;
    st.world.pvp_type = pvp_type;
    st.world.is_sub_zone_pvp = is_sub_zone;
    st.world.pvp_faction_name = faction;
    Ok(0)
}
