use wow_ui_sim::xml::{
    FrameChildElement, TextureXml, XmlElement, parse_xml, register_texture_template,
    resolve_texture_inheritance,
};

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
    let f = match &ui.elements[0] {
        XmlElement::EditBox(f) => f,
        _ => panic!("Expected EditBox"),
    };
    let insets = f
        .children
        .iter()
        .find_map(|c| match c {
            FrameChildElement::TextInsets(i) => Some(i),
            _ => None,
        })
        .expect("Expected TextInsets");
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
    let f = match &ui.elements[0] {
        XmlElement::Button(f) => f,
        _ => panic!("Expected Button"),
    };
    let offset = f
        .children
        .iter()
        .find_map(|c| match c {
            FrameChildElement::PushedTextOffset(s) => Some(s),
            _ => None,
        })
        .expect("Expected PushedTextOffset");
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
    let f = match &ui.elements[0] {
        XmlElement::Cooldown(f) => f,
        _ => panic!("Expected Cooldown"),
    };

    let has_swipe = f
        .children
        .iter()
        .any(|c| matches!(c, FrameChildElement::SwipeTexture(_)));
    let has_edge = f
        .children
        .iter()
        .any(|c| matches!(c, FrameChildElement::EdgeTexture(_)));
    let has_bling = f
        .children
        .iter()
        .any(|c| matches!(c, FrameChildElement::BlingTexture(_)));
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
    let f = match &ui.elements[0] {
        XmlElement::ColorSelect(f) => f,
        _ => panic!("Expected ColorSelect"),
    };

    let checks = [
        f.children
            .iter()
            .any(|c| matches!(c, FrameChildElement::ColorWheelTexture(_))),
        f.children
            .iter()
            .any(|c| matches!(c, FrameChildElement::ColorWheelThumbTexture(_))),
        f.children
            .iter()
            .any(|c| matches!(c, FrameChildElement::ColorValueTexture(_))),
        f.children
            .iter()
            .any(|c| matches!(c, FrameChildElement::ColorValueThumbTexture(_))),
        f.children
            .iter()
            .any(|c| matches!(c, FrameChildElement::ColorAlphaTexture(_))),
        f.children
            .iter()
            .any(|c| matches!(c, FrameChildElement::ColorAlphaThumbTexture(_))),
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
    let f = match &ui.elements[0] {
        XmlElement::SimpleHTML(f) => f,
        _ => panic!("Expected SimpleHTML"),
    };

    let h1 = f
        .children
        .iter()
        .find_map(|c| match c {
            FrameChildElement::FontStringHeader1(fs) => Some(fs),
            _ => None,
        })
        .expect("Missing FontStringHeader1");
    assert_eq!(h1.inherits.as_deref(), Some("GameFontNormalLarge"));

    assert!(
        f.children
            .iter()
            .any(|c| matches!(c, FrameChildElement::FontStringHeader2(_)))
    );
    assert!(
        f.children
            .iter()
            .any(|c| matches!(c, FrameChildElement::FontStringHeader3(_)))
    );
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
    let f = match &ui.elements[0] {
        XmlElement::Button(f) => f,
        _ => panic!("Expected Button"),
    };

    let normal = f
        .children
        .iter()
        .find_map(|c| match c {
            FrameChildElement::NormalColor(c) => Some(c),
            _ => None,
        })
        .expect("Missing NormalColor");
    assert_eq!(normal.r, Some(1.0));
    assert_eq!(normal.g, Some(0.82));

    assert!(
        f.children
            .iter()
            .any(|c| matches!(c, FrameChildElement::HighlightColor(_)))
    );
    assert!(
        f.children
            .iter()
            .any(|c| matches!(c, FrameChildElement::DisabledColor(_)))
    );
}

#[test]
fn test_parse_texture_alpha_mode_and_blend_mode_attributes() {
    let xml = r#"
        <Ui>
            <Frame name="ModeFrame">
                <Layers>
                    <Layer level="ARTWORK">
                        <Texture parentKey="AlphaModeTex" alphaMode="ADD"/>
                        <Texture parentKey="BlendModeTex" blendMode="MOD"/>
                    </Layer>
                </Layers>
            </Frame>
        </Ui>
    "#;
    let ui = parse_xml(xml).expect("Failed to parse");
    let frame = match &ui.elements[0] {
        XmlElement::Frame(frame) => frame,
        other => panic!("expected frame, got {:?}", other),
    };
    let layer = &frame.layers().next().unwrap().layers[0];
    let alpha_mode = match &layer.elements[0] {
        wow_ui_sim::xml::LayerElement::Texture(texture) => texture,
        other => panic!("expected texture, got {:?}", other),
    };
    let blend_mode = match &layer.elements[1] {
        wow_ui_sim::xml::LayerElement::Texture(texture) => texture,
        other => panic!("expected texture, got {:?}", other),
    };

    assert_eq!(alpha_mode.alpha_mode.as_deref(), Some("ADD"));
    assert_eq!(alpha_mode.blend_mode.as_deref(), None);
    assert_eq!(blend_mode.alpha_mode.as_deref(), None);
    assert_eq!(blend_mode.blend_mode.as_deref(), Some("MOD"));
}

#[test]
fn test_resolve_texture_inheritance_preserves_azerite_parent_array() {
    // Mirrors the Retail AzeritePlugTexture parentArray declaration.
    const TEMPLATE_NAME: &str = "AzeritePlugTexture";
    register_texture_template(
        TEMPLATE_NAME,
        TextureXml {
            parent_array: Some("Plugs".to_string()),
            ..Default::default()
        },
    );

    let inherited = resolve_texture_inheritance(&TextureXml {
        inherits: Some(TEMPLATE_NAME.to_string()),
        ..Default::default()
    });
    assert_eq!(inherited.parent_array.as_deref(), Some("Plugs"));

    let overridden = resolve_texture_inheritance(&TextureXml {
        inherits: Some(TEMPLATE_NAME.to_string()),
        parent_array: Some("InstancePlugs".to_string()),
        ..Default::default()
    });
    assert_eq!(overridden.parent_array.as_deref(), Some("InstancePlugs"));
}

#[test]
fn test_resolve_texture_inheritance_carries_blend_mode_aliases() {
    let template_name = "CodexTextureBlendModeTemplate";
    register_texture_template(
        template_name,
        TextureXml {
            blend_mode: Some("ADD".to_string()),
            ..Default::default()
        },
    );
    let inherited = resolve_texture_inheritance(&TextureXml {
        inherits: Some(template_name.to_string()),
        alpha_mode: Some("MOD".to_string()),
        ..Default::default()
    });

    assert_eq!(inherited.alpha_mode.as_deref(), Some("MOD"));
    assert_eq!(inherited.blend_mode.as_deref(), Some("MOD"));
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
    let f = match &ui.elements[0] {
        XmlElement::ModelScene(f) => f,
        _ => panic!("Expected ModelScene"),
    };

    let actors = f
        .children
        .iter()
        .find_map(|c| match c {
            FrameChildElement::Actors(a) => Some(a),
            _ => None,
        })
        .expect("Missing Actors container");
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
    let f = match &ui.elements[0] {
        XmlElement::Model(f) => f,
        _ => panic!("Expected Model"),
    };

    let fog = f
        .children
        .iter()
        .find_map(|c| match c {
            FrameChildElement::FogColor(c) => Some(c),
            _ => None,
        })
        .expect("Missing FogColor");
    assert_eq!(fog.r, Some(0.5));

    let insets = f
        .children
        .iter()
        .find_map(|c| match c {
            FrameChildElement::ViewInsets(i) => Some(i),
            _ => None,
        })
        .expect("Missing ViewInsets");
    assert_eq!(insets.left, Some(10.0));
    assert_eq!(insets.top, Some(5.0));
}
