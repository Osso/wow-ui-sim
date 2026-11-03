use wow_ui_sim::lua_api::WowLuaEnv;

#[cfg(not(feature = "client-wrath"))]
#[test]
fn addon_message_prefix_registration_has_observable_state_and_fresh_snapshots() {
    let env = WowLuaEnv::new().unwrap();
    let result: String = env
        .eval(
            r##"
            local register = rawget(C_ChatInfo, "RegisterAddonMessagePrefix")
            local contains = rawget(C_ChatInfo, "IsAddonMessagePrefixRegistered")
            local get = rawget(C_ChatInfo, "GetRegisteredAddonMessagePrefixes")
            assert(type(register) == "function" and type(contains) == "function" and type(get) == "function")
            assert(select("#", get()) == 1 and #get() == 0)
            assert(contains("Auctionator") == false)
            assert(select("#", register("Auctionator")) == 1)
            assert(contains("Auctionator") == true and contains("auctionator") == false)
            assert(register("Auctionator") == Enum.RegisterAddonMessagePrefixResult.DuplicatePrefix)
            assert(register("LibKeystone") == Enum.RegisterAddonMessagePrefixResult.Success)
            local snapshot = get()
            table.sort(snapshot)
            assert(#snapshot == 2 and snapshot[1] == "Auctionator" and snapshot[2] == "LibKeystone")
            snapshot[1] = "changed"
            snapshot[3] = "injected"
            assert(contains("Auctionator") and not contains("changed") and not contains("injected"))
            assert(#get() == 2 and get() ~= snapshot)
            return "ok"
            "##,
        )
        .unwrap();
    assert_eq!(result, "ok");
}

#[cfg(not(feature = "client-wrath"))]
#[test]
fn addon_message_prefix_validation_counts_bytes_and_preserves_capacity() {
    let env = WowLuaEnv::new().unwrap();
    let result: String = env
        .eval(
            r#"
            local register = C_ChatInfo.RegisterAddonMessagePrefix
            local result = Enum.RegisterAddonMessagePrefixResult
            assert(register("") == result.InvalidPrefix)
            assert(register(string.rep("x", 17)) == result.InvalidPrefix)
            assert(register(string.rep("é", 9)) == result.InvalidPrefix)
            assert(not pcall(register) and not pcall(register, {}) and not pcall(register, true))
            local edge = string.rep("é", 8)
            assert(#edge == 16 and register(edge) == result.Success)
            assert(C_ChatInfo.IsAddonMessagePrefixRegistered(edge))
            assert(register(string.char(255)) == result.Success)
            assert(C_ChatInfo.IsAddonMessagePrefixRegistered(string.char(255)))
            assert(C_ChatInfo.GetRegisteredAddonMessagePrefixes()[2] == string.char(255))
            for i = 1, 510 do
                assert(register("Prefix" .. i) == result.Success)
            end
            assert(#C_ChatInfo.GetRegisteredAddonMessagePrefixes() == 512)
            assert(register(edge) == result.DuplicatePrefix)
            assert(register("") == result.InvalidPrefix)
            assert(register("Overflow") == result.MaxPrefixes)
            assert(not C_ChatInfo.IsAddonMessagePrefixRegistered("Overflow"))
            assert(#C_ChatInfo.GetRegisteredAddonMessagePrefixes() == 512)
            return "ok"
            "#,
        )
        .unwrap();
    assert_eq!(result, "ok");
}

#[cfg(not(feature = "client-wrath"))]
#[test]
fn addon_message_prefix_registry_is_local_to_the_lua_environment() {
    let first = WowLuaEnv::new().unwrap();
    let second = WowLuaEnv::new().unwrap();
    let registered: i32 = first
        .eval("return C_ChatInfo.RegisterAddonMessagePrefix('SessionOne')")
        .unwrap();
    assert_eq!(registered, 0);
    let first_has_prefix: bool = first
        .eval("return C_ChatInfo.IsAddonMessagePrefixRegistered('SessionOne')")
        .unwrap();
    let second_is_empty: bool = second
        .eval(
            "return not C_ChatInfo.IsAddonMessagePrefixRegistered('SessionOne') and #C_ChatInfo.GetRegisteredAddonMessagePrefixes() == 0",
        )
        .unwrap();
    assert!(first_has_prefix);
    assert!(second_is_empty);
}

#[cfg(not(feature = "client-wrath"))]
#[test]
fn addon_message_prefix_cstring_normalization_precedes_validation_and_lookup() {
    let env = WowLuaEnv::new().unwrap();
    let result: String = env
        .eval(
            r#"
            local register = C_ChatInfo.RegisterAddonMessagePrefix
            local result = Enum.RegisterAddonMessagePrefixResult
            assert(register("Prefix\0" .. string.rep("x", 17)) == result.Success)
            assert(register("Prefix") == result.DuplicatePrefix)
            assert(C_ChatInfo.IsAddonMessagePrefixRegistered("Prefix\0suffix"))
            local prefixes = C_ChatInfo.GetRegisteredAddonMessagePrefixes()
            assert(#prefixes == 1 and prefixes[1] == "Prefix")
            assert(register("\0Prefix") == result.InvalidPrefix)
            assert(not C_ChatInfo.IsAddonMessagePrefixRegistered("\0Prefix"))
            assert(#C_ChatInfo.GetRegisteredAddonMessagePrefixes() == 1)
            return "ok"
            "#,
        )
        .unwrap();
    assert_eq!(result, "ok");
}

#[cfg(feature = "client-wrath")]
#[test]
fn addon_message_prefix_modern_namespace_is_not_added_to_legacy_wrath() {
    let env = WowLuaEnv::new().unwrap();
    let absent: bool = env
        .eval(
            r#"
            return rawget(C_ChatInfo, "RegisterAddonMessagePrefix") == nil
                and rawget(C_ChatInfo, "IsAddonMessagePrefixRegistered") == nil
                and rawget(C_ChatInfo, "GetRegisteredAddonMessagePrefixes") == nil
            "#,
        )
        .unwrap();
    assert!(absent);
}
