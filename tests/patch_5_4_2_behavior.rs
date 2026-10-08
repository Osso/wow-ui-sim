//! Bounded current behavior, not historical RNG, glue or native parity.
#![cfg(feature = "client-retail")]

use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::lua_api::state::GuildMember;

prefork_full_ui_case! {
fn patch_5_4_2_existing_roster_backing(env: &WowLuaEnv) {
    env.state().borrow_mut().world.guild_members = vec![GuildMember {
        name: "Arthas-Silvermoon".into(),
        rank_index: 1,
        online: true,
    }];
    let name: String = env.eval("return (GetGuildRosterInfo(1))").expect("roster name");
    assert_eq!(name, "Arthas-Silvermoon");

    env.state().borrow_mut().world.guild_members[0].name = "Jaina-Proudmoore".into();
    let name: String = env.eval("return (GetGuildRosterInfo(1))").expect("updated roster name");
    assert_eq!(name, "Jaina-Proudmoore");

    // Missing realm data is a documented model gap, not automatic qualification.
    env.state().borrow_mut().world.guild_members[0].name = "Jaina".into();
    let name: String = env.eval("return (GetGuildRosterInfo(1))").expect("unqualified input");
    assert_eq!(name, "Jaina");
    env.state().borrow_mut().world.guild_members.clear();
    let missing: Option<String> = env.eval("return (GetGuildRosterInfo(1))").expect("empty roster");
    assert_eq!(missing, None);
}
}

prefork_full_ui_case! {
fn patch_5_4_2_enum_values_and_existing_absence(env: &WowLuaEnv) {
    let result: bool = env.eval(r#"
        assert(LE_AUTOCOMPLETE_PRIORITY_OTHER == 1)
        assert(LE_AUTOCOMPLETE_PRIORITY_INTERACTED == 2)
        assert(LE_AUTOCOMPLETE_PRIORITY_IN_GROUP == 3)
        assert(LE_AUTOCOMPLETE_PRIORITY_GUILD == 4)
        assert(LE_AUTOCOMPLETE_PRIORITY_FRIEND == 5)
        assert(rawget(_G, 'StartUnratedArena') == nil and _G.StartUnratedArena == nil)
        assert(rawget(_G, 'securerandom') == nil and _G.securerandom == nil)
        return true
    "#).expect("five retained numeric values and already-absent globals");
    assert!(result);
}
}
