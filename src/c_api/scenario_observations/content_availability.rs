use super::borrow_state;
use super::numbers;
use super::project;
use super::{borrow_state_mut, read_input, register_methods};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

#[derive(Debug, Default, serde::Deserialize)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct ContentAvailability {
    club_finder_shown: bool,
    reservoir_accessible: bool,
    sanctum_type: Option<u32>,
    invasions_in_player_area: bool,
    immersive_interaction_active: bool,
    raid_lock_extension_enabled: bool,
    available_campaign_ids: Vec<u32>,
    default_order_skill_line_id: Option<u32>,
    delve_season_affix_spell_ids: Vec<u32>,
}

pub(super) fn supply(state: &mut LuaState) -> LuaResult<u32> {
    let observation = read_input(state)?;
    borrow_state_mut(state)?
        .scenario_observations
        .content_availability = observation;
    Ok(0)
}

pub(super) fn register(state: &mut LuaState) -> LuaResult<()> {
    register_clubfinder(state)?;
    register_covenantsanctumui(state)?;
    register_invasioninfo(state)?;
    register_immersiveinteraction(state)?;
    register_raidlocks(state)?;
    register_campaigninfo(state)?;
    register_craftingorders(state)?;
    register_delvesui(state)?;
    Ok(())
}

fn register_clubfinder(state: &mut LuaState) -> LuaResult<()> {
    register_methods(
        state,
        "C_ClubFinder",
        &[
            // ClubFinderDocumentation.lua:620
            ("ShouldShowClubFinder", club_finder_shown),
        ],
    )
}

fn register_covenantsanctumui(state: &mut LuaState) -> LuaResult<()> {
    register_methods(
        state,
        "C_CovenantSanctumUI",
        &[
            // CovenantSanctumDocumentation.lua:11
            ("CanAccessReservoir", reservoir_accessible),
            // CovenantSanctumDocumentation.lua:105
            ("GetSanctumType", sanctum_type),
        ],
    )
}

fn register_invasioninfo(state: &mut LuaState) -> LuaResult<()> {
    register_methods(
        state,
        "C_InvasionInfo",
        &[
            // InvasionInfoDocumentation.lua:11
            ("AreInvasionsAvailable", invasions_in_player_area),
        ],
    )
}

fn register_immersiveinteraction(state: &mut LuaState) -> LuaResult<()> {
    register_methods(
        state,
        "C_ImmersiveInteraction",
        &[
            // ImmersiveInteractionManagerDocumentation.lua:11
            ("HasImmersiveInteraction", immersive_interaction_active),
        ],
    )
}

fn register_raidlocks(state: &mut LuaState) -> LuaResult<()> {
    register_methods(
        state,
        "C_RaidLocks",
        &[
            // RaidLocksDocumentation.lua:44
            (
                "IsRaidLockExtendFeatureEnabled",
                raid_lock_extension_enabled,
            ),
        ],
    )
}

fn register_campaigninfo(state: &mut LuaState) -> LuaResult<()> {
    register_methods(
        state,
        "C_CampaignInfo",
        &[
            // WarCampaignDocumentation.lua:11
            ("GetAvailableCampaigns", available_campaign_ids),
        ],
    )
}

fn register_craftingorders(state: &mut LuaState) -> LuaResult<()> {
    register_methods(
        state,
        "C_CraftingOrders",
        &[
            // CraftingOrderUIDocumentation.lua:163
            ("GetDefaultOrdersSkillLine", default_order_skill_line_id),
        ],
    )
}

fn register_delvesui(state: &mut LuaState) -> LuaResult<()> {
    register_methods(
        state,
        "C_DelvesUI",
        &[
            // DelvesUIDocumentation.lua:156
            (
                "GetDelvesAffixSpellsForSeason",
                delve_season_affix_spell_ids,
            ),
        ],
    )
}

fn club_finder_shown(state: &mut LuaState) -> LuaResult<u32> {
    project(state, |o| {
        Val::Bool(o.content_availability.club_finder_shown)
    })
}

fn reservoir_accessible(state: &mut LuaState) -> LuaResult<u32> {
    project(state, |o| {
        Val::Bool(o.content_availability.reservoir_accessible)
    })
}

fn sanctum_type(state: &mut LuaState) -> LuaResult<u32> {
    project(state, |o| {
        o.content_availability
            .sanctum_type
            .map_or(Val::Nil, |v| Val::Num(f64::from(v)))
    })
}

fn invasions_in_player_area(state: &mut LuaState) -> LuaResult<u32> {
    project(state, |o| {
        Val::Bool(o.content_availability.invasions_in_player_area)
    })
}

fn immersive_interaction_active(state: &mut LuaState) -> LuaResult<u32> {
    project(state, |o| {
        Val::Bool(o.content_availability.immersive_interaction_active)
    })
}

fn raid_lock_extension_enabled(state: &mut LuaState) -> LuaResult<u32> {
    project(state, |o| {
        Val::Bool(o.content_availability.raid_lock_extension_enabled)
    })
}

fn available_campaign_ids(state: &mut LuaState) -> LuaResult<u32> {
    let values = borrow_state(state)?
        .scenario_observations
        .content_availability
        .available_campaign_ids
        .clone();
    numbers(state, values)
}

fn default_order_skill_line_id(state: &mut LuaState) -> LuaResult<u32> {
    project(state, |o| {
        o.content_availability
            .default_order_skill_line_id
            .map_or(Val::Nil, |v| Val::Num(f64::from(v)))
    })
}

fn delve_season_affix_spell_ids(state: &mut LuaState) -> LuaResult<u32> {
    let values = borrow_state(state)?
        .scenario_observations
        .content_availability
        .delve_season_affix_spell_ids
        .clone();
    numbers(state, values)
}
