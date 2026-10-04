//! Frame name resolution and creation preparation.

use crate::lua_api::LoaderEnv;

use crate::loader::helpers::rand_id;

pub(super) struct PreparedFrameCreation {
    pub(super) name: String,
    pub(super) subst_parent: String,
    pub(super) explicit_parent: Option<String>,
    pub(super) parent: String,
    pub(super) inherits: String,
    pub(super) initial_hidden: bool,
}

pub(super) fn prepare_frame_creation(
    env: &LoaderEnv<'_>,
    frame: &crate::xml::FrameXml,
    parent_override: Option<&str>,
    subst_parent_override: Option<&str>,
    intrinsic_base: Option<&str>,
) -> Option<PreparedFrameCreation> {
    let creator_name = current_loading_addon_name(env);
    let inherited_parent_buf = resolve_parent(frame, parent_override);
    let parent_for_name = frame
        .parent
        .as_deref()
        .or(subst_parent_override)
        .or(parent_override)
        .or(inherited_parent_buf.as_deref());
    let name = resolve_frame_name(frame, parent_for_name, creator_name.as_deref())?;
    let subst_parent = resolve_subst_parent(frame, &name, subst_parent_override, parent_override);
    let explicit_parent = frame
        .parent
        .as_deref()
        .or(parent_override)
        .or(inherited_parent_buf.as_deref())
        .map(str::to_string);
    let parent = explicit_parent
        .clone()
        .unwrap_or_else(|| "UIParent".to_string());
    let inherits_buf = build_inherits_chain(frame, intrinsic_base);
    let inherits = inherits_buf
        .as_deref()
        .unwrap_or(frame.inherits.as_deref().unwrap_or(""))
        .to_string();
    let initial_hidden = resolve_xml_hidden(frame, &inherits);

    Some(PreparedFrameCreation {
        name,
        subst_parent,
        explicit_parent,
        parent,
        inherits,
        initial_hidden,
    })
}

pub(super) fn current_loading_addon_name(env: &LoaderEnv<'_>) -> Option<String> {
    let s = env.state().borrow();
    s.loading_addon_index
        .and_then(|idx| s.addons.get(idx as usize))
        .map(|a| a.folder_name.clone())
}

/// Register virtual/intrinsic frames as templates. Returns Some(None) to skip instantiation
/// for top-level virtual frames, or None to continue with normal creation.
pub(super) fn register_virtual_or_intrinsic(
    env: &LoaderEnv<'_>,
    frame: &crate::xml::FrameXml,
    widget_type: &str,
    parent_override: Option<&str>,
    intrinsic_base: Option<&str>,
) -> Option<Option<String>> {
    if frame.is_virtual != Some(true) && frame.intrinsic != Some(true) {
        return None;
    }
    if let Some(ref name) = frame.name {
        // Prepend intrinsic base to inherits so the template chain includes
        // the intrinsic mixin (e.g. DropdownButton → DropdownButtonMixin).
        let mut registered = frame.clone();
        {
            let state = env.state().borrow();
            registered.use_forbidden_object_table = state.loading_use_forbidden_object_table;
            registered.allow_untainted_creation = state.loading_allow_untainted_creation;
        }
        if let Some(base) = intrinsic_base {
            registered.inherits = Some(match &registered.inherits {
                Some(existing) if !existing.is_empty() => format!("{base}, {existing}"),
                _ => base.to_string(),
            });
        }
        let local_source = template_local_source(env);
        let declaration_taint = current_loading_template_taint(env);
        crate::xml::register_template_with_local_source(
            name,
            widget_type,
            registered,
            local_source,
            declaration_taint,
        );
    }
    if let Some(ref sm) = frame.secure_mixin {
        super::secure_mixin::apply_secure_mixins(env, sm);
    }
    if parent_override.is_none() {
        Some(None) // skip instantiation for top-level virtual frames
    } else {
        None // child virtual frames are still created
    }
}

fn current_loading_template_taint(env: &LoaderEnv<'_>) -> Option<String> {
    let state = env.state().borrow();
    let addon = state
        .loading_addon_index
        .and_then(|index| state.addons.get(index as usize))?;
    if crate::toc::addon_loads_as_blizzard_code(
        Some(&addon.folder_name),
        addon.metadata.contains_key("AllowLoad"),
        addon.use_secure_env,
    ) {
        None
    } else {
        Some(addon.folder_name.clone())
    }
}

fn template_local_source(env: &LoaderEnv<'_>) -> Option<rilua::Val> {
    let scoped_env = env.state().borrow().loading_scoped_script_env;
    if scoped_env.is_some() {
        return scoped_env;
    }
    env.with_state(|state| {
        let globals = rilua::Val::Table(state.global);
        let addon_table =
            crate::lua_api::methods::table_get_static(state, globals, "__wow_loading_addon_table");
        Ok::<rilua::Val, crate::Error>(addon_table)
    })
    .ok()
    .filter(|value| !matches!(value, rilua::Val::Nil))
}

/// Prepend intrinsic base template to the inherits chain.
pub(super) fn build_inherits_chain(
    frame: &crate::xml::FrameXml,
    intrinsic_base: Option<&str>,
) -> Option<String> {
    let explicit = frame.inherits.as_deref().unwrap_or("");
    match intrinsic_base {
        Some(base) if !explicit.is_empty() => Some(format!("{}, {}", base, explicit)),
        Some(base) => Some(base.to_string()),
        None => None,
    }
}

/// Resolve the frame name, applying `$parent` substitution and generating anonymous names.
/// Returns `None` if the frame should be skipped (anonymous top-level frame).
fn resolve_frame_name(
    frame: &crate::xml::FrameXml,
    parent_for_name: Option<&str>,
    creator: Option<&str>,
) -> Option<String> {
    match &frame.name {
        Some(n) => {
            if let Some(parent_name) = parent_for_name {
                Some(n.replace("$parent", parent_name))
            } else {
                Some(n.clone())
            }
        }
        None => {
            if parent_for_name.is_some() {
                Some(format!("__{}_{}", creator.unwrap_or("anon"), rand_id()))
            } else {
                None // Anonymous top-level frames are templates
            }
        }
    }
}

fn resolve_subst_parent(
    frame: &crate::xml::FrameXml,
    resolved_name: &str,
    subst_parent_override: Option<&str>,
    parent_override: Option<&str>,
) -> String {
    if frame.name.is_some() {
        resolved_name.to_string()
    } else {
        frame
            .parent
            .as_deref()
            .or(subst_parent_override)
            .or(parent_override)
            .unwrap_or(resolved_name)
            .to_string()
    }
}

/// Resolve the parent for a frame, checking inherited templates when the frame
/// itself has no explicit `parent` attribute (e.g. ClassPowerBarFrame defines
/// `parent="PlayerFrame"` which propagates to PaladinPowerBarFrame).
///
/// Returns `Some(parent_name)` from the template chain, or `None` if no
/// template provides a parent.  The caller should prefer `parent_override`
/// and `frame.parent` first.
fn resolve_parent(frame: &crate::xml::FrameXml, parent_override: Option<&str>) -> Option<String> {
    if parent_override.is_some() || frame.parent.is_some() {
        return None; // Already have an explicit parent, no need to search templates.
    }
    frame.inherits.as_deref().and_then(|inherits| {
        crate::xml::get_template_chain(inherits)
            .iter()
            .find_map(|entry| entry.frame.parent.clone())
    })
}

fn resolve_xml_hidden(frame: &crate::xml::FrameXml, inherits: &str) -> bool {
    let mut hidden = frame.hidden;
    if hidden.is_none() && !inherits.is_empty() {
        for entry in &*crate::xml::get_template_chain(inherits) {
            if let Some(h) = entry.frame.hidden {
                hidden = Some(h);
                break;
            }
        }
    }
    hidden == Some(true)
}
