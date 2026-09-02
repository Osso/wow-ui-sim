//! Behavior probes for APIDocumentation open-dump link output.

use crate::common::blizzard_addon_harness::{
    load_blizzard_addon_closure_into_env, with_blizzard_addon_startup_shape,
};
use crate::common::panel_fixtures::{
    blizzard_ui_dir, clear_recorded_lua_errors, recorded_lua_errors,
};

const ROOT: &str = "Blizzard_APIDocumentation";
const DUMP_TEXT: &str = "/dump GetTime()";

#[test]
fn opendump_link_records_chat_edit_dump_command() {
    with_blizzard_addon_startup_shape(&[], &[], |env, _loaded| {
        load_api_documentation(env);
        seed_active_chat_edit_window(env);

        let generated_link_uses_api_payload: bool = env
            .eval(
                r#"
                APIDocumentation:AddDocumentationTable({
                    Name = "DumpSystem",
                    Type = "System",
                    Namespace = "",
                    Tables = {},
                    Functions = {
                        {
                            Name = "GetTime",
                            Type = "Function",
                            Arguments = {},
                            Returns = {},
                        },
                    },
                    Events = {},
                })

                local apiInfo = APIDocumentation.functions[1]
                local generatedLink = apiInfo:GenerateAPILink()
                local generatedPayload = generatedLink:match("|H([^|]+)|h")

                APIDocumentation:HandleAPILink(
                    generatedPayload,
                    APIDocumentation.Commands.OpenDump
                )

                return generatedLink:find("|Hapi:function:GetTime:", 1, true) ~= nil
                "#,
            )
            .expect("APIDocumentation open-dump link probe must run cleanly");

        let (editbox_is_shown, editbox_has_focus, editbox_text, desired_cursor_position): (
            bool,
            bool,
            String,
            i64,
        ) = env
            .eval(
                r#"
                local editBox = ChatFrameUtil.GetActiveWindow()
                return editBox ~= nil and editBox:IsShown(),
                    editBox ~= nil and editBox:HasFocus(),
                    editBox and editBox.text,
                    editBox and editBox.desiredCursorPosition
                "#,
            )
            .expect("OpenDump must seed the active chat edit window");

        assert!(
            generated_link_uses_api_payload,
            "generated APIDocumentation links must use the `api:function:GetTime:` payload"
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
            "OpenDump link must include the target function call"
        );
        assert_eq!(
            (DUMP_TEXT.len() - 1) as i64,
            desired_cursor_position,
            "OpenDump parks the cursor just before the closing parenthesis"
        );
    });
}

fn load_api_documentation(env: &wow_ui_sim::lua_api::WowLuaEnv) {
    clear_recorded_lua_errors(env);
    let ui_dir = blizzard_ui_dir();
    let loaded =
        load_blizzard_addon_closure_into_env(env, &ui_dir, &["Blizzard_ChatFrameBase", ROOT], &[]);

    assert!(
        loaded.iter().any(|addon| addon == ROOT),
        "{ROOT} must be included in the loaded addon closure; loaded={loaded:?}"
    );

    let errors = recorded_lua_errors(env);
    assert!(
        errors.is_empty(),
        "{ROOT} must settle without recorded Lua errors:\n  {}",
        errors.join("\n  ")
    );
}

fn seed_active_chat_edit_window(env: &wow_ui_sim::lua_api::WowLuaEnv) {
    env.exec(
        r#"
        local chatFrame = CreateFrame("Frame", "APIDocumentationChatFrame", UIParent)
        local editBox = CreateFrame(
            "EditBox",
            "APIDocumentationChatFrameEditBox",
            chatFrame
        )
        editBox.header = CreateFrame("Frame", nil, editBox)
        editBox.chatFrame = chatFrame

        function editBox:UpdateNewcomerEditBoxHint() end
        function editBox:SetFocusRegionsShown() end
        function editBox:UpdateHeader() end

        chatFrame.editBox = editBox
        DEFAULT_CHAT_FRAME = chatFrame
        GENERAL_CHAT_DOCK = {}

        function FCFDock_GetSelectedWindow()
            return chatFrame
        end

        function GetCVar(name)
            if name == "chatStyle" then
                return "classic"
            end
        end
        "#,
    )
    .expect("APIDocumentation fixture must seed an active ChatFrameBase edit window");
}
