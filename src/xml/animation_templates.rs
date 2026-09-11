//! Virtual animation definitions, separate from widget and animation-group templates.
use std::cell::RefCell;
use std::collections::HashMap;

use super::AnimationXml;

thread_local! {
    static ANIMATION_TEMPLATES: RefCell<HashMap<String, AnimationXml>> = RefCell::new(HashMap::new());
}

pub fn register_animation_template(name: &str, animation: AnimationXml) {
    ANIMATION_TEMPLATES.with(|templates| {
        templates.borrow_mut().insert(name.to_owned(), animation);
    });
}

pub(super) fn clear_animation_templates() {
    ANIMATION_TEMPLATES.with(|templates| templates.borrow_mut().clear());
}

/// Resolve parents before children, retaining explicit left-to-right inheritance order.
/// Unknown names and recursion are rejected before the factory allocates an instance.
pub fn resolve_animation_templates(inherits: &str) -> Result<Vec<AnimationXml>, String> {
    let mut chain = Vec::new();
    let mut visiting = Vec::new();
    ANIMATION_TEMPLATES.with(|templates| {
        let templates = templates.borrow();
        for name in inherits
            .split(',')
            .map(str::trim)
            .filter(|name| !name.is_empty())
        {
            append_template(name, &templates, &mut visiting, &mut chain)?;
        }
        Ok(chain)
    })
}

fn append_template(
    name: &str,
    templates: &HashMap<String, AnimationXml>,
    visiting: &mut Vec<String>,
    chain: &mut Vec<AnimationXml>,
) -> Result<(), String> {
    if visiting.iter().any(|parent| parent == name) {
        return Err(format!(
            "cyclic animation template inheritance: {} -> {name}",
            visiting.join(" -> ")
        ));
    }
    let template = templates
        .get(name)
        .ok_or_else(|| format!("unknown animation template: {name}"))?;
    visiting.push(name.to_owned());
    if let Some(inherits) = &template.inherits {
        for parent in inherits
            .split(',')
            .map(str::trim)
            .filter(|parent| !parent.is_empty())
        {
            append_template(parent, templates, visiting, chain)?;
        }
    }
    visiting.pop();
    chain.push(template.clone());
    Ok(())
}
