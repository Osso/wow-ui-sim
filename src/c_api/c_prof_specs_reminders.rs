//! `C_ProfSpecs` reminder queries (WM49 part A, Retail 12.1.5 build 70688).
//!
//! The simulator has no profession trait configuration, purchase-eligibility,
//! condition or cost model; hosts supply those results in
//! `SimState::profession_spec_inputs`. SkillLine relations, tabs and point
//! currencies come from 70391 data.

use super::profession_spec_inputs::SkillRank;
use crate::client_profile::{ACTIVE, ClientProfile};
use crate::lua_api::SimState;
use crate::lua_api::globals::profession_data::{PROFESSIONS, ProfessionInfo};
use crate::lua_api::methods::{borrow_state, create_string};
use crate::lua_bridge::table_set_rust_fn_static;
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};
use sim_data::profession_spec_reminders::{
    SKILL_LINE_TABS, SKILL_LINES, TAB_POINT_CURRENCIES, TabCurrency,
};

/// `Enum.ProfessionsSpecTabState.Unlockable`, the only state a name reminder accepts.
const TAB_STATE_UNLOCKABLE: i32 = 2;
/// `Enum.Profession` Fishing and Cooking, visited after the two stored professions.
const FALLBACK_PROFESSIONS: [i32; 2] = [10, 5];
const POINTS_REMINDER_THRESHOLD: i32 = 5;

pub(crate) fn register(state: &mut LuaState) -> LuaResult<()> {
    if !matches!(
        ACTIVE,
        ClientProfile::Retail | ClientProfile::Ptr | ClientProfile::WowForever
    ) {
        return Ok(());
    }
    // ProfessionSpecUIDocumentation.lua:142 and :344 in Retail.
    let namespace = super::ensure_namespace(state, "C_ProfSpecs")?;
    table_set_rust_fn_static(
        state,
        namespace,
        "GetNewSpecReminderProfName",
        get_new_spec_reminder_prof_name,
    )?;
    table_set_rust_fn_static(
        state,
        namespace,
        "ShouldShowPointsReminder",
        should_show_points_reminder,
    )
}

/// EMULATED (WM49-A): no match publishes one nil.
fn get_new_spec_reminder_prof_name(state: &mut LuaState) -> LuaResult<u32> {
    let name = new_spec_reminder_name(&*borrow_state(state)?);
    let value = match name {
        Some(name) => create_string(state, name),
        None => Val::Nil,
    };
    state.push(value);
    Ok(1)
}

fn should_show_points_reminder(state: &mut LuaState) -> LuaResult<u32> {
    let show = points_reminder(&*borrow_state(state)?);
    state.push(Val::Bool(show));
    Ok(1)
}

/// The first outer candidate with a new choice selects its own name; empty
/// text is selected too and stops the search.
fn new_spec_reminder_name(sim: &SimState) -> Option<&'static str> {
    candidates(sim)
        .into_iter()
        .filter(|&skill| is_learned(sim, skill) && has_new_spec_choice(sim, skill))
        .find_map(skill_line_name)
}

/// WM55-Q2: only an enabled, eligible candidate reads its currency, and a
/// low quantity ends that candidate alone.
fn points_reminder(sim: &SimState) -> bool {
    let inputs = &sim.profession_spec_inputs;
    candidates(sim).into_iter().any(|skill| {
        inputs.enabled_skill_lines.contains(&skill)
            && has_points_eligibility(sim, skill)
            && point_currency_quantity(sim, skill)
                .is_some_and(|quantity| quantity >= POINTS_REMINDER_THRESHOLD)
    })
}

/// The two stored professions, then the first SkillLine of `Enum.Profession` 10 and 5.
fn candidates(sim: &SimState) -> Vec<u32> {
    let stored = PROFESSIONS
        .iter()
        .filter(|profession| profession.parent_profession_name.is_empty())
        .filter(|profession| !is_unlearned(sim, profession))
        .take(2);
    let fallback = FALLBACK_PROFESSIONS.iter().filter_map(|&kind| {
        PROFESSIONS
            .iter()
            .find(|profession| profession.profession == kind)
    });
    stored
        .chain(fallback)
        .map(|profession| profession.skill_line_id as u32)
        .collect()
}

fn is_unlearned(sim: &SimState, profession: &ProfessionInfo) -> bool {
    sim.crafting
        .unlearned_profession_ids
        .contains(&profession.skill_line_id)
}

/// A positive SkillLine held with nonzero current and current-plus-bonus rank.
fn is_learned(sim: &SimState, skill: u32) -> bool {
    let base = PROFESSIONS
        .iter()
        .find(|profession| profession.skill_line_id as u32 == skill)
        .filter(|profession| !is_unlearned(sim, profession))
        .map(|profession| SkillRank {
            current: profession.skill_level,
            bonus: profession.skill_modifier,
        });
    let rank = base.or_else(|| {
        sim.profession_spec_inputs
            .related_skill_ranks
            .get(&skill)
            .copied()
    });
    skill > 0 && rank.is_some_and(|rank| rank.current > 0 && rank.current + rank.bonus > 0)
}

/// Own unlockable tab under the enabled flag, else a learned related line.
fn has_new_spec_choice(sim: &SimState, skill: u32) -> bool {
    let inputs = &sim.profession_spec_inputs;
    let own = inputs.enabled_skill_lines.contains(&skill)
        && tabs(skill).any(|tab| inputs.tab_states.get(&tab) == Some(&TAB_STATE_UNLOCKABLE));
    own || related(skill)
        .filter(|&related| is_learned(sim, related))
        .any(|related| has_new_spec_choice(sim, related))
}

/// WM55-Q3: points subscans apply no learned gate to related lines.
fn has_points_eligibility(sim: &SimState, skill: u32) -> bool {
    let eligible = &sim.profession_spec_inputs.points_eligible_skill_lines;
    eligible.contains(&skill) || related(skill).any(|related| eligible.contains(&related))
}

/// The original skill's tabs first; related lines only when it has no result.
fn point_currency(skill: u32) -> Option<&'static TabCurrency> {
    tabs(skill)
        .find_map(tab_currency)
        .or_else(|| related(skill).find_map(|related| tabs(related).find_map(tab_currency)))
}

/// Quantity from the current currency state, zero when absent.
fn point_currency_quantity(sim: &SimState, skill: u32) -> Option<i32> {
    let currency = point_currency(skill)?;
    let quantity = sim
        .currency_info
        .get(&(currency.currency_id as i32))
        .map_or(0, |info| info.quantity);
    Some(reminder_quantity(quantity, currency.divide_by_100))
}

/// CurrencyTypes flag 8 divides the signed quantity by 100.
fn reminder_quantity(quantity: i32, divide_by_100: bool) -> i32 {
    if divide_by_100 {
        quantity / 100
    } else {
        quantity
    }
}

/// WM55-Q4: direct children, by ParentTierIndex descending (the data order).
fn related(skill: u32) -> impl Iterator<Item = u32> {
    SKILL_LINES
        .iter()
        .filter(move |line| line.parent_id == skill)
        .map(|line| line.id)
}

fn tabs(skill: u32) -> impl Iterator<Item = u32> {
    SKILL_LINE_TABS
        .iter()
        .filter(move |(line, _)| *line == skill)
        .map(|(_, tab)| *tab)
}

fn tab_currency(tab: u32) -> Option<&'static TabCurrency> {
    let index = TAB_POINT_CURRENCIES
        .binary_search_by_key(&tab, |currency| currency.tab_id)
        .ok()?;
    Some(&TAB_POINT_CURRENCIES[index])
}

fn skill_line_name(skill: u32) -> Option<&'static str> {
    SKILL_LINES
        .iter()
        .find(|line| line.id == skill)
        .map(|line| line.name)
}

#[cfg(test)]
mod tests {
    use super::reminder_quantity;

    #[test]
    fn flag_8_divides_the_signed_quantity_by_100() {
        assert_eq!(reminder_quantity(499, true), 4);
        assert_eq!(reminder_quantity(500, true), 5);
        assert_eq!(reminder_quantity(-150, true), -1);
        assert_eq!(reminder_quantity(7, false), 7);
    }
}
