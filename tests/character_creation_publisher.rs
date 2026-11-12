#![cfg(any(feature = "client-retail", feature = "client-wowforever"))]

use wow_ui_sim::c_api::c_character_creation_customizations::{
    Category, Choice, Icon, OptionRecord, PublishedValue as V,
};
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::screen::ScreenKind;

fn env() -> WowLuaEnv {
    let env = WowLuaEnv::new().unwrap();
    env.set_screen_mode(ScreenKind::CharacterCreate);
    env
}

#[test]
fn host_seeded_customizations_publish_nested_records_without_sorting() {
    let env = env();
    let choice = |id, order| Choice {
        fields: vec![
            ("id".into(), V::Number(id)),
            (
                "name".into(),
                V::Text(format!("choice{order}").into_bytes()),
            ),
            (
                "swatchColor1".into(),
                V::Object(vec![("hostOpaque".into(), V::Boolean(true))]),
            ),
            ("isNew".into(), V::Boolean(true)),
        ],
    };
    let option = |id, order| OptionRecord {
        fields: vec![
            ("id".into(), V::Number(id)),
            ("orderIndex".into(), V::Number(order)),
            ("currentChoiceIndex".into(), V::Number(2.0)),
        ],
        choices: vec![choice(31.0, 9), choice(32.0, 1)],
    };
    env.state().borrow_mut().character_customizations = Some(vec![
        Category {
            fields: vec![
                ("id".into(), V::Number(11.0)),
                ("orderIndex".into(), V::Number(9.0)),
            ],
            icon: Icon {
                id: 0,
                resolved: Some(b"ignored".to_vec()),
            },
            selected_icon: Icon {
                id: 8,
                resolved: None,
            },
            camera_zoom_level: Some(17),
            camera_distance_offset: Some(0.375),
            options: vec![option(21.0, 9.0), option(22.0, 1.0)],
        },
        Category {
            fields: vec![
                ("id".into(), V::Number(12.0)),
                ("orderIndex".into(), V::Number(1.0)),
            ],
            ..Default::default()
        },
    ]);
    env.exec(r#"
        assert(select('#', C_CharacterCreation.GetAvailableCustomizations()) == 1)
        local rows = C_CharacterCreation.GetAvailableCustomizations()
        assert(#rows == 2 and rows[1].id == 11 and rows[2].id == 12)
        local row = rows[1]
        assert(row.icon == nil and row.selectedIcon == '')
        assert(row.cameraZoomLevel == 17 and row.cameraZoomLevel % 1 == 0)
        assert(row.cameraDistanceOffset == 0.375)
        assert(row.options[1].id == 21 and row.options[2].id == 22)
        assert(row.options[1].choices[1].id == 31 and row.options[1].choices[2].id == 32)
        -- Opaque values are host fixtures, not claims about native swatch shape.
        assert(row.options[1].choices[1].swatchColor1.hostOpaque == true)
        assert(row.options[1].currentChoiceIndex == 2)
        assert(row.options[1].choices[1].lockedText == nil)
        C_CharacterCreation.SaveSeenChoices()
        assert(C_CharacterCreation.GetAvailableCustomizations()[1].options[1].choices[1].isNew == false)
    "#).unwrap();
}

#[test]
fn seeded_empty_is_distinct_from_unseeded_temporary_provider() {
    let env = env();
    env.exec("__beforeCustomizations = C_CharacterCreation.GetAvailableCustomizations()")
        .unwrap();
    env.state().borrow_mut().character_customizations = Some(vec![]);
    env.exec("local t = C_CharacterCreation.GetAvailableCustomizations(); assert(type(t) == 'table' and next(t) == nil); assert(select('#', C_CharacterCreation.GetAvailableCustomizations()) == 1)").unwrap();
    env.state().borrow_mut().character_customizations = None;
    env.exec("local t = C_CharacterCreation.GetAvailableCustomizations(); assert(#t == #__beforeCustomizations); for i,row in ipairs(t) do assert(row.id == __beforeCustomizations[i].id) end").unwrap();
}
