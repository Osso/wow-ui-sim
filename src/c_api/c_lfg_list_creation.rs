//! Listing creation text fields (`C_LFGList.ClearCreationTextFields`,
//! `CopyActiveEntryInfoToCreationFields`; Forever 70707
//! `LFGListInfoDocumentation.lua:54-68`).
//!
//! The fields are the edit boxes Blizzard registers with
//! `StoreSecureReference`: `LFGListCreationName`, `LFGListCreationDescription`
//! and `LFGListCreationVoiceChat` (`Blizzard_GroupFinder/Mainline/LFGList.xml:1730,1847`,
//! `LFGList.lua:4814`) and the Vanilla-style `LFGListingComment`
//! (`Blizzard_GroupFinder_VanillaStyle/Mainline/Blizzard_LFGVanilla_Listing.xml:615`).
//! Their text is protected from Lua (`securityDisableSetText`), so the client
//! writes it natively. `CreateListing`/`UpdateListing` capture the field text
//! into the active entry's `name`, `comment` and `voiceChat`.
//! INFERRED (R-request pending): the reference-name to entry-field mapping.

use crate::lua_api::frame::methods::text_attribute_event::set_frame_text_value;
use crate::lua_api::methods::{borrow_state, extract_frame_id, table_get};
use crate::widget::WidgetType;
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

/// Entry text captured from the creation fields.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct CreationText {
    pub name: String,
    pub comment: String,
    pub voice_chat: String,
}

#[derive(Clone, Copy)]
enum Field {
    Name,
    Comment,
    VoiceChat,
}

const FIELD_REFERENCES: &[(Field, &str)] = &[
    (Field::Name, "LFGListCreationName"),
    (Field::Comment, "LFGListCreationDescription"),
    (Field::Comment, "LFGListingComment"),
    (Field::VoiceChat, "LFGListCreationVoiceChat"),
];

impl CreationText {
    fn field(&self, field: Field) -> &str {
        match field {
            Field::Name => &self.name,
            Field::Comment => &self.comment,
            Field::VoiceChat => &self.voice_chat,
        }
    }

    fn field_mut(&mut self, field: Field) -> &mut String {
        match field {
            Field::Name => &mut self.name,
            Field::Comment => &mut self.comment,
            Field::VoiceChat => &mut self.voice_chat,
        }
    }
}

pub(crate) fn clear_creation_text_fields(state: &mut LuaState) -> LuaResult<u32> {
    write_fields(state, &CreationText::default())?;
    Ok(0)
}

/// Without an active listing the fields keep their text.
pub(crate) fn copy_active_entry_info_to_creation_fields(state: &mut LuaState) -> LuaResult<u32> {
    let text = borrow_state(state)?
        .lfg_active_entry
        .as_ref()
        .map(|entry| entry.text.clone());
    if let Some(text) = text {
        write_fields(state, &text)?;
    }
    Ok(0)
}

/// Text of the registered creation fields; an unregistered field reads empty.
pub(crate) fn read_creation_text(state: &mut LuaState) -> LuaResult<CreationText> {
    let mut text = CreationText::default();
    for (field, reference) in FIELD_REFERENCES {
        let Some(id) = registered_edit_box(state, reference)? else {
            continue;
        };
        let value = borrow_state(state)?
            .widgets
            .get(id)
            .and_then(|frame| frame.text.clone())
            .unwrap_or_default();
        *text.field_mut(*field) = value;
    }
    Ok(text)
}

fn write_fields(state: &mut LuaState, text: &CreationText) -> LuaResult<()> {
    for (field, reference) in FIELD_REFERENCES {
        if let Some(id) = registered_edit_box(state, reference)? {
            set_frame_text_value(state, id, Some(text.field(*field).to_string()), false)?;
        }
    }
    Ok(())
}

/// `StoreSecureReference` currently publishes references as globals
/// (`workarounds/temporary/secure_reference_defaults.rs`).
fn registered_edit_box(state: &mut LuaState, reference: &str) -> LuaResult<Option<u64>> {
    let globals = Val::Table(state.global);
    let value = table_get(state, globals, reference);
    let Some(id) = extract_frame_id(state, value) else {
        return Ok(None);
    };
    let is_edit_box = borrow_state(state)?
        .widgets
        .get(id)
        .is_some_and(|frame| frame.widget_type == WidgetType::EditBox);
    Ok(is_edit_box.then_some(id))
}
