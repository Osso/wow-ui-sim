//! Global widget registry for tracking all widgets.

use super::Frame;
use std::collections::HashMap;

/// Registry of all widgets in the UI.
#[derive(Debug, Default)]
pub struct WidgetRegistry {
    /// Widgets by ID.
    widgets: HashMap<u64, Frame>,
    /// Widget IDs by name.
    names: HashMap<String, u64>,
}

impl WidgetRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a new widget.
    pub fn register(&mut self, widget: Frame) -> u64 {
        let id = widget.id;
        if let Some(ref name) = widget.name {
            self.names.insert(name.clone(), id);
        }
        self.widgets.insert(id, widget);
        id
    }

    /// Get a widget by ID.
    pub fn get(&self, id: u64) -> Option<&Frame> {
        self.widgets.get(&id)
    }

    /// Get a mutable widget by ID.
    pub fn get_mut(&mut self, id: u64) -> Option<&mut Frame> {
        self.widgets.get_mut(&id)
    }

    /// Get a widget by name.
    pub fn get_by_name(&self, name: &str) -> Option<&Frame> {
        self.names.get(name).and_then(|id| self.widgets.get(id))
    }

    /// Get a widget ID by name.
    pub fn get_id_by_name(&self, name: &str) -> Option<u64> {
        self.names.get(name).copied()
    }

    /// Get all widgets registered for a specific event.
    pub fn get_event_listeners(&self, event: &str) -> Vec<u64> {
        self.widgets
            .values()
            .filter(|w| w.is_registered_for_event(event))
            .map(|w| w.id)
            .collect()
    }

    /// Add a child to a parent widget.
    pub fn add_child(&mut self, parent_id: u64, child_id: u64) {
        if let Some(parent) = self.widgets.get_mut(&parent_id) {
            parent.children.push(child_id);
        }
    }

    /// Get all widget IDs.
    pub fn all_ids(&self) -> Vec<u64> {
        self.widgets.keys().copied().collect()
    }
}
