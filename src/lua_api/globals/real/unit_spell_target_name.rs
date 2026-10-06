//! Bounded actual-player-cast query; channel exclusion and nil arity are inferences.
//! See docs/specs/unit-spell-target-name.md.

use crate::lua_api::methods::borrow_state;
use crate::lua_bridge::stack_val;
use rilua::table_security::{unwrap_secret, wrap_host_secret_string};
use rilua::vm::state::LuaState;
use rilua::{LuaApiMut, LuaResult, Val};

fn unit_spell_target_name(state: &mut LuaState) -> LuaResult<u32> {
    // AllowedWhenUntainted: decode only after the VM validates the caller.
    let value = unwrap_secret(state, stack_val(state, 1))?;
    let Val::Str(reference) = value else {
        return Err(rilua::runtime_error(
            "UnitSpellTargetName requires a unit token string",
        ));
    };
    let bytes = state
        .gc
        .string_arena
        .get(reference)
        .ok_or_else(|| rilua::runtime_error("UnitSpellTargetName unit token is unavailable"))?
        .data();
    let unit = std::str::from_utf8(bytes)
        .map_err(|_| rilua::runtime_error("UnitSpellTargetName requires a UTF-8 unit token"))?;
    let name = if unit == "player" {
        let sim = borrow_state(state)?;
        sim.casting
            .as_ref()
            .and_then(|cast| cast.target.as_ref())
            .filter(|target| target.is_player)
            .map(|target| target.name.clone())
    } else {
        None
    };
    let result = match name {
        Some(name) => wrap_host_secret_string(state, &name),
        None => Val::Nil,
    };
    // Root the allocated wrapper before any subsequent GC safe point.
    state.push(result);
    Ok(1)
}

fn player_is_spell_target(state: &mut LuaState) -> LuaResult<u32> {
    use crate::lua_bridge::FromStack;
    let unit = String::from_stack(state, 1)?;
    let targeted = {
        let sim = borrow_state(state)?;
        let caster = super::super::unit_misc::existing_guid_for_unit(&sim, &unit);
        let player = super::super::unit_misc::existing_guid_for_unit(&sim, "player");
        // Only the player's active cast is modeled; other casters have no cast
        // record. Compare GUIDs, not the target's broad is_player flag.
        caster.is_some()
            && caster == player
            && sim
                .casting
                .as_ref()
                .and_then(|cast| cast.target.as_ref())
                .is_some_and(|target| Some(target.guid.as_str()) == player.as_deref())
    };
    let value = rilua::table_security::wrap_host_secret_bool(state, targeted);
    state.push(value);
    Ok(1)
}

pub(crate) fn register_all(lua: &mut rilua::Lua) -> LuaResult<()> {
    LuaApiMut::register_function(lua, "PlayerIsSpellTarget", player_is_spell_target)?;
    LuaApiMut::register_function(lua, "UnitSpellTargetName", unit_spell_target_name)
}
