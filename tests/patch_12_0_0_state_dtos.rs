use super::patch_12_0_0_struct_shapes::assert_shape;
use wow_ui_sim::lua_api::WowLuaEnv;

#[test]
fn major_faction_parent_and_optional_companion() {
    let env = WowLuaEnv::new().unwrap();
    {
        let state = env.state(); let mut state = state.borrow_mut();
        let first = state.major_factions.values_mut().next().unwrap();
        first.description = "Fixture description".into(); first.player_companion_id = Some(77);
        first.highlights = vec![wow_ui_sim::c_api::c_major_factions::RenownHighlightInfo {
            title: "First".into(), description: "Highlight".into(), level: 7
        }];
        let mut other = first.clone(); other.faction_id = 9999;
        other.description = "Second".into(); other.highlights.clear(); other.player_companion_id = None;
        state.major_factions.insert(9999, other);
    }
    env.exec(r#"local id
        for _, candidate in ipairs(C_MajorFactions.GetMajorFactionIDs()) do
            local info = C_MajorFactions.GetMajorFactionData(candidate)
            if info.playerCompanionID == 77 then id = candidate end
        end
        assert(id)
        local a, b = C_MajorFactions.GetMajorFactionData(id), C_MajorFactions.GetMajorFactionData(id)
        assert(a.description == 'Fixture description' and a.highlights[1].level == 7)
        assert(#a.highlights == 2 and a.highlights[2].title == 'Second highlight' and a.highlights[2].level == 9)
        a.highlights[1].title = 'mutation'; assert(b.highlights[1].title == 'First')
        assert(C_MajorFactions.GetMajorFactionData(id).highlights[1].description == 'Highlight')
        assert(C_MajorFactions.GetMajorFactionData(9999).playerCompanionID == nil)
        assert(C_MajorFactions.GetMajorFactionData(9999).description == 'Second')"#).unwrap();
    assert_shape(&env, "MajorFactionsDocumentation.lua", "MajorFactionData", "return C_MajorFactions.GetMajorFactionData(C_MajorFactions.GetMajorFactionIDs()[1])");
}

#[test]
fn item_interaction_parent_and_combined_flags() {
    let env = WowLuaEnv::new().unwrap();
    env.exec("assert(C_ItemInteraction.GetItemInteractionInfo() == nil)").unwrap();
    env.state().borrow_mut().item_interaction = Some(wow_ui_sim::c_api::c_item_interaction::ItemInteractionInfo {
        texture_kit: "fixture".into(), title_text: "First".into(), tutorial_text: "Tutorial".into(),
        button_text: "Apply".into(), open_sound_kit_id: 11, close_sound_kit_id: 12,
        interaction_type: 1, flags: 17, description: Some("Description".into()),
        cost: Some(42.5), ..Default::default()
    });
    env.exec(r#"local a, b = C_ItemInteraction.GetItemInteractionInfo(), C_ItemInteraction.GetItemInteractionInfo()
        assert(a.flags == Enum.UIItemInteractionFlags.DisplayWithInset + Enum.UIItemInteractionFlags.AddCurrency)
        a.flags = 0; assert(b.flags == 17 and C_ItemInteraction.GetItemInteractionInfo().flags == 17)
        assert(b.cost == 42.5 and b.buttonTooltip == nil)"#).unwrap();
    assert_shape(&env, "ItemInteractionUIDocumentation.lua", "ItemInteractionFrameInfo", "return C_ItemInteraction.GetItemInteractionInfo()");
}

#[test]
fn scheduled_event_parent_and_independent_snapshots() {
    let env = WowLuaEnv::new().unwrap();
    assert_shape(&env, "EventSchedulerUIDocumentation.lua", "ScheduledEventInfo", "return C_EventScheduler.GetScheduledEvents()[1]");
    assert_shape(&env, "EventSchedulerUIDocumentation.lua", "EventDisplayInfo", "return C_EventScheduler.GetScheduledEvents()[1].displayInfo");
    env.exec(r#"
        local first = C_EventScheduler.GetScheduledEvents()
        local second = C_EventScheduler.GetScheduledEvents()
        first[1].displayInfo.hideDescription = true
        first[1].eventID = -1
        assert(second[1].eventID == 2001 and second[1].displayInfo.hideDescription == false,
            'scheduled events must be independent snapshots')
        assert(second[2].eventID == 2002)
        C_EventScheduler._state.scheduledEvents[1].eventID = 3001
        C_EventScheduler._state.scheduledEvents[1].displayInfo.overrideAtlas = 'FixtureAtlas'
        local updated = C_EventScheduler.GetScheduledEvents()
        assert(updated[1].eventID == 3001 and updated[2].eventID == 2002)
        assert(updated[1].displayInfo.overrideAtlas == 'FixtureAtlas')
        C_EventScheduler._state.scheduledEvents[1].displayInfo.hideTimeLeft = true
        C_EventScheduler._state.scheduledEvents[1].displayInfo.overrideTooltipWidgetSetID = 45
        local display = C_EventScheduler.GetScheduledEvents()[1].displayInfo
        assert(display.hideTimeLeft and display.overrideTooltipWidgetSetID == 45)
        assert(second[1].displayInfo.overrideTooltipWidgetSetID == nil)
        -- Host delivers ongoing/upcoming state; getters do not synthesize events.
        table.remove(C_EventScheduler._state.scheduledEvents, 1)
        local upcoming = C_EventScheduler.GetScheduledEvents()
        assert(#upcoming == 1 and upcoming[1].eventID == 2002)
        assert(#second == 2 and second[1].eventID == 2001)
        assert(second[1].eventID == 2001 and second[1].displayInfo.overrideAtlas == nil)
    "#).unwrap();
}

#[test]
fn transmog_set_parent_variants_and_flags() {
    use wow_ui_sim::c_api::c_transmog_sets::TransmogSetInfo;
    let env = WowLuaEnv::new().unwrap();
    env.state().borrow_mut().transmog_sets.entries = vec![
        TransmogSetInfo { set_id: 501, name: "Base fixture".into(), grant_as_preceding_variant: true,
            valid_for_character: true, ..Default::default() },
        TransmogSetInfo { set_id: 502, base_set_id: Some(501), name: "Variant fixture".into(),
            grant_as_preceding_variant: false, valid_for_character: true, ..Default::default() },
    ];
    assert_shape(&env, "TransmogSetsDocumentation.lua", "TransmogSetInfo", "return C_TransmogSets.GetSetInfo(501)");
    env.exec(r#"
        local base = C_TransmogSets.GetSetInfo(501)
        local variant = C_TransmogSets.GetSetInfo(502)
        assert(base.setID == 501 and base.grantAsPrecedingVariant == true)
        assert(variant.setID == 502 and variant.grantAsPrecedingVariant == false)
        local variants = C_TransmogSets.GetVariantSets(501)
        assert(#variants == 1 and variants[1].setID == 502)
        variants[1].grantAsPrecedingVariant = true
        base.name = 'mutation'
        assert(C_TransmogSets.GetSetInfo(501).name == 'Base fixture')
        assert(C_TransmogSets.GetSetInfo(502).grantAsPrecedingVariant == false)
        assert(C_TransmogSets.GetSetInfo(999) == nil)
    "#).unwrap();
}
