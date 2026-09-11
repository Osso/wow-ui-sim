//! Template registry for virtual frames.

use super::types::{FrameChildElement, FrameXml};
use rilua::Val;
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

/// Stores a template (virtual frame) with its widget type.
#[derive(Debug, Clone)]
pub struct TemplateEntry {
    pub name: String,
    pub widget_type: String,
    pub frame: FrameXml,
    pub local_source: Option<Val>,
}

#[derive(Default)]
struct TemplateRegistry {
    entries: HashMap<String, Arc<TemplateEntry>>,
    entries_ci: HashMap<String, String>,
    chain_cache: HashMap<String, Arc<Vec<Arc<TemplateEntry>>>>,
    lifecycle_cache: HashMap<String, (bool, bool)>,
}

thread_local! {
    static TEMPLATE_REGISTRY: RefCell<TemplateRegistry> = RefCell::new(TemplateRegistry::default());
    static TEXTURE_TEMPLATE_REGISTRY: RefCell<HashMap<String, TextureXml>> = RefCell::new(HashMap::new());
    static ANIM_GROUP_TEMPLATE_REGISTRY: RefCell<HashMap<String, AnimationGroupXml>> = RefCell::new(HashMap::new());
    static FONT_STRING_TEMPLATE_REGISTRY: RefCell<HashMap<String, FontStringXml>> = RefCell::new(HashMap::new());
}

fn with_template_registry<R>(f: impl FnOnce(&TemplateRegistry) -> R) -> R {
    TEMPLATE_REGISTRY.with(|registry| f(&registry.borrow()))
}

fn with_template_registry_mut<R>(f: impl FnOnce(&mut TemplateRegistry) -> R) -> R {
    TEMPLATE_REGISTRY.with(|registry| f(&mut registry.borrow_mut()))
}

fn with_texture_template_registry<R>(f: impl FnOnce(&HashMap<String, TextureXml>) -> R) -> R {
    TEXTURE_TEMPLATE_REGISTRY.with(|registry| f(&registry.borrow()))
}

fn with_texture_template_registry_mut<R>(
    f: impl FnOnce(&mut HashMap<String, TextureXml>) -> R,
) -> R {
    TEXTURE_TEMPLATE_REGISTRY.with(|registry| f(&mut registry.borrow_mut()))
}

fn with_anim_group_template_registry<R>(
    f: impl FnOnce(&HashMap<String, AnimationGroupXml>) -> R,
) -> R {
    ANIM_GROUP_TEMPLATE_REGISTRY.with(|registry| f(&registry.borrow()))
}

fn with_anim_group_template_registry_mut<R>(
    f: impl FnOnce(&mut HashMap<String, AnimationGroupXml>) -> R,
) -> R {
    ANIM_GROUP_TEMPLATE_REGISTRY.with(|registry| f(&mut registry.borrow_mut()))
}

/// Register a template (virtual frame) in the global registry.
pub fn register_template(name: &str, widget_type: &str, frame: FrameXml) {
    register_template_with_local_source(name, widget_type, frame, None);
}

pub fn register_template_with_local_source(
    name: &str,
    widget_type: &str,
    frame: FrameXml,
    local_source: Option<Val>,
) {
    with_template_registry_mut(|registry| {
        let lower = name.to_ascii_lowercase();
        registry.entries.insert(
            name.to_string(),
            Arc::new(TemplateEntry {
                name: name.to_string(),
                widget_type: widget_type.to_string(),
                frame,
                local_source,
            }),
        );
        registry.entries_ci.insert(lower, name.to_string());
        registry.chain_cache.clear();
        registry.lifecycle_cache.clear();
    });
}

/// Get a template by name from the registry (case-insensitive).
///
/// WoW's CreateFrame passes type names in various cases (e.g. "DROPDOWNBUTTON"
/// from Lua vs "DropdownButton" from XML). The registry stores the canonical
/// PascalCase name from the XML definition.
pub fn get_template(name: &str) -> Option<TemplateEntry> {
    get_template_arc(name).map(|entry| entry.as_ref().clone())
}

fn get_template_arc(name: &str) -> Option<Arc<TemplateEntry>> {
    with_template_registry(|registry| {
        if let Some(entry) = registry.entries.get(name) {
            return Some(Arc::clone(entry));
        }
        let lower = name.to_ascii_lowercase();
        registry
            .entries_ci
            .get(&lower)
            .and_then(|canonical| registry.entries.get(canonical))
            .cloned()
    })
}

/// Template info for C_XMLUtil.GetTemplateInfo.
#[derive(Debug, Clone)]
pub struct TemplateKeyValueInfo {
    pub key: String,
    pub value: String,
    pub value_type: Option<String>,
}

#[derive(Debug, Clone)]
pub struct TemplateInfo {
    pub frame_type: String,
    pub template_name: String,
    pub width: f32,
    pub height: f32,
    pub key_values: Vec<TemplateKeyValueInfo>,
}

/// Get template info (type, width, height) by resolving inheritance chain.
pub fn get_template_info(name: &str) -> Option<TemplateInfo> {
    let chain = get_template_chain(name);
    if chain.is_empty() {
        return None;
    }
    let frame_type = resolve_frame_type(&chain);
    let (width, height) = resolve_chain_size(&chain);
    let template_name = chain
        .last()
        .map(|entry| entry.name.clone())
        .unwrap_or_else(|| name.to_string());
    let key_values = collect_key_values(&chain);
    Some(TemplateInfo {
        frame_type,
        template_name,
        width,
        height,
        key_values,
    })
}

/// Resolve the frame type from an inheritance chain.
///
/// The most derived entry's widget_type wins. The chain is base-to-derived;
/// e.g. `<Button inherits="FrameTemplate">` is a Button, not a Frame.
/// However, many templates inherit from Frame-based parents without explicitly
/// redefining their type. The last entry in the chain is the template itself
/// (most derived) — use its type if non-empty, otherwise fall back to parents.
fn resolve_frame_type(chain: &[Arc<TemplateEntry>]) -> String {
    chain
        .last()
        .filter(|e| !e.widget_type.is_empty())
        .map(|e| e.widget_type.clone())
        .or_else(|| {
            chain
                .iter()
                .find(|e| !e.widget_type.is_empty())
                .map(|e| e.widget_type.clone())
        })
        .unwrap_or_else(|| "Frame".to_string())
}

/// Collect key-value pairs from all entries in the inheritance chain.
fn collect_key_values(chain: &[Arc<TemplateEntry>]) -> Vec<TemplateKeyValueInfo> {
    chain
        .iter()
        .flat_map(|entry| entry.frame.all_key_values())
        .flat_map(|key_values| key_values.values.iter())
        .map(|key_value| TemplateKeyValueInfo {
            key: key_value.key.clone(),
            value: key_value.value.clone(),
            value_type: key_value.value_type.clone(),
        })
        .collect()
}

/// Resolve (width, height) across the inheritance chain.
/// Most derived entry wins. Within an entry, direct `x`/`y` attrs override `AbsDimension`.
fn resolve_chain_size(chain: &[Arc<TemplateEntry>]) -> (f32, f32) {
    let mut width: f32 = 0.0;
    let mut height: f32 = 0.0;
    for entry in chain {
        let Some(size) = entry.frame.size() else {
            continue;
        };
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
    (width, height)
}

/// Get the full inheritance chain for a template (including the template itself).
/// Returns templates in order from most base to most derived.
/// Returns Arc to avoid cloning the chain on every access.
pub fn get_template_chain(names: &str) -> Arc<Vec<Arc<TemplateEntry>>> {
    let key = names.trim().to_string();
    if key.is_empty() {
        return Arc::new(Vec::new());
    }

    if let Some(cached) = with_template_registry(|registry| registry.chain_cache.get(&key).cloned())
    {
        return cached;
    }

    let mut chain = Vec::new();
    let mut visited = HashSet::new();

    // Process comma-separated template names
    for name in key.split(',').map(|s| s.trim()) {
        if name.is_empty() || visited.contains(name) {
            continue;
        }
        collect_template_chain(name, &mut chain, &mut visited);
    }

    let arc_chain = Arc::new(chain);
    with_template_registry_mut(|registry| {
        registry.chain_cache.insert(key, Arc::clone(&arc_chain));
    });
    arc_chain
}

/// Get cached lifecycle flags (OnLoad, OnShow) for a template inheritance chain.
pub fn get_template_lifecycle_flags(names: &str) -> (bool, bool) {
    let key = names.trim().to_string();
    if key.is_empty() {
        return (false, false);
    }

    if let Some(cached) =
        with_template_registry(|registry| registry.lifecycle_cache.get(&key).copied())
    {
        return cached;
    }

    let chain = get_template_chain(&key);
    let mut on_load = false;
    let mut on_show = false;
    for entry in chain.iter() {
        let Some(scripts) = entry.frame.scripts() else {
            continue;
        };
        on_load |= !scripts.on_load.is_empty();
        on_show |= !scripts.on_show.is_empty();
        if on_load && on_show {
            break;
        }
    }

    let flags = (on_load, on_show);
    with_template_registry_mut(|registry| {
        registry.lifecycle_cache.insert(key, flags);
    });
    flags
}

/// Recursively collect templates in the inheritance chain.
fn collect_template_chain(
    name: &str,
    chain: &mut Vec<Arc<TemplateEntry>>,
    visited: &mut HashSet<String>,
) {
    if visited.contains(name) {
        return;
    }
    visited.insert(name.to_string());

    if let Some(entry) = get_template_arc(name) {
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

/// Register synthetic templates for C++ intrinsic frame types.
///
/// WoW has several frame types built into the C++ engine that don't have XML
/// definitions in the extracted Interface files. They behave as templates that
/// inherit from a base template and apply a mixin. Register them so the
/// template chain resolution and CreateFrame can find them.
pub fn register_intrinsic_templates() {
    register_intrinsic_frame_templates();
    register_button_frame_template();
    super::profile_templates::register_all();
}

struct IntrinsicFrameTemplate {
    name: &'static str,
    widget_type: &'static str,
    inherits: &'static str,
    mixin: &'static str,
}

fn register_intrinsic_frame_templates() {
    for spec in intrinsic_frame_templates() {
        register_intrinsic_frame_template(spec);
    }
}

fn intrinsic_frame_templates() -> &'static [IntrinsicFrameTemplate] {
    &[
        IntrinsicFrameTemplate {
            name: "WoWScrollBoxList",
            widget_type: "Frame",
            inherits: "ScrollBoxBaseTemplate",
            mixin: "ScrollBoxListMixin",
        },
        IntrinsicFrameTemplate {
            name: "WoWScrollBox",
            widget_type: "Frame",
            inherits: "ScrollBoxBaseTemplate",
            mixin: "ScrollBoxBaseMixin",
        },
        IntrinsicFrameTemplate {
            name: "WoWTrimScrollBar",
            widget_type: "EventFrame",
            inherits: "WowTrimScrollBarTemplate",
            mixin: "",
        },
        IntrinsicFrameTemplate {
            name: "UIThemeContainerFrame",
            widget_type: "Frame",
            inherits: "",
            mixin: "UIThemeContainerMixin",
        },
    ]
}

fn register_intrinsic_frame_template(spec: &IntrinsicFrameTemplate) {
    register_template(spec.name, spec.widget_type, intrinsic_frame_xml(spec));
}

fn intrinsic_frame_xml(spec: &IntrinsicFrameTemplate) -> FrameXml {
    FrameXml {
        inherits: Some(spec.inherits.to_string()),
        mixin: intrinsic_mixin(spec.mixin),
        is_virtual: Some(true),
        ..Default::default()
    }
}

fn intrinsic_mixin(mixin: &str) -> Option<String> {
    if mixin.is_empty() {
        None
    } else {
        Some(mixin.to_string())
    }
}

fn register_button_frame_template() {
    register_template("ButtonFrameTemplate", "Frame", button_frame_template_xml());
}

fn button_frame_template_xml() -> FrameXml {
    FrameXml {
        is_virtual: Some(true),
        children: vec![button_frame_inset_child()],
        ..Default::default()
    }
}

fn button_frame_inset_child() -> FrameChildElement {
    FrameChildElement::Frame(FrameXml {
        name: Some("$parentInset".to_string()),
        parent_key: Some("Inset".to_string()),
        inherits: Some("InsetFrameTemplate".to_string()),
        ..Default::default()
    })
}

/// Clear the template registry (useful for testing).
pub fn clear_templates() {
    with_template_registry_mut(|registry| {
        registry.entries.clear();
        registry.entries_ci.clear();
        registry.chain_cache.clear();
        registry.lifecycle_cache.clear();
    });

    with_texture_template_registry_mut(|registry| registry.clear());
    with_anim_group_template_registry_mut(|registry| registry.clear());
    super::animation_templates::clear_animation_templates();
    with_font_string_template_registry_mut(|registry| registry.clear());
}

// ---------------------------------------------------------------------------
// Texture template registry (virtual textures with mixin/inherits)
// ---------------------------------------------------------------------------

use super::types_animation::AnimationGroupXml;
use super::types_elements::{FontStringXml, TextureXml};

fn with_font_string_template_registry<R>(
    f: impl FnOnce(&HashMap<String, FontStringXml>) -> R,
) -> R {
    FONT_STRING_TEMPLATE_REGISTRY.with(|registry| f(&registry.borrow()))
}

fn with_font_string_template_registry_mut<R>(
    f: impl FnOnce(&mut HashMap<String, FontStringXml>) -> R,
) -> R {
    FONT_STRING_TEMPLATE_REGISTRY.with(|registry| f(&mut registry.borrow_mut()))
}

/// Register a virtual `<FontString>` template (e.g.
/// `UserScaledFontStringTemplate` from Blizzard_AccessibilityTemplates).
///
/// FontStrings live in their own registry rather than the unified
/// `TemplateRegistry` (which only holds `FrameXml`) so the inherits-chain
/// walker can keep its frame-vs-fontstring shape distinction.
pub fn register_font_string_template(name: &str, fontstring: FontStringXml) {
    with_font_string_template_registry_mut(|registry| {
        registry.insert(name.to_string(), fontstring);
    });
}

/// Get a virtual `<FontString>` template by name. Returns `None` when the
/// name is unknown — callers should treat this as the inherits-chain
/// terminator for FontString instances.
pub fn get_font_string_template(name: &str) -> Option<FontStringXml> {
    with_font_string_template_registry(|registry| registry.get(name).cloned())
}

/// Collect all mixins for a FontString by resolving its `inherits` chain.
pub fn collect_font_string_mixins(inherits: Option<&str>, own_mixin: Option<&str>) -> Vec<String> {
    with_font_string_template_registry(|registry| {
        collect_inherited_mixins(inherits, own_mixin, |name| {
            registry.get(name).and_then(|p| p.mixin.clone())
        })
    })
}

/// Register a virtual texture template.
pub fn register_texture_template(name: &str, texture: TextureXml) {
    with_texture_template_registry_mut(|registry| {
        registry.insert(name.to_string(), texture);
    });
}

/// Get size from a texture template by name. Returns (width, height) if found.
pub fn get_texture_template_size(name: &str) -> Option<(f32, f32)> {
    with_texture_template_registry(|registry| {
        let tex = registry.get(name)?;
        let size = tex.size.as_ref()?;
        let w = size.x.unwrap_or(0.0);
        let h = size.y.unwrap_or(0.0);
        if w > 0.0 && h > 0.0 {
            Some((w, h))
        } else {
            None
        }
    })
}

/// Resolve texture inheritance: merge properties from the template chain.
///
/// Returns a new `TextureXml` with inherited properties filled in.
/// Instance properties override template properties (most-derived wins).
pub fn resolve_texture_inheritance(texture: &TextureXml) -> TextureXml {
    let Some(ref inherits) = texture.inherits else {
        return texture.clone();
    };

    let templates = with_texture_template_registry(|registry| {
        let mut templates = Vec::new();
        for parent_name in inherits.split(',').map(|s| s.trim()) {
            if let Some(parent) = registry.get(parent_name) {
                templates.push(parent.clone());
            }
        }
        templates
    });

    if templates.is_empty() {
        return texture.clone();
    }

    // Start with first template as base, overlay subsequent templates, then instance
    let mut merged = templates[0].clone();
    for tmpl in &templates[1..] {
        merge_texture_fields(&mut merged, tmpl);
    }
    merge_texture_fields(&mut merged, texture);

    // Preserve instance identity fields
    merged.name = texture.name.clone();
    merged.parent_key = texture.parent_key.clone();
    merged.parent_array = texture.parent_array.clone();
    merged.is_virtual = texture.is_virtual;
    merged.inherits = texture.inherits.clone();
    merged.anchors = texture.anchors.clone();
    merged.animations = texture.animations.clone();
    merged.scripts = texture.scripts.clone();
    merged.masked_textures = texture.masked_textures.clone();

    merged
}

/// Overlay `src` fields onto `dst` where `src` has a value.
fn merge_texture_fields(dst: &mut TextureXml, src: &TextureXml) {
    macro_rules! merge_opt {
        ($field:ident) => {
            if src.$field.is_some() {
                dst.$field = src.$field.clone();
            }
        };
    }
    merge_opt!(file);
    merge_opt!(atlas);
    merge_opt!(use_atlas_size);
    merge_opt!(tex_coords);
    merge_opt!(size);
    merge_opt!(color);
    merge_opt!(horiz_tile);
    merge_opt!(vert_tile);
    merge_opt!(h_wrap_mode);
    merge_opt!(v_wrap_mode);
    merge_opt!(thickness);
    merge_opt!(hidden);
    merge_opt!(alpha);
    merge_opt!(set_all_points);
    merge_opt!(mixin);
    merge_texture_blend_mode(dst, src);
}

fn merge_texture_blend_mode(dst: &mut TextureXml, src: &TextureXml) {
    let Some(mode) = src.effective_blend_mode() else {
        return;
    };
    let mode = mode.to_string();
    dst.alpha_mode = Some(mode.clone());
    dst.blend_mode = Some(mode);
}

/// Collect all mixins for a texture by resolving its `inherits` chain.
pub fn collect_texture_mixins(texture: &TextureXml) -> Vec<String> {
    with_texture_template_registry(|registry| {
        collect_inherited_mixins(
            texture.inherits.as_deref(),
            texture.mixin.as_deref(),
            |name| registry.get(name).and_then(|p| p.mixin.clone()),
        )
    })
}

/// Collect all key-value pairs for a texture by resolving its `inherits` chain.
pub fn collect_texture_key_values(texture: &TextureXml) -> Vec<TemplateKeyValueInfo> {
    with_texture_template_registry(|registry| {
        let mut values = Vec::new();
        let mut visited = HashSet::new();
        collect_texture_key_values_from_inherits(
            registry,
            texture.inherits.as_deref(),
            &mut visited,
            &mut values,
        );
        append_texture_key_values(texture, &mut values);
        values
    })
}

fn collect_texture_key_values_from_inherits(
    registry: &HashMap<String, TextureXml>,
    inherits: Option<&str>,
    visited: &mut HashSet<String>,
    values: &mut Vec<TemplateKeyValueInfo>,
) {
    let Some(inherits) = inherits else { return };
    for name in inherits.split(',').map(|s| s.trim()) {
        if name.is_empty() || !visited.insert(name.to_string()) {
            continue;
        }
        let Some(template) = registry.get(name) else {
            continue;
        };
        collect_texture_key_values_from_inherits(
            registry,
            template.inherits.as_deref(),
            visited,
            values,
        );
        append_texture_key_values(template, values);
    }
}

fn append_texture_key_values(texture: &TextureXml, values: &mut Vec<TemplateKeyValueInfo>) {
    let Some(key_values) = &texture.key_values else {
        return;
    };
    values.extend(
        key_values
            .values
            .iter()
            .map(|key_value| TemplateKeyValueInfo {
                key: key_value.key.clone(),
                value: key_value.value.clone(),
                value_type: key_value.value_type.clone(),
            }),
    );
}

// ---------------------------------------------------------------------------
// AnimationGroup template registry (virtual animation groups with mixin)
// ---------------------------------------------------------------------------

/// Register a virtual AnimationGroup template.
pub fn register_anim_group_template(name: &str, anim_group: AnimationGroupXml) {
    with_anim_group_template_registry_mut(|registry| {
        registry.insert(name.to_string(), anim_group);
    });
}

pub fn get_anim_group_template(name: &str) -> Option<AnimationGroupXml> {
    with_anim_group_template_registry(|registry| registry.get(name).cloned())
}

/// Collect all mixins for an AnimationGroup by resolving its `inherits` chain.
pub fn collect_anim_group_mixins(anim_group: &AnimationGroupXml) -> Vec<String> {
    with_anim_group_template_registry(|registry| {
        collect_inherited_mixins(
            anim_group.inherits.as_deref(),
            anim_group.mixin.as_deref(),
            |name| registry.get(name).and_then(|p| p.mixin.clone()),
        )
    })
}

/// Append unique mixin names from a comma-separated string.
/// Resolve mixins from an inherits chain + own mixin attribute.
/// `lookup_parent_mixin` maps a parent template name to its mixin string (owned).
fn collect_inherited_mixins(
    inherits: Option<&str>,
    own_mixin: Option<&str>,
    lookup_parent_mixin: impl Fn(&str) -> Option<String>,
) -> Vec<String> {
    let mut mixins = Vec::new();
    let mut seen_mixins = HashSet::new();
    if let Some(inherits) = inherits {
        for parent_name in inherits.split(',').map(|s| s.trim()) {
            if let Some(parent_mixin) = lookup_parent_mixin(parent_name) {
                append_unique_mixins(&mut mixins, &mut seen_mixins, Some(&parent_mixin));
            }
        }
    }
    append_unique_mixins(&mut mixins, &mut seen_mixins, own_mixin);
    mixins
}

fn append_unique_mixins(
    mixins: &mut Vec<String>,
    seen_mixins: &mut HashSet<String>,
    attr: Option<&str>,
) {
    let Some(attr) = attr else { return };
    for name in attr.split(',').map(|s| s.trim()) {
        if !name.is_empty() && seen_mixins.insert(name.to_string()) {
            mixins.push(name.to_string());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn intrinsic_templates_register_engine_frames_and_button_frame_parts() {
        register_intrinsic_templates();

        let scroll_box = get_template("WoWScrollBoxList").expect("scroll box list intrinsic");
        assert_eq!(scroll_box.widget_type, "Frame");
        assert_eq!(
            scroll_box.frame.inherits.as_deref(),
            Some("ScrollBoxBaseTemplate")
        );
        assert_eq!(
            scroll_box.frame.mixin.as_deref(),
            Some("ScrollBoxListMixin")
        );

        let button_frame = get_template("ButtonFrameTemplate").expect("button frame intrinsic");
        assert_eq!(button_frame.widget_type, "Frame");
        assert_eq!(button_frame.frame.children.len(), 1);
        let FrameChildElement::Frame(inset) = &button_frame.frame.children[0] else {
            panic!("ButtonFrameTemplate child should be an inset frame");
        };
        assert_eq!(inset.parent_key.as_deref(), Some("Inset"));
        assert_eq!(inset.inherits.as_deref(), Some("InsetFrameTemplate"));
    }
}
