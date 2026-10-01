#![cfg(all(
    feature = "retail-12-0-5",
    any(feature = "profile-retail", feature = "client-ptr")
))]

use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::lua_api::state::{AppearanceSourceInfo, PvpBrawlInfo, ViewedOutfitSlotInfo};

// Arbitrary test-local inputs, not claims about live game records.
fn brawl_fixture() -> PvpBrawlInfo {
    PvpBrawlInfo {
        brawl_id: 701,
        name: "Fixture brawl".into(),
        short_description: "Short fixture".into(),
        long_description: "Long fixture".into(),
        can_queue: true,
        min_level: 20,
        max_level: 80,
        groups_allowed: false,
        cross_faction_allowed: true,
        time_left_until_next_change: Some(91.5),
        brawl_type: 2,
        map_names: vec!["First map".into(), "Second map".into()],
        includes_all_arenas: false,
        min_item_level: 123.5,
        should_hide_reward_icon: true,
    }
}

fn appearance_fixture() -> AppearanceSourceInfo {
    AppearanceSourceInfo {
        category: 4,
        item_appearance_id: 901,
        can_have_illusion: true,
        icon: 902,
        is_collected: false,
        item_link: "item:fixture-a".into(),
        transmoglink: "transmog:fixture-a".into(),
        source_type: Some(3),
        item_subclass: 7,
        ignore_model_attachment_checks_for_illusion: true,
    }
}

fn slot_fixture() -> ViewedOutfitSlotInfo {
    ViewedOutfitSlotInfo {
        transmog_id: 801,
        display_type: 1,
        is_transmogrified: true,
        has_pending: false,
        is_pending_collected: true,
        can_transmogrify: false,
        warning: 2,
        warning_text: "Fixture warning".into(),
        error: 3,
        error_text: "Fixture error".into(),
        texture: Some(802),
        sheathe_category: 1,
    }
}

#[test]
fn structure_brawl_complete_record_and_reward_icon_values() {
    let env = WowLuaEnv::new().unwrap();
    env.state().borrow_mut().active_brawl = Some(brawl_fixture());
    env.exec(
        r#"
        local query = C_PvP.GetActiveBrawlInfo
        local info = query()
        assert(type(info) == 'table', 'populated active brawl must return a table')
        assert(select('#', query()) == 1)
        assert(info.brawlID == 701 and info.name == 'Fixture brawl')
        assert(info.shortDescription == 'Short fixture' and info.longDescription == 'Long fixture')
        assert(info.canQueue == true and info.groupsAllowed == false)
        assert(info.crossFactionAllowed == true and info.includesAllArenas == false)
        assert(info.shouldHideRewardIcon == true)
        assert(info.minLevel == 20 and info.maxLevel == 80 and info.brawlType == 2)
        assert(info.minItemLevel == 123.5 and info.timeLeftUntilNextChange == 91.5)
        assert(type(info.mapNames) == 'table' and #info.mapNames == 2)
        assert(info.mapNames[1] == 'First map' and info.mapNames[2] == 'Second map')
        local count = 0
        for _ in pairs(info) do count = count + 1 end
        assert(count == 15)
        "#,
    )
    .unwrap();
    let mut other = brawl_fixture();
    other.brawl_id = 702;
    other.should_hide_reward_icon = false;
    other.time_left_until_next_change = None;
    other.map_names.clear();
    env.state().borrow_mut().active_brawl = Some(other);
    env.exec(
        r#"
        local info = C_PvP.GetActiveBrawlInfo()
        assert(type(info) == 'table', 'replacement brawl must return a table')
        assert(info.brawlID == 702 and info.shouldHideRewardIcon == false)
        assert(info.timeLeftUntilNextChange == nil and next(info.mapNames) == nil)
        "#,
    )
    .unwrap();
}

#[test]
fn structure_brawl_snapshots_do_not_mutate_inputs() {
    let env = WowLuaEnv::new().unwrap();
    env.state().borrow_mut().active_brawl = Some(brawl_fixture());
    env.exec(
        r#"
        local query = C_PvP.GetActiveBrawlInfo
        local first, second = query(), query()
        assert(type(first) == 'table' and type(second) == 'table', 'populated brawl tables required')
        first.brawlID = -1
        first.shouldHideRewardIcon = false
        first.mapNames[1] = 'changed'
        local fresh = query()
        assert(type(fresh) == 'table')
        assert(second.brawlID == 701 and fresh.brawlID == 701)
        assert(second.shouldHideRewardIcon == true and fresh.shouldHideRewardIcon == true)
        assert(second.mapNames[1] == 'First map' and fresh.mapNames[1] == 'First map')
        "#,
    )
    .unwrap();
    let state = env.state();
    let state = state.borrow();
    assert_eq!(state.active_brawl.as_ref().unwrap().brawl_id, 701);
    assert_eq!(
        state.active_brawl.as_ref().unwrap().map_names[0],
        "First map"
    );
}

#[test]
fn structure_brawl_empty_and_removed_input_return_one_nil() {
    let env = WowLuaEnv::new().unwrap();
    assert!(env.state().borrow().active_brawl.is_none());
    env.exec("assert(select('#', C_PvP.GetActiveBrawlInfo()) == 1); assert(C_PvP.GetActiveBrawlInfo() == nil)")
        .unwrap();
    env.state().borrow_mut().active_brawl = Some(brawl_fixture());
    env.exec("assert(type(C_PvP.GetActiveBrawlInfo()) == 'table', 'populated brawl required')")
        .unwrap();
    env.state().borrow_mut().active_brawl = None;
    env.exec("assert(select('#', C_PvP.GetActiveBrawlInfo()) == 1); assert(C_PvP.GetActiveBrawlInfo() == nil)")
        .unwrap();
}

#[test]
fn structure_appearance_complete_records_and_illusion_values() {
    let env = WowLuaEnv::new().unwrap();
    let mut other = appearance_fixture();
    other.item_appearance_id = 911;
    other.ignore_model_attachment_checks_for_illusion = false;
    other.source_type = None;
    {
        let state = env.state();
        let mut state = state.borrow_mut();
        state
            .transmog_appearance_sources
            .insert(501, appearance_fixture());
        state.transmog_appearance_sources.insert(502, other);
    }
    env.exec(
        r#"
        local query = C_TransmogCollection.GetAppearanceSourceInfo
        local a, b = query(501), query(502)
        assert(type(a) == 'table' and type(b) == 'table', 'populated appearance sources must return tables')
        assert(select('#', query(501)) == 1)
        assert(a.category == 4 and a.itemAppearanceID == 901)
        assert(a.canHaveIllusion == true and a.icon == 902 and a.isCollected == false)
        assert(a.itemLink == 'item:fixture-a' and a.transmoglink == 'transmog:fixture-a')
        assert(a.sourceType == 3 and a.itemSubclass == 7)
        assert(a.ignoreModelAttachmentChecksForIllusion == true)
        assert(b.itemAppearanceID == 911 and b.sourceType == nil)
        assert(b.ignoreModelAttachmentChecksForIllusion == false)
        local count = 0
        for _ in pairs(a) do count = count + 1 end
        assert(count == 10)
        "#,
    )
    .unwrap();
}

#[test]
fn structure_appearance_keys_snapshots_and_secret_arguments() {
    let env = WowLuaEnv::new().unwrap();
    env.state()
        .borrow_mut()
        .transmog_appearance_sources
        .insert(501, appearance_fixture());
    env.exec(
        r#"
        local query = C_TransmogCollection.GetAppearanceSourceInfo
        local first = query(501)
        assert(type(first) == 'table', 'populated source must return a table')
        first.itemAppearanceID = -1
        first.ignoreModelAttachmentChecksForIllusion = false
        local fresh = query(501)
        assert(type(fresh) == 'table' and fresh.itemAppearanceID == 901)
        assert(fresh.ignoreModelAttachmentChecksForIllusion == true)
        assert(select('#', query(901)) == 0, 'appearance ID must not select source ID')
        local secret = secretwrap(501)
        local info = query(secret)
        assert(type(info) == 'table' and info.itemAppearanceID == 901)
        local function addon() return query(secret) end
        debug.setobjecttaint(addon, 'StructureInputsProbe')
        assert(not pcall(addon), 'AllowedWhenUntainted rejects tainted secret arguments')
        local function ordinary() return query(501) end
        debug.setobjecttaint(ordinary, 'StructureInputsProbe')
        local ok, result = pcall(ordinary)
        assert(ok and type(result) == 'table' and result.itemAppearanceID == 901)
        "#,
    )
    .unwrap();
    assert_eq!(
        env.state().borrow().transmog_appearance_sources[&501].item_appearance_id,
        901
    );
}

#[test]
fn structure_appearance_empty_and_removed_inputs_return_nothing() {
    let env = WowLuaEnv::new().unwrap();
    assert!(env.state().borrow().transmog_appearance_sources.is_empty());
    env.exec("assert(select('#', C_TransmogCollection.GetAppearanceSourceInfo(501)) == 0)")
        .unwrap();
    env.state()
        .borrow_mut()
        .transmog_appearance_sources
        .insert(501, appearance_fixture());
    env.exec("assert(type(C_TransmogCollection.GetAppearanceSourceInfo(501)) == 'table', 'populated source required')")
        .unwrap();
    env.state()
        .borrow_mut()
        .transmog_appearance_sources
        .remove(&501);
    env.exec("assert(select('#', C_TransmogCollection.GetAppearanceSourceInfo(501)) == 0)")
        .unwrap();
}

#[test]
fn structure_viewed_slot_complete_records_and_sheathe_values() {
    let env = WowLuaEnv::new().unwrap();
    let mut other = slot_fixture();
    other.transmog_id = 811;
    other.sheathe_category = 2;
    other.texture = None;
    {
        let state = env.state();
        let mut state = state.borrow_mut();
        state.viewed_outfit_slots.insert((1, 0, 0), slot_fixture());
        state.viewed_outfit_slots.insert((1, 0, 1), other);
    }
    env.exec(
        r#"
        local query = C_TransmogOutfitInfo.GetViewedOutfitSlotInfo
        local a, b = query(1, 0, 0), query(1, 0, 1)
        assert(type(a) == 'table' and type(b) == 'table', 'populated viewed slots must return tables')
        assert(select('#', query(1, 0, 0)) == 1)
        assert(a.transmogID == 801 and a.displayType == 1)
        assert(a.isTransmogrified == true and a.hasPending == false)
        assert(a.isPendingCollected == true and a.canTransmogrify == false)
        assert(a.warning == 2 and a.warningText == 'Fixture warning')
        assert(a.error == 3 and a.errorText == 'Fixture error')
        assert(a.texture == 802 and a.sheatheCategory == 1)
        assert(b.transmogID == 811 and b.texture == nil and b.sheatheCategory == 2)
        local count = 0
        for _ in pairs(a) do count = count + 1 end
        assert(count == 12)
        "#,
    )
    .unwrap();
}

#[test]
fn structure_viewed_slot_tuple_isolation_snapshots_and_secrets() {
    let env = WowLuaEnv::new().unwrap();
    env.state()
        .borrow_mut()
        .viewed_outfit_slots
        .insert((1, 0, 0), slot_fixture());
    env.exec(
        r#"
        local query = C_TransmogOutfitInfo.GetViewedOutfitSlotInfo
        local first = query(1, 0, 0)
        assert(type(first) == 'table', 'populated viewed slot must return a table')
        first.transmogID = -1
        first.sheatheCategory = 2
        local fresh = query(1, 0, 0)
        assert(type(fresh) == 'table' and fresh.transmogID == 801 and fresh.sheatheCategory == 1)
        assert(select('#', query(2, 0, 0)) == 0)
        assert(select('#', query(1, 1, 0)) == 0)
        assert(select('#', query(1, 0, 1)) == 0)
        for position = 1, 3 do
            local args = {1, 0, 0}
            args[position] = secretwrap(args[position])
            local info = query(unpack(args))
            assert(type(info) == 'table' and info.transmogID == 801)
            local function addon() return query(unpack(args)) end
            debug.setobjecttaint(addon, 'StructureInputsProbe')
            assert(not pcall(addon), 'each secret enum argument requires untainted caller')
        end
        local function ordinary() return query(1, 0, 0) end
        debug.setobjecttaint(ordinary, 'StructureInputsProbe')
        local ok, result = pcall(ordinary)
        assert(ok and type(result) == 'table' and result.transmogID == 801)
        "#,
    )
    .unwrap();
    assert_eq!(
        env.state().borrow().viewed_outfit_slots[&(1, 0, 0)].sheathe_category,
        1
    );
}

#[test]
fn structure_viewed_slot_empty_and_removed_inputs_return_nothing() {
    let env = WowLuaEnv::new().unwrap();
    assert!(env.state().borrow().viewed_outfit_slots.is_empty());
    env.exec("assert(select('#', C_TransmogOutfitInfo.GetViewedOutfitSlotInfo(1, 0, 0)) == 0)")
        .unwrap();
    env.state()
        .borrow_mut()
        .viewed_outfit_slots
        .insert((1, 0, 0), slot_fixture());
    env.exec("assert(type(C_TransmogOutfitInfo.GetViewedOutfitSlotInfo(1, 0, 0)) == 'table', 'populated slot required')")
        .unwrap();
    env.state()
        .borrow_mut()
        .viewed_outfit_slots
        .remove(&(1, 0, 0));
    env.exec("assert(select('#', C_TransmogOutfitInfo.GetViewedOutfitSlotInfo(1, 0, 0)) == 0)")
        .unwrap();
}
