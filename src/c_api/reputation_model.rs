//! Host-fed modern reputation rows. No faction or guild data is seeded.
//!
//! Row selection/grouping is a simulator projection, not DB2 acquisition.

use super::c_reputation::{
    is_account_wide_reputation, is_faction_active, read_faction_sort_index, read_required_number,
    set_faction_active, set_legacy_reputations_shown, set_reputation_sort_type,
};
use crate::lua_api::methods::{
    borrow_state, borrow_state_mut, create_string, create_table, table_set_static,
};
use crate::lua_bridge::table_set_rust_fn_static;
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val, runtime_error};

pub(crate) const fn active() -> bool {
    use crate::client_profile::{ACTIVE, ClientProfile};
    matches!(ACTIVE, ClientProfile::Retail | ClientProfile::WowForever)
}

#[derive(Clone, Debug, Default)]
pub struct ReputationFaction {
    pub faction_id: i32,
    pub name: Option<String>,
    pub description: Option<String>,
    pub reaction: i32,
    pub bottom: i32,
    pub top: i32,
    pub standing: i32,
    pub at_war: bool,
    pub can_toggle_at_war: bool,
    pub is_child: bool,
    pub is_header: bool,
    pub is_header_with_rep: bool,
    pub is_collapsed: bool,
    pub has_bonus_rep_gain: bool,
    pub can_set_inactive: bool,
    pub is_account_wide: bool,
    pub is_inactive: bool,
    pub reputation_index: Option<u16>,
}

#[derive(Debug)]
pub struct ReputationModel {
    /// Explicit host inputs. Order and group headers are supplied by the host.
    pub factions: Vec<ReputationFaction>,
    /// Explicit host input, separately from the visible list.
    pub guild: Option<ReputationFaction>,
    pub selected_faction_id: Option<i32>,
    pub watched_faction_id: Option<i32>,
    pub sort_type: i32,
    pub show_legacy: bool,
}

impl Default for ReputationModel {
    fn default() -> Self {
        Self {
            factions: Vec::new(),
            guild: None,
            selected_faction_id: None,
            watched_faction_id: None,
            sort_type: 0,
            // Preserve the existing simulator preference, not native evidence.
            show_legacy: true,
        }
    }
}

impl ReputationModel {
    // INFERRED: an ordinary supplied header owns following child rows.
    pub(super) fn visible_indices(&self) -> impl Iterator<Item = usize> + '_ {
        let mut collapsed = false;
        self.factions
            .iter()
            .enumerate()
            .filter_map(move |(i, row)| {
                if row.is_header {
                    collapsed = row.is_collapsed;
                }
                (row.is_header || !row.is_child || !collapsed).then_some(i)
            })
    }

    pub(super) fn position(&self, index: i32) -> Option<usize> {
        self.visible_indices().nth(usize::try_from(index).ok()?)
    }

    pub fn faction(&self, id: i32) -> Option<&ReputationFaction> {
        self.factions.iter().find(|row| {
            row.faction_id == id && (!row.is_header || row.is_header_with_rep || id <= 0)
        })
    }
}

fn push_num(state: &mut LuaState, table: Val, key: &'static str, value: i32) {
    table_set_static(state, table, key, Val::Num(f64::from(value)));
}

// VERIFIED field shape: ReputationInfoDocumentation.lua FactionData on
// both pins, and R88f.A.59/A.60/A.73 on Retail 12.1.5.70688.
fn publish_row(state: &mut LuaState, row: ReputationFaction, watched: Option<i32>) -> Val {
    let table = create_table(state);
    state.push(table);
    if let Some(name) = row.name.as_deref() {
        let name = create_string(state, name);
        table_set_static(state, table, "name", name);
    }
    if let Some(description) = row.description.as_deref() {
        let description = create_string(state, description);
        table_set_static(state, table, "description", description);
    }
    push_num(state, table, "factionID", row.faction_id);
    push_num(state, table, "reaction", row.reaction);
    push_num(state, table, "currentReactionThreshold", row.bottom);
    push_num(state, table, "nextReactionThreshold", row.top);
    push_num(state, table, "currentStanding", row.standing);
    publish_flags(state, table, &row, watched);
    state.top -= 1;
    table
}

fn publish_flags(state: &mut LuaState, table: Val, row: &ReputationFaction, watched: Option<i32>) {
    for (key, value) in [
        ("atWarWith", row.at_war),
        ("canToggleAtWar", row.can_toggle_at_war),
        ("isChild", row.is_child),
        ("isHeader", row.is_header),
        ("isHeaderWithRep", row.is_header_with_rep),
        ("isCollapsed", row.is_collapsed),
        (
            "isWatched",
            watched == Some(row.faction_id) && row.faction_id > 0,
        ),
        ("hasBonusRepGain", row.has_bonus_rep_gain),
        ("canSetInactive", row.can_set_inactive),
        ("isAccountWide", row.is_account_wide),
    ] {
        table_set_static(state, table, key, Val::Bool(value));
    }
}

fn push_row(state: &mut LuaState, row: Option<ReputationFaction>) -> LuaResult<u32> {
    let watched = borrow_state(state)?.reputation.watched_faction_id;
    let output = row.map_or(Val::Nil, |row| publish_row(state, row, watched));
    state.push(output);
    Ok(1)
}

fn num_factions(state: &mut LuaState) -> LuaResult<u32> {
    let count = borrow_state(state)?.reputation.visible_indices().count();
    state.push(Val::Num(count as f64));
    Ok(1)
}

fn row_by_id(state: &LuaState, id: i32) -> LuaResult<Option<ReputationFaction>> {
    Ok(borrow_state(state)?.reputation.faction(id).cloned())
}

fn row_by_index(state: &LuaState, index: Option<i32>) -> LuaResult<Option<ReputationFaction>> {
    let sim = borrow_state(state)?;
    let model = &sim.reputation;
    Ok(index
        .and_then(|index| model.position(index))
        .and_then(|i| model.factions.get(i))
        .cloned())
}

fn faction_by_index(state: &mut LuaState) -> LuaResult<u32> {
    let index = read_faction_sort_index(
        state,
        "Usage: local factionData = C_Reputation.GetFactionDataByIndex(factionSortIndex)",
    )?;
    let row = if index.is_some() {
        row_by_index(state, index)?
    } else {
        row_by_id(state, 0)?
    };
    push_row(state, row)
}

fn required_id(state: &mut LuaState, usage: &str) -> LuaResult<i32> {
    let num = read_required_number(state, usage)?;
    if num < f64::from(i32::MIN) || num > f64::from(i32::MAX) {
        return Err(runtime_error(format!(
            "outside of expected range {} to {}",
            i32::MIN,
            i32::MAX
        )));
    }
    if !num.is_finite() {
        return Err(runtime_error(usage));
    }
    Ok(num as i32)
}

fn faction_by_id(state: &mut LuaState) -> LuaResult<u32> {
    let id = required_id(
        state,
        "Usage: local factionData = C_Reputation.GetFactionDataByID(factionID)",
    )?;
    let row = row_by_id(state, id)?;
    push_row(state, row)
}

fn guild(state: &mut LuaState) -> LuaResult<u32> {
    let row = borrow_state(state)?.reputation.guild.clone();
    push_row(state, row)
}

fn watched(state: &mut LuaState) -> LuaResult<u32> {
    let id = borrow_state(state)?.reputation.watched_faction_id;
    let row = match id {
        Some(id) => row_by_id(state, id)?,
        None => None,
    };
    push_row(state, row)
}

fn select_index(state: &mut LuaState) -> LuaResult<u32> {
    let index = read_faction_sort_index(
        state,
        "Usage: C_Reputation.SetSelectedFaction(factionSortIndex)",
    )?;
    let id = row_by_index(state, index)?.map(|row| row.faction_id);
    borrow_state_mut(state)?.reputation.selected_faction_id = id;
    Ok(0)
}

fn selected_index(state: &mut LuaState) -> LuaResult<u32> {
    let index = {
        let sim = borrow_state(state)?;
        let model = &sim.reputation;
        model
            .visible_indices()
            .position(|i| model.selected_faction_id == Some(model.factions[i].faction_id))
            .map_or(0, |i| i + 1)
    };
    state.push(Val::Num(index as f64));
    Ok(1)
}

fn assign_watched(state: &mut LuaState, id: Option<i32>) -> LuaResult<u32> {
    let mut sim = borrow_state_mut(state)?;
    let model = &mut sim.reputation;
    model.watched_faction_id = id.filter(|&id| id > 0 && model.faction(id).is_some());
    Ok(0)
}

fn watch_id(state: &mut LuaState) -> LuaResult<u32> {
    let id = required_id(
        state,
        "Usage: C_Reputation.SetWatchedFactionByID(factionID)",
    )?;
    assign_watched(state, Some(id))
}

fn watch_index(state: &mut LuaState) -> LuaResult<u32> {
    let index = read_faction_sort_index(
        state,
        "Usage: C_Reputation.SetWatchedFactionByIndex(factionSortIndex)",
    )?;
    let id = row_by_index(state, index)?.map(|row| row.faction_id);
    assign_watched(state, id)
}

fn all_headers(state: &mut LuaState, collapsed: bool) -> LuaResult<u32> {
    for row in &mut borrow_state_mut(state)?.reputation.factions {
        if row.is_header {
            row.is_collapsed = collapsed;
        }
    }
    Ok(0)
}

fn change_header(state: &mut LuaState, collapsed: bool, usage: &str) -> LuaResult<u32> {
    let index = read_faction_sort_index(state, usage)?;
    let mut sim = borrow_state_mut(state)?;
    let model = &mut sim.reputation;
    if let Some(i) = index.and_then(|i| model.position(i))
        && model.factions[i].is_header
    {
        model.factions[i].is_collapsed = collapsed;
    }
    Ok(0)
}

fn collapse(state: &mut LuaState) -> LuaResult<u32> {
    change_header(
        state,
        true,
        "Usage: C_Reputation.CollapseFactionHeader(factionSortIndex)",
    )
}

fn expand(state: &mut LuaState) -> LuaResult<u32> {
    let usage = "Usage: C_Reputation.ExpandFactionHeader(factionSortIndex)";
    if read_required_number(state, usage)? == 0.0 {
        return all_headers(state, false);
    }
    change_header(state, false, usage)
}

fn collapse_all(state: &mut LuaState) -> LuaResult<u32> {
    all_headers(state, true)
}
fn expand_all(state: &mut LuaState) -> LuaResult<u32> {
    all_headers(state, false)
}

fn sort_type(state: &mut LuaState) -> LuaResult<u32> {
    let sort = borrow_state(state)?.reputation.sort_type;
    state.push(Val::Num(f64::from(sort)));
    Ok(1)
}

fn show_legacy(state: &mut LuaState) -> LuaResult<u32> {
    let shown = borrow_state(state)?.reputation.show_legacy;
    state.push(Val::Bool(shown));
    Ok(1)
}

// These methods had no stateful behavior in the temporary fixture. The
// request/preload and native war-toggle lifecycle are outside this model.
fn request_reward(_state: &mut LuaState) -> LuaResult<u32> {
    Ok(0)
}

const METHODS: &[(&str, fn(&mut LuaState) -> LuaResult<u32>)] = &[
    ("GetNumFactions", num_factions),
    ("GetFactionDataByIndex", faction_by_index),
    ("GetFactionDataByID", faction_by_id),
    // INFERRED: retain the existing compatibility alias over host data.
    ("GetFactionInfo", faction_by_index),
    ("GetGuildFactionData", guild),
    ("GetWatchedFactionData", watched),
    ("GetSelectedFaction", selected_index),
    ("SetSelectedFaction", select_index),
    ("SetWatchedFactionByID", watch_id),
    ("SetWatchedFactionByIndex", watch_index),
    ("CollapseFactionHeader", collapse),
    ("ExpandFactionHeader", expand),
    ("CollapseAllFactionHeaders", collapse_all),
    ("ExpandAllFactionHeaders", expand_all),
    ("GetReputationSortType", sort_type),
    ("AreLegacyReputationsShown", show_legacy),
    ("ToggleFactionAtWar", request_reward),
    ("SetLegacyReputationsShown", set_legacy_reputations_shown),
    ("SetReputationSortType", set_reputation_sort_type),
    ("IsAccountWideReputation", is_account_wide_reputation),
    ("IsFactionActive", is_faction_active),
    ("SetFactionActive", set_faction_active),
    ("RequestFactionParagonPreloadRewardData", request_reward),
];

pub(crate) fn register(
    state: &mut LuaState,
    table: rilua::vm::gc::arena::GcRef<rilua::vm::table::Table>,
) -> LuaResult<()> {
    for &(name, method) in METHODS {
        table_set_rust_fn_static(state, table, name, method)?;
    }
    Ok(())
}
