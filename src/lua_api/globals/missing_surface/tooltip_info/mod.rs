mod builders;
mod probes;
mod sources;
mod spell;
mod unit;

use super::ensure_namespace;
use crate::lua_bridge::table_set_rust_fn_static;
use builders::{
    ensure_pet_info_state, get_pet_tamers_for_map, get_spell_for_pet_action, is_pet_action_passive,
};
use probes::*;
use rilua::LuaResult;
use rilua::vm::gc::arena::GcRef;
use rilua::vm::state::LuaState;
use rilua::vm::table::Table;

pub(super) fn register_tooltip_surface(state: &mut LuaState) -> LuaResult<()> {
    register_pet_info_surface(state)?;
    ensure_pet_info_state(state);
    register_c_tooltip_info(state)
}

pub(super) fn register_pet_info_surface(state: &mut LuaState) -> LuaResult<()> {
    let table_ref = ensure_namespace(state, "C_PetInfo")?;
    ensure_pet_info_state(state);
    table_set_rust_fn_static(
        state,
        table_ref,
        "GetPetTamersForMap",
        get_pet_tamers_for_map,
    )?;
    table_set_rust_fn_static(
        state,
        table_ref,
        "GetSpellForPetAction",
        get_spell_for_pet_action,
    )?;
    table_set_rust_fn_static(
        state,
        table_ref,
        "IsPetActionPassive",
        is_pet_action_passive,
    )?;
    Ok(())
}

fn register_c_tooltip_info(state: &mut LuaState) -> LuaResult<()> {
    let table_ref = ensure_namespace(state, "C_TooltipInfo")?;
    register_item_spell_aura_methods(state, table_ref)?;
    register_spell_aura_unit_methods(state, table_ref)?;
    #[cfg(feature = "retail-12-0-5")]
    crate::c_api::c_tooltip_info_spell_mount::register(state, table_ref)?;
    Ok(())
}

/// Bounded bridge to the existing payload model; no identifier parsing here.
#[cfg(feature = "retail-12-0-5")]
pub(crate) fn tooltip_for_spell_identifier(
    state: &mut LuaState,
    spell_id: Option<u32>,
    is_mount: bool,
) -> rilua::Val {
    match spell_id {
        Some(id) if is_mount => spell::tooltip_for_mount_spell_id(state, id),
        Some(id) => spell::tooltip_for_spell_id(state, id),
        None => builders::empty_tooltip(state, super::TOOLTIP_TYPE_SPELL),
    }
}

type TooltipScriptFn = fn(&mut LuaState) -> LuaResult<u32>;

fn register_tooltip_methods(
    state: &mut LuaState,
    table_ref: GcRef<Table>,
    entries: &[(&'static str, TooltipScriptFn)],
) -> LuaResult<()> {
    for &(name, func) in entries {
        table_set_rust_fn_static(state, table_ref, name, func)?;
    }
    Ok(())
}

fn register_item_spell_aura_methods(
    state: &mut LuaState,
    table_ref: GcRef<Table>,
) -> LuaResult<()> {
    register_item_container_methods(state, table_ref)?;
    register_quest_and_recipe_methods(state, table_ref)?;
    register_socket_and_currency_methods(state, table_ref)?;
    register_misc_content_methods(state, table_ref)?;
    Ok(())
}

fn register_item_container_methods(state: &mut LuaState, table_ref: GcRef<Table>) -> LuaResult<()> {
    register_tooltip_methods(
        state,
        table_ref,
        &[
            ("GetBagItem", c_tooltip_get_bag_item),
            ("GetGuildBankItem", c_tooltip_get_guild_bank_item),
            ("GetItem", c_tooltip_get_item),
            ("GetItemByID", c_tooltip_get_item_by_id),
            ("GetItemByGUID", c_tooltip_get_item_by_guid),
            ("GetOwnedItemByID", c_tooltip_get_owned_item_by_id),
            ("GetInventoryItem", c_tooltip_get_inventory_item),
            ("GetMerchantItem", c_tooltip_get_merchant_item),
            ("GetUpgradeItem", c_tooltip_get_upgrade_item),
            ("GetTooltipDataForItem", c_tooltip_get_tooltip_data_for_item),
        ],
    )
}

fn register_quest_and_recipe_methods(
    state: &mut LuaState,
    table_ref: GcRef<Table>,
) -> LuaResult<()> {
    register_tooltip_methods(
        state,
        table_ref,
        &[
            ("GetQuestCurrency", c_tooltip_get_quest_currency),
            ("GetQuestItem", c_tooltip_get_quest_item),
            ("GetQuestLogCurrency", c_tooltip_get_quest_log_currency),
            ("GetQuestLogItem", c_tooltip_get_quest_log_item),
            ("GetRecipeReagentItem", c_tooltip_get_recipe_reagent_item),
            ("GetRecipeResultItem", c_tooltip_get_recipe_result_item),
            (
                "GetRecipeResultItemForOrder",
                c_tooltip_get_recipe_result_item_for_order,
            ),
            ("GetTradeSkillItem", c_tooltip_get_trade_skill_item),
            ("GetTradePlayerItem", c_tooltip_get_trade_player_item),
            ("GetTradeTargetItem", c_tooltip_get_trade_target_item),
            ("GetTrainerService", c_tooltip_get_trainer_service),
        ],
    )
}

fn register_socket_and_currency_methods(
    state: &mut LuaState,
    table_ref: GcRef<Table>,
) -> LuaResult<()> {
    register_tooltip_methods(
        state,
        table_ref,
        &[
            ("GetBackpackToken", c_tooltip_get_backpack_token),
            ("GetCurrencyByID", c_tooltip_get_currency_by_id),
            ("GetCurrencyToken", c_tooltip_get_currency_token),
            ("GetSocketedItem", c_tooltip_get_socketed_item),
            ("GetSocketGem", c_tooltip_get_socket_gem),
            ("GetExistingSocketGem", c_tooltip_get_existing_socket_gem),
        ],
    )
}

fn register_misc_content_methods(state: &mut LuaState, table_ref: GcRef<Table>) -> LuaResult<()> {
    register_tooltip_methods(
        state,
        table_ref,
        &[
            ("GetTraitEntry", c_tooltip_get_trait_entry),
            ("GetAction", c_tooltip_get_action),
            ("GetAchievementByID", c_tooltip_get_achievement_by_id),
            ("GetAura", c_tooltip_get_aura),
            (
                "GetInstanceLockEncountersComplete",
                c_tooltip_get_instance_lock_encounters_complete,
            ),
            ("GetLFGDungeon", c_tooltip_get_lfg_dungeon),
            ("GetPetAction", c_tooltip_get_pet_action),
            ("GetShapeshift", c_tooltip_get_shapeshift),
            #[cfg(not(feature = "retail-12-0-5"))]
            ("GetMountBySpellID", c_tooltip_get_mount_by_spell_id),
            ("GetCompanionPet", c_tooltip_get_companion_pet),
            ("GetTalent", c_tooltip_get_talent),
            ("GetToyByItemID", c_tooltip_get_toy_by_item_id),
            ("GetHeirloomByItemID", c_tooltip_get_heirloom_by_item_id),
            ("GetMinimapMouseover", c_tooltip_get_minimap_mouseover),
        ],
    )
}

fn register_spell_aura_unit_methods(
    state: &mut LuaState,
    table_ref: GcRef<Table>,
) -> LuaResult<()> {
    register_tooltip_methods(
        state,
        table_ref,
        &[
            ("GetSpellBookItem", c_tooltip_get_spell_book_item),
            #[cfg(not(feature = "retail-12-0-5"))]
            ("GetSpellByID", c_tooltip_get_spell_by_id),
            ("GetUnitBuff", c_tooltip_get_unit_buff),
            (
                "GetUnitBuffByAuraInstanceID",
                c_tooltip_get_unit_buff_by_aura_instance_id,
            ),
            ("GetUnitDebuff", c_tooltip_get_unit_debuff),
            (
                "GetUnitDebuffByAuraInstanceID",
                c_tooltip_get_unit_debuff_by_aura_instance_id,
            ),
            ("GetUnitAura", c_tooltip_get_unit_aura),
            (
                "GetUnitAuraByAuraInstanceID",
                c_tooltip_get_unit_aura_by_aura_instance_id,
            ),
            ("GetHyperlink", c_tooltip_get_hyperlink),
            ("GetInboxItem", c_tooltip_get_inbox_item),
            ("GetSendMailItem", c_tooltip_get_send_mail_item),
            ("GetSpell", c_tooltip_get_spell),
            ("GetWorldCursor", c_tooltip_get_world_cursor),
            ("GetWorldLootObject", c_tooltip_get_world_loot_object),
            ("GetUnit", c_tooltip_get_unit),
        ],
    )
}

#[cfg(test)]
mod tests {
    use super::super::{LINE_TYPE_ITEM_LEVEL, LINE_TYPE_ITEM_NAME};
    use super::builders::tooltip_for_item_id;
    use super::unit::tooltip_for_unit;
    use crate::lua_api::env::WowLuaEnv;
    use crate::lua_api::methods::{table_get, val_to_string};
    use rilua::vm::state::LuaState;
    use rilua::{Lua, LuaApiMut, Val};

    #[test]
    fn tooltip_for_item_id_populates_name_and_level_lines() {
        let mut lua = Lua::new().expect("should create rilua state");
        let tooltip = {
            let state = lua.state_mut();
            tooltip_for_item_id(state, 6948)
        };
        let mut state = lua.state_mut();
        let lines = table_get(&mut state, tooltip, "lines");

        let name_line = get_array_element(&mut state, lines, 1);
        assert_eq!(line_type(&mut state, name_line), Some(LINE_TYPE_ITEM_NAME));
        assert_eq!(
            line_text(&mut state, name_line).as_deref(),
            Some("Hearthstone")
        );

        let level_line = get_array_element(&mut state, lines, 2);
        assert_eq!(
            line_type(&mut state, level_line),
            Some(LINE_TYPE_ITEM_LEVEL)
        );
        assert_eq!(
            line_text(&mut state, level_line).as_deref(),
            Some("Item Level 1")
        );
    }

    #[test]
    fn tooltip_for_unit_player_shows_name_and_level() {
        let env = WowLuaEnv::new().expect("should create WowLuaEnv");
        {
            let mut sim = env.state().borrow_mut();
            sim.player.name = "Tester".to_string();
            sim.player.level = 99;
            sim.player.class_index = 3;
            sim.player.race_index = 1;
        }
        let tooltip = {
            let mut lua = env.rilua_mut();
            tooltip_for_unit(lua.state_mut(), "player")
        };
        let mut lua = env.rilua_mut();
        let state = lua.state_mut();
        let lines = table_get(state, tooltip, "lines");
        let name_line = get_array_element(state, lines, 1);
        assert_eq!(line_text(state, name_line).as_deref(), Some("Tester"));

        let level_line = get_array_element(state, lines, 2);
        assert_eq!(line_text(state, level_line).as_deref(), Some("Level 99"));
    }

    fn get_array_element(state: &mut LuaState, table: Val, index: i64) -> Val {
        let Val::Table(table_ref) = table else {
            return Val::Nil;
        };
        state
            .gc
            .tables
            .get(table_ref)
            .map(|table| table.get_int(index))
            .unwrap_or(Val::Nil)
    }

    fn line_type(state: &mut LuaState, line: Val) -> Option<f64> {
        match table_get(state, line, "type") {
            Val::Num(value) => Some(value),
            _ => None,
        }
    }

    fn line_text(state: &mut LuaState, line: Val) -> Option<String> {
        let text_val = table_get(state, line, "leftText");
        val_to_string(state, text_val)
    }
}
