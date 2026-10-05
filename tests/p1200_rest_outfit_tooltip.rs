#![cfg(feature = "retail-12-0-5")]
use wow_ui_sim::lua_api::WowLuaEnv;

#[test]
fn p1200_rest_outfit_tooltip_missing() {
    let env = WowLuaEnv::new().unwrap();
    env.exec("assert(select('#', C_TooltipInfo.GetOutfit(999999)) == 0)")
        .unwrap();
    use wow_ui_sim::c_api::c_transmog_outfit_info::OutfitEntry;
    env.state()
        .borrow_mut()
        .transmog_outfit_catalog
        .entries
        .push(OutfitEntry {
            outfit_id: 42,
            name: "Raid armor".into(),
            situation_categories: vec![],
            icon: 134400,
            is_event_outfit: false,
            is_disabled: false,
            player_facing_outfit_index: 1,
        });
    env.exec(
        r#"
        local data = C_TooltipInfo.GetOutfit(42)
        assert(data.type == 27 and data.id == 42 and #data.lines == 1)
        assert(data.lines[1].leftText == 'Raid armor')
        data.lines[1].leftText = 'mutated'
        assert(C_TooltipInfo.GetOutfit(42).lines[1].leftText == 'Raid armor')
    "#,
    )
    .unwrap();
    env.state().borrow_mut().transmog_outfit_catalog.entries[0].name = "Dungeon armor".into();
    env.exec("assert(C_TooltipInfo.GetOutfit(42).lines[1].leftText == 'Dungeon armor')")
        .unwrap();
    env.state()
        .borrow_mut()
        .transmog_outfit_catalog
        .entries
        .clear();
    env.exec("assert(select('#', C_TooltipInfo.GetOutfit(42)) == 0)")
        .unwrap();
}
