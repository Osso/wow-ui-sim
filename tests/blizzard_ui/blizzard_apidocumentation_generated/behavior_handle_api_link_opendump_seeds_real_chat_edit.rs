//! Behavior probes for generated APIDocumentation open-dump links.

use crate::common::blizzard_addon_harness::load_blizzard_addon_closure_into_env;
use crate::common::blizzard_addon_harness::new_blizzard_addon_env;
use crate::common::panel_fixtures::{
    blizzard_ui_dir, clear_recorded_lua_errors, load_panel_addons, recorded_lua_errors,
};

const ROOT: &str = "Blizzard_APIDocumentationGenerated";
const DUMP_TEXT: &str = "/dump GetTime()";

#[test]
fn opendump_link_seeds_generated_get_time_in_real_chat_editbox() {
    let env = load_generated_api_documentation();

    let generated_link_uses_get_time_payload: bool = env
        .eval(
            r#"
            local apiInfo = APIDocumentation:FindAPIByName("function", "GetTime", "SystemTime")
            local generatedLink = apiInfo:GenerateAPILink()
            local generatedPayload = generatedLink:match("|H([^|]+)|h")

            APIDocumentation:HandleAPILink(
                generatedPayload,
                APIDocumentation.Commands.OpenDump
            )

            return generatedLink:find("|Hapi:function:GetTime:SystemTime", 1, true) ~= nil
            "#,
        )
        .expect("generated APIDocumentation open-dump link probe must run cleanly");

    let editbox_is_shown: bool = env
        .eval(
            r#"
            local editBox = ChatFrameUtil.GetActiveWindow()
            return editBox ~= nil and editBox:IsShown()
            "#,
        )
        .expect("active chat editbox visibility probe must run cleanly");
    let editbox_has_focus: bool = env
        .eval(
            r#"
            local editBox = ChatFrameUtil.GetActiveWindow()
            return editBox ~= nil and editBox:HasFocus()
            "#,
        )
        .expect("active chat editbox focus probe must run cleanly");
    let editbox_text: String = env
        .eval("return ChatFrameUtil.GetActiveWindow().text")
        .expect("active chat editbox text probe must run cleanly");
    let desired_cursor_position: i64 = env
        .eval("return ChatFrameUtil.GetActiveWindow().desiredCursorPosition")
        .expect("active chat editbox cursor probe must run cleanly");

    assert!(
        generated_link_uses_get_time_payload,
        "generated API links must include the real GetTime SystemTime payload"
    );
    assert!(
        editbox_is_shown,
        "OpenDump link must show the active real ChatFrameUtil editbox"
    );
    assert!(
        editbox_has_focus,
        "OpenDump link must focus the active real ChatFrameUtil editbox"
    );
    assert_eq!(
        DUMP_TEXT, editbox_text,
        "OpenDump link must include the generated target function call"
    );
    assert_eq!(
        (DUMP_TEXT.len() - 1) as i64,
        desired_cursor_position,
        "OpenDump parks the cursor just before the closing parenthesis"
    );
}

fn load_generated_api_documentation() -> wow_ui_sim::lua_api::WowLuaEnv {
    let ui_dir = blizzard_ui_dir();
    let env = new_blizzard_addon_env(&ui_dir);
    load_panel_addons(&env);
    clear_recorded_lua_errors(&env);

    let loaded =
        load_blizzard_addon_closure_into_env(&env, &ui_dir, &["Blizzard_ChatFrameBase", ROOT], &[]);
    assert!(
        loaded.iter().any(|addon| addon == ROOT),
        "{ROOT} must be included in the loaded addon closure; loaded={loaded:?}"
    );

    let errors = recorded_lua_errors(&env);
    assert!(
        errors.is_empty(),
        "{ROOT} must load without recorded Lua errors:\n  {}",
        errors.join("\n  ")
    );

    env
}
