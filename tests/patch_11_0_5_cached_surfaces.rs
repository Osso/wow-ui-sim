//! Explicit producers and retirements survive unmodified cached Game UI preload.
#![cfg(feature = "client-retail")]

use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::lua_api::state::{BarberShopAlternateFormRace, BarberShopCharacterData};

prefork_full_ui_case! {
fn patch_11_0_5_cached_surfaces_preserve_retirements_and_character_query(env: &WowLuaEnv) {
    let errors_before = env.state().borrow().lua_errors.len();
    env.state().borrow_mut().barber_shop.current_character = Some(BarberShopCharacterData {
        name: "Gilnean".into(),
        alternate_form_race: Some(BarberShopAlternateFormRace {
            race_id: 22,
            name: "Worgen".into(),
            ..Default::default()
        }),
        ..Default::default()
    });
    env.exec(
        r#"
        assert(C_BarberShop.HasAlteredForm() == true)
        for i = 1, 2 do
            assert(C_AuctionHouse.RequestFavorites == nil)
            assert(rawget(C_AuctionHouse, 'RequestFavorites') == nil)
            assert(C_MajorFactions.GetCovenantIDForMajorFaction == nil)
            assert(rawget(C_MajorFactions, 'GetCovenantIDForMajorFaction') == nil)
        end
        assert(C_Glue.IsOnGlueScreen() == false)
        assert(IsOnGlueScreen == false)
        assert(GetCVarBool('loadDeprecationFallbacks'))
        for _, name in ipairs({'ChromaEffectsEnable', 'ChromaEffectsFactionColor'}) do
            assert(GetCVarDefault(name) == '1')
            assert(SetCVar(name, '0'))
            assert(C_CVar.GetCVarBool(name) == false)
            assert(GetCVarDefault(name) == '1')
        end
        "#,
    ).unwrap();
    assert_eq!(env.state().borrow().lua_errors.len(), errors_before);
}
}
