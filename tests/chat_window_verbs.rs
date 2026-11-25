//! Integration tests for `src/lua_api/globals/chat_window_verbs.rs`.

use wow_ui_sim::lua_api::WowLuaEnv;

fn env() -> WowLuaEnv {
    WowLuaEnv::new().expect("WowLuaEnv init")
}

// ── SetChatWindowAlpha ────────────────────────────────────────────────────────

#[test]
fn set_chat_window_alpha_creates_window_and_stores_value() {
    let env = env();
    env.exec("SetChatWindowAlpha(1, 0.75)").unwrap();
    let st = env.state().borrow();
    let w = st.chat_windows.get(&1).expect("window 1 must exist");
    assert!((w.alpha - 0.75).abs() < 1e-6);
}

#[test]
fn set_chat_window_alpha_clamps_out_of_range_values() {
    let env = env();
    env.exec(
        "SetChatWindowAlpha(1, 2.0)
               SetChatWindowAlpha(2, -1.0)",
    )
    .unwrap();
    let st = env.state().borrow();
    assert!((st.chat_windows[&1].alpha - 1.0).abs() < 1e-6);
    assert!((st.chat_windows[&2].alpha - 0.0).abs() < 1e-6);
}

#[test]
fn set_chat_window_size_stores_font_size() {
    let env = env();
    env.exec("SetChatWindowSize(3, 16)").unwrap();
    let st = env.state().borrow();
    let w = st.chat_windows.get(&3).expect("window 3 must exist");
    assert!((w.font_size - 16.0).abs() < 1e-6);
}

// ── SetChatWindowColor ────────────────────────────────────────────────────────

#[test]
fn set_chat_window_color_stores_rgb_triple() {
    let env = env();
    env.exec("SetChatWindowColor(1, 0.1, 0.2, 0.3)").unwrap();
    let st = env.state().borrow();
    let w = &st.chat_windows[&1];
    assert!((w.r - 0.1).abs() < 1e-6);
    assert!((w.g - 0.2).abs() < 1e-6);
    assert!((w.b - 0.3).abs() < 1e-6);
}

// ── SetChatWindowLocked / Uninteractable ──────────────────────────────────────

#[test]
fn set_chat_window_locked_true_and_false() {
    let env = env();
    env.exec("SetChatWindowLocked(1, true)").unwrap();
    assert!(env.state().borrow().chat_windows[&1].locked);
    env.exec("SetChatWindowLocked(1, false)").unwrap();
    assert!(!env.state().borrow().chat_windows[&1].locked);
}

#[test]
fn set_chat_window_locked_default_is_true_when_arg_omitted() {
    let env = env();
    env.exec("SetChatWindowLocked(1)").unwrap();
    assert!(env.state().borrow().chat_windows[&1].locked);
}

#[test]
fn set_chat_window_uninteractable_round_trips() {
    let env = env();
    env.exec("SetChatWindowUninteractable(2, true)").unwrap();
    assert!(env.state().borrow().chat_windows[&2].uninteractable);
}

// ── AddChatWindowChannel ──────────────────────────────────────────────────────

#[test]
fn add_chat_window_channel_appends_unique_channels() {
    let env = env();
    env.exec(
        r#"AddChatWindowChannel(1, "Trade")
           AddChatWindowChannel(1, "General")
           AddChatWindowChannel(1, "Trade")"#,
    )
    .unwrap();
    let st = env.state().borrow();
    let w = &st.chat_windows[&1];
    assert_eq!(w.channels, vec!["Trade".to_string(), "General".to_string()]);
    assert!(w.channel_names.contains("Trade"));
    assert!(w.channel_names.contains("General"));
    assert_eq!(w.channel_names.len(), 2);
}

#[test]
fn add_and_remove_chat_window_messages_mutates_unique_groups() {
    let env = env();
    env.exec(
        r#"AddChatWindowMessages(1, "PET_BATTLE_COMBAT_LOG")
           AddChatWindowMessages(1, "SAY")
           AddChatWindowMessages(1, "SAY")
           RemoveChatWindowMessages(1, "PET_BATTLE_COMBAT_LOG")"#,
    )
    .unwrap();
    let st = env.state().borrow();
    let w = &st.chat_windows[&1];
    assert_eq!(w.messages, vec!["SAY".to_string()]);
}

#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
#[test]
fn first_login_chat_windows_one_and_two_are_shown_and_docked() {
    let env = env();
    let states: (bool, f64, bool, f64, bool) = env
        .eval("local _,_,_,_,_,_,s1,_,d1=GetChatWindowInfo(1); local _,_,_,_,_,_,s2,_,d2=GetChatWindowInfo(2); local _,_,_,_,_,_,s3=GetChatWindowInfo(3); return s1,d1,s2,d2,s3")
        .unwrap();
    assert_eq!(states, (true, 1.0, true, 2.0, false));
}

#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
#[test]
fn chat_window_layout_verbs_round_trip_distinct_records() {
    let env = env();
    env.exec(r#"SetChatWindowName(1, "General Probe"); SetChatWindowShown(1, true); SetChatWindowDocked(1, 1); SetChatWindowSavedDimensions(1, 410, 120); SetChatWindowSavedPosition(1, "BOTTOMLEFT", 0.1, 0.2); SetChatWindowName(2, "Combat Probe"); SetChatWindowShown(2, false); SetChatWindowDocked(2, nil); SetChatWindowSavedDimensions(2, 520, 240); SetChatWindowSavedPosition(2, "TOPRIGHT", 0.3, 0.4)"#).unwrap();
    let values: (String, bool, bool, String, bool, bool) = env
        .eval("local a=GetChatWindowInfo(1); local _,_,_,_,_,_,s1,_,d1=GetChatWindowInfo(1); local b=GetChatWindowInfo(2); local _,_,_,_,_,_,s2,_,d2=GetChatWindowInfo(2); return a,s1,d1==1,b,s2,d2==nil")
        .unwrap();
    assert_eq!(values, ("General Probe".into(), true, true, "Combat Probe".into(), false, true));
    let geometry: (f64, f64, String, f64, f64) = env.eval("local w,h=GetChatWindowSavedDimensions(2); local p,x,y=GetChatWindowSavedPosition(2); return w,h,p,x,y").unwrap();
    assert_eq!((geometry.0, geometry.1, geometry.2.as_str()), (520.0, 240.0, "TOPRIGHT"));
    assert!((geometry.3 - 0.3).abs() < 1e-6);
    assert!((geometry.4 - 0.4).abs() < 1e-6);
    let model = env.state().borrow();
    assert_eq!(model.chat_windows[&1].name, "General Probe");
    assert_eq!(model.chat_windows[&2].name, "Combat Probe");
}

// ── ChangeChatColor ───────────────────────────────────────────────────────────

#[test]
fn change_chat_color_stores_uppercase_channel_key() {
    let env = env();
    env.exec(r#"ChangeChatColor("say", 1, 0, 0)"#).unwrap();
    let st = env.state().borrow();
    let color = st.chat_type_colors.get("SAY").copied().unwrap_or_default();
    assert!((color.0 - 1.0).abs() < 1e-6);
    assert!((color.1 - 0.0).abs() < 1e-6);
    assert!((color.2 - 0.0).abs() < 1e-6);
}

// ── GetChatWindowChannels ─────────────────────────────────────────────────────

#[test]
fn get_chat_window_channels_returns_flat_list() {
    let env = env();
    env.exec(
        r#"AddChatWindowChannel(1, "Trade")
           AddChatWindowChannel(1, "General")"#,
    )
    .unwrap();
    let (a, b): (String, String) = env
        .eval(
            r#"
            local t = GetChatWindowChannels(1)
            return t[1], t[2]
            "#,
        )
        .unwrap();
    assert_eq!(a, "Trade");
    assert_eq!(b, "General");
}

#[test]
fn get_chat_window_channels_empty_for_unknown_window() {
    let env = env();
    let count: i64 = env.eval("return #GetChatWindowChannels(99)").unwrap();
    assert_eq!(count, 0);
}

// ── GetChatWindowMessages ─────────────────────────────────────────────────────

#[test]
fn get_chat_window_messages_returns_subscribed_types() {
    let env = env();
    {
        let mut st = env.state().borrow_mut();
        let w = st.chat_windows.entry(1).or_default();
        w.messages.push("SAY".into());
        w.messages.push("YELL".into());
    }
    let (a, b): (String, String) = env
        .eval(
            r#"
            local t = GetChatWindowMessages(1)
            return t[1], t[2]
            "#,
        )
        .unwrap();
    assert_eq!(a, "SAY");
    assert_eq!(b, "YELL");
}
