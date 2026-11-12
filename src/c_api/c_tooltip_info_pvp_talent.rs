//! Retail `C_TooltipInfo.GetPvpTalent` composition.
//!
//! R44-Q20, R53 and R56 establish the success path. PvP talent rows and
//! resolved spell IDs are host inputs; empty maps do not fabricate records or
//! override results. Native failure publication and override ranking remain
//! unknown.

use crate::c_api::tooltip_getter_args::{
    allowed_when_untainted, lua_index, round7_optional_bool, round7_signed,
};
use crate::lua_api::globals::missing_surface::tooltip_info::{
    append_line, format_global_numeric, format_global_text, global_text, named_color,
    restore_plain_line_type, tooltip_for_spell_identifier,
};
use crate::lua_api::methods::{borrow_state, table_set};
use crate::lua_bridge::{stack_val, table_set_rust_fn_static};
use rilua::vm::{gc::arena::GcRef, state::LuaState, table::Table};
use rilua::{LuaResult, Val};
use std::collections::HashMap;

const API_NAME: &str = "C_TooltipInfo.GetPvpTalent";
const USAGE: &str = "Usage: local data = C_TooltipInfo.GetPvpTalent(talentID [, isInspect, groupIndex, talentIndex])";
const PLAIN_LINE_TYPE: f64 = 0.0;
const BLANK_LINE_TYPE: f64 = 1.0;
const TALENT_POSITION_COUNT: i32 = 4;
const SUPPRESSED_OVERRIDE_ATTRIBUTE: u32 = 32768;
const BLANK_LINE_TEXT: &str = " ";

/// One host-supplied PvP talent row. The pinned CSV export has no PvP talent
/// table, so the simulator does not seed records.
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
    /// Granting items in the inspected-unit context, keyed by GUID and talent ID.
    pub inspect_grant_item_ids: HashMap<(String, i32), u32>,
}

impl Default for PvpTalentTooltipInputs {
    fn default() -> Self {
        Self {
            catalog: HashMap::new(),
            active_group_index: 0,
            groups: HashMap::new(),
            resolved_spell_ids: HashMap::new(),
            inspect_grant_item_ids: HashMap::new(),
        }
    }
}

pub(crate) fn register(state: &mut LuaState, namespace: GcRef<Table>) -> LuaResult<()> {
    table_set_rust_fn_static(state, namespace, "GetPvpTalent", get_pvp_talent)
}

fn get_pvp_talent(state: &mut LuaState) -> LuaResult<u32> {
    let talent_id = round7_signed(state, 1, USAGE)?;
    let is_inspect = round7_optional_bool(state, 2, USAGE)?;
    let group_value = allowed_when_untainted(state, 3, API_NAME)?;
    let group_was_supplied = !group_value.is_nil();
    let parsed_group = if group_was_supplied {
        lua_index(state, 3)?
    } else {
        None
    };
    let talent_position = if stack_val(state, 4).is_nil() {
        0
    } else {
        round7_signed(state, 4, USAGE)?
    };

    if talent_id == 0 {
        return crate::c_api::tooltip_data::unknown_getter_failure();
    }

    let (record, active_group, player_present) = {
        let sim = borrow_state(state)?;
        (
            sim.pvp_talent_tooltip.catalog.get(&talent_id).cloned(),
            sim.pvp_talent_tooltip.active_group_index,
            sim.player.present,
        )
    };
    let Some(record) = record else {
        return crate::c_api::tooltip_data::unknown_getter_failure();
    };
    if crate::spells::get_spell(record.spell_id).is_none() {
        return crate::c_api::tooltip_data::unknown_getter_failure();
    }

    let resolved_spell_id = resolve_spell_id(state, record.spell_id, None)?;
    if !player_present || crate::spells::get_spell(resolved_spell_id).is_none() {
        // R53 confirms annotation continuation, but the public result is UNKNOWN.
        return crate::c_api::tooltip_data::unknown_getter_failure();
    }

    let tooltip = tooltip_for_spell_identifier(state, Some(resolved_spell_id), false);
    append_replacement_line(state, tooltip, &record)?;
    let local_group = if is_inspect {
        None
    } else if group_was_supplied {
        parsed_group
    } else {
        Some(active_group)
    };
    append_talent_annotation(
        state,
        tooltip,
        talent_id,
        talent_position,
        is_inspect,
        local_group,
        &record,
    )?;
    state.push(tooltip);
    Ok(1)
}

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
    let position_mask = 1_u8 << talent_position;
    if record.category_position_mask & position_mask == 0 {
        return Ok(());
    }

    match record.eligibility_by_position.get(&talent_position) {
        Some(true) => append_key_line(
            state,
            tooltip,
            "TALENT_TOOLTIP_ADDPREVIEWPOINT",
            "FRIENDLY_STATUS_COLOR",
            true,
        ),
        Some(false) => {
            if record.eligibility_level_failed {
                append_unlock_line(state, tooltip, record.level_required, true)
            } else {
                append_local_failure_line(state, tooltip, context, record)
            }
        }
        None => Ok(()),
    }
}

#[derive(Clone, Copy)]
struct LocalAnnotationContext {
    active_group_index: usize,
    player_level: i32,
    in_combat: bool,
    is_resting: bool,
    rest_state: i32,
    selected_talents: [Option<i32>; TALENT_POSITION_COUNT as usize],
}

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
        is_resting: sim.player.is_resting,
        rest_state: sim.player_xp.rest_state,
        selected_talents,
    }))
}

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

fn append_local_failure_line(
    state: &mut LuaState,
    tooltip: Val,
    context: LocalAnnotationContext,
    record: &PvpTalentTooltipRecord,
) -> LuaResult<()> {
    if append_local_grant_line(state, tooltip, record)? {
        return Ok(());
    }
    if context.in_combat {
        return append_key_line(
            state,
            tooltip,
            "TALENT_TOOLTIP_ADD_COMBAT_ERROR",
            "HOSTILE_STATUS_COLOR",
            true,
        );
    }
    if !rest_change_allowed(context) {
        return append_key_line(
            state,
            tooltip,
            "TALENT_TOOLTIP_ADD_REST_ERROR_PVP",
            "HOSTILE_STATUS_COLOR",
            true,
        );
    }
    // The R56 default for other helper failures is known, but those failure
    // predicates are UNKNOWN and stay unmodelled.
    Ok(())
}

fn rest_change_allowed(context: LocalAnnotationContext) -> bool {
    (context.rest_state > 0 && !context.in_combat) || context.is_resting
}

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

fn append_colored_text_line(
    state: &mut LuaState,
    tooltip: Val,
    text: &str,
    color_key: &str,
    wrap: bool,
) {
    let color = named_color(state, color_key);
    let line = append_line(state, tooltip, PLAIN_LINE_TYPE, text, color);
    restore_plain_line_type(state, line);
    if wrap {
        table_set(state, line, "wrapText", Val::Bool(true));
    }
}

fn append_blank_line(state: &mut LuaState, tooltip: Val) {
    let color = named_color(state, "NORMAL_FONT_COLOR");
    append_line(state, tooltip, BLANK_LINE_TYPE, BLANK_LINE_TEXT, color);
}
