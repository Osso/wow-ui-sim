//! OnUpdate dispatch within one pass: handlers run in frame order with the
//! frame and elapsed time, hooks run after the script, and bindings added by
//! an earlier handler take effect for later frames in the same pass.

use super::*;

const SETUP: &str = r#"
    PassLog = {}
    PassFirst = CreateFrame("Frame", "PassFirst", UIParent)
    PassSecond = CreateFrame("Frame", "PassSecond", UIParent)
    PassFirst:SetScript("OnUpdate", function(self, elapsed)
        table.insert(PassLog, self:GetName() .. ":" .. string.format("%.3f", elapsed))
        if not PassHooked then
            PassHooked = true
            PassSecond:HookScript("OnUpdate", function(frame)
                table.insert(PassLog, frame:GetName() .. ":hook")
            end)
        end
    end)
    PassSecond:SetScript("OnUpdate", function(self)
        table.insert(PassLog, self:GetName() .. ":script")
    end)
"#;

#[test]
fn hook_added_by_an_earlier_handler_runs_in_the_same_pass() {
    let (t, _) = load_test_lua("on-update-pass", SETUP);
    t.env.exec("PassLog = {}").unwrap();

    t.env.fire_on_update(0.25).unwrap();

    let log: String = t.env.eval("return table.concat(PassLog, ',')").unwrap();
    assert_eq!(log, "PassFirst:0.250,PassSecond:script,PassSecond:hook");
}

#[test]
fn on_update_error_names_frame_and_handler_source() {
    let (t, _) = load_test_lua(
        "on-update-error",
        r#"
        PassBroken = CreateFrame("Frame", "PassBroken", UIParent)
        PassBroken:SetScript("OnUpdate", function() error("pass boom") end)
    "#,
    );

    t.env.fire_on_update(0.016).unwrap();

    let errors = t.env.state().borrow().lua_errors.clone();
    let message = errors
        .iter()
        .find(|e| e.contains("pass boom"))
        .unwrap_or_else(|| panic!("no pass boom error in {errors:?}"));
    assert!(message.contains("[OnUpdate] frame=PassBroken"), "{message}");
}

/// OnPostUpdate is not settable from Lua on common widgets, so install it
/// through the same registry binding path that SetScript uses.
fn install_on_post_update(t: &TestCtx, frame_name: &str, handler_global: &str) {
    let frame_id = t
        .env
        .state()
        .borrow()
        .widgets
        .get_id_by_name(frame_name)
        .unwrap();
    let mut lua = t.env.rilua_mut();
    let state = lua.state_mut();
    let chunk = state.load(&format!("return {handler_global}")).unwrap();
    let results = crate::lua_api::script_helpers::protected_call_state(
        state,
        rilua::Val::Function(chunk.gc_ref()),
        &[],
    )
    .unwrap();
    let handler = results[0];
    crate::lua_api::script_helpers::set_script(state, frame_id, "OnPostUpdate", handler);
}

#[test]
fn on_post_update_runs_after_on_update_for_frames_that_have_it() {
    let (t, _) = load_test_lua(
        "on-post-update",
        r#"
        PostLog = {}
        PostBoth = CreateFrame("Frame", "PostBoth", UIParent)
        PostBoth:SetScript("OnUpdate", function() table.insert(PostLog, "both:update") end)
        PostBothPost = function(self, elapsed)
            table.insert(PostLog, "both:post:" .. string.format("%.2f", elapsed))
        end
        PostOnly = CreateFrame("Frame", "PostOnly", UIParent)
        PostOnlyPost = function() table.insert(PostLog, "only:post") end
        PostNone = CreateFrame("Frame", "PostNone", UIParent)
        PostNone:SetScript("OnUpdate", function() table.insert(PostLog, "none:update") end)
    "#,
    );
    install_on_post_update(&t, "PostBoth", "PostBothPost");
    install_on_post_update(&t, "PostOnly", "PostOnlyPost");
    t.env.exec("PostLog = {}").unwrap();

    t.env.fire_on_update(0.5).unwrap();

    let log: String = t.env.eval("return table.concat(PostLog, ',')").unwrap();
    assert_eq!(log, "both:update,none:update,both:post:0.50,only:post");
}
