use wow_ui_sim::client_profile::{ACTIVE, ClientProfile};
use wow_ui_sim::lua_api::WowLuaEnv;

#[test]
fn item_use_spell_resolves_hearthstone_by_id_link_and_catalog_name() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        for _, item in ipairs({6948, '6948', 'item:6948',
            '|cffffffff|Hitem:6948::::::::80:70:::::::::|h[Hearthstone]|h|r', 'Hearthstone'}) do
            local name, id = C_Item.GetItemSpell(item)
            assert(name == 'Hearthstone' and id == 8690, tostring(item))
            assert(select('#', C_Item.GetItemSpell(item)) == 2)
        end
    "#,
    )
    .unwrap();
}

#[test]
fn item_use_spell_returns_no_values_for_passive_or_unknown_items() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        for _, item in ipairs({25, 19379, 4294967294, 'not an item'}) do
            assert(select('#', C_Item.GetItemSpell(item)) == 0, tostring(item))
        end
    "#,
    )
    .unwrap();
}

#[test]
fn item_use_spell_rejects_invalid_argument_types() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        for _, query in ipairs({function() C_Item.GetItemSpell() end,
            function() C_Item.GetItemSpell(false) end,
            function() C_Item.GetItemSpell({}) end}) do
            local ok, message = pcall(query)
            assert(not ok and message:find('itemInfo', 1, true))
        end
    "#,
    )
    .unwrap();
}

#[test]
fn item_use_spell_selects_the_active_profile_catalog() {
    let env = WowLuaEnv::new().unwrap();
    let (name, id): (String, i64) = env.eval("return C_Item.GetItemSpell(159)").unwrap();
    assert_eq!(name, "Drink");
    assert_eq!(
        id,
        match ACTIVE {
            ClientProfile::Retail | ClientProfile::Ptr => 1291950,
            _ => 430,
        }
    );
    if ACTIVE.has_legacy_item_globals() {
        env.exec(
            r#"
            assert(select('#', C_Item.GetItemSpell(225654)) == 0)
            assert(select('#', C_Item.GetItemSpell(285065)) == 0)
        "#,
        )
        .unwrap();
    } else {
        env.exec(
            r#"
            local name, id = C_Item.GetItemSpell(225654)
            assert(name == 'Ascension' and id == 455482)
        "#,
        )
        .unwrap();
    }
    if ACTIVE == ClientProfile::Ptr {
        env.exec("assert(select('#', C_Item.GetItemSpell(285065)) == 0)")
            .unwrap();
    }
    match ACTIVE {
        ClientProfile::Wrath => env.exec(
            r#"
            local name, id = C_Item.GetItemSpell(33447)
            assert(name == 'Healing Potion' and id == 43185)
            assert(select('#', C_Item.GetItemSpell(33447)) == 2)
            assert(select('#', C_Item.GetItemSpell(76097)) == 0)
            assert(select('#', C_Item.GetItemSpell(233986)) == 0)
        "#,
        ),
        ClientProfile::Mists => env.exec(
            r#"
            local name, id = C_Item.GetItemSpell(76097)
            assert(name == 'Healing Potion' and id == 105708)
            assert(select('#', C_Item.GetItemSpell(76097)) == 2)
            assert(select('#', C_Item.GetItemSpell(233986)) == 0)
        "#,
        ),
        ClientProfile::Era | ClientProfile::Anniversary => env.exec(
            r#"
            local name, id = C_Item.GetItemSpell(233986)
            assert(name == 'Obsidian Bomb' and id == 1213946)
            assert(select('#', C_Item.GetItemSpell(233986)) == 2)
            assert(select('#', C_Item.GetItemSpell(33447)) == 0)
            assert(select('#', C_Item.GetItemSpell(76097)) == 0)
        "#,
        ),
        _ => Ok(()),
    }
    .unwrap();
}

#[test]
fn item_use_spell_legacy_global_preserves_results_arity_and_argument_validation() {
    let env = WowLuaEnv::new().unwrap();
    if !ACTIVE.has_legacy_item_globals() {
        env.exec("assert(rawget(_G, 'GetItemSpell') == nil)")
            .unwrap();
        return;
    }
    env.exec(
        r#"
        assert(type(rawget(_G, 'GetItemSpell')) == 'function')
        for _, item in ipairs({6948, '6948', 'item:6948', 'Hearthstone'}) do
            local name, id = GetItemSpell(item)
            assert(name == 'Hearthstone' and id == 8690, tostring(item))
            assert(select('#', GetItemSpell(item)) == 2)
        end
        local name, id = GetItemSpell(159)
        assert(name == 'Drink' and id == 430)
        for _, item in ipairs({25, 19379, 225654, 285065, 4294967294, 'not an item'}) do
            assert(select('#', GetItemSpell(item)) == 0, tostring(item))
        end
        for _, query in ipairs({function() GetItemSpell() end,
            function() GetItemSpell(false) end,
            function() GetItemSpell({}) end}) do
            local ok, message = pcall(query)
            assert(not ok and message:find('itemInfo', 1, true))
        end
    "#,
    )
    .unwrap();
}

#[cfg(feature = "profile-retail")]
#[test]
fn item_use_spell_retail_catalog_preserves_existing_and_new_associations() {
    use wow_ui_sim::lua_api::state::EquippedItem;

    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        -- One profile catalog contains both the existing and newer associations.
        local name, id = C_Item.GetItemSpell(278743)
        assert(name == 'Learning' and id == 55884)
        local name, id = C_Item.GetItemSpell(34812)
        assert(name == "Crafty's Ultra-Advanced Proto-Typical Shortening Blaster" and id == 51912)
        -- Preserve this existing association even though the profile CSV export lacks it.
        local name, id = C_Item.GetItemSpell(285065)
        assert(name == 'Mythical Soul Fragment' and id == 1317439)
    "#,
    )
    .unwrap();
    env.state().borrow_mut().player.equipped_items.insert(
        1,
        EquippedItem {
            item_id: 278743,
            enchant_id: 3290,
            gem_ids: [0; 3],
            applied_transmog: None,
        },
    );
    env.exec(
        "local name, id = C_Item.GetItemSpell(278743); assert(name == 'Learning' and id == 55884)",
    )
    .unwrap();
}

#[cfg(feature = "client-wowforever")]
#[test]
fn item_use_spell_forever_preserves_use_effect_with_shared_non_use_slot() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        for _, query in ipairs({C_Item.GetItemSpell, GetItemSpell}) do
            for _, item in ipairs({239695, 'item:239695'}) do
                local name, id = query(item)
                assert(name == 'Learning' and id == 55884)
                assert(select('#', query(item)) == 2)
            end
            local name, id = query(6948)
            assert(name == 'Hearthstone' and id == 8690)
            assert(select('#', query(19379)) == 0)
        end
        "#,
    )
    .unwrap();
}

#[cfg(feature = "profile-retail")]
#[test]
fn item_use_spell_unnamed_selection_suppresses_enchantment_fallback() {
    use wow_ui_sim::lua_api::state::EquippedItem;

    let env = WowLuaEnv::new().unwrap();
    {
        let state = env.state();
        let mut sim = state.borrow_mut();
        // The catalog retains spell 7291 for item 1165 without an available name.
        sim.player.equipped_items.insert(
            1,
            EquippedItem {
                item_id: 1165,
                enchant_id: 3290,
                gem_ids: [0; 3],
                applied_transmog: None,
            },
        );
        sim.item_use_eligibility.insert((1165, 7291), true);
    }
    env.exec(
        r#"
        assert(select('#', C_Item.GetItemSpell(1165)) == 0)
        local usable, insufficient = C_Item.IsUsableItem(1165)
        assert(usable and not insufficient)
        "#,
    )
    .unwrap();
}

#[test]
fn item_use_spell_keeps_equipped_enchantment_fallback_scoped_to_retail() {
    use wow_ui_sim::lua_api::state::EquippedItem;

    let env = WowLuaEnv::new().unwrap();
    env.exec("assert(select('#', C_Item.GetItemSpell(211993)) == 0)")
        .unwrap();
    env.state().borrow_mut().player.equipped_items.insert(
        1,
        EquippedItem {
            item_id: 211993,
            enchant_id: 3290,
            gem_ids: [0; 3],
            applied_transmog: None,
        },
    );
    if cfg!(feature = "profile-retail") {
        let (name, id): (String, i64) = env.eval("return C_Item.GetItemSpell(211993)").unwrap();
        assert_eq!(name, "Spring Loaded Cape");
        assert_eq!(id, 52641);
    } else {
        env.exec("assert(select('#', C_Item.GetItemSpell(211993)) == 0)")
            .unwrap();
    }
    env.state().borrow_mut().player.equipped_items.clear();
    env.exec("assert(select('#', C_Item.GetItemSpell(211993)) == 0)")
        .unwrap();
}

#[cfg(feature = "client-wrath")]
#[test]
fn item_use_spell_cached_wrath_chat_consumer_chooses_use_or_equip() {
    use std::path::Path;
    use wow_ui_sim::toc::TocFile;
    use wow_ui_sim::xml::{XmlElement, parse_xml_file};

    let Some(toc_path) = wow_ui_sim::client_profile::blizzard_ui_framexml_toc() else {
        eprintln!("skipping original Wrath chat consumer: active FrameXML cache is absent");
        return;
    };
    let toc = TocFile::from_file(&toc_path).expect("parse active Wrath FrameXML TOC");
    assert!(
        toc.files
            .iter()
            .any(|path| path == Path::new("ChatFrame.xml")),
        "active FrameXML TOC must select ChatFrame.xml"
    );
    let directory = toc_path.parent().unwrap();
    let xml = parse_xml_file(&directory.join("ChatFrame.xml")).unwrap();
    assert!(xml.elements.iter().any(|element| {
        matches!(element, XmlElement::Script(script) if script.file.as_deref() == Some("ChatFrame.lua"))
    }));
    let source = std::fs::read_to_string(directory.join("ChatFrame.lua"))
        .expect("read entire unchanged selected Wrath chat consumer");
    let env = WowLuaEnv::new().unwrap();
    env.loader_env()
        .exec(&source)
        .expect("load entire unchanged Wrath ChatFrame.lua");
    env.exec(
        r#"
        MacroFrameText = CreateFrame('EditBox')
        MacroFrameText:Show()
        assert(MacroFrameText:IsVisible())
        for _, row in ipairs({{6948, SLASH_USE1}, {19379, SLASH_EQUIP1}}) do
            local item, command = row[1], assert(row[2])
            MacroFrameText:SetText('')
            MacroFrameText:SetCursorPosition(0)
            local itemName = assert(GetItemInfo(item))
            assert(ChatEdit_InsertLink('item:' .. item))
            assert(MacroFrameText:GetText() == command .. ' ' .. itemName .. '\n',
                MacroFrameText:GetText())
        end
    "#,
    )
    .expect("original chat consumer must distinguish an activated item from a passive trinket");
}
