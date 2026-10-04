//! Rilua A_Admin handlers — Buffs.
//!
//! Extracted from rilua_admin_extras.rs per the 750-line file cap and to keep
//! each sub-module focused on a single concern. The parent entry
//! point in admin.rs imports these as pub(super) and weaves
//! them into the A_Admin TableBuilder chain.

use super::admin::{build_admin_aura, build_admin_buff, opt_string_stack};
use crate::lua_api::methods::{borrow_state_mut, create_string, create_table, table_set};
use crate::lua_api::script_helpers::fire_named_event_state;
use crate::lua_bridge::FromStack;
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

// ── Buffs ─────────────────────────────────────────────────────────────────────

/// Read the icon argument as a string fileDataID, accepting both `"136207"`
/// and `136207` (the documented numeric form).
fn icon_string_from_stack(state: &mut LuaState, index: i32) -> LuaResult<String> {
    if let Ok(icon) = String::from_stack(state, index) {
        return Ok(icon);
    }
    let icon = f64::from_stack(state, index)?;
    Ok(format!("{}", icon as i64))
}

/// Fire `UNIT_AURA` for the player with a full-update payload, matching the
/// event listeners in Blizzard's BuffFrame/DebuffFrame which ignore the event
/// unless `unitAuraUpdateInfo` reports changed auras.
fn fire_player_unit_aura(state: &mut LuaState) -> LuaResult<()> {
    let unit = create_string(state, "player");
    let info = create_table(state);
    table_set(state, info, "isFullUpdate", Val::Bool(true));
    let info = crate::c_api::unit_aura_access::unit_aura_event_payload(state, info)?;
    fire_named_event_state(state, "UNIT_AURA", &[unit, info]);
    Ok(())
}

pub(super) fn add_buff(state: &mut LuaState) -> LuaResult<u32> {
    let spell_id = i32::from_stack(state, 1)?;
    let name = String::from_stack(state, 2)?;
    let icon = icon_string_from_stack(state, 3)?;
    let duration = f64::from_stack(state, 4)?;
    let stacks = i32::from_stack(state, 5)?;
    {
        let mut st = borrow_state_mut(state)?;
        let buff = build_admin_buff(&st, spell_id, name, icon, duration, stacks);
        st.player.buffs.push(buff);
    }
    fire_player_unit_aura(state)?;
    Ok(0)
}

pub(super) fn add_debuff(state: &mut LuaState) -> LuaResult<u32> {
    let spell_id = i32::from_stack(state, 1)?;
    let name = String::from_stack(state, 2)?;
    let icon = icon_string_from_stack(state, 3)?;
    let duration = f64::from_stack(state, 4)?;
    let stacks = i32::from_stack(state, 5)?;
    let dispel_type = match opt_string_stack(state, 6, "") {
        s if s.is_empty() => None,
        s => Some(s),
    };
    {
        let mut st = borrow_state_mut(state)?;
        let is_helpful = false;
        let debuff = build_admin_aura(
            &st,
            spell_id,
            name,
            icon,
            duration,
            stacks,
            is_helpful,
            dispel_type,
        );
        st.player.buffs.push(debuff);
    }
    fire_player_unit_aura(state)?;
    Ok(0)
}

pub(super) fn remove_buff(state: &mut LuaState) -> LuaResult<u32> {
    let spell_id = i32::from_stack(state, 1)?;
    {
        let mut st = borrow_state_mut(state)?;
        st.player.buffs.retain(|a| a.spell_id != spell_id);
    }
    fire_player_unit_aura(state)?;
    Ok(0)
}

pub(super) fn clear_buffs(state: &mut LuaState) -> LuaResult<u32> {
    {
        let mut st = borrow_state_mut(state)?;
        st.player.buffs.clear();
    }
    fire_player_unit_aura(state)?;
    Ok(0)
}

#[cfg(test)]
mod tests {
    use crate::lua_api::WowLuaEnv;

    #[test]
    fn add_debuff_populates_player_harmful_aura_queries() {
        let env = WowLuaEnv::new().expect("lua env should initialize");

        let result: String = env
            .eval(
                r#"
                A_Admin.ClearBuffs()
                A_Admin.AddDebuff(589, "Shadow Word: Pain", "136207", 30, 1, "Magic")

                local name, icon, count, debuffName = UnitDebuff("player", 1)
                if name ~= "Shadow Word: Pain" then
                    return "missing_unit_debuff"
                end
                if debuffName ~= "Magic" then
                    return "missing_dispel_name"
                end

                local aura = C_UnitAuras.GetAuraDataByIndex("player", 1, "HARMFUL")
                if not aura or aura.dispelName ~= "Magic" then
                    return "missing_harmful_aura_data"
                end

                return "ok"
                "#,
            )
            .expect("admin aura probe should run");

        assert_eq!(result, "ok");
    }
}
