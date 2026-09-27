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
