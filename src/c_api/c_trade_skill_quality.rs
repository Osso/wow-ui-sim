//! Recipe quality snapshots supplied by the host; no DB2 quality catalog inferred.
use super::item_spell::parse_item_id_from_val;
use crate::client_profile::{ACTIVE, ClientProfile};
use crate::lua_api::methods::{borrow_state, create_string, create_table, table_set};
use crate::lua_bridge::{stack_val, table_set_rust_fn_static};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};
use std::collections::HashMap;

#[derive(Debug, Clone, Default)]
pub struct CraftingQualityInfo {
    pub quality: u32,
    pub icon: String,
    pub icon_small: String,
    pub icon_inventory: String,
    pub icon_mixed: String,
    pub icon_appear: String,
    pub icon_dissolve: String,
    pub bar_fill: String,
    pub bar_background: String,
    pub bar_background_cap: String,
    pub bar_highlight: String,
    pub icon_chat: String,
    pub icon_quest_objective: String,
}

#[derive(Debug, Default)]
pub struct RecipeQualityInputs {
    pub item_quality: HashMap<(u32, u32), CraftingQualityInfo>,
    pub reagent_item_quality: HashMap<u32, CraftingQualityInfo>,
    pub reagent_links: HashMap<(u32, u32, u32), String>,
}

pub(super) fn register(state: &mut LuaState) -> LuaResult<()> {
    if !matches!(
        ACTIVE,
        ClientProfile::Retail
            | ClientProfile::Ptr
            | ClientProfile::WowForever
            | ClientProfile::Mists
    ) {
        return Ok(());
    }
    let ns = super::ensure_namespace(state, "C_TradeSkillUI")?;
    if matches!(ACTIVE, ClientProfile::Retail | ClientProfile::Ptr) {
        table_set_rust_fn_static(state, ns, "GetRecipeItemQualityInfo", item_quality)?;
        table_set_rust_fn_static(state, ns, "GetRecipeQualityReagentLink", reagent_link)?;
        table_set_rust_fn_static(state, ns, "GetItemCraftedQualityInfo", crafted_item_quality)?;
    }
    table_set_rust_fn_static(state, ns, "GetItemReagentQualityInfo", reagent_item_quality)?;
    Ok(())
}

fn read_integer(state: &LuaState, index: i32) -> LuaResult<u32> {
    let value = rilua::table_security::unwrap_secret(state, stack_val(state, index))?;
    match value {
        Val::Num(n) if n.is_finite() && n.fract() == 0.0 && n > 0.0 && n <= u32::MAX as f64 => {
            Ok(n as u32)
        }
        _ => Err(rilua::runtime_error(format!(
            "argument {index} must be a positive integer"
        ))),
    }
}

fn item_quality(state: &mut LuaState) -> LuaResult<u32> {
    let key = (read_integer(state, 1)?, read_integer(state, 2)?);
    let record = borrow_state(state)?
        .recipe_quality_inputs
        .item_quality
        .get(&key)
        .cloned();
    let Some(record) = record else {
        state.push(Val::Nil);
        return Ok(1);
    };
    Ok(push_crafting_quality_snapshot(state, &record))
}

fn crafted_item_quality(state: &mut LuaState) -> LuaResult<u32> {
    let value = rilua::table_security::unwrap_secret(state, stack_val(state, 1))?;
    if !matches!(value, Val::Num(_) | Val::Str(_)) {
        return Err(rilua::runtime_error(
            "C_TradeSkillUI.GetItemCraftedQualityInfo: itemInfo must be an item ID or link",
        ));
    }
    let record = match parse_item_id_from_val(state, value) {
        Some(item_id) => borrow_state(state)?
            .crafted_item_quality
            .get(&item_id)
            .cloned(),
        None => None,
    };
    let Some(record) = record else {
        state.push(Val::Nil);
        return Ok(1);
    };
    Ok(push_crafting_quality_snapshot(state, &record))
}

fn reagent_item_quality(state: &mut LuaState) -> LuaResult<u32> {
    let value = rilua::table_security::unwrap_secret(state, stack_val(state, 1))?;
    if ACTIVE == ClientProfile::Mists && !matches!(value, Val::Num(_)) {
        return Err(rilua::runtime_error(
            "C_TradeSkillUI.GetItemReagentQualityInfo: itemID must be a number",
        ));
    }
    if !matches!(value, Val::Num(_) | Val::Str(_)) {
        return Err(rilua::runtime_error(
            "C_TradeSkillUI.GetItemReagentQualityInfo: itemInfo must be an item ID or link",
        ));
    }
    let record = match parse_item_id_from_val(state, value) {
        Some(item_id) => borrow_state(state)?
            .recipe_quality_inputs
            .reagent_item_quality
            .get(&item_id)
            .cloned(),
        None => None,
    };
    let Some(record) = record else {
        state.push(Val::Nil);
        return Ok(1);
    };
    Ok(push_crafting_quality_snapshot(state, &record))
}

fn push_crafting_quality_snapshot(state: &mut LuaState, record: &CraftingQualityInfo) -> u32 {
    let result = create_table(state);
    state.push(result);
    table_set(
        state,
        result,
        "quality",
        Val::Num(f64::from(record.quality)),
    );
    for (key, text) in [
        ("icon", &record.icon),
        ("iconSmall", &record.icon_small),
        ("iconInventory", &record.icon_inventory),
        ("iconMixed", &record.icon_mixed),
        ("iconAppear", &record.icon_appear),
        ("iconDissolve", &record.icon_dissolve),
        ("barFill", &record.bar_fill),
        ("barBackground", &record.bar_background),
        ("barBackgroundCap", &record.bar_background_cap),
        ("barHighlight", &record.bar_highlight),
        ("iconChat", &record.icon_chat),
    ] {
        let value = create_string(state, text);
        table_set(state, result, key, value);
    }
    if ACTIVE != ClientProfile::Mists {
        let value = create_string(state, &record.icon_quest_objective);
        table_set(state, result, "iconQuestObjective", value);
    }
    1
}

fn reagent_link(state: &mut LuaState) -> LuaResult<u32> {
    let key = (
        read_integer(state, 1)?,
        read_integer(state, 2)?,
        read_integer(state, 3)?,
    );
    // INFERRED: an unconfigured tuple is an explicit error, not a fabricated link.
    let link = borrow_state(state)?
        .recipe_quality_inputs
        .reagent_links
        .get(&key)
        .cloned()
        .ok_or_else(|| rilua::runtime_error("recipe reagent quality link is not configured"))?;
    let value = create_string(state, &link);
    state.push(value);
    Ok(1)
}
