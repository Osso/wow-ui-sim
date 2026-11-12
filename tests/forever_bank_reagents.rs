#![cfg(feature = "client-wowforever")]

use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::screen::ScreenKind;

#[test]
fn forever_reagents_from_bank_allowed_reads_player_and_restriction_state() {
    let env = WowLuaEnv::new().expect("Forever bank reagent environment");
    env.set_screen_mode(ScreenKind::Game);
    env.state().borrow_mut().bank_reagent_object = None;
    env.exec(
        r#"
        assert(type(ReagentsFromBankAllowed) == 'function', 'Game installer must register the query')
        assert(select('#', ReagentsFromBankAllowed()) == 1, 'query returns one Boolean')
        assert(ReagentsFromBankAllowed() == false, 'no player object returns false')
        "#,
    )
    .expect("missing player state returns false");

    env.exec("A_Admin.SetGameRule('CraftFromBankRestricted', true)")
        .expect("set the restriction rule");
    env.exec("assert(ReagentsFromBankAllowed() == false, 'no player remains false')")
        .expect("active rule does not change missing-player result");

    env.state().borrow_mut().bank_reagent_object = Some(false);
    env.exec("assert(ReagentsFromBankAllowed() == false, 'active restriction denies clear permission bit')")
        .expect("active restriction denies the clear permission bit");

    env.exec("A_Admin.SetGameRule('CraftFromBankRestricted', nil)")
        .expect("remove the restriction rule");
    env.exec(
        "assert(ReagentsFromBankAllowed() == true, 'inactive restriction allows present player')",
    )
    .expect("inactive restriction allows a present player");

    env.exec("A_Admin.SetGameRule('CraftFromBankRestricted', true)")
        .expect("restore the restriction rule");
    env.state().borrow_mut().bank_reagent_object = Some(true);
    env.exec(
        "assert(ReagentsFromBankAllowed() == true, 'active restriction follows permission bit')",
    )
    .expect("active restriction allows the set permission bit");
}
