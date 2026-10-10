use crate::c_api::ensure_namespace;
use crate::lua_api::globals::currency_data;
use crate::lua_api::methods::{
    borrow_state, create_string, create_table, create_table_with_capacity, table_set,
    table_set_static, val_to_string,
};
use crate::lua_api::state::CurrencyInfo;
use crate::lua_bridge::{FromStack, stack_val, table_set_rust_fn_static};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

const CURRENCY_INFO_METHODS: &[(&str, rilua::RustFn)] = &[
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
];
const CURRENCY_LIST_INFO_HASH_FIELDS: usize = 9;
const CURRENCY_INFO_HASH_FIELDS: usize = 24;
const CURRENCY_DISPLAY_INFO_HASH_FIELDS: usize = 6;
const GOLD_COIN_ICON_FILE_ID: i32 = 133784;
const SILVER_COIN_ICON_FILE_ID: i32 = 133785;
const COPPER_COIN_ICON_FILE_ID: i32 = 133786;

pub(super) fn register_c_currency_info(state: &mut LuaState) -> LuaResult<()> {
    let table_ref = ensure_namespace(state, "C_CurrencyInfo")?;
    for &(name, func) in CURRENCY_INFO_METHODS {
        table_set_rust_fn_static(state, table_ref, name, func)?;
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

fn c_currency_get_list_size(state: &mut LuaState) -> LuaResult<u32> {
    state.push(Val::Num(currency_data::currency_list_size() as f64));
    Ok(1)
}

fn c_currency_get_list_info(state: &mut LuaState) -> LuaResult<u32> {
    let index = i32::from_stack(state, 1)?;
    let Some(entry) = currency_data::get_currency_list_entry(index) else {
        state.push(Val::Nil);
        return Ok(1);
    };
    let info = create_table_with_capacity(state, CURRENCY_LIST_INFO_HASH_FIELDS);
    let name = create_string(state, entry.name);
    table_set_static(
        state,
        info,
        "currencyTypesID",
        Val::Num(entry.currency_id as f64),
    );
    table_set_static(state, info, "name", name);
    table_set_static(state, info, "quantity", Val::Num(entry.quantity as f64));
    table_set_static(
        state,
        info,
        "iconFileID",
        Val::Num(entry.icon_file_id as f64),
    );
    write_currency_list_status_fields(state, info, entry);
    table_set_static(state, info, "quality", Val::Num(entry.quality as f64));
    state.push(info);
    Ok(1)
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
    table_set_static(
        state,
        info,
        "maxQuantity",
        Val::Num(entry.max_quantity as f64),
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

fn c_currency_get_backpack_currency_info(state: &mut LuaState) -> LuaResult<u32> {
    let index = i32::from_stack(state, 1)?;
    let Some(entry) = currency_data::backpack_currencies().nth(index.saturating_sub(1) as usize)
    else {
        return Ok(0);
    };
    let info = create_table(state);
    let name = create_string(state, entry.name);
    table_set(
        state,
        info,
        "currencyTypesID",
        Val::Num(entry.currency_id as f64),
    );
    table_set(state, info, "name", name);
    table_set(state, info, "quantity", Val::Num(entry.quantity as f64));
    table_set(
        state,
        info,
        "iconFileID",
        Val::Num(entry.icon_file_id as f64),
    );
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
