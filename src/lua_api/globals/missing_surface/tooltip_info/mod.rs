mod builders;
mod probes;
mod sources;
mod spell;
mod unit;

use super::ensure_namespace;
use crate::lua_api::methods::{
    call_function_state, create_string, table_get, table_set, val_to_string,
};
use crate::lua_bridge::table_set_rust_fn_static;
use builders::{
    ensure_pet_info_state, get_pet_tamers_for_map, get_spell_for_pet_action, is_pet_action_passive,
};
use probes::*;
use rilua::vm::gc::arena::GcRef;
use rilua::vm::state::LuaState;
use rilua::vm::table::Table;
use rilua::{LuaResult, Val};

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
    if matches!(
        crate::client_profile::ACTIVE,
        crate::client_profile::ClientProfile::Retail
            | crate::client_profile::ClientProfile::Ptr
            | crate::client_profile::ClientProfile::WowForever
    ) {
        crate::c_api::c_tooltip_info_bag_child::register(state, table_ref)?;
        crate::c_api::c_tooltip_info_totem::register(state, table_ref)?;
        crate::c_api::c_tooltip_info_artifact_item::register(state, table_ref)?;
        crate::c_api::c_tooltip_info_equipment_set::register(state, table_ref)?;
        crate::c_api::c_tooltip_info_inventory_by_id::register(state, table_ref)?;
        crate::c_api::c_tooltip_info_item_key::register(state, table_ref)?;
        // WM94 §5: Retail, PTR and Forever pinned docs declare GetAzeritePower
        // (TooltipInfoDocumentation.lua:102) and GetConduit (:204).
        crate::c_api::c_tooltip_info_azerite_power::register(state, table_ref)?;
        crate::c_api::c_tooltip_info_conduit::register(state, table_ref)?;
        // WM108 §A/§B: Retail, PTR and Forever pinned docs declare
        // GetArtifactPowerByID (TooltipInfoDocumentation.lua:53) and
        // GetEnhancedConduit (:254).
        crate::c_api::c_tooltip_info_artifact_power::register(state, table_ref)?;
        crate::c_api::c_tooltip_info_enhanced_conduit::register(state, table_ref)?;
        // WM107: the pinned docs declare GetPvpBrawl
        // (TooltipInfoDocumentation.lua:710) on Retail, PTR and Forever; the
        // supplied brawl state is compiled for the same profiles.
        #[cfg(any(feature = "retail-12-0-5", feature = "client-wowforever"))]
        crate::c_api::c_tooltip_info_pvp_brawl::register(state, table_ref)?;
    }
    // Retail, PTR and Forever pinned docs all declare the loot getters,
    // including GetLootRollItem (Forever TooltipInfoDocumentation.lua:554-584).
    if matches!(
        crate::client_profile::ACTIVE,
        crate::client_profile::ClientProfile::Retail
            | crate::client_profile::ClientProfile::Ptr
            | crate::client_profile::ClientProfile::WowForever
    ) {
        crate::c_api::c_tooltip_info_loot::register(state, table_ref)?;
        crate::c_api::c_tooltip_info_weekly::register(state, table_ref)?;
    }
    // Retail, PTR and Forever pinned docs declare GetSocketedRelic
    // (TooltipInfoDocumentation.lua:1012). Forever keeps the catalog item.
    if matches!(
        crate::client_profile::ACTIVE,
        crate::client_profile::ClientProfile::Retail
            | crate::client_profile::ClientProfile::Ptr
            | crate::client_profile::ClientProfile::WowForever
    ) {
        crate::c_api::c_tooltip_info_socketed_relic::register(state, table_ref)?;
    }
    #[cfg(feature = "retail-12-0-5")]
    {
        if matches!(
            crate::client_profile::ACTIVE,
            crate::client_profile::ClientProfile::Retail
                | crate::client_profile::ClientProfile::Ptr
        ) {
            crate::c_api::c_tooltip_info_appearance::register(state, table_ref)?;
        }
        crate::c_api::c_tooltip_info_item_context::register(state, table_ref)?;
        crate::c_api::c_tooltip_info_outfit::register(state, table_ref)?;
    }
    // Decomp Round 6 §2 decodes the merchant extended-cost selection block for
    // Retail 12.1.5; Forever's block is undecoded, so the getter stays off the
    // Forever surface.
    #[cfg(feature = "retail-12-1-5")]
    {
        if matches!(
            crate::client_profile::ACTIVE,
            crate::client_profile::ClientProfile::Retail
                | crate::client_profile::ClientProfile::Ptr
        ) {
            crate::c_api::c_tooltip_info_merchant_cost::register(state, table_ref)?;
        }
    }
    register_item_spell_aura_methods(state, table_ref)?;
    register_spell_aura_unit_methods(state, table_ref)?;
    #[cfg(feature = "retail-12-0-5")]
    {
        crate::c_api::c_tooltip_info_spell_mount::register(state, table_ref)?;
        crate::c_api::c_tooltip_info_aura_instance::register(state, table_ref)?;
        crate::c_api::c_tooltip_info_indexed_aura::register(state, table_ref)?;
    }
    Ok(())
}

/// Dispatch the selected catalog clone through the unchanged item line builder.
pub(crate) fn tooltip_for_item_context(
    state: &mut LuaState,
    item_id: u32,
    item: Option<&crate::items::ItemInfo>,
) -> rilua::Val {
    match item {
        Some(item) => builders::tooltip_for_item_info(state, item_id, item),
        None => builders::empty_tooltip(state, super::TOOLTIP_TYPE_ITEM),
    }
}

/// Feed a spell ID through the existing spell payload builder. The identical
/// bridge is used by every spell-identifying getter, so the published type/id and
/// line set stay in one place.
pub(crate) fn tooltip_for_spell(state: &mut LuaState, spell_id: u32) -> Val {
    spell::tooltip_for_spell_id(state, spell_id)
}

/// Feed a merchant currency cost through the existing currency payload builder.
/// The native cost getter passes an explicit zero quantity rather than the
/// extended-cost amount (decomp Round 6 §2c), so the record's identity is the
/// only composition input this path supplies. A currency identity with no
/// resident currency row keeps the verified type-5 identity and publishes no
/// lines, because the native post-title rows are UNKNOWN.
///
/// Only the Retail/PTR merchant-cost getter calls this bridge, so it is compiled
/// with that epoch to keep other profiles warning-free.
#[cfg(feature = "retail-12-1-5")]
pub(crate) fn tooltip_for_cost_currency(state: &mut LuaState, currency_id: u32) -> Val {
    let saved_top = state.top;
    let tooltip = match i32::try_from(currency_id)
        .ok()
        .and_then(crate::lua_api::globals::currency_data::get_currency_by_id)
    {
        Some(currency) => sources::tooltip_for_currency(state, currency, Some(0)),
        None => {
            let tooltip = builders::create_identified_tooltip(
                state,
                super::TOOLTIP_TYPE_CURRENCY,
                currency_id,
            );
            state.push(tooltip);
            builders::stamp_data_instance(state, tooltip);
            tooltip
        }
    };
    state.top = saved_top;
    tooltip
}

/// Resolved spell description text for getters that compose their own rows.
pub(crate) fn spell_description_text(state: &LuaState, spell_id: u32) -> String {
    spell::resolved_spell_description(state, spell_id)
}

/// Item quality colour, shared by item and conduit composition.
pub(crate) fn item_quality_color(quality: u8) -> (f64, f64, f64) {
    builders::item_quality_color(quality)
}

/// WM97 DQ4: relic-aware item composition for the viewed artifact's socket.
pub(crate) fn tooltip_for_socketed_relic_context(
    state: &mut LuaState,
    item_id: u32,
    item: &crate::items::ItemInfo,
    relic: &crate::c_api::item_descriptor::RelicMetadata,
) -> LuaResult<rilua::Val> {
    builders::tooltip_for_socketed_relic(state, item_id, item, relic)
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

/// Narrow player-only lookup; clone before the unchanged builder allocates in the VM.
#[cfg(feature = "retail-12-0-5")]
pub(crate) fn tooltip_for_aura_instance(
    state: &mut LuaState,
    unit: &str,
    instance_id: i32,
    helpful: Option<bool>,
) -> rilua::Val {
    let aura = if unit == "player" {
        spell::lookup_player_aura_by_instance_id(state, instance_id)
            .filter(|aura| helpful.is_none_or(|helpful| aura.is_helpful == helpful))
    } else {
        None
    };
    spell::tooltip_for_unit_aura(state, aura)
}

/// Dispatch the selected clone through the unchanged aura payload builder.
#[cfg(feature = "retail-12-0-5")]
pub(crate) fn tooltip_for_selected_indexed_aura(
    state: &mut LuaState,
    aura: Option<crate::lua_api::state::AuraInfo>,
) -> rilua::Val {
    spell::tooltip_for_unit_aura(state, aura)
}

/// Feed the selected totem's text through the existing tooltip publication path.
pub(crate) fn tooltip_for_totem_text(
    state: &mut LuaState,
    name: &str,
    remaining: &str,
) -> LuaResult<rilua::Val> {
    builders::tooltip_for_totem_text(state, name, remaining)
}

/// Bounded custom payload identity for getters that compose their own lines.
/// The shared D56 stamp stays in one place; the caller supplies the declared
/// `Enum.TooltipDataType` and, when the getter has one, its identity id.
pub(crate) fn compose_tooltip(state: &mut LuaState, tooltip_type: f64, id: Option<f64>) -> Val {
    let tooltip = builders::empty_tooltip(state, tooltip_type);
    if let Some(id) = id {
        table_set(state, tooltip, "id", Val::Num(id));
    }
    builders::stamp_data_instance(state, tooltip);
    tooltip
}

/// Append one line after the payload's current last line.
pub(crate) fn append_line(
    state: &mut LuaState,
    tooltip: Val,
    line_type: f64,
    text: &str,
    color: Option<(f64, f64, f64)>,
) -> Val {
    append_line_with_right(state, tooltip, line_type, text, color, None, None)
}

/// Append one double line (left and right text) after the current last line.
pub(crate) fn append_double_line(
    state: &mut LuaState,
    tooltip: Val,
    line_type: f64,
    left_text: &str,
    left_color: Option<(f64, f64, f64)>,
    right_text: &str,
    right_color: Option<(f64, f64, f64)>,
) -> Val {
    append_line_with_right(
        state,
        tooltip,
        line_type,
        left_text,
        left_color,
        Some(right_text),
        right_color,
    )
}

fn append_line_with_right(
    state: &mut LuaState,
    tooltip: Val,
    line_type: f64,
    left_text: &str,
    left_color: Option<(f64, f64, f64)>,
    right_text: Option<&str>,
    right_color: Option<(f64, f64, f64)>,
) -> Val {
    let saved_top = state.top;
    state.push(tooltip);
    let lines = table_get(state, tooltip, "lines");
    let index = next_line_index(state, lines);
    let line =
        builders::push_tooltip_line(state, lines, index, line_type, left_text, left_color, false);
    if let Some(right_text) = right_text {
        if builders::uses_modern_tooltip_data() {
            let rgb = right_color
                .or_else(|| builders::global_color(state, b"NORMAL_FONT_COLOR"))
                .unwrap_or((1.0, 0.82, 0.0));
            let color = builders::color_table(state, rgb.0, rgb.1, rgb.2, 1.0);
            state.push(color);
            crate::c_api::tooltip_data::publish_rgb(state, color, rgb);
            crate::c_api::tooltip_data::publish_side(state, line, true, Some(right_text), color);
        } else {
            let text = create_string(state, right_text);
            table_set(state, line, "rightText", text);
            if let Some((r, g, b)) = right_color {
                let color = builders::color_table(state, r, g, b, 1.0);
                table_set(state, line, "rightColor", color);
            }
        }
    }
    state.top = saved_top;
    line
}

fn next_line_index(state: &mut LuaState, lines: Val) -> i64 {
    let Val::Table(lines_ref) = lines else {
        return 1;
    };
    let mut index = 1;
    while state
        .gc
        .tables
        .get(lines_ref)
        .is_some_and(|table| table.get_int(index) != Val::Nil)
    {
        index += 1;
    }
    index
}

/// Format a pinned global string with one numeric argument.
pub(crate) fn format_global_numeric(
    state: &mut LuaState,
    key: &str,
    value: f64,
) -> LuaResult<String> {
    let template = crate::c_api::global_val(state, key);
    let formatter = crate::c_api::global_val(state, "format");
    let formatted = call_function_state(state, formatter, &[template, Val::Num(value)])?;
    val_to_string(state, formatted)
        .ok_or_else(|| rilua::runtime_error(format!("localized formatter did not return {key}")))
}

/// Format a pinned global string with two numeric arguments (for example
/// `TOOLTIP_TALENT_RANK`, which carries a current and a maximum rank).
pub(crate) fn format_global_pair(
    state: &mut LuaState,
    key: &str,
    first: f64,
    second: f64,
) -> LuaResult<String> {
    let template = crate::c_api::global_val(state, key);
    let formatter = crate::c_api::global_val(state, "format");
    let formatted = call_function_state(
        state,
        formatter,
        &[template, Val::Num(first), Val::Num(second)],
    )?;
    val_to_string(state, formatted)
        .ok_or_else(|| rilua::runtime_error(format!("localized formatter did not return {key}")))
}

/// Restore a plain line type that the shared value-based remap rewrote. The
/// remap compares a type-0 row's colour against the named `ERROR_COLOR` and
/// `DISABLED_FONT_COLOR` entries, which both resolve white while the colour
/// table is a placeholder, so a verified white type-0 row would become 41.
pub(crate) fn restore_plain_line_type(state: &mut LuaState, line: Val) {
    crate::lua_api::methods::table_set(state, line, "type", Val::Num(0.0));
}

/// Format a pinned global string with one text argument.

pub(crate) fn format_global_text(
    state: &mut LuaState,
    key: &str,
    value: &str,
) -> LuaResult<String> {
    let template = crate::c_api::global_val(state, key);
    let formatter = crate::c_api::global_val(state, "format");
    let text = create_string(state, value);
    let formatted = call_function_state(state, formatter, &[template, text])?;
    val_to_string(state, formatted)
        .ok_or_else(|| rilua::runtime_error(format!("localized formatter did not return {key}")))
}

/// Pinned named color as RGB, or `None` when the global is absent.
pub(crate) fn named_color(state: &mut LuaState, key: &str) -> Option<(f64, f64, f64)> {
    builders::global_color(state, key.as_bytes())
}

/// Pinned global string value, or `None` when the global is absent.
pub(crate) fn global_text(state: &mut LuaState, key: &str) -> Option<String> {
    let value = crate::c_api::global_val(state, key);
    val_to_string(state, value)
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
            #[cfg(not(feature = "retail-12-0-5"))]
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
            #[cfg(not(feature = "retail-12-0-5"))]
            ("GetUnitBuff", c_tooltip_get_unit_buff),
            #[cfg(not(feature = "retail-12-0-5"))]
            (
                "GetUnitBuffByAuraInstanceID",
                c_tooltip_get_unit_buff_by_aura_instance_id,
            ),
            #[cfg(not(feature = "retail-12-0-5"))]
            ("GetUnitDebuff", c_tooltip_get_unit_debuff),
            #[cfg(not(feature = "retail-12-0-5"))]
            (
                "GetUnitDebuffByAuraInstanceID",
                c_tooltip_get_unit_debuff_by_aura_instance_id,
            ),
            ("GetUnitAura", c_tooltip_get_unit_aura),
            #[cfg(not(feature = "retail-12-0-5"))]
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
