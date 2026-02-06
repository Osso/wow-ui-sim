//! Event system for WoW-style event dispatch.

use std::collections::HashMap;

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
}

impl ScriptHandler {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "OnEvent" => Some(Self::OnEvent),
            "OnUpdate" => Some(Self::OnUpdate),
            "OnShow" => Some(Self::OnShow),
            "OnHide" => Some(Self::OnHide),
            "OnClick" => Some(Self::OnClick),
            "OnEnter" => Some(Self::OnEnter),
            "OnLeave" => Some(Self::OnLeave),
            "OnMouseDown" => Some(Self::OnMouseDown),
            "OnMouseUp" => Some(Self::OnMouseUp),
            "OnDragStart" => Some(Self::OnDragStart),
            "OnDragStop" => Some(Self::OnDragStop),
            "OnReceiveDrag" => Some(Self::OnReceiveDrag),
            "OnMouseWheel" => Some(Self::OnMouseWheel),
            "OnSizeChanged" => Some(Self::OnSizeChanged),
            "OnLoad" => Some(Self::OnLoad),
            "OnAttributeChanged" => Some(Self::OnAttributeChanged),
            "OnTooltipCleared" => Some(Self::OnTooltipCleared),
            "OnTooltipSetItem" => Some(Self::OnTooltipSetItem),
            "OnTooltipSetUnit" => Some(Self::OnTooltipSetUnit),
            "OnTooltipSetSpell" => Some(Self::OnTooltipSetSpell),
            "OnPostUpdate" => Some(Self::OnPostUpdate),
            "OnPostShow" => Some(Self::OnPostShow),
            "OnPostHide" => Some(Self::OnPostHide),
            "OnPostClick" => Some(Self::OnPostClick),
            "OnKeyDown" => Some(Self::OnKeyDown),
            "OnKeyUp" => Some(Self::OnKeyUp),
            "OnChar" => Some(Self::OnChar),
            "OnEnterPressed" => Some(Self::OnEnterPressed),
            "OnEscapePressed" => Some(Self::OnEscapePressed),
            "OnTabPressed" => Some(Self::OnTabPressed),
            "OnSpacePressed" => Some(Self::OnSpacePressed),
            "OnEditFocusGained" => Some(Self::OnEditFocusGained),
            "OnEditFocusLost" => Some(Self::OnEditFocusLost),
            "OnTextChanged" => Some(Self::OnTextChanged),
            "OnValueChanged" => Some(Self::OnValueChanged),
            "OnMinMaxChanged" => Some(Self::OnMinMaxChanged),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::OnEvent => "OnEvent",
            Self::OnUpdate => "OnUpdate",
            Self::OnShow => "OnShow",
            Self::OnHide => "OnHide",
            Self::OnClick => "OnClick",
            Self::OnEnter => "OnEnter",
            Self::OnLeave => "OnLeave",
            Self::OnMouseDown => "OnMouseDown",
            Self::OnMouseUp => "OnMouseUp",
            Self::OnDragStart => "OnDragStart",
            Self::OnDragStop => "OnDragStop",
            Self::OnReceiveDrag => "OnReceiveDrag",
            Self::OnMouseWheel => "OnMouseWheel",
            Self::OnSizeChanged => "OnSizeChanged",
            Self::OnLoad => "OnLoad",
            Self::OnAttributeChanged => "OnAttributeChanged",
            Self::OnTooltipCleared => "OnTooltipCleared",
            Self::OnTooltipSetItem => "OnTooltipSetItem",
            Self::OnTooltipSetUnit => "OnTooltipSetUnit",
            Self::OnTooltipSetSpell => "OnTooltipSetSpell",
            Self::OnPostUpdate => "OnPostUpdate",
            Self::OnPostShow => "OnPostShow",
            Self::OnPostHide => "OnPostHide",
            Self::OnPostClick => "OnPostClick",
            Self::OnKeyDown => "OnKeyDown",
            Self::OnKeyUp => "OnKeyUp",
            Self::OnChar => "OnChar",
            Self::OnEnterPressed => "OnEnterPressed",
            Self::OnEscapePressed => "OnEscapePressed",
            Self::OnTabPressed => "OnTabPressed",
            Self::OnSpacePressed => "OnSpacePressed",
            Self::OnEditFocusGained => "OnEditFocusGained",
            Self::OnEditFocusLost => "OnEditFocusLost",
            Self::OnTextChanged => "OnTextChanged",
            Self::OnValueChanged => "OnValueChanged",
            Self::OnMinMaxChanged => "OnMinMaxChanged",
        }
    }
}

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
