#![cfg(any(feature = "client-retail", feature = "client-wowforever"))]
//! `C_LFGInfo.ConfirmLfgExpandSearch` consumes the supplied expand-search
//! prompt. Original consumer: `LFGEventFrame_OnEvent` shows the
//! `LFG_QUEUE_EXPAND` popup on `SHOW_LFG_EXPAND_SEARCH_PROMPT`, and the popup's
//! accept button runs the API (Blizzard_GroupFinder/Shared/LFGFrame.lua:87-96, 264-265).

use wow_ui_sim::lua_api::WowLuaEnv;

const ACCEPT_PROMPT: &str = r#"
local key, data = ...
A_Admin.ShowLfgExpandSearchPrompt(key, data)
local _, dialog = StaticPopup_Visible("LFG_QUEUE_EXPAND")
if not dialog then return false end
dialog:GetButton1():Click()
return true
"#;

fn accept_prompt(env: &WowLuaEnv, key: u64, data: &str) -> Option<(u64, Vec<u8>)> {
    let chunk = format!("return (function(...) {ACCEPT_PROMPT} end)({key}, {data:?})");
    let shown: bool = env.eval(&chunk).expect("the original LFG popup should run");
    assert!(shown, "SHOW_LFG_EXPAND_SEARCH_PROMPT shows LFG_QUEUE_EXPAND");
    env.state().borrow().lfg_expand_prompt.clone()
}

fn assert_accept_consumes_nonzero_prompts(env: &WowLuaEnv) {
    assert_eq!(accept_prompt(env, 19, "ab"), None);
    assert_eq!(accept_prompt(env, 0, "z"), Some((0, b"z".to_vec())));
    assert_eq!(accept_prompt(env, 42, "cde"), None);
}

#[cfg(feature = "client-retail")]
prefork_full_ui_case! {
fn lfg_expand_popup_accept_consumes_the_supplied_prompt(env: &WowLuaEnv) {
    assert_accept_consumes_nonzero_prompts(env);
}
}

#[cfg(feature = "client-wowforever")]
#[test]
fn forever_lfg_expand_popup_accept_consumes_the_supplied_prompt() {
    crate::common::with_timeout(120, || {
        let env = crate::common::forever_game_env();
        assert_accept_consumes_nonzero_prompts(&env);
    });
}
