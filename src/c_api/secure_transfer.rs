//! Retail and Forever secure-transfer API model.
//!
//! Trade and outgoing-mail getters read the simulator's active trade and mail
//! compose state. Their mapping from native secure-transfer state is INFERRED
//! from the selected Blizzard consumer (pending R-request recorded on lt-e53a).
//! INFERRED (R88by): the cleanup restore republishes this model to secureenv.

use super::helpers::{ensure_secure_namespace, global_val, register_namespace_function_with_flags};
use crate::lua_api::methods::{
    borrow_state, borrow_state_mut, call_function_state, create_string, table_set_static,
};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val, runtime_error};

#[derive(Clone, Debug, Default)]
pub struct SecureTransferInputs {
    pub housing_purchase_cost: Option<f64>,
    pub housing_purchase_quantity: Option<f64>,
    pub housing_vc_purchase_product_id: Option<f64>,
}

const PROFILE_ENABLED: bool = matches!(
    crate::client_profile::ACTIVE,
    crate::client_profile::ClientProfile::Retail | crate::client_profile::ClientProfile::WowForever
);

#[derive(Clone, Copy)]
enum RegistrationTarget {
    NamespacePair,
    SecureEnvironment,
}

pub(crate) fn register(state: &mut LuaState) -> LuaResult<()> {
    register_members(state, RegistrationTarget::NamespacePair)
}

pub(crate) fn register_secure_environment(state: &mut LuaState) -> LuaResult<()> {
    register_members(state, RegistrationTarget::SecureEnvironment)
}

fn register_members(state: &mut LuaState, target: RegistrationTarget) -> LuaResult<()> {
    if !PROFILE_ENABLED {
        return Ok(());
    }
    register_shared_members(state, target)?;
    if crate::client_profile::ACTIVE == crate::client_profile::ClientProfile::Retail {
        register_retail_mail_member(state, target)
    } else {
        register_forever_mail_members(state, target)
    }
}

fn register_shared_members(state: &mut LuaState, target: RegistrationTarget) -> LuaResult<()> {
    for (name, function) in [
        ("AcceptTrade", accept_trade as rilua::RustFn),
        ("Cancel", cancel),
        ("CompleteHousingPurchase", complete_housing_purchase),
        ("CompleteHousingVCPurchase", complete_housing_vc_purchase),
        ("GetHousingPurchaseCost", get_housing_purchase_cost),
        ("GetHousingPurchaseQuantity", get_housing_purchase_quantity),
        (
            "GetHousingVCPurchaseProductID",
            get_housing_vc_purchase_product_id,
        ),
        ("GetTradePartner", get_trade_partner),
        ("SendMail", send_mail),
        (
            "ShouldShowTradeOfferWarning",
            should_show_trade_offer_warning,
        ),
    ] {
        register_member(state, target, name, function)?;
    }
    Ok(())
}

fn register_retail_mail_member(state: &mut LuaState, target: RegistrationTarget) -> LuaResult<()> {
    register_member(state, target, "GetMailInfo", get_mail_send_info)
}

fn register_forever_mail_members(
    state: &mut LuaState,
    target: RegistrationTarget,
) -> LuaResult<()> {
    for (name, function) in [
        ("GetMailSendInfo", get_mail_send_info as rilua::RustFn),
        ("GetMailReceiveInfo", get_mail_receive_info),
        ("ShouldShowTradeRMTWarning", should_show_trade_rmt_warning),
        ("TakeMailMoney", take_mail_money),
    ] {
        register_member(state, target, name, function)?;
    }
    Ok(())
}

fn register_member(
    state: &mut LuaState,
    target: RegistrationTarget,
    member: &str,
    function: rilua::RustFn,
) -> LuaResult<()> {
    match target {
        RegistrationTarget::NamespacePair => {
            register_namespace_function_with_flags(state, "C_SecureTransfer", member, function, 3)
        }
        RegistrationTarget::SecureEnvironment => {
            let namespace = ensure_secure_namespace(state, "C_SecureTransfer")?;
            crate::lua_bridge::table_set_rust_fn_unpaired(state, namespace, member, function)
        }
    }
}

fn get_trade_partner(state: &mut LuaState) -> LuaResult<u32> {
    let partner = borrow_state(state)?
        .active_trade
        .as_ref()
        .map(|trade| trade.target.clone());
    push_optional_string(state, partner)
}

fn should_show_trade_offer_warning(state: &mut LuaState) -> LuaResult<u32> {
    push_trade_warning(state, "shouldShowTradeOfferWarning")
}

fn should_show_trade_rmt_warning(state: &mut LuaState) -> LuaResult<u32> {
    push_trade_warning(state, "shouldShowTradeRMTWarning")
}

fn push_trade_warning(state: &mut LuaState, key: &str) -> LuaResult<u32> {
    let trade_info = global_val(state, "C_TradeInfo");
    let host_input = crate::lua_api::methods::table_get(state, trade_info, "_state");
    let value = crate::lua_api::methods::table_get(state, host_input, key);
    state.push(Val::Bool(value == Val::Bool(true)));
    Ok(1)
}

fn get_mail_send_info(state: &mut LuaState) -> LuaResult<u32> {
    let target = edit_box_text(state, "SendMailNameEditBox")?;
    let send_money = borrow_state(state)?.player.send_mail_money;
    push_mail_info(state, "target", target, "sendMoney", send_money)
}

fn get_mail_receive_info(state: &mut LuaState) -> LuaResult<u32> {
    let mail_index = open_mail_index(state)?;
    let message = borrow_state(state)?
        .player
        .inbox
        .get(mail_index)
        .cloned()
        .ok_or_else(|| runtime_error("secure transfer mail selection is unavailable"))?;
    push_mail_info(state, "sender", message.sender, "money", message.money)
}

fn edit_box_text(state: &mut LuaState, name: &str) -> LuaResult<String> {
    let edit_box = global_val(state, name);
    let key = Val::Str(state.gc.intern_string(b"GetText"));
    let method = state.gettable(edit_box, key)?;
    let value = call_function_state(state, method, &[edit_box])?;
    Ok(crate::lua_api::methods::val_to_string(state, value).unwrap_or_default())
}

fn open_mail_index(state: &mut LuaState) -> LuaResult<usize> {
    let inbox = global_val(state, "InboxFrame");
    let key = Val::Str(state.gc.intern_string(b"openMailID"));
    match state.gettable(inbox, key)? {
        Val::Num(index) if index >= 1.0 => Ok(index as usize - 1),
        _ => Err(runtime_error(
            "secure transfer mail selection is unavailable",
        )),
    }
}

fn push_mail_info(
    state: &mut LuaState,
    name_key: &'static str,
    name: String,
    amount_key: &'static str,
    amount: u64,
) -> LuaResult<u32> {
    let info = crate::lua_api::methods::create_table(state);
    let name = create_string(state, &name);
    table_set_static(state, info, name_key, name);
    table_set_static(state, info, amount_key, Val::Num(amount as f64));
    state.push(info);
    Ok(1)
}

fn push_optional_string(state: &mut LuaState, value: Option<String>) -> LuaResult<u32> {
    let value = match value {
        Some(text) => create_string(state, &text),
        None => Val::Nil,
    };
    state.push(value);
    Ok(1)
}

fn get_housing_purchase_cost(state: &mut LuaState) -> LuaResult<u32> {
    push_housing_input(state, "purchase cost", |input| input.housing_purchase_cost)
}

fn get_housing_purchase_quantity(state: &mut LuaState) -> LuaResult<u32> {
    push_housing_input(state, "purchase quantity", |input| {
        input.housing_purchase_quantity
    })
}

fn get_housing_vc_purchase_product_id(state: &mut LuaState) -> LuaResult<u32> {
    push_housing_input(state, "virtual-currency product ID", |input| {
        input.housing_vc_purchase_product_id
    })
}

fn push_housing_input(
    state: &mut LuaState,
    field: &str,
    select: impl FnOnce(&SecureTransferInputs) -> Option<f64>,
) -> LuaResult<u32> {
    let value = borrow_state(state)?
        .secure_transfer
        .as_ref()
        .and_then(select)
        .ok_or_else(|| runtime_error(format!("secure transfer {field} is unavailable")))?;
    state.push(Val::Num(value));
    Ok(1)
}

fn accept_trade(state: &mut LuaState) -> LuaResult<u32> {
    call_host_global(state, "AcceptTrade")
}

fn send_mail(state: &mut LuaState) -> LuaResult<u32> {
    call_host_global(state, "SendMailFrame_SendMail")
}

fn cancel(state: &mut LuaState) -> LuaResult<u32> {
    call_host_global(state, "CancelTrade")
}

fn take_mail_money(state: &mut LuaState) -> LuaResult<u32> {
    let index = open_mail_index(state)?;
    let function = global_val(state, "TakeInboxMoney");
    call_function_state(state, function, &[Val::Num((index + 1) as f64)])?;
    Ok(0)
}

fn complete_housing_purchase(state: &mut LuaState) -> LuaResult<u32> {
    call_catalog_shop(state, "ConfirmHousingPurchase")
}

fn complete_housing_vc_purchase(state: &mut LuaState) -> LuaResult<u32> {
    if let Some(input) = borrow_state_mut(state)?.secure_transfer.as_mut() {
        input.housing_vc_purchase_product_id = None;
    }
    Ok(0)
}

fn call_catalog_shop(state: &mut LuaState, member: &str) -> LuaResult<u32> {
    let catalog_shop = global_val(state, "C_CatalogShop");
    let key = Val::Str(state.gc.intern_string(member.as_bytes()));
    let function = state.gettable(catalog_shop, key)?;
    if !matches!(function, Val::Function(_)) {
        return Err(runtime_error(format!(
            "C_CatalogShop.{member} is unavailable"
        )));
    }
    call_function_state(state, function, &[])?;
    Ok(0)
}

fn call_host_global(state: &mut LuaState, name: &str) -> LuaResult<u32> {
    let function = global_val(state, name);
    if !matches!(function, Val::Function(_)) {
        return Err(runtime_error(format!(
            "secure transfer host action {name} is unavailable"
        )));
    }
    call_function_state(state, function, &[])?;
    Ok(0)
}
