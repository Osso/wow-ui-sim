//! Current host-provided item interaction, absent when no interaction is open.
use super::helpers::ensure_namespace;
use crate::lua_api::methods::{borrow_state, create_string, create_table, table_set};
use crate::lua_bridge::table_set_rust_fn_static;
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

#[derive(Clone, Debug, Default)]
pub struct ItemInteractionInfo {
    pub texture_kit: String,
    pub open_sound_kit_id: i32,
    pub close_sound_kit_id: i32,
    pub title_text: String,
    pub tutorial_text: String,
    pub button_text: String,
    pub interaction_type: i32,
    pub flags: i32,
    pub description: Option<String>,
    pub button_tooltip: Option<String>,
    pub confirmation_description: Option<String>,
    pub slot_tooltip: Option<String>,
    pub cost: Option<f64>,
    pub currency_type_id: Option<i32>,
    pub drop_in_slot_sound_kit_id: Option<i32>,
}

pub(super) fn register(state: &mut LuaState) -> LuaResult<()> {
    let ns = ensure_namespace(state, "C_ItemInteraction")?;
    table_set_rust_fn_static(state, ns, "GetItemInteractionInfo", get_info)
}

fn get_info(state: &mut LuaState) -> LuaResult<u32> {
    let Some(info) = borrow_state(state)?.item_interaction.clone() else {
        state.push(Val::Nil);
        return Ok(1);
    };
    let table = create_table(state);
    state.push(table);
    publish_text(state, table, &info);
    publish_numbers(state, table, &info);
    Ok(1)
}

fn publish_text(state: &mut LuaState, table: Val, info: &ItemInteractionInfo) {
    for (key, value) in [
        ("textureKit", Some(&info.texture_kit)),
        ("titleText", Some(&info.title_text)),
        ("tutorialText", Some(&info.tutorial_text)),
        ("buttonText", Some(&info.button_text)),
        ("description", info.description.as_ref()),
        ("buttonTooltip", info.button_tooltip.as_ref()),
        (
            "confirmationDescription",
            info.confirmation_description.as_ref(),
        ),
        ("slotTooltip", info.slot_tooltip.as_ref()),
    ] {
        if let Some(value) = value {
            let value = create_string(state, value);
            table_set(state, table, key, value);
        }
    }
}

fn publish_numbers(state: &mut LuaState, table: Val, info: &ItemInteractionInfo) {
    for (key, value) in [
        ("openSoundKitID", Some(f64::from(info.open_sound_kit_id))),
        ("closeSoundKitID", Some(f64::from(info.close_sound_kit_id))),
        ("interactionType", Some(f64::from(info.interaction_type))),
        ("flags", Some(f64::from(info.flags))),
        ("cost", info.cost),
        ("currencyTypeId", info.currency_type_id.map(f64::from)),
        (
            "dropInSlotSoundKitId",
            info.drop_in_slot_sound_kit_id.map(f64::from),
        ),
    ] {
        if let Some(value) = value {
            table_set(state, table, key, Val::Num(value));
        }
    }
}
