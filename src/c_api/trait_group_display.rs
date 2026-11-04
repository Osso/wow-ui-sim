//! WM101 Forever trait-group display records, supplied in provider order.
use std::collections::HashMap;

use crate::client_profile::{ACTIVE, ClientProfile};
use crate::lua_api::methods::{
    borrow_state, create_string, create_table, table_set, table_set_num,
};
use crate::lua_bridge::{stack_val, table_set_rust_fn_static};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

#[derive(Clone, Debug)]
pub struct GroupDisplayRow {
    pub group_id: i32,
    pub skill_line_id: i32,
    pub order_index: i32,
}

#[derive(Clone, Debug)]
pub struct SkillLineDisplay {
    pub name: String,
    pub icon: u32,
}

#[derive(Clone, Debug, Default)]
pub struct GroupDisplayState {
    pub rows_by_tree: HashMap<i32, Vec<GroupDisplayRow>>,
    pub skill_lines: HashMap<i32, SkillLineDisplay>,
}

pub(crate) fn register(state: &mut LuaState) -> LuaResult<()> {
    if ACTIVE != ClientProfile::WowForever {
        return Ok(());
    }
    // Forever SharedTraitsDocumentation.lua:297-310,1080-1092.
    let namespace = super::ensure_namespace(state, "C_Traits")?;
    table_set_rust_fn_static(
        state,
        namespace,
        "GetGroupDisplayInfoByTreeID",
        get_display_info,
    )
}

fn read_tree_id(state: &LuaState) -> LuaResult<i32> {
    let value = rilua::table_security::unwrap_secret(state, stack_val(state, 1))?;
    let Val::Num(number) = value else {
        return Err(rilua::runtime_error(
            "Usage: local displayInfos = C_Traits.GetGroupDisplayInfoByTreeID(treeID)",
        ));
    };
    if !number.is_finite() || !(f64::from(i32::MIN)..=f64::from(i32::MAX)).contains(&number) {
        return Err(rilua::runtime_error(
            "C_Traits.GetGroupDisplayInfoByTreeID: treeID is outside signed 32-bit range",
        ));
    }
    Ok(number.trunc() as i32)
}

/// INFERRED WM101 seam: absent supplied tree relations are an empty provider.
/// No local Forever TraitNodeGroupDisplayInfo CSV exists; do not borrow Retail rows.
fn inferred_rows_for_tree(model: &GroupDisplayState, tree_id: i32) -> &[GroupDisplayRow] {
    model
        .rows_by_tree
        .get(&tree_id)
        .map(Vec::as_slice)
        .unwrap_or(&[])
}

fn get_display_info(state: &mut LuaState) -> LuaResult<u32> {
    let tree_id = read_tree_id(state)?;
    let records = {
        let sim = borrow_state(state)?;
        let model = &sim.talents.forever.group_display;
        inferred_rows_for_tree(model, tree_id)
            .iter()
            .map(|row| {
                let skill = model.skill_lines.get(&row.skill_line_id);
                (
                    row.clone(),
                    skill.map(|skill| skill.name.clone()).unwrap_or_default(),
                    skill.map_or(0, |skill| skill.icon),
                )
            })
            .collect::<Vec<_>>()
    };
    let result = create_table(state);
    let Val::Table(result_ref) = result else {
        unreachable!("create_table returns a table");
    };
    state.push(result);
    for (index, (row, name, icon)) in records.into_iter().enumerate() {
        let record = create_table(state);
        for (key, value) in [
            ("groupID", row.group_id),
            ("treeID", tree_id),
            ("skillLineID", row.skill_line_id),
            ("orderIndex", row.order_index),
        ] {
            table_set(state, record, key, Val::Num(f64::from(value)));
        }
        let name = create_string(state, &name);
        table_set(state, record, "displayName", name);
        table_set(state, record, "icon", Val::Num(f64::from(icon)));
        table_set_num(state, result_ref, (index + 1) as f64, record);
    }
    Ok(1)
}
