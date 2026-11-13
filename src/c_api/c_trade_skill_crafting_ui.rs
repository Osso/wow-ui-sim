use crate::lua_api::methods::borrow_state;
use crate::lua_bridge::table_set_rust_fn_static;
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

const CRAFTING_UI_EFFECT: u32 = 47;
const FUNCTION_NAME: &str = "C_TradeSkillUI.CanTradeSkillShowCraftingUI";
const USAGE: &str =
    "Usage: local canShow = C_TradeSkillUI.CanTradeSkillShowCraftingUI(tradeSkillSpellID)";

#[derive(Debug, Default)]
pub struct TradeSkillCraftingUiInputs {
    /// Supplied non-null result from the opaque helper; the missing-input default is false.
    pub opaque_helper_non_null: bool,
}

pub(crate) fn register(state: &mut LuaState) -> LuaResult<()> {
    if crate::client_profile::ACTIVE != crate::client_profile::ClientProfile::Ptr {
        return Ok(());
    }
    let namespace = super::ensure_namespace(state, "C_TradeSkillUI")?;
    table_set_rust_fn_static(
        state,
        namespace,
        "CanTradeSkillShowCraftingUI",
        can_trade_skill_show_crafting_ui,
    )
}

fn can_trade_skill_show_crafting_ui(state: &mut LuaState) -> LuaResult<u32> {
    let spell_id =
        super::helpers::read_untainted_signed32_argument(state, 1, FUNCTION_NAME, USAGE)?;
    let helper_non_null = borrow_state(state)?
        .trade_skill_crafting_ui_inputs
        .opaque_helper_non_null;
    let can_show = helper_non_null
        && u32::try_from(spell_id).ok().is_some_and(|spell_id| {
            crate::spells::get_spell(spell_id).is_some()
                && sim_data::spell_effects::spell_effect_0(spell_id) == Some(CRAFTING_UI_EFFECT)
        });
    state.push(Val::Bool(can_show));
    Ok(1)
}
