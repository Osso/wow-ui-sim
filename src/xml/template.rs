//! Template registry for virtual frames.

use super::types::FrameXml;
use std::collections::{HashMap, HashSet};
use std::sync::{OnceLock, RwLock};

/// Stores a template (virtual frame) with its widget type.
#[derive(Debug, Clone)]
pub struct TemplateEntry {
    pub name: String,
    pub widget_type: String,
    pub frame: FrameXml,
}

/// Global registry of XML templates (virtual frames).
fn template_registry() -> &'static RwLock<HashMap<String, TemplateEntry>> {
    static REGISTRY: OnceLock<RwLock<HashMap<String, TemplateEntry>>> = OnceLock::new();
    REGISTRY.get_or_init(|| RwLock::new(HashMap::new()))
}

/// Register a template (virtual frame) in the global registry.
pub fn register_template(name: &str, widget_type: &str, frame: FrameXml) {
    let mut registry = template_registry().write().unwrap();
    registry.insert(
        name.to_string(),
        TemplateEntry {
            name: name.to_string(),
            widget_type: widget_type.to_string(),
            frame,
        },
    );
}

/// Get a template by name from the registry.
pub fn get_template(name: &str) -> Option<TemplateEntry> {
    let registry = template_registry().read().unwrap();
    registry.get(name).cloned()
}

/// Template info for C_XMLUtil.GetTemplateInfo.
pub struct TemplateInfo {
    pub frame_type: String,
    pub width: f32,
    pub height: f32,
}

/// Get template info (type, width, height) by resolving inheritance chain.
pub fn get_template_info(name: &str) -> Option<TemplateInfo> {
    let chain = get_template_chain(name);
    if chain.is_empty() {
        return None;
    }

    // Get the widget type from the first entry that defines it
    let frame_type = chain
        .iter()
        .find(|e| !e.widget_type.is_empty())
        .map(|e| e.widget_type.clone())
        .unwrap_or_else(|| "Frame".to_string());

    // Resolve size by looking through inheritance chain (most derived wins)
    let mut width: f32 = 0.0;
    let mut height: f32 = 0.0;

    for entry in &chain {
        if let Some(size) = entry.frame.size() {
            // Check AbsDimension first, then direct attributes
            if let Some(ref abs) = size.abs_dimension {
                if let Some(x) = abs.x {
                    width = x;
                }
                if let Some(y) = abs.y {
                    height = y;
                }
            }
            if let Some(x) = size.x {
                width = x;
            }
            if let Some(y) = size.y {
                height = y;
            }
        }
    }

    Some(TemplateInfo {
        frame_type,
        width,
        height,
    })
}

/// Get the full inheritance chain for a template (including the template itself).
/// Returns templates in order from most base to most derived.
pub fn get_template_chain(names: &str) -> Vec<TemplateEntry> {
    let mut chain = Vec::new();
    let mut visited = HashSet::new();

    // Process comma-separated template names
    for name in names.split(',').map(|s| s.trim()) {
        if name.is_empty() || visited.contains(name) {
            continue;
        }
        collect_template_chain(name, &mut chain, &mut visited);
    }

    chain
}

/// Recursively collect templates in the inheritance chain.
fn collect_template_chain(name: &str, chain: &mut Vec<TemplateEntry>, visited: &mut HashSet<String>) {
    if visited.contains(name) {
        return;
    }
    visited.insert(name.to_string());

    if let Some(entry) = get_template(name) {
        // First, process parent templates (if this template inherits from others)
        if let Some(ref inherits) = entry.frame.inherits {
            for parent in inherits.split(',').map(|s| s.trim()) {
                if !parent.is_empty() {
                    collect_template_chain(parent, chain, visited);
                }
            }
        }
        // Then add this template
        chain.push(entry);
    }
}

/// Clear the template registry (useful for testing).
#[allow(dead_code)]
pub fn clear_templates() {
    let mut registry = template_registry().write().unwrap();
    registry.clear();
}

// ---------------------------------------------------------------------------
// Texture template registry (virtual textures with mixin/inherits)
// ---------------------------------------------------------------------------

use super::types::TextureXml;

/// Global registry of virtual texture templates.
fn texture_template_registry() -> &'static RwLock<HashMap<String, TextureXml>> {
    static REGISTRY: OnceLock<RwLock<HashMap<String, TextureXml>>> = OnceLock::new();
    REGISTRY.get_or_init(|| RwLock::new(HashMap::new()))
}

/// Register a virtual texture template.
pub fn register_texture_template(name: &str, texture: TextureXml) {
    let mut registry = texture_template_registry().write().unwrap();
    registry.insert(name.to_string(), texture);
}

/// Collect all mixins for a texture by resolving its `inherits` chain.
pub fn collect_texture_mixins(texture: &TextureXml) -> Vec<String> {
    let mut mixins = Vec::new();

    // Collect mixins from inherited templates
    if let Some(ref inherits) = texture.inherits {
        let registry = texture_template_registry().read().unwrap();
        for parent_name in inherits.split(',').map(|s| s.trim()) {
            if let Some(parent) = registry.get(parent_name) {
                if let Some(ref m) = parent.mixin {
                    for mixin in m.split(',').map(|s| s.trim()) {
                        if !mixin.is_empty() && !mixins.contains(&mixin.to_string()) {
                            mixins.push(mixin.to_string());
                        }
                    }
                }
            }
        }
    }

    // Collect direct mixins on the texture itself
    if let Some(ref m) = texture.mixin {
        for mixin in m.split(',').map(|s| s.trim()) {
            if !mixin.is_empty() && !mixins.contains(&mixin.to_string()) {
                mixins.push(mixin.to_string());
            }
        }
    }

    mixins
}
