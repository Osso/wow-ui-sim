//! Retail 12.1.5.70688 R88o.1: a legacy ordinary achievement query.
//! Host observations supplement existing achievement rows; the visible criteria
//! count, earned state, and tracker events never determine this result.

use crate::c_api::parse_numeric_string;
use crate::client_profile::{ACTIVE, ClientProfile};
use crate::lua_api::globals::security::{secure_env_table, set_secure_env_key_state};
use crate::lua_api::methods::borrow_state;
use crate::lua_api::state::{AchievementEligibilityInputs, AchievementEligibilityOperation};
use crate::lua_bridge::stack_val;
use rilua::table_security::{is_secret_value, unwrap_secret};
use rilua::vm::state::LuaState;
use rilua::{LuaApiMut, LuaResult, Val};

const NAME: &str = "IsAchievementEligible";
const USAGE: &str = "Usage: IsAchievementEligible(achievementID)";

pub(crate) fn register(lua: &mut rilua::Lua) -> crate::Result<()> {
    if !published_for_active_profile() {
        return Ok(());
    }
    LuaApiMut::register_function(lua, NAME, get_eligibility)?;
    // Normal bootstrap takes the shared secure table snapshot later.
    // On re-registration, update an already present table using the standard
    // simulator mechanism; never manufacture a shared context.
    if secure_env_table(lua.state_mut()).is_ok() {
        let handler = LuaApiMut::get_global_val(lua, NAME);
        set_secure_env_key_state(lua.state_mut(), NAME, handler)?;
    }
    Ok(())
}

fn published_for_active_profile() -> bool {
    // INFERRED for PTR/Forever: each selected ObjectiveTracker TOC loads
    // Blizzard_AchievementObjectiveTracker.lua, whose zero-visible branch
    // calls this same legacy global. Native verification applies only to Retail.
    matches!(
        ACTIVE,
        ClientProfile::Retail | ClientProfile::Ptr | ClientProfile::WowForever
    )
}

fn get_eligibility(state: &mut LuaState) -> LuaResult<u32> {
    let id = achievement_id(state)?;
    let result = {
        let sim = borrow_state(state)?;
        !sim.achievements.contains_key(&id) || modeled_eligibility(&sim.achievement_eligibility, id)
    };
    state.push(Val::Bool(result));
    Ok(1)
}

fn achievement_id(state: &LuaState) -> LuaResult<i32> {
    let raw = stack_val(state, 1);
    let readable = unwrap_secret(state, raw).ok();
    let number = match readable {
        Some(Val::Num(number)) => Some(number),
        Some(Val::Str(text)) if !is_secret_value(state, raw) => state
            .gc
            .string_arena
            .get(text)
            .and_then(|s| parse_numeric_string(s.data())),
        _ => None,
    }
    .ok_or_else(|| rilua::runtime_error(USAGE))?;
    let truncated = number.trunc();
    if (f64::from(i32::MIN)..=f64::from(i32::MAX)).contains(&truncated) {
        Ok(truncated as i32)
    } else {
        // The observed masked-invalid result has ordinary key semantics.
        Ok(i32::MIN)
    }
}

fn modeled_eligibility(inputs: &AchievementEligibilityInputs, id: i32) -> bool {
    let group = inputs
        .achievement_tree_ids
        .get(&id)
        .and_then(|key| inputs.groups.get(key));
    let Some(group) = group else {
        return true;
    };
    let Some(root) = group else {
        return false;
    };

    use AchievementEligibilityOperation::{All, DefaultTrue, Leaf};
    let mut pending = vec![root];
    while let Some(node) = pending.pop() {
        match inputs.criteria_tree.get(&node.id) {
            None => return false,
            Some(All) => pending.extend(node.children.iter().rev()),
            Some(Leaf { criterion_id }) if !leaf_passes(inputs, *criterion_id) => return false,
            Some(Leaf { .. } | DefaultTrue) => {}
        }
    }
    true
}

fn leaf_passes(inputs: &AchievementEligibilityInputs, id: i32) -> bool {
    let Some(criterion) = inputs.criteria.get(&id) else {
        return true;
    };
    criterion.world_state_id == 0
        || inputs
            .world_states
            .get(&criterion.world_state_id)
            .is_none_or(|value| *value == criterion.required_value)
}
