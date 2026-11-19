//! Frame creation from XML definitions.

use std::time::Instant;

use crate::lua_api::LoaderEnv;

use crate::loader::LoadTiming;
use crate::loader::error::LoadError;
use crate::loader::helpers::lua_frame_ref_by_id;
use crate::loader::xml_frame_codegen::build_frame_lua_code;

use finalize::finalize_frame;
use preparation::{PreparedFrameCreation, prepare_frame_creation, register_virtual_or_intrinsic};
use setup::{SetupFrame, setup_frame};

#[cfg(test)]
pub(crate) use finalize::frame_element_to_type;

#[path = "xml_frame/finalize.rs"]
mod finalize;
#[path = "xml_frame/preparation.rs"]
mod preparation;
#[path = "xml_frame/secure_mixin.rs"]
mod secure_mixin;
#[path = "xml_frame/setup.rs"]
mod setup;

#[cfg(test)]
#[path = "xml_frame/tests.rs"]
mod tests;

/// Resolve a cached intrinsic declaration for tags not already mapped as intrinsics.
pub(super) fn registered_intrinsic_tag(tag: &'static str) -> Option<&'static str> {
    crate::xml::get_template(tag)
        .filter(|template| template.frame.intrinsic == Some(true))
        .map(|_| tag)
}

/// Create a frame from XML and return its widget ID, or None if skipped.
///
/// `intrinsic_base` is set when the XML element is an intrinsic type (e.g.
/// `<ContainedAlertFrame>`) whose registered template should be implicitly
/// inherited before any explicit `inherits` attribute.
pub fn create_frame_from_xml(
    env: &LoaderEnv<'_>,
    frame: &crate::xml::FrameXml,
    widget_type: &str,
    parent_override: Option<&str>,
    parent_id_override: Option<u64>,
    subst_parent_override: Option<&str>,
    intrinsic_base: Option<&str>,
    timing: &mut LoadTiming,
) -> Result<Option<u64>, LoadError> {
    let _profile = crate::loader::chunk_cache::GeneratedChunkScope::enter("xml-frame");
    if register_virtual_or_intrinsic(env, frame, widget_type, parent_override, intrinsic_base)
        .is_some()
    {
        return Ok(None);
    }
    let Some(prepared) = prepare_frame_creation(
        env,
        frame,
        parent_override,
        subst_parent_override,
        intrinsic_base,
    ) else {
        return Ok(None);
    };
    let frame_id = create_prepared_frame_from_xml(
        env,
        frame,
        widget_type,
        parent_override,
        parent_id_override,
        &prepared,
        intrinsic_base,
        timing,
    )?;
    Ok(Some(frame_id))
}

fn create_prepared_frame_from_xml(
    env: &LoaderEnv<'_>,
    frame: &crate::xml::FrameXml,
    widget_type: &str,
    parent_override: Option<&str>,
    parent_id_override: Option<u64>,
    prepared: &PreparedFrameCreation,
    intrinsic_base: Option<&str>,
    timing: &mut LoadTiming,
) -> Result<u64, LoadError> {
    let parent_id = resolve_xml_parent_id(
        env,
        frame,
        parent_override,
        parent_id_override,
        &prepared.parent,
    )?;
    let frame_id = build_and_setup_frame(
        env,
        timing,
        frame,
        widget_type,
        prepared,
        parent_id,
        intrinsic_base,
    )?;
    finalize_frame(
        env,
        frame,
        frame_id,
        &prepared.name,
        &prepared.subst_parent,
        &prepared.inherits,
        timing,
    )?;
    Ok(frame_id)
}

fn resolve_xml_parent_id(
    env: &LoaderEnv<'_>,
    frame: &crate::xml::FrameXml,
    parent_override: Option<&str>,
    parent_id_override: Option<u64>,
    parent_name: &str,
) -> Result<u64, LoadError> {
    let inherited_parent_id = if frame.parent.is_none() && parent_override.is_some() {
        parent_id_override
    } else {
        None
    };
    let named_parent_id = env.state().borrow().widgets.get_id_by_name(parent_name);
    inherited_parent_id
        .or(named_parent_id)
        .ok_or_else(|| LoadError::Lua(format!("Failed to locate XML parent frame {parent_name}")))
}

pub fn fast_create_frame_profile_report() -> Option<String> {
    setup::fast_create_frame_profile_report()
}

pub fn fast_create_frame_profile_body_report() -> Option<String> {
    setup::fast_create_frame_profile_body_report()
}

/// Build the CreateFrame Lua code and run the setup phase.
fn build_and_setup_frame(
    env: &LoaderEnv<'_>,
    timing: &mut LoadTiming,
    frame: &crate::xml::FrameXml,
    widget_type: &str,
    prepared: &PreparedFrameCreation,
    parent_id: u64,
    intrinsic_base: Option<&str>,
) -> Result<u64, LoadError> {
    let build_start = Instant::now();
    let lua_code = build_prepared_frame_lua(frame, widget_type, prepared, parent_id);
    timing.frame_code_build_time += build_start.elapsed();

    setup_frame(
        env,
        timing,
        SetupFrame {
            widget_type,
            lua_code: &lua_code,
            name: &prepared.name,
            explicit_parent: prepared.explicit_parent.is_some(),
            initial_hidden: prepared.initial_hidden,
            frame,
            inherits: &prepared.inherits,
            parent: &prepared.parent,
            parent_id,
            intrinsic_base,
        },
    )
}

fn build_prepared_frame_lua(
    frame: &crate::xml::FrameXml,
    widget_type: &str,
    prepared: &PreparedFrameCreation,
    parent_id: u64,
) -> String {
    let parent_ref_expr = lua_frame_ref_by_id(parent_id);
    build_frame_lua_code(
        widget_type,
        &prepared.name,
        prepared.explicit_parent.as_deref(),
        &prepared.inherits,
        frame,
        &prepared.parent,
        &parent_ref_expr,
    )
}
