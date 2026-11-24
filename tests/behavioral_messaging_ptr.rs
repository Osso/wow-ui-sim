#![cfg(feature = "client-ptr")]

use wow_ui_sim::lua_api::WowLuaEnv;

#[test]
fn ptr_behavioral_messaging_surface_is_unchanged() {
    let env = WowLuaEnv::new().expect("Lua environment should initialize");
    let surface: String = env
        .eval(
            r#"
            local members = {}
            for name, value in pairs(C_BehavioralMessaging) do
                table.insert(members, name .. ":" .. type(value))
            end
            table.sort(members)
            local returns = select("#", C_BehavioralMessaging.SendNotificationReceipt(87001, 0, 0))
            return table.concat(members, ",") .. ":" .. tostring(returns)
            "#,
        )
        .expect("PTR namespace query should succeed");

    assert_eq!(surface, "SendNotificationReceipt:function:0");
}
