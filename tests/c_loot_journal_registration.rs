#![cfg(any(feature = "client-retail", feature = "client-wowforever"))]

use wow_ui_sim::lua_api::WowLuaEnv;

#[test]
fn rust_item_set_registration_survives_bootstrap() {
    let env = WowLuaEnv::new().expect("Lua environment should initialize");
    env.exec(
        r#"
        A_Admin.SetLootJournalItemSets({
            { setID=501, name="Registration Alpha", itemLevel=620, classID=1, specID=71,
              items={{ itemID=41001, icon=91001, slot=1, invType=13 }} },
        })
        "#,
    )
    .expect("host-supplied item-set data should be accepted");

    let result: String = env
        .eval(
            r#"
            local sets = C_LootJournal.GetItemSets(1, 71)
            local items = C_LootJournal.GetItemSetItems(501)
            return #sets .. ":" .. sets[1].name .. ":" .. #items .. ":" .. items[1].itemID
            "#,
        )
        .expect("C_LootJournal should return the modeled item-set data");

    assert_eq!(result, "1:Registration Alpha:1:41001");
}
