//! Frame-surface probes for `Blizzard_AddOnList`.

use crate::common::blizzard_addon_harness::{
    with_blizzard_addon_glue_smoke_shape, with_blizzard_addon_startup_shape,
};
use wow_ui_sim::lua_api::WowLuaEnv;

const ROOT: &str = "Blizzard_AddOnList";
const GLUE_PARENT_ROOT: &str = "Blizzard_GlueParent";
const ADDON_LIST_WIDTH: f32 = 600.0;
const ADDON_LIST_HEIGHT: f32 = 550.0;
const ADDON_LIST_CENTER_X: f32 = 0.0;
const ADDON_LIST_CENTER_Y: f32 = 24.0;
const ADDON_LIST_SURFACE_PROBE: &str = r#"
local expectedParent = _G[%q]
local point, _, relativePoint, xOfs, yOfs = AddonList:GetPoint(1)
return AddonList ~= nil,
       AddonList:GetObjectType(),
       AddonList:GetParent() == expectedParent,
       AddonList:IsShown(),
       AddonList:GetWidth(),
       AddonList:GetHeight(),
       AddonList:GetNumPoints(),
       point,
       relativePoint,
       xOfs,
       yOfs,
       AddonList.CloseButton ~= nil
           and AddonList.Inset ~= nil
           and AddonList.Bg ~= nil
"#;
const ADDON_LIST_PARENT_KEY_CHILDREN: &[ParentKeyChild] = &[
    ParentKeyChild::new("Dropdown", "DropdownButton", "Button"),
    ParentKeyChild::new("ForceLoad", "CheckButton", "CheckButton"),
    ParentKeyChild::new("SearchBox", "EditBox", "EditBox"),
    ParentKeyChild::new("Performance", "Frame", "Frame"),
    ParentKeyChild::new("CancelButton", "Button", "Button"),
    ParentKeyChild::new("OkayButton", "Button", "Button"),
    ParentKeyChild::new("EnableAllButton", "Button", "Button"),
    ParentKeyChild::new("DisableAllButton", "Button", "Button"),
    ParentKeyChild::new("ScrollBox", "Frame", "Frame"),
    ParentKeyChild::new("ScrollBar", "EventFrame", "EventFrame"),
];
const ADDON_DIALOG_CHILDREN: &[DialogChild] = &[
    DialogChild::new("AddonDialogText", "FontString", 0),
    DialogChild::new("AddonDialogButton1", "Button", 1),
    DialogChild::new("AddonDialogButton2", "Button", 2),
];
const ENTRY_TEMPLATE_CHILDREN: &[ParentKeyChild] = &[
    ParentKeyChild::new("Title", "FontString", "FontString"),
    ParentKeyChild::new("Status", "FontString", "FontString"),
    ParentKeyChild::new("Reload", "FontString", "FontString"),
    ParentKeyChild::new("Enabled", "CheckButton", "CheckButton"),
    ParentKeyChild::new("LoadAddonButton", "Button", "Button"),
];
const CATEGORY_TEMPLATE_CHILDREN: &[ParentKeyChild] = &[
    ParentKeyChild::new("Title", "FontString", "FontString"),
    ParentKeyChild::new("CollapseExpand", "Button", "Button"),
];
const ENTRY_TEMPLATE_PROBE: &str = "AddonListEntryTemplateProbe";
const CATEGORY_TEMPLATE_PROBE: &str = "AddonListCategoryTemplateProbe";
type AddonListSurfaceProbe = (
    bool,
    String,
    bool,
    bool,
    f32,
    f32,
    i32,
    String,
    String,
    f32,
    f32,
    bool,
);

prefork_full_ui_case! {
fn addon_list_frame_matches_game_xml_surface(env: &WowLuaEnv) {
        let surface = probe_addon_list_surface(env, "UIParent");

        assert_addon_list_surface(surface, "UIParent");
    }
}

#[test]
fn addon_list_frame_uses_glue_parent_in_glue() {
    with_blizzard_addon_glue_smoke_shape(&[GLUE_PARENT_ROOT, ROOT], &[], |env, _loaded| {
        let surface = probe_addon_list_surface(env, "GlueParent");

        assert_addon_list_surface(surface, "GlueParent");
    });
}

#[test]
fn addon_list_exposes_plan_parent_key_children() {
    with_blizzard_addon_startup_shape(&[ROOT], &[], |env, _loaded| {
        for child in ADDON_LIST_PARENT_KEY_CHILDREN {
            let surface = probe_parent_key_child(env, child.key);

            assert_parent_key_child_surface(child, surface);
        }
    });
}

#[test]
fn addon_dialog_frame_matches_xml_surface() {
    with_blizzard_addon_glue_smoke_shape(&[GLUE_PARENT_ROOT, ROOT], &[], |env, _loaded| {
        let surface = probe_addon_dialog_surface(env);

        assert_addon_dialog_surface(surface);

        for child in ADDON_DIALOG_CHILDREN {
            let surface = probe_dialog_child(env, child.name);

            assert_dialog_child_surface(child, surface);
        }
    });
}

prefork_full_ui_case! {
fn addon_list_virtual_templates_expose_parent_keys(env: &WowLuaEnv) {
        create_template_probe_frames(env);

        assert_template_button(env, ENTRY_TEMPLATE_PROBE);
        assert_template_children(env, ENTRY_TEMPLATE_PROBE, ENTRY_TEMPLATE_CHILDREN);
        assert_template_button(env, CATEGORY_TEMPLATE_PROBE);
        assert_template_children(env, CATEGORY_TEMPLATE_PROBE, CATEGORY_TEMPLATE_CHILDREN);
    }
}

struct ParentKeyChild {
    key: &'static str,
    xml_tag: &'static str,
    object_type: &'static str,
}

impl ParentKeyChild {
    const fn new(key: &'static str, xml_tag: &'static str, object_type: &'static str) -> Self {
        Self {
            key,
            xml_tag,
            object_type,
        }
    }
}

struct ParentKeyChildSurface {
    actual_type: String,
    parent_matches: bool,
}

struct DialogChild {
    name: &'static str,
    object_type: &'static str,
    id: i32,
}

impl DialogChild {
    const fn new(name: &'static str, object_type: &'static str, id: i32) -> Self {
        Self {
            name,
            object_type,
            id,
        }
    }
}

struct DialogChildSurface {
    object_type: String,
    parent_matches: bool,
    id: i32,
}

struct AddonDialogSurface {
    exists: bool,
    object_type: String,
    parent_is_glue_parent: bool,
    is_shown: bool,
    frame_strata: String,
    background_is_child: bool,
    background_has_dialog_bg: bool,
}

struct AddonListSurface {
    exists: bool,
    object_type: String,
    parent_matches: bool,
    is_shown: bool,
    width: f32,
    height: f32,
    point_count: i32,
    point: String,
    relative_point: String,
    x_offset: f32,
    y_offset: f32,
    has_button_frame_template_children: bool,
}

fn probe_parent_key_child(
    env: &wow_ui_sim::lua_api::WowLuaEnv,
    child_key: &str,
) -> ParentKeyChildSurface {
    let (actual_type, parent_is_addon_list): (String, bool) = env
        .eval(&format!(
            r#"
            local child = AddonList[{child_key:?}]
            return child and child:GetObjectType() or "nil",
                   child and child:GetParent() == AddonList or false
            "#
        ))
        .unwrap_or_else(|err| panic!("failed to probe `AddonList.{child_key}`: {err}"));

    ParentKeyChildSurface {
        actual_type,
        parent_matches: parent_is_addon_list,
    }
}

fn create_template_probe_frames(env: &wow_ui_sim::lua_api::WowLuaEnv) {
    env.exec(
        r#"
        AddonListEntryTemplateProbe = CreateFrame("Button", "AddonListEntryTemplateProbe", UIParent, "AddonListEntryTemplate")
        AddonListCategoryTemplateProbe = CreateFrame("Button", "AddonListCategoryTemplateProbe", UIParent, "AddonListCategoryTemplate")
        "#,
    )
    .expect("AddonList virtual template probe frames must be created");
}

fn assert_template_button(env: &wow_ui_sim::lua_api::WowLuaEnv, frame_name: &str) {
    let object_type: String = env
        .eval(&format!(
            r#"
            return _G[{frame_name:?}]:GetObjectType()
            "#
        ))
        .unwrap_or_else(|err| panic!("failed to probe `{frame_name}` object type: {err}"));

    assert_eq!(
        object_type, "Button",
        "`{frame_name}` must instantiate a virtual Button template"
    );
}

fn assert_template_children(
    env: &wow_ui_sim::lua_api::WowLuaEnv,
    frame_name: &str,
    children: &[ParentKeyChild],
) {
    for child in children {
        let surface = probe_template_child(env, frame_name, child.key);

        assert_template_child_surface(frame_name, child, surface);
    }
}

fn probe_template_child(
    env: &wow_ui_sim::lua_api::WowLuaEnv,
    frame_name: &str,
    child_key: &str,
) -> ParentKeyChildSurface {
    let (actual_type, parent_matches): (String, bool) = env
        .eval(&format!(
            r#"
            local frame = _G[{frame_name:?}]
            local child = frame and frame[{child_key:?}]
            return child and child:GetObjectType() or "nil",
                   child and child:GetParent() == frame or false
            "#
        ))
        .unwrap_or_else(|err| panic!("failed to probe `{frame_name}.{child_key}`: {err}"));

    ParentKeyChildSurface {
        actual_type,
        parent_matches,
    }
}

fn probe_addon_dialog_surface(env: &wow_ui_sim::lua_api::WowLuaEnv) -> AddonDialogSurface {
    let (
        exists,
        object_type,
        parent_is_glue_parent,
        is_shown,
        frame_strata,
        background_is_child,
        background_has_dialog_bg,
    ): (bool, String, bool, bool, String, bool, bool) = env
        .eval(
            r#"
            return AddonDialog ~= nil,
                   AddonDialog:GetObjectType(),
                   AddonDialog:GetParent() == GlueParent,
                   AddonDialog:IsShown(),
                   AddonDialog:GetFrameStrata(),
                   AddonDialogBackground:GetParent() == AddonDialog,
                   AddonDialogBackground.Bg ~= nil
            "#,
        )
        .expect("AddonDialog frame surface probe must run cleanly");

    AddonDialogSurface {
        exists,
        object_type,
        parent_is_glue_parent,
        is_shown,
        frame_strata,
        background_is_child,
        background_has_dialog_bg,
    }
}

fn probe_dialog_child(
    env: &wow_ui_sim::lua_api::WowLuaEnv,
    child_name: &str,
) -> DialogChildSurface {
    let (object_type, parent_matches, id): (String, bool, i32) = env
        .eval(&format!(
            r#"
            local child = _G[{child_name:?}]
            return child and child:GetObjectType() or "nil",
                   child and child:GetParent() == AddonDialogBackground or false,
                   child and child:GetID() or 0
            "#
        ))
        .unwrap_or_else(|err| panic!("failed to probe `{child_name}`: {err}"));

    DialogChildSurface {
        object_type,
        parent_matches,
        id,
    }
}

fn probe_addon_list_surface(
    env: &wow_ui_sim::lua_api::WowLuaEnv,
    expected_parent_name: &str,
) -> AddonListSurface {
    let probe = ADDON_LIST_SURFACE_PROBE.replace("%q", &format!("{expected_parent_name:?}"));
    let raw_surface = env
        .eval::<AddonListSurfaceProbe>(&probe)
        .expect("AddonList frame surface probe must run cleanly");

    AddonListSurface::from(raw_surface)
}

impl From<AddonListSurfaceProbe> for AddonListSurface {
    fn from(raw_surface: AddonListSurfaceProbe) -> Self {
        let (
            exists,
            object_type,
            parent_matches,
            is_shown,
            width,
            height,
            point_count,
            point,
            relative_point,
            x_offset,
            y_offset,
            has_button_frame_template_children,
        ) = raw_surface;

        AddonListSurface {
            exists,
            object_type,
            parent_matches,
            is_shown,
            width,
            height,
            point_count,
            point,
            relative_point,
            x_offset,
            y_offset,
            has_button_frame_template_children,
        }
    }
}

fn assert_addon_list_surface(surface: AddonListSurface, expected_parent_name: &str) {
    assert_frame_shape(&surface, expected_parent_name);
    assert_frame_size(&surface);
    assert_frame_anchor(&surface);
    assert_button_frame_template_children(&surface);
}

fn assert_frame_shape(surface: &AddonListSurface, expected_parent_name: &str) {
    assert!(
        surface.exists,
        "`AddonList` must exist after `{ROOT}` loads"
    );
    assert_eq!(
        surface.object_type, "Frame",
        "`AddonList` XML declares a Frame"
    );
    assert!(
        surface.parent_matches,
        "`AddonList` must be parented to `{expected_parent_name}` in this screen branch"
    );
    assert!(
        !surface.is_shown,
        "`AddonList` XML declares hidden=\"true\""
    );
}

fn assert_frame_size(surface: &AddonListSurface) {
    assert_eq!(
        surface.width, ADDON_LIST_WIDTH,
        "`AddonList` XML width must be 600"
    );
    assert_eq!(
        surface.height, ADDON_LIST_HEIGHT,
        "`AddonList` XML height must be 550"
    );
}

fn assert_frame_anchor(surface: &AddonListSurface) {
    assert_eq!(
        surface.point_count, 1,
        "`AddonList` XML declares exactly one anchor"
    );
    assert_eq!(
        surface.point, "CENTER",
        "`AddonList` anchor point must be CENTER"
    );
    assert_eq!(
        surface.relative_point, "CENTER",
        "`AddonList` relative anchor point must be CENTER"
    );
    assert_eq!(
        surface.x_offset, ADDON_LIST_CENTER_X,
        "`AddonList` x offset must be 0"
    );
    assert_eq!(
        surface.y_offset, ADDON_LIST_CENTER_Y,
        "`AddonList` y offset must be 24"
    );
}

fn assert_button_frame_template_children(surface: &AddonListSurface) {
    assert!(
        surface.has_button_frame_template_children,
        "`AddonList` must inherit concrete children from ButtonFrameTemplate"
    );
}

fn assert_parent_key_child_surface(child: &ParentKeyChild, surface: ParentKeyChildSurface) {
    assert_eq!(
        surface.actual_type, child.object_type,
        "`AddonList.{}` must expose the XML `{}` parentKey child as a runtime {}",
        child.key, child.xml_tag, child.object_type
    );
    assert!(
        surface.parent_matches,
        "`AddonList.{}` must be parented to `AddonList`",
        child.key
    );
}

fn assert_template_child_surface(
    frame_name: &str,
    child: &ParentKeyChild,
    surface: ParentKeyChildSurface,
) {
    assert_eq!(
        surface.actual_type, child.object_type,
        "`{frame_name}.{}` must expose the XML `{}` parentKey child as a runtime {}",
        child.key, child.xml_tag, child.object_type
    );
    assert!(
        surface.parent_matches,
        "`{frame_name}.{}` must be parented to `{frame_name}`",
        child.key
    );
}

fn assert_addon_dialog_surface(surface: AddonDialogSurface) {
    assert!(
        surface.exists,
        "`AddonDialog` must exist after `{ROOT}` loads"
    );
    assert_eq!(
        surface.object_type, "Frame",
        "`AddonDialog` XML declares a Frame"
    );
    assert!(
        surface.parent_is_glue_parent,
        "`AddonDialog` must be parented to `GlueParent` in the glue branch"
    );
    assert!(
        !surface.is_shown,
        "`AddonDialog` XML declares hidden=\"true\""
    );
    assert_eq!(
        surface.frame_strata, "DIALOG",
        "`AddonDialog` XML declares DIALOG frame strata"
    );
    assert!(
        surface.background_is_child,
        "`AddonDialogBackground` must be parented to `AddonDialog`"
    );
    assert!(
        surface.background_has_dialog_bg,
        "`AddonDialogBackground` must inherit `DialogBorderTemplate` background surface"
    );
}

fn assert_dialog_child_surface(child: &DialogChild, surface: DialogChildSurface) {
    assert_eq!(
        surface.object_type, child.object_type,
        "`{}` must be a runtime {}",
        child.name, child.object_type
    );
    assert!(
        surface.parent_matches,
        "`{}` must be parented to `AddonDialogBackground`",
        child.name
    );
    assert_eq!(
        surface.id, child.id,
        "`{}` must preserve the XML id",
        child.name
    );
}
