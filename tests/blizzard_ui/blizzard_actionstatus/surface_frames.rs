//! Frame-surface probes for `Blizzard_ActionStatus`.

use crate::common::blizzard_addon_harness::with_blizzard_addon_smoke_shape;

const ROOT: &str = "Blizzard_ActionStatus";
const ACTION_STATUS_MIXIN_METHODS: &[&str] = &[
    "OnLoad",
    "OnEvent",
    "OnUpdate",
    "SetAlternateParentFrame",
    "ClearAlternateParentFrame",
    "DisplayMessage",
    "GetBestParent",
    "UpdateParent",
];
const ACTION_STATUS_SCRIPT_HANDLERS: &[&str] = &["OnLoad", "OnEvent", "OnUpdate"];
const ACTION_STATUS_TEXT_SURFACE_PROBE: &str = r#"
local point, _, relativePoint, xOfs, yOfs = ActionStatus.Text:GetPoint(1)
local fontPath, fontHeight, fontFlags = ActionStatus.Text:GetFont()
local templatePath, templateHeight, templateFlags = GameFontNormalLarge:GetFont()
return ActionStatus.Text:GetObjectType(),
       ActionStatus.Text:GetParent() == ActionStatus,
       point,
       relativePoint,
       xOfs,
       yOfs,
       fontPath == templatePath
           and fontHeight == templateHeight
           and fontFlags == templateFlags
"#;

struct ActionStatusTextSurface {
    object_type: String,
    parent_is_action_status: bool,
    point: String,
    relative_point: String,
    x_offset: f32,
    y_offset: f32,
    font_matches_inherited_template: bool,
}

#[cfg(feature = "optional-tests")]
#[test]
fn action_status_frame_surface_matches_xml_contract() {
    with_blizzard_addon_smoke_shape(&[ROOT], &[], |env, _loaded| {
        assert_action_status_frame_shape(env);
        assert_action_status_mixin_methods_are_bound(env);
        assert_action_status_script_handlers_are_bound(env);
        assert_action_status_text_surface(env);
    });
}

fn assert_action_status_frame_shape(env: &wow_ui_sim::lua_api::WowLuaEnv) {
    let (exists, object_type, frame_strata, is_shown) = env
        .eval::<(bool, String, String, bool)>(
            r#"
            return ActionStatus ~= nil,
                   ActionStatus:GetObjectType(),
                   ActionStatus:GetFrameStrata(),
                   ActionStatus:IsShown()
            "#,
        )
        .expect("ActionStatus frame shape probe must run cleanly");

    assert!(exists, "`ActionStatus` must exist after `{ROOT}` loads");
    assert_eq!(object_type, "Frame", "`ActionStatus` XML declares a Frame");
    assert_eq!(
        frame_strata, "TOOLTIP",
        "`ActionStatus` XML declares frameStrata=\"TOOLTIP\""
    );
    assert!(!is_shown, "`ActionStatus` XML declares hidden=\"true\"");
}

fn assert_action_status_mixin_methods_are_bound(env: &wow_ui_sim::lua_api::WowLuaEnv) {
    for method_name in ACTION_STATUS_MIXIN_METHODS {
        let is_function: bool = env
            .eval(&format!(
                r#"return type(ActionStatus["{method_name}"]) == "function""#
            ))
            .unwrap_or_else(|err| panic!("ActionStatus.{method_name} probe failed: {err}"));

        assert!(
            is_function,
            "`ActionStatus.{method_name}` must be bound from ActionStatusMixin"
        );
    }
}

fn assert_action_status_script_handlers_are_bound(env: &wow_ui_sim::lua_api::WowLuaEnv) {
    for script_name in ACTION_STATUS_SCRIPT_HANDLERS {
        let is_function: bool = env
            .eval(&format!(
                r#"return type(ActionStatus:GetScript("{script_name}")) == "function""#
            ))
            .unwrap_or_else(|err| panic!("ActionStatus {script_name} script probe failed: {err}"));

        assert!(
            is_function,
            "`ActionStatus` must bind its `{script_name}` script from XML"
        );
    }
}

fn assert_action_status_text_surface(env: &wow_ui_sim::lua_api::WowLuaEnv) {
    let surface = probe_action_status_text_surface(env);

    assert_eq!(
        surface.object_type, "FontString",
        "`ActionStatus.Text` must be the parentKey FontString declared by XML"
    );
    assert!(
        surface.parent_is_action_status,
        "`ActionStatus.Text` must be parented to `ActionStatus`"
    );
    assert_eq!(
        surface.point, "CENTER",
        "`ActionStatus.Text` anchor point must be CENTER"
    );
    assert_eq!(
        surface.relative_point, "CENTER",
        "`ActionStatus.Text` relative anchor point must be CENTER"
    );
    assert_eq!(
        surface.x_offset, 0.0,
        "`ActionStatus.Text` x offset must be 0"
    );
    assert_eq!(
        surface.y_offset, 0.0,
        "`ActionStatus.Text` y offset must be 0"
    );
    assert!(
        surface.font_matches_inherited_template,
        "`ActionStatus.Text` must inherit GameFontNormalLarge's font path, height, and flags"
    );
}

fn probe_action_status_text_surface(
    env: &wow_ui_sim::lua_api::WowLuaEnv,
) -> ActionStatusTextSurface {
    let (
        object_type,
        parent_is_action_status,
        point,
        relative_point,
        x_offset,
        y_offset,
        font_matches_inherited_template,
    ) = env
        .eval::<(String, bool, String, String, f32, f32, bool)>(ACTION_STATUS_TEXT_SURFACE_PROBE)
        .expect("ActionStatus.Text surface probe must run cleanly");

    ActionStatusTextSurface {
        object_type,
        parent_is_action_status,
        point,
        relative_point,
        x_offset,
        y_offset,
        font_matches_inherited_template,
    }
}
