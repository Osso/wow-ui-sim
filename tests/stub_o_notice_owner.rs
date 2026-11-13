//! Original CodeOfConduct callback must retain its observable chat-history effect.
//! This is a retirement RED for the notice-suppression source rewrite, not a
//! replacement callback or a native-behavior inference.
#![cfg(all(feature = "optional-tests", feature = "client-retail"))]

use wow_ui_sim::lua_api::WowLuaEnv;

fn assert_notice_path_errors_absent(env: &WowLuaEnv, phase: &str) {
    let state = env.state().borrow();
    let errors: Vec<_> = state
        .lua_error_records
        .iter()
        .filter(|record| {
            ["Blizzard_CodeOfConduct", "Blizzard_ChatFrameBase"]
                .iter()
                .any(|owner| record.addon_name.as_deref() == Some(*owner))
                || [
                    "Blizzard_CodeOfConduct.lua",
                    "ChatFrameUtil.lua",
                    "ScrollingMessageFrame.lua",
                ]
                .iter()
                .any(|path| record.message.contains(path))
        })
        .map(|record| (&record.addon_name, &record.message))
        .collect();
    assert!(
        errors.is_empty(),
        "{phase}: notice/chat owner errors: {errors:?}"
    );
}

fn notice_count(env: &WowLuaEnv) -> i32 {
    env.eval(
        r#"
        local count = 0
        for i = 1, DEFAULT_CHAT_FRAME:GetNumMessages() do
            local text = DEFAULT_CHAT_FRAME:GetMessageInfo(i)
            if text == ONLINE_SAFETY_NOTICE then count = count + 1 end
        end
        return count
        "#,
    )
    .expect("read actual original scrolling-message-frame history")
}

#[test]
fn stub_o_notice_original_initial_login_callback_adds_chat_history() {
    let ui = crate::common::blizzard_ui_dir();
    // Selected Game TOC, default enabled, with an unconditional Lua entry.
    // Do not directly eval the original file: exercise normal addon loading and
    // therefore the production source-rewrite path under test.
    let toc = std::fs::read_to_string(ui.join("Blizzard_CodeOfConduct/Blizzard_CodeOfConduct.toc"))
        .unwrap();
    assert!(toc.contains("## AllowLoad: game"));
    assert!(toc.contains("## DefaultState: enabled"));
    assert!(
        toc.lines()
            .any(|line| line.trim() == "Blizzard_CodeOfConduct.lua")
    );
    let original =
        std::fs::read_to_string(ui.join("Blizzard_CodeOfConduct/Blizzard_CodeOfConduct.lua"))
            .unwrap();
    assert!(original.contains("RegisterFrameEventAndCallback"));
    assert!(original.contains("ChatFrameUtil.AddSystemMessage(ONLINE_SAFETY_NOTICE);"));

    wow_ui_sim::xml::register_intrinsic_templates();
    let (env, loaded) = crate::common::blizzard_addon_harness::build_blizzard_addon_closure_env(
        &ui,
        &["Blizzard_CodeOfConduct"],
        &[],
    );
    assert!(loaded.iter().any(|name| name == "Blizzard_CodeOfConduct"));
    assert!(loaded.iter().any(|name| name == "Blizzard_ChatFrameBase"));
    println!("O notice selected Retail closure: {loaded:?}");
    // Retain and print dependency diagnostics; never clear or stub away failures.
    for record in &env.state().borrow().lua_error_records {
        println!(
            "O notice load diagnostic {:?}: {}",
            record.addon_name, record.message
        );
    }
    assert_notice_path_errors_absent(&env, "load");
    env.exec(
        r#"
        assert(type(ONLINE_SAFETY_NOTICE) == "string" and #ONLINE_SAFETY_NOTICE > 0,
            "real localized online-safety notice is required")
        assert(DEFAULT_CHAT_FRAME == ChatFrame1 and ChatFrame1.historyBuffer,
            "original selected ChatFrame XML/OnLoad must own the actual history")
        local source = debug.getinfo(ChatFrameUtil.AddSystemMessage, "S").source
        assert(source:find("Blizzard_ChatFrameBase/Shared/ChatFrameUtil.lua", 1, true),
            "must call original ChatFrameUtil, not a substitute")
        local historySource = debug.getinfo(DEFAULT_CHAT_FRAME.GetMessageInfo, "S").source
        assert(historySource:find("ScrollingMessageFrame.lua", 1, true),
            "must inspect original scrolling-frame history, not a fake collector")
        "#,
    )
    .expect("original notice and chat-owner prerequisites");

    let before = notice_count(&env);
    env.fire_event_with_args(
        "PLAYER_ENTERING_WORLD",
        &[rilua::Val::Bool(true), rilua::Val::Bool(false)],
    )
    .expect("dispatch actual initial-login frame event");
    assert_notice_path_errors_absent(&env, "initial login");
    assert_eq!(
        notice_count(&env),
        before + 1,
        "original initial-login callback must append ONLINE_SAFETY_NOTICE; suppression is not allowed"
    );

    env.fire_event_with_args(
        "PLAYER_ENTERING_WORLD",
        &[rilua::Val::Bool(false), rilua::Val::Bool(true)],
    )
    .expect("dispatch actual reload frame event");
    assert_notice_path_errors_absent(&env, "reload");
    assert_eq!(
        notice_count(&env),
        before + 1,
        "original callback must not append an additional notice on reload"
    );
}
