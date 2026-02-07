use std::path::{Path, PathBuf};
use wow_ui_sim::xml::{parse_xml, parse_xml_file, AnimationElement, FrameChildElement, XmlElement};

fn blizzard_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("Interface/BlizzardUI/Blizzard_SharedXMLBase")
}

#[test]
fn test_parse_callback_registrant_xml() {
    let path = blizzard_dir().join("CallbackRegistrant.xml");
    let ui = parse_xml_file(&path).expect("Failed to parse XML");

    // Should have Script and Frame elements
    assert!(!ui.elements.is_empty());

    let mut has_script = false;
    let mut has_frame = false;

    for element in &ui.elements {
        match element {
            XmlElement::Script(s) => {
                assert_eq!(s.file.as_deref(), Some("CallbackRegistrant.lua"));
                has_script = true;
            }
            XmlElement::Frame(f) => {
                assert_eq!(f.name.as_deref(), Some("CallbackRegistrantTemplate"));
                assert_eq!(f.mixin.as_deref(), Some("CallbackRegistrantMixin"));
                assert_eq!(f.is_virtual, Some(true));
                has_frame = true;
            }
            _ => {}
        }
    }

    assert!(has_script, "Expected Script element");
    assert!(has_frame, "Expected Frame element");
}

#[test]
fn test_parse_color_swatch_xml() {
    let path = blizzard_dir().join("ColorSwatch.xml");
    let ui = parse_xml_file(&path).expect("Failed to parse XML");

    // Find the ColorSwatchTemplate frame
    for element in &ui.elements {
        if let XmlElement::Frame(f) = element {
            if f.name.as_deref() == Some("ColorSwatchTemplate") {
                // Check mixin
                assert_eq!(f.mixin.as_deref(), Some("ColorSwatchMixin"));

                // Check size
                let size = f.size().expect("Expected size");
                assert_eq!(size.x, Some(16.0));
                assert_eq!(size.y, Some(16.0));

                // Check layers exist
                let layers: Vec<_> = f.layers().collect();
                assert!(!layers.is_empty(), "Expected at least one layer");

                // Check for textures in layers
                let has_textures = layers.iter().any(|l| l.layers.iter().any(|layer| layer.textures().next().is_some()));
                assert!(has_textures, "Expected textures in layers");

                return;
            }
        }
    }
    panic!("ColorSwatchTemplate frame not found");
}

#[test]
fn test_parse_all_xml_in_shared_xml_base() {
    // Try to parse all XML files in the directory
    let dir = blizzard_dir();
    let mut parsed = 0;
    let mut failed = Vec::new();

    for entry in std::fs::read_dir(dir).expect("Failed to read directory") {
        let entry = entry.expect("Failed to read entry");
        let path = entry.path();

        if path.extension().map(|e| e == "xml").unwrap_or(false) {
            match parse_xml_file(&path) {
                Ok(_) => parsed += 1,
                Err(e) => failed.push((path.clone(), e)),
            }
        }
    }

    // Report results
    println!("Parsed {} XML files successfully", parsed);
    if !failed.is_empty() {
        println!("Failed to parse {} files:", failed.len());
        for (path, error) in &failed {
            println!("  {:?}: {}", path.file_name().unwrap(), error);
        }
    }

    // At least some files should parse
    assert!(parsed > 0, "Expected to parse at least some XML files");

    // Allow some failures for now (complex elements we haven't implemented)
    // but most should parse
    let total = parsed + failed.len();
    let success_rate = parsed as f64 / total as f64;
    assert!(
        success_rate >= 0.5,
        "Expected at least 50% success rate, got {:.0}%",
        success_rate * 100.0
    );
}

#[test]
fn test_xml_with_scripts() {
    // Test parsing XML with inline scripts
    let xml = r#"
        <Ui>
            <Frame name="TestFrame">
                <Scripts>
                    <OnLoad>
                        self:RegisterEvent("PLAYER_LOGIN")
                    </OnLoad>
                    <OnEvent method="OnEvent"/>
                    <OnShow inherit="append">
                        print("shown")
                    </OnShow>
                </Scripts>
            </Frame>
        </Ui>
    "#;

    let ui = wow_ui_sim::xml::parse_xml(xml).expect("Failed to parse XML");

    if let XmlElement::Frame(f) = &ui.elements[0] {
        let scripts = f.scripts().expect("Expected scripts");

        // Check OnLoad has inline code
        let on_load = scripts.on_load.first().expect("Expected OnLoad");
        assert!(on_load.body.is_some());

        // Check OnEvent uses method reference
        let on_event = scripts.on_event.first().expect("Expected OnEvent");
        assert_eq!(on_event.method.as_deref(), Some("OnEvent"));

        // Check OnShow has inherit attribute
        let on_show = scripts.on_show.first().expect("Expected OnShow");
        assert_eq!(on_show.inherit.as_deref(), Some("append"));
    } else {
        panic!("Expected Frame element");
    }
}

// --- New frame type XML elements (#3-#18) ---

#[test]
fn test_parse_new_frame_types_top_level() {
    let frame_types = [
        "TaxiRouteFrame",
        "ModelFFX",
        "TabardModel",
        "UiCamera",
        "UnitPositionFrame",
        "OffScreenFrame",
        "Checkout",
        "FogOfWarFrame",
        "QuestPOIFrame",
        "ArchaeologyDigSiteFrame",
        "ScenarioPOIFrame",
        "UIThemeContainerFrame",
        "EventScrollFrame",
        "ContainedAlertFrame",
        "MapScene",
        "ScopedModifier",
    ];

    for ft in &frame_types {
        let xml = format!(
            r#"<Ui><{ft} name="Test{ft}" virtual="true"><Size x="100" y="50"/></{ft}></Ui>"#,
        );
        let ui = parse_xml(&xml).unwrap_or_else(|e| panic!("Failed to parse {ft}: {e}"));
        assert_eq!(ui.elements.len(), 1, "{ft} should produce one element");

        // Verify name and size parsed
        let f = match &ui.elements[0] {
            XmlElement::TaxiRouteFrame(f)
            | XmlElement::ModelFFX(f)
            | XmlElement::TabardModel(f)
            | XmlElement::UiCamera(f)
            | XmlElement::UnitPositionFrame(f)
            | XmlElement::OffScreenFrame(f)
            | XmlElement::Checkout(f)
            | XmlElement::FogOfWarFrame(f)
            | XmlElement::QuestPOIFrame(f)
            | XmlElement::ArchaeologyDigSiteFrame(f)
            | XmlElement::ScenarioPOIFrame(f)
            | XmlElement::UIThemeContainerFrame(f)
            | XmlElement::EventScrollFrame(f)
            | XmlElement::ContainedAlertFrame(f)
            | XmlElement::MapScene(f)
            | XmlElement::ScopedModifier(f) => f,
            other => panic!("Expected {ft} variant, got {other:?}"),
        };
        assert_eq!(f.name.as_deref(), Some(&format!("Test{ft}")[..]));
        assert_eq!(f.size().unwrap().x, Some(100.0));
    }
}

#[test]
fn test_parse_new_frame_types_as_children() {
    let xml = r#"
        <Ui>
            <Frame name="Parent">
                <Frames>
                    <TaxiRouteFrame name="Child1"/>
                    <ModelFFX name="Child2"/>
                    <TabardModel name="Child3"/>
                    <UiCamera name="Child4"/>
                    <UnitPositionFrame name="Child5"/>
                    <OffScreenFrame name="Child6"/>
                    <Checkout name="Child7"/>
                    <FogOfWarFrame name="Child8"/>
                    <QuestPOIFrame name="Child9"/>
                    <ArchaeologyDigSiteFrame name="Child10"/>
                    <ScenarioPOIFrame name="Child11"/>
                    <UIThemeContainerFrame name="Child12"/>
                    <EventScrollFrame name="Child13"/>
                    <ContainedAlertFrame name="Child14"/>
                    <MapScene name="Child15"/>
                    <ScopedModifier name="Child16"/>
                </Frames>
            </Frame>
        </Ui>
    "#;
    let ui = parse_xml(xml).expect("Failed to parse child frame types");
    let frame = match &ui.elements[0] {
        XmlElement::Frame(f) => f,
        _ => panic!("Expected Frame"),
    };
    let frames = frame.all_frame_elements();
    assert_eq!(frames.len(), 16);
}

// --- Animation elements (#19-#29) ---

#[test]
fn test_parse_animation_group_with_alpha() {
    let xml = r#"
        <Ui>
            <Frame name="TestFrame">
                <Animations>
                    <AnimationGroup parentKey="FadeIn" looping="NONE">
                        <Alpha fromAlpha="0" toAlpha="1" duration="0.3" order="1" smoothing="OUT"/>
                    </AnimationGroup>
                </Animations>
            </Frame>
        </Ui>
    "#;
    let ui = parse_xml(xml).expect("Failed to parse");
    let f = match &ui.elements[0] {
        XmlElement::Frame(f) => f,
        _ => panic!("Expected Frame"),
    };
    let anims: Vec<_> = f.children.iter().filter_map(|c| match c {
        FrameChildElement::Animations(a) => Some(a),
        _ => None,
    }).collect();
    assert_eq!(anims.len(), 1);
    let group = &anims[0].animations[0];
    assert_eq!(group.parent_key.as_deref(), Some("FadeIn"));
    assert_eq!(group.looping.as_deref(), Some("NONE"));

    let alpha = group.elements.iter().find_map(|e| match e {
        AnimationElement::Alpha(a) => Some(a),
        _ => None,
    }).expect("Expected Alpha animation");
    assert_eq!(alpha.from_alpha, Some(0.0));
    assert_eq!(alpha.to_alpha, Some(1.0));
    assert_eq!(alpha.duration, Some(0.3));
    assert_eq!(alpha.order, Some(1));
    assert_eq!(alpha.smoothing.as_deref(), Some("OUT"));
}

#[test]
fn test_parse_translation_animation() {
    let xml = r#"
        <Ui>
            <Frame name="T">
                <Animations>
                    <AnimationGroup>
                        <Translation offsetX="10" offsetY="-20" duration="0.5" order="1"/>
                        <LineTranslation offsetX="5" offsetY="5" duration="1.0"/>
                    </AnimationGroup>
                </Animations>
            </Frame>
        </Ui>
    "#;
    let ui = parse_xml(xml).expect("Failed to parse");
    let f = match &ui.elements[0] { XmlElement::Frame(f) => f, _ => panic!() };
    let group = &f.children.iter().find_map(|c| match c {
        FrameChildElement::Animations(a) => Some(a),
        _ => None,
    }).unwrap().animations[0];

    let tr = group.elements.iter().find_map(|e| match e {
        AnimationElement::Translation(a) => Some(a),
        _ => None,
    }).expect("Expected Translation");
    assert_eq!(tr.offset_x, Some(10.0));
    assert_eq!(tr.offset_y, Some(-20.0));

    let lt = group.elements.iter().find_map(|e| match e {
        AnimationElement::LineTranslation(a) => Some(a),
        _ => None,
    }).expect("Expected LineTranslation");
    assert_eq!(lt.offset_x, Some(5.0));
}

#[test]
fn test_parse_rotation_animation() {
    let xml = r#"
        <Ui>
            <Frame name="T">
                <Animations>
                    <AnimationGroup>
                        <Rotation degrees="-180" duration="1.0" smoothing="OUT" childKey="Swirl"/>
                    </AnimationGroup>
                </Animations>
            </Frame>
        </Ui>
    "#;
    let ui = parse_xml(xml).expect("Failed to parse");
    let f = match &ui.elements[0] { XmlElement::Frame(f) => f, _ => panic!() };
    let group = &f.children.iter().find_map(|c| match c {
        FrameChildElement::Animations(a) => Some(a),
        _ => None,
    }).unwrap().animations[0];

    let rot = group.elements.iter().find_map(|e| match e {
        AnimationElement::Rotation(a) => Some(a),
        _ => None,
    }).expect("Expected Rotation");
    assert_eq!(rot.degrees, Some(-180.0));
    assert_eq!(rot.child_key.as_deref(), Some("Swirl"));
}

#[test]
fn test_parse_scale_animations() {
    let xml = r#"
        <Ui>
            <Frame name="T">
                <Animations>
                    <AnimationGroup>
                        <Scale fromScaleX="0" fromScaleY="0" toScaleX="1" toScaleY="1" duration="0.4"/>
                        <LineScale scaleX="2.0" scaleY="2.0" duration="0.2"/>
                    </AnimationGroup>
                </Animations>
            </Frame>
        </Ui>
    "#;
    let ui = parse_xml(xml).expect("Failed to parse");
    let f = match &ui.elements[0] { XmlElement::Frame(f) => f, _ => panic!() };
    let group = &f.children.iter().find_map(|c| match c {
        FrameChildElement::Animations(a) => Some(a),
        _ => None,
    }).unwrap().animations[0];

    let scale = group.elements.iter().find_map(|e| match e {
        AnimationElement::Scale(a) => Some(a),
        _ => None,
    }).expect("Expected Scale");
    assert_eq!(scale.from_scale_x, Some(0.0));
    assert_eq!(scale.to_scale_x, Some(1.0));

    assert!(group.elements.iter().any(|e| matches!(e, AnimationElement::LineScale(_))));
}

#[test]
fn test_parse_path_flipbook_vertexcolor_texcoord() {
    let xml = r#"
        <Ui>
            <Frame name="T">
                <Animations>
                    <AnimationGroup>
                        <Path curve="SMOOTH" duration="1.0"/>
                        <FlipBook flipBookRows="4" flipBookColumns="4" flipBookFrames="16" duration="2.0"/>
                        <VertexColor duration="0.5"/>
                        <TextureCoordTranslation offsetU="0.5" offsetV="-0.5" duration="1.0"/>
                        <Animation duration="0.1" order="1"/>
                    </AnimationGroup>
                </Animations>
            </Frame>
        </Ui>
    "#;
    let ui = parse_xml(xml).expect("Failed to parse");
    let f = match &ui.elements[0] { XmlElement::Frame(f) => f, _ => panic!() };
    let group = &f.children.iter().find_map(|c| match c {
        FrameChildElement::Animations(a) => Some(a),
        _ => None,
    }).unwrap().animations[0];

    let path = group.elements.iter().find_map(|e| match e {
        AnimationElement::Path(a) => Some(a),
        _ => None,
    }).expect("Expected Path");
    assert_eq!(path.curve.as_deref(), Some("SMOOTH"));

    let fb = group.elements.iter().find_map(|e| match e {
        AnimationElement::FlipBook(a) => Some(a),
        _ => None,
    }).expect("Expected FlipBook");
    assert_eq!(fb.flip_book_rows, Some(4));
    assert_eq!(fb.flip_book_columns, Some(4));
    assert_eq!(fb.flip_book_frames, Some(16));

    assert!(group.elements.iter().any(|e| matches!(e, AnimationElement::VertexColor(_))));

    let tc = group.elements.iter().find_map(|e| match e {
        AnimationElement::TextureCoordTranslation(a) => Some(a),
        _ => None,
    }).expect("Expected TextureCoordTranslation");
    assert_eq!(tc.offset_u, Some(0.5));
    assert_eq!(tc.offset_v, Some(-0.5));

    assert!(group.elements.iter().any(|e| matches!(e, AnimationElement::Animation(_))));
}

// --- Frame child elements (#30-#37) ---

#[test]
fn test_parse_text_insets() {
    let xml = r#"
        <Ui>
            <EditBox name="TestEditBox">
                <TextInsets left="5" right="5" top="2" bottom="2"/>
            </EditBox>
        </Ui>
    "#;
    let ui = parse_xml(xml).expect("Failed to parse");
    let f = match &ui.elements[0] { XmlElement::EditBox(f) => f, _ => panic!("Expected EditBox") };
    let insets = f.children.iter().find_map(|c| match c {
        FrameChildElement::TextInsets(i) => Some(i),
        _ => None,
    }).expect("Expected TextInsets");
    assert_eq!(insets.left, Some(5.0));
    assert_eq!(insets.right, Some(5.0));
    assert_eq!(insets.top, Some(2.0));
    assert_eq!(insets.bottom, Some(2.0));
}

#[test]
fn test_parse_pushed_text_offset() {
    let xml = r#"
        <Ui>
            <Button name="TestButton">
                <PushedTextOffset x="1" y="-1"/>
            </Button>
        </Ui>
    "#;
    let ui = parse_xml(xml).expect("Failed to parse");
    let f = match &ui.elements[0] { XmlElement::Button(f) => f, _ => panic!("Expected Button") };
    let offset = f.children.iter().find_map(|c| match c {
        FrameChildElement::PushedTextOffset(s) => Some(s),
        _ => None,
    }).expect("Expected PushedTextOffset");
    assert_eq!(offset.x, Some(1.0));
    assert_eq!(offset.y, Some(-1.0));
}

#[test]
fn test_parse_cooldown_textures() {
    let xml = r#"
        <Ui>
            <Cooldown name="TestCooldown">
                <SwipeTexture parentKey="swipe" atlas="CooldownSwipe"/>
                <EdgeTexture parentKey="edge" atlas="CooldownEdge"/>
                <BlingTexture parentKey="bling" atlas="CooldownBling"/>
            </Cooldown>
        </Ui>
    "#;
    let ui = parse_xml(xml).expect("Failed to parse");
    let f = match &ui.elements[0] { XmlElement::Cooldown(f) => f, _ => panic!("Expected Cooldown") };

    let has_swipe = f.children.iter().any(|c| matches!(c, FrameChildElement::SwipeTexture(_)));
    let has_edge = f.children.iter().any(|c| matches!(c, FrameChildElement::EdgeTexture(_)));
    let has_bling = f.children.iter().any(|c| matches!(c, FrameChildElement::BlingTexture(_)));
    assert!(has_swipe, "Missing SwipeTexture");
    assert!(has_edge, "Missing EdgeTexture");
    assert!(has_bling, "Missing BlingTexture");
}

#[test]
fn test_parse_color_select_textures() {
    let xml = r#"
        <Ui>
            <ColorSelect name="TestColorSelect">
                <ColorWheelTexture parentKey="Wheel"/>
                <ColorWheelThumbTexture parentKey="WheelThumb"/>
                <ColorValueTexture parentKey="Value"/>
                <ColorValueThumbTexture parentKey="ValueThumb"/>
                <ColorAlphaTexture parentKey="Alpha"/>
                <ColorAlphaThumbTexture parentKey="AlphaThumb"/>
            </ColorSelect>
        </Ui>
    "#;
    let ui = parse_xml(xml).expect("Failed to parse");
    let f = match &ui.elements[0] { XmlElement::ColorSelect(f) => f, _ => panic!("Expected ColorSelect") };

    let checks = [
        f.children.iter().any(|c| matches!(c, FrameChildElement::ColorWheelTexture(_))),
        f.children.iter().any(|c| matches!(c, FrameChildElement::ColorWheelThumbTexture(_))),
        f.children.iter().any(|c| matches!(c, FrameChildElement::ColorValueTexture(_))),
        f.children.iter().any(|c| matches!(c, FrameChildElement::ColorValueThumbTexture(_))),
        f.children.iter().any(|c| matches!(c, FrameChildElement::ColorAlphaTexture(_))),
        f.children.iter().any(|c| matches!(c, FrameChildElement::ColorAlphaThumbTexture(_))),
    ];
    for (i, present) in checks.iter().enumerate() {
        assert!(present, "Missing ColorSelect texture child #{i}");
    }
}

#[test]
fn test_parse_simple_html_headers() {
    let xml = r#"
        <Ui>
            <SimpleHTML name="TestHTML">
                <FontStringHeader1 inherits="GameFontNormalLarge"/>
                <FontStringHeader2 inherits="GameFontNormal"/>
                <FontStringHeader3 inherits="GameFontNormalSmall"/>
            </SimpleHTML>
        </Ui>
    "#;
    let ui = parse_xml(xml).expect("Failed to parse");
    let f = match &ui.elements[0] { XmlElement::SimpleHTML(f) => f, _ => panic!("Expected SimpleHTML") };

    let h1 = f.children.iter().find_map(|c| match c {
        FrameChildElement::FontStringHeader1(fs) => Some(fs),
        _ => None,
    }).expect("Missing FontStringHeader1");
    assert_eq!(h1.inherits.as_deref(), Some("GameFontNormalLarge"));

    assert!(f.children.iter().any(|c| matches!(c, FrameChildElement::FontStringHeader2(_))));
    assert!(f.children.iter().any(|c| matches!(c, FrameChildElement::FontStringHeader3(_))));
}

#[test]
fn test_parse_button_state_colors() {
    let xml = r#"
        <Ui>
            <Button name="TestButton">
                <NormalColor r="1" g="0.82" b="0" a="1"/>
                <HighlightColor r="1" g="1" b="1" a="1"/>
                <DisabledColor r="0.5" g="0.5" b="0.5" a="1"/>
            </Button>
        </Ui>
    "#;
    let ui = parse_xml(xml).expect("Failed to parse");
    let f = match &ui.elements[0] { XmlElement::Button(f) => f, _ => panic!("Expected Button") };

    let normal = f.children.iter().find_map(|c| match c {
        FrameChildElement::NormalColor(c) => Some(c),
        _ => None,
    }).expect("Missing NormalColor");
    assert_eq!(normal.r, Some(1.0));
    assert_eq!(normal.g, Some(0.82));

    assert!(f.children.iter().any(|c| matches!(c, FrameChildElement::HighlightColor(_))));
    assert!(f.children.iter().any(|c| matches!(c, FrameChildElement::DisabledColor(_))));
}

#[test]
fn test_parse_actors_container() {
    let xml = r#"
        <Ui>
            <ModelScene name="TestScene">
                <Actors>
                    <Actor parentKey="Actor1" mixin="TestMixin"/>
                    <Actor parentKey="Actor2"/>
                </Actors>
            </ModelScene>
        </Ui>
    "#;
    let ui = parse_xml(xml).expect("Failed to parse");
    let f = match &ui.elements[0] { XmlElement::ModelScene(f) => f, _ => panic!("Expected ModelScene") };

    let actors = f.children.iter().find_map(|c| match c {
        FrameChildElement::Actors(a) => Some(a),
        _ => None,
    }).expect("Missing Actors container");
    assert_eq!(actors.actors.len(), 2);
    assert_eq!(actors.actors[0].parent_key.as_deref(), Some("Actor1"));
    assert_eq!(actors.actors[0].mixin.as_deref(), Some("TestMixin"));
}

#[test]
fn test_parse_model_fog_and_view_insets() {
    let xml = r#"
        <Ui>
            <Model name="TestModel">
                <FogColor r="0.5" g="0.5" b="0.5" a="1.0"/>
                <ViewInsets left="10" right="10" top="5" bottom="5"/>
            </Model>
        </Ui>
    "#;
    let ui = parse_xml(xml).expect("Failed to parse");
    let f = match &ui.elements[0] { XmlElement::Model(f) => f, _ => panic!("Expected Model") };

    let fog = f.children.iter().find_map(|c| match c {
        FrameChildElement::FogColor(c) => Some(c),
        _ => None,
    }).expect("Missing FogColor");
    assert_eq!(fog.r, Some(0.5));

    let insets = f.children.iter().find_map(|c| match c {
        FrameChildElement::ViewInsets(i) => Some(i),
        _ => None,
    }).expect("Missing ViewInsets");
    assert_eq!(insets.left, Some(10.0));
    assert_eq!(insets.top, Some(5.0));
}
