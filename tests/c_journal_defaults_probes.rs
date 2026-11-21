use wow_ui_sim::lua_api::WowLuaEnv;

fn env() -> WowLuaEnv {
    WowLuaEnv::new().expect("Failed to create Lua environment")
}

#[cfg(not(any(feature = "client-retail", feature = "client-wowforever")))]
#[test]
fn loot_journal_and_specialization_defaults_are_empty() {
    let env = env();
    let result: String = env
        .eval(
            r#"
            if #C_LootJournal.GetItemSets(1, 1) ~= 0 then return "sets" end
            if #C_LootJournal.GetItemSetItems(1) ~= 0 then return "items" end
            if C_SpecializationInfo.GetInspectSelectedPvpTalent() ~= nil then return "pvp-talent" end
            return "ok"
            "#,
        )
        .expect("journal defaults should be callable");

    assert_eq!(result, "ok");
}
