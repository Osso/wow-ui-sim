use wow_ui_sim::lua_api::WowLuaEnv;

fn env() -> WowLuaEnv {
    WowLuaEnv::new().expect("Failed to create Lua environment")
}

#[test]
fn party_info_loot_method_availability_matches_seeded_methods() {
    let env = env();
    let expected_method = if cfg!(feature = "retail-12-0-5") {
        "Personal"
    } else {
        "Group"
    };
    env.exec(&format!(
        "ExpectedDefaultLootMethod = Enum.LootMethod.{expected_method}"
    ))
    .unwrap();
    let result: String = env
        .eval(
            r#"
            if type(C_PartyInfo.IsLootMethodAvailable) ~= "function" then
                return "missing_is_loot_method_available"
            end

            local available = C_PartyInfo.GetAvailableLootMethods()
            if type(available) ~= "table" then
                return "available_loot_methods_should_be_table"
            end
            if #available ~= 5 then
                return "expected_five_seeded_loot_methods"
            end

            local expected = {
                [Enum.LootMethod.Freeforall] = true,
                [Enum.LootMethod.Roundrobin] = true,
                [Enum.LootMethod.Masterlooter] = true,
                [Enum.LootMethod.Group] = true,
                [Enum.LootMethod.Needbeforegreed] = true,
            }

            for _, method in ipairs(available) do
                if not expected[method] then
                    return "unexpected_seeded_loot_method"
                end
                expected[method] = nil

                if not C_PartyInfo.IsLootMethodAvailable(method) then
                    return "seeded_loot_method_should_be_available"
                end
            end

            if next(expected) ~= nil then
                return "missing_seeded_loot_methods"
            end

            if C_PartyInfo.IsLootMethodAvailable(Enum.LootMethod.Personal) then
                return "personal_loot_should_not_be_available"
            end

            local method, masterLootPartyID, masterLooterRaidID = C_PartyInfo.GetLootMethod()
            if method ~= ExpectedDefaultLootMethod then
                return "expected_epoch_specific_default_loot_method"
            end
            if masterLootPartyID ~= nil or masterLooterRaidID ~= nil then
                return "default_loot_should_not_have_master_looter_ids"
            end

            return "ok"
            "#,
        )
        .unwrap();

    assert_eq!(
        result, "ok",
        "C_PartyInfo loot method APIs should expose a coherent seeded availability set"
    );
}
