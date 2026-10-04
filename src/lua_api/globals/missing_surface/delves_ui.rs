//! `C_DelvesUI` probe surface for delves startup widgets and tests.

use super::{ensure_namespace, set_table_array};
use crate::lua_api::methods::{create_string, create_table, table_set};
use crate::lua_bridge::{FromStack, table_set_rust_fn_static};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

const ACTIVE_TIER_FIELD: &str = "__wow_ui_sim_active_tier";
const DEFAULT_ACTIVE_TIER: i32 = 4;
const WORLD_TIER_NORMAL: f64 = 1.0;
const DELVE_HEADER: &str = "Fungal Folly";
const DELVE_DESCRIPTION: &str = "The Fungal Folly winds deeper with every tier.";
const DELVE_ENTRANCE_MAP_ID: i32 = 2339;
const LOCKED_TIER_REASON: &str = "Complete Tier 4 to unlock this delve tier.";
const UNKNOWN_TIER_REASON: &str = "Unknown tier";
#[cfg(feature = "retail-12-1-0")]
const DELVE_ITEM_REWARD_ID: i32 = 228361;
#[cfg(feature = "retail-12-1-0")]
const DELVE_CURRENCY_REWARD_ID: i32 = 2815;
#[cfg(feature = "retail-12-1-0")]
const TIER_REWARD_ITEM: i32 = 0;
#[cfg(feature = "retail-12-1-0")]
const TIER_REWARD_CURRENCY: i32 = 1;
#[cfg(feature = "retail-12-1-0")]
const DEFAULT_ITEM_CREATION_CONTEXT: i32 = 0;

const COMPANION_ID: f64 = 4401.0;
const COMPANION_PDE_ID: f64 = 9101.0;
const COMPANION_TRAIT_TREE_ID: f64 = 9201.0;
const COMPANION_FACTION_ID: f64 = 2507.0;
const COMPANION_DISPLAY_ID: f64 = 111999.0;
const ROLE_NODE_ID: f64 = 9301.0;
const UTILITY_CURIO_NODE_ID: f64 = 9302.0;
const COMBAT_CURIO_NODE_ID: f64 = 9303.0;
const FLAVOR_NODE_ID: f64 = 9304.0;
const FLAVOR_NODE_NAME: &str = "Brann's Favorite";

pub(super) fn register_delves_ui_surface(state: &mut LuaState) -> LuaResult<()> {
    let ns = ensure_namespace(state, "C_DelvesUI")?;
    for (name, func) in METHODS {
        table_set_rust_fn_static(state, ns, name, func)?;
    }
    store_active_tier(state, DEFAULT_ACTIVE_TIER);
    Ok(())
}

const METHODS: [(&str, rilua::vm::closure::RustFn); 30] = [
    ("GetActiveDelveTier", get_active_delve_tier),
    (
        "GetCompanionInfoForActivePlayer",
        get_companion_info_for_active_player,
    ),
    (
        "GetCreatureDisplayInfoForCompanion",
        get_creature_display_info_for_companion,
    ),
    ("GetCurioLink", get_curio_link),
    ("GetCurioNodeForCompanion", get_curio_node_for_companion),
    (
        "GetCurioRarityByTraitCondAccountElementID",
        get_curio_rarity_by_trait_cond_account_element_id,
    ),
    (
        "GetCurrentDelvesSeasonNumber",
        get_current_delves_season_number,
    ),
    (
        "GetDelveEntranceBackgroundWidgetSetID",
        get_delve_entrance_background_widget_set_id,
    ),
    (
        "GetDelveEntranceDescriptionString",
        get_delve_entrance_description_string,
    ),
    (
        "GetDelveEntranceHeaderString",
        get_delve_entrance_header_string,
    ),
    (
        "GetDelveEntranceTitleString",
        get_delve_entrance_title_string,
    ),
    ("GetDelveEntranceMapID", get_delve_entrance_map_id),
    ("GetDelveEntranceTiers", get_delve_entrance_tiers),
    ("GetDelvesMinRequiredLevel", get_delves_min_required_level),
    ("GetFactionForCompanion", get_faction_for_companion),
    ("GetFlavorNodeForCompanion", get_flavor_node_for_companion),
    (
        "GetFlavorNodeNameForCompanion",
        get_flavor_node_name_for_companion,
    ),
    ("GetPlayerCompanionPDEID", get_player_companion_pde_id),
    ("GetRoleNodeForCompanion", get_role_node_for_companion),
    ("GetRoleSubtreeForCompanion", get_role_subtree_for_companion),
    (
        "GetTieredEntranceOptionalAffixTraitTreeID",
        get_tiered_entrance_optional_affix_trait_tree_id,
    ),
    ("GetTieredEntrancePDEID", get_tiered_entrance_pde_id),
    ("GetTraitTreeForCompanion", get_trait_tree_for_companion),
    ("GetUnseenCuriosBySlotType", get_unseen_curios_by_slot_type),
    (
        "GetWorldTierDifficultyForActivePlayer",
        get_world_tier_difficulty_for_active_player,
    ),
    ("HasActiveDelve", has_active_delve),
    ("IsDelveEntranceTierEnabled", is_delve_entrance_tier_enabled),
    (
        "RequestPartyEligibilityForDelveTiers",
        request_party_eligibility_for_delve_tiers,
    ),
    ("SaveSeenCuriosBySlotType", save_seen_curios_by_slot_type),
    ("SelectDelveEntranceTier", select_delve_entrance_tier),
];

fn get_active_delve_tier(state: &mut LuaState) -> LuaResult<u32> {
    let active_tier = load_active_tier(state);
    push_tier_info(state, active_tier);
    Ok(1)
}

fn get_companion_info_for_active_player(state: &mut LuaState) -> LuaResult<u32> {
    state.push(Val::Num(COMPANION_ID));
    Ok(1)
}

fn get_creature_display_info_for_companion(state: &mut LuaState) -> LuaResult<u32> {
    let _companion_id = f64::from_stack(state, 1).ok();
    state.push(Val::Num(COMPANION_DISPLAY_ID));
    Ok(1)
}

fn get_curio_link(state: &mut LuaState) -> LuaResult<u32> {
    let rarity = crate::lua_bridge::stack_val(state, 2);
    if rilua::table_security::is_secret_value(state, rarity) {
        return Err(rilua::runtime_error(
            "C_DelvesUI.GetCurioLink: argument 2 is NeverSecret",
        ));
    }
    // INFERRED strict u32 rarity representation; no native enum membership claim.
    let rarity = match rarity {
        Val::Num(number)
            if number.is_finite()
                && number.fract() == 0.0
                && number >= 0.0
                && number <= u32::MAX as f64 =>
        {
            number as u32
        }
        _ => {
            return Err(rilua::runtime_error(
                "C_DelvesUI.GetCurioLink: rarity must be a finite integral u32 number",
            ));
        }
    };
    // INFERRED public-only identifier permissions inherited from the shared reader.
    let spell_id = crate::c_api::c_spell::read_public_spell_identifier_at(
        state,
        1,
        "C_DelvesUI.GetCurioLink",
    )?;
    let link = {
        let sim = crate::lua_api::methods::borrow_state(state)?;
        spell_id.and_then(|id| sim.curio_links.get(&(id, rarity)).cloned())
    };
    // INFERRED miss error: the cached declaration requires a nonnil link.
    let link = link.ok_or_else(|| {
        rilua::runtime_error("C_DelvesUI.GetCurioLink: no host-declared curio link")
    })?;
    let link = create_string(state, &link);
    state.push(link);
    Ok(1)
}

fn get_curio_node_for_companion(state: &mut LuaState) -> LuaResult<u32> {
    let curio_type = i32::from_stack(state, 1).unwrap_or_default();
    let node_id = match curio_type {
        1 => UTILITY_CURIO_NODE_ID,
        2 => COMBAT_CURIO_NODE_ID,
        _ => UTILITY_CURIO_NODE_ID,
    };
    state.push(Val::Num(node_id));
    Ok(1)
}

fn get_curio_rarity_by_trait_cond_account_element_id(state: &mut LuaState) -> LuaResult<u32> {
    let _id = i32::from_stack(state, 1).unwrap_or_default();
    state.push(Val::Num(1.0));
    Ok(1)
}

fn get_current_delves_season_number(state: &mut LuaState) -> LuaResult<u32> {
    state.push(Val::Num(1.0));
    Ok(1)
}

fn get_delve_entrance_background_widget_set_id(state: &mut LuaState) -> LuaResult<u32> {
    state.push(Val::Num(5501.0));
    Ok(1)
}

fn get_delve_entrance_description_string(state: &mut LuaState) -> LuaResult<u32> {
    let description = create_string(state, DELVE_DESCRIPTION);
    state.push(description);
    Ok(1)
}

fn get_delve_entrance_title_string(state: &mut LuaState) -> LuaResult<u32> {
    get_delve_entrance_header_string(state)
}

fn get_delve_entrance_header_string(state: &mut LuaState) -> LuaResult<u32> {
    let header = create_string(state, DELVE_HEADER);
    state.push(header);
    Ok(1)
}

fn get_delve_entrance_map_id(state: &mut LuaState) -> LuaResult<u32> {
    state.push(Val::Num(DELVE_ENTRANCE_MAP_ID as f64));
    Ok(1)
}

fn get_delve_entrance_tiers(state: &mut LuaState) -> LuaResult<u32> {
    let tiers = create_table(state);
    for (idx, tier) in tier_rows().iter().enumerate() {
        let entry = build_tier_info(state, *tier);
        set_table_array(state, tiers, idx as i64 + 1, entry);
    }
    state.push(tiers);
    Ok(1)
}

fn get_delves_min_required_level(state: &mut LuaState) -> LuaResult<u32> {
    state.push(Val::Num(80.0));
    Ok(1)
}

fn get_faction_for_companion(state: &mut LuaState) -> LuaResult<u32> {
    let _companion_id = f64::from_stack(state, 1).ok();
    state.push(Val::Num(COMPANION_FACTION_ID));
    Ok(1)
}

fn get_flavor_node_for_companion(state: &mut LuaState) -> LuaResult<u32> {
    let _companion_id = f64::from_stack(state, 1).ok();
    state.push(Val::Num(FLAVOR_NODE_ID));
    Ok(1)
}

fn get_flavor_node_name_for_companion(state: &mut LuaState) -> LuaResult<u32> {
    let _companion_id = f64::from_stack(state, 1).ok();
    let name = create_string(state, FLAVOR_NODE_NAME);
    state.push(name);
    Ok(1)
}

fn get_player_companion_pde_id(state: &mut LuaState) -> LuaResult<u32> {
    state.push(Val::Num(COMPANION_PDE_ID));
    Ok(1)
}

fn get_role_node_for_companion(state: &mut LuaState) -> LuaResult<u32> {
    let _companion_id = f64::from_stack(state, 1).ok();
    state.push(Val::Num(ROLE_NODE_ID));
    Ok(1)
}

fn get_role_subtree_for_companion(state: &mut LuaState) -> LuaResult<u32> {
    let role_type = i32::from_stack(state, 1).unwrap_or_default();
    let _companion_id = f64::from_stack(state, 2).ok();
    let subtree_id = match role_type {
        0 => 9401.0,
        1 => 9402.0,
        2 => 9403.0,
        _ => 9401.0,
    };
    state.push(Val::Num(subtree_id));
    Ok(1)
}

fn get_tiered_entrance_optional_affix_trait_tree_id(state: &mut LuaState) -> LuaResult<u32> {
    state.push(Val::Num(77001.0));
    Ok(1)
}

fn get_tiered_entrance_pde_id(state: &mut LuaState) -> LuaResult<u32> {
    let pde_id = crate::lua_api::methods::borrow_state(state)?.tiered_entrance_pde_id;
    state.push(Val::Num(f64::from(pde_id)));
    Ok(1)
}

fn get_trait_tree_for_companion(state: &mut LuaState) -> LuaResult<u32> {
    let _companion_id = f64::from_stack(state, 1).ok();
    state.push(Val::Num(COMPANION_TRAIT_TREE_ID));
    Ok(1)
}

fn get_unseen_curios_by_slot_type(state: &mut LuaState) -> LuaResult<u32> {
    let unseen = create_table(state);
    state.push(unseen);
    Ok(1)
}

fn get_world_tier_difficulty_for_active_player(state: &mut LuaState) -> LuaResult<u32> {
    state.push(Val::Num(WORLD_TIER_NORMAL));
    Ok(1)
}

fn has_active_delve(state: &mut LuaState) -> LuaResult<u32> {
    let active = crate::lua_api::methods::borrow_state(state)?.has_active_delve;
    state.push(Val::Bool(active));
    Ok(1)
}

fn is_delve_entrance_tier_enabled(state: &mut LuaState) -> LuaResult<u32> {
    let tier = i32::from_stack(state, 1).unwrap_or_default();
    if let Some(info) = tier_rows().iter().find(|info| info.tier == tier) {
        state.push(Val::Bool(info.unlocked));
        if info.unlocked {
            state.push(Val::Nil);
        } else {
            let reason = create_string(state, info.locked_reason.unwrap_or(LOCKED_TIER_REASON));
            state.push(reason);
        }
        return Ok(2);
    }
    state.push(Val::Bool(false));
    let reason = create_string(state, UNKNOWN_TIER_REASON);
    state.push(reason);
    Ok(2)
}

fn request_party_eligibility_for_delve_tiers(state: &mut LuaState) -> LuaResult<u32> {
    let map_id =
        rilua::table_security::unwrap_secret(state, crate::lua_bridge::stack_val(state, 1))?;
    // INFERRED i32 storage range; authenticate before validating or mutating.
    let map_id = match map_id {
        Val::Num(number)
            if number.is_finite()
                && number.fract() == 0.0
                && number >= i32::MIN as f64
                && number <= i32::MAX as f64 =>
        {
            number as i32
        }
        _ => {
            return Err(rilua::runtime_error(
                "C_DelvesUI.RequestPartyEligibilityForDelveTiers: mapID must be a finite integral i32 number",
            ));
        }
    };
    crate::lua_api::methods::borrow_state_mut(state)?.last_delve_eligibility_map_id = Some(map_id);
    Ok(0)
}

fn save_seen_curios_by_slot_type(_state: &mut LuaState) -> LuaResult<u32> {
    Ok(0)
}

fn select_delve_entrance_tier(state: &mut LuaState) -> LuaResult<u32> {
    let tier = i32::from_stack(state, 1).unwrap_or(DEFAULT_ACTIVE_TIER);
    if tier_rows().iter().any(|info| info.tier == tier) {
        store_active_tier(state, tier);
    }
    Ok(0)
}

#[cfg(feature = "retail-12-1-0")]
#[derive(Clone, Copy)]
struct TierRewardRow {
    id: i32,
    quantity: i32,
    reward_type: i32,
    context: i32,
}

#[cfg(feature = "retail-12-1-0")]
const TIER_REWARDS: [TierRewardRow; 2] = [
    TierRewardRow {
        id: DELVE_ITEM_REWARD_ID,
        quantity: 1,
        reward_type: TIER_REWARD_ITEM,
        context: DEFAULT_ITEM_CREATION_CONTEXT,
    },
    TierRewardRow {
        id: DELVE_CURRENCY_REWARD_ID,
        quantity: 25,
        reward_type: TIER_REWARD_CURRENCY,
        context: DEFAULT_ITEM_CREATION_CONTEXT,
    },
];

#[derive(Clone, Copy)]
struct TierInfoRow {
    tier: i32,
    unlocked: bool,
    suggested_ilvl: i32,
    modifier_widget_set_id: i32,
    tier_description: &'static str,
    locked_reason: Option<&'static str>,
}

const TIER_ROWS: [TierInfoRow; 5] = [
    TierInfoRow {
        tier: 1,
        unlocked: true,
        suggested_ilvl: 571,
        modifier_widget_set_id: 4401,
        tier_description: "Tier 1",
        locked_reason: None,
    },
    TierInfoRow {
        tier: 2,
        unlocked: true,
        suggested_ilvl: 584,
        modifier_widget_set_id: 4402,
        tier_description: "Tier 2",
        locked_reason: None,
    },
    TierInfoRow {
        tier: 3,
        unlocked: true,
        suggested_ilvl: 597,
        modifier_widget_set_id: 4403,
        tier_description: "Tier 3",
        locked_reason: None,
    },
    TierInfoRow {
        tier: 4,
        unlocked: true,
        suggested_ilvl: 610,
        modifier_widget_set_id: 4404,
        tier_description: "Tier 4",
        locked_reason: None,
    },
    TierInfoRow {
        tier: 5,
        unlocked: false,
        suggested_ilvl: 623,
        modifier_widget_set_id: 4405,
        tier_description: "Tier 5",
        locked_reason: Some(LOCKED_TIER_REASON),
    },
];

fn tier_rows() -> [TierInfoRow; 5] {
    TIER_ROWS
}

fn push_tier_info(state: &mut LuaState, tier: i32) {
    let row = tier_rows()
        .into_iter()
        .find(|info| info.tier == tier)
        .unwrap_or(TierInfoRow {
            tier: DEFAULT_ACTIVE_TIER,
            unlocked: true,
            suggested_ilvl: 610,
            modifier_widget_set_id: 4404,
            tier_description: "Tier 4",
            locked_reason: None,
        });
    let tier_info = build_tier_info(state, row);
    state.push(tier_info);
}

fn build_tier_info(state: &mut LuaState, row: TierInfoRow) -> Val {
    let entry = create_table(state);
    table_set(state, entry, "tier", Val::Num(row.tier as f64));
    table_set(state, entry, "unlocked", Val::Bool(row.unlocked));
    table_set(
        state,
        entry,
        "suggestedILvl",
        Val::Num(row.suggested_ilvl as f64),
    );
    table_set(
        state,
        entry,
        "modifierUIWidgetSetID",
        Val::Num(row.modifier_widget_set_id as f64),
    );
    let tier_description = create_string(state, row.tier_description);
    table_set(state, entry, "tierDescription", tier_description);
    match row.locked_reason {
        Some(reason) => {
            let locked_reason = create_string(state, reason);
            table_set(state, entry, "lockedReason", locked_reason);
        }
        None => table_set(state, entry, "lockedReason", Val::Nil),
    }
    set_patch_12_1_tier_info_fields(state, entry);
    entry
}

#[cfg(feature = "retail-12-1-0")]
fn set_patch_12_1_tier_info_fields(state: &mut LuaState, entry: Val) {
    let rewards = build_tier_rewards(state);
    table_set(state, entry, "rewards", rewards);
    table_set(state, entry, "overrideTooltipSpellID", Val::Nil);
    table_set(state, entry, "isLFG", Val::Bool(false));
}

#[cfg(feature = "retail-12-1-0")]
fn build_tier_rewards(state: &mut LuaState) -> Val {
    let rewards = create_table(state);
    for (index, reward) in TIER_REWARDS.iter().enumerate() {
        let reward = build_tier_reward(state, *reward);
        set_table_array(state, rewards, index as i64 + 1, reward);
    }
    rewards
}

#[cfg(feature = "retail-12-1-0")]
fn build_tier_reward(state: &mut LuaState, reward: TierRewardRow) -> Val {
    let entry = create_table(state);
    table_set(state, entry, "id", Val::Num(f64::from(reward.id)));
    table_set(
        state,
        entry,
        "quantity",
        Val::Num(f64::from(reward.quantity)),
    );
    table_set(
        state,
        entry,
        "rewardType",
        Val::Num(f64::from(reward.reward_type)),
    );
    table_set(state, entry, "context", Val::Num(f64::from(reward.context)));
    entry
}

#[cfg(not(feature = "retail-12-1-0"))]
fn set_patch_12_1_tier_info_fields(_state: &mut LuaState, _entry: Val) {}

fn store_active_tier(state: &mut LuaState, tier: i32) {
    if let Ok(ns) = ensure_namespace(state, "C_DelvesUI") {
        let key = state.gc.intern_string_static(ACTIVE_TIER_FIELD.as_bytes());
        if let Some(table) = state.gc.tables.get_mut(ns) {
            let _ = table.raw_set(Val::Str(key), Val::Num(tier as f64), &state.gc.string_arena);
        }
        state.gc.barrier_back(ns);
    }
}

fn load_active_tier(state: &mut LuaState) -> i32 {
    namespace_field(state, "C_DelvesUI", ACTIVE_TIER_FIELD)
        .and_then(|value| match value {
            Val::Num(n) => Some(n as i32),
            _ => None,
        })
        .unwrap_or(DEFAULT_ACTIVE_TIER)
}

fn namespace_field(
    state: &mut LuaState,
    namespace: &'static str,
    field: &'static str,
) -> Option<Val> {
    let ns = ensure_namespace(state, namespace).ok()?;
    let key = state.gc.intern_string_static(field.as_bytes());
    let table = state.gc.tables.get(ns)?;
    Some(table.get_str(key, &state.gc.string_arena))
}
