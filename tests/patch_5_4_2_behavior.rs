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
fn patch_5_4_2_current_enum_aliases_and_existing_absence(env: &WowLuaEnv) {
    // Current cached documentation uses 0..4, unlike this page's historical 1..5.
    let values: (i32, i32, i32, i32, i32) = env.eval(r#"
        return LE_AUTOCOMPLETE_PRIORITY_OTHER, LE_AUTOCOMPLETE_PRIORITY_INTERACTED,
            LE_AUTOCOMPLETE_PRIORITY_IN_GROUP, LE_AUTOCOMPLETE_PRIORITY_GUILD,
            LE_AUTOCOMPLETE_PRIORITY_FRIEND
    "#).expect("current cached autocomplete values");
    assert_eq!(values, (0, 1, 2, 3, 4));
    let result: bool = env.eval(r#"
        assert(LE_AUTOCOMPLETE_PRIORITY_OTHER == Enum.AutoCompletePriority.Other)
        assert(LE_AUTOCOMPLETE_PRIORITY_INTERACTED == Enum.AutoCompletePriority.Interacted)
        assert(LE_AUTOCOMPLETE_PRIORITY_IN_GROUP == Enum.AutoCompletePriority.InGroup)
        assert(LE_AUTOCOMPLETE_PRIORITY_GUILD == Enum.AutoCompletePriority.Guild)
        assert(LE_AUTOCOMPLETE_PRIORITY_FRIEND == Enum.AutoCompletePriority.Friend)
        assert(rawget(_G, 'StartUnratedArena') == nil and _G.StartUnratedArena == nil)
        assert(rawget(_G, 'securerandom') == nil and _G.securerandom == nil)
        return true
    "#).expect("current aliases and already-absent globals");
    assert!(result);
}
}
