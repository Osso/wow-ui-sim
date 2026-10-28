//! Event system facade with composition-owned validation policy.
pub use sim_event::{Event, EventArg, EventQueue, ScriptHandler, ScriptRegistry, events};
mod ptr_documented_events;
pub mod valid_events;
pub use valid_events::{
    callback_events, is_callback_event, is_registerable_event, is_restricted_event, is_valid_event,
    restricted_events,
};
