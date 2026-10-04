//! Mail verbs that mutate `SimState.player.inbox` and dispatch
//! `MAIL_INBOX_UPDATE` / `MAIL_SEND_SUCCESS` / `MAIL_CLOSED` events.
//!
//! Migrates 4 entries off `GLOBAL_NIL_STUBS`:
//!
//! - `SendMail(recipient, subject, body)` — delivers outgoing mail to the
//!   recipient's inbox. Clears `send_mail_items` and `send_mail_money` /
//!   `send_mail_cod`. Fires `MAIL_SEND_SUCCESS`. For the simulator,
//!   "delivery" means the mail lands in the player's own inbox so admin
//!   flows can observe it — retail routes by account but the sim has
//!   a single account view.
//! - `DeleteMail(index)` — removes the 1-based inbox entry. Fires
//!   `MAIL_INBOX_UPDATE` when the index was valid.
//! - `ForwardMail(index, recipient, [subject])` — removes the inbox
//!   entry and re-enqueues a copy in the inbox with a new sender
//!   (the forwarder) and the supplied subject. Fires `MAIL_INBOX_UPDATE`.
//! - `CloseInbox()` — fires `MAIL_CLOSED`. No state change; the inbox
//!   list persists.
//!
//! Registered from `register_tail_globals` after `missing_surface` so the
//! Rust impls overwrite any pre-existing stub_nil entries.

use crate::event::Event;
use crate::lua_api::globals::admin::build_mail;
use crate::lua_api::methods::{
    borrow_state, borrow_state_mut, call_function_state, create_string, frame_ref,
};
use crate::lua_api::script_helpers::{get_dispatch_script, get_event_listeners};
use crate::lua_api::state_types::{CursorInfo, MailAttachment, MailMessage};
use crate::lua_bridge::{FromStack, stack_val};
use rilua::vm::state::LuaState;
use rilua::{LuaApiMut, LuaResult, Val};

fn push_event(state: &mut LuaState, name: &str) -> LuaResult<()> {
    borrow_state_mut(state)?.events.push(Event {
        name: name.to_string(),
        args: Vec::new(),
    });
    for widget_id in get_event_listeners(state, name) {
        let Some(handler) = get_dispatch_script(state, widget_id, "OnEvent") else {
            continue;
        };
        let Ok(frame) = frame_ref(state, widget_id) else {
            continue;
        };
        let event_name_val = create_string(state, name);
        let _ = call_function_state(state, handler, &[frame, event_name_val]);
    }
    Ok(())
}

fn stack_i32(state: &mut LuaState, index: i32) -> Option<i32> {
    match stack_val(state, index) {
        Val::Num(n) => Some(n as i32),
        _ => None,
    }
}

fn opt_string(state: &mut LuaState, index: i32) -> Option<String> {
    Option::<String>::from_stack(state, index).ok().flatten()
}

fn required_string(state: &mut LuaState, index: i32) -> Option<String> {
    let value = opt_string(state, index)?;
    (!value.is_empty()).then_some(value)
}

fn inbox_index(index: i32) -> Option<usize> {
    index
        .checked_sub(1)
        .and_then(|value| usize::try_from(value).ok())
}

/// `SendMail(recipient, subject, body)` — delivers a copy to the player's
/// inbox (single-account sim), attaches pending `send_mail_*` items / money,
/// then clears the pending-send slots.
fn send_mail(state: &mut LuaState) -> LuaResult<u32> {
    let (Some(recipient), Some(subject)) = (required_string(state, 1), required_string(state, 2))
    else {
        return Ok(0);
    };
    let body = opt_string(state, 3).unwrap_or_default();

    {
        let mut st = borrow_state_mut(state)?;
        let items = drain_send_mail_items(&mut st.player.send_mail_items);
        let money = std::mem::take(&mut st.player.send_mail_money);
        let cod = std::mem::take(&mut st.player.send_mail_cod);

        let id = st.player.next_mail_id;
        st.player.next_mail_id = st.player.next_mail_id.saturating_add(1);
        let mut mail = build_mail(id, recipient, subject, body, money, items);
        mail.cod_amount = cod;
        st.player.inbox.push(mail);
    }
    push_event(state, "MAIL_SEND_SUCCESS")?;
    push_event(state, "MAIL_INBOX_UPDATE")?;
    Ok(0)
}

fn drain_send_mail_items(slots: &mut [Option<MailAttachment>; 12]) -> Vec<MailAttachment> {
    let mut items = Vec::new();
    for slot in slots.iter_mut() {
        if let Some(item) = slot.take() {
            items.push(item);
        }
    }
    items
}

/// `DeleteMail(index)` — remove the 1-based inbox entry. Silent no-op on
/// out-of-range or non-numeric input.
fn delete_mail(state: &mut LuaState) -> LuaResult<u32> {
    let Some(index) = stack_i32(state, 1) else {
        return Ok(0);
    };
    let removed = {
        let mut st = borrow_state_mut(state)?;
        let Some(zero_based) = index.checked_sub(1).and_then(|n| usize::try_from(n).ok()) else {
            return Ok(0);
        };
        if zero_based >= st.player.inbox.len() {
            return Ok(0);
        }
        st.player.inbox.remove(zero_based);
        true
    };
    if removed {
        push_event(state, "MAIL_INBOX_UPDATE")?;
    }
    Ok(0)
}

/// `ForwardMail(index, recipient, [subject])` — re-enqueue a copy of the
/// mail with a new sender/subject, remove the original. Silent no-op when
/// index or recipient is missing.
fn forward_mail(state: &mut LuaState) -> LuaResult<u32> {
    let (Some(index), Some(recipient)) = (stack_i32(state, 1), required_string(state, 2)) else {
        return Ok(0);
    };
    let new_subject = opt_string(state, 3);

    let changed = {
        let mut st = borrow_state_mut(state)?;
        let Some(zero_based) = index.checked_sub(1).and_then(|n| usize::try_from(n).ok()) else {
            return Ok(0);
        };
        if zero_based >= st.player.inbox.len() {
            return Ok(0);
        }
        let original = st.player.inbox.remove(zero_based);
        let subject = new_subject.unwrap_or_else(|| format!("Fwd: {}", original.subject));
        let id = st.player.next_mail_id;
        st.player.next_mail_id = st.player.next_mail_id.saturating_add(1);
        st.player.inbox.push(build_mail(
            id,
            recipient,
            subject,
            original.body.clone(),
            original.money,
            original.items.clone(),
        ));
        true
    };
    if changed {
        push_event(state, "MAIL_INBOX_UPDATE")?;
    }
    Ok(0)
}

/// `CloseInbox()` — fire `MAIL_CLOSED`. The inbox itself is not cleared.
fn close_inbox(state: &mut LuaState) -> LuaResult<u32> {
    push_event(state, "MAIL_CLOSED")?;
    Ok(0)
}

fn check_inbox(state: &mut LuaState) -> LuaResult<u32> {
    push_event(state, "MAIL_INBOX_UPDATE")?;
    Ok(0)
}

fn take_inbox_item(state: &mut LuaState) -> LuaResult<u32> {
    let (Some(mail_index), Some(attachment_index)) = (stack_i32(state, 1), stack_i32(state, 2))
    else {
        return Ok(0);
    };
    let changed = {
        let mut st = borrow_state_mut(state)?;
        let Some(mail_index) = inbox_index(mail_index) else {
            return Ok(0);
        };
        let Some(attachment_index) = inbox_index(attachment_index) else {
            return Ok(0);
        };
        let Some(mail) = st.player.inbox.get_mut(mail_index) else {
            return Ok(0);
        };
        if attachment_index >= mail.items.len() {
            return Ok(0);
        }
        mail.items.remove(attachment_index);
        true
    };
    if changed {
        push_event(state, "MAIL_INBOX_UPDATE")?;
    }
    Ok(0)
}

fn auto_loot_mail_item(state: &mut LuaState) -> LuaResult<u32> {
    clear_inbox_payload_from_stack(state, true)
}

fn take_inbox_money(state: &mut LuaState) -> LuaResult<u32> {
    clear_inbox_payload_from_stack(state, false)
}

fn clear_inbox_payload_from_stack(state: &mut LuaState, include_items: bool) -> LuaResult<u32> {
    let Some(mail_index) = stack_i32(state, 1) else {
        return Ok(0);
    };
    let changed = clear_inbox_mail_payload(state, mail_index, include_items)?;
    if changed {
        push_event(state, "MAIL_INBOX_UPDATE")?;
    }
    Ok(0)
}

fn clear_inbox_mail_payload(
    state: &mut LuaState,
    mail_index: i32,
    include_items: bool,
) -> LuaResult<bool> {
    let mut st = borrow_state_mut(state)?;
    let Some(mail_index) = inbox_index(mail_index) else {
        return Ok(false);
    };
    let Some(mail) = st.player.inbox.get_mut(mail_index) else {
        return Ok(false);
    };
    Ok(clear_mail_payload(mail, include_items))
}

fn clear_mail_payload(mail: &mut MailMessage, include_items: bool) -> bool {
    let changed = mail.money > 0 || (include_items && !mail.items.is_empty());
    mail.money = 0;
    if include_items {
        mail.items.clear();
    }
    changed
}

fn delete_inbox_item(state: &mut LuaState) -> LuaResult<u32> {
    delete_mail(state)
}

fn return_inbox_item(state: &mut LuaState) -> LuaResult<u32> {
    delete_mail(state)
}

fn close_mail(state: &mut LuaState) -> LuaResult<u32> {
    close_inbox(state)
}

fn set_send_mail_money(state: &mut LuaState) -> LuaResult<u32> {
    let money = stack_i32(state, 1).unwrap_or(0).max(0) as u64;
    borrow_state_mut(state)?.player.send_mail_money = money;
    Ok(0)
}

fn get_send_mail_money(state: &mut LuaState) -> LuaResult<u32> {
    let money = borrow_state(state)?.player.send_mail_money as f64;
    state.push(Val::Num(money));
    Ok(1)
}

fn set_send_mail_cod(state: &mut LuaState) -> LuaResult<u32> {
    let cod = stack_i32(state, 1).unwrap_or(0).max(0) as u64;
    borrow_state_mut(state)?.player.send_mail_cod = cod;
    Ok(0)
}

fn get_send_mail_cod(state: &mut LuaState) -> LuaResult<u32> {
    let cod = borrow_state(state)?.player.send_mail_cod as f64;
    state.push(Val::Num(cod));
    Ok(1)
}

fn get_send_mail_price(state: &mut LuaState) -> LuaResult<u32> {
    state.push(Val::Num(30.0));
    Ok(1)
}

fn clear_send_mail(state: &mut LuaState) -> LuaResult<u32> {
    let mut st = borrow_state_mut(state)?;
    st.player.send_mail_items.fill(None);
    st.player.send_mail_money = 0;
    st.player.send_mail_cod = 0;
    Ok(0)
}

fn set_send_mail_showing(state: &mut LuaState) -> LuaResult<u32> {
    let showing = matches!(stack_val(state, 1), Val::Bool(true));
    borrow_state_mut(state)?.player.send_mail_showing = showing;
    Ok(0)
}

fn can_complain_inbox_item(state: &mut LuaState) -> LuaResult<u32> {
    state.push(Val::Bool(false));
    Ok(1)
}

fn click_send_mail_item_button(state: &mut LuaState) -> LuaResult<u32> {
    let slot = stack_i32(state, 1).and_then(inbox_index);
    let remove = matches!(stack_val(state, 2), Val::Bool(true));

    let changed = if remove {
        remove_send_mail_item(state, slot)
    } else {
        attach_cursor_item_to_send_mail(state, slot)
    };

    if changed {
        push_event(state, "MAIL_SEND_INFO_UPDATE")?;
    }
    Ok(0)
}

fn remove_send_mail_item(state: &mut LuaState, slot: Option<usize>) -> bool {
    let Some(slot) = slot else {
        return false;
    };
    let Ok(mut st) = borrow_state_mut(state) else {
        return false;
    };
    let Some(attachment) = st
        .player
        .send_mail_items
        .get_mut(slot)
        .and_then(Option::take)
    else {
        return false;
    };
    st.cursor_item = Some(CursorInfo::Item {
        item_id: attachment.item_id,
        stack_count: attachment.count,
        origin: crate::lua_api::state_types::CursorItemOrigin::Unknown,
    });
    true
}

fn attach_cursor_item_to_send_mail(state: &mut LuaState, slot: Option<usize>) -> bool {
    let Ok(mut st) = borrow_state_mut(state) else {
        return false;
    };
    let Some(CursorInfo::Item {
        item_id,
        stack_count,
        ..
    }) = st.cursor_item.clone()
    else {
        return false;
    };
    let Some(slot) = slot.or_else(|| first_free_send_mail_slot(&st.player.send_mail_items)) else {
        return false;
    };
    if slot >= st.player.send_mail_items.len() {
        return false;
    }

    let quality = crate::items::get_item(item_id)
        .map(|item| item.quality as i32)
        .unwrap_or(1);
    st.player.send_mail_items[slot] = Some(MailAttachment {
        item_id,
        count: stack_count,
        quality,
    });
    st.cursor_item = None;
    true
}

fn first_free_send_mail_slot(slots: &[Option<MailAttachment>; 12]) -> Option<usize> {
    slots.iter().position(Option::is_none)
}

pub fn register_all(lua: &mut rilua::Lua) -> crate::Result<()> {
    register_inbox_verbs(lua)?;
    register_send_mail_verbs(lua)?;
    Ok(())
}

fn register_inbox_verbs(lua: &mut rilua::Lua) -> crate::Result<()> {
    LuaApiMut::register_function(lua, "SendMail", send_mail)?;
    LuaApiMut::register_function(lua, "DeleteMail", delete_mail)?;
    LuaApiMut::register_function(lua, "ForwardMail", forward_mail)?;
    LuaApiMut::register_function(lua, "CloseInbox", close_inbox)?;
    LuaApiMut::register_function(lua, "CheckInbox", check_inbox)?;
    LuaApiMut::register_function(lua, "TakeInboxItem", take_inbox_item)?;
    LuaApiMut::register_function(lua, "TakeInboxMoney", take_inbox_money)?;
    LuaApiMut::register_function(lua, "AutoLootMailItem", auto_loot_mail_item)?;
    LuaApiMut::register_function(lua, "DeleteInboxItem", delete_inbox_item)?;
    LuaApiMut::register_function(lua, "ReturnInboxItem", return_inbox_item)?;
    LuaApiMut::register_function(lua, "CloseMail", close_mail)?;
    LuaApiMut::register_function(lua, "CanComplainInboxItem", can_complain_inbox_item)?;
    Ok(())
}

fn register_send_mail_verbs(lua: &mut rilua::Lua) -> crate::Result<()> {
    LuaApiMut::register_function(lua, "ClearSendMail", clear_send_mail)?;
    LuaApiMut::register_function(lua, "SetSendMailMoney", set_send_mail_money)?;
    LuaApiMut::register_function(lua, "GetSendMailMoney", get_send_mail_money)?;
    LuaApiMut::register_function(lua, "SetSendMailCOD", set_send_mail_cod)?;
    LuaApiMut::register_function(lua, "GetSendMailCOD", get_send_mail_cod)?;
    LuaApiMut::register_function(lua, "GetSendMailPrice", get_send_mail_price)?;
    LuaApiMut::register_function(lua, "SetSendMailShowing", set_send_mail_showing)?;
    LuaApiMut::register_function(lua, "ClickSendMailItemButton", click_send_mail_item_button)?;
    Ok(())
}
