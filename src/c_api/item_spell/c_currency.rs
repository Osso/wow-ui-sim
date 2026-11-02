use crate::c_api::ensure_namespace;
use crate::lua_api::globals::currency_data;
#[cfg(feature = "client-retail")]
use crate::lua_api::methods::borrow_state_mut;
use crate::lua_api::methods::{
    borrow_state, create_string, create_table, create_table_with_capacity, table_set,
    table_set_static, val_to_string,
};
use crate::lua_api::state::CurrencyInfo;
use crate::lua_bridge::{FromStack, stack_val, table_set_rust_fn_static};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

const CURRENCY_INFO_METHODS: &[(&str, rilua::RustFn)] = &[
    #[cfg(feature = "retail-12-0-0")]
    ("IsCurrencyTransferInProgress", query_transfer_activity),
    #[cfg(feature = "client-retail")]
    (
        "DoesCurrentFilterRequireAccountCurrencyData",
        currency_filter_requires_account_data,
    ),
    #[cfg(feature = "client-retail")]
    ("GetCurrencyFilter", get_currency_filter),
    #[cfg(feature = "client-retail")]
    ("SetCurrencyFilter", set_currency_filter),
    #[cfg(feature = "client-retail")]
    (
        "IsAccountCharacterCurrencyDataReady",
        account_currency_data_ready,
    ),
    #[cfg(feature = "client-retail")]
    (
        "RequestCurrencyDataForAccountCharacters",
        request_account_currency_data,
    ),
    #[cfg(feature = "client-retail")]
    (
        "IsAccountTransferableCurrency",
        is_account_transferable_currency,
    ),
    #[cfg(feature = "client-retail")]
    ("CanTransferCurrency", can_transfer_currency),
    #[cfg(feature = "client-retail")]
    ("ExpandCurrencyList", expand_currency_list),
    #[cfg(feature = "client-retail")]
    ("SetCurrencyBackpack", set_currency_backpack),
    #[cfg(feature = "client-retail")]
    ("SetCurrencyBackpackByID", set_currency_backpack_by_id),
    #[cfg(feature = "client-retail")]
    ("SetCurrencyUnused", set_currency_unused),
    #[cfg(feature = "client-retail")]
    ("PickupCurrency", pickup_currency),
    ("GetCurrencyListSize", c_currency_get_list_size),
    ("GetCurrencyListInfo", c_currency_get_list_info),
    (
        "GetBackpackCurrencyInfo",
        c_currency_get_backpack_currency_info,
    ),
    ("GetBasicCurrencyInfo", c_currency_get_basic_currency_info),
    ("GetCoinIcon", c_currency_get_coin_icon),
    ("GetCoinText", c_currency_get_coin_text),
    ("GetCoinTextureString", c_currency_get_coin_texture_string),
    ("GetCurrencyInfo", c_currency_get_currency_info),
    (
        "GetCurrencyInfoFromLink",
        c_currency_get_currency_info_from_link,
    ),
    (
        "GetCurrencyContainerInfo",
        c_currency_get_currency_container_info,
    ),
    ("GetAzeriteCurrencyID", c_currency_get_azerite_currency_id),
    (
        "GetWarResourcesCurrencyID",
        c_currency_get_war_resources_currency_id,
    ),
    #[cfg(feature = "client-retail")]
    (
        "GetDragonIslesSuppliesCurrencyID",
        c_currency_get_dragon_isles_supplies_currency_id,
    ),
];
const CURRENCY_LIST_INFO_HASH_FIELDS: usize = if cfg!(feature = "client-retail") {
    12
} else {
    9
};
const CURRENCY_INFO_HASH_FIELDS: usize = 24;
const CURRENCY_DISPLAY_INFO_HASH_FIELDS: usize = 6;
const GOLD_COIN_ICON_FILE_ID: i32 = 133784;
const SILVER_COIN_ICON_FILE_ID: i32 = 133785;
const COPPER_COIN_ICON_FILE_ID: i32 = 133786;

/// Cursor mode 13 (WM46, WM54). The wrapper's guards are UNKNOWN, so any
/// currency ID is accepted; a missing record projects zero values.
#[cfg(feature = "client-retail")]
fn pickup_currency(state: &mut LuaState) -> LuaResult<u32> {
    let currency_id = i32::from_stack(state, 1)?;
    let cursor = crate::lua_api::state::CursorInfo::Currency { currency_id };
    crate::lua_api::globals::real::cursor_model::place(state, cursor)?;
    Ok(0)
}

pub(super) fn register_c_currency_info(state: &mut LuaState) -> LuaResult<()> {
    #[cfg(feature = "client-wrath")]
    crate::lua_api::globals::real::wrath_currency::register(state)?;
    let table_ref = ensure_namespace(state, "C_CurrencyInfo")?;
    for &(name, func) in CURRENCY_INFO_METHODS {
        table_set_rust_fn_static(state, table_ref, name, func)?;
    }
    if matches!(
        crate::client_profile::ACTIVE,
        crate::client_profile::ClientProfile::Retail | crate::client_profile::ClientProfile::Ptr
    ) {
        // Retail CurrencyInfoDocumentation.lua:442-455.
        table_set_rust_fn_static(
            state,
            table_ref,
            "IsCurrencyContainer",
            is_currency_container,
        )?;
        table_set_rust_fn_static(
            state,
            table_ref,
            "DoesWarModeBonusApply",
            query_currency_war_mode_bonus,
        )?;
        table_set_rust_fn_static(
            state,
            table_ref,
            "GetFactionGrantedByCurrency",
            query_currency_faction_grant,
        )?;
    }
    #[cfg(feature = "client-mists")]
    table_set_rust_fn_static(
        state,
        state.global,
        "GetCurrencyListSize",
        c_currency_get_list_size,
    )?;
    Ok(())
}

fn read_container_integer(state: &LuaState, index: i32) -> LuaResult<i32> {
    let value = stack_val(state, index);
    let number = match value {
        Val::Num(number) => Some(number),
        Val::Str(_) => val_to_string(state, value).and_then(|text| text.trim().parse().ok()),
        _ => None,
    }
    .ok_or_else(|| rilua::runtime_error("IsCurrencyContainer expects two numeric arguments"))?;
    if !number.is_finite() || number < i32::MIN as f64 || number > i32::MAX as f64 {
        return Err(rilua::runtime_error(
            "IsCurrencyContainer argument out of range",
        ));
    }
    Ok(number as i32)
}

fn is_currency_container(state: &mut LuaState) -> LuaResult<u32> {
    let currency = read_container_integer(state, 1)?;
    let quantity = read_container_integer(state, 2)?;
    let rows = sim_data::currency_containers::CURRENCY_CONTAINERS;
    let first = rows.partition_point(|row| row.0 < currency);
    let found = rows[first..]
        .iter()
        .take_while(|row| row.0 == currency)
        .any(|&(_, min, max)| quantity >= min && (max == 0 || quantity <= max));
    state.push(Val::Bool(found));
    Ok(1)
}

#[cfg(feature = "retail-12-0-0")]
fn query_transfer_activity(state: &mut LuaState) -> LuaResult<u32> {
    let active = borrow_state(state)?.currency_transfer_in_progress;
    state.push(Val::Bool(active));
    Ok(1)
}

fn query_currency_war_mode_bonus(state: &mut LuaState) -> LuaResult<u32> {
    let value = stack_val(state, 1);
    // Authenticate before type validation or lookup without declassifying the input.
    #[cfg(feature = "retail-12-1-0")]
    let value = rilua::table_security::unwrap_secret(state, value)?;
    let currency_id = crate::c_api::helpers::validate_supplied_relation_id(
        value,
        "C_CurrencyInfo.DoesWarModeBonusApply: currencyID must be a number",
    )?;
    let (applies, once) = {
        let sim = borrow_state(state)?;
        currency_id
            .and_then(|id| sim.currency_war_mode_bonus.get(&id))
            .copied()
            .unwrap_or((None, None))
    };
    state.push(applies.map_or(Val::Nil, Val::Bool));
    state.push(once.map_or(Val::Nil, Val::Bool));
    Ok(2)
}

fn query_currency_faction_grant(state: &mut LuaState) -> LuaResult<u32> {
    let value = stack_val(state, 1);
    #[cfg(feature = "retail-12-1-0")]
    let value = rilua::table_security::unwrap_secret(state, value)?;
    let currency_id = crate::c_api::helpers::validate_supplied_relation_id(
        value,
        "C_CurrencyInfo.GetFactionGrantedByCurrency: currencyID must be a number",
    )?;
    let faction_id = {
        let sim = borrow_state(state)?;
        currency_id.and_then(|id| sim.currency_faction_grants.get(&id).copied())
    };
    state.push(faction_id.map_or(Val::Nil, |id| Val::Num(f64::from(id))));
    Ok(1)
}

// Retail CurrencyInfoDocumentation.lua:10-24, 463-475.
#[cfg(feature = "client-retail")]
fn is_account_transferable_currency(state: &mut LuaState) -> LuaResult<u32> {
    let id = i32::from_stack(state, 1)?;
    let transferable = borrow_state(state)?
        .currency_info
        .get(&id)
        .and_then(|info| info.transfer_percentage)
        .is_some_and(|percentage| percentage > 0.0);
    state.push(Val::Bool(transferable));
    Ok(1)
}

#[cfg(feature = "client-retail")]
fn can_transfer_currency(state: &mut LuaState) -> LuaResult<u32> {
    let id = i32::from_stack(state, 1)?;
    let reason = {
        let sim = borrow_state(state)?;
        currency_transfer_rejection(&sim, id)?
    };
    state.push(Val::Bool(reason.is_none()));
    state.push(
        reason
            .flatten()
            .map_or(Val::Nil, |value| Val::Num(f64::from(value))),
    );
    Ok(2)
}

#[cfg(feature = "client-retail")]
fn currency_transfer_rejection(
    sim: &crate::lua_api::state::SimState,
    id: i32,
) -> LuaResult<Option<Option<i32>>> {
    let list = &sim.currency_list;
    let enabled = list
        .transfer_server_enabled
        .ok_or_else(|| rilua::runtime_error("currency transfer server control not modeled"))?;
    if !enabled {
        return Ok(Some(Some(10)));
    }
    if sim.currency_transfer_in_progress {
        return Ok(Some(Some(9)));
    }
    if !list.account_context_present {
        return Ok(Some(None));
    }
    let Some(info) = sim.currency_info.get(&id) else {
        return Ok(Some(Some(5)));
    };
    if !info
        .transfer_percentage
        .is_some_and(|percentage| percentage > 0.0)
    {
        return Ok(Some(Some(5)));
    }
    let rules = list
        .transfer_rules
        .get(&id)
        .ok_or_else(|| rilua::runtime_error("currency transfer eligibility not modeled"))?;
    let category = rules
        .category_flags
        .ok_or_else(|| rilua::runtime_error("currency category flags not modeled"))?;
    if category & 12 != 0 {
        return Ok(Some(Some(5)));
    }
    let flags = rules
        .currency_flags
        .ok_or_else(|| rilua::runtime_error("currency flags not modeled"))?;
    if flags & (1 << 29) != 0
        && !rules
            .faction_key_0
            .ok_or_else(|| rilua::runtime_error("currency faction key 0 not modeled"))?
    {
        return Ok(Some(Some(5)));
    }
    if flags & (1 << 28) != 0
        && !rules
            .faction_key_1
            .ok_or_else(|| rilua::runtime_error("currency faction key 1 not modeled"))?
    {
        return Ok(Some(Some(5)));
    }
    if !rules
        .award_condition_met
        .ok_or_else(|| rilua::runtime_error("currency award condition not modeled"))?
    {
        return Ok(Some(Some(8)));
    }
    if rules
        .quantity_cap_reached
        .ok_or_else(|| rilua::runtime_error("currency quantity cap not modeled"))?
        || rules
            .weekly_cap_reached
            .ok_or_else(|| rilua::runtime_error("currency weekly cap not modeled"))?
    {
        return Ok(Some(Some(4)));
    }
    let current = list
        .current_character_guid
        .as_ref()
        .ok_or_else(|| rilua::runtime_error("currency current character GUID not modeled"))?;
    let valid_source = list
        .account_currency_sources
        .get(&id)
        .is_some_and(|sources| {
            sources
                .iter()
                .any(|(guid, quantity)| guid != current && *quantity > 0)
        });
    Ok((!valid_source).then_some(Some(6)))
}

// Retail CurrencyInfoDocumentation.lua:27-33,225-231,555-563.
#[cfg(feature = "client-retail")]
fn currency_filter_requires_account_data(state: &mut LuaState) -> LuaResult<u32> {
    let required = borrow_state(state)?.currency_list.filter() == 2;
    state.push(Val::Bool(required));
    Ok(1)
}

#[cfg(feature = "client-retail")]
fn get_currency_filter(state: &mut LuaState) -> LuaResult<u32> {
    let filter = borrow_state(state)?.currency_list.filter();
    state.push(Val::Num(f64::from(filter)));
    Ok(1)
}

#[cfg(feature = "client-retail")]
fn account_currency_data_ready(state: &mut LuaState) -> LuaResult<u32> {
    let ready = borrow_state(state)?.currency_list.account_data_ready();
    state.push(Val::Bool(ready));
    Ok(1)
}

// Retail CurrencyInfoDocumentation.lua:516-518.
#[cfg(feature = "client-retail")]
fn request_account_currency_data(state: &mut LuaState) -> LuaResult<u32> {
    borrow_state_mut(state)?
        .currency_list
        .request_account_data();
    Ok(0)
}

#[cfg(feature = "client-retail")]
fn set_currency_filter(state: &mut LuaState) -> LuaResult<u32> {
    let value = rilua::table_security::unwrap_secret(state, stack_val(state, 1))?;
    let number = match value {
        Val::Nil => return Err(rilua::runtime_error("currency filter is required")),
        Val::Num(number) => number,
        Val::Str(_) => val_to_string(state, value)
            .and_then(|text| text.trim().parse::<f64>().ok())
            .unwrap_or(0.0),
        _ => 0.0,
    };
    if !number.is_finite() || number < i32::MIN as f64 || number > i32::MAX as f64 {
        return Err(rilua::runtime_error("currency filter is out of range"));
    }
    let filter = number.trunc() as i32;
    if !(0..=2).contains(&filter) {
        return Err(rilua::runtime_error("currency filter is out of range"));
    }
    borrow_state_mut(state)?.currency_list.set_filter(filter);
    crate::lua_api::globals::state_backed_queries::dispatch_event_now(
        state,
        "CURRENCY_DISPLAY_UPDATE",
        &[],
    )?;
    Ok(0)
}

#[cfg(feature = "client-retail")]
fn expand_currency_list(state: &mut LuaState) -> LuaResult<u32> {
    let index = i32::from_stack(state, 1)?;
    let expanded = bool::from_stack(state, 2)?;
    borrow_state_mut(state)?
        .currency_list
        .set_header_expanded(index, expanded);
    Ok(0)
}

fn c_currency_get_list_size(state: &mut LuaState) -> LuaResult<u32> {
    #[cfg(feature = "client-retail")]
    let size = borrow_state(state)?.currency_list.displayed_entries().len() as f64;
    #[cfg(not(feature = "client-retail"))]
    let size = currency_data::currency_list_size() as f64;
    state.push(Val::Num(size));
    Ok(1)
}

fn read_currency_list_entry(
    state: &LuaState,
    index: i32,
) -> LuaResult<Option<currency_data::CurrencyEntry>> {
    #[cfg(feature = "client-retail")]
    {
        Ok(borrow_state(state)?
            .currency_list
            .displayed_entry(index)
            .map(|(_, entry)| entry))
    }
    #[cfg(not(feature = "client-retail"))]
    {
        let _ = state;
        Ok(currency_data::get_currency_list_entry(index).copied())
    }
}

#[cfg(feature = "client-retail")]
fn set_currency_unused(state: &mut LuaState) -> LuaResult<u32> {
    let index = i32::from_stack(state, 1)?;
    let unused = bool::from_stack(state, 2)?;
    let Some(entry) = read_currency_list_entry(state, index)? else {
        return Ok(0);
    };
    if entry.is_header {
        return Ok(0);
    }
    let mut sim = borrow_state_mut(state)?;
    if let Some(info) = sim.currency_info.get_mut(&entry.currency_id) {
        info.is_type_unused = unused;
    }
    Ok(0)
}

fn c_currency_get_list_info(state: &mut LuaState) -> LuaResult<u32> {
    let index = i32::from_stack(state, 1)?;
    let Some(entry) = read_currency_list_entry(state, index)? else {
        state.push(Val::Nil);
        return Ok(1);
    };
    let Some(display) = read_currency_list_display_info(state, &entry)? else {
        state.push(Val::Nil);
        return Ok(1);
    };
    let info = create_table_with_capacity(state, CURRENCY_LIST_INFO_HASH_FIELDS);
    table_set_static(
        state,
        info,
        "currencyTypesID",
        Val::Num(entry.currency_id as f64),
    );
    #[cfg(feature = "client-retail")]
    table_set_static(
        state,
        info,
        "currencyID",
        Val::Num(entry.currency_id as f64),
    );
    #[cfg(feature = "client-retail")]
    table_set_static(
        state,
        info,
        "currencyListDepth",
        Val::Num(entry.depth as f64),
    );
    write_currency_row_display_fields(state, info, &display);
    write_currency_list_status_fields(state, info, &entry);
    #[cfg(feature = "client-retail")]
    {
        let watched = borrow_state(state)?.currency_list.is_watched(&entry);
        table_set_static(state, info, "isShowInBackpack", Val::Bool(watched));
        table_set_static(
            state,
            info,
            "isTypeUnused",
            Val::Bool(display.is_type_unused),
        );
        table_set_static(state, info, "discovered", Val::Bool(display.discovered));
    }
    #[cfg(feature = "client-retail")]
    if entry.is_header {
        let expanded = borrow_state(state)?.currency_list.header_expanded(index);
        table_set_static(state, info, "isHeaderExpanded", Val::Bool(expanded));
    }
    table_set_static(
        state,
        info,
        "maxQuantity",
        Val::Num(display.max_quantity as f64),
    );
    table_set_static(state, info, "quality", Val::Num(display.quality as f64));
    state.push(info);
    Ok(1)
}

fn read_currency_list_display_info(
    state: &LuaState,
    entry: &currency_data::CurrencyEntry,
) -> LuaResult<Option<CurrencyInfo>> {
    if entry.is_header {
        return Ok(Some(CurrencyInfo {
            name: entry.name.to_string(),
            quantity: entry.quantity,
            max_quantity: entry.max_quantity,
            icon_file_id: entry.icon_file_id,
            quality: entry.quality,
            ..CurrencyInfo::default()
        }));
    }
    Ok(borrow_state(state)?
        .currency_info
        .get(&entry.currency_id)
        .cloned())
}

fn write_currency_row_display_fields(state: &mut LuaState, table: Val, display: &CurrencyInfo) {
    let name = create_string(state, &display.name);
    table_set_static(state, table, "name", name);
    table_set_static(state, table, "quantity", Val::Num(display.quantity as f64));
    table_set_static(
        state,
        table,
        "iconFileID",
        Val::Num(display.icon_file_id as f64),
    );
}

fn write_currency_list_status_fields(
    state: &mut LuaState,
    info: Val,
    entry: &currency_data::CurrencyEntry,
) {
    table_set_static(state, info, "isHeader", Val::Bool(entry.is_header));
    table_set_static(
        state,
        info,
        "isHeaderExpanded",
        Val::Bool(entry.is_header_expanded),
    );
    table_set_static(
        state,
        info,
        "isShowInBackpack",
        Val::Bool(entry.is_show_in_backpack),
    );
}

fn c_currency_get_coin_texture_string(state: &mut LuaState) -> LuaResult<u32> {
    let amount = i64::from_stack(state, 1)?;
    let amount = create_string(state, &format!("{amount}"));
    state.push(amount);
    Ok(1)
}

fn c_currency_get_coin_icon(state: &mut LuaState) -> LuaResult<u32> {
    let amount = i64::from_stack(state, 1)?;
    let icon = coin_icon_for_amount(amount);
    state.push(Val::Num(icon as f64));
    Ok(1)
}

fn c_currency_get_coin_text(state: &mut LuaState) -> LuaResult<u32> {
    let amount = i64::from_stack(state, 1)?;
    let separator = Option::<String>::from_stack(state, 2)?.unwrap_or_else(|| ", ".to_string());
    let text = coin_text(amount, &separator);
    let text = create_string(state, &text);
    state.push(text);
    Ok(1)
}

fn coin_icon_for_amount(amount: i64) -> i32 {
    let abs_amount = amount.unsigned_abs();
    if abs_amount >= 10_000 {
        return GOLD_COIN_ICON_FILE_ID;
    }
    if abs_amount >= 100 {
        return SILVER_COIN_ICON_FILE_ID;
    }
    COPPER_COIN_ICON_FILE_ID
}

fn coin_text(amount: i64, separator: &str) -> String {
    let sign = if amount < 0 { "-" } else { "" };
    let abs_amount = amount.unsigned_abs();
    let gold = abs_amount / 10_000;
    let silver = (abs_amount / 100) % 100;
    let copper = abs_amount % 100;

    let mut parts = Vec::new();
    if gold > 0 {
        parts.push(format!("{sign}{gold} Gold"));
    }
    if silver > 0 {
        parts.push(format!("{silver} Silver"));
    }
    if copper > 0 || parts.is_empty() {
        parts.push(format!("{copper} Copper"));
    }
    parts.join(separator)
}

#[cfg(feature = "client-retail")]
fn set_currency_backpack(state: &mut LuaState) -> LuaResult<u32> {
    let index = i32::from_stack(state, 1)?;
    let watched = bool::from_stack(state, 2)?;
    if let Some(entry) = read_currency_list_entry(state, index)? {
        update_currency_watch(state, entry.currency_id, watched)?;
    }
    Ok(0)
}

#[cfg(feature = "client-retail")]
fn set_currency_backpack_by_id(state: &mut LuaState) -> LuaResult<u32> {
    let currency_id = i32::from_stack(state, 1)?;
    let watched = bool::from_stack(state, 2)?;
    update_currency_watch(state, currency_id, watched)?;
    Ok(0)
}

#[cfg(feature = "client-retail")]
fn update_currency_watch(state: &LuaState, currency_id: i32, watched: bool) -> LuaResult<()> {
    if currency_data::get_currency_by_id(currency_id).is_none() {
        return Ok(());
    }
    let mut sim = borrow_state_mut(state)?;
    let Some(info) = sim.currency_info.get_mut(&currency_id) else {
        return Ok(());
    };
    info.is_show_in_backpack = watched;
    sim.currency_list.set_watched(currency_id, watched);
    Ok(())
}

fn read_backpack_currency_entry(
    state: &LuaState,
    index: i32,
) -> LuaResult<Option<&'static currency_data::CurrencyEntry>> {
    #[cfg(feature = "client-retail")]
    {
        let sim = borrow_state(state)?;
        let offset = match usize::try_from(index.saturating_sub(1)) {
            Ok(offset) if index > 0 => offset,
            _ => return Ok(None),
        };
        Ok((1..=currency_data::currency_list_size())
            .filter_map(currency_data::get_currency_list_entry)
            .filter(|entry| !entry.is_header && sim.currency_list.is_watched(entry))
            .nth(offset))
    }
    #[cfg(not(feature = "client-retail"))]
    {
        let _ = state;
        Ok(currency_data::backpack_currencies().nth(index.saturating_sub(1) as usize))
    }
}

fn c_currency_get_backpack_currency_info(state: &mut LuaState) -> LuaResult<u32> {
    let index = i32::from_stack(state, 1)?;
    let Some(entry) = read_backpack_currency_entry(state, index)? else {
        return Ok(0);
    };
    let display = borrow_state(state)?
        .currency_info
        .get(&entry.currency_id)
        .cloned();
    let Some(display) = display else {
        return Ok(0);
    };
    let info = create_table(state);
    table_set(
        state,
        info,
        "currencyTypesID",
        Val::Num(entry.currency_id as f64),
    );
    write_currency_row_display_fields(state, info, &display);
    state.push(info);
    Ok(1)
}

fn push_currency_info_table(state: &mut LuaState, info: &CurrencyInfo) -> Val {
    let t = create_table_with_capacity(state, CURRENCY_INFO_HASH_FIELDS);
    write_currency_identity_fields(state, t, info);
    write_currency_quantity_fields(state, t, info);
    write_currency_flag_fields(state, t, info);
    write_currency_weekly_fields(state, t, info);
    write_currency_transfer_fields(state, t, info);
    t
}

fn write_currency_identity_fields(state: &mut LuaState, t: Val, info: &CurrencyInfo) {
    let name = create_string(state, &info.name);
    let description = create_string(state, &info.description);
    table_set_static(state, t, "currencyID", Val::Num(info.currency_id as f64));
    table_set_static(state, t, "name", name);
    table_set_static(state, t, "description", description);
    table_set_static(state, t, "iconFileID", Val::Num(info.icon_file_id as f64));
    table_set_static(state, t, "quality", Val::Num(info.quality as f64));
    table_set_static(
        state,
        t,
        "currencyListDepth",
        Val::Num(info.currency_list_depth as f64),
    );
}

fn write_currency_quantity_fields(state: &mut LuaState, t: Val, info: &CurrencyInfo) {
    table_set_static(state, t, "quantity", Val::Num(info.quantity as f64));
    table_set_static(state, t, "maxQuantity", Val::Num(info.max_quantity as f64));
    table_set_static(state, t, "totalEarned", Val::Num(info.total_earned as f64));
    table_set_static(
        state,
        t,
        "trackedQuantity",
        Val::Num(info.tracked_quantity as f64),
    );
    table_set_static(
        state,
        t,
        "useTotalEarnedForMaxQty",
        Val::Bool(info.use_total_earned_for_max_qty),
    );
}

fn write_currency_flag_fields(state: &mut LuaState, t: Val, info: &CurrencyInfo) {
    table_set_static(state, t, "isHeader", Val::Bool(info.is_header));
    table_set_static(
        state,
        t,
        "isHeaderExpanded",
        Val::Bool(info.is_header_expanded),
    );
    table_set_static(
        state,
        t,
        "isShowInBackpack",
        Val::Bool(info.is_show_in_backpack),
    );
    table_set_static(state, t, "discovered", Val::Bool(info.discovered));
    table_set_static(state, t, "isTradeable", Val::Bool(info.is_tradeable));
    table_set_static(state, t, "isTypeUnused", Val::Bool(info.is_type_unused));
}

fn write_currency_weekly_fields(state: &mut LuaState, t: Val, info: &CurrencyInfo) {
    table_set_static(
        state,
        t,
        "canEarnPerWeek",
        Val::Bool(info.can_earn_per_week),
    );
    table_set_static(
        state,
        t,
        "maxWeeklyQuantity",
        Val::Num(info.max_weekly_quantity as f64),
    );
    table_set_static(
        state,
        t,
        "quantityEarnedThisWeek",
        Val::Num(info.quantity_earned_this_week as f64),
    );
    table_set_static(
        state,
        t,
        "rechargingAmountPerCycle",
        Val::Num(info.recharging_amount_per_cycle as f64),
    );
    table_set_static(
        state,
        t,
        "rechargingCycleDurationMS",
        Val::Num(info.recharging_cycle_duration_ms as f64),
    );
}

fn write_currency_transfer_fields(state: &mut LuaState, t: Val, info: &CurrencyInfo) {
    table_set_static(
        state,
        t,
        "isAccountTransferable",
        Val::Bool(info.is_account_transferable),
    );
    table_set_static(state, t, "isAccountWide", Val::Bool(info.is_account_wide));
    match info.transfer_percentage {
        Some(pct) => table_set_static(state, t, "transferPercentage", Val::Num(pct)),
        None => table_set_static(state, t, "transferPercentage", Val::Nil),
    }
}

fn push_currency_info_by_id(state: &mut LuaState, currency_id: i32) -> LuaResult<u32> {
    let info = borrow_state(state)?
        .currency_info
        .get(&currency_id)
        .cloned();
    let Some(info) = info else {
        return Ok(0);
    };
    let table = push_currency_info_table(state, &info);
    state.push(table);
    Ok(1)
}

fn c_currency_get_currency_info(state: &mut LuaState) -> LuaResult<u32> {
    let currency_id = i32::from_stack(state, 1)?;
    push_currency_info_by_id(state, currency_id)
}

fn c_currency_get_basic_currency_info(state: &mut LuaState) -> LuaResult<u32> {
    let currency_id = i32::from_stack(state, 1)?;
    let quantity = match stack_val(state, 2) {
        Val::Num(n) => n as i32,
        _ => 0,
    };
    push_currency_display_info_by_id(state, currency_id, quantity)
}

fn c_currency_get_currency_info_from_link(state: &mut LuaState) -> LuaResult<u32> {
    let Some(link) = val_to_string(state, stack_val(state, 1)) else {
        return Ok(0);
    };
    let Some(currency_id) = parse_currency_id_from_link(&link) else {
        return Ok(0);
    };
    push_currency_info_by_id(state, currency_id)
}

fn c_currency_get_azerite_currency_id(state: &mut LuaState) -> LuaResult<u32> {
    state.push(Val::Num(1553.0));
    Ok(1)
}

fn c_currency_get_war_resources_currency_id(state: &mut LuaState) -> LuaResult<u32> {
    state.push(Val::Num(1560.0));
    Ok(1)
}

/// VERIFIED (WM50): native writes this literal constant and publishes one number.
#[cfg(feature = "client-retail")]
fn c_currency_get_dragon_isles_supplies_currency_id(state: &mut LuaState) -> LuaResult<u32> {
    state.push(Val::Num(2003.0));
    Ok(1)
}

fn c_currency_get_currency_container_info(state: &mut LuaState) -> LuaResult<u32> {
    let currency_id = i32::from_stack(state, 1)?;
    let quantity = i32::from_stack(state, 2)?;
    push_currency_display_info_by_id(state, currency_id, quantity)
}

fn push_currency_display_info_by_id(
    state: &mut LuaState,
    currency_id: i32,
    quantity: i32,
) -> LuaResult<u32> {
    let info = borrow_state(state)?
        .currency_info
        .get(&currency_id)
        .cloned();
    let Some(info) = info else {
        return Ok(0);
    };
    let t = create_table_with_capacity(state, CURRENCY_DISPLAY_INFO_HASH_FIELDS);
    let name = create_string(state, &info.name);
    let description = create_string(state, &info.description);
    table_set_static(state, t, "actualAmount", Val::Num(quantity as f64));
    table_set_static(state, t, "displayAmount", Val::Num(quantity as f64));
    table_set_static(state, t, "name", name);
    table_set_static(state, t, "description", description);
    table_set_static(state, t, "icon", Val::Num(info.icon_file_id as f64));
    table_set_static(state, t, "quality", Val::Num(info.quality as f64));
    state.push(t);
    Ok(1)
}

/// Parse the currency id from a `|Hcurrency:<id>:...|h` link.
/// Returns `None` for non-currency links or malformed input.
fn parse_currency_id_from_link(link: &str) -> Option<i32> {
    let after_prefix = link.split("|Hcurrency:").nth(1)?;
    let id_str = after_prefix.split(':').next()?;
    id_str.parse::<i32>().ok()
}

pub(super) fn register_c_bank(state: &mut LuaState) -> LuaResult<()> {
    let table_ref = ensure_namespace(state, "C_Bank")?;
    table_set_rust_fn_static(
        state,
        table_ref,
        "FetchDepositedMoney",
        c_bank_fetch_deposited_money,
    )?;
    table_set_rust_fn_static(
        state,
        table_ref,
        "FetchNumPurchasedBankTabs",
        c_bank_fetch_num_purchased_bank_tabs,
    )?;
    Ok(())
}

fn c_bank_fetch_deposited_money(state: &mut LuaState) -> LuaResult<u32> {
    state.push(Val::Num(0.0));
    Ok(1)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lua_api::WowLuaEnv;

    #[test]
    fn get_currency_info_returns_seeded_full_row() {
        let env = WowLuaEnv::new().expect("env should initialize");
        let (name, field_count): (String, i32) = env
            .eval(
                r#"
                local info = C_CurrencyInfo.GetCurrencyInfo(2245)
                local count = 0
                for _ in pairs(info) do
                    count = count + 1
                end
                return info.name, count
                "#,
            )
            .expect("currency info should evaluate");

        assert_eq!(name, "Valorstones");
        assert_eq!(field_count, CURRENCY_INFO_HASH_FIELDS as i32);
    }

    #[test]
    fn coin_text_formats_gold_silver_and_copper_parts() {
        assert_eq!(coin_text(0, ", "), "0 Copper");
        assert_eq!(coin_text(99, ", "), "99 Copper");
        assert_eq!(coin_text(12_345, " / "), "1 Gold / 23 Silver / 45 Copper");
        assert_eq!(coin_text(-10_000, ", "), "-1 Gold");
    }

    #[test]
    fn coin_icon_uses_highest_nonzero_denomination() {
        assert_eq!(coin_icon_for_amount(99), COPPER_COIN_ICON_FILE_ID);
        assert_eq!(coin_icon_for_amount(100), SILVER_COIN_ICON_FILE_ID);
        assert_eq!(coin_icon_for_amount(10_000), GOLD_COIN_ICON_FILE_ID);
        assert_eq!(coin_icon_for_amount(-10_000), GOLD_COIN_ICON_FILE_ID);
    }

    #[test]
    fn coin_icon_and_text_are_registered_on_c_currency_info() {
        let env = WowLuaEnv::new().expect("env should initialize");
        let (coin_icon, coin_text): (i32, String) = env
            .eval(
                "return C_CurrencyInfo.GetCoinIcon(12345), C_CurrencyInfo.GetCoinText(12345, ' / ')",
            )
            .expect("coin helpers should evaluate");

        assert_eq!(coin_icon, GOLD_COIN_ICON_FILE_ID);
        assert_eq!(coin_text, "1 Gold / 23 Silver / 45 Copper");
    }

    #[test]
    fn coin_helpers_can_be_aliased_by_deprecated_currency_script() {
        let env = WowLuaEnv::new().expect("env should initialize");
        let aliases_match: bool = env
            .eval(
                r#"
                GetCoinIcon = C_CurrencyInfo.GetCoinIcon
                GetCoinText = C_CurrencyInfo.GetCoinText
                return GetCoinIcon == C_CurrencyInfo.GetCoinIcon
                    and GetCoinText == C_CurrencyInfo.GetCoinText
                "#,
            )
            .expect("coin aliases should evaluate");

        assert!(aliases_match);
    }
}

fn c_bank_fetch_num_purchased_bank_tabs(state: &mut LuaState) -> LuaResult<u32> {
    state.push(Val::Num(0.0));
    Ok(1)
}
