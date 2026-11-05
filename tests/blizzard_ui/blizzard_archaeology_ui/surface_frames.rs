//! Frame-surface probes for `Blizzard_ArchaeologyUI`.

use crate::common::blizzard_addon_harness::with_blizzard_addon_startup_shape;

const ROOT: &str = "Blizzard_ArchaeologyUI";
const PARENT_KEY_CHILDREN: &[&str] = &[
    "summaryPage",
    "completedPage",
    "artifactPage",
    "helpPage",
    "RaceFilterDropdown",
    "bgLeft",
    "bgRight",
    "factionIcon",
    "tab1",
    "tab2",
    "infoButton",
];
const RACE_TEMPLATE_CHILDREN: &[&str] = &["raceName", "glow", "readyAnim"];
const ARTIFACT_TEMPLATE_CHILDREN: &[&str] = &["border", "icon", "artifactName", "artifactSubText"];
const KEYSTONE_TEMPLATE_CHILDREN: &[&str] = &["icon"];

#[cfg(feature = "optional-tests")]
#[test]
fn archaeology_frame_matches_xml_surface() {
    with_blizzard_addon_startup_shape(&[ROOT], &[], |env, _loaded| {
        let surface: ArchaeologyFrameSurface = env
            .eval(
                r#"
                return type(ArchaeologyFrame),
                       ArchaeologyFrame:GetObjectType(),
                       ArchaeologyFrame:GetParent() == UIParent,
                       ArchaeologyFrame.CloseButton ~= nil,
                       ArchaeologyFrame.Bg ~= nil,
                       ArchaeologyFrame.Inset ~= nil
                "#,
            )
            .expect("ArchaeologyFrame surface probe must run cleanly");

        assert_archaeology_frame_surface(surface);

        for child_name in PARENT_KEY_CHILDREN {
            let child_exists = archaeology_frame_child_exists(env, child_name);

            assert!(
                child_exists,
                "`ArchaeologyFrame.{child_name}` must be exposed as an XML parentKey child"
            );
        }
    });
}

#[cfg(feature = "optional-tests")]
#[test]
fn archaeology_frame_instantiates_template_slots() {
    with_blizzard_addon_startup_shape(&[ROOT], &[], |env, _loaded| {
        assert_template_slot_range(
            env,
            SlotRange {
                page_path: "ArchaeologyFrame.summaryPage",
                global_prefix: "ArchaeologyFrameSummaryPageRace",
                parent_key_prefix: "race",
                count: 12,
                inherited_children: RACE_TEMPLATE_CHILDREN,
            },
        );
        assert_template_slot_range(
            env,
            SlotRange {
                page_path: "ArchaeologyFrame.completedPage",
                global_prefix: "ArchaeologyFrameCompletedPageArtifact",
                parent_key_prefix: "artifact",
                count: 12,
                inherited_children: ARTIFACT_TEMPLATE_CHILDREN,
            },
        );
        assert_template_slot_range(
            env,
            SlotRange {
                page_path: "ArchaeologyFrame.artifactPage.solveFrame",
                global_prefix: "ArchaeologyFrameArtifactPageSolveFrameKeystone",
                parent_key_prefix: "keystone",
                count: 4,
                inherited_children: KEYSTONE_TEMPLATE_CHILDREN,
            },
        );
    });
}

struct SlotRange<'a> {
    page_path: &'a str,
    global_prefix: &'a str,
    parent_key_prefix: &'a str,
    count: i32,
    inherited_children: &'a [&'a str],
}

type ArchaeologyFrameSurface = (String, String, bool, bool, bool, bool);
type TemplateSlotSurface = (bool, String, bool, bool, bool);

fn assert_archaeology_frame_surface(surface: ArchaeologyFrameSurface) {
    let (frame_type, object_type, parent_is_ui_parent, has_close_button, has_bg, has_inset) =
        surface;

    assert_eq!(frame_type, "table", "`ArchaeologyFrame` must exist");
    assert_eq!(
        object_type, "Frame",
        "`ArchaeologyFrame` must be a Frame object"
    );
    assert!(
        parent_is_ui_parent,
        "`ArchaeologyFrame` must be parented to `UIParent`"
    );
    assert!(
        has_close_button && has_bg && has_inset,
        "`ArchaeologyFrame` must inherit `ButtonFrameTemplate` frame parts"
    );
}

fn archaeology_frame_child_exists(env: &wow_ui_sim::lua_api::WowLuaEnv, child_name: &str) -> bool {
    env.eval(&format!("return ArchaeologyFrame[{child_name:?}] ~= nil"))
        .unwrap_or_else(|err| {
            panic!("failed to probe ArchaeologyFrame parentKey child `{child_name}`: {err}")
        })
}

fn assert_template_slot_range(env: &wow_ui_sim::lua_api::WowLuaEnv, slot_range: SlotRange<'_>) {
    for index in 1..=slot_range.count {
        let surface = probe_template_slot(env, &slot_range, index);

        assert_template_slot_surface(&slot_range, index, surface);
    }
}

fn probe_template_slot(
    env: &wow_ui_sim::lua_api::WowLuaEnv,
    slot_range: &SlotRange<'_>,
    index: i32,
) -> TemplateSlotSurface {
    let global_name = format!("{}{}", slot_range.global_prefix, index);
    let parent_key = format!("{}{}", slot_range.parent_key_prefix, index);
    let inherited_children = inherited_child_check_expression(slot_range.inherited_children);
    let probe = format!(
        r#"
        local page = {page_path}
        local slot = _G[{global_name:?}]
        return slot ~= nil,
               slot and slot:GetObjectType() or "nil",
               slot and slot:GetParent() == page or false,
               page and page[{parent_key:?}] == slot or false,
               slot and ({inherited_children}) or false
        "#,
        page_path = slot_range.page_path,
        global_name = global_name,
        parent_key = parent_key,
        inherited_children = inherited_children,
    );

    env.eval(&probe)
        .unwrap_or_else(|err| panic!("failed to probe template slot `{global_name}`: {err}"))
}

fn inherited_child_check_expression(child_names: &[&str]) -> String {
    child_names
        .iter()
        .map(|child_name| format!("slot[{child_name:?}] ~= nil"))
        .collect::<Vec<_>>()
        .join(" and ")
}

fn assert_template_slot_surface(
    slot_range: &SlotRange<'_>,
    index: i32,
    surface: TemplateSlotSurface,
) {
    let (exists, object_type, parent_matches, parent_key_matches, has_inherited_children) = surface;
    let global_name = format!("{}{}", slot_range.global_prefix, index);
    let parent_key = format!("{}{}", slot_range.parent_key_prefix, index);

    assert!(exists, "`{global_name}` must exist");
    assert_eq!(object_type, "Button", "`{global_name}` must be a Button");
    assert!(
        parent_matches,
        "`{global_name}` must be parented to `{}`",
        slot_range.page_path
    );
    assert!(
        parent_key_matches,
        "`{}` must expose `{global_name}` through parentKey `{parent_key}`",
        slot_range.page_path
    );
    assert!(
        has_inherited_children,
        "`{global_name}` must expose parentKey children inherited from its XML template"
    );
}
