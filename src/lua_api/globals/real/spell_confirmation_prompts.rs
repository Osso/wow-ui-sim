//! Explicit pending prompts only; lifecycle policy is inferred, not native-verified.
//! See docs/specs/spell-confirmation-prompts.md.

use crate::lua_api::globals::state_backed_queries::dispatch_event_now;
use crate::lua_api::methods::{
    borrow_state, borrow_state_mut, create_string, create_table, table_get_static, table_set_num,
    table_set_static,
};
use crate::lua_bridge::{TableBuilder, stack_val};
use rilua::table_security::check_table_access;
use rilua::vm::state::LuaState;
use rilua::{LuaApiMut, LuaResult, Val, runtime_error};

#[derive(Clone, Debug)]
pub(crate) struct SpellConfirmationPrompt {
    spell_id: f64,
    confirm_type: f64,
    text: String,
    duration: f64,
    currency_id: f64,
    currency_cost: f64,
    difficulty_id: f64,
    display_item_id: f64,
    item_context: f64,
    treasure_context_level: f64,
}

impl SpellConfirmationPrompt {
    fn parse(state: &mut LuaState) -> LuaResult<Self> {
        let record = stack_val(state, 1);
        let Val::Table(table) = record else {
            return Err(runtime_error(
                "QueueSpellConfirmationPrompt requires a record table",
            ));
        };
        check_table_access(state, table, None)?;
        Ok(Self {
            spell_id: read_number(state, record, "spellID")?,
            confirm_type: read_number(state, record, "confirmType")?,
            text: read_text(state, record)?,
            duration: read_number(state, record, "duration")?,
            currency_id: read_number(state, record, "currencyID")?,
            currency_cost: read_number(state, record, "currencyCost")?,
            difficulty_id: read_number(state, record, "difficultyID")?,
            display_item_id: read_number(state, record, "displayItemID")?,
            item_context: read_number(state, record, "itemContext")?,
            treasure_context_level: read_number(state, record, "treasureContextLevel")?,
        })
    }

    fn numeric_fields(&self) -> [(&'static str, f64); 9] {
        [
            ("spellID", self.spell_id),
            ("confirmType", self.confirm_type),
            ("duration", self.duration),
            ("currencyID", self.currency_id),
            ("currencyCost", self.currency_cost),
            ("difficultyID", self.difficulty_id),
            ("displayItemID", self.display_item_id),
            ("itemContext", self.item_context),
            ("treasureContextLevel", self.treasure_context_level),
        ]
    }

    fn event_args(&self, text: Val) -> [Val; 10] {
        // Cached effectValue is confirmType directly, without enum conversion.
        [
            Val::Num(self.spell_id),
            Val::Num(self.confirm_type),
            text,
            Val::Num(self.duration),
            Val::Num(self.currency_id),
            Val::Num(self.currency_cost),
            Val::Num(self.difficulty_id),
            Val::Num(self.display_item_id),
            Val::Num(self.item_context),
            Val::Num(self.treasure_context_level),
        ]
    }
}

fn read_number(state: &mut LuaState, record: Val, field: &'static str) -> LuaResult<f64> {
    require_number(table_get_static(state, record, field), field)
}

fn require_number(value: Val, field: &str) -> LuaResult<f64> {
    match value {
        Val::Num(number) if number.is_finite() => Ok(number),
        _ => Err(runtime_error(format!(
            "spell confirmation {field} requires a finite public number"
        ))),
    }
}

fn read_text(state: &mut LuaState, record: Val) -> LuaResult<String> {
    let Val::Str(reference) = table_get_static(state, record, "text") else {
        return Err(runtime_error(
            "spell confirmation text requires a public string",
        ));
    };
    let string = state
        .gc
        .string_arena
        .get(reference)
        .ok_or_else(|| runtime_error("spell confirmation text is unavailable"))?;
    String::from_utf8(string.data().to_vec())
        .map_err(|_| runtime_error("spell confirmation text requires UTF-8"))
}

/// Finite numeric identity without truncation; Lua equates signed zero.
fn spell_key(spell_id: f64) -> u64 {
    if spell_id == 0.0 {
        0
    } else {
        spell_id.to_bits()
    }
}

pub(crate) fn queue_prompt(state: &mut LuaState) -> LuaResult<u32> {
    let prompt = SpellConfirmationPrompt::parse(state)?;
    let text = create_string(state, &prompt.text);
    let saved_top = state.top;
    state.push(text);
    let args = prompt.event_args(text);
    borrow_state_mut(state)?
        .pending_spell_confirmation_prompts
        .insert(spell_key(prompt.spell_id), prompt);
    let result = dispatch_event_now(state, "SPELL_CONFIRMATION_PROMPT", &args);
    state.top = saved_top;
    result?;
    Ok(0)
}

fn get_prompts(state: &mut LuaState) -> LuaResult<u32> {
    let prompts: Vec<_> = borrow_state(state)?
        .pending_spell_confirmation_prompts
        .values()
        .cloned()
        .collect();
    let sequence = create_table(state);
    state.push(sequence);
    let Val::Table(sequence_ref) = sequence else {
        unreachable!()
    };
    for (index, prompt) in prompts.iter().enumerate() {
        let row = push_prompt_snapshot(state, prompt);
        table_set_num(state, sequence_ref, (index + 1) as f64, row);
        state.top -= 1;
    }
    Ok(1)
}

fn push_prompt_snapshot(state: &mut LuaState, prompt: &SpellConfirmationPrompt) -> Val {
    let row = create_table(state);
    state.push(row);
    for (field, number) in prompt.numeric_fields() {
        table_set_static(state, row, field, Val::Num(number));
    }
    let text = create_string(state, &prompt.text);
    table_set_static(state, row, "text", text);
    row
}

fn remove_prompt(state: &mut LuaState) -> LuaResult<u32> {
    let spell_id = require_number(stack_val(state, 1), "spellID")?;
    borrow_state_mut(state)?
        .pending_spell_confirmation_prompts
        .remove(&spell_key(spell_id));
    Ok(0)
}

pub(crate) fn register_admin(builder: TableBuilder) -> LuaResult<TableBuilder> {
    builder.set_function("QueueSpellConfirmationPrompt", queue_prompt)
}

pub(crate) fn register_all(lua: &mut rilua::Lua) -> LuaResult<()> {
    LuaApiMut::register_function(lua, "GetSpellConfirmationPromptsInfo", get_prompts)?;
    LuaApiMut::register_function(lua, "AcceptSpellConfirmationPrompt", remove_prompt)?;
    LuaApiMut::register_function(lua, "DeclineSpellConfirmationPrompt", remove_prompt)
}
