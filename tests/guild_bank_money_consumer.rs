#![cfg(any(feature = "client-retail", feature = "client-wowforever"))]
//! Original consumer: `Blizzard_GuildBankUI/Mainline/Blizzard_GuildBankUI.xml:534-541`
//! (`GuildBankMoneyFrame`, `SmallMoneyFrame_OnLoad(self, "GUILDBANK")`), filled by
//! `Blizzard_GuildBankUI.lua:204` and `MoneyFrame.lua:158-165` from `GetGuildBankMoney()`.

use wow_ui_sim::lua_api::WowLuaEnv;

const OPEN_GUILD_BANK: &str = r#"
A_Admin.SetGuildInfo("Vault Keepers", "Banker", 12)
A_Admin.AddGuildBankTab("General", "Interface\\Icons\\INV_Misc_QuestionMark", true, true, -1, -1, false)
A_Admin.SetGuildBankMoney(1234567)
A_Admin.OpenGuildBank()
"#;

const SHOWN_COINS: &str = r#"
assert(GuildBankFrame:IsShown(), "GuildBankFrame opens for the GuildBanker interaction")
return GuildBankMoneyFrameGoldButton:GetText() .. "/"
    .. GuildBankMoneyFrameSilverButton:GetText() .. "/"
    .. GuildBankMoneyFrameCopperButton:GetText()
"#;

const OPEN_EMPTY_GUILD_BANK: &str = r#"
A_Admin.SetGuildInfo("Vault Keepers", "Banker", 12)
A_Admin.OpenGuildBank()
"#;

fn shown_guild_bank_coins(env: &WowLuaEnv) -> String {
    env.eval(SHOWN_COINS)
        .expect("GuildBankMoneyFrame coin buttons read")
}

fn guild_bank_lua_errors(env: &WowLuaEnv) -> Vec<String> {
    let state = env.state().borrow();
    let errors = state.lua_errors.iter();
    errors.filter(|error| error.contains("GuildBank")).cloned().collect()
}

prefork_full_ui_case! {
fn guild_bank_frame_shows_supplied_money(env: &WowLuaEnv) {
    env.exec(OPEN_GUILD_BANK).expect("original guild bank opens");
    assert_eq!(shown_guild_bank_coins(env), "123/45/67");

    env.exec("A_Admin.SetGuildBankMoney(9876543)")
        .expect("second guild bank amount is supplied");
    assert_eq!(shown_guild_bank_coins(env), "987/65/43");
    assert_eq!(guild_bank_lua_errors(env), Vec::<String>::new());
}
}

prefork_full_ui_case! {
fn guild_bank_withdraw_state_uses_supplied_permissions(env: &WowLuaEnv) {
    let empty_money: i64 = env.eval("return GetGuildBankMoney()").unwrap();
    let empty_allowance: i64 = env.eval("return GetGuildBankWithdrawMoney()").unwrap();
    let empty_gold_limit: i64 = env.eval("return GetGuildBankWithdrawGoldLimit()").unwrap();
    let empty_can_withdraw: bool = env.eval("return CanWithdrawGuildBankMoney()").unwrap();
    let empty_can_edit: bool = env.eval("return CanEditGuildBankTabInfo()").unwrap();
    let empty_tab_cost: i64 = env.eval("return GetGuildBankTabCost()").unwrap();
    assert_eq!((empty_money, empty_allowance, empty_gold_limit), (0, 0, 0));
    assert_eq!(empty_tab_cost, 0);
    assert!(!empty_can_withdraw);
    assert!(!empty_can_edit);
    env.exec(OPEN_EMPTY_GUILD_BANK).expect("original guild bank opens empty");
    assert_eq!(shown_guild_bank_coins(env), "0/0/0");
    let empty_withdraw_enabled: bool = env
        .eval("return GuildBankFrame.WithdrawButton:IsEnabled()")
        .unwrap();
    assert!(!empty_withdraw_enabled);

    env.exec(r#"
A_Admin.AddGuildBankTab("General", "Interface\\Icons\\INV_Misc_QuestionMark", true, true, 3, 2, false)
A_Admin.SetGuildBankMoney(500000)
A_Admin.SetGuildBankRankPermissions(1, 300000, {7,7,7,7,7,7}, {10,10,10,10,10,10}, false)
"#).expect("first withdrawal permission snapshot is supplied");
    let first_withdraw_enabled: bool = env
        .eval("return GuildBankFrame.WithdrawButton:IsEnabled()")
        .unwrap();
    assert!(first_withdraw_enabled);
    let first_limit: i64 = env.eval("return GetGuildBankWithdrawMoney()").unwrap();
    assert_eq!(first_limit, 300000);
    let first_remaining: String = env
        .eval("return GuildBankFrame.LimitLabel:GetText()")
        .unwrap();

    env.exec(r#"
A_Admin.ClearGuildBankTabs()
A_Admin.AddGuildBankTab("General", "Interface\\Icons\\INV_Misc_QuestionMark", true, true, 3, 1, false)
A_Admin.SetGuildBankRankPermissions(1, 0, {3,3,3,3,3,3}, {10,10,10,10,10,10}, false)
"#).expect("second withdrawal permission snapshot is supplied");
    let second_withdraw_enabled: bool = env
        .eval("return GuildBankFrame.WithdrawButton:IsEnabled()")
        .unwrap();
    assert!(!second_withdraw_enabled);
    let second_limit: i64 = env.eval("return GetGuildBankWithdrawMoney()").unwrap();
    assert_eq!(second_limit, 0);
    let second_remaining: String = env
        .eval("return GuildBankFrame.LimitLabel:GetText()")
        .unwrap();
    assert_ne!(first_remaining, second_remaining);
    assert_eq!(guild_bank_lua_errors(env), Vec::<String>::new());
}
}

#[cfg(feature = "client-wowforever")]
fn new_forever_game_env() -> WowLuaEnv {
    let env = WowLuaEnv::new().expect("create Forever Lua environment");
    crate::common::load_game_startup_inventory(&env);
    env.sync_addon_names_to_lua();
    env.apply_post_load_workarounds();
    let game = wow_ui_sim::screen::ScreenKind::Game;
    wow_ui_sim::startup::fire_startup_events_for_screen(&env, game);
    env
}

#[cfg(feature = "client-wowforever")]
#[test]
fn forever_guild_bank_frame_shows_supplied_money() {
    crate::common::with_timeout(120, || {
        guild_bank_frame_shows_supplied_money::run(&new_forever_game_env());
        guild_bank_withdraw_state_uses_supplied_permissions::run(&new_forever_game_env());
    });
}
