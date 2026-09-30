#![cfg(feature = "retail-12-0-5")]

use wow_ui_sim::c_api::c_transmog_outfit_info::OutfitEntry;
use wow_ui_sim::lua_api::WowLuaEnv;

fn seeded_env() -> WowLuaEnv {
    let env = WowLuaEnv::new().unwrap();
    env.state().borrow_mut().transmog_outfit_catalog.entries = vec![
        OutfitEntry {
            outfit_id: 91,
            name: "rAiD Set".into(),
            situation_categories: vec!["Dungeon".into(), "Raid".into()],
            icon: 135771,
            is_event_outfit: true,
            is_disabled: false,
            player_facing_outfit_index: 7,
        },
        OutfitEntry {
            outfit_id: 305,
            name: "Travel".into(),
            situation_categories: vec![],
            icon: 132489,
            is_event_outfit: false,
            is_disabled: true,
            player_facing_outfit_index: 42,
        },
    ];
    env
}

#[test]
fn catalog_queries_share_records_and_complete_schema() {
    let env = seeded_env();
    env.exec(
        r#"
        local api = C_TransmogOutfitInfo
        local byID = api.GetOutfitInfo(91)
        local byName = api.GetOutfitInfoByName("RAID sET")
        local byIndex = api.GetOutfitInfoByPlayerFacingIndex(7)
        local all = api.GetOutfitsInfo()
        assert(#all == 2)
        for _, entry in ipairs({byID, byName, byIndex, all[1]}) do
            assert(entry.outfitID == 91 and entry.name == "rAiD Set")
            assert(entry.icon == 135771)
            assert(entry.isEventOutfit == true and entry.isDisabled == false)
            assert(entry.playerFacingOutfitIndex == 7)
            assert(#entry.situationCategories == 2)
            assert(entry.situationCategories[1] == "Dungeon")
            assert(entry.situationCategories[2] == "Raid")
            local count = 0
            for _ in pairs(entry) do count = count + 1 end
            assert(count == 7, "exact official schema")
        end
        local other = api.GetOutfitInfoByPlayerFacingIndex(42)
        assert(other.outfitID == 305 and other.name == "Travel")
        assert(other.icon == 132489 and other.isDisabled and not other.isEventOutfit)
        assert(next(other.situationCategories) == nil)
        assert(api.GetOutfitInfoByName("travel").outfitID == 305)
        assert(api.GetOutfitInfo(305).playerFacingOutfitIndex == 42)
        assert(all[2].outfitID == 305)
        assert(api.GetOutfitInfo(7) == nil)
        assert(api.GetOutfitInfoByPlayerFacingIndex(91) == nil)
        assert(api.GetOutfitInfoByPlayerFacingIndex(1) == nil)
        assert(api.GetOutfitInfoByName("missing") == nil)
        assert(select('#', api.GetOutfitInfo(999)) == 0)
        assert(select('#', api.GetOutfitInfoByName("missing")) == 0)
        assert(select('#', api.GetOutfitInfoByPlayerFacingIndex(999)) == 0)
        "#,
    )
    .unwrap();
}

#[test]
fn returned_catalog_tables_never_alias_state_or_each_other() {
    let env = seeded_env();
    env.exec(
        r#"
        local api = C_TransmogOutfitInfo
        local first = api.GetOutfitInfo(91)
        local second = api.GetOutfitInfoByName("raid set")
        local third = api.GetOutfitInfoByPlayerFacingIndex(7)
        local all = api.GetOutfitsInfo()
        assert(first ~= second and second ~= third and third ~= all[1])
        assert(first.situationCategories ~= second.situationCategories)
        first.name = "changed"
        first.situationCategories[1] = "changed"
        second.outfitID = 0
        third.situationCategories[2] = nil
        all[1].icon = 0
        all[2] = nil
        local fresh = api.GetOutfitInfo(91)
        assert(fresh.name == "rAiD Set" and fresh.icon == 135771)
        assert(fresh.situationCategories[1] == "Dungeon")
        assert(fresh.situationCategories[2] == "Raid")
        assert(#api.GetOutfitsInfo() == 2)
        "#,
    )
    .unwrap();
    let state = env.state();
    let state = state.borrow();
    assert_eq!(state.transmog_outfit_catalog.entries[0].name, "rAiD Set");
    assert_eq!(state.transmog_outfit_catalog.entries[0].outfit_id, 91);
}

#[test]
fn invalid_lookup_inputs_do_not_select_an_outfit() {
    let env = seeded_env();
    env.exec(
        r#"
        local api = C_TransmogOutfitInfo
        for _, query in ipairs({api.GetOutfitInfo, api.GetOutfitInfoByPlayerFacingIndex}) do
            for _, value in ipairs({false, {}, "91"}) do
                assert(not pcall(query, value))
            end
            assert(not pcall(query))
            assert(query(-1) == nil and query(0) == nil)
            assert(query(7.5) == nil and query(0/0) == nil and query(math.huge) == nil)
        end
        assert(not pcall(api.GetOutfitInfoByName))
        assert(not pcall(api.GetOutfitInfoByName, 91))
        assert(not pcall(api.GetOutfitInfoByName, {}))
        assert(api.GetOutfitInfoByName("") == nil)
        "#,
    )
    .unwrap();
}

#[test]
fn lookup_arguments_use_secret_caller_validation() {
    let env = seeded_env();
    env.exec(
        r#"
        local queries = {
            {C_TransmogOutfitInfo.GetOutfitInfoByName, "RAID SET"},
            {C_TransmogOutfitInfo.GetOutfitInfoByPlayerFacingIndex, 7},
            {C_TransmogOutfitInfo.GetOutfitInfo, 91},
        }
        for _, query in ipairs(queries) do
            local secret = secretwrap(query[2])
            local accepted, entry = pcall(query[1], secret)
            assert(accepted and entry.outfitID == 91,
                "untainted caller must resolve secret input")
            local function addon() return query[1](secret) end
            debug.setobjecttaint(addon, "OutfitCatalogProbe")
            assert(not pcall(addon), "tainted caller must reject secret")
            local function ordinary() return query[1](query[2]) end
            debug.setobjecttaint(ordinary, "OutfitCatalogProbe")
            assert(pcall(ordinary), "tainted caller can pass ordinary input")
        end
        "#,
    )
    .unwrap();
}

#[test]
fn empty_catalog_has_no_fabricated_outfits() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        assert(type(C_TransmogOutfitInfo.GetOutfitInfoByName) == "function",
            "name lookup missing")
        assert(type(C_TransmogOutfitInfo.GetOutfitInfoByPlayerFacingIndex) == "function",
            "player index lookup missing")
        assert(C_TransmogOutfitInfo.GetOutfitInfo(91) == nil)
        assert(C_TransmogOutfitInfo.GetOutfitInfoByName("Raid") == nil)
        assert(C_TransmogOutfitInfo.GetOutfitInfoByPlayerFacingIndex(7) == nil)
        local outfits = C_TransmogOutfitInfo.GetOutfitsInfo()
        assert(type(outfits) == "table" and next(outfits) == nil)
        "#,
    )
    .unwrap();
}
