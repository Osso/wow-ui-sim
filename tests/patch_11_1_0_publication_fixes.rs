//! Bounded current simulator contracts, not native 11.1.0 parity.
#![cfg(feature = "client-retail")]

use wow_ui_sim::lua_api::WowLuaEnv;

#[test]
fn patch_11_1_0_retired_members_survive_repeated_lookup() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        for _, entry in ipairs({
            {C_BarberShop, 'GetCustomizationScope'},
            {C_TransmogCollection, 'CanAppearanceBeDisplayedOnPlayer'},
        }) do
            for i = 1, 2 do
                assert(entry[1][entry[2]] == nil, entry[2])
                assert(rawget(entry[1], entry[2]) == nil, entry[2])
            end
        end
        "#,
    )
    .unwrap();
}

#[test]
fn patch_11_1_0_specialization_names_use_catalog_identity() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        assert(GetSpecializationNameForSpecID(70, 2) == 'Retribution')
        assert(GetSpecializationNameForSpecID(65, 3) == 'Holy')
        assert(GetSpecializationNameForSpecID(577) == 'Havoc')
        assert(GetSpecializationNameForSpecID(999999, 2) == nil)
        assert(GetSpecializationNameForSpecID(0) == nil)
        "#,
    )
    .unwrap();
}
