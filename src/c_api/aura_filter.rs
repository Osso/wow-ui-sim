//! Aura filter strings (`AuraUtil.AuraFilters`) over explicit aura and raid state.
//!
//! Components split on `|` and spaces, case-insensitively. Retail 12.1.0 adds
//! `!` negation; MAW ignores it, as documented beside
//! `AuraUtil.AuraFilterNegationPrefix`. Components without backing state
//! (CANCELABLE, CROWD_CONTROL, BIG_DEFENSIVE, RAID_IN_COMBAT,
//! INCLUDE_NAME_PLATE_ONLY) and unknown components impose no restriction.

use crate::lua_api::game_data::{AuraFilterFacts, AuraInfo};
use crate::lua_api::methods::borrow_state;
use rilua::vm::state::LuaState;
use std::collections::HashSet;

const PATCH_12_1_0_FILTERS: bool = cfg!(feature = "retail-12-1-0");

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum FilterToken {
    Helpful,
    Harmful,
    Player,
    Raid,
    Maw,
    ExternalDefensive,
    Dispellable,
    Important,
    RaidPlayerDispellable,
}

#[derive(Clone, Copy)]
pub(crate) struct FilterTerm {
    pub token: FilterToken,
    pub negated: bool,
}

/// Unit-scoped facts consulted by state-dependent components.
pub(crate) struct AuraFilterContext {
    facts: AuraFilterFacts,
    unit_is_enemy: bool,
}

impl AuraFilterContext {
    pub(crate) fn for_unit(state: &mut LuaState, unit: &str) -> Self {
        let facts = borrow_state(state)
            .map(|sim| sim.aura_filter_facts.clone())
            .unwrap_or_default();
        let unit_is_enemy = crate::lua_api::globals::group_queries::is_attackable_unit(state, unit)
            .unwrap_or(false);
        Self {
            facts,
            unit_is_enemy,
        }
    }
}

pub(crate) fn parse_filter_terms(filter: &str) -> Vec<FilterTerm> {
    filter
        .split(['|', ' '])
        .filter(|component| !component.is_empty())
        .filter_map(parse_component)
        .collect()
}

fn parse_component(component: &str) -> Option<FilterTerm> {
    let (negated, name) = match component.strip_prefix('!') {
        Some(name) if PATCH_12_1_0_FILTERS => (true, name),
        Some(_) => return None,
        None => (false, component),
    };
    let token = match name.to_ascii_uppercase().as_str() {
        "HELPFUL" => FilterToken::Helpful,
        "HARMFUL" => FilterToken::Harmful,
        "PLAYER" => FilterToken::Player,
        "RAID" => FilterToken::Raid,
        "MAW" => FilterToken::Maw,
        "EXTERNAL_DEFENSIVE" => FilterToken::ExternalDefensive,
        "DISPELLABLE" if PATCH_12_1_0_FILTERS => FilterToken::Dispellable,
        "IMPORTANT" if PATCH_12_1_0_FILTERS => FilterToken::Important,
        "RAID_PLAYER_DISPELLABLE" => FilterToken::RaidPlayerDispellable,
        _ => return None,
    };
    let negated = negated && token != FilterToken::Maw;
    Some(FilterTerm { token, negated })
}

/// Harmful when HARMFUL is required or HELPFUL is excluded.
pub(crate) fn is_harmful_filter(terms: &[FilterTerm]) -> bool {
    terms.iter().any(|term| match term.token {
        FilterToken::Harmful => !term.negated,
        FilterToken::Helpful => term.negated,
        _ => false,
    })
}

pub(crate) fn aura_matches_terms(
    aura: &AuraInfo,
    terms: &[FilterTerm],
    context: &AuraFilterContext,
) -> bool {
    let has_polarity = terms
        .iter()
        .any(|term| matches!(term.token, FilterToken::Helpful | FilterToken::Harmful));
    // Retained simulator default: a filter without polarity selects helpful auras.
    (has_polarity || aura.is_helpful)
        && terms
            .iter()
            .all(|term| token_matches(aura, term.token, context) != term.negated)
}

fn token_matches(aura: &AuraInfo, token: FilterToken, context: &AuraFilterContext) -> bool {
    match token {
        FilterToken::Helpful => aura.is_helpful,
        FilterToken::Harmful => !aura.is_helpful,
        FilterToken::Player => aura.is_from_player_or_player_pet,
        FilterToken::Raid => aura.is_raid,
        // No MAW or external-defensive aura classification is modeled.
        FilterToken::Maw | FilterToken::ExternalDefensive => false,
        FilterToken::Dispellable => aura.dispel_type.is_some(),
        FilterToken::Important => context.facts.important_spell_ids.contains(&aura.spell_id),
        FilterToken::RaidPlayerDispellable => raid_can_dispel(aura, context),
    }
}

/// Harmful auras on friendly units with a raid-removable dispel type; from
/// 12.1.0 also helpful auras on enemies that a raid member can purge or steal.
fn raid_can_dispel(aura: &AuraInfo, context: &AuraFilterContext) -> bool {
    let facts = &context.facts;
    let dispel_in = |types: &HashSet<String>| {
        aura.dispel_type
            .as_ref()
            .is_some_and(|dispel| types.contains(dispel))
    };
    match (aura.is_helpful, context.unit_is_enemy) {
        (false, false) => dispel_in(&facts.raid_defensive_dispel_types),
        (true, true) => {
            PATCH_12_1_0_FILTERS
                && (dispel_in(&facts.raid_offensive_dispel_types)
                    || aura.is_stealable && facts.raid_can_spellsteal)
        }
        // INFERRED: harmful auras on enemies and helpful auras on friends are
        // never raid-dispellable.
        _ => false,
    }
}
