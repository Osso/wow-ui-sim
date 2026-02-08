//! Tests for keyboard input handling.
//!
//! Tests for key press simulation via `WowLuaEnv::send_key_press`.

use wow_ui_sim::lua_api::WowLuaEnv;

// --- Existing GameMenuFrame toggle tests ---

#[test]
fn test_escape_shows_game_menu() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(
        r#"
        GameMenuFrame = CreateFrame("Frame", "GameMenuFrame", UIParent)
        GameMenuFrame:Hide()
    "#,
    )
    .unwrap();

    let shown: bool = env.eval("return GameMenuFrame:IsShown()").unwrap();
    assert!(!shown, "GameMenuFrame should start hidden");

    env.send_key_press("ESCAPE", None).unwrap();

    let shown: bool = env.eval("return GameMenuFrame:IsShown()").unwrap();
    assert!(shown, "Escape should show GameMenuFrame");
}

#[test]
fn test_escape_hides_game_menu_when_shown() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(
        r#"
        GameMenuFrame = CreateFrame("Frame", "GameMenuFrame", UIParent)
        GameMenuFrame:Show()
    "#,
    )
    .unwrap();

    let shown: bool = env.eval("return GameMenuFrame:IsShown()").unwrap();
    assert!(shown, "GameMenuFrame should start shown");

    env.send_key_press("ESCAPE", None).unwrap();

    let shown: bool = env.eval("return GameMenuFrame:IsShown()").unwrap();
    assert!(!shown, "Escape should hide GameMenuFrame");
}

#[test]
fn test_escape_toggles_game_menu_twice() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(
        r#"
        GameMenuFrame = CreateFrame("Frame", "GameMenuFrame", UIParent)
        GameMenuFrame:Hide()
    "#,
    )
    .unwrap();

    env.send_key_press("ESCAPE", None).unwrap();
    let shown: bool = env.eval("return GameMenuFrame:IsShown()").unwrap();
    assert!(shown, "First Escape should show menu");

    env.send_key_press("ESCAPE", None).unwrap();
    let shown: bool = env.eval("return GameMenuFrame:IsShown()").unwrap();
    assert!(!shown, "Second Escape should hide menu");
}

#[test]
fn test_escape_without_game_menu_frame() {
    let env = WowLuaEnv::new().unwrap();

    // Should not error when GameMenuFrame doesn't exist
    env.send_key_press("ESCAPE", None).unwrap();
}

// --- OnKeyDown tests ---

#[test]
fn test_set_script_on_key_down() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(
        r#"
        _G.key_pressed = nil
        local f = CreateFrame("Frame", "KeyTestFrame", UIParent)
        f:EnableKeyboard(true)
        f:Show()
        f:SetScript("OnKeyDown", function(self, key)
            _G.key_pressed = key
        end)
    "#,
    )
    .unwrap();

    env.send_key_press("A", None).unwrap();

    let key: String = env.eval("return _G.key_pressed").unwrap();
    assert_eq!(key, "A");
}

#[test]
fn test_enable_keyboard_gates_on_key_down() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(
        r#"
        _G.key_fired = false
        local f = CreateFrame("Frame", "KeyGateFrame", UIParent)
        f:Show()
        -- keyboard NOT enabled
        f:SetScript("OnKeyDown", function(self, key)
            _G.key_fired = true
        end)
    "#,
    )
    .unwrap();

    env.send_key_press("A", None).unwrap();

    let fired: bool = env.eval("return _G.key_fired").unwrap();
    assert!(!fired, "OnKeyDown should not fire when keyboard is disabled");

    // Now enable keyboard and try again
    env.exec("KeyGateFrame:EnableKeyboard(true)").unwrap();
    env.send_key_press("A", None).unwrap();

    let fired: bool = env.eval("return _G.key_fired").unwrap();
    assert!(fired, "OnKeyDown should fire after EnableKeyboard(true)");
}

#[test]
fn test_propagate_keyboard_input() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(
        r#"
        _G.parent_key = nil
        _G.child_key = nil
        local parent = CreateFrame("Frame", "PropParent", UIParent)
        parent:EnableKeyboard(true)
        parent:Show()
        parent:SetScript("OnKeyDown", function(self, key)
            _G.parent_key = key
        end)

        local child = CreateFrame("Frame", "PropChild", parent)
        child:EnableKeyboard(true)
        child:SetPropagateKeyboardInput(true)
        child:Show()
        child:SetScript("OnKeyDown", function(self, key)
            _G.child_key = key
        end)

        -- Focus the child so it receives input first
        child:SetFocus()
    "#,
    )
    .unwrap();

    env.send_key_press("X", None).unwrap();

    let child_key: String = env.eval("return _G.child_key").unwrap();
    assert_eq!(child_key, "X", "Child should receive OnKeyDown");

    let parent_key: String = env.eval("return _G.parent_key").unwrap();
    assert_eq!(parent_key, "X", "Parent should receive propagated OnKeyDown");
}

#[test]
fn test_propagate_keyboard_input_stops_without_flag() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(
        r#"
        _G.parent_key = nil
        local parent = CreateFrame("Frame", "NoPropParent", UIParent)
        parent:EnableKeyboard(true)
        parent:Show()
        parent:SetScript("OnKeyDown", function(self, key)
            _G.parent_key = key
        end)

        local child = CreateFrame("Frame", "NoPropChild", parent)
        child:EnableKeyboard(true)
        child:Show()
        -- propagate is false by default
        child:SetScript("OnKeyDown", function() end)

        child:SetFocus()
    "#,
    )
    .unwrap();

    env.send_key_press("Y", None).unwrap();

    let parent_key: mlua::Value = env.eval("return _G.parent_key").unwrap();
    assert!(
        matches!(parent_key, mlua::Value::Nil),
        "Parent should NOT receive OnKeyDown without propagation"
    );
}

// --- EditBox special handler tests ---

#[test]
fn test_editbox_on_escape_pressed() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(
        r#"
        _G.escape_handled = false
        GameMenuFrame = CreateFrame("Frame", "GameMenuFrame", UIParent)
        GameMenuFrame:Hide()

        local eb = CreateFrame("EditBox", "EscEditBox", UIParent)
        eb:SetFocus()
        eb:SetScript("OnEscapePressed", function(self)
            _G.escape_handled = true
            return true  -- consume the event
        end)
    "#,
    )
    .unwrap();

    env.send_key_press("ESCAPE", None).unwrap();

    let handled: bool = env.eval("return _G.escape_handled").unwrap();
    assert!(handled, "OnEscapePressed should fire on focused EditBox");

    let menu_shown: bool = env.eval("return GameMenuFrame:IsShown()").unwrap();
    assert!(
        !menu_shown,
        "GameMenuFrame should NOT toggle when EditBox consumes Escape"
    );
}

#[test]
fn test_editbox_escape_not_consumed_falls_through() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(
        r#"
        _G.escape_fired = false
        GameMenuFrame = CreateFrame("Frame", "GameMenuFrame", UIParent)
        GameMenuFrame:Hide()

        local eb = CreateFrame("EditBox", "FallEditBox", UIParent)
        eb:SetFocus()
        eb:SetScript("OnEscapePressed", function(self)
            _G.escape_fired = true
            -- return nil/false → does not consume
        end)
    "#,
    )
    .unwrap();

    env.send_key_press("ESCAPE", None).unwrap();

    let fired: bool = env.eval("return _G.escape_fired").unwrap();
    assert!(fired, "OnEscapePressed should still fire");

    let menu_shown: bool = env.eval("return GameMenuFrame:IsShown()").unwrap();
    assert!(
        menu_shown,
        "GameMenuFrame should toggle when EditBox doesn't consume Escape"
    );
}

#[test]
fn test_key_press_enter() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(
        r#"
        _G.enter_pressed = false
        local eb = CreateFrame("EditBox", "EnterEditBox", UIParent)
        eb:SetFocus()
        eb:SetScript("OnEnterPressed", function(self)
            _G.enter_pressed = true
        end)
    "#,
    )
    .unwrap();

    env.send_key_press("ENTER", None).unwrap();

    let pressed: bool = env.eval("return _G.enter_pressed").unwrap();
    assert!(pressed, "OnEnterPressed should fire on focused EditBox");
}

// --- CloseSpecialWindows tests ---

#[test]
fn test_close_special_windows() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(
        r#"
        GameMenuFrame = CreateFrame("Frame", "GameMenuFrame", UIParent)
        GameMenuFrame:Hide()

        -- Create a special frame and register it
        local sf = CreateFrame("Frame", "SpecialTestFrame", UIParent)
        sf:Show()
        UISpecialFrames = UISpecialFrames or {}
        table.insert(UISpecialFrames, "SpecialTestFrame")
    "#,
    )
    .unwrap();

    let visible: bool = env.eval("return SpecialTestFrame:IsShown()").unwrap();
    assert!(visible, "Special frame should start visible");

    env.send_key_press("ESCAPE", None).unwrap();

    let visible: bool = env.eval("return SpecialTestFrame:IsShown()").unwrap();
    assert!(!visible, "Special frame should be hidden by Escape");

    let menu_shown: bool = env.eval("return GameMenuFrame:IsShown()").unwrap();
    assert!(
        !menu_shown,
        "GameMenuFrame should NOT toggle when special frames were closed"
    );
}

// --- Escape priority chain test ---

#[test]
fn test_escape_priority_chain() {
    let env = WowLuaEnv::new().unwrap();

    // Set up all three layers: EditBox, special frame, GameMenuFrame
    env.exec(
        r#"
        _G.escape_order = {}
        GameMenuFrame = CreateFrame("Frame", "GameMenuFrame", UIParent)
        GameMenuFrame:Hide()

        local sf = CreateFrame("Frame", "PrioritySpecialFrame", UIParent)
        sf:Show()
        UISpecialFrames = UISpecialFrames or {}
        table.insert(UISpecialFrames, "PrioritySpecialFrame")

        local eb = CreateFrame("EditBox", "PriorityEditBox", UIParent)
        eb:SetFocus()
        eb:SetScript("OnEscapePressed", function(self)
            table.insert(_G.escape_order, "editbox")
            return true  -- consume
        end)
    "#,
    )
    .unwrap();

    // First Escape: EditBox consumes it
    env.send_key_press("ESCAPE", None).unwrap();

    let count: i32 = env.eval("return #_G.escape_order").unwrap();
    assert_eq!(count, 1, "One entry in escape_order after first Escape");
    let first: String = env.eval("return _G.escape_order[1]").unwrap();
    assert_eq!(first, "editbox", "First Escape consumed by EditBox");

    let sf_visible: bool = env
        .eval("return PrioritySpecialFrame:IsShown()")
        .unwrap();
    assert!(sf_visible, "Special frame still visible after EditBox consumed Escape");

    let menu_shown: bool = env.eval("return GameMenuFrame:IsShown()").unwrap();
    assert!(!menu_shown, "GameMenuFrame still hidden");

    // Remove EditBox handler (don't consume), clear focus
    env.exec(
        r#"
        PriorityEditBox:ClearFocus()
    "#,
    )
    .unwrap();

    // Second Escape: special frame gets closed
    env.send_key_press("ESCAPE", None).unwrap();

    let sf_visible: bool = env
        .eval("return PrioritySpecialFrame:IsShown()")
        .unwrap();
    assert!(!sf_visible, "Special frame hidden by second Escape");

    let menu_shown: bool = env.eval("return GameMenuFrame:IsShown()").unwrap();
    assert!(!menu_shown, "GameMenuFrame still hidden (special frame consumed)");

    // Third Escape: nothing else to close, GameMenuFrame toggles
    env.send_key_press("ESCAPE", None).unwrap();

    let menu_shown: bool = env.eval("return GameMenuFrame:IsShown()").unwrap();
    assert!(menu_shown, "Third Escape should toggle GameMenuFrame");
}

// --- EditBox click-to-type integration test ---

#[test]
fn test_click_editbox_type_message_and_submit() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(
        r#"
        _G.received_message = nil

        local eb = CreateFrame("EditBox", "ChatEditBox", UIParent)
        eb:SetSize(200, 30)
        eb:SetScript("OnEnterPressed", function(self)
            _G.received_message = self:GetText()
            self:SetText("")
        end)
    "#,
    )
    .unwrap();

    // EditBox should not have focus yet
    let has_focus: bool = env.eval("return ChatEditBox:HasFocus()").unwrap();
    assert!(!has_focus, "EditBox should not have focus before click");

    // Click the EditBox to give it focus
    let frame_id = env
        .state()
        .borrow()
        .widgets
        .get_id_by_name("ChatEditBox")
        .expect("ChatEditBox should exist");
    env.send_click(frame_id).unwrap();

    let has_focus: bool = env.eval("return ChatEditBox:HasFocus()").unwrap();
    assert!(has_focus, "EditBox should have focus after click");

    // Type "hello" character by character
    for ch in "hello".chars() {
        let s = ch.to_string();
        env.send_key_press(&s.to_uppercase(), Some(&s)).unwrap();
    }

    let text: String = env.eval("return ChatEditBox:GetText()").unwrap();
    assert_eq!(text, "hello", "EditBox should contain typed text");

    // Press Enter to submit
    env.send_key_press("ENTER", None).unwrap();

    let received: String = env.eval("return _G.received_message").unwrap();
    assert_eq!(received, "hello", "OnEnterPressed should receive the message");

    // EditBox should be cleared after submit
    let text_after: String = env
        .eval("return ChatEditBox:GetText() or ''")
        .unwrap();
    assert_eq!(text_after, "", "EditBox should be cleared after submit");
}

// --- Targeting system tests ---

#[test]
fn test_f1_targets_player() {
    let env = WowLuaEnv::new().unwrap();

    let exists_before: bool = env.eval("return UnitExists('target')").unwrap();
    assert!(!exists_before, "No target should exist initially");

    env.send_key_press("F1", None).unwrap();

    let exists: bool = env.eval("return UnitExists('target')").unwrap();
    assert!(exists, "Target should exist after F1");

    let name: String = env.eval("return UnitName('target')").unwrap();
    assert_eq!(name, "SimPlayer", "F1 should target player");
}

#[test]
fn test_f2_targets_party1() {
    let env = WowLuaEnv::new().unwrap();

    env.send_key_press("F2", None).unwrap();

    let exists: bool = env.eval("return UnitExists('target')").unwrap();
    assert!(exists, "Target should exist after F2");

    let name: String = env.eval("return UnitName('target')").unwrap();
    assert_eq!(name, "Thrynn", "F2 should target party member 1");

    let is_player: bool = env.eval("return UnitIsPlayer('target')").unwrap();
    assert!(is_player, "Party target should be a player");
}

#[test]
fn test_f6_targets_enemy() {
    let env = WowLuaEnv::new().unwrap();

    env.send_key_press("F6", None).unwrap();

    let exists: bool = env.eval("return UnitExists('target')").unwrap();
    assert!(exists, "Target should exist after F6");

    let name: String = env.eval("return UnitName('target')").unwrap();
    assert_eq!(name, "Hogger", "F6 should target enemy NPC");

    let is_enemy: bool = env.eval("return UnitIsEnemy('player', 'target')").unwrap();
    assert!(is_enemy, "Enemy target should be an enemy");

    let is_player: bool = env.eval("return UnitIsPlayer('target')").unwrap();
    assert!(!is_player, "Enemy NPC should not be a player");
}

#[test]
fn test_tab_targets_enemy() {
    let env = WowLuaEnv::new().unwrap();

    env.send_key_press("TAB", None).unwrap();

    let name: String = env.eval("return UnitName('target')").unwrap();
    assert_eq!(name, "Hogger", "Tab should target nearest enemy");
}

#[test]
fn test_escape_clears_target() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(r#"
        GameMenuFrame = CreateFrame("Frame", "GameMenuFrame", UIParent)
        GameMenuFrame:Hide()
    "#).unwrap();

    // Set a target first
    env.send_key_press("F1", None).unwrap();
    let exists: bool = env.eval("return UnitExists('target')").unwrap();
    assert!(exists, "Target should exist after F1");

    // Escape should clear target, not open game menu
    env.send_key_press("ESCAPE", None).unwrap();

    let exists: bool = env.eval("return UnitExists('target')").unwrap();
    assert!(!exists, "Escape should clear target");

    let menu_shown: bool = env.eval("return GameMenuFrame:IsShown()").unwrap();
    assert!(!menu_shown, "GameMenuFrame should NOT open when clearing target");
}

#[test]
fn test_escape_no_target_opens_game_menu() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(r#"
        GameMenuFrame = CreateFrame("Frame", "GameMenuFrame", UIParent)
        GameMenuFrame:Hide()
    "#).unwrap();

    // No target set, escape should toggle game menu
    env.send_key_press("ESCAPE", None).unwrap();

    let menu_shown: bool = env.eval("return GameMenuFrame:IsShown()").unwrap();
    assert!(menu_shown, "Escape without target should toggle game menu");
}

#[test]
fn test_target_fires_player_target_changed() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(r#"
        _G.target_changed_count = 0
        local f = CreateFrame("Frame", "TargetEventFrame", UIParent)
        f:RegisterEvent("PLAYER_TARGET_CHANGED")
        f:SetScript("OnEvent", function(self, event)
            if event == "PLAYER_TARGET_CHANGED" then
                _G.target_changed_count = _G.target_changed_count + 1
            end
        end)
    "#).unwrap();

    env.send_key_press("F1", None).unwrap();
    let count: i32 = env.eval("return _G.target_changed_count").unwrap();
    assert_eq!(count, 1, "Setting target should fire PLAYER_TARGET_CHANGED");

    env.send_key_press("ESCAPE", None).unwrap();
    let count: i32 = env.eval("return _G.target_changed_count").unwrap();
    assert_eq!(count, 2, "Clearing target should fire PLAYER_TARGET_CHANGED");
}

#[test]
fn test_unit_exists_target_follows_state() {
    let env = WowLuaEnv::new().unwrap();

    let exists: bool = env.eval("return UnitExists('target')").unwrap();
    assert!(!exists, "UnitExists('target') should be false with no target");

    env.exec("TargetUnit('player')").unwrap();
    let exists: bool = env.eval("return UnitExists('target')").unwrap();
    assert!(exists, "UnitExists('target') should be true after targeting");

    env.exec("ClearTarget()").unwrap();
    let exists: bool = env.eval("return UnitExists('target')").unwrap();
    assert!(!exists, "UnitExists('target') should be false after clearing");
}
