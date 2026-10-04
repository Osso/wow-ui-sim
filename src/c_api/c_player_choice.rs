//! `C_PlayerChoice` payloads backed by deterministic local simulator state.
//!
//! The simulator does not model the live PlayerChoice service. Tests and addons
//! can seed `SimState.player_choice`; query methods expose the documented 12.1
//! table shape and mutators record local intent without claiming server timing
//! or validation semantics.

#[cfg(feature = "retail-12-1-0")]
use crate::c_api::helpers::ensure_namespace;
#[cfg(feature = "retail-12-1-0")]
use crate::lua_api::methods::{
    borrow_state, borrow_state_mut, create_string, create_table, table_set, table_set_num,
};
#[cfg(feature = "retail-12-1-0")]
use crate::lua_api::state::{
    PlayerChoiceInfo, PlayerChoiceOptionButtonInfo, PlayerChoiceOptionInfo,
    PlayerChoiceOptionRewardInfo, PlayerChoiceRewardCurrencyInfo, PlayerChoiceRewardItemInfo,
    PlayerChoiceRewardReputationInfo,
};
#[cfg(feature = "retail-12-1-0")]
use crate::lua_bridge::{FromStack, table_set_rust_fn_static};
use rilua::LuaResult;
#[cfg(feature = "retail-12-1-0")]
use rilua::Val;
use rilua::vm::state::LuaState;

#[cfg(feature = "retail-12-1-0")]
pub(crate) fn register_c_player_choice_surface(state: &mut LuaState) -> LuaResult<()> {
    let player_choice = ensure_namespace(state, "C_PlayerChoice")?;
    table_set_rust_fn_static(
        state,
        player_choice,
        "GetCurrentPlayerChoiceInfo",
        get_current_player_choice_info,
    )?;
    table_set_rust_fn_static(state, player_choice, "GetNumRerolls", get_num_rerolls)?;
    table_set_rust_fn_static(state, player_choice, "GetRemainingTime", get_remaining_time)?;
    table_set_rust_fn_static(
        state,
        player_choice,
        "IsWaitingForPlayerChoiceResponse",
        is_waiting_for_player_choice_response,
    )?;
    table_set_rust_fn_static(state, player_choice, "OnUIClosed", on_ui_closed)?;
    table_set_rust_fn_static(
        state,
        player_choice,
        "RequestRerollPlayerChoice",
        request_reroll_player_choice,
    )?;
    table_set_rust_fn_static(
        state,
        player_choice,
        "SendPlayerChoiceResponse",
        send_player_choice_response,
    )
}

#[cfg(not(feature = "retail-12-1-0"))]
pub(crate) fn register_c_player_choice_surface(_state: &mut LuaState) -> LuaResult<()> {
    Ok(())
}

#[cfg(feature = "retail-12-1-0")]
fn get_current_player_choice_info(state: &mut LuaState) -> LuaResult<u32> {
    let current = borrow_state(state)?.player_choice.current.clone();
    let Some(current) = current else {
        return Ok(0);
    };
    let info = player_choice_info_table(state, &current);
    state.push(info);
    Ok(1)
}

#[cfg(feature = "retail-12-1-0")]
fn get_num_rerolls(state: &mut LuaState) -> LuaResult<u32> {
    let num_rerolls = borrow_state(state)?.player_choice.num_rerolls;
    state.push(Val::Num(f64::from(num_rerolls)));
    Ok(1)
}

#[cfg(feature = "retail-12-1-0")]
fn get_remaining_time(state: &mut LuaState) -> LuaResult<u32> {
    let remaining_time = borrow_state(state)?.player_choice.remaining_time;
    state.push(remaining_time.map_or(Val::Nil, Val::Num));
    Ok(1)
}

#[cfg(feature = "retail-12-1-0")]
fn is_waiting_for_player_choice_response(state: &mut LuaState) -> LuaResult<u32> {
    let is_waiting = borrow_state(state)?.player_choice.waiting_for_response;
    state.push(Val::Bool(is_waiting));
    Ok(1)
}

#[cfg(feature = "retail-12-1-0")]
fn on_ui_closed(state: &mut LuaState) -> LuaResult<u32> {
    borrow_state_mut(state)?.player_choice.ui_closed = true;
    Ok(0)
}

#[cfg(feature = "retail-12-1-0")]
fn request_reroll_player_choice(state: &mut LuaState) -> LuaResult<u32> {
    borrow_state_mut(state)?.player_choice.reroll_requested = true;
    Ok(0)
}

#[cfg(feature = "retail-12-1-0")]
fn send_player_choice_response(state: &mut LuaState) -> LuaResult<u32> {
    let response_id = i32::from_stack(state, 1)?;
    borrow_state_mut(state)?.player_choice.last_response_id = Some(response_id);
    Ok(0)
}

#[cfg(feature = "retail-12-1-0")]
fn player_choice_info_table(state: &mut LuaState, info: &PlayerChoiceInfo) -> Val {
    let table = create_table(state);
    write_player_choice_identity_fields(state, table, info);
    write_player_choice_layout_fields(state, table, info);
    let options = array_table(state, &info.options, player_choice_option_table);
    table_set(state, table, "options", options);
    set_optional_i32_field(state, table, "soundKitID", info.sound_kit_id);
    set_optional_i32_field(
        state,
        table,
        "closeUISoundKitID",
        info.close_ui_sound_kit_id,
    );
    table
}

#[cfg(feature = "retail-12-1-0")]
fn write_player_choice_identity_fields(state: &mut LuaState, table: Val, info: &PlayerChoiceInfo) {
    set_string_field(state, table, "objectGUID", &info.object_guid);
    table_set(
        state,
        table,
        "choiceID",
        Val::Num(f64::from(info.choice_id)),
    );
    set_string_field(state, table, "questionText", &info.question_text);
    set_string_field(state, table, "pendingChoiceText", &info.pending_choice_text);
    set_string_field(state, table, "uiTextureKit", &info.ui_texture_kit);
}

#[cfg(feature = "retail-12-1-0")]
fn write_player_choice_layout_fields(state: &mut LuaState, table: Val, info: &PlayerChoiceInfo) {
    table_set(
        state,
        table,
        "hideWarboardHeader",
        Val::Bool(info.hide_warboard_header),
    );
    table_set(
        state,
        table,
        "keepOpenAfterChoice",
        Val::Bool(info.keep_open_after_choice),
    );
    table_set(
        state,
        table,
        "showChoicesAsList",
        Val::Bool(info.show_choices_as_list),
    );
    table_set(
        state,
        table,
        "requiresSelection",
        Val::Bool(info.requires_selection),
    );
    table_set(
        state,
        table,
        "hideAnswerArt",
        Val::Bool(info.hide_answer_art),
    );
    table_set(
        state,
        table,
        "showChoicesAsGrid",
        Val::Bool(info.show_choices_as_grid),
    );
}

#[cfg(feature = "retail-12-1-0")]
fn player_choice_option_table(state: &mut LuaState, option: &PlayerChoiceOptionInfo) -> Val {
    let table = create_table(state);
    write_player_choice_option_identity(state, table, option);
    write_player_choice_option_content(state, table, option);
    write_player_choice_option_optional_fields(state, table, option);
    table_set(
        state,
        table,
        "consolidateWidgets",
        Val::Bool(option.consolidate_widgets),
    );
    table
}

#[cfg(feature = "retail-12-1-0")]
fn write_player_choice_option_identity(
    state: &mut LuaState,
    table: Val,
    option: &PlayerChoiceOptionInfo,
) {
    table_set(state, table, "id", Val::Num(f64::from(option.id)));
    set_string_field(state, table, "description", &option.description);
    set_string_field(state, table, "header", &option.header);
    table_set(
        state,
        table,
        "choiceArtID",
        Val::Num(f64::from(option.choice_art_id)),
    );
    table_set(
        state,
        table,
        "desaturatedArt",
        Val::Bool(option.desaturated_art),
    );
    table_set(
        state,
        table,
        "disabledOption",
        Val::Bool(option.disabled_option),
    );
}

#[cfg(feature = "retail-12-1-0")]
fn write_player_choice_option_content(
    state: &mut LuaState,
    table: Val,
    option: &PlayerChoiceOptionInfo,
) {
    table_set(state, table, "hasRewards", Val::Bool(option.has_rewards));
    let rewards = player_choice_reward_info_table(state, &option.reward_info);
    table_set(state, table, "rewardInfo", rewards);
    set_string_field(state, table, "uiTextureKit", &option.ui_texture_kit);
    table_set(
        state,
        table,
        "maxStacks",
        Val::Num(f64::from(option.max_stacks)),
    );
    let buttons = array_table(state, &option.buttons, player_choice_button_table);
    table_set(state, table, "buttons", buttons);
}

#[cfg(feature = "retail-12-1-0")]
fn write_player_choice_option_optional_fields(
    state: &mut LuaState,
    table: Val,
    option: &PlayerChoiceOptionInfo,
) {
    set_optional_i32_field(state, table, "widgetSetID", option.widget_set_id);
    set_optional_i32_field(state, table, "spellID", option.spell_id);
    set_optional_i32_field(state, table, "rarity", option.rarity);
    set_optional_i32_field(state, table, "typeArtID", option.type_art_id);
    set_optional_string_field(
        state,
        table,
        "headerIconAtlasElement",
        option.header_icon_atlas_element.as_deref(),
    );
    set_optional_string_field(state, table, "subHeader", option.sub_header.as_deref());
}

#[cfg(feature = "retail-12-1-0")]
fn player_choice_button_table(state: &mut LuaState, button: &PlayerChoiceOptionButtonInfo) -> Val {
    let table = create_table(state);
    table_set(state, table, "id", Val::Num(f64::from(button.id)));
    set_string_field(state, table, "text", &button.text);
    table_set(state, table, "disabled", Val::Bool(button.disabled));
    table_set(
        state,
        table,
        "showCheckmark",
        Val::Bool(button.show_checkmark),
    );
    table_set(
        state,
        table,
        "hideButtonShowText",
        Val::Bool(button.hide_button_show_text),
    );
    table_set(state, table, "selected", Val::Bool(button.selected));
    set_optional_string_field(state, table, "confirmation", button.confirmation.as_deref());
    set_optional_string_field(state, table, "tooltip", button.tooltip.as_deref());
    set_optional_i32_field(state, table, "rewardQuestID", button.reward_quest_id);
    set_optional_i32_field(state, table, "soundKitID", button.sound_kit_id);
    set_optional_string_field(state, table, "listText", button.list_text.as_deref());
    table
}

#[cfg(feature = "retail-12-1-0")]
fn player_choice_reward_info_table(
    state: &mut LuaState,
    rewards: &PlayerChoiceOptionRewardInfo,
) -> Val {
    let table = create_table(state);
    let currency = array_table(state, &rewards.currency_rewards, currency_reward_table);
    let items = array_table(state, &rewards.item_rewards, item_reward_table);
    let reputation = array_table(state, &rewards.reputation_rewards, reputation_reward_table);
    table_set(state, table, "currencyRewards", currency);
    table_set(state, table, "itemRewards", items);
    table_set(state, table, "repRewards", reputation);
    table
}

#[cfg(feature = "retail-12-1-0")]
fn currency_reward_table(state: &mut LuaState, reward: &PlayerChoiceRewardCurrencyInfo) -> Val {
    let table = create_table(state);
    table_set(
        state,
        table,
        "currencyId",
        Val::Num(f64::from(reward.currency_id)),
    );
    set_string_field(state, table, "name", &reward.name);
    table_set(
        state,
        table,
        "currencyTexture",
        Val::Num(f64::from(reward.currency_texture)),
    );
    table_set(
        state,
        table,
        "quantity",
        Val::Num(f64::from(reward.quantity)),
    );
    table_set(
        state,
        table,
        "isCurrencyContainer",
        Val::Bool(reward.is_currency_container),
    );
    table
}

#[cfg(feature = "retail-12-1-0")]
fn item_reward_table(state: &mut LuaState, reward: &PlayerChoiceRewardItemInfo) -> Val {
    let table = create_table(state);
    table_set(state, table, "itemId", Val::Num(f64::from(reward.item_id)));
    set_string_field(state, table, "name", &reward.name);
    table_set(
        state,
        table,
        "quantity",
        Val::Num(f64::from(reward.quantity)),
    );
    table
}

#[cfg(feature = "retail-12-1-0")]
fn reputation_reward_table(state: &mut LuaState, reward: &PlayerChoiceRewardReputationInfo) -> Val {
    let table = create_table(state);
    table_set(
        state,
        table,
        "factionId",
        Val::Num(f64::from(reward.faction_id)),
    );
    table_set(
        state,
        table,
        "quantity",
        Val::Num(f64::from(reward.quantity)),
    );
    table
}

#[cfg(feature = "retail-12-1-0")]
fn array_table<T>(state: &mut LuaState, values: &[T], build: fn(&mut LuaState, &T) -> Val) -> Val {
    let table = create_table(state);
    let Val::Table(table_ref) = table else {
        return Val::Nil;
    };
    for (index, value) in values.iter().enumerate() {
        let value = build(state, value);
        table_set_num(state, table_ref, (index + 1) as f64, value);
    }
    Val::Table(table_ref)
}

#[cfg(feature = "retail-12-1-0")]
fn set_string_field(state: &mut LuaState, table: Val, key: &str, value: &str) {
    let value = create_string(state, value);
    table_set(state, table, key, value);
}

#[cfg(feature = "retail-12-1-0")]
fn set_optional_string_field(state: &mut LuaState, table: Val, key: &str, value: Option<&str>) {
    if let Some(value) = value {
        set_string_field(state, table, key, value);
    }
}

#[cfg(feature = "retail-12-1-0")]
fn set_optional_i32_field(state: &mut LuaState, table: Val, key: &str, value: Option<i32>) {
    if let Some(value) = value {
        table_set(state, table, key, Val::Num(f64::from(value)));
    }
}
