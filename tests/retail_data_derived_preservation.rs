#![cfg(feature = "client-retail")]

use wow_ui_sim::lua_api::WowLuaEnv;

#[test]
fn retail_does_not_publish_ptr_only_catalog_queries() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        "assert(C_SpecializationInfo.GetAllClassIDs == nil); assert(C_TransmogSets.HasAnyValidSets == nil)",
    )
    .unwrap();
}
