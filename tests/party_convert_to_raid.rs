#![cfg(any(feature = "client-retail", feature = "client-wowforever", feature = "client-mists"))]

use crate::common::blizzard_addon_harness::with_blizzard_addon_closure;
use wow_ui_sim::client_profile::{self, ClientProfile};
use wow_ui_sim::lua_api::state::PartyMember;

fn consumer_source() -> &'static str {
    match client_profile::ACTIVE {
        ClientProfile::Retail => "Standard/UnitPopupButtons.lua",
        ClientProfile::WowForever => "Mainline/UnitPopupButtons.lua",
        ClientProfile::Mists => "Wrath/UnitPopupButtons.lua",
        _ => unreachable!("test is enabled only for Retail, Forever and Mists"),
    }
}

fn party_member() -> PartyMember {
    PartyMember {
        map_position: None,
        name: "PartyMember".into(),
        name_cached: true,
        connected: true,
        class_index: 2,
        level: 60,
        health: 100,
        health_max: 100,
        power: 100,
        power_max: 100,
        power_type: 0,
        power_type_name: "MANA".into(),
        is_leader: false,
        dead_since: None,
        buffs: vec![],
        debuffs: vec![],
    }
}

#[test]
fn original_unit_popup_convert_to_raid_updates_group_readers() {
    with_blizzard_addon_closure(&["Blizzard_UnitPopup"], &[], |env, loaded| {
        assert!(loaded.iter().any(|addon| addon == "Blizzard_UnitPopup"));
        let source: String = env
            .eval("return debug.getinfo(UnitPopupConvertToRaidButtonMixin.OnClick, 'S').source")
            .unwrap();
        assert!(source.contains(consumer_source()), "unexpected UnitPopup source: {source}");
        {
            let mut sim = env.state().borrow_mut();
            sim.party_group_active = true;
            sim.party_members = vec![party_member()];
            sim.party_is_raid = Some(false);
        }
        env.exec(
            r##"
            assert(not IsInRaid())
            assert(select("#", C_PartyInfo.ConvertToRaid()) == 0)
            assert(IsInRaid())
            "##,
        )
        .expect("documented conversion call should update raid state");
        env.state().borrow_mut().party_is_raid = Some(false);
        env.exec(
            r##"
            UnitPopupConvertToRaidButtonMixin:OnClick({})
            assert(IsInRaid() and UnitInRaid("party1"))
            "##,
        )
        .expect("selected Blizzard UnitPopup consumer should convert the active group");
    });
}
