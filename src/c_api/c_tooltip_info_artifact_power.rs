//! WM108 §A: `GetArtifactPowerByID` selects one artifact-power rank row and
//! composes the name, rank row, spacer and spell text.
//!
//! Retail/PTR/Forever `TooltipInfoDocumentation.lua:53` declare the getter.
//! Publication on PTR and Forever is **INFERRED** shared-model reuse.
//!
//! Modelled from WM108 §A and WM94 §5:
//! - the power record supplies `ArtifactID`, `MaxPurchasableRank`, `Flags` and
//!   `Tier`; the selected rank is the stored rank, a stored zero selects rank 1,
//!   and `Flags & 0x40` forces rank 1;
//! - the rank record (`RankIndex == selected - 1`) supplies the spell ID, and a
//!   missing power or rank record is a miss;
//! - the rank row is emitted only when neither `0x40` nor `0x4` is set; it shows
//!   the stored rank and the displayed maximum, and a nonzero bonus selects
//!   `INCREASE_STAT_COLOR` instead of `HIGHLIGHT_FONT_COLOR`;
//! - the displayed maximum is `MaxPurchasableRank`, plus the artifact tier minus
//!   the power tier when `0x20` is set, and 255 when `0x8` is set; a nonzero
//!   bonus is reduced by one before the colour test when `0x10` is set;
//! - the spacer before the spell text exists only when the rank row is shown.
//!
//! EMULATED seams, each named:
//! - the footer gate: the pinned export has no `Artifact` table, so the simulator
//!   cannot resolve an artifact object to `ArtifactCategoryID == 2`. WM108 §A2
//!   makes the footer-suppression query true in that case, so every footer row —
//!   cost, learn, end-game unlinked and unlinked — is skipped. Affordability,
//!   `ArtifactCanLearn` and `ArtifactOtherLinked` therefore stay unreached rather
//!   than invented.
//! - the "local player-state bit" that selects the raw maximum instead of the
//!   capped maximum is not identified in the evidence, so the capped maximum is
//!   published.
//! - the published `data.type` is not recorded by WM94/WM108. The simulator
//!   publishes 1 (`Enum.TooltipDataType.Spell`) because the composition is a name
//!   plus spell text; replace it when native evidence lands.
//! - the rank row's line type is not recorded either; the producer owns a plain
//!   row and restores that type after the shared colour remap.

use crate::c_api::tooltip_getter_args::signed;
use crate::c_api::tooltip_power_catalog_data::{ARTIFACT_POWER_RANKS, ARTIFACT_POWERS};
use crate::lua_api::globals::missing_surface::tooltip_info::{
    append_line, compose_tooltip, format_global_pair, named_color, restore_plain_line_type,
    spell_description_text,
};
use crate::lua_api::methods::borrow_state;
use crate::lua_bridge::table_set_rust_fn_static;
use rilua::LuaResult;
use rilua::vm::{gc::arena::GcRef, state::LuaState, table::Table};

/// EMULATED: WM94/WM108 leave the published data type UNKNOWN.
const TOOLTIP_TYPE_ARTIFACT_POWER: f64 = 1.0;
const LINE_TYPE_SPELL_NAME: f64 = 13.0;
const LINE_TYPE_PLAIN: f64 = 0.0;
const LINE_TYPE_BLANK: f64 = 1.0;
const LINE_TYPE_SPELL_DESCRIPTION: f64 = 34.0;
const BLANK_TEXT: &str = " ";
/// WM108 §A1 masks read by this tooltip path.
const FLAG_RANK_ONE: u32 = 0x40;
const FLAG_END_GAME_UNLINKED: u32 = 0x4;
const FLAG_RANK_LIMIT_UNCAPPED: u32 = 0x8;
const FLAG_BONUS_ADJUST: u32 = 0x10;
const FLAG_TIER_SCALED_MAX: u32 = 0x20;
/// WM108 §A1: the rank-limit helper substitutes 255.
const RANK_LIMIT_UNCAPPED: i64 = 255;
pub(crate) fn register(state: &mut LuaState, namespace: GcRef<Table>) -> LuaResult<()> {
    table_set_rust_fn_static(
        state,
        namespace,
        "GetArtifactPowerByID",
        get_artifact_power_by_id,
    )
}

fn get_artifact_power_by_id(state: &mut LuaState) -> LuaResult<u32> {
    let power_id = signed(state, 1)?;
    let Some(power) = power_record(power_id) else {
        return crate::c_api::tooltip_data::unknown_getter_failure();
    };
    let stored = stored_power(state, power_id)?;
    let selected_rank = selected_rank(power.flags, stored.current_rank);
    let Some(spell_id) = rank_spell(power_id, selected_rank) else {
        return crate::c_api::tooltip_data::unknown_getter_failure();
    };
    let tooltip = compose_tooltip(state, TOOLTIP_TYPE_ARTIFACT_POWER, None);
    if let Some(spell) = crate::spells::get_spell(spell_id) {
        append_line(state, tooltip, LINE_TYPE_SPELL_NAME, spell.name, None);
    }
    let hides_rank_row = power.flags & (FLAG_RANK_ONE | FLAG_END_GAME_UNLINKED) != 0;
    if !hides_rank_row {
        let maximum = displayed_maximum(state, &power)?;
        let text = format_global_pair(
            state,
            "TOOLTIP_TALENT_RANK",
            f64::from(stored.current_rank),
            maximum as f64,
        )?;
        let colour = rank_row_color(state, power.flags, stored.bonus_ranks);
        let line = append_line(state, tooltip, LINE_TYPE_PLAIN, &text, colour);
        restore_plain_line_type(state, line);
        append_line(state, tooltip, LINE_TYPE_BLANK, BLANK_TEXT, None);
    }
    let description = spell_description_text(state, spell_id);
    if !description.is_empty() {
        let green = named_color(state, "GREEN_FONT_COLOR");
        append_line(
            state,
            tooltip,
            LINE_TYPE_SPELL_DESCRIPTION,
            &description,
            green,
        );
    }
    state.push(tooltip);
    Ok(1)
}

struct PowerRecord {
    max_purchasable_rank: u32,
    flags: u32,
    tier: u32,
}

fn power_record(power_id: i32) -> Option<PowerRecord> {
    let power_id = u32::try_from(power_id).ok()?;
    let index = ARTIFACT_POWERS
        .binary_search_by_key(&power_id, |&(id, _, _, _, _)| id)
        .ok()?;
    let (_, _artifact_id, max_purchasable_rank, flags, tier) = ARTIFACT_POWERS[index];
    Some(PowerRecord {
        max_purchasable_rank,
        flags,
        tier,
    })
}

/// The stored totals for one power. Missing state supplies zero counts, which is
/// not an unconditional rejection (WM94 §5).
fn stored_power(
    state: &LuaState,
    power_id: i32,
) -> LuaResult<crate::lua_api::state::ArtifactPowerInfo> {
    let sim = borrow_state(state)?;
    Ok(sim
        .viewed_artifact
        .powers
        .get(&power_id)
        .cloned()
        .unwrap_or_default())
}

fn selected_rank(flags: u32, stored_rank: i32) -> i32 {
    if flags & FLAG_RANK_ONE != 0 || stored_rank == 0 {
        1
    } else {
        stored_rank
    }
}

fn rank_spell(power_id: i32, rank: i32) -> Option<u32> {
    let power_id = u32::try_from(power_id).ok()?;
    let rank_index = u8::try_from(rank.checked_sub(1)?).ok()?;
    let key = (power_id, rank_index);
    ARTIFACT_POWER_RANKS
        .binary_search_by_key(&key, |&(id, index, _)| (id, index))
        .ok()
        .map(|index| ARTIFACT_POWER_RANKS[index].2)
}

/// `MaxPurchasableRank` plus the tier adjustment and the 255 rank limit.
fn displayed_maximum(state: &LuaState, power: &PowerRecord) -> LuaResult<i64> {
    let sim = borrow_state(state)?;
    let artifact_tier = sim
        .viewed_artifact
        .info
        .as_ref()
        .map_or(0, |info| i64::from(info.tier));
    let mut maximum = i64::from(power.max_purchasable_rank);
    if power.flags & FLAG_TIER_SCALED_MAX != 0 {
        maximum += artifact_tier - i64::from(power.tier);
    }
    if power.flags & FLAG_RANK_LIMIT_UNCAPPED != 0 {
        maximum = RANK_LIMIT_UNCAPPED;
    }
    Ok(maximum.clamp(0, RANK_LIMIT_UNCAPPED))
}

/// `INCREASE_STAT_COLOR` for a nonzero displayed bonus, else `HIGHLIGHT_FONT_COLOR`.
fn rank_row_color(state: &mut LuaState, flags: u32, stored_bonus: i32) -> Option<(f64, f64, f64)> {
    let bonus = if flags & FLAG_BONUS_ADJUST != 0 && stored_bonus > 0 {
        stored_bonus - 1
    } else {
        stored_bonus
    };
    if bonus != 0 {
        named_color(state, "INCREASE_STAT_COLOR")
    } else {
        named_color(state, "HIGHLIGHT_FONT_COLOR")
    }
}
