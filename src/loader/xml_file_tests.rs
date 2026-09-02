use super::*;

fn default_frame() -> FrameXml {
    FrameXml::default()
}

/// Helper: call resolve_frame_element and return (widget_type, intrinsic).
fn resolve(elem: &XmlElement) -> Option<(&'static str, Option<&'static str>)> {
    resolve_frame_element(elem).map(|(_, wt, intr)| (wt, intr))
}

#[test]
fn specialized_widget_types() {
    let f = default_frame();
    assert_eq!(
        resolve(&XmlElement::Frame(f.clone())),
        Some(("Frame", None))
    );
    assert_eq!(
        resolve(&XmlElement::Button(f.clone())),
        Some(("Button", None))
    );
    assert_eq!(
        resolve(&XmlElement::ItemButton(f.clone())),
        Some(("Button", Some("ItemButton")))
    );
    assert_eq!(
        resolve(&XmlElement::CheckButton(f.clone())),
        Some(("CheckButton", None))
    );
    assert_eq!(
        resolve(&XmlElement::EditBox(f.clone())),
        Some(("EditBox", None))
    );
    assert_eq!(
        resolve(&XmlElement::EventEditBox(f.clone())),
        Some(("EditBox", Some("EventEditBox")))
    );
    assert_eq!(
        resolve(&XmlElement::ScrollFrame(f.clone())),
        Some(("ScrollFrame", None))
    );
    assert_eq!(
        resolve(&XmlElement::EventScrollFrame(f.clone())),
        Some(("ScrollFrame", Some("EventScrollFrame")))
    );
    assert_eq!(
        resolve(&XmlElement::Slider(f.clone())),
        Some(("Slider", None))
    );
    assert_eq!(
        resolve(&XmlElement::StatusBar(f.clone())),
        Some(("StatusBar", None))
    );
    assert_eq!(
        resolve(&XmlElement::Cooldown(f.clone())),
        Some(("Cooldown", None))
    );
    assert_eq!(
        resolve(&XmlElement::GameTooltip(f.clone())),
        Some(("GameTooltip", None))
    );
    assert_eq!(
        resolve(&XmlElement::ColorSelect(f.clone())),
        Some(("ColorSelect", None))
    );
    assert_eq!(
        resolve(&XmlElement::Model(f.clone())),
        Some(("Model", None))
    );
    assert_eq!(
        resolve(&XmlElement::ModelScene(f.clone())),
        Some(("ModelScene", None))
    );
    assert_eq!(
        resolve(&XmlElement::SimpleHTML(f.clone())),
        Some(("SimpleHTML", None))
    );
    assert_eq!(
        resolve(&XmlElement::Minimap(f.clone())),
        Some(("Minimap", None))
    );
    assert_eq!(
        resolve(&XmlElement::MessageFrame(f.clone())),
        Some(("MessageFrame", None))
    );
}

#[test]
fn player_model_variants_all_map_to_player_model() {
    let f = default_frame();
    assert_eq!(
        resolve(&XmlElement::PlayerModel(f.clone())),
        Some(("PlayerModel", None))
    );
    assert_eq!(
        resolve(&XmlElement::CinematicModel(f.clone())),
        Some(("PlayerModel", None))
    );
    assert_eq!(
        resolve(&XmlElement::TabardModel(f.clone())),
        Some(("PlayerModel", None))
    );
    assert_eq!(
        resolve(&XmlElement::DressUpModel(f.clone())),
        Some(("PlayerModel", None))
    );
}

#[test]
fn button_intrinsic_variants() {
    let f = default_frame();
    // DropDownToggleButton and EventButton map to plain Button (no intrinsic) in XmlElement
    assert_eq!(
        resolve(&XmlElement::DropDownToggleButton(f.clone())),
        Some(("Button", None))
    );
    assert_eq!(
        resolve(&XmlElement::EventButton(f.clone())),
        Some(("Button", None))
    );
    // DropdownButton has intrinsic
    assert_eq!(
        resolve(&XmlElement::DropdownButton(f.clone())),
        Some(("Button", Some("DropdownButton")))
    );
    // ContainedAlertFrame has intrinsic
    assert_eq!(
        resolve(&XmlElement::ContainedAlertFrame(f.clone())),
        Some(("Button", Some("ContainedAlertFrame")))
    );
}

#[test]
fn scrolling_message_frame_has_intrinsic() {
    let f = default_frame();
    assert_eq!(
        resolve(&XmlElement::ScrollingMessageFrame(f)),
        Some(("MessageFrame", Some("ScrollingMessageFrame")))
    );
}

#[test]
fn frame_like_elements_preserve_supported_alias_types() {
    let f = default_frame();
    let preserved_aliases = [
        XmlElement::EventFrame(f.clone()),
        XmlElement::UnitPositionFrame(f.clone()),
        XmlElement::OffScreenFrame(f.clone()),
        XmlElement::Checkout(f.clone()),
        XmlElement::FogOfWarFrame(f.clone()),
        XmlElement::QuestPOIFrame(f.clone()),
        XmlElement::ArchaeologyDigSiteFrame(f.clone()),
        XmlElement::ScenarioPOIFrame(f.clone()),
        XmlElement::Browser(f.clone()),
        XmlElement::MovieFrame(f.clone()),
    ];
    for elem in &preserved_aliases {
        let (_, tag) = elem.as_frame_data().unwrap();
        assert_eq!(
            resolve(elem),
            Some((tag, None)),
            "Expected preserved type for {:?}",
            std::mem::discriminant(elem)
        );
    }
}

#[test]
fn unsupported_frame_like_elements_still_fall_back_to_frame() {
    let f = default_frame();
    let frame_likes = [
        XmlElement::TaxiRouteFrame(f.clone()),
        XmlElement::ModelFFX(f.clone()),
        XmlElement::UiCamera(f.clone()),
        XmlElement::UIThemeContainerFrame(f.clone()),
        XmlElement::MapScene(f.clone()),
        XmlElement::Line(f.clone()),
        XmlElement::WorldFrame(f.clone()),
    ];
    for elem in &frame_likes {
        assert_eq!(
            resolve(elem),
            Some(("Frame", None)),
            "Expected Frame for {:?}",
            std::mem::discriminant(elem)
        );
    }
}

#[test]
fn non_frame_elements_return_none() {
    use crate::xml::ScriptXml;
    assert_eq!(
        resolve(&XmlElement::Script(ScriptXml {
            file: None,
            inline: None
        })),
        None
    );
    assert_eq!(resolve(&XmlElement::Text("hello".into())), None);
    assert_eq!(resolve(&XmlElement::Unknown), None);
}

/// Document the differences between XmlElement and FrameElement mappings.
/// ItemButton resolves as a Button with the ItemButton intrinsic base here,
/// while FrameElement preserves the raw alias and inherits are resolved later.
/// DropDownToggleButton/EventButton have no intrinsic here but do in FrameElement.
#[test]
fn xml_vs_frame_element_divergences() {
    let f = default_frame();
    // XmlElement::ItemButton -> ("Button", Some("ItemButton"))
    assert_eq!(
        resolve(&XmlElement::ItemButton(f.clone())),
        Some(("Button", Some("ItemButton")))
    );
    // XmlElement::DropDownToggleButton -> ("Button", None) — no intrinsic
    assert_eq!(
        resolve(&XmlElement::DropDownToggleButton(f.clone())),
        Some(("Button", None))
    );
    // XmlElement::EventButton -> ("Button", None) — no intrinsic
    assert_eq!(
        resolve(&XmlElement::EventButton(f.clone())),
        Some(("Button", None))
    );
}

#[test]
fn roman_font_overrides_with_all_fields() {
    let ff = crate::xml::FontFamilyXml {
        name: Some("TestFont".to_string()),
        is_virtual: None,
        members: vec![crate::xml::FontFamilyMemberXml {
            alphabet: Some("roman".to_string()),
            font: Some(crate::xml::FontXml {
                font: Some("Fonts\\Test.TTF".to_string()),
                height: Some(14.0),
                outline: Some("OUTLINE".to_string()),
                ..Default::default()
            }),
        }],
    };
    let code = build_roman_font_overrides("TestFont", &ff);
    let env = crate::lua_api::WowLuaEnv::new().expect("env");
    let lua_code = FONT_FAMILY_LUA_TEMPLATE.replace("{name}", "TestFont");
    env.exec(&lua_code)
        .expect("font family template should load");
    env.exec(&code)
        .expect("roman font overrides should apply to font object");

    let (path, height, flags): (String, f64, String) = env
        .eval(
            r#"
            local font = TestFont:GetFontObjectForAlphabet("roman")
            return font:GetFont()
            "#,
        )
        .expect("roman font override values should be observable");

    assert_eq!(path, "Fonts/Test.TTF");
    assert_eq!(height, 14.0);
    assert_eq!(flags, "OUTLINE");
}

#[test]
fn roman_font_overrides_emit_shadow_from_xml() {
    let ff = crate::xml::FontFamilyXml {
        name: Some("ShadowFont".to_string()),
        is_virtual: None,
        members: vec![crate::xml::FontFamilyMemberXml {
            alphabet: Some("roman".to_string()),
            font: Some(crate::xml::FontXml {
                font: Some("Fonts\\Test.TTF".to_string()),
                height: Some(14.0),
                shadow: Some(crate::xml::ShadowXml {
                    offset: Some(crate::xml::ShadowOffsetXml {
                        x: Some(1.0),
                        y: Some(-1.0),
                        abs_dimension: None,
                    }),
                    color: Some(crate::xml::ColorXml {
                        r: Some(0.1),
                        g: Some(0.2),
                        b: Some(0.3),
                        a: Some(0.4),
                        color: None,
                    }),
                }),
                ..Default::default()
            }),
        }],
    };
    let code = build_roman_font_overrides("ShadowFont", &ff);
    let env = crate::lua_api::WowLuaEnv::new().expect("env");
    let lua_code = FONT_FAMILY_LUA_TEMPLATE.replace("{name}", "ShadowFont");
    env.exec(&lua_code)
        .expect("font family template should load");
    env.exec(&code)
        .expect("roman font overrides should apply to font object");

    let (x, y, r, g, b, a): (f64, f64, f64, f64, f64, f64) = env
        .eval(
            r#"
            local font = ShadowFont:GetFontObjectForAlphabet("roman")
            local x, y = font:GetShadowOffset()
            local r, g, b, a = font:GetShadowColor()
            return x, y, r, g, b, a
            "#,
        )
        .expect("shadow values should be observable on the font object");

    assert_eq!((x, y), (1.0, -1.0));
    assert!(
        (r - 0.1).abs() < 1e-6 && (g - 0.2).abs() < 1e-6,
        "shadow colour r/g: {r} {g}"
    );
    assert!(
        (b - 0.3).abs() < 1e-6 && (a - 0.4).abs() < 1e-6,
        "shadow colour b/a: {b} {a}"
    );
}

#[test]
fn font_family_template_supports_simple_font_alphabet_lookup() {
    let env = crate::lua_api::WowLuaEnv::new().expect("env");
    let lua_code = FONT_FAMILY_LUA_TEMPLATE.replace("{name}", "XmlFamilyAlphabetProbe");
    env.exec(&lua_code)
        .expect("font family template should load");

    let (same_object, path, height, flags): (bool, String, f64, String) = env
        .eval(
            r#"
            local font = XmlFamilyAlphabetProbe:GetFontObjectForAlphabet("roman")
            local path, height, flags = font:GetFont()
            return font == XmlFamilyAlphabetProbe, path, height, flags
            "#,
        )
        .expect("font family alphabet lookup should return a usable font object");

    assert!(same_object);
    assert_eq!(path, "Fonts/FRIZQT__.TTF");
    assert_eq!(height, 12.0);
    assert_eq!(flags, "");
}

#[test]
fn font_family_template_supports_shadow_methods() {
    let env = crate::lua_api::WowLuaEnv::new().expect("env");
    let lua_code = FONT_FAMILY_LUA_TEMPLATE.replace("{name}", "XmlFamilyShadowProbe");
    env.exec(&lua_code)
        .expect("font family template should load");

    let (r, g, b, a, x, y, metatable_type, index_type): (
        f64,
        f64,
        f64,
        f64,
        f64,
        f64,
        String,
        String,
    ) = env
        .eval(
            r#"
            XmlFamilyShadowProbe:SetShadowColor(0.2, 0.3, 0.4, 0.5)
            XmlFamilyShadowProbe:SetShadowOffset(3, -4)
            local r, g, b, a = XmlFamilyShadowProbe:GetShadowColor()
            local x, y = XmlFamilyShadowProbe:GetShadowOffset()
            local mt = getmetatable(XmlFamilyShadowProbe)
            return r, g, b, a, x, y, type(mt), type(mt and mt.__index)
            "#,
        )
        .expect("font family shadow API should round trip");

    assert_eq!((r, g, b, a), (0.2, 0.3, 0.4, 0.5));
    assert_eq!((x, y), (3.0, -4.0));
    assert_eq!(metatable_type, "table");
    assert_eq!(index_type, "table");
}

#[test]
fn font_template_supports_simple_font_alphabet_lookup() {
    let env = crate::lua_api::WowLuaEnv::new().expect("env");
    let lua_code = build_font_lua_code(
        "XmlFontAlphabetProbe",
        &crate::xml::FontXml {
            font: Some("Fonts\\Alphabet.TTF".to_string()),
            height: Some(18.0),
            outline: Some("THICKOUTLINE".to_string()),
            ..Default::default()
        },
        "Fonts/Alphabet.TTF",
    );
    env.exec(&lua_code).expect("font template should load");

    let (same_object, path, height, flags): (bool, String, f64, String) = env
        .eval(
            r#"
            local font = XmlFontAlphabetProbe:GetFontObjectForAlphabet("roman")
            local path, height, flags = font:GetFont()
            return font == XmlFontAlphabetProbe, path, height, flags
            "#,
        )
        .expect("font alphabet lookup should return a usable font object");

    assert!(same_object);
    assert_eq!(path, "Fonts/Alphabet.TTF");
    assert_eq!(height, 18.0);
    assert_eq!(flags, "THICKOUTLINE");
}

#[test]
fn roman_font_overrides_no_roman_member() {
    let ff = crate::xml::FontFamilyXml {
        name: Some("TestFont".to_string()),
        is_virtual: None,
        members: vec![crate::xml::FontFamilyMemberXml {
            alphabet: Some("hangul".to_string()),
            font: Some(crate::xml::FontXml::default()),
        }],
    };
    let code = build_roman_font_overrides("TestFont", &ff);
    assert!(code.is_empty());
}

#[test]
fn roman_font_overrides_partial_fields() {
    let ff = crate::xml::FontFamilyXml {
        name: Some("TestFont".to_string()),
        is_virtual: None,
        members: vec![crate::xml::FontFamilyMemberXml {
            alphabet: Some("roman".to_string()),
            font: Some(crate::xml::FontXml {
                height: Some(16.0),
                ..Default::default()
            }),
        }],
    };
    let code = build_roman_font_overrides("TestFont", &ff);
    let env = crate::lua_api::WowLuaEnv::new().expect("env");
    let lua_code = FONT_FAMILY_LUA_TEMPLATE.replace("{name}", "TestFont");
    env.exec(&lua_code)
        .expect("font family template should load");
    env.exec(&code)
        .expect("partial roman font overrides should apply to font object");

    let (path, height, flags): (String, f64, String) = env
        .eval(
            r#"
            local font = TestFont:GetFontObjectForAlphabet("roman")
            return font:GetFont()
            "#,
        )
        .expect("partial roman font override values should be observable");

    assert_eq!(path, "Fonts/FRIZQT__.TTF");
    assert_eq!(height, 16.0);
    assert_eq!(flags, "");
}
