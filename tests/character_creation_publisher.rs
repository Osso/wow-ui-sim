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
fn host_seeded_customizations_publish_nested_records() {
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
        stored_order_key: 0,
        fields: vec![
            ("id".into(), V::Number(id)),
            ("orderIndex".into(), V::Number(order)),
            ("currentChoiceIndex".into(), V::Number(2.0)),
        ],
        choices: vec![choice(32.0, 9), choice(31.0, 1)],
    };
    env.state().borrow_mut().character_customizations = Some(vec![
        Category {
            stored_order_key: 0,
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
            camera_zoom_level: 17,
            camera_distance_offset: Some(0.375),
            options: vec![option(21.0, 9.0), option(22.0, 1.0)],
        },
        Category {
            fields: vec![
                ("id".into(), V::Number(12.0)),
                ("orderIndex".into(), V::Number(1.0)),
            ],
            ..Category::new(0)
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
        assert(row.options[1].choices[1].id == 32 and row.options[1].choices[2].id == 31)
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

#[test]
fn publisher_field_sets_and_unknown_host_values_are_not_reinterpreted() {
    let env = env();
    let fields = |keys: &[&str]| {
        keys.iter()
            .map(|key| ((*key).into(), V::Number(1.0)))
            .collect()
    };
    env.state().borrow_mut().character_customizations = Some(vec![Category {
        stored_order_key: 0,
        fields: fields(&[
            "id",
            "orderIndex",
            "name",
            "undressModel",
            "subcategory",
            "spellShapeshiftFormID",
            "chrModelID",
            "hasNewChoices",
            "needsNativeFormCategory",
            "notNative",
        ]),
        icon: Icon {
            id: 1,
            resolved: Some(b"host/icon".to_vec()),
        },
        selected_icon: Icon {
            id: 2,
            resolved: Some(b"host/selected".to_vec()),
        },
        camera_zoom_level: -4,
        camera_distance_offset: Some(-0.625),
        options: vec![OptionRecord {
            stored_order_key: 0,
            fields: fields(&[
                "id",
                "name",
                "orderIndex",
                "optionType",
                "currentChoiceIndex",
                "hasNewChoices",
                "isSound",
                "notNative",
            ]),
            choices: vec![Choice {
                fields: fields(&[
                    "id",
                    "name",
                    "ineligibleChoice",
                    "isNew",
                    "swatchColor1",
                    "swatchColor2",
                    "soundKit",
                    "isLocked",
                    "lockedText",
                    "notNative",
                ]),
            }],
        }],
    }]);
    env.exec(r#"
        local function fieldsAre(row, expected)
            local count = 0
            for key in pairs(row) do assert(expected[key], key); count = count + 1 end
            local expectedCount = 0
            for key in pairs(expected) do assert(row[key] ~= nil, key); expectedCount = expectedCount + 1 end
            assert(count == expectedCount)
        end
        local row = C_CharacterCreation.GetAvailableCustomizations()[1]
        fieldsAre(row, {id=true,orderIndex=true,name=true,icon=true,selectedIcon=true,undressModel=true,subcategory=true,cameraZoomLevel=true,cameraDistanceOffset=true,spellShapeshiftFormID=true,chrModelID=true,options=true,hasNewChoices=true,needsNativeFormCategory=true})
        fieldsAre(row.options[1], {id=true,name=true,orderIndex=true,optionType=true,choices=true,currentChoiceIndex=true,hasNewChoices=true,isSound=true})
        fieldsAre(row.options[1].choices[1], {id=true,name=true,ineligibleChoice=true,isNew=true,swatchColor1=true,swatchColor2=true,soundKit=true,isLocked=true,lockedText=true})
        assert(row.icon == 'host/icon' and row.selectedIcon == 'host/selected')
        assert(row.cameraZoomLevel == -4 and row.cameraDistanceOffset == -0.625)
        -- Arbitrary values here test the host seam, not native field types.
        assert(row.name == 1 and row.options[1].isSound == 1)
        assert(row.options[1].choices[1].swatchColor2 == 1)
        collectgarbage('collect')
        assert(row.options[1].choices[1].id == 1)
    "#).unwrap();
}

// R79b/#305 and R79c/#314: stored-key sorting and rewritten positions are
// VERIFIED. These neutral host keys make no claim about the inferred DB2 fields.
#[test]
fn host_customization_finalizer_sorts_and_numbers_each_list() {
    let env = env();
    let option = |id, key| OptionRecord {
        stored_order_key: key,
        fields: vec![
            ("id".into(), V::Number(id)),
            ("orderIndex".into(), V::Number(888.0)),
        ],
        ..Default::default()
    };
    let category = |id, key| Category {
        stored_order_key: key,
        fields: vec![
            ("id".into(), V::Number(id)),
            ("orderIndex".into(), V::Number(999.0)),
        ],
        options: vec![option(21.0, 70), option(22.0, -5), option(23.0, 9)],
        ..Category::new(0)
    };
    for rows in [
        vec![category(11.0, 50), category(12.0, -10), category(13.0, 0)],
        vec![category(13.0, 0), category(11.0, 50), category(12.0, -10)],
    ] {
        env.state().borrow_mut().character_customizations = Some(rows);
        env.exec(r#"
            local rows = C_CharacterCreation.GetAvailableCustomizations()
            assert(#rows == 3)
            assert(rows[1].id == 12 and rows[2].id == 13 and rows[3].id == 11)
            for i, row in ipairs(rows) do
                assert(row.orderIndex == i)
                assert(#row.options == 3)
                assert(row.options[1].id == 22 and row.options[2].id == 23 and row.options[3].id == 21)
                for j, option in ipairs(row.options) do assert(option.orderIndex == j) end
            end
        "#).unwrap();
    }
}

#[test]
fn host_customization_finalizer_preserves_small_equal_key_input_order() {
    let env = env();
    for count in [1, 2, 31, 32] {
        let rows = (1..=count)
            .rev()
            .map(|id| Category {
                stored_order_key: 7,
                fields: vec![("id".into(), V::Number(f64::from(id)))],
                options: (1..=count)
                    .rev()
                    .map(|option_id| OptionRecord {
                        stored_order_key: -4,
                        fields: vec![("id".into(), V::Number(f64::from(option_id)))],
                        ..Default::default()
                    })
                    .collect(),
                ..Category::new(0)
            })
            .collect();
        env.state().borrow_mut().character_customizations = Some(rows);
        env.exec(&format!(
            r#"
            local rows = C_CharacterCreation.GetAvailableCustomizations()
            assert(#rows == {count})
            for i, row in ipairs(rows) do
                assert(row.id == {count} + 1 - i and row.orderIndex == i)
                for j, option in ipairs(row.options) do
                    assert(option.id == {count} + 1 - j and option.orderIndex == j)
                end
            end
        "#
        ))
        .unwrap();
    }
}

#[test]
fn host_customization_finalizer_sorts_large_lists_without_tie_order_claims() {
    let env = env();
    env.state().borrow_mut().character_customizations = Some(
        (1..=33)
            .rev()
            .map(|key| Category {
                stored_order_key: key,
                fields: vec![("id".into(), V::Number(key as f64))],
                options: (1..=33)
                    .rev()
                    .map(|option_key| OptionRecord {
                        stored_order_key: option_key,
                        fields: vec![("id".into(), V::Number(option_key as f64))],
                        ..Default::default()
                    })
                    .collect(),
                ..Category::new(0)
            })
            .collect(),
    );
    env.exec(
        r#"
        local rows = C_CharacterCreation.GetAvailableCustomizations()
        assert(#rows == 33)
        for i, row in ipairs(rows) do
            assert(row.id == i and row.orderIndex == i)
            assert(#row.options == 33)
            for j, option in ipairs(row.options) do
                assert(option.id == j and option.orderIndex == j)
            end
        end
    "#,
    )
    .unwrap();
}
