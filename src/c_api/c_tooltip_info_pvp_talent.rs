//! Retail `C_TooltipInfo.GetPvpTalent` composition and the Forever supplied-input path.
//!
//! R44-Q20, R53 and R56 establish the Retail success path. The Retail talent catalog
//! is generated from the local-CASC export recorded in provenance; resolved
//! spell IDs and eligibility/grant outcomes remain host inputs. Forever validates
//! its arguments and composes only supplied talent-to-spell rows.

#[cfg(feature = "client-retail")]
use crate::c_api::tooltip_getter_args::lua_index;
#[cfg(feature = "client-retail")]
use crate::c_api::tooltip_getter_args::round7_optional_bool;
use crate::c_api::tooltip_getter_args::{allowed_when_untainted, round7_signed};
use crate::lua_api::globals::missing_surface::tooltip_info::tooltip_for_spell;
#[cfg(feature = "client-retail")]
use crate::lua_api::globals::missing_surface::tooltip_info::{
    append_line, format_global_numeric, format_global_text, global_text, named_color,
};
use crate::lua_api::methods::borrow_state;
#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
use crate::lua_api::methods::table_set;
use crate::lua_bridge::{stack_val, table_set_rust_fn_static};
use rilua::LuaResult;
#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
use rilua::Val;
use rilua::vm::{gc::arena::GcRef, state::LuaState, table::Table};
#[cfg(feature = "client-retail")]
use std::collections::HashMap;

#[cfg(feature = "client-retail")]
include!("c_tooltip_info_pvp_talent_data.rs");

const API_NAME: &str = "C_TooltipInfo.GetPvpTalent";
const USAGE: &str = "Usage: local data = C_TooltipInfo.GetPvpTalent(talentID [, isInspect, groupIndex, talentIndex])";
#[cfg(feature = "client-retail")]
const PLAIN_LINE_TYPE: f64 = 0.0;
#[cfg(feature = "client-retail")]
const BLANK_LINE_TYPE: f64 = 1.0;
#[cfg(feature = "client-retail")]
const TALENT_POSITION_COUNT: i32 = 4;
#[cfg(feature = "client-retail")]
const SUPPRESSED_OVERRIDE_ATTRIBUTE: u32 = 32768;
#[cfg(feature = "client-retail")]
const BLANK_LINE_TEXT: &str = " ";

#[cfg(feature = "client-retail")]
/// One catalogued PvP talent row plus host-supplied eligibility and grant outcomes.
#[derive(Clone, Debug, Default)]
pub struct PvpTalentTooltipRecord {
    pub spell_id: u32,
    pub overrides_spell_id: u32,
    /// Host-supplied `LevelRequired`; zero disables the early level message.
    pub level_required: i32,
    /// Category positions accepted by the external eligibility helper.
    pub category_position_mask: u8,
    /// Optional result from the external talent-eligibility helper by zero-based
    /// talent position. Missing results leave that helper outcome unmodelled.
    pub eligibility_by_position: HashMap<i32, bool>,
    /// The host confirms the native item-grant rejection predicate.
    pub grant_rejected_by_item: bool,
    /// Candidate item from the host's local effect scan.
    pub granted_item_id: Option<u32>,
    /// True only when the host confirms the essence lookup succeeded.
    pub granted_by_essence: bool,
    /// External eligibility helper reports its later level gate failed.
    pub eligibility_level_failed: bool,
}

#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
/// Host-supplied values with the pinned PvpTalentSlotInfo shape.
#[derive(Clone, Debug)]
pub struct PvpTalentSlotInput {
    pub enabled: bool,
    pub level: i32,
    pub available_talent_ids: Vec<i32>,
}

#[cfg(feature = "client-retail")]
/// Host inputs used by the Retail PvP talent tooltip producer.
#[derive(Clone, Debug)]
pub struct PvpTalentTooltipInputs {
    pub catalog: HashMap<i32, PvpTalentTooltipRecord>,
    /// Zero-based active group. `GetActiveSpecGroup` publishes this plus one.
    pub active_group_index: usize,
    /// Valid local groups and their four selected talent IDs. A missing group
    /// has no local membership context.
    pub groups: HashMap<usize, [Option<i32>; TALENT_POSITION_COUNT as usize]>,
    /// Resolved spell outcomes supplied by the host; absent entries mean identity.
    pub resolved_spell_ids: HashMap<u32, u32>,
    /// None leaves the eligibility result unmodelled; not a native default.
    pub rest_permission: Option<bool>,
    /// One-based UI slot records; selected IDs come from `groups`.
    pub slots: HashMap<usize, PvpTalentSlotInput>,
    /// Granting items in the inspected-unit context, keyed by GUID and talent ID.
    pub inspect_grant_item_ids: HashMap<(String, i32), u32>,
}

#[cfg(feature = "client-retail")]
impl Default for PvpTalentTooltipInputs {
    fn default() -> Self {
        let catalog = ROWS
            .iter()
            .map(
                |&(
                    talent_id,
                    spell_id,
                    overrides_spell_id,
                    level_required,
                    category_position_mask,
                )| {
                    (
                        talent_id,
                        PvpTalentTooltipRecord {
                            spell_id,
                            overrides_spell_id,
                            level_required,
                            category_position_mask,
                            ..PvpTalentTooltipRecord::default()
                        },
                    )
                },
            )
            .collect();
        Self {
            catalog,
            active_group_index: 0,
            groups: HashMap::new(),
            resolved_spell_ids: HashMap::new(),
            rest_permission: None,
            slots: HashMap::new(),
            inspect_grant_item_ids: HashMap::new(),
        }
    }
}

pub(crate) fn register(state: &mut LuaState, namespace: GcRef<Table>) -> LuaResult<()> {
    table_set_rust_fn_static(state, namespace, "GetPvpTalent", get_pvp_talent)
}

#[cfg(feature = "client-retail")]
fn read_group_index(state: &LuaState, position: i32) -> LuaResult<Option<usize>> {
    lua_index(state, position)
}

#[cfg(feature = "client-wowforever")]
fn read_group_index(state: &LuaState, position: i32) -> LuaResult<Option<usize>> {
    crate::c_api::tooltip_getter_args::forever_lua_index(state, position, USAGE)
}

#[derive(Clone, Copy)]
struct PvpTalentArguments {
    talent_id: i32,
    #[cfg(feature = "client-retail")]
    retail: RetailPvpTalentArguments,
}

#[cfg(feature = "client-retail")]
#[derive(Clone, Copy)]
struct RetailPvpTalentArguments {
    is_inspect: bool,
    group_was_supplied: bool,
    parsed_group: Option<usize>,
    talent_position: i32,
}

#[cfg(feature = "client-retail")]
struct RetailPvpTalentContext {
    record: PvpTalentTooltipRecord,
    active_group_index: usize,
    player_present: bool,
}

fn read_pvp_talent_arguments(state: &LuaState) -> LuaResult<PvpTalentArguments> {
    let talent_id = round7_signed(state, 1, USAGE)?;
    #[cfg(feature = "client-wowforever")]
    validate_forever_optional_pvp_talent_arguments(state)?;
    #[cfg(feature = "client-retail")]
    let retail = read_retail_pvp_talent_arguments(state)?;

    Ok(PvpTalentArguments {
        talent_id,
        #[cfg(feature = "client-retail")]
        retail,
    })
}

#[cfg(feature = "client-wowforever")]
fn validate_forever_optional_pvp_talent_arguments(state: &LuaState) -> LuaResult<()> {
    // Forever's supplied source map indexes only talent ID. Other selectors are
    // validated, but they do not select records.
    let _is_inspect = crate::c_api::tooltip_getter_args::optional_bool(state, 2)?;
    let group_value = allowed_when_untainted(state, 3, API_NAME)?;
    if !group_value.is_nil() {
        let _group_index = read_group_index(state, 3)?;
    }
    let _talent_position = read_talent_position(state)?;
    Ok(())
}

#[cfg(feature = "client-retail")]
fn read_retail_pvp_talent_arguments(state: &LuaState) -> LuaResult<RetailPvpTalentArguments> {
    let is_inspect = round7_optional_bool(state, 2, USAGE)?;
    let group_value = allowed_when_untainted(state, 3, API_NAME)?;
    let group_was_supplied = !group_value.is_nil();
    let parsed_group = if group_was_supplied {
        read_group_index(state, 3)?
    } else {
        None
    };
    let talent_position = read_talent_position(state)?;
    Ok(RetailPvpTalentArguments {
        is_inspect,
        group_was_supplied,
        parsed_group,
        talent_position,
    })
}

fn read_talent_position(state: &LuaState) -> LuaResult<i32> {
    if stack_val(state, 4).is_nil() {
        Ok(0)
    } else {
        round7_signed(state, 4, USAGE)
    }
}

fn get_pvp_talent(state: &mut LuaState) -> LuaResult<u32> {
    let arguments = read_pvp_talent_arguments(state)?;
    #[cfg(feature = "client-wowforever")]
    {
        publish_forever_pvp_talent(state, arguments.talent_id)
    }
    #[cfg(feature = "client-retail")]
    {
        publish_retail_pvp_talent(state, arguments)
    }
}

#[cfg(feature = "client-wowforever")]
fn publish_forever_pvp_talent(state: &mut LuaState, talent_id: i32) -> LuaResult<u32> {
    let spell_id = borrow_state(state)?
        .forever_pvp_talent_spell_ids
        .get(&talent_id)
        .copied();
    let Some(spell_id) = spell_id else {
        return Ok(0);
    };
    let tooltip = tooltip_for_spell(state, spell_id);
    state.push(tooltip);
    Ok(1)
}

#[cfg(feature = "client-retail")]
fn publish_retail_pvp_talent(
    state: &mut LuaState,
    arguments: PvpTalentArguments,
) -> LuaResult<u32> {
    if arguments.talent_id == 0 {
        return crate::c_api::tooltip_data::unknown_getter_failure();
    }
    let context = {
        let sim = borrow_state(state)?;
        sim.pvp_talent_tooltip
            .catalog
            .get(&arguments.talent_id)
            .cloned()
            .map(|record| RetailPvpTalentContext {
                record,
                active_group_index: sim.pvp_talent_tooltip.active_group_index,
                player_present: sim.player.present,
            })
    };
    let Some(context) = context else {
        return crate::c_api::tooltip_data::unknown_getter_failure();
    };
    if crate::spells::get_spell(context.record.spell_id).is_none() {
        return crate::c_api::tooltip_data::unknown_getter_failure();
    }
    let resolved_spell_id = resolve_spell_id(state, context.record.spell_id, None)?;
    if !context.player_present || crate::spells::get_spell(resolved_spell_id).is_none() {
        // R53 confirms annotation continuation, but the public result is UNKNOWN.
        return crate::c_api::tooltip_data::unknown_getter_failure();
    }
    compose_retail_pvp_talent(state, arguments, context, resolved_spell_id)
}

#[cfg(feature = "client-retail")]
fn compose_retail_pvp_talent(
    state: &mut LuaState,
    arguments: PvpTalentArguments,
    context: RetailPvpTalentContext,
    spell_id: u32,
) -> LuaResult<u32> {
    let tooltip = tooltip_for_spell(state, spell_id);
    append_replacement_line(state, tooltip, &context.record)?;
    let local_group = if arguments.retail.is_inspect {
        None
    } else if arguments.retail.group_was_supplied {
        arguments.retail.parsed_group
    } else {
        Some(context.active_group_index)
    };
    append_talent_annotation(
        state,
        tooltip,
        arguments.talent_id,
        arguments.retail.talent_position,
        arguments.retail.is_inspect,
        local_group,
        &context.record,
    )?;
    state.push(tooltip);
    Ok(1)
}

#[cfg(feature = "client-retail")]
fn resolve_spell_id(
    state: &LuaState,
    spell_id: u32,
    excluded_spell_id: Option<u32>,
) -> LuaResult<u32> {
    let resolved = borrow_state(state)?
        .pvp_talent_tooltip
        .resolved_spell_ids
        .get(&spell_id)
        .copied()
        .unwrap_or(spell_id);
    Ok(if Some(resolved) == excluded_spell_id {
        spell_id
    } else {
        resolved
    })
}

#[cfg(feature = "client-retail")]
fn append_replacement_line(
    state: &mut LuaState,
    tooltip: Val,
    record: &PvpTalentTooltipRecord,
) -> LuaResult<()> {
    if record.overrides_spell_id == 0 {
        return Ok(());
    }

    let replacement_id = resolve_spell_id(state, record.overrides_spell_id, Some(record.spell_id))?;
    let Some(replacement) = crate::spells::get_spell(replacement_id) else {
        return Ok(());
    };
    let Some(override_record) = crate::spells::get_spell(record.overrides_spell_id) else {
        return Ok(());
    };
    let suppressed = override_record
        .attributes_9
        .is_some_and(|attributes| attributes & SUPPRESSED_OVERRIDE_ATTRIBUTE != 0);
    if suppressed {
        return Ok(());
    }

    let text = format_global_text(state, "REPLACES_SPELL", replacement.name)?;
    append_line(state, tooltip, PLAIN_LINE_TYPE, &text, None);
    Ok(())
}

#[cfg(feature = "client-retail")]
fn append_talent_annotation(
    state: &mut LuaState,
    tooltip: Val,
    talent_id: i32,
    talent_position: i32,
    is_inspect: bool,
    local_group: Option<usize>,
    record: &PvpTalentTooltipRecord,
) -> LuaResult<()> {
    if is_inspect {
        append_inspect_grant_line(state, tooltip, talent_id)?;
        return Ok(());
    }

    let Some(group_index) = local_group else {
        return Ok(());
    };
    let context = local_annotation_context(state, group_index)?;
    let Some(context) = context else {
        return Ok(());
    };
    if context.active_group_index != group_index {
        return Ok(());
    }
    if context.player_level < record.level_required {
        append_unlock_line(state, tooltip, record.level_required, false)?;
        return Ok(());
    }
    if context.selected_talents.contains(&Some(talent_id)) {
        return Ok(());
    }
    if !(0..TALENT_POSITION_COUNT).contains(&talent_position) {
        return Ok(());
    }
    let position_mask = 1_u32 << (talent_position as u32 & 31);
    if u32::from(record.category_position_mask) & position_mask == 0 {
        return Ok(());
    }

    let eligibility = record
        .eligibility_by_position
        .get(&talent_position)
        .copied();
    if eligibility.is_none() && context.rest_permission.is_none() {
        return Ok(());
    }
    append_local_eligibility_line(state, tooltip, context, record, eligibility)
}

#[cfg(feature = "client-retail")]
#[derive(Clone, Copy)]
struct LocalAnnotationContext {
    active_group_index: usize,
    player_level: i32,
    in_combat: bool,
    rest_permission: Option<bool>,
    selected_talents: [Option<i32>; TALENT_POSITION_COUNT as usize],
}

#[cfg(feature = "client-retail")]
fn local_annotation_context(
    state: &LuaState,
    group_index: usize,
) -> LuaResult<Option<LocalAnnotationContext>> {
    let sim = borrow_state(state)?;
    if !sim.player.present {
        return Ok(None);
    }
    let Some(selected_talents) = sim.pvp_talent_tooltip.groups.get(&group_index).copied() else {
        return Ok(None);
    };
    let player_level = if sim.player.effective_level != 0 {
        sim.player.effective_level
    } else {
        sim.player.level
    };
    Ok(Some(LocalAnnotationContext {
        active_group_index: sim.pvp_talent_tooltip.active_group_index,
        player_level,
        in_combat: sim.player.in_combat,
        rest_permission: sim.pvp_talent_tooltip.rest_permission,
        selected_talents,
    }))
}

#[cfg(feature = "client-retail")]
fn append_unlock_line(
    state: &mut LuaState,
    tooltip: Val,
    required_level: i32,
    wrap: bool,
) -> LuaResult<()> {
    let text = format_global_numeric(state, "UNLOCKED_AT_LEVEL", f64::from(required_level))?;
    append_colored_text_line(state, tooltip, &text, "HOSTILE_STATUS_COLOR", wrap);
    Ok(())
}

#[cfg(feature = "client-retail")]
fn append_local_eligibility_line(
    state: &mut LuaState,
    tooltip: Val,
    context: LocalAnnotationContext,
    record: &PvpTalentTooltipRecord,
    eligibility: Option<bool>,
) -> LuaResult<()> {
    // INFERRED (simulator mapping): R88at.1/at3 orders these supplied failures as
    // late level, grant rejection, then combat; display priority does not reorder eligibility.
    if record.eligibility_level_failed {
        return append_unlock_line(state, tooltip, record.level_required, true);
    }
    if record.grant_rejected_by_item {
        append_local_grant_line(state, tooltip, record)?;
        return Ok(());
    }
    if context.in_combat {
        return append_combat_error_line(state, tooltip);
    }
    if eligibility == Some(false) || context.rest_permission == Some(false) {
        return append_rest_error_line(state, tooltip);
    }
    append_preview_line(state, tooltip)
}

#[cfg(feature = "client-retail")]
fn append_combat_error_line(state: &mut LuaState, tooltip: Val) -> LuaResult<()> {
    append_key_line(
        state,
        tooltip,
        "TALENT_TOOLTIP_ADD_COMBAT_ERROR",
        "HOSTILE_STATUS_COLOR",
        true,
    )
}

#[cfg(feature = "client-retail")]
fn append_rest_error_line(state: &mut LuaState, tooltip: Val) -> LuaResult<()> {
    append_key_line(
        state,
        tooltip,
        "TALENT_TOOLTIP_ADD_REST_ERROR_PVP",
        "HOSTILE_STATUS_COLOR",
        true,
    )
}

#[cfg(feature = "client-retail")]
fn append_preview_line(state: &mut LuaState, tooltip: Val) -> LuaResult<()> {
    append_key_line(
        state,
        tooltip,
        "TALENT_TOOLTIP_ADDPREVIEWPOINT",
        "FRIENDLY_STATUS_COLOR",
        true,
    )
}

#[cfg(feature = "client-retail")]
fn append_local_grant_line(
    state: &mut LuaState,
    tooltip: Val,
    record: &PvpTalentTooltipRecord,
) -> LuaResult<bool> {
    if !record.grant_rejected_by_item {
        return Ok(false);
    }
    let Some(item_id) = record.granted_item_id.filter(|item_id| *item_id != 0) else {
        return Ok(false);
    };
    let Some(item) = crate::items::get_item(item_id) else {
        return Ok(false);
    };

    append_blank_line(state, tooltip);
    if record.granted_by_essence {
        append_key_line(
            state,
            tooltip,
            "PVP_TALENT_GRANTED_BY_AZERITE_ESSENCE",
            "REFUND_TOOLTIP_COLOR",
            true,
        )?;
    } else {
        append_blank_line(state, tooltip);
        append_item_grant_line(state, tooltip, item.name, "REFUND_TOOLTIP_COLOR")?;
    }
    Ok(true)
}

#[cfg(feature = "client-retail")]
fn append_inspect_grant_line(state: &mut LuaState, tooltip: Val, talent_id: i32) -> LuaResult<()> {
    let item_id = {
        let sim = borrow_state(state)?;
        sim.inspect_specialization_cache
            .as_ref()
            .and_then(|(inspect_guid, _)| {
                sim.pvp_talent_tooltip
                    .inspect_grant_item_ids
                    .get(&(inspect_guid.clone(), talent_id))
                    .copied()
            })
    };
    let Some(item) = item_id.and_then(crate::items::get_item) else {
        return Ok(());
    };
    append_item_grant_line(state, tooltip, item.name, "ITEM_LEGENDARY_COLOR")
}

#[cfg(feature = "client-retail")]
fn append_item_grant_line(
    state: &mut LuaState,
    tooltip: Val,
    item_name: &str,
    color_key: &str,
) -> LuaResult<()> {
    let text = format_global_text(state, "TALENT_GRANTED_BY_ITEM", item_name)?;
    append_colored_text_line(state, tooltip, &text, color_key, true);
    Ok(())
}

#[cfg(feature = "client-retail")]
fn append_key_line(
    state: &mut LuaState,
    tooltip: Val,
    key: &str,
    color_key: &str,
    wrap: bool,
) -> LuaResult<()> {
    let text = global_text(state, key)
        .ok_or_else(|| rilua::runtime_error(format!("missing localized global {key}")))?;
    append_colored_text_line(state, tooltip, &text, color_key, wrap);
    Ok(())
}

#[cfg(feature = "client-retail")]
fn append_colored_text_line(
    state: &mut LuaState,
    tooltip: Val,
    text: &str,
    color_key: &str,
    wrap: bool,
) {
    let color = named_color(state, color_key);
    let line = append_line(state, tooltip, PLAIN_LINE_TYPE, text, color);
    if wrap {
        table_set(state, line, "wrapText", Val::Bool(true));
    }
}

#[cfg(feature = "client-retail")]
fn append_blank_line(state: &mut LuaState, tooltip: Val) {
    let color = named_color(state, "NORMAL_FONT_COLOR");
    append_line(state, tooltip, BLANK_LINE_TYPE, BLANK_LINE_TEXT, color);
}

// Retail SpecializationInfoDocumentation.lua:153-165,491-499.
#[cfg(feature = "client-retail")]
pub(crate) fn register_slot_info(state: &mut LuaState, namespace: GcRef<Table>) -> LuaResult<()> {
    table_set_rust_fn_static(state, namespace, "GetPvpTalentSlotInfo", get_slot_info)
}

#[cfg(feature = "client-wowforever")]
pub(crate) fn register_forever_slot_info(
    state: &mut LuaState,
    namespace: GcRef<Table>,
) -> LuaResult<()> {
    table_set_rust_fn_static(
        state,
        namespace,
        "GetPvpTalentSlotInfo",
        get_forever_slot_info,
    )
}

// INFERRED Forever record and missing-row behavior mirrors Retail. R88eq eq1.
#[cfg(feature = "client-wowforever")]
fn get_forever_slot_info(state: &mut LuaState) -> LuaResult<u32> {
    let slot_index = super::helpers::read_untainted_signed32_argument(
        state,
        1,
        "C_SpecializationInfo.GetPvpTalentSlotInfo",
        "Usage: local slotInfo = C_SpecializationInfo.GetPvpTalentSlotInfo(talentIndex)",
    )?;
    let (slot, selected) = {
        let sim = borrow_state(state)?;
        let selected_index = slot_index
            .checked_sub(1)
            .and_then(|index| usize::try_from(index).ok());
        let selected = selected_index
            .and_then(|index| sim.pvp_talents.selected_talent_ids.get(index).copied());
        (
            sim.pvp_talents.slot_infos.get(&slot_index).cloned(),
            selected,
        )
    };
    let result = if let Some(slot) = slot {
        push_forever_slot_info(state, &slot, selected)
    } else {
        crate::lua_api::workarounds::pvp_slot_default(
            state,
            usize::try_from(slot_index).unwrap_or(0),
        )
    };
    state.push(result);
    Ok(1)
}

#[cfg(feature = "client-wowforever")]
fn push_forever_slot_info(
    state: &mut LuaState,
    slot: &PvpTalentSlotInput,
    selected: Option<i32>,
) -> Val {
    let result = crate::lua_api::methods::create_table(state);
    table_set(state, result, "enabled", Val::Bool(slot.enabled));
    table_set(state, result, "level", Val::Num(f64::from(slot.level)));
    if let Some(id) = selected {
        table_set(state, result, "selectedTalentID", Val::Num(f64::from(id)));
    }
    let available = crate::lua_api::methods::create_table(state);
    for (position, id) in slot.available_talent_ids.iter().enumerate() {
        super::helpers::set_table_array(
            state,
            available,
            (position + 1) as i64,
            Val::Num(f64::from(*id)),
        );
    }
    table_set(state, result, "availableTalentIDs", available);
    result
}

#[cfg(feature = "client-retail")]
fn get_slot_info(state: &mut LuaState) -> LuaResult<u32> {
    use crate::lua_api::methods::create_table;
    // Preserve the old no-host call behavior; native argument failures are UNKNOWN.
    let value = allowed_when_untainted(state, 1, "C_SpecializationInfo.GetPvpTalentSlotInfo")?;
    let ui_index = match value {
        Val::Num(number)
            if number.fract() == 0.0 && (1.0..=f64::from(u32::MAX)).contains(&number) =>
        {
            number as usize
        }
        _ => {
            state.push(Val::Nil);
            return Ok(1);
        }
    };
    let index = ui_index - 1;
    let (slot, selected) = {
        let sim = borrow_state(state)?;
        let inputs = &sim.pvp_talent_tooltip;
        let selected = inputs
            .groups
            .get(&inputs.active_group_index)
            .and_then(|group| group.get(index))
            .copied()
            .flatten();
        (inputs.slots.get(&ui_index).cloned(), selected)
    };
    let result = if let Some(slot) = slot {
        let result = create_table(state);
        table_set(state, result, "enabled", Val::Bool(slot.enabled));
        table_set(state, result, "level", Val::Num(f64::from(slot.level)));
        if let Some(id) = selected {
            table_set(state, result, "selectedTalentID", Val::Num(f64::from(id)));
        }
        let available = create_table(state);
        for (position, id) in slot.available_talent_ids.iter().enumerate() {
            super::helpers::set_table_array(
                state,
                available,
                (position + 1) as i64,
                Val::Num(f64::from(*id)),
            );
        }
        table_set(state, result, "availableTalentIDs", available);
        result
    } else {
        crate::lua_api::workarounds::pvp_slot_default(state, ui_index)
    };
    state.push(result);
    Ok(1)
}
