//! SHARD_TRANSFER registration on Retail and PTR. The supplied current-house
//! snapshot test moved to housing_supplied_policies.rs.
#![cfg(any(feature = "client-retail", feature = "client-ptr"))]

use wow_ui_sim::client_profile::{ACTIVE, ClientProfile};
use wow_ui_sim::lua_api::WowLuaEnv;

#[test]
fn shard_transfer_registration_matches_active_profile() {
    let env = WowLuaEnv::new().unwrap();
    let observed: (bool, bool, bool) = env
        .eval(
            r#"
            local frame = CreateFrame('Frame')
            local ok = pcall(frame.RegisterEvent, frame, 'SHARD_TRANSFER')
            return ok, frame:IsEventRegistered('SHARD_TRANSFER'), C_EventUtils.IsEventValid('SHARD_TRANSFER')
            "#,
        )
        .unwrap();
    let declared = ACTIVE == ClientProfile::Ptr;
    assert_eq!(observed, (declared, declared, declared));
}
