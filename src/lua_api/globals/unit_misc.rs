//! Misc unit globals that do not fit the core group-query bucket.

#[cfg(not(all(
    feature = "retail-12-0-5",
    any(feature = "profile-retail", feature = "client-ptr")
)))]
use crate::lua_api::globals::security::secret_values::mark_secret_value;
use crate::lua_api::methods::{borrow_state, create_string, create_string_static, create_table};
use crate::lua_api::state::SEEDED_LOCAL_CHARACTER_GUID;
use crate::lua_bridge::FromStack;
use rilua::vm::state::LuaState;
use rilua::{LuaApiMut, LuaResult, Val};

const SIM_REALM: &str = "SimRealm";
const UNKNOWN_CREATURE_GUID: &str = "Creature-0000-00000000";

fn unit_name_for(state: &mut LuaState, unit: &str) -> String {
    let Ok(sim) = borrow_state(state) else {
        return "Unknown".to_string();
    };
    match unit {
        "player" | "pet" | "vehicle" => sim.player.name.clone(),
        "target" => sim
            .current_target
            .as_ref()
            .map(|target| target.name.clone())
            .unwrap_or_else(|| "Unknown".to_string()),
        "focus" => sim
            .current_focus
            .as_ref()
            .map(|target| target.name.clone())
            .unwrap_or_else(|| "Unknown".to_string()),
        other => {
            if let Some(idx) = crate::lua_api::globals::unit_api::parse_party_index(other) {
                sim.party_members
                    .get(idx)
                    .map(|member| member.name.clone())
                    .unwrap_or_else(|| "Unknown".to_string())
            } else {
                "Unknown".to_string()
            }
        }
    }
}

#[cfg(all(
    feature = "retail-12-0-5",
    any(feature = "profile-retail", feature = "client-ptr")
))]
pub(crate) fn unit_identity_is_secret(state: &mut LuaState, unit: &str) -> LuaResult<bool> {
    let sim = borrow_state(state)?;
    Ok(unit_identity_is_secret_in_state(&sim, unit))
}

#[cfg(not(all(
    feature = "retail-12-0-5",
    any(feature = "profile-retail", feature = "client-ptr")
)))]
pub(crate) fn unit_identity_is_secret(_state: &mut LuaState, unit: &str) -> LuaResult<bool> {
    Ok(crate::lua_api::globals::unit_api::parse_party_index(unit).is_some())
}

pub(crate) fn identity_output(state: &mut LuaState, text: &str, secret: bool) -> Val {
    #[cfg(all(
        feature = "retail-12-0-5",
        any(feature = "profile-retail", feature = "client-ptr")
    ))]
    if secret {
        return rilua::table_security::wrap_host_secret_string(state, text);
    }
    let value = create_string(state, text);
    #[cfg(not(all(
        feature = "retail-12-0-5",
        any(feature = "profile-retail", feature = "client-ptr")
    )))]
    if secret {
        mark_secret_value(state, value);
    }
    value
}

fn unit_name_string(state: &mut LuaState) -> LuaResult<u32> {
    let unit = Option::<String>::from_stack(state, 1)?.unwrap_or_default();
    let name = unit_name_for(state, &unit);
    let secret = unit_identity_is_secret(state, &unit)?;
    let name = identity_output(state, &name, secret);
    state.push(name);
    Ok(1)
}

fn get_unit_name(state: &mut LuaState) -> LuaResult<u32> {
    unit_name_string(state)
}

fn unit_pvp_name(state: &mut LuaState) -> LuaResult<u32> {
    unit_name_string(state)
}

fn unit_full_name(state: &mut LuaState) -> LuaResult<u32> {
    let unit = Option::<String>::from_stack(state, 1)?.unwrap_or_default();
    let name = unit_name_for(state, &unit);
    let secret = unit_identity_is_secret(state, &unit)?;
    let name = identity_output(state, &name, secret);
    state.push(name);
    let realm = identity_output(state, SIM_REALM, secret);
    state.push(realm);
    Ok(2)
}

fn unit_class_base(state: &mut LuaState) -> LuaResult<u32> {
    let unit = Option::<String>::from_stack(state, 1)?.unwrap_or_default();
    let class_index = {
        let Ok(sim) = borrow_state(state) else {
            state.push(Val::Nil);
            return Ok(1);
        };
        match unit.as_str() {
            "player" | "pet" | "vehicle" => sim.player.class_index,
            "target" => sim
                .current_target
                .as_ref()
                .map(|target| target.class_index)
                .unwrap_or(0),
            "focus" => sim
                .current_focus
                .as_ref()
                .map(|target| target.class_index)
                .unwrap_or(0),
            other => {
                if crate::lua_api::globals::unit_api::parse_party_index(other).is_some() {
                    1
                } else {
                    0
                }
            }
        }
    };
    let (_, class_file, _) = crate::lua_api::game_data::class_info_by_index(class_index);
    let class_file = create_string_static(state, class_file);
    state.push(class_file);
    Ok(1)
}

fn unit_guid(state: &mut LuaState) -> LuaResult<u32> {
    let unit = Option::<String>::from_stack(state, 1)?.unwrap_or_default();
    let guid = {
        let Ok(sim) = borrow_state(state) else {
            return Ok(0);
        };
        existing_guid_for_unit(&sim, &unit)
    };
    #[cfg(all(
        feature = "retail-12-0-5",
        any(feature = "profile-retail", feature = "client-ptr")
    ))]
    let secret = unit_identity_is_secret(state, &unit)?;
    #[cfg(not(all(
        feature = "retail-12-0-5",
        any(feature = "profile-retail", feature = "client-ptr")
    )))]
    let secret = false;
    let result = guid
        .map(|guid| identity_output(state, &guid, secret))
        .unwrap_or(Val::Nil);
    state.push(result);
    Ok(1)
}

fn unit_creature_id(state: &mut LuaState) -> LuaResult<u32> {
    let unit = Option::<String>::from_stack(state, 1)?.unwrap_or_default();
    let creature_id = {
        let Ok(sim) = borrow_state(state) else {
            return Ok(0);
        };
        crate::c_api::c_creature_info::creature_id_from_guid(&guid_for_unit(&sim, &unit))
    };
    match creature_id {
        Some(id) => state.push(Val::Num(id as f64)),
        None => state.push(Val::Nil),
    }
    Ok(1)
}

#[cfg(all(
    feature = "retail-12-0-5",
    any(feature = "profile-retail", feature = "client-ptr")
))]
pub(crate) fn unit_identity_is_secret_in_state(
    sim: &crate::lua_api::state::SimState,
    unit: &str,
) -> bool {
    super::real::instanced_identity::identity_is_secret(sim, unit)
}

pub(crate) fn existing_guid_for_unit(
    sim: &crate::lua_api::state::SimState,
    unit: &str,
) -> Option<String> {
    if !super::group_queries::unit_exists_in_state(sim, unit) {
        return None;
    }
    let guid = guid_for_unit(sim, unit);
    (guid != UNKNOWN_CREATURE_GUID && !guid.is_empty()).then_some(guid)
}

pub(crate) fn guid_for_unit(sim: &crate::lua_api::state::SimState, unit: &str) -> String {
    match unit {
        "player" => SEEDED_LOCAL_CHARACTER_GUID.to_string(),
        "target" => target_guid(sim),
        "focus" => focus_guid(sim),
        other => party_guid_for_unit(other).unwrap_or_else(|| UNKNOWN_CREATURE_GUID.to_string()),
    }
}

fn target_guid(sim: &crate::lua_api::state::SimState) -> String {
    sim.current_target
        .as_ref()
        .map(|target| target.guid.clone())
        .unwrap_or_else(|| UNKNOWN_CREATURE_GUID.to_string())
}

fn focus_guid(sim: &crate::lua_api::state::SimState) -> String {
    sim.current_focus
        .as_ref()
        .map(|target| target.guid.clone())
        .unwrap_or_else(|| UNKNOWN_CREATURE_GUID.to_string())
}

pub(crate) fn party_guid_for_index(index: usize) -> String {
    format!("Player-0000-000000{:02}", index + 2)
}

fn party_guid_for_unit(unit: &str) -> Option<String> {
    crate::lua_api::globals::unit_api::parse_party_index(unit).map(party_guid_for_index)
}

fn target_or_focus_token_from_guid(
    sim: &crate::lua_api::state::SimState,
    guid: &str,
) -> Option<&'static str> {
    if sim
        .current_target
        .as_ref()
        .is_some_and(|target| target.guid == guid)
    {
        return Some("target");
    }
    if sim
        .current_focus
        .as_ref()
        .is_some_and(|target| target.guid == guid)
    {
        return Some("focus");
    }
    None
}

fn party_token_from_guid(sim: &crate::lua_api::state::SimState, guid: &str) -> Option<String> {
    if !sim.party_group_active {
        return None;
    }
    #[cfg(all(
        feature = "retail-12-0-5",
        any(feature = "profile-retail", feature = "client-ptr")
    ))]
    if sim.identity_secret_guids.contains(guid) {
        return None;
    }
    sim.party_members.iter().enumerate().find_map(|(idx, _)| {
        let party_guid = format!("Player-0000-000000{:02}", idx + 2);
        (party_guid == guid).then(|| format!("party{}", idx + 1))
    })
}

fn unit_token_from_guid(state: &mut LuaState) -> LuaResult<u32> {
    let guid = Option::<String>::from_stack(state, 1)?.unwrap_or_default();
    let token = {
        let Ok(sim) = borrow_state(state) else {
            state.push(Val::Nil);
            return Ok(1);
        };
        if guid == SEEDED_LOCAL_CHARACTER_GUID {
            Some("player".to_string())
        } else {
            target_or_focus_token_from_guid(&sim, &guid)
                .map(str::to_string)
                .or_else(|| party_token_from_guid(&sim, &guid))
        }
    };
    match token {
        Some(token) => {
            let token = create_string(state, &token);
            state.push(token);
        }
        None => state.push(Val::Nil),
    }
    Ok(1)
}

fn unit_creature_family(state: &mut LuaState) -> LuaResult<u32> {
    let _ = Option::<String>::from_stack(state, 1)?;
    state.push(Val::Nil);
    Ok(1)
}

fn unit_player_controlled(state: &mut LuaState) -> LuaResult<u32> {
    let unit = Option::<String>::from_stack(state, 1)?.unwrap_or_default();
    let controlled = match unit.as_str() {
        "player" | "pet" | "vehicle" => true,
        "target" => borrow_state(state)?
            .current_target
            .as_ref()
            .is_some_and(|target| target.is_player),
        "focus" => borrow_state(state)?
            .current_focus
            .as_ref()
            .is_some_and(|target| target.is_player),
        other => crate::lua_api::globals::unit_api::parse_party_index(other).is_some(),
    };
    state.push(Val::Bool(controlled));
    Ok(1)
}

fn unit_is_afk(state: &mut LuaState) -> LuaResult<u32> {
    let _ = Option::<String>::from_stack(state, 1)?;
    state.push(Val::Bool(false));
    Ok(1)
}

fn unit_is_dnd(state: &mut LuaState) -> LuaResult<u32> {
    let _ = Option::<String>::from_stack(state, 1)?;
    state.push(Val::Bool(false));
    Ok(1)
}

fn comparison_unit_token(state: &LuaState, index: i32) -> LuaResult<Option<String>> {
    #[cfg(feature = "retail-12-0-5")]
    {
        let input = crate::lua_bridge::stack_val(state, index);
        if rilua::table_security::is_secret_value(state, input) {
            // VM-owned wrappers only; unwrap_secret enforces an untainted caller.
            let payload = rilua::table_security::unwrap_secret(state, input)?;
            let Val::Str(reference) = payload else {
                return Err(rilua::runtime_error(
                    "UnitIsUnit requires a secret string token",
                ));
            };
            let bytes = state.gc.string_arena.get(reference).ok_or_else(|| {
                rilua::runtime_error(format!("string at argument {index} has been collected"))
            })?;
            let token = std::str::from_utf8(bytes.data()).map_err(|_| {
                rilua::runtime_error(format!("string at argument {index} is not valid UTF-8"))
            })?;
            return Ok(Some(token.to_owned()));
        }
    }
    Option::<String>::from_stack(state, index)
}

#[cfg(feature = "retail-12-0-5")]
fn is_base_comparison_token(unit: &str) -> bool {
    matches!(
        unit,
        "player"
            | "pet"
            | "vehicle"
            | "mouseover"
            | "target"
            | "softenemy"
            | "softfriend"
            | "softinteract"
            | "focus"
            | "none"
            | "npc"
            | "questnpc"
    )
}

#[cfg(feature = "retail-12-0-5")]
fn is_group_comparison_token(unit: &str) -> bool {
    const GROUP_TOKEN_LIMITS: [(&str, u8); 4] =
        [("party", 4), ("partypet", 4), ("raid", 40), ("raidpet", 40)];
    GROUP_TOKEN_LIMITS.iter().any(|(prefix, limit)| {
        let Some(suffix) = unit.strip_prefix(prefix) else {
            return false;
        };
        let Ok(index) = suffix.parse::<u8>() else {
            return false;
        };
        let canonical = suffix == index.to_string();
        canonical && (1..=*limit).contains(&index)
    })
}

#[cfg(feature = "retail-12-0-5")]
fn is_restricted_comparison_counterpart(unit: &str) -> bool {
    unit.starts_with("nameplate") || unit.ends_with("target")
}

#[cfg(feature = "retail-12-0-5")]
fn unit_comparison_permitted(lhs: Option<&str>, rhs: Option<&str>) -> bool {
    let (Some(lhs), Some(rhs)) = (lhs, rhs) else {
        // Inferred compatibility policy: missing tokens retain false, not denial.
        return true;
    };
    if is_base_comparison_token(lhs) || is_base_comparison_token(rhs) {
        return true;
    }
    let lhs_allows = is_group_comparison_token(lhs) && !is_restricted_comparison_counterpart(rhs);
    let rhs_allows = is_group_comparison_token(rhs) && !is_restricted_comparison_counterpart(lhs);
    lhs_allows || rhs_allows
}

fn unit_is_unit(state: &mut LuaState) -> LuaResult<u32> {
    let lhs = comparison_unit_token(state, 1)?;
    let rhs = comparison_unit_token(state, 2)?;
    #[cfg(feature = "retail-12-0-5")]
    if !unit_comparison_permitted(lhs.as_deref(), rhs.as_deref()) {
        return Ok(0);
    }
    let lhs = lhs.unwrap_or_default();
    let rhs = rhs.unwrap_or_default();
    let same_unit = {
        let sim = borrow_state(state)?;
        match (
            existing_guid_for_unit(&sim, &lhs),
            existing_guid_for_unit(&sim, &rhs),
        ) {
            (Some(lhs), Some(rhs)) => lhs == rhs,
            _ => false,
        }
    };
    state.push(Val::Bool(same_unit));
    Ok(1)
}

fn unit_threat_situation(state: &mut LuaState) -> LuaResult<u32> {
    let _ = Option::<String>::from_stack(state, 1)?;
    let _ = Option::<String>::from_stack(state, 2)?;
    state.push(Val::Nil);
    Ok(1)
}

fn unit_affecting_combat(state: &mut LuaState) -> LuaResult<u32> {
    let unit = Option::<String>::from_stack(state, 1)?.unwrap_or_default();
    let in_combat = matches!(unit.as_str(), "player" | "pet" | "vehicle")
        && borrow_state(state)?.player.in_combat;
    state.push(Val::Bool(in_combat));
    Ok(1)
}

fn unit_is_feign_death(state: &mut LuaState) -> LuaResult<u32> {
    let unit = Option::<String>::from_stack(state, 1)?.unwrap_or_default();
    let feigning = unit == "player"
        && borrow_state(state)?
            .world
            .mirror_timers
            .iter()
            .any(|timer| timer.name.eq_ignore_ascii_case("FEIGNDEATH"));
    state.push(Val::Bool(feigning));
    Ok(1)
}

fn get_corruption(state: &mut LuaState) -> LuaResult<u32> {
    state.push(Val::Num(0.0));
    Ok(1)
}

fn get_corruption_resistance(state: &mut LuaState) -> LuaResult<u32> {
    state.push(Val::Num(0.0));
    Ok(1)
}

fn get_negative_corruption_effect_info(state: &mut LuaState) -> LuaResult<u32> {
    let effects = create_table(state);
    state.push(effects);
    Ok(1)
}

pub fn register_all(lua: &mut rilua::Lua) -> crate::Result<()> {
    LuaApiMut::register_function(lua, "GetUnitName", get_unit_name)?;
    LuaApiMut::register_function(lua, "UnitPVPName", unit_pvp_name)?;
    LuaApiMut::register_function(lua, "UnitFullName", unit_full_name)?;
    LuaApiMut::register_function(lua, "UnitClassBase", unit_class_base)?;
    LuaApiMut::register_function(lua, "UnitGUID", unit_guid)?;
    LuaApiMut::register_function(lua, "UnitCreatureID", unit_creature_id)?;
    LuaApiMut::register_function(lua, "UnitTokenFromGUID", unit_token_from_guid)?;
    LuaApiMut::register_function(lua, "UnitCreatureFamily", unit_creature_family)?;
    LuaApiMut::register_function(lua, "UnitPlayerControlled", unit_player_controlled)?;
    LuaApiMut::register_function(lua, "UnitIsAFK", unit_is_afk)?;
    LuaApiMut::register_function(lua, "UnitIsDND", unit_is_dnd)?;
    LuaApiMut::register_function(lua, "UnitIsUnit", unit_is_unit)?;
    LuaApiMut::register_function(lua, "UnitThreatSituation", unit_threat_situation)?;
    LuaApiMut::register_function(lua, "UnitAffectingCombat", unit_affecting_combat)?;
    LuaApiMut::register_function(lua, "UnitIsFeignDeath", unit_is_feign_death)?;
    LuaApiMut::register_function(lua, "GetCorruption", get_corruption)?;
    LuaApiMut::register_function(lua, "GetCorruptionResistance", get_corruption_resistance)?;
    LuaApiMut::register_function(
        lua,
        "GetNegativeCorruptionEffectInfo",
        get_negative_corruption_effect_info,
    )?;
    Ok(())
}
