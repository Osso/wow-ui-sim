//! Server-supplied quest reward spell candidates and current detail slots.

use crate::lua_api::methods::{borrow_state, create_table};
use crate::lua_bridge::{stack_val, table_set_rust_fn_static};
use rilua::table_security::unwrap_secret;
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

pub(crate) fn register(state: &mut LuaState) -> LuaResult<()> {
    if !matches!(
        crate::client_profile::ACTIVE,
        crate::client_profile::ClientProfile::Retail | crate::client_profile::ClientProfile::Ptr
    ) {
        return Ok(());
    }
    let namespace = super::ensure_namespace(state, "C_QuestInfoSystem")?;
    // Retail QuestInfoSystemDocumentation.lua:90-104.
    table_set_rust_fn_static(state, namespace, "GetQuestRewardSpells", get_reward_spells)
}

fn get_reward_spells(state: &mut LuaState) -> LuaResult<u32> {
    let input = unwrap_secret(state, stack_val(state, 1))?;
    let explicit = match input {
        Val::Nil => None,
        Val::Num(number)
            if number.is_finite()
                && number.fract() == 0.0
                && (0.0..=f64::from(u32::MAX)).contains(&number) =>
        {
            Some(number as u32)
        }
        _ => {
            return Err(rilua::runtime_error(
                "GetQuestRewardSpells: questID must be a finite integral u32 or nil",
            ));
        }
    };
    let rewards = {
        let sim = borrow_state(state)?;
        let quest = explicit.or(sim.selected_quest_log_id);
        if let Some(candidates) = quest.and_then(|id| sim.quest_reward_spell_cache.get(&id)) {
            let mut selected = Vec::new();
            for &id in candidates {
                if id != 0 && !selected.contains(&id) {
                    selected.push(id);
                    if selected.len() == 3 {
                        break;
                    }
                }
            }
            Some(selected)
        } else {
            sim.quest_detail_reward_spells
                .as_ref()
                .filter(|(id, _)| Some(*id) == explicit)
                .map(|(_, slots)| slots.iter().copied().filter(|id| *id != 0).collect())
        }
    };
    let Some(rewards) = rewards else { return Ok(0) };
    let table = create_table(state);
    let Val::Table(table_ref) = table else {
        return Ok(0);
    };
    for (index, id) in rewards.iter().enumerate() {
        if let Some(table) = state.gc.tables.get_mut(table_ref) {
            let _ = table.raw_set(
                Val::Num((index + 1) as f64),
                Val::Num(f64::from(*id)),
                &state.gc.string_arena,
            );
        }
        state.gc.barrier_back(table_ref);
    }
    state.push(table);
    Ok(1)
}
