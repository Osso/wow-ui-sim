//! Retail R84 currency tooltip producers and explicit host inputs.

use std::collections::HashMap;

use crate::c_api::tooltip_getter_args::one_based_index;
use crate::lua_api::globals::missing_surface::tooltip_info::{
    append_line, compose_tooltip, global_text, item_quality_color, named_color,
    restore_plain_line_type,
};
use crate::lua_api::methods::{
    borrow_state, call_function_state, create_string, table_get, table_set, val_to_string,
};
use crate::lua_api::state::LootSlot;
use crate::lua_bridge::FromStack;
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val, runtime_error};

const TOOLTIP_TYPE_CURRENCY: f64 = 5.0;
const CURRENCY_HUNDREDTHS_PER_UNIT: i32 = 100;
const CURRENCY_COLOR_MARKUP_PREFIX: &str = "|c";
const MISSING_CURRENCY_COLOR_PREFIX: &str = "|cffff00ff";
const FIELD_6_REPUTATION_LABEL: u8 = 1 << 2;
const LINE_TYPE_PLAIN: f64 = 0.0;
const LINE_TYPE_SPACER: f64 = 1.0;
const LINE_TYPE_NUMERIC: f64 = 14.0;
const FLAGS_0_AMOUNT_IS_HUNDREDTHS: u32 = 1 << 3;
const FLAGS_0_WEEKLY_MAXIMUM_OVERRIDE: u32 = 1 << 2;
const FLAGS_0_TOTAL_MAXIMUM_OVERRIDE: u32 = 1 << 18;
const FLAGS_0_HIDE_LOCAL_SECTION: u32 = 1 << 22;
const FLAGS_0_ACCOUNT_WIDE: u32 = 1 << 24;
const FLAGS_1_ACCOUNT_WIDE: u32 = 1 << 3;
const FLAGS_1_XP_ROW: u32 = 1 << 1;
const FLAGS_1_SEASONAL_ROWS: u32 = 1;
const FLAGS_1_TOTAL_MAXIMUM_OVERRIDE: u32 = (1 << 6) | (1 << 7);
const QUEST_LOG_CURRENCY_USAGE: &str =
    "Usage: local data = C_TooltipInfo.GetQuestLogCurrency(type, currencyIndex [, questID])";

/// Supplied Retail `CurrencyTypes` and local currency inputs used by the
/// currency tooltip producer. Missing values stay unknown; they are not inferred.
#[derive(Debug, Clone, Default)]
pub struct CurrencyTooltipInputs {
    /// Supplied `CurrencyTypes.Flags_0`; absence means no flag data was supplied.
    pub flags_0: Option<u32>,
    /// Supplied `CurrencyTypes.Flags_1`; absence means no flag data was supplied.
    pub flags_1: Option<u32>,
    /// The low byte read from `CurrencyTypes` field 6 for the reputation label.
    pub field_6_low_byte: Option<u8>,
    /// `CurrencyTypes` field 11 (`XpQuestDifficulty`).
    pub field_11_xp_quest_difficulty: Option<i32>,
    /// `CurrencyTypes` field 16 transfer percentage.
    pub field_16_transfer_percentage: Option<f64>,
    /// Whether a local player-currency record was supplied.
    pub local_currency_record: Option<bool>,
    /// Ordered CurrencyContainer rows with host-resolved title and description text.
    pub text_ranges: Vec<CurrencyTooltipTextRange>,
    /// Local currency quantity used by weekly-cap rows.
    pub local_quantity: Option<i32>,
    /// Local weekly maximum override; selected when Flags_0 bit 2 is set.
    pub weekly_maximum_override: Option<i32>,
    /// CurrencyTypes field 7, used when Flags_0 bit 2 is clear.
    pub field_7_weekly_maximum: Option<i32>,
    /// Local total-maximum override, selected by the verified flags.
    pub local_total_maximum_override: Option<i32>,
    /// Resolved value for CurrencyTypes field 13's world-state maximum.
    pub field_13_world_state_maximum: Option<i32>,
    /// Value read from CurrencyTypes field 6 by the total-maximum fallback.
    pub field_6_progression_maximum: Option<i32>,
    /// Local seasonal quantity.
    pub seasonal_quantity: Option<i32>,
    /// Opaque native balance-helper result; its formula is not modeled.
    pub balance_helper_result: Option<i32>,
    /// `|c` prefix supplied for total and seasonal rows; its native selector is unknown.
    pub total_row_color_prefix: Option<String>,
    /// Host-formatted integer strings keyed by the exact value passed to the formatter.
    /// The native runtime formatter mode is UNKNOWN, so no locale mode is inferred here.
    pub formatted_numbers: HashMap<i32, String>,
}

/// One CurrencyContainer row selected by signed quantity.
#[derive(Debug, Clone)]
pub struct CurrencyTooltipTextRange {
    pub min_quantity: i32,
    /// Zero means that the range has no upper bound.
    pub max_quantity: i32,
    pub name: String,
    pub description: String,
}

/// Build the populated Retail currency payload. `amount` is already normalized
/// by its getter; this shared producer never divides it.
pub(crate) fn build_currency_tooltip(
    state: &mut LuaState,
    currency_id: i32,
    amount: Option<i32>,
) -> LuaResult<Option<Val>> {
    let (currency, inputs) = {
        let model = borrow_state(state)?;
        let Some(currency) = model.currency_info.get(&currency_id).cloned() else {
            return Ok(None);
        };
        let inputs = model
            .currency_tooltip_inputs
            .get(&currency_id)
            .cloned()
            .unwrap_or_default();
        (currency, inputs)
    };
    let (title, description) = select_currency_text(&currency, &inputs, amount);
    let tooltip_id = (currency_id != 0).then_some(f64::from(currency_id));
    let saved_top = state.top;
    let result = (|| {
        let tooltip = compose_tooltip(state, TOOLTIP_TYPE_CURRENCY, tooltip_id);
        state.push(tooltip);
        let normal_color =
            append_currency_text_rows(state, tooltip, &currency, &inputs, (&title, &description))?;
        if append_xp_row(state, tooltip, &inputs, amount, normal_color)? {
            return Ok(Some(tooltip));
        }
        append_currency_cap_section(state, tooltip, &inputs, normal_color)?;
        Ok(Some(tooltip))
    })();
    state.top = saved_top;
    result
}

fn select_currency_text<'a>(
    currency: &'a crate::lua_api::state::CurrencyInfo,
    inputs: &'a CurrencyTooltipInputs,
    amount: Option<i32>,
) -> (String, String) {
    let range = amount.and_then(|quantity| {
        inputs.text_ranges.iter().find(|row| {
            quantity >= row.min_quantity && (row.max_quantity == 0 || quantity <= row.max_quantity)
        })
    });
    let title = range
        .map(|row| row.name.as_str())
        .unwrap_or(&currency.name)
        .to_owned();
    let description = range
        .map(|row| row.description.as_str())
        .unwrap_or(&currency.description)
        .to_owned();
    (title, description)
}

fn append_currency_text_rows(
    state: &mut LuaState,
    tooltip: Val,
    currency: &crate::lua_api::state::CurrencyInfo,
    inputs: &CurrencyTooltipInputs,
    text: (&str, &str),
) -> LuaResult<(f64, f64, f64)> {
    let (title, description) = text;
    append_plain_currency_line(
        state,
        tooltip,
        title,
        item_quality_color(currency.quality as u8),
    );
    let normal_color = required_color(state, "NORMAL_FONT_COLOR")?;
    append_account_rows(state, tooltip, inputs)?;
    let description_line = append_plain_currency_line(state, tooltip, description, normal_color);
    table_set(state, description_line, "wrapText", Val::Bool(true));
    Ok(normal_color)
}

fn append_plain_currency_line(
    state: &mut LuaState,
    tooltip: Val,
    text: &str,
    color: (f64, f64, f64),
) -> Val {
    let line = append_line(state, tooltip, LINE_TYPE_PLAIN, text, Some(color));
    restore_plain_line_type(state, line);
    line
}

fn append_account_rows(
    state: &mut LuaState,
    tooltip: Val,
    inputs: &CurrencyTooltipInputs,
) -> LuaResult<()> {
    let level = has_flag(inputs.flags_0, FLAGS_0_ACCOUNT_WIDE)
        || has_flag(inputs.flags_1, FLAGS_1_ACCOUNT_WIDE);
    let transferable = inputs
        .field_16_transfer_percentage
        .is_some_and(|percentage| percentage > 0.0);
    let reputation = inputs
        .field_6_low_byte
        .is_some_and(|field| field & FIELD_6_REPUTATION_LABEL != 0);
    if level {
        append_account_label(state, tooltip, "ACCOUNT_LEVEL_CURRENCY")?;
    }
    if transferable {
        append_account_label(state, tooltip, "ACCOUNT_TRANSFERRABLE_CURRENCY")?;
    }
    if reputation {
        append_account_label(state, tooltip, "REPUTATION_TOOLTIP_ACCOUNT_WIDE_LABEL")?;
    }
    Ok(())
}

fn append_account_label(state: &mut LuaState, tooltip: Val, key: &str) -> LuaResult<()> {
    let text = required_global_text(state, key)?;
    let color = required_color(state, "ACCOUNT_WIDE_FONT_COLOR")?;
    append_plain_currency_line(state, tooltip, &text, color);
    Ok(())
}

fn append_xp_row(
    state: &mut LuaState,
    tooltip: Val,
    inputs: &CurrencyTooltipInputs,
    amount: Option<i32>,
    normal_color: (f64, f64, f64),
) -> LuaResult<bool> {
    let gate = inputs
        .field_11_xp_quest_difficulty
        .is_some_and(|difficulty| difficulty > 0)
        && has_flag(inputs.flags_1, FLAGS_1_XP_ROW);
    let Some(amount) = amount.filter(|_| gate) else {
        return Ok(false);
    };
    let xp = required_global_text(state, "XP")?;
    let amount = formatted_number(inputs, amount, "XP amount")?;
    let text = format!("{xp}: {amount}");
    append_line(state, tooltip, LINE_TYPE_NUMERIC, &text, Some(normal_color));
    Ok(true)
}

/// Retail `GetLootCurrency` reads the selected loot row and normalizes its
/// amount before it enters the shared producer.
pub(crate) fn get_loot_currency(state: &mut LuaState) -> LuaResult<u32> {
    let index = one_based_index(
        state,
        1,
        "Usage: local data = C_TooltipInfo.GetLootCurrency(slot)",
    )?;
    let currency = {
        let model = borrow_state(state)?;
        index.and_then(|index| match model.loot_slots.get(index) {
            Some(LootSlot::Currency {
                currency_id,
                quantity,
            }) => Some((*currency_id, *quantity)),
            Some(LootSlot::Item(_)) | None => None,
        })
    };
    let Some((currency_id, quantity)) = currency else {
        return crate::c_api::tooltip_data::unknown_getter_failure();
    };
    let amount = normalize_amount(state, currency_id, quantity)?;
    let Some(tooltip) = build_currency_tooltip(state, currency_id, amount)? else {
        return crate::c_api::tooltip_data::unknown_getter_failure();
    };
    state.push(tooltip);
    Ok(1)
}

/// Retail `GetQuestLogCurrency` uses the selected quest when its optional ID is
/// absent. The host map supplies a selected source result; it does not infer
/// reward, required-objective, or choice-record data from other quest fields.
pub(crate) fn get_quest_log_currency(state: &mut LuaState) -> LuaResult<u32> {
    let selector =
        String::from_stack(state, 1).map_err(|_| runtime_error(QUEST_LOG_CURRENCY_USAGE))?;
    let source_index = one_based_index(state, 2, QUEST_LOG_CURRENCY_USAGE)?;
    let requested_quest =
        Option::<i32>::from_stack(state, 3).map_err(|_| runtime_error(QUEST_LOG_CURRENCY_USAGE))?;
    let selector = selector.to_ascii_lowercase();
    if !matches!(selector.as_str(), "reward" | "required" | "choice") {
        return crate::c_api::tooltip_data::unknown_getter_failure();
    }
    let Some(source_index) = source_index else {
        return crate::c_api::tooltip_data::unknown_getter_failure();
    };
    let Some(quest_id) = positive_quest_id(state, requested_quest)? else {
        return crate::c_api::tooltip_data::unknown_getter_failure();
    };
    let Some((currency_id, raw_quantity)) =
        find_quest_currency_source(state, quest_id, selector, source_index)?
    else {
        return crate::c_api::tooltip_data::unknown_getter_failure();
    };
    let amount = normalize_amount(state, currency_id, raw_quantity)?;
    let Some(tooltip) = build_currency_tooltip(state, currency_id, amount)? else {
        return crate::c_api::tooltip_data::unknown_getter_failure();
    };
    state.push(tooltip);
    Ok(1)
}

fn positive_quest_id(state: &LuaState, requested: Option<i32>) -> LuaResult<Option<i32>> {
    let quest_id = match requested {
        Some(quest_id) => Some(quest_id),
        None => borrow_state(state)?
            .selected_quest_log_id
            .and_then(|quest_id| i32::try_from(quest_id).ok()),
    };
    Ok(quest_id.filter(|quest_id| *quest_id > 0))
}

fn find_quest_currency_source(
    state: &LuaState,
    quest_id: i32,
    selector: String,
    source_index: usize,
) -> LuaResult<Option<(i32, i32)>> {
    let source = borrow_state(state)?
        .quest_currency_tooltip_results
        .get(&(quest_id, selector, source_index))
        .copied();
    Ok(source)
}

/// Retail `GetLootCurrency` and `GetQuestLogCurrency` apply this scaling before
/// the shared producer. Direct IDs and hyperlinks do not use this helper.
fn normalize_amount(state: &LuaState, currency_id: i32, quantity: i32) -> LuaResult<Option<i32>> {
    let flags_0 = borrow_state(state)?
        .currency_tooltip_inputs
        .get(&currency_id)
        .and_then(|inputs| inputs.flags_0);
    Ok(flags_0.map(|flags| scale_quantity(quantity, flags)))
}

fn append_currency_cap_section(
    state: &mut LuaState,
    tooltip: Val,
    inputs: &CurrencyTooltipInputs,
    normal_color: (f64, f64, f64),
) -> LuaResult<()> {
    if inputs.local_currency_record != Some(true) {
        return Ok(());
    }
    let Some(flags_0) = inputs.flags_0 else {
        return Ok(());
    };
    if flags_0 & FLAGS_0_HIDE_LOCAL_SECTION != 0 {
        return Ok(());
    }
    append_line(state, tooltip, LINE_TYPE_SPACER, " ", Some(normal_color));
    let weekly_maximum = append_weekly_cap_row(state, tooltip, inputs, normal_color)?;
    append_total_rows(state, tooltip, inputs, weekly_maximum, normal_color)
}

fn append_weekly_cap_row(
    state: &mut LuaState,
    tooltip: Val,
    inputs: &CurrencyTooltipInputs,
    normal_color: (f64, f64, f64),
) -> LuaResult<Option<i32>> {
    let Some(flags_0) = inputs.flags_0 else {
        return Ok(None);
    };
    let weekly_maximum = resolve_weekly_maximum(inputs, flags_0);
    let Some(maximum) = weekly_maximum.filter(|maximum| *maximum > 0) else {
        return Ok(weekly_maximum);
    };
    let current = inputs
        .local_quantity
        .ok_or_else(|| runtime_error("currency weekly cap requires local quantity input"))?;
    let current = scale_quantity(current, flags_0);
    let color_name = if current >= maximum {
        "ERROR_COLOR"
    } else {
        "HIGHLIGHT_FONT_COLOR"
    };
    let prefix = currency_color_prefix(state, color_name)?;
    let current_text = formatted_number(inputs, current, "weekly current quantity")?;
    let maximum_text = formatted_number(inputs, maximum, "weekly maximum")?;
    append_formatted_currency_row(
        state,
        tooltip,
        "CURRENCY_WEEKLY_CAP",
        &[prefix, current_text, maximum_text],
        normal_color,
    )?;
    Ok(weekly_maximum)
}

fn append_total_rows(
    state: &mut LuaState,
    tooltip: Val,
    inputs: &CurrencyTooltipInputs,
    weekly_maximum: Option<i32>,
    normal_color: (f64, f64, f64),
) -> LuaResult<()> {
    let Some(flags_0) = inputs.flags_0 else {
        return Ok(());
    };
    let total_maximum = resolve_total_maximum(inputs, flags_0);
    let seasonal = inputs
        .flags_1
        .map(|flags| flags & FLAGS_1_SEASONAL_ROWS != 0);
    match (total_maximum, seasonal) {
        (Some(maximum), Some(true)) if maximum > 0 => {
            append_total_and_season_maximum(state, tooltip, inputs, maximum, normal_color)?;
        }
        (Some(maximum), Some(false)) if maximum > 0 => {
            append_total_cap_row(state, tooltip, inputs, maximum, normal_color)?;
        }
        (Some(maximum), Some(true)) if maximum <= 0 => {
            append_season_total_row(state, tooltip, inputs, flags_0, normal_color)?;
        }
        (Some(0), Some(false)) if weekly_maximum == Some(0) => {
            append_fallback_total_row(state, tooltip, inputs, normal_color)?;
        }
        _ => {}
    }
    Ok(())
}

fn append_total_and_season_maximum(
    state: &mut LuaState,
    tooltip: Val,
    inputs: &CurrencyTooltipInputs,
    maximum: i32,
    normal_color: (f64, f64, f64),
) -> LuaResult<()> {
    let prefix = total_row_prefix(inputs)?;
    let balance = balance_text(inputs)?;
    append_formatted_currency_row(
        state,
        tooltip,
        "CURRENCY_TOTAL",
        &[prefix.clone(), balance],
        normal_color,
    )?;

    let flags_0 = inputs
        .flags_0
        .ok_or_else(|| runtime_error("seasonal maximum requires CurrencyTypes flags"))?;
    let seasonal_amount = scaled_local_quantity(inputs.seasonal_quantity, flags_0)?;
    let seasonal_text = formatted_number(inputs, seasonal_amount, "seasonal quantity")?;
    let maximum_text = formatted_number(inputs, maximum, "total maximum")?;
    append_formatted_currency_row(
        state,
        tooltip,
        "CURRENCY_SEASON_TOTAL_MAXIMUM",
        &[prefix, seasonal_text, maximum_text],
        normal_color,
    )?;
    Ok(())
}

fn append_total_cap_row(
    state: &mut LuaState,
    tooltip: Val,
    inputs: &CurrencyTooltipInputs,
    maximum: i32,
    normal_color: (f64, f64, f64),
) -> LuaResult<()> {
    let prefix = total_row_prefix(inputs)?;
    let balance = balance_text(inputs)?;
    let maximum_text = formatted_number(inputs, maximum, "total maximum")?;
    append_formatted_currency_row(
        state,
        tooltip,
        "CURRENCY_TOTAL_CAP",
        &[prefix, balance, maximum_text],
        normal_color,
    )?;
    Ok(())
}

fn append_season_total_row(
    state: &mut LuaState,
    tooltip: Val,
    inputs: &CurrencyTooltipInputs,
    flags_0: u32,
    normal_color: (f64, f64, f64),
) -> LuaResult<()> {
    let prefix = total_row_prefix(inputs)?;
    let seasonal_amount = scaled_local_quantity(inputs.seasonal_quantity, flags_0)?;
    let seasonal_text = formatted_number(inputs, seasonal_amount, "seasonal quantity")?;
    append_formatted_currency_row(
        state,
        tooltip,
        "CURRENCY_SEASON_TOTAL",
        &[prefix, seasonal_text],
        normal_color,
    )?;
    Ok(())
}

fn append_fallback_total_row(
    state: &mut LuaState,
    tooltip: Val,
    inputs: &CurrencyTooltipInputs,
    normal_color: (f64, f64, f64),
) -> LuaResult<()> {
    let prefix = total_row_prefix(inputs)?;
    let balance = balance_text(inputs)?;
    append_formatted_currency_row(
        state,
        tooltip,
        "CURRENCY_TOTAL",
        &[prefix, balance],
        normal_color,
    )?;
    Ok(())
}

fn resolve_weekly_maximum(inputs: &CurrencyTooltipInputs, flags_0: u32) -> Option<i32> {
    let raw = if flags_0 & FLAGS_0_WEEKLY_MAXIMUM_OVERRIDE != 0 {
        inputs.weekly_maximum_override
    } else {
        inputs.field_7_weekly_maximum
    }?;
    Some(scale_quantity(raw, flags_0))
}

fn resolve_total_maximum(inputs: &CurrencyTooltipInputs, flags_0: u32) -> Option<i32> {
    let flags_1 = inputs.flags_1;
    let local_override = flags_0 & FLAGS_0_TOTAL_MAXIMUM_OVERRIDE != 0
        || flags_1.is_some_and(|flags| flags & FLAGS_1_TOTAL_MAXIMUM_OVERRIDE != 0);
    if local_override {
        return inputs.local_total_maximum_override;
    }
    flags_1?;
    let field_13 = inputs.field_13_world_state_maximum?;
    let maximum = if field_13 != 0 {
        field_13
    } else {
        inputs.field_6_progression_maximum?
    };
    Some(scale_quantity(maximum, flags_0))
}

fn scaled_local_quantity(quantity: Option<i32>, flags_0: u32) -> LuaResult<i32> {
    quantity
        .map(|quantity| scale_quantity(quantity, flags_0))
        .ok_or_else(|| runtime_error("currency cap row requires local quantity input"))
}

fn scale_quantity(quantity: i32, flags_0: u32) -> i32 {
    if flags_0 & FLAGS_0_AMOUNT_IS_HUNDREDTHS != 0 {
        quantity / CURRENCY_HUNDREDTHS_PER_UNIT
    } else {
        quantity
    }
}

fn formatted_number(inputs: &CurrencyTooltipInputs, value: i32, source: &str) -> LuaResult<String> {
    inputs
        .formatted_numbers
        .get(&value)
        .cloned()
        .ok_or_else(|| {
            runtime_error(format!(
                "currency tooltip missing host-formatted {source} for {value}"
            ))
        })
}

fn balance_text(inputs: &CurrencyTooltipInputs) -> LuaResult<String> {
    let balance = inputs
        .balance_helper_result
        .ok_or_else(|| runtime_error("currency total row requires balance-helper input"))?;
    formatted_number(inputs, balance, "balance-helper result")
}

fn total_row_prefix(inputs: &CurrencyTooltipInputs) -> LuaResult<String> {
    let prefix = inputs
        .total_row_color_prefix
        .as_deref()
        .ok_or_else(|| runtime_error("currency total row requires color-prefix input"))?;
    if !prefix.starts_with(CURRENCY_COLOR_MARKUP_PREFIX) {
        return Err(runtime_error(
            "currency total-row color prefix must start with |c",
        ));
    }
    Ok(prefix.to_owned())
}

fn append_formatted_currency_row(
    state: &mut LuaState,
    tooltip: Val,
    key: &str,
    values: &[String],
    normal_color: (f64, f64, f64),
) -> LuaResult<()> {
    let text = format_global_strings(state, key, values)?;
    append_line(state, tooltip, LINE_TYPE_NUMERIC, &text, Some(normal_color));
    Ok(())
}

fn format_global_strings(state: &mut LuaState, key: &str, values: &[String]) -> LuaResult<String> {
    let saved_top = state.top;
    let result = (|| {
        let template = crate::c_api::global_val(state, key);
        state.push(template);
        let formatter = crate::c_api::global_val(state, "format");
        state.push(formatter);
        let mut arguments = Vec::with_capacity(values.len() + 1);
        arguments.push(template);
        for value in values {
            let argument = create_string(state, value);
            state.push(argument);
            arguments.push(argument);
        }
        let formatted = call_function_state(state, formatter, &arguments)?;
        val_to_string(state, formatted)
            .ok_or_else(|| runtime_error(format!("localized formatter did not return {key}")))
    })();
    state.top = saved_top;
    result
}

fn currency_color_prefix(state: &mut LuaState, key: &str) -> LuaResult<String> {
    let color = crate::lua_api::global_colors::global_value(state, key);
    let Val::Table(_) = color else {
        return Ok(MISSING_CURRENCY_COLOR_PREFIX.to_owned());
    };
    let formatter = table_get(state, color, "GenerateHexColorMarkup");
    if !matches!(formatter, Val::Function(_)) {
        return Ok(MISSING_CURRENCY_COLOR_PREFIX.to_owned());
    }
    let prefix = call_function_state(state, formatter, &[color])?;
    val_to_string(state, prefix)
        .ok_or_else(|| runtime_error(format!("{key}.GenerateHexColorMarkup did not return text")))
}

fn required_global_text(state: &mut LuaState, key: &str) -> LuaResult<String> {
    global_text(state, key).ok_or_else(|| runtime_error(format!("missing global string {key}")))
}

fn required_color(state: &mut LuaState, key: &str) -> LuaResult<(f64, f64, f64)> {
    named_color(state, key).ok_or_else(|| runtime_error(format!("missing named color {key}")))
}

fn has_flag(flags: Option<u32>, flag: u32) -> bool {
    flags.is_some_and(|flags| flags & flag != 0)
}
