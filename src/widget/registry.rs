//! Global widget registry for tracking all widgets.

use super::Frame;
use std::cell::Cell;
use std::collections::{HashMap, HashSet, VecDeque};

/// Registry of all widgets in the UI.
#[derive(Debug, Default)]
pub struct WidgetRegistry {
    /// Widgets by ID.
    widgets: HashMap<u64, Frame>,
    /// Widget IDs by name.
    names: HashMap<String, u64>,
    /// Set to true when any widget is mutated; cleared by the render loop.
    render_dirty: Cell<bool>,
}

impl WidgetRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a new widget.
    pub fn register(&mut self, widget: Frame) -> u64 {
        let id = widget.id;
        // Debug: check for re-registration that would lose children
        if let Some(existing) = self.widgets.get(&id)
            && !existing.children.is_empty() {
                eprintln!("[WARN] Re-registering widget id={} name={:?} which has {} children!",
                    id, existing.name, existing.children.len());
            }
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

    /// Get a mutable widget by ID. Marks the registry as render-dirty.
    pub fn get_mut(&mut self, id: u64) -> Option<&mut Frame> {
        let result = self.widgets.get_mut(&id);
        if result.is_some() {
            self.render_dirty.set(true);
        }
        result
    }

    /// Get a mutable widget by ID without marking render-dirty.
    ///
    /// Use for mutations that don't affect rendering (script tables, internal
    /// bookkeeping, event registration, etc.).
    pub fn get_mut_silent(&mut self, id: u64) -> Option<&mut Frame> {
        self.widgets.get_mut(&id)
    }

    /// Explicitly mark the registry as render-dirty.
    pub fn mark_render_dirty(&self) {
        self.render_dirty.set(true);
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

    /// Check and clear the render-dirty flag. Returns true if any widget was mutated.
    pub fn take_render_dirty(&self) -> bool {
        self.render_dirty.replace(false)
    }

    /// Check if setting a point from `frame_id` to `relative_to_id` would create a cycle.
    /// A cycle exists if relative_to (or any of its anchor dependencies) already
    /// depends on frame_id.
    pub fn would_create_anchor_cycle(&self, frame_id: u64, relative_to_id: u64) -> bool {
        // Can't anchor to yourself
        if frame_id == relative_to_id {
            return true;
        }

        // BFS from relative_to, checking if any dependency points back to frame_id
        let mut queue = VecDeque::new();
        let mut seen = HashSet::new();

        queue.push_back(relative_to_id);
        seen.insert(relative_to_id);

        while let Some(check_id) = queue.pop_front() {
            if let Some(frame) = self.widgets.get(&check_id) {
                for anchor in &frame.anchors {
                    if let Some(anchor_target) = anchor.relative_to_id {
                        let target_id = anchor_target as u64;
                        if target_id == frame_id {
                            return true;
                        }
                        if seen.insert(target_id) {
                            queue.push_back(target_id);
                        }
                    }
                }
            }
        }

        false
    }
}
