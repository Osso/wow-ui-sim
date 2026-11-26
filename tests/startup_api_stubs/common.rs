use wow_ui_sim::lua_api::WowLuaEnv;

pub fn env() -> WowLuaEnv {
    WowLuaEnv::new().unwrap_or_else(|error| panic!("create Lua environment: {error:?}"))
}
