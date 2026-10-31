//! Group role counts projected from represented player and active party roles.
use crate::lua_api::game_data::CLASS_LABELS;
use crate::lua_api::globals::real::unit_roles::read_role;
use crate::lua_api::methods::{borrow_state, create_table, table_set};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

pub(crate) fn get_group_member_counts(state: &mut LuaState) -> LuaResult<u32> {
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
                .take(40)
                .collect::<Vec<_>>()
        } else {
            Vec::new()
        }
    };
    let mut counts = [0u32; 4];
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
    for (name, count) in ["TANK", "HEALER", "DAMAGER", "NOROLE"]
        .into_iter()
        .zip(counts)
    {
        table_set(state, table, name, Val::Num(count as f64));
    }
    for (name, count) in CLASS_LABELS.iter().zip(classes) {
        table_set(state, table, name, Val::Num(count as f64));
    }
    state.push(table);
    Ok(1)
}
