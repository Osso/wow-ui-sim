//! Retail group-member counts use host-supplied native roster fields.
use crate::client_profile::{ACTIVE, ClientProfile};
use crate::lua_api::game_data::{CLASS_FILES, CLASS_LABELS};
use crate::lua_api::globals::real::unit_roles::read_role;
use crate::lua_api::methods::{borrow_state, create_table, table_set};
use crate::lua_bridge::stack_val;
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

const MEMBER_FLAG_SKIP_COUNTS: u32 = 16;
const ROLE_TANK: u32 = 2;
const ROLE_HEALER: u32 = 4;
const ROLE_DAMAGER: u32 = 8;
const MAX_GROUP_MEMBERS: usize = 40;
const ROLE_COUNT_KEYS: [&str; 4] = ["TANK", "HEALER", "DAMAGER", "NOROLE"];

pub(crate) fn get_group_member_counts(state: &mut LuaState) -> LuaResult<u32> {
    if ACTIVE == ClientProfile::Retail {
        return get_retail_group_member_counts(state);
    }
    get_simulator_group_member_counts(state)
}

fn get_retail_group_member_counts(state: &mut LuaState) -> LuaResult<u32> {
    let argument = stack_val(state, 1);
    let selector = if matches!(argument, Val::Table(_)) {
        None
    } else {
        requested_container(state)
    };
    let table = match argument {
        value @ Val::Table(_) => value,
        _ => create_table(state),
    };
    let (roles, classes) = {
        let sim = borrow_state(state)?;
        let members = select_container(&sim.group_member_counts, selector);
        count_retail_members(members)
    };

    for (key, count) in ROLE_COUNT_KEYS.into_iter().zip(roles) {
        table_set(state, table, key, Val::Num(f64::from(count)));
    }
    for (index, token) in CLASS_FILES.iter().enumerate() {
        table_set(state, table, token, Val::Num(f64::from(classes[index + 1])));
    }
    state.push(table);
    Ok(1)
}

fn requested_container(state: &LuaState) -> Option<usize> {
    let value = match stack_val(state, 1) {
        Val::Num(number) => Some(number),
        Val::Str(reference) => state
            .gc
            .string_arena
            .get(reference)
            .and_then(|string| crate::c_api::helpers::parse_numeric_string(string.data())),
        _ => None,
    }?;
    match value.trunc() {
        1.0 => Some(0),
        2.0 => Some(1),
        _ => None,
    }
}

fn select_container(
    state: &crate::lua_api::state::GroupMemberCountsState,
    selector: Option<usize>,
) -> Option<&[Option<crate::lua_api::state::GroupMemberCountsMember>]> {
    match selector {
        Some(0) => state.container_one.as_deref(),
        Some(1) => state.container_two.as_deref(),
        None => state
            .container_two
            .as_deref()
            .or_else(|| state.container_one.as_deref()),
        _ => unreachable!("container selector is limited to 1 or 2"),
    }
}

fn count_retail_members(
    members: Option<&[Option<crate::lua_api::state::GroupMemberCountsMember>]>,
) -> ([u32; 4], Vec<u32>) {
    let mut roles = [0; ROLE_COUNT_KEYS.len()];
    let mut classes = vec![0; CLASS_FILES.len() + 1];
    let Some(members) = members else {
        return (roles, classes);
    };

    for member in members.iter().take(MAX_GROUP_MEMBERS) {
        let Some(member) = member else {
            break;
        };
        if member.flags & MEMBER_FLAG_SKIP_COUNTS != 0 {
            continue;
        }
        let class_id = resolve_class_id(member.resolved_class_id, member.cached_class_id);
        if class_id < 0 || class_id as usize >= classes.len() {
            continue;
        }
        classes[class_id as usize] += 1;
        roles[role_index(member.role_mask_a, member.role_mask_b)] += 1;
    }
    (roles, classes)
}

fn resolve_class_id(resolved_class_id: Option<i32>, cached_class_id: i32) -> i32 {
    match resolved_class_id {
        Some(class_id) => class_id,
        None if (1..=CLASS_FILES.len() as i32).contains(&cached_class_id) => cached_class_id,
        None => 1,
    }
}

fn role_index(role_mask_a: u32, role_mask_b: u32) -> usize {
    let tank_from_a = role_mask_a & ROLE_TANK != 0;
    let tank_from_b = role_mask_b & ROLE_TANK != 0
        && role_mask_b & ROLE_DAMAGER == 0
        && role_mask_a & ROLE_DAMAGER != 0;
    if tank_from_a || tank_from_b {
        return 0;
    }
    if role_mask_a & ROLE_HEALER != 0 {
        return 1;
    }
    if role_mask_a & ROLE_DAMAGER != 0 && role_mask_b & (ROLE_TANK | ROLE_DAMAGER) != ROLE_TANK {
        return 2;
    }
    3
}

fn get_simulator_group_member_counts(state: &mut LuaState) -> LuaResult<u32> {
    let members = {
        let sim = borrow_state(state)?;
        if sim.party_group_active && !sim.party_members.is_empty() {
            std::iter::once(("player".to_owned(), sim.player.class_index))
                .chain(
                    sim.party_members
                        .iter()
                        .enumerate()
                        .map(|(index, member)| (format!("party{}", index + 1), member.class_index)),
                )
                .take(MAX_GROUP_MEMBERS)
                .collect::<Vec<_>>()
        } else {
            Vec::new()
        }
    };
    let mut counts = [0u32; ROLE_COUNT_KEYS.len()];
    let mut classes = vec![0u32; CLASS_LABELS.len()];
    for (unit, class_id) in members {
        let class = class_id
            .checked_sub(1)
            .and_then(|id| usize::try_from(id).ok())
            .filter(|&index| index < classes.len())
            .unwrap_or(0);
        classes[class] += 1;
        let role = read_role(state, &unit)?.0;
        let index = match role {
            "TANK" => 0,
            "HEALER" => 1,
            "DAMAGER" => 2,
            _ => 3,
        };
        counts[index] += 1;
    }
    let table = create_table(state);
    for (name, count) in ROLE_COUNT_KEYS.into_iter().zip(counts) {
        table_set(state, table, name, Val::Num(f64::from(count)));
    }
    for (name, count) in CLASS_LABELS.iter().zip(classes) {
        table_set(state, table, name, Val::Num(f64::from(count)));
    }
    state.push(table);
    Ok(1)
}
