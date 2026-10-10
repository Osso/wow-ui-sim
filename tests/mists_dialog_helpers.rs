#![cfg(feature = "client-mists")]

use wow_ui_sim::lua_api::WowLuaEnv;

fn money_frame_lua() -> String {
    std::fs::read_to_string(
        wow_ui_sim::client_profile::blizzard_ui_addons_dir_under(std::path::Path::new(env!(
            "CARGO_MANIFEST_DIR"
        )))
        .join("Blizzard_MoneyFrame/Classic/MoneyFrame.lua"),
    )
    .expect("Mists MoneyFrame Lua should be available in the profile UI source")
}

fn load_money_frame_lua(env: &WowLuaEnv) {
    let loader = format!(
        r#"
        local source = [==[
{}
        ]==]
        local chunk = assert(loadstring(source))
        chunk("Blizzard_MoneyFrame", {{ MoneyTypeInfo = {{}} }})
        "#,
        money_frame_lua()
    );

    env.exec(&loader)
        .expect("Mists MoneyFrame Lua should define money helpers");
}

#[test]
fn money_frame_set_type_reproduces_missing_basic_message_dialog_helper() {
    let env = WowLuaEnv::new().expect("Lua environment should initialize");
    load_money_frame_lua(&env);

    // Exclude bootstrap/source-load observations from the fault-injection interval.
    let previous_accesses = std::mem::take(&mut env.state().borrow_mut().nil_symbol_accesses);
    let result = env.eval::<(bool, String)>(
        r#"
        rawset(_G, "SetBasicMessageDialogText", nil)
        local handler = MoneyFrame_SetType
        local frame = {}
        local previousDedupe = rawget(_G, "__wow_logged_nil_symbols")
        rawset(_G, "__wow_logged_nil_symbols", {})
        local ok, err = pcall(handler, frame, "INVALID")
        rawset(_G, "__wow_logged_nil_symbols", previousDedupe)
        return ok, tostring(err)
        "#,
    );
    let accesses = std::mem::replace(
        &mut env.state().borrow_mut().nil_symbol_accesses,
        previous_accesses,
    );
    let (ok, err) = result.expect("MoneyFrame_SetType pcall should return a status");

    assert!(!ok, "MoneyFrame_SetType should reproduce the nil global");
    assert!(
        err.contains("attempt to call a nil value"),
        "expected a nil-call failure, got: {err}"
    );
    assert_eq!(
        accesses.len(),
        1,
        "expected only the injected global lookup during MoneyFrame_SetType: {accesses:?}"
    );
    assert_eq!(accesses[0].container, "_G");
    assert_eq!(accesses[0].key, "SetBasicMessageDialogText");
}

#[test]
fn basic_message_dialog_helper_updates_text() {
    let env = WowLuaEnv::new().expect("Lua environment should initialize");

    let result: (String, i32, String, i32, String, i32) = env
        .eval(
            r#"
            local textValue = nil
            local shown = false
            local showCount = 0
            BasicMessageDialog = {
                IsShown = function()
                    return shown
                end,
                Show = function()
                    shown = true
                    showCount = showCount + 1
                end,
                Text = {
                    SetText = function(_self, text)
                        textValue = text
                    end,
                },
            }

            SetBasicMessageDialogText("Invalid money type: TEST")
            local firstText = textValue
            local firstShowCount = showCount

            SetBasicMessageDialogText("Ignored while shown")
            local secondText = textValue
            local secondShowCount = showCount

            SetBasicMessageDialogText("Forced replacement", true)
            local thirdText = textValue
            local thirdShowCount = showCount

            return firstText, firstShowCount, secondText, secondShowCount, thirdText, thirdShowCount
            "#,
        )
        .expect("SetBasicMessageDialogText should mutate BasicMessageDialog.Text");

    assert_eq!(
        result,
        (
            "Invalid money type: TEST".to_string(),
            1,
            "Invalid money type: TEST".to_string(),
            1,
            "Forced replacement".to_string(),
            2
        ),
        "SetBasicMessageDialogText should update and show only when hidden or forced"
    );
}
