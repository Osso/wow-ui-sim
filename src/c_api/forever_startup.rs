//! Host-supplied startup inputs. These are simulator fixtures, not native defaults.

use std::collections::{HashMap, HashSet};

use crate::c_api::helpers::{ensure_namespace, set_table_array};
use crate::client_profile::{ACTIVE, ClientProfile};
pub use crate::lua_api::globals::auras::GroupBuffAlert;
use crate::lua_api::methods::{
    borrow_state, create_string, create_table, create_table_with_fields,
};
#[cfg(feature = "client-wowforever")]
use crate::lua_api::methods::{borrow_state_mut, table_get, val_to_string};
#[cfg(feature = "client-wowforever")]
use crate::lua_bridge::TableBuilder;
use crate::lua_bridge::{FromStack, stack_val, table_set_rust_fn_static};
use rilua::vm::state::LuaState;
use rilua::{LuaApiMut, LuaResult, Val};

// BankType values follow pinned ItemConstantsDocumentation.lua:17-27.
const BANK_TYPE_IDS: [i32; 3] = [0, 1, 2];

/// Explicit inputs for 70532 consumers whose native policy is not established.
/// Empty collections mean the host supplied no entries; false flags describe
/// the empty simulator world, not an inferred native account or character.
#[derive(Debug, Default)]
pub struct StartupInputs {
    /// PTR/Forever observation; absence does not imply a native false value.
    pub ammo_required: Option<bool>,
    pub cursor_visible: bool,
    pub ping_enabled: bool,
    pub party_ping_restriction: u8,
    pub purchasable_bank_types: HashSet<i32>,
    pub bank_locked_reasons: HashMap<i32, i32>,
    pub max_bank_tab_types: HashSet<i32>,
    pub next_bank_tabs: HashMap<i32, PurchasableBankTab>,
    pub should_use_player_bags_in_bank: bool,
    pub viewable_bank_types: HashSet<i32>,
    pub purchased_bank_tabs: HashMap<i32, Vec<BankTabData>>,
    pub match_complete: bool,
    pub target_spell_id: Option<i32>,
    pub external_event: Option<ExternalEvent>,
    pub group_buff_alerts: Vec<GroupBuffAlert>,
}

impl StartupInputs {
    /// Best guesses for the stock simulator scenario, not native observations.
    pub(crate) fn stock() -> Self {
        let mut inputs = Self::default();
        if matches!(ACTIVE, ClientProfile::Ptr | ClientProfile::WowForever) {
            // PTR false is a user-decided scenario default, not native evidence.
            inputs.ammo_required = Some(false);
        }
        if ACTIVE == ClientProfile::WowForever {
            // Idle stock scenario: no targeting spell. The pinned declaration
            // (SpellDocumentation.lua:631) returns a non-nilable number; zero is
            // a researched best guess, not aso native evidence.
            inputs.target_spell_id = Some(0);
        }
        inputs
    }
}

pub(crate) fn register(state: &mut LuaState) -> LuaResult<()> {
    if matches!(
        ACTIVE,
        ClientProfile::Retail | ClientProfile::Ptr | ClientProfile::WowForever
    ) {
        let ping = ensure_namespace(state, "C_Ping")?;
        table_set_rust_fn_static(state, ping, "IsPingSystemEnabled", ping_enabled)?;
    }
    if matches!(ACTIVE, ClientProfile::Ptr | ClientProfile::WowForever) {
        // Both pins declare PaperDollInfoDocumentation.lua:11–18 (no-argument bool).
        let paper_doll = ensure_namespace(state, "C_PaperDollInfo")?;
        table_set_rust_fn_static(state, paper_doll, "AmmoNeeded", ammo_needed)?;
        let spell = ensure_namespace(state, "C_Spell")?;
        table_set_rust_fn_static(state, spell, "GetTargetSpellID", target_spell_id)?;
    }
    if ACTIVE == ClientProfile::Ptr {
        // User-decided scenario defaults, not native game-type policy.
        // PTR BattleNetDocumentation.lua:272 and RecentAlliesDocumentation.lua:69.
        let battle_net = ensure_namespace(state, "C_BattleNet")?;
        table_set_rust_fn_static(
            state,
            battle_net,
            "IsFriendTagSupportedForCurrentGameType",
            ptr_social_supported,
        )?;
        let recent_allies = ensure_namespace(state, "C_RecentAllies")?;
        table_set_rust_fn_static(
            state,
            recent_allies,
            "IsInteractionCategoryFilterSupportedForCurrentGameType",
            ptr_social_supported,
        )?;
    }
    if ACTIVE != ClientProfile::WowForever {
        return Ok(());
    }
    register_external_event(state)?;
    register_character_queries(state)?;
    register_services(state)?;
    // The Forever consumer expects parallel arrays; native defaults are unknown.
    let account = ensure_namespace(state, "C_AccountServices")?;
    table_set_rust_fn_static(
        state,
        account,
        "GetPreferredDatacenterLocalities",
        preferred_datacenter_localities,
    )?;
    let cvar = ensure_namespace(state, "C_CVar")?;
    // Forever 70532 CVarDocumentation.lua:171 and :114; both return no values.
    table_set_rust_fn_static(state, cvar, "SetTempCVar", set_temporary_cvar)?;
    table_set_rust_fn_static(state, cvar, "RemoveTempCVar", remove_temporary_cvar)
}

fn set_temporary_cvar(state: &mut LuaState) -> LuaResult<u32> {
    let name = String::from_stack(state, 1)?;
    let value = match crate::lua_bridge::stack_val(state, 2) {
        Val::Nil => {
            return Err(rilua::runtime_error(
                "SetTempCVar: unmodelled nil-value case; native semantics are unknown",
            ));
        }
        // Original Gamepad/Core.lua supplies numbers to the cstring argument.
        Val::Num(number) => number.to_string(),
        _ => String::from_stack(state, 2)?,
    };
    borrow_state(state)?.cvars.set_temporary(&name, &value);
    Ok(0)
}

fn remove_temporary_cvar(state: &mut LuaState) -> LuaResult<u32> {
    let name = String::from_stack(state, 1)?;
    borrow_state(state)?.cvars.remove_temporary(&name);
    Ok(0)
}

/// Lua and Rust current-value readers share the same effective CVar storage.
pub(crate) fn read_current_cvar(state: &LuaState, name: &str) -> LuaResult<Option<String>> {
    Ok(borrow_state(state)?.cvars.get(name))
}

fn register_services(state: &mut LuaState) -> LuaResult<()> {
    let party = ensure_namespace(state, "C_PartyInfo")?;
    table_set_rust_fn_static(state, party, "GetRestrictPings", party_ping_restriction)?;
    let housing = ensure_namespace(state, "C_Housing")?;
    // C_TTSSettings is owned by the persisted c_tts_settings model, which
    // init_lua_state registers after these globals (docs/specs/c-factory-cf2-tts.md).
    table_set_rust_fn_static(state, housing, "IsInsideHouseOrPlot", inside_house_or_plot)
}

fn ping_enabled(state: &mut LuaState) -> LuaResult<u32> {
    let enabled = borrow_state(state)?.forever_startup.ping_enabled;
    state.push(Val::Bool(enabled));
    Ok(1)
}

fn party_ping_restriction(state: &mut LuaState) -> LuaResult<u32> {
    let restriction = borrow_state(state)?.forever_startup.party_ping_restriction;
    state.push(Val::Num(f64::from(restriction)));
    Ok(1)
}

fn inside_house_or_plot(state: &mut LuaState) -> LuaResult<u32> {
    let inside = borrow_state(state)?.housing.current_housing_area.is_some();
    state.push(Val::Bool(inside));
    Ok(1)
}

fn register_character_queries(state: &mut LuaState) -> LuaResult<()> {
    LuaApiMut::register_function(state, "IsCursorVisible", cursor_visible)?;
    let bank = ensure_namespace(state, "C_Bank")?;
    table_set_rust_fn_static(
        state,
        bank,
        "ShouldUsePlayerBagsInBank",
        should_use_player_bags_in_bank,
    )?;
    table_set_rust_fn_static(
        state,
        bank,
        "AreAnyBankTypesViewable",
        are_any_bank_types_viewable,
    )?;
    table_set_rust_fn_static(state, bank, "CanViewBank", can_view_bank)?;
    table_set_rust_fn_static(
        state,
        bank,
        "FetchPurchasedBankTabData",
        fetch_purchased_bank_tab_data,
    )?;
    table_set_rust_fn_static(state, bank, "CanPurchaseBankTab", can_purchase_bank_tab)?;
    table_set_rust_fn_static(state, bank, "FetchBankLockedReason", bank_locked_reason)?;
    table_set_rust_fn_static(state, bank, "HasMaxBankTabs", has_max_bank_tabs)?;
    table_set_rust_fn_static(
        state,
        bank,
        "FetchNextPurchasableBankTabData",
        next_bank_tab,
    )?;
    let pvp = ensure_namespace(state, "C_PvP")?;
    table_set_rust_fn_static(state, pvp, "IsMatchComplete", match_complete)
}

fn preferred_datacenter_localities(state: &mut LuaState) -> LuaResult<u32> {
    let ids = create_table(state);
    let names = create_table(state);
    state.push(ids);
    state.push(names);
    Ok(2)
}

fn cursor_visible(state: &mut LuaState) -> LuaResult<u32> {
    let visible = borrow_state(state)?.forever_startup.cursor_visible;
    state.push(Val::Bool(visible));
    Ok(1)
}

fn can_purchase_bank_tab(state: &mut LuaState) -> LuaResult<u32> {
    let bank_type = i32::from_stack(state, 1)?;
    let allowed = borrow_state(state)?
        .forever_startup
        .purchasable_bank_types
        .contains(&bank_type);
    state.push(Val::Bool(allowed));
    Ok(1)
}

#[cfg(feature = "client-wowforever")]
pub(crate) fn register_admin(builder: TableBuilder) -> LuaResult<TableBuilder> {
    builder
        .set_function(
            "SetShouldUsePlayerBagsInBank",
            set_should_use_player_bags_in_bank,
        )?
        .set_function("SetBankViewableTypes", set_bank_viewable_types)?
        .set_function("SetPurchasedBankTabData", set_purchased_bank_tab_data)
}

fn should_use_player_bags_in_bank(state: &mut LuaState) -> LuaResult<u32> {
    let enabled = borrow_state(state)?
        .forever_startup
        .should_use_player_bags_in_bank;
    state.push(Val::Bool(enabled));
    Ok(1)
}

#[cfg(feature = "client-wowforever")]
fn set_should_use_player_bags_in_bank(state: &mut LuaState) -> LuaResult<u32> {
    let Val::Bool(enabled) = stack_val(state, 1) else {
        return Err(rilua::runtime_error(
            "A_Admin.SetShouldUsePlayerBagsInBank expects a boolean",
        ));
    };
    borrow_state_mut(state)?
        .forever_startup
        .should_use_player_bags_in_bank = enabled;
    Ok(0)
}

fn are_any_bank_types_viewable(state: &mut LuaState) -> LuaResult<u32> {
    let viewable = !borrow_state(state)?
        .forever_startup
        .viewable_bank_types
        .is_empty();
    state.push(Val::Bool(viewable));
    Ok(1)
}

fn can_view_bank(state: &mut LuaState) -> LuaResult<u32> {
    let bank_type = read_bank_type(stack_val(state, 1))?;
    let viewable = borrow_state(state)?
        .forever_startup
        .viewable_bank_types
        .contains(&bank_type);
    state.push(Val::Bool(viewable));
    Ok(1)
}

#[cfg(feature = "client-wowforever")]
fn set_bank_viewable_types(state: &mut LuaState) -> LuaResult<u32> {
    let types = read_bank_type_ids(state, stack_val(state, 1))?;
    borrow_state_mut(state)?.forever_startup.viewable_bank_types = types;
    Ok(0)
}

fn fetch_purchased_bank_tab_data(state: &mut LuaState) -> LuaResult<u32> {
    let bank_type = read_bank_type(stack_val(state, 1))?;
    let tabs = borrow_state(state)?
        .forever_startup
        .purchased_bank_tabs
        .get(&bank_type)
        .cloned()
        .unwrap_or_default();
    let result = create_table(state);
    for (index, tab) in tabs.iter().enumerate() {
        let row = create_bank_tab_data(state, bank_type, tab);
        set_table_array(state, result, (index + 1) as i64, row);
    }
    state.push(result);
    Ok(1)
}

fn create_bank_tab_data(state: &mut LuaState, bank_type: i32, tab: &BankTabData) -> Val {
    let name = create_string(state, &tab.name);
    let cleanup = create_string(state, &tab.tab_cleanup_confirmation);
    let header = create_string(state, &tab.tab_name_edit_box_header);
    create_table_with_fields(
        state,
        &[
            ("ID", Val::Num(f64::from(tab.id))),
            ("bankType", Val::Num(f64::from(bank_type))),
            ("name", name),
            ("icon", Val::Num(f64::from(tab.icon))),
            ("depositFlags", Val::Num(f64::from(tab.deposit_flags))),
            ("tabCleanupConfirmation", cleanup),
            ("tabNameEditBoxHeader", header),
        ],
    )
}

#[cfg(feature = "client-wowforever")]
fn set_purchased_bank_tab_data(state: &mut LuaState) -> LuaResult<u32> {
    let bank_type = read_bank_type(stack_val(state, 1))?;
    let tabs = parse_bank_tab_data(state, stack_val(state, 2))?;
    borrow_state_mut(state)?
        .forever_startup
        .purchased_bank_tabs
        .insert(bank_type, tabs);
    Ok(0)
}

#[cfg(feature = "client-wowforever")]
fn parse_bank_tab_data(state: &mut LuaState, value: Val) -> LuaResult<Vec<BankTabData>> {
    let Val::Table(array) = value else {
        return Err(rilua::runtime_error(
            "A_Admin.SetPurchasedBankTabData expects an array",
        ));
    };
    let length = state
        .gc
        .tables
        .get(array)
        .ok_or_else(|| rilua::runtime_error("invalid bank tab array"))?
        .array_len();
    (1..=length)
        .map(|index| {
            let row = state
                .gc
                .tables
                .get(array)
                .map(|table| table.get_int(index as i64))
                .unwrap_or(Val::Nil);
            parse_bank_tab(state, row, index)
        })
        .collect()
}

#[cfg(feature = "client-wowforever")]
fn parse_bank_tab(state: &mut LuaState, row: Val, index: usize) -> LuaResult<BankTabData> {
    if !matches!(row, Val::Table(_)) {
        return Err(rilua::runtime_error(format!(
            "bank tabs[{index}] must be a record table"
        )));
    }
    Ok(BankTabData {
        id: read_bank_tab_number(state, row, "ID", index)?,
        name: read_bank_tab_text(state, row, "name", index)?,
        icon: read_bank_tab_number(state, row, "icon", index)?,
        deposit_flags: read_bank_tab_number(state, row, "depositFlags", index)?,
        tab_cleanup_confirmation: read_bank_tab_text(state, row, "tabCleanupConfirmation", index)?,
        tab_name_edit_box_header: read_bank_tab_text(state, row, "tabNameEditBoxHeader", index)?,
    })
}

#[cfg(feature = "client-wowforever")]
fn read_bank_tab_number(
    state: &mut LuaState,
    row: Val,
    field: &str,
    index: usize,
) -> LuaResult<u32> {
    match table_get(state, row, field) {
        Val::Num(value)
            if value.is_finite()
                && value.fract() == 0.0
                && (0.0..=f64::from(u32::MAX)).contains(&value) =>
        {
            Ok(value as u32)
        }
        _ => Err(rilua::runtime_error(format!(
            "bank tabs[{index}].{field} must be an unsigned integer"
        ))),
    }
}

#[cfg(feature = "client-wowforever")]
fn read_bank_tab_text(
    state: &mut LuaState,
    row: Val,
    field: &str,
    index: usize,
) -> LuaResult<String> {
    let value = table_get(state, row, field);
    val_to_string(state, value)
        .ok_or_else(|| rilua::runtime_error(format!("bank tabs[{index}].{field} must be a string")))
}

fn read_bank_type(value: Val) -> LuaResult<i32> {
    match value {
        Val::Num(number)
            if number.is_finite()
                && number.fract() == 0.0
                && number >= f64::from(i32::MIN)
                && number <= f64::from(i32::MAX)
                && BANK_TYPE_IDS.contains(&(number as i32)) =>
        {
            Ok(number as i32)
        }
        _ => Err(rilua::runtime_error(
            "bankType must be a documented Enum.BankType value",
        )),
    }
}

#[cfg(feature = "client-wowforever")]
fn read_bank_type_ids(state: &mut LuaState, value: Val) -> LuaResult<HashSet<i32>> {
    let Val::Table(array) = value else {
        return Err(rilua::runtime_error(
            "A_Admin.SetBankViewableTypes expects an array",
        ));
    };
    let length = state
        .gc
        .tables
        .get(array)
        .ok_or_else(|| rilua::runtime_error("invalid bank type array"))?
        .array_len();
    (1..=length)
        .map(|index| {
            let value = state
                .gc
                .tables
                .get(array)
                .map(|table| table.get_int(index as i64))
                .unwrap_or(Val::Nil);
            match value {
                Val::Num(number)
                    if number.is_finite()
                        && number.fract() == 0.0
                        && number >= f64::from(i32::MIN)
                        && number <= f64::from(i32::MAX)
                        && BANK_TYPE_IDS.contains(&(number as i32)) =>
                {
                    Ok(number as i32)
                }
                _ => Err(rilua::runtime_error(format!(
                    "bank types[{index}] must be an integer"
                ))),
            }
        })
        .collect()
}

fn next_bank_tab(state: &mut LuaState) -> LuaResult<u32> {
    let bank_type = i32::from_stack(state, 1)?;
    let offer = borrow_state(state)?
        .forever_startup
        .next_bank_tabs
        .get(&bank_type)
        .cloned();
    let value = offer
        .as_ref()
        .map_or(Val::Nil, |offer| create_bank_offer_table(state, offer));
    state.push(value);
    Ok(1)
}

fn create_bank_offer_table(state: &mut LuaState, offer: &PurchasableBankTab) -> Val {
    let title = create_string(state, &offer.purchase_prompt_title);
    let body = create_string(state, &offer.purchase_prompt_body);
    let confirmation = create_string(state, &offer.purchase_prompt_confirmation);
    create_table_with_fields(
        state,
        &[
            ("tabCost", Val::Num(offer.tab_cost as f64)),
            ("canAfford", Val::Bool(offer.can_afford)),
            ("purchasePromptTitle", title),
            ("purchasePromptBody", body),
            ("purchasePromptConfirmation", confirmation),
        ],
    )
}

fn has_max_bank_tabs(state: &mut LuaState) -> LuaResult<u32> {
    let bank_type = i32::from_stack(state, 1)?;
    let has_max = borrow_state(state)?
        .forever_startup
        .max_bank_tab_types
        .contains(&bank_type);
    state.push(Val::Bool(has_max));
    Ok(1)
}

fn bank_locked_reason(state: &mut LuaState) -> LuaResult<u32> {
    let bank_type = i32::from_stack(state, 1)?;
    let reason = borrow_state(state)?
        .forever_startup
        .bank_locked_reasons
        .get(&bank_type)
        .copied();
    state.push(reason.map_or(Val::Nil, |reason| Val::Num(f64::from(reason))));
    Ok(1)
}

fn match_complete(state: &mut LuaState) -> LuaResult<u32> {
    let complete = borrow_state(state)?.forever_startup.match_complete;
    state.push(Val::Bool(complete));
    Ok(1)
}

fn target_spell_id(state: &mut LuaState) -> LuaResult<u32> {
    let id = borrow_state(state)?.forever_startup.target_spell_id;
    // The no-target native value is unknown; nil is the simulator scenario default.
    state.push(id.map_or(Val::Nil, |id| Val::Num(f64::from(id))));
    Ok(1)
}

fn register_external_event(state: &mut LuaState) -> LuaResult<()> {
    let external = ensure_namespace(state, "C_ExternalEventURL")?;
    table_set_rust_fn_static(state, external, "CanShow", external_can_show)?;
    table_set_rust_fn_static(state, external, "HasURL", external_has_url)?;
    table_set_rust_fn_static(state, external, "IsNew", external_is_new)
}

fn external_can_show(state: &mut LuaState) -> LuaResult<u32> {
    push_external_flag(state, |event| event.can_show)
}

fn external_has_url(state: &mut LuaState) -> LuaResult<u32> {
    push_external_flag(state, |event| !event.url.is_empty())
}

fn external_is_new(state: &mut LuaState) -> LuaResult<u32> {
    push_external_flag(state, |event| event.is_new)
}

fn push_external_flag(state: &mut LuaState, query: fn(&ExternalEvent) -> bool) -> LuaResult<u32> {
    let value = borrow_state(state)?
        .forever_startup
        .external_event
        .as_ref()
        .is_some_and(query);
    state.push(Val::Bool(value));
    Ok(1)
}

/// Pinned Forever API/selected-consumer event names; registration is not event generation.
pub(crate) fn accepts_event(name: &str) -> bool {
    ACTIVE == ClientProfile::WowForever
        && matches!(
            name,
            "SECURE_TRANSFER_CONFIRM_TAKE_MAIL_MONEY"
                | "SHARD_TRANSFER_CANCELLED"
                | "EXTERNAL_EVENT_LAUNCH_URL_RESULT"
                | "EXTERNAL_EVENT_URL_CHANGED"
                | "SOCIAL_UI_SOCIAL_QUEUE_SYSTEM_STATUS_UPDATED"
                | "SOCIAL_UI_RAID_LIST_SYSTEM_STATUS_UPDATED"
                // Selected Mainline glue consumers register these during OnLoad.
                | "CHARACTER_UNDELETE_STARTED"
                | "SUPER_DISTRICT_LIST_UPDATED"
                | "GAME_RULES_CHANGED"
                | "CHAR_SELECT_BACKGROUND_MODEL_READY"
                | "CHAR_CREATE_BACKGROUND_MODEL_READY"
        )
}

fn ptr_social_supported(state: &mut LuaState) -> LuaResult<u32> {
    state.push(Val::Bool(true));
    Ok(1)
}

fn ammo_needed(state: &mut LuaState) -> LuaResult<u32> {
    let required = borrow_state(state)?.forever_startup.ammo_required;
    let required = required.ok_or_else(|| {
        rilua::runtime_error("C_PaperDollInfo.AmmoNeeded: ammo requirement has not been supplied")
    })?;
    state.push(Val::Bool(required));
    Ok(1)
}

/// Supplied 70532 PurchasableBankTabData; no price or affordability policy is inferred.
#[derive(Debug, Clone)]
pub struct PurchasableBankTab {
    pub tab_cost: u64,
    pub can_afford: bool,
    pub purchase_prompt_title: String,
    pub purchase_prompt_body: String,
    pub purchase_prompt_confirmation: String,
}

#[derive(Debug, Clone)]
pub struct BankTabData {
    pub id: u32,
    pub name: String,
    pub icon: u32,
    pub deposit_flags: u32,
    pub tab_cleanup_confirmation: String,
    pub tab_name_edit_box_header: String,
}

#[derive(Debug)]
pub struct ExternalEvent {
    pub url: String,
    pub can_show: bool,
    pub is_new: bool,
}
