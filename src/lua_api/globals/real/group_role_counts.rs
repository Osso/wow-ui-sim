//! Group role counts projected from represented player and active party roles.
use crate::lua_api::globals::real::unit_roles::read_role;
use crate::lua_api::methods::{borrow_state, create_table, table_set};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

pub(crate) fn get_group_member_counts(state: &mut LuaState) -> LuaResult<u32> {
    let party_count = {
        let sim = borrow_state(state)?;
        if sim.party_group_active && !sim.party_members.is_empty() {
            Some(sim.party_members.len())
        } else {
            None
        }
    };
    let mut counts = [0u32; 4];
    for unit in party_count.into_iter().flat_map(|party_count| {
        std::iter::once("player".to_owned())
            .chain((1..=party_count).map(|index| format!("party{index}")))
    }) {
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
    state.push(table);
    Ok(1)
}
