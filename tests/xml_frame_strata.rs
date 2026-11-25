//! Tests for XML frameStrata and frameLevel attribute parsing.

use wow_ui_sim::loader::{LoadTiming, create_frame_from_xml};
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::xml::{XmlElement, clear_templates, parse_xml};

const XML_FIXED_FRAME_LEVEL_PROBE: &str = r#"
<Ui>
    <Frame name="XmlFixedParent" parent="UIParent" frameLevel="50">
        <Size x="10" y="10"/>
    </Frame>
    <Frame name="XmlFixedChild" parent="XmlFixedParent" frameLevel="10" fixedFrameLevel="true">
        <Size x="10" y="10"/>
    </Frame>
</Ui>
"#;

#[test]
fn test_create_frame_from_xml_frame_strata() {
    clear_templates();
    let env = WowLuaEnv::new().unwrap();

    let xml = r#"
        <Ui>
            <Frame name="DialogStrataFrame" parent="UIParent" frameStrata="DIALOG">
                <Size x="200" y="100"/>
            </Frame>
        </Ui>
    "#;

    let ui = parse_xml(xml).unwrap();
    if let XmlElement::Frame(frame) = &ui.elements[0] {
        create_frame_from_xml(
            &env.loader_env(),
            frame,
            "Frame",
            None,
            None,
            None,
            &mut LoadTiming::default(),
        )
        .unwrap();
    }

    let (strata, fixed): (String, bool) = env
        .eval(
            "return DialogStrataFrame:GetFrameStrata(), \
             DialogStrataFrame:HasFixedFrameStrata()",
        )
        .unwrap();
    assert_eq!(strata, "DIALOG");
    assert!(!fixed, "XML literal frameStrata remains non-fixed");

    // Children should inherit the parent's strata
    let child_strata: String = env
        .eval(
            r#"
            local child = CreateFrame("Frame", "DialogChild", DialogStrataFrame)
            return child:GetFrameStrata()
            "#,
        )
        .unwrap();
    assert_eq!(child_strata, "DIALOG");
}

#[test]
fn test_xml_protected_frame_retains_protection_and_lacks_legacy_setters() {
    clear_templates();
    let env = WowLuaEnv::new().unwrap();

    let xml = r#"
        <Ui>
            <Frame name="XmlProtectedProbeFrame" parent="UIParent" protected="true" hidden="true" />
        </Ui>
    "#;

    let ui = parse_xml(xml).unwrap();
    if let XmlElement::Frame(frame) = &ui.elements[0] {
        create_frame_from_xml(
            &env.loader_env(),
            frame,
            "Frame",
            None,
            None,
            None,
            &mut LoadTiming::default(),
        )
        .unwrap();
    }

    let (
        protected_before,
        forbidden,
        protect_missing,
        set_protected_missing,
        protect_call_ok,
        set_protected_true_ok,
        set_protected_false_ok,
        protected_after,
    ): (bool, bool, bool, bool, bool, bool, bool, bool) = env
        .eval(
            r#"
            local frame = XmlProtectedProbeFrame
            local protectedBefore = frame:IsProtected()
            local forbidden = frame:IsForbidden()
            local protectMissing = type(frame.Protect) == "nil"
            local setProtectedMissing = type(frame.SetProtected) == "nil"
            local protectCallOk = pcall(function() frame:Protect() end)
            local setProtectedTrueOk = pcall(function() frame:SetProtected(true) end)
            local setProtectedFalseOk = pcall(function() frame:SetProtected(false) end)
            return protectedBefore, forbidden, protectMissing, setProtectedMissing,
                protectCallOk, setProtectedTrueOk, setProtectedFalseOk, frame:IsProtected()
            "#,
        )
        .unwrap();

    assert!(protected_before);
    assert!(!forbidden);
    assert!(protect_missing);
    assert!(set_protected_missing);
    assert!(!protect_call_ok);
    assert!(!set_protected_true_ok);
    assert!(!set_protected_false_ok);
    assert!(protected_after);
}

#[test]
fn test_xml_parent_frame_strata_matches_dialog_parent_and_later_change() {
    clear_templates();
    let env = WowLuaEnv::new().unwrap();

    let xml = r#"
        <Ui>
            <Frame name="ParentStrataHost" parent="UIParent" frameStrata="DIALOG">
                <Frames>
                    <Frame name="ParentStrataChild" frameStrata="PARENT"/>
                </Frames>
            </Frame>
        </Ui>
    "#;

    let ui = parse_xml(xml).unwrap();
    if let XmlElement::Frame(frame) = &ui.elements[0] {
        create_frame_from_xml(
            &env.loader_env(),
            frame,
            "Frame",
            None,
            None,
            None,
            &mut LoadTiming::default(),
        )
        .unwrap();
    }

    let (initial, updated, fixed): (String, String, bool) = env
        .eval(
            r#"
            local initial = ParentStrataChild:GetFrameStrata()
            ParentStrataHost:SetFrameStrata("LOW")
            return initial, ParentStrataChild:GetFrameStrata(), ParentStrataChild:HasFixedFrameStrata()
            "#,
        )
        .unwrap();
    assert_eq!(initial, "DIALOG");
    assert_eq!(updated, "LOW");
    assert!(!fixed);
}

#[test]
fn test_base_template_literal_precedes_derived_parent_frame_strata() {
    clear_templates();
    let env = WowLuaEnv::new().unwrap();

    let templates = parse_xml(
        r#"
        <Ui>
            <Frame name="BaseHighStrataTemplate" virtual="true" frameStrata="HIGH"/>
            <Frame name="DerivedParentStrataTemplate" virtual="true"
                   inherits="BaseHighStrataTemplate" frameStrata="PARENT"/>
        </Ui>
        "#,
    )
    .unwrap();
    for element in &templates.elements {
        if let XmlElement::Frame(frame) = element {
            create_frame_from_xml(
                &env.loader_env(),
                frame,
                "Frame",
                None,
                None,
                None,
                &mut LoadTiming::default(),
            )
            .unwrap();
        }
    }

    let instances = parse_xml(
        r#"
        <Ui>
            <Frame name="DerivedParentStrataHost" parent="UIParent" frameStrata="DIALOG"/>
            <Frame name="DerivedParentStrataChild" parent="DerivedParentStrataHost"
                   inherits="DerivedParentStrataTemplate"/>
        </Ui>
        "#,
    )
    .unwrap();
    for element in &instances.elements {
        if let XmlElement::Frame(frame) = element {
            create_frame_from_xml(
                &env.loader_env(),
                frame,
                "Frame",
                None,
                None,
                None,
                &mut LoadTiming::default(),
            )
            .unwrap();
        }
    }

    let (strata, fixed): (String, bool) = env
        .eval(
            "return DerivedParentStrataChild:GetFrameStrata(), \
             DerivedParentStrataChild:HasFixedFrameStrata()",
        )
        .unwrap();
    assert_eq!(strata, "HIGH");
    assert!(!fixed);
}

#[test]
fn test_derived_template_literal_overrides_base_frame_strata() {
    clear_templates();
    let env = WowLuaEnv::new().unwrap();

    let templates = parse_xml(
        r#"
        <Ui>
            <Frame name="BaseHighStrataTemplateForLiteral" virtual="true" frameStrata="HIGH"/>
            <Frame name="DerivedLowStrataTemplate" virtual="true"
                   inherits="BaseHighStrataTemplateForLiteral" frameStrata="LOW"/>
        </Ui>
        "#,
    )
    .unwrap();
    for element in &templates.elements {
        if let XmlElement::Frame(frame) = element {
            create_frame_from_xml(
                &env.loader_env(),
                frame,
                "Frame",
                None,
                None,
                None,
                &mut LoadTiming::default(),
            )
            .unwrap();
        }
    }

    let instances = parse_xml(
        r#"
        <Ui>
            <Frame name="DerivedLowStrataHost" parent="UIParent" frameStrata="DIALOG"/>
            <Frame name="DerivedLowStrataChild" parent="DerivedLowStrataHost"
                   inherits="DerivedLowStrataTemplate"/>
        </Ui>
        "#,
    )
    .unwrap();
    for element in &instances.elements {
        if let XmlElement::Frame(frame) = element {
            create_frame_from_xml(
                &env.loader_env(),
                frame,
                "Frame",
                None,
                None,
                None,
                &mut LoadTiming::default(),
            )
            .unwrap();
        }
    }

    let (strata, fixed): (String, bool) = env
        .eval(
            "return DerivedLowStrataChild:GetFrameStrata(), \
             DerivedLowStrataChild:HasFixedFrameStrata()",
        )
        .unwrap();
    assert_eq!(strata, "LOW");
    assert!(!fixed);
}

#[test]
fn test_ignored_template_strata_does_not_erase_base_literal() {
    clear_templates();
    let env = WowLuaEnv::new().unwrap();

    let templates = parse_xml(
        r#"
        <Ui>
            <Frame name="BaseHighStrataTemplateForIgnored" virtual="true" frameStrata="HIGH"/>
            <Frame name="DerivedIgnoredStrataTemplate" virtual="true"
                   inherits="BaseHighStrataTemplateForIgnored" frameStrata="BLIZZARD"/>
        </Ui>
        "#,
    )
    .unwrap();
    for element in &templates.elements {
        if let XmlElement::Frame(frame) = element {
            create_frame_from_xml(
                &env.loader_env(),
                frame,
                "Frame",
                None,
                None,
                None,
                &mut LoadTiming::default(),
            )
            .unwrap();
        }
    }

    let instances = parse_xml(
        r#"
        <Ui>
            <Frame name="DerivedIgnoredStrataHost" parent="UIParent" frameStrata="DIALOG"/>
            <Frame name="DerivedIgnoredStrataChild" parent="DerivedIgnoredStrataHost"
                   inherits="DerivedIgnoredStrataTemplate"/>
        </Ui>
        "#,
    )
    .unwrap();
    for element in &instances.elements {
        if let XmlElement::Frame(frame) = element {
            create_frame_from_xml(
                &env.loader_env(),
                frame,
                "Frame",
                None,
                None,
                None,
                &mut LoadTiming::default(),
            )
            .unwrap();
        }
    }

    let strata: String = env
        .eval("return DerivedIgnoredStrataChild:GetFrameStrata()")
        .unwrap();
    assert_eq!(strata, "HIGH");
}

#[test]
fn test_xml_parent_frame_strata_overrides_fixed_widget_default() {
    clear_templates();
    let env = WowLuaEnv::new().unwrap();

    let host_xml = parse_xml(
        r#"
        <Ui>
            <Frame name="ParentStrataTooltipHost" parent="UIParent" frameStrata="HIGH"/>
        </Ui>
        "#,
    )
    .unwrap();
    if let XmlElement::Frame(frame) = &host_xml.elements[0] {
        create_frame_from_xml(
            &env.loader_env(),
            frame,
            "Frame",
            None,
            None,
            None,
            &mut LoadTiming::default(),
        )
        .unwrap();
    }

    let tooltip_xml = parse_xml(
        r#"
        <Ui>
            <Frame name="ParentStrataTooltip" parent="ParentStrataTooltipHost" frameStrata="PARENT"/>
        </Ui>
        "#,
    )
    .unwrap();
    if let XmlElement::Frame(frame) = &tooltip_xml.elements[0] {
        create_frame_from_xml(
            &env.loader_env(),
            frame,
            "GameTooltip",
            None,
            None,
            None,
            &mut LoadTiming::default(),
        )
        .unwrap();
    }

    let (initial, updated, fixed): (String, String, bool) = env
        .eval(
            r#"
            local initial = ParentStrataTooltip:GetFrameStrata()
            ParentStrataTooltipHost:SetFrameStrata("DIALOG")
            return initial, ParentStrataTooltip:GetFrameStrata(), ParentStrataTooltip:HasFixedFrameStrata()
            "#,
        )
        .unwrap();
    assert_eq!(initial, "HIGH");
    assert_eq!(updated, "DIALOG");
    assert!(!fixed);
}

#[test]
fn test_xml_blizzard_frame_strata_is_ignored() {
    clear_templates();
    let env = WowLuaEnv::new().unwrap();

    let xml = r#"
        <Ui>
            <Frame name="BlizzardStrataHost" parent="UIParent" frameStrata="HIGH">
                <Frames>
                    <Frame name="BlizzardStrataChild" frameStrata="BLIZZARD"/>
                </Frames>
            </Frame>
        </Ui>
    "#;

    let ui = parse_xml(xml).unwrap();
    if let XmlElement::Frame(frame) = &ui.elements[0] {
        create_frame_from_xml(
            &env.loader_env(),
            frame,
            "Frame",
            None,
            None,
            None,
            &mut LoadTiming::default(),
        )
        .unwrap();
    }

    let strata: String = env
        .eval("return BlizzardStrataChild:GetFrameStrata()")
        .unwrap();
    assert_eq!(strata, "HIGH");
}

#[test]
fn test_set_frame_strata_blizzard_is_ignored() {
    let env = WowLuaEnv::new().unwrap();
    let strata: String = env
        .eval(
            r#"
            local frame = CreateFrame("Frame", nil, UIParent)
            frame:SetFrameStrata("HIGH")
            frame:SetFrameStrata("BLIZZARD")
            return frame:GetFrameStrata()
            "#,
        )
        .unwrap();
    assert_eq!(strata, "HIGH");
}

#[test]
fn test_set_frame_strata_overwrites_xml_literal_child_strata() {
    clear_templates();
    let env = WowLuaEnv::new().unwrap();

    let xml = parse_xml(
        r#"
        <Ui>
            <Frame name="OverwriteChildStrataParent" parent="UIParent" frameStrata="HIGH">
                <Frames>
                    <Frame parentKey="ParentChild" frameStrata="PARENT"/>
                    <Frame parentKey="FixedChild" frameStrata="MEDIUM"/>
                </Frames>
            </Frame>
        </Ui>
        "#,
    )
    .unwrap();
    if let XmlElement::Frame(frame) = &xml.elements[0] {
        create_frame_from_xml(
            &env.loader_env(),
            frame,
            "Frame",
            None,
            None,
            None,
            &mut LoadTiming::default(),
        )
        .unwrap();
    }

    let (parent_before, fixed_before, parent_after, fixed_after): (
        String,
        String,
        String,
        String,
    ) = env
        .eval(
            r#"
            local frame = OverwriteChildStrataParent
            local parentBefore = frame.ParentChild:GetFrameStrata()
            local fixedBefore = frame.FixedChild:GetFrameStrata()
            frame:SetFrameStrata("LOW")
            return parentBefore, fixedBefore,
                   frame.ParentChild:GetFrameStrata(), frame.FixedChild:GetFrameStrata()
            "#,
        )
        .unwrap();

    assert_eq!(parent_before, "HIGH");
    assert_eq!(fixed_before, "MEDIUM");
    assert_eq!(parent_after, "LOW");
    assert_eq!(fixed_after, "LOW");
}

#[test]
fn test_set_frame_strata_preserves_runtime_fixed_child() {
    let env = WowLuaEnv::new().unwrap();
    let (strata, fixed): (String, bool) = env
        .eval(
            r#"
            local parent = CreateFrame("Frame", nil, UIParent)
            parent:SetFrameStrata("HIGH")
            local child = CreateFrame("Frame", nil, parent)
            child:SetFrameStrata("MEDIUM")
            child:SetFixedFrameStrata(true)
            parent:SetFrameStrata("LOW")
            return child:GetFrameStrata(), child:HasFixedFrameStrata()
            "#,
        )
        .unwrap();

    assert_eq!(strata, "MEDIUM");
    assert!(fixed);
}

#[test]
fn test_set_parent_recomputes_non_fixed_xml_literal_strata() {
    clear_templates();
    let env = WowLuaEnv::new().unwrap();

    let xml = parse_xml(
        r#"
        <Ui>
            <Frame name="ReparentHighStrata" parent="UIParent" frameStrata="HIGH"/>
            <Frame name="ReparentLowStrata" parent="UIParent" frameStrata="LOW"/>
            <Frame name="ReparentXmlLiteralChild" parent="ReparentHighStrata" frameStrata="MEDIUM"/>
        </Ui>
        "#,
    )
    .unwrap();
    for element in &xml.elements {
        if let XmlElement::Frame(frame) = element {
            create_frame_from_xml(
                &env.loader_env(),
                frame,
                "Frame",
                None,
                None,
                None,
                &mut LoadTiming::default(),
            )
            .unwrap();
        }
    }

    let (before, after, fixed): (String, String, bool) = env
        .eval(
            r#"
            local child = ReparentXmlLiteralChild
            local before = child:GetFrameStrata()
            child:SetParent(ReparentLowStrata)
            return before, child:GetFrameStrata(), child:HasFixedFrameStrata()
            "#,
        )
        .unwrap();

    assert_eq!(before, "MEDIUM");
    assert_eq!(after, "LOW");
    assert!(!fixed);
}

#[test]
fn test_frame_strata_inherited_from_template() {
    clear_templates();
    let env = WowLuaEnv::new().unwrap();

    let template_xml = r#"
        <Ui>
            <Frame name="HighStrataTemplate" virtual="true" frameStrata="HIGH">
                <Size x="100" y="100"/>
            </Frame>
        </Ui>
    "#;
    let ui = parse_xml(template_xml).unwrap();
    if let XmlElement::Frame(frame) = &ui.elements[0] {
        create_frame_from_xml(
            &env.loader_env(),
            frame,
            "Frame",
            None,
            None,
            None,
            &mut LoadTiming::default(),
        )
        .unwrap();
    }

    let frame_xml = r#"
        <Ui>
            <Frame name="InheritsHighStrata" parent="UIParent" inherits="HighStrataTemplate">
                <Anchors><Anchor point="CENTER"/></Anchors>
            </Frame>
        </Ui>
    "#;
    let ui2 = parse_xml(frame_xml).unwrap();
    if let XmlElement::Frame(frame) = &ui2.elements[0] {
        create_frame_from_xml(
            &env.loader_env(),
            frame,
            "Frame",
            None,
            None,
            None,
            &mut LoadTiming::default(),
        )
        .unwrap();
    }

    let strata: String = env
        .eval("return InheritsHighStrata:GetFrameStrata()")
        .unwrap();
    assert_eq!(strata, "HIGH");
}

#[test]
fn test_xml_frame_level_uses_parent_relative_offset() {
    clear_templates();
    let env = WowLuaEnv::new().unwrap();

    let xml = r#"
        <Ui>
            <Frame name="XmlLevelParent" parent="UIParent" frameLevel="50">
                <Size x="10" y="10"/>
            </Frame>
            <Frame name="XmlLevelChild" parent="XmlLevelParent" frameLevel="10">
                <Size x="10" y="10"/>
            </Frame>
        </Ui>
    "#;

    let ui = parse_xml(xml).unwrap();
    for element in &ui.elements {
        if let XmlElement::Frame(frame) = element {
            create_frame_from_xml(
                &env.loader_env(),
                frame,
                "Frame",
                None,
                None,
                None,
                &mut LoadTiming::default(),
            )
            .unwrap();
        }
    }

    // Ground truth (XmlFrameLevelProbe, retail 12.0.5): a bare XML
    // `frameLevel` is the ABSOLUTE level (10), not a parent-relative offset,
    // and the frame does NOT report IsUsingParentLevel.
    let (parent_level, child_level, child_uses_parent): (i32, i32, bool) = env
        .eval(
            r#"
            return XmlLevelParent:GetFrameLevel(), XmlLevelChild:GetFrameLevel(), XmlLevelChild:IsUsingParentLevel()
            "#,
        )
        .unwrap();
    assert_eq!(parent_level, 50);
    assert_eq!(
        child_level, 10,
        "bare XML frameLevel is absolute, not parent + offset"
    );
    assert!(!child_uses_parent);

    // It is still non-fixed: a later parent level change shifts the child by
    // the parent's delta (probe: parent 50->60 moved childPlain 10->20). Here
    // parent 50->300 is +250, so the child's absolute 10 becomes 260.
    let updated_child_level: i32 = env
        .eval(
            r#"
            XmlLevelParent:SetFrameLevel(300)
            return XmlLevelChild:GetFrameLevel()
            "#,
        )
        .unwrap();
    assert_eq!(updated_child_level, 260);
}

fn load_xml_fixed_frame_level_probe(env: &WowLuaEnv) {
    let ui = parse_xml(XML_FIXED_FRAME_LEVEL_PROBE).unwrap();
    for element in &ui.elements {
        if let XmlElement::Frame(frame) = element {
            create_frame_from_xml(
                &env.loader_env(),
                frame,
                "Frame",
                None,
                None,
                None,
                &mut LoadTiming::default(),
            )
            .unwrap();
        }
    }

}

#[test]
fn test_xml_fixed_frame_level_shifts_when_parent_level_changes() {
    clear_templates();
    let env = WowLuaEnv::new().unwrap();
    load_xml_fixed_frame_level_probe(&env);
    let before: i32 = env.eval("return XmlFixedChild:GetFrameLevel()").unwrap();
    let after: i32 = env
        .eval("XmlFixedParent:SetFrameLevel(400); return XmlFixedChild:GetFrameLevel()")
        .unwrap();
    assert_eq!(before, 10);
    // VERIFIED (R14 Q6, pr46-49-decomp-answers.md:2225-2239): fixed same-strata children shift by the parent's delta.
    assert_eq!(after, 360);
}

#[test]
fn test_xml_frame_level_inherited_from_template_is_parent_relative_offset() {
    clear_templates();
    let env = WowLuaEnv::new().unwrap();

    let template_xml = r#"
        <Ui>
            <Frame name="XmlLevelOffsetTemplate" virtual="true" frameLevel="10">
                <Size x="10" y="10"/>
            </Frame>
        </Ui>
    "#;
    let template_ui = parse_xml(template_xml).unwrap();
    if let XmlElement::Frame(frame) = &template_ui.elements[0] {
        create_frame_from_xml(
            &env.loader_env(),
            frame,
            "Frame",
            None,
            None,
            None,
            &mut LoadTiming::default(),
        )
        .unwrap();
    }

    let instance_xml = r#"
        <Ui>
            <Frame name="XmlTemplateLevelParent" parent="UIParent" frameLevel="80">
                <Size x="10" y="10"/>
            </Frame>
            <Frame name="XmlTemplateLevelChild" parent="XmlTemplateLevelParent" inherits="XmlLevelOffsetTemplate">
                <Size x="10" y="10"/>
            </Frame>
        </Ui>
    "#;
    let instance_ui = parse_xml(instance_xml).unwrap();
    for element in &instance_ui.elements {
        if let XmlElement::Frame(frame) = element {
            create_frame_from_xml(
                &env.loader_env(),
                frame,
                "Frame",
                None,
                None,
                None,
                &mut LoadTiming::default(),
            )
            .unwrap();
        }
    }

    // Ground truth (XmlFrameLevelProbe childTemplated): a frameLevel inherited
    // from a template behaves like an instance bare frameLevel — absolute 10,
    // non-fixed, not using parent level. Parent is 80, so the gap is -70.
    let (parent_level, child_level, child_uses_parent): (i32, i32, bool) = env
        .eval(
            r#"
            return XmlTemplateLevelParent:GetFrameLevel(), XmlTemplateLevelChild:GetFrameLevel(), XmlTemplateLevelChild:IsUsingParentLevel()
            "#,
        )
        .unwrap();
    assert_eq!(parent_level, 80);
    assert_eq!(
        child_level, 10,
        "template-inherited XML frameLevel is absolute, not parent + offset"
    );
    assert!(!child_uses_parent);
}

#[test]
fn test_xml_use_parent_level_overrides_inherited_frame_level() {
    clear_templates();
    let env = WowLuaEnv::new().unwrap();

    // Mirrors the DialogBorderTemplate / NineSlicePanelTemplate combo:
    // a high explicit frameLevel on a chain entry that should be overridden
    // by `useParentLevel="true"` on a sibling chain entry.
    let template_xml = r#"
        <Ui>
            <Frame name="HighLevelTemplate" virtual="true" frameLevel="500">
                <Size x="10" y="10"/>
            </Frame>
            <Frame name="ParentLevelBorderTemplate" virtual="true" inherits="HighLevelTemplate" useParentLevel="true">
                <Size x="10" y="10"/>
            </Frame>
        </Ui>
    "#;
    let template_ui = parse_xml(template_xml).unwrap();
    for element in &template_ui.elements {
        if let XmlElement::Frame(frame) = element {
            create_frame_from_xml(
                &env.loader_env(),
                frame,
                "Frame",
                None,
                None,
                None,
                &mut LoadTiming::default(),
            )
            .unwrap();
        }
    }

    let instance_xml = r#"
        <Ui>
            <Frame name="UseParentLevelHost" parent="UIParent" frameLevel="40">
                <Size x="100" y="100"/>
                <Frames>
                    <Frame parentKey="Border" inherits="ParentLevelBorderTemplate"/>
                    <Frame parentKey="Header">
                        <Size x="50" y="20"/>
                    </Frame>
                </Frames>
            </Frame>
        </Ui>
    "#;
    let instance_ui = parse_xml(instance_xml).unwrap();
    if let XmlElement::Frame(frame) = &instance_ui.elements[0] {
        create_frame_from_xml(
            &env.loader_env(),
            frame,
            "Frame",
            None,
            None,
            None,
            &mut LoadTiming::default(),
        )
        .unwrap();
    }

    let (host_level, border_level, header_level): (i32, i32, i32) = env
        .eval(
            r#"
            return UseParentLevelHost:GetFrameLevel(),
                   UseParentLevelHost.Border:GetFrameLevel(),
                   UseParentLevelHost.Header:GetFrameLevel()
            "#,
        )
        .unwrap();
    assert_eq!(
        border_level, host_level,
        "useParentLevel=true should pin the border to the host's level (not host+500)"
    );
    assert!(
        header_level > border_level,
        "default-level sibling should render above the useParentLevel border (header={header_level}, border={border_level})"
    );

    // Border must follow parent level changes (offset 0, not fixed).
    let (host_after, border_after): (i32, i32) = env
        .eval(
            r#"
            UseParentLevelHost:SetFrameLevel(120)
            return UseParentLevelHost:GetFrameLevel(), UseParentLevelHost.Border:GetFrameLevel()
            "#,
        )
        .unwrap();
    assert_eq!(host_after, 120);
    assert_eq!(border_after, host_after);
}
