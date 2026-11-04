//! Forever 70707 declarations exercised through the public Lua endpoints.

use crate::lua_api::WowLuaEnv;

#[test]
fn merchant_info_reads_supplied_inventory() {
    let env = WowLuaEnv::new().unwrap();
    env.state().borrow_mut().merchant_items = vec![6948, 229181];
    let actual: (String, String, bool) = env
        .eval(
            r#"
        local first = C_MerchantFrame.GetItemInfo(1)
        local second = C_MerchantFrame.GetItemInfo(2)
        return first.name, second.name, C_MerchantFrame.GetItemInfo(3) == nil
    "#,
        )
        .unwrap();
    assert_eq!(
        actual,
        ("Hearthstone".into(), "Ordained Forge Maul".into(), true)
    );
    env.state().borrow_mut().merchant_items = vec![229181];
    assert_eq!(
        env.eval::<String>("return C_MerchantFrame.GetItemInfo(1).name")
            .unwrap(),
        "Ordained Forge Maul"
    );
}

#[test]
fn transmog_sets_query_supplied_records() {
    use super::c_transmog_sets::TransmogSetInfo;
    let env = WowLuaEnv::new().unwrap();
    env.state().borrow_mut().transmog_sets.entries = vec![
        TransmogSetInfo {
            set_id: 91,
            name: "Hidden set".into(),
            valid_for_character: false,
            ..Default::default()
        },
        TransmogSetInfo {
            set_id: 305,
            name: "Available set".into(),
            valid_for_character: true,
            ..Default::default()
        },
    ];
    let actual: (String, String, bool) = env
        .eval(
            r#"
        return C_TransmogSets.GetSetInfo(91).name,
            C_TransmogSets.GetSetInfo(305).name,
            C_TransmogSets.HasAvailableSets()
    "#,
        )
        .unwrap();
    assert_eq!(actual, ("Hidden set".into(), "Available set".into(), true));
    env.state().borrow_mut().transmog_sets.entries[1].valid_for_character = false;
    assert!(
        !env.eval::<bool>("return C_TransmogSets.HasAvailableSets()")
            .unwrap()
    );
}

#[test]
fn outfit_info_uses_existing_forever_catalog_and_emulated_empty() {
    use super::outfit_entry::OutfitEntry;
    let env = WowLuaEnv::new().unwrap();
    assert_eq!(
        env.eval::<i32>("return select('#', C_TransmogOutfitInfo.GetOutfitInfo(91))")
            .unwrap(),
        0
    );
    env.state().borrow_mut().forever_outfits = Some(None);
    assert_eq!(
        env.eval::<i32>("return select('#', C_TransmogOutfitInfo.GetOutfitInfo(91))")
            .unwrap(),
        0
    );
    env.state().borrow_mut().forever_outfits = Some(Some(vec![
        OutfitEntry {
            outfit_id: 91,
            name: "Copper".into(),
            situation_categories: vec!["combat".into()],
            icon: 123,
            is_event_outfit: false,
            is_disabled: true,
            player_facing_outfit_index: 7,
        },
        OutfitEntry {
            outfit_id: 305,
            name: "Silver".into(),
            situation_categories: vec!["rest".into()],
            icon: 456,
            is_event_outfit: true,
            is_disabled: false,
            player_facing_outfit_index: 42,
        },
    ]));
    let actual: (String, String, i32, i32, bool) = env.eval(r#"
        local first = C_TransmogOutfitInfo.GetOutfitInfo(91)
        local second = C_TransmogOutfitInfo.GetOutfitInfo(305)
        assert(first.situationCategories[1] == 'combat' and first.icon == 123 and first.isDisabled and not first.isEventOutfit)
        assert(second.situationCategories[1] == 'rest' and second.icon == 456 and not second.isDisabled and second.isEventOutfit)
        local list = C_TransmogOutfitInfo.GetOutfitsInfo()
        assert(list[1].outfitID == first.outfitID and list[2].outfitID == second.outfitID)
        first.name = 'changed'
        first.situationCategories[1] = 'changed'
        assert(C_TransmogOutfitInfo.GetOutfitInfo(91).situationCategories[1] == 'combat')
        return C_TransmogOutfitInfo.GetOutfitInfo(91).name, second.name,
            first.playerFacingOutfitIndex, second.playerFacingOutfitIndex,
            C_TransmogOutfitInfo.GetOutfitInfo(7) == nil
    "#).unwrap();
    assert_eq!(actual, ("Copper".into(), "Silver".into(), 7, 42, true));
}

#[test]
fn group_buffs_publish_six_supplied_fields_and_fresh_snapshots() {
    use super::c_cooldown_viewer::GroupBuffItem;
    let env = WowLuaEnv::new().unwrap();
    assert_eq!(
        env.eval::<i32>("return #C_CooldownViewer.GetGroupBuffItems()")
            .unwrap(),
        0
    );
    env.state().borrow_mut().cooldown_viewer_group_buffs = vec![
        GroupBuffItem {
            spell_id: 91,
            display_spell_id: 92,
            name: "Copper".into(),
            icon_id: 123,
            flags: 1,
            is_known: false,
        },
        GroupBuffItem {
            spell_id: 305,
            display_spell_id: 306,
            name: "Silver".into(),
            icon_id: 456,
            flags: 2,
            is_known: true,
        },
    ];
    let actual: (String, String, i32) = env.eval(r#"
        local items = C_CooldownViewer.GetGroupBuffItems()
        assert(#items == 2)
        assert(items[1].spellID == 91 and items[1].displaySpellID == 92 and items[1].iconID == 123 and items[1].flags == 1 and not items[1].isKnown)
        assert(items[2].spellID == 305 and items[2].displaySpellID == 306 and items[2].iconID == 456 and items[2].flags == 2 and items[2].isKnown)
        local count = 0
        for _ in pairs(items[1]) do count = count + 1 end
        items[1].name = 'changed'
        return C_CooldownViewer.GetGroupBuffItems()[1].name, items[2].name, count
    "#).unwrap();
    assert_eq!(actual, ("Copper".into(), "Silver".into(), 6));
    env.state().borrow_mut().cooldown_viewer_group_buffs.clear();
    assert_eq!(
        env.eval::<i32>("return #C_CooldownViewer.GetGroupBuffItems()")
            .unwrap(),
        0
    );
}

#[test]
fn forbidden_aspects_publish_pinned_fifteen_bits() {
    let env = WowLuaEnv::new().unwrap();
    let actual: bool = env.eval(r#"
        local names = {'SetToDefaults', 'ScriptBindings', 'UntrustedScriptExecution',
            'UntrustedLayoutScriptExecution', 'EventRegistrations', 'AlwaysPropagateInput',
            'ScriptedInput', 'QueryFocus', 'ChangeAnimationTarget', 'RemoveSecretAspects',
            'ChangeParent', 'SetTexture', 'QueryRotation', 'QueryAnimationProgress', 'AddAnimations'}
        for index, name in ipairs(names) do
            if Enum.ForbiddenAspect[name] ~= 2 ^ (index - 1) then return false end
        end
        return Enum.ForbiddenAspectMeta.MinValue == 1
            and Enum.ForbiddenAspectMeta.MaxValue == 16384
            and Enum.ForbiddenAspectMeta.NumValues == 15
    "#).unwrap();
    assert!(actual);
}
