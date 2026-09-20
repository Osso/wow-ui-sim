//! Event system for WoW-style event dispatch.

use std::collections::HashMap;

#[cfg(any(feature = "retail-12-0-0", feature = "client-wowforever"))]
mod known_events;
pub mod valid_events;
pub use valid_events::{
    callback_events, is_callback_event, is_registerable_event, is_restricted_event, is_valid_event,
    restricted_events,
};

/// Common WoW events that addons typically use.
pub mod events {
    pub const PLAYER_LOGIN: &str = "PLAYER_LOGIN";
    pub const PLAYER_LOGOUT: &str = "PLAYER_LOGOUT";
    pub const PLAYER_ENTERING_WORLD: &str = "PLAYER_ENTERING_WORLD";
    pub const ADDON_LOADED: &str = "ADDON_LOADED";
    pub const VARIABLES_LOADED: &str = "VARIABLES_LOADED";
    pub const UPDATE_BINDINGS: &str = "UPDATE_BINDINGS";
    pub const DISPLAY_SIZE_CHANGED: &str = "DISPLAY_SIZE_CHANGED";
    pub const UI_SCALE_CHANGED: &str = "UI_SCALE_CHANGED";
    pub const PLAYER_TARGET_CHANGED: &str = "PLAYER_TARGET_CHANGED";
    pub const UNIT_HEALTH: &str = "UNIT_HEALTH";
    pub const UNIT_POWER_UPDATE: &str = "UNIT_POWER_UPDATE";
    pub const COMBAT_LOG_EVENT: &str = "COMBAT_LOG_EVENT";
    pub const CHAT_MSG_CHANNEL: &str = "CHAT_MSG_CHANNEL";
    pub const CHAT_MSG_SAY: &str = "CHAT_MSG_SAY";
    pub const CHAT_MSG_WHISPER: &str = "CHAT_MSG_WHISPER";
    pub const BAG_UPDATE: &str = "BAG_UPDATE";
    pub const BAG_UPDATE_DELAYED: &str = "BAG_UPDATE_DELAYED";
    pub const UPDATE_MOUSEOVER_UNIT: &str = "UPDATE_MOUSEOVER_UNIT";
}

/// Event queue for pending events.
#[derive(Debug, Default)]
pub struct EventQueue {
    pending: Vec<Event>,
}

/// An event with optional arguments.
#[derive(Debug, Clone)]
pub struct Event {
    pub name: String,
    pub args: Vec<EventArg>,
}

/// Event argument types.
#[derive(Debug, Clone)]
pub enum EventArg {
    String(String),
    Number(f64),
    Boolean(bool),
    Nil,
}

impl EventQueue {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push(&mut self, event: Event) {
        self.pending.push(event);
    }

    pub fn push_simple(&mut self, name: &str) {
        self.pending.push(Event {
            name: name.to_string(),
            args: Vec::new(),
        });
    }

    pub fn drain(&mut self) -> Vec<Event> {
        std::mem::take(&mut self.pending)
    }

    /// Read-only view into the pending queue — useful for tests that want to
    /// observe dispatched events without consuming them.
    pub fn pending(&self) -> &[Event] {
        &self.pending
    }

    pub fn is_empty(&self) -> bool {
        self.pending.is_empty()
    }
}

/// Script handlers that can be attached to widgets.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ScriptHandler {
    OnEvent,
    OnUpdate,
    OnShow,
    OnHide,
    OnClick,
    OnEnter,
    OnLeave,
    OnMouseDown,
    OnMouseUp,
    OnDragStart,
    OnDragStop,
    OnReceiveDrag,
    OnMouseWheel,
    OnSizeChanged,
    OnLoad,
    OnAttributeChanged,
    OnTooltipCleared,
    OnTooltipSetItem,
    OnTooltipSetUnit,
    OnTooltipSetSpell,
    OnPostUpdate,
    OnPostShow,
    OnPostHide,
    OnPostClick,
    OnKeyDown,
    OnKeyUp,
    OnChar,
    OnEnterPressed,
    OnEscapePressed,
    OnTabPressed,
    OnSpacePressed,
    OnEditFocusGained,
    OnEditFocusLost,
    OnTextChanged,
    OnValueChanged,
    OnMinMaxChanged,
    // ScrollFrame
    OnVerticalScroll,
    OnHorizontalScroll,
    OnScrollRangeChanged,
    // ColorPickerFrame
    OnColorSelect,
    // FontString / EditBox hyperlinks
    OnHyperlinkClick,
    OnHyperlinkEnter,
    OnHyperlinkLeave,
    // Button extras
    OnDoubleClick,
    OnEnable,
    OnDisable,
    // EditBox extras
    OnCursorChanged,
    OnInputLanguageChanged,
    // Animation handlers
    OnAnimFinished,
    OnAnimStarted,
    OnFinished,
    OnLoop,
    OnPlay,
    OnStop,
    // Cooldown
    OnCooldownDone,
    // GamePad
    OnGamePadButtonDown,
    OnGamePadButtonUp,
    OnGamePadStick,
    // Model / PlayerModel
    OnModelLoaded,
    OnModelCleared,
    OnDressModel,
    // Tooltip extras
    OnTooltipSetDefaultAnchor,
    OnTooltipSetFramestack,
    // Misc
    OnArrowPressed,
    OnButtonUpdate,
    OnError,
    OnExternalLink,
    OnMovieFinished,
    OnRequestNewSize,
    OnTextSet,
    OnUiMapChanged,
}

impl ScriptHandler {
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Option<Self> {
        SCRIPT_HANDLERS_BY_NAME
            .iter()
            .find_map(|(name, handler)| (*name == s).then_some(*handler))
    }

    pub fn as_str(&self) -> &'static str {
        SCRIPT_HANDLERS_BY_NAME
            .iter()
            .find_map(|(name, handler)| (*handler == *self).then_some(*name))
            .expect("ScriptHandler must exist in SCRIPT_HANDLERS_BY_NAME")
    }
}

const SCRIPT_HANDLERS_BY_NAME: &[(&'static str, ScriptHandler)] = &[
    ("OnEvent", ScriptHandler::OnEvent),
    ("OnUpdate", ScriptHandler::OnUpdate),
    ("OnShow", ScriptHandler::OnShow),
    ("OnHide", ScriptHandler::OnHide),
    ("OnClick", ScriptHandler::OnClick),
    ("OnEnter", ScriptHandler::OnEnter),
    ("OnLeave", ScriptHandler::OnLeave),
    ("OnMouseDown", ScriptHandler::OnMouseDown),
    ("OnMouseUp", ScriptHandler::OnMouseUp),
    ("OnDragStart", ScriptHandler::OnDragStart),
    ("OnDragStop", ScriptHandler::OnDragStop),
    ("OnReceiveDrag", ScriptHandler::OnReceiveDrag),
    ("OnMouseWheel", ScriptHandler::OnMouseWheel),
    ("OnSizeChanged", ScriptHandler::OnSizeChanged),
    ("OnLoad", ScriptHandler::OnLoad),
    ("OnAttributeChanged", ScriptHandler::OnAttributeChanged),
    ("OnTooltipCleared", ScriptHandler::OnTooltipCleared),
    ("OnTooltipSetItem", ScriptHandler::OnTooltipSetItem),
    ("OnTooltipSetUnit", ScriptHandler::OnTooltipSetUnit),
    ("OnTooltipSetSpell", ScriptHandler::OnTooltipSetSpell),
    ("OnPostUpdate", ScriptHandler::OnPostUpdate),
    ("OnPostShow", ScriptHandler::OnPostShow),
    ("OnPostHide", ScriptHandler::OnPostHide),
    ("OnPostClick", ScriptHandler::OnPostClick),
    ("OnKeyDown", ScriptHandler::OnKeyDown),
    ("OnKeyUp", ScriptHandler::OnKeyUp),
    ("OnChar", ScriptHandler::OnChar),
    ("OnEnterPressed", ScriptHandler::OnEnterPressed),
    ("OnEscapePressed", ScriptHandler::OnEscapePressed),
    ("OnTabPressed", ScriptHandler::OnTabPressed),
    ("OnSpacePressed", ScriptHandler::OnSpacePressed),
    ("OnEditFocusGained", ScriptHandler::OnEditFocusGained),
    ("OnEditFocusLost", ScriptHandler::OnEditFocusLost),
    ("OnTextChanged", ScriptHandler::OnTextChanged),
    ("OnValueChanged", ScriptHandler::OnValueChanged),
    ("OnMinMaxChanged", ScriptHandler::OnMinMaxChanged),
    ("OnVerticalScroll", ScriptHandler::OnVerticalScroll),
    ("OnHorizontalScroll", ScriptHandler::OnHorizontalScroll),
    ("OnScrollRangeChanged", ScriptHandler::OnScrollRangeChanged),
    ("OnColorSelect", ScriptHandler::OnColorSelect),
    ("OnHyperlinkClick", ScriptHandler::OnHyperlinkClick),
    ("OnHyperlinkEnter", ScriptHandler::OnHyperlinkEnter),
    ("OnHyperlinkLeave", ScriptHandler::OnHyperlinkLeave),
    ("OnDoubleClick", ScriptHandler::OnDoubleClick),
    ("OnEnable", ScriptHandler::OnEnable),
    ("OnDisable", ScriptHandler::OnDisable),
    ("OnCursorChanged", ScriptHandler::OnCursorChanged),
    (
        "OnInputLanguageChanged",
        ScriptHandler::OnInputLanguageChanged,
    ),
    ("OnAnimFinished", ScriptHandler::OnAnimFinished),
    ("OnAnimStarted", ScriptHandler::OnAnimStarted),
    ("OnFinished", ScriptHandler::OnFinished),
    ("OnLoop", ScriptHandler::OnLoop),
    ("OnPlay", ScriptHandler::OnPlay),
    ("OnStop", ScriptHandler::OnStop),
    ("OnCooldownDone", ScriptHandler::OnCooldownDone),
    ("OnGamePadButtonDown", ScriptHandler::OnGamePadButtonDown),
    ("OnGamePadButtonUp", ScriptHandler::OnGamePadButtonUp),
    ("OnGamePadStick", ScriptHandler::OnGamePadStick),
    // Forever Lua uses this spelling; XML uses the canonical spelling above.
    ("OnGamepadStick", ScriptHandler::OnGamePadStick),
    ("OnModelLoaded", ScriptHandler::OnModelLoaded),
    ("OnModelCleared", ScriptHandler::OnModelCleared),
    ("OnDressModel", ScriptHandler::OnDressModel),
    (
        "OnTooltipSetDefaultAnchor",
        ScriptHandler::OnTooltipSetDefaultAnchor,
    ),
    (
        "OnTooltipSetFramestack",
        ScriptHandler::OnTooltipSetFramestack,
    ),
    ("OnArrowPressed", ScriptHandler::OnArrowPressed),
    ("OnButtonUpdate", ScriptHandler::OnButtonUpdate),
    ("OnError", ScriptHandler::OnError),
    ("OnExternalLink", ScriptHandler::OnExternalLink),
    ("OnMovieFinished", ScriptHandler::OnMovieFinished),
    ("OnRequestNewSize", ScriptHandler::OnRequestNewSize),
    ("OnTextSet", ScriptHandler::OnTextSet),
    ("OnUiMapChanged", ScriptHandler::OnUiMapChanged),
];

/// Storage for script handlers (references to Lua functions).
#[derive(Debug, Default)]
pub struct ScriptRegistry {
    /// Map of widget ID -> handler type -> Lua registry key
    handlers: HashMap<u64, HashMap<ScriptHandler, i32>>,
}

impl ScriptRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set(&mut self, widget_id: u64, handler: ScriptHandler, registry_key: i32) {
        self.handlers
            .entry(widget_id)
            .or_default()
            .insert(handler, registry_key);
    }

    pub fn get(&self, widget_id: u64, handler: ScriptHandler) -> Option<i32> {
        self.handlers
            .get(&widget_id)
            .and_then(|h| h.get(&handler).copied())
    }

    pub fn remove(&mut self, widget_id: u64, handler: ScriptHandler) -> Option<i32> {
        self.handlers
            .get_mut(&widget_id)
            .and_then(|h| h.remove(&handler))
    }

    /// Remove all script handlers for a widget.
    pub fn remove_all(&mut self, widget_id: u64) {
        self.handlers.remove(&widget_id);
    }
}
