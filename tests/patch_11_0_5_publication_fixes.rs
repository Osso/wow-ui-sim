//! Bounded current simulator contracts, not native 11.0.5 parity.
#![cfg(feature = "client-retail")]

use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::lua_api::state::{BarberShopAlternateFormRace, BarberShopCharacterData};

#[test]
fn patch_11_0_5_retired_members_survive_repeated_lookup() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        for _, entry in ipairs({
            {C_AuctionHouse, 'RequestFavorites'},
            {C_MajorFactions, 'GetCovenantIDForMajorFaction'},
        }) do
            for i = 1, 2 do
                assert(entry[1][entry[2]] == nil, entry[2])
                assert(rawget(entry[1], entry[2]) == nil, entry[2])
            end
        end
        assert(type(C_AuctionHouse.GetBrowseResults) == 'function')
        assert(type(C_MajorFactions.GetMajorFactionData) == 'function')
        "#,
    )
    .unwrap();
}

#[test]
fn patch_11_0_5_chroma_cvars_have_mutable_values_and_immutable_defaults() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        for _, name in ipairs({'ChromaEffectsEnable', 'ChromaEffectsFactionColor'}) do
            assert(GetCVarDefault(name) == '1', name)
            assert(C_CVar.GetCVarDefault(name) == '1', name)
            assert(GetCVar(name) == '1', name)
            assert(C_CVar.SetCVar(string.upper(name), '0'))
            assert(GetCVarBool(name) == false)
            assert(GetCVarDefault(name) == '1')
            assert(SetCVar(name, GetCVarDefault(name)))
            assert(C_CVar.GetCVarBool(name) == true)
        end
        "#,
    )
    .unwrap();
}

#[test]
fn patch_11_0_5_altered_form_availability_uses_character_snapshot() {
    let env = WowLuaEnv::new().unwrap();
    let has_altered_form = || env.eval::<bool>("return C_BarberShop.HasAlteredForm()").unwrap();
    assert!(!has_altered_form());
    env.state().borrow_mut().barber_shop.current_character = Some(BarberShopCharacterData {
        name: "Gilnean".into(),
        alternate_form_race: Some(BarberShopAlternateFormRace {
            race_id: 22,
            name: "Worgen".into(),
            file_name: "worgen".into(),
            create_screen_icon_atlas: "raceicon-worgen-male".into(),
        }),
        ..Default::default()
    });
    assert!(has_altered_form());
    env.exec("C_BarberShop.SetViewingAlteredForm(true)").unwrap();
    assert!(has_altered_form());
    env.exec("C_BarberShop.SetViewingAlteredForm(false)").unwrap();
    assert!(has_altered_form());
    env.state()
        .borrow_mut()
        .barber_shop
        .current_character
        .as_mut()
        .unwrap()
        .alternate_form_race = None;
    assert!(!has_altered_form());
    env.state().borrow_mut().barber_shop.current_character = None;
    assert!(!has_altered_form());
}
