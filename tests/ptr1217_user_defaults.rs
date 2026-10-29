//! User-approved PTR 12.1.7 defaults (not native evidence).
#![cfg(feature = "client-ptr")]

use wow_ui_sim::lua_api::WowLuaEnv;

#[test]
fn paid_transfer_destinations_default_to_empty_list() {
    let env = WowLuaEnv::new().expect("lua env");
    let (kind, count): (String, i32) = env
        .eval(
            "local d = C_CharacterServices.GetPaidCharacterTransferSuperDistrictDestinations() \
             return type(d), #d",
        )
        .expect("paid transfer destinations");
    assert_eq!((kind.as_str(), count), ("table", 0));
}

#[test]
fn actor_unit_creature_display_load_reports_no_model() {
    let env = WowLuaEnv::new().expect("lua env");
    let success: bool = env
        .eval("return CreateFrame('ModelScene'):CreateActor():SetModelByUnitCreatureDisplayID('pet')")
        .expect("actor method");
    assert!(!success);
}
