use wow_ui_sim::lua_api::WowLuaEnv;

#[test]
fn unknown_c_members_remain_nil_without_mutating_namespace() {
    let env = WowLuaEnv::new().expect("Lua environment");
    env.exec(
        r#"
        local namespaces = {
            "C_Container", "C_GameRules", "C_GuildInfo", "C_SecureTransfer",
            "C_CatalogShop", "C_Housing", "C_HousingBasicMode", "C_HousingExpertMode",
            "C_HousingLayout", "C_HousingDecor", "C_HousingCustomizeMode",
            "C_HousingNeighborhood", "C_HouseExterior", "C_HousingCatalog",
        }
        local key = "F0UnknownMember"
        for _, index in ipairs(namespaces) do
            local namespace = rawget(_G, index)
            assert(type(namespace) == "table", index)
            assert(rawget(namespace, key) == nil, index .. ": raw lookup")
            assert(namespace[key] == nil, index .. ": first lookup")
            assert(namespace[key] == nil, index .. ": second lookup")
            assert(type(namespace[key]) == "nil", index .. ": type")
            assert(rawget(namespace, key) == nil, index .. ": no memoization")
            for member in pairs(namespace) do
                assert(member ~= key, index .. ": enumeration")
            end
        end
        A_Admin.SetGameRule("F0Rule", true)
        assert(C_GameRules.IsGameRuleActive("F0Rule") == true)
        A_Admin.SetGuildClubId("f0-guild")
        assert(C_GuildInfo.GetClubId() == "f0-guild")
        assert(type(C_Container.GetContainerNumSlots) == "function")
        assert(C_HousingCatalog.GetCartSizeLimit() == 20)
        "#,
    )
    .expect("unknown C members must remain nil; explicit registrations must survive");
}

#[cfg(not(any(feature = "client-retail", feature = "client-wowforever")))]
#[test]
fn secure_transfer_shim_cancel_increments_count() {
    let env = WowLuaEnv::new().expect("Lua environment");
    env.exec(
        r#"
        local before = C_SecureTransfer._state.cancelCount
        C_SecureTransfer.Cancel()
        assert(C_SecureTransfer._state.cancelCount == before + 1)
        "#,
    )
    .expect("the legacy secure transfer shim must record cancellation");
}
