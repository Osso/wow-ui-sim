use super::ensure_namespace;
use crate::lua_api::methods::{
    borrow_state, borrow_state_mut, create_string, create_table, table_set, table_set_num,
};
use crate::lua_bridge::{FromStack, stack_val, table_set_rust_fn_static};
use rilua::vm::closure::RustFn;
use rilua::vm::gc::arena::GcRef;
use rilua::vm::state::LuaState;
use rilua::vm::table::Table;
use rilua::{LuaResult, Val};

const ADVENTURE_VISIBLE_SUGGESTIONS: usize = 3;
const ADVENTURE_SUGGESTION_CLEAR_LIMIT: usize = 16;

struct AdventureSuggestion {
    title: &'static str,
    description: &'static str,
    button_text: &'static str,
    icon_path: &'static str,
}

const ADVENTURE_SUGGESTIONS: &[AdventureSuggestion] = &[
    AdventureSuggestion {
        title: "Dungeons",
        description: "Queue for current dungeons.",
        button_text: "View",
        icon_path: "Interface\\Icons\\Achievement_Dungeon_UlduarRaid_Titan_01",
    },
    AdventureSuggestion {
        title: "Nerub-ar Palace",
        description: "Study bosses and loot.",
        button_text: "Open",
        icon_path: "Interface\\Icons\\INV_Nerubian_Ring_01_Color5",
    },
    AdventureSuggestion {
        title: "Delves",
        description: "Explore short adventures.",
        button_text: "Start",
        icon_path: "Interface\\Icons\\INV_Misc_ScrollUnrolled03",
    },
    AdventureSuggestion {
        title: "Legacy Raids",
        description: "Browse raid appearances.",
        button_text: "Browse",
        icon_path: "Interface\\Icons\\Achievement_Raid_DragonSoul",
    },
    AdventureSuggestion {
        title: "Collections",
        description: "Track mounts and outfits.",
        button_text: "Open",
        icon_path: "Interface\\Icons\\Achievement_General_StayClassy",
    },
];

const ADVENTURE_JOURNAL_FUNCTIONS: &[(&str, RustFn)] = &[
    ("UpdateSuggestions", adventure_update_suggestions),
    ("GetPrimaryOffset", adventure_get_primary_offset),
    ("SetPrimaryOffset", adventure_set_primary_offset),
    (
        "GetNumAvailableSuggestions",
        adventure_get_num_available_suggestions,
    ),
    ("GetSuggestions", adventure_get_suggestions),
    ("GetReward", adventure_get_reward),
    ("ActivateEntry", adventure_activate_entry),
];

pub(super) fn register_adventure_journal_surface(state: &mut LuaState) -> LuaResult<()> {
    let adventure_table_ref = ensure_namespace(state, "C_AdventureJournal")?;
    for &(name, handler) in ADVENTURE_JOURNAL_FUNCTIONS {
        table_set_rust_fn_static(state, adventure_table_ref, name, handler)?;
    }
    #[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
    table_set_rust_fn_static(
        state,
        adventure_table_ref,
        "CanBeShown",
        adventure_can_be_shown,
    )?;
    Ok(())
}

#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
fn adventure_can_be_shown(state: &mut LuaState) -> LuaResult<u32> {
    let shown = borrow_state(state)?
        .encounter_journal
        .adventure_can_be_shown;
    state.push(Val::Bool(shown));
    Ok(1)
}

fn adventure_update_suggestions(_state: &mut LuaState) -> LuaResult<u32> {
    Ok(0)
}

fn adventure_get_primary_offset(state: &mut LuaState) -> LuaResult<u32> {
    let offset = borrow_state(state)?
        .encounter_journal
        .adventure_primary_offset;
    state.push(Val::Num(offset as f64));
    Ok(1)
}

fn adventure_set_primary_offset(state: &mut LuaState) -> LuaResult<u32> {
    let offset = u32::from_stack(state, 1).unwrap_or(0);
    let max_offset = adventure_max_primary_offset();
    borrow_state_mut(state)?
        .encounter_journal
        .adventure_primary_offset = offset.min(max_offset);
    Ok(0)
}

fn adventure_get_num_available_suggestions(state: &mut LuaState) -> LuaResult<u32> {
    state.push(Val::Num(ADVENTURE_SUGGESTIONS.len() as f64));
    Ok(1)
}

fn adventure_get_suggestions(state: &mut LuaState) -> LuaResult<u32> {
    let Val::Table(output_ref) = stack_val(state, 1) else {
        return Ok(0);
    };
    clear_adventure_suggestions(state, output_ref);

    let offset = borrow_state(state)?
        .encounter_journal
        .adventure_primary_offset
        .min(adventure_max_primary_offset()) as usize;

    for visible_index in 0..ADVENTURE_VISIBLE_SUGGESTIONS {
        let suggestion_index = (offset + visible_index) % ADVENTURE_SUGGESTIONS.len();
        let suggestion = build_adventure_suggestion_table(state, suggestion_index);
        table_set_num(state, output_ref, (visible_index + 1) as f64, suggestion);
    }

    Ok(0)
}

fn adventure_get_reward(state: &mut LuaState) -> LuaResult<u32> {
    state.push(Val::Nil);
    Ok(1)
}

fn adventure_activate_entry(_state: &mut LuaState) -> LuaResult<u32> {
    Ok(0)
}

fn adventure_max_primary_offset() -> u32 {
    ADVENTURE_SUGGESTIONS.len().saturating_sub(1) as u32
}

fn clear_adventure_suggestions(state: &mut LuaState, output_ref: GcRef<Table>) {
    for index in 1..=ADVENTURE_SUGGESTION_CLEAR_LIMIT {
        table_set_num(state, output_ref, index as f64, Val::Nil);
    }
}

fn build_adventure_suggestion_table(state: &mut LuaState, suggestion_index: usize) -> Val {
    let suggestion = &ADVENTURE_SUGGESTIONS[suggestion_index];
    let table = create_table(state);
    let title = create_string(state, suggestion.title);
    let description = create_string(state, suggestion.description);
    let button_text = create_string(state, suggestion.button_text);
    let icon_path = create_string(state, suggestion.icon_path);

    table_set(
        state,
        table,
        "index",
        Val::Num((suggestion_index + 1) as f64),
    );
    table_set(state, table, "title", title);
    table_set(state, table, "description", description);
    table_set(state, table, "buttonText", button_text);
    table_set(state, table, "iconPath", icon_path);
    table
}
