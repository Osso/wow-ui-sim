//! SP76-96 ChatFrameUtil source-patch retirement.
//! Valid loaded-owner cases are preservation. Deliberately damaged Lua inputs
//! distinguish the fallback/skip patches; they are not normal-client/native REDs.
#![cfg(all(feature = "optional-tests", feature = "client-retail"))]

use crate::common::blizzard_addon_harness::{
    load_blizzard_addon_closure_into_env, new_blizzard_addon_env,
};
use wow_ui_sim::lua_api::WowLuaEnv;

fn chat_owner() -> WowLuaEnv {
    let cache = wow_ui_sim::paths::default_blizzard_ui_addons_path().unwrap();
    let env = new_blizzard_addon_env(&cache);
    wow_ui_sim::xml::register_intrinsic_templates();
    // The selected Base TOC loads Shared constants/colors/util, then Mainline
    // editbox/chat/floating XML. No replacement original functions or TOC overrides.
    let loaded = load_blizzard_addon_closure_into_env(
        &env,
        &cache,
        &[
            "Blizzard_SharedXMLGame",
            "Blizzard_Fonts_Shared",
            "Blizzard_ChatFrameBase",
        ],
        &[],
    );
    assert!(loaded.iter().any(|n| n == "Blizzard_ChatFrameBase"));
    println!("SP76-96 selected Retail closure: {loaded:?}");
    for error in &env.state().borrow().lua_errors {
        eprintln!("SP76-96 original closure Lua error: {error:?}");
    }
    env.exec(r#"
        local function source(fn, suffix)
            local s = debug.getinfo(fn, 'S').source:gsub('\\', '/')
            assert(s:find(suffix, 1, true), s)
        end
        source(ChatFrameUtil.DisplaySystemMessage, 'Blizzard_ChatFrameBase/Shared/ChatFrameUtil.lua')
        source(ChatFrameUtil.SetLastActiveWindow, 'Blizzard_ChatFrameBase/Shared/ChatFrameUtil.lua')
        source(FCFClickAnywhereButton_UpdateState, 'Blizzard_ChatFrameBase/Mainline/FloatingChatFrame.lua')
        for _, frame in ipairs({ChatFrame1, ChatFrame3}) do
            -- The simulator can reuse a precreated MessageFrame here. Native
            -- ScrollingMessageFrame class parity is outside this source-patch test.
            -- Retain original Lua/XML owner and required message-history proof.
            assert(type(frame.AddMessage) == 'function' and type(frame.GetNumMessages) == 'function'
                and type(frame.GetMessageInfo) == 'function', frame:GetName() .. ' missing message history methods')
            assert(frame.editBox and frame.editBox:IsObjectType('EditBox'), frame:GetName() .. ' missing/wrong editBox')
            assert(frame.editBox.chatFrame == frame, frame:GetName() .. ' editBox.chatFrame=' .. tostring(frame.editBox.chatFrame) .. ' expected=' .. tostring(frame))
            assert(frame.clickAnywhereButton and frame.clickAnywhereButton:IsObjectType('Button'), frame:GetName() .. ' missing/wrong clickAnywhereButton')
            assert(frame.clickAnywhereButton:GetParent() == frame, frame:GetName() .. ' clickAnywhereButton parent mismatch')
            source(frame.editBox.Deactivate, 'Blizzard_ChatFrameBase/Shared/ChatFrameEditBox.lua')
        end
    "#).expect("selected original chat owner and XML objects");
    env
}

#[test]
fn original_system_consumer_uses_loaded_metadata_preservation() {
    let env = chat_owner();
    env.exec(r#"
        local info = assert(ChatTypeInfo.SYSTEM)
        assert(info.sticky == 0 and info.flashTab == false, 'original SYSTEM constants missing')
        assert(type(info.r) == 'number' and type(info.g) == 'number' and type(info.b) == 'number')
        assert(info.id == GetChatTypeIndex('SYSTEM'), 'original color initialization/type-index binding')
        local output = CreateFrame('ScrollingMessageFrame', 'StubOSystemOutput', UIParent)
        ChatFrameUtil.DisplaySystemMessage(output, 'SP76 original SYSTEM metadata')
        assert(output:GetNumMessages() == 1)
        local text, r, g, b = output:GetMessageInfo(1)
        assert(text == 'SP76 original SYSTEM metadata')
        assert(r == info.r and g == info.g and b == info.b, 'original message retains loaded SYSTEM color')
    "#).unwrap();
}

#[test]
fn missing_system_metadata_does_not_invent_yellow_retirement_red() {
    let env = chat_owner();
    env.exec(
        r#"
        local output = CreateFrame('ScrollingMessageFrame', 'StubOMissingSystemOutput', UIParent)
        local saved = ChatTypeInfo
        -- Original ChatFrameSetup/ImportAllListsToHash moves values into a proxy.
        -- Replace the Lua table explicitly; clearing its raw SYSTEM slot is not absence.
        ChatTypeInfo = {} -- explicit invalid-input probe, not a native default
        local ok = pcall(ChatFrameUtil.DisplaySystemMessage, output, 'must not fabricate yellow')
        ChatTypeInfo = saved
        assert(not ok, 'SP76 yellow fallback concealed missing original metadata')
        assert(output:GetNumMessages() == 0, 'fallback emitted a fabricated SYSTEM message')
    "#,
    )
    .unwrap();
}

#[test]
fn original_active_editbox_and_click_anywhere_lifecycle_preservation() {
    let env = chat_owner();
    env.exec(r#"
        SetCVar('chatStyle', 'im')
        SetCVar('wholeChatWindowClickable', '1')
        local first, second = ChatFrame1.editBox, ChatFrame3.editBox
        ChatFrame1:Show()
        ChatFrame3:Show()
        ChatFrameUtil.SetLastActiveWindow(first)
        first:Show()
        ChatFrameUtil.SetLastActiveWindow(second)
        assert(LAST_ACTIVE_CHAT_EDIT_BOX == second)
        assert(not first:IsShown(), 'previous original editbox not hidden')
        assert(second:IsShown(), 'new original editbox not shown')
        assert(ChatFrame1.clickAnywhereButton:IsShown(), 'previous chat click target not enabled')
        assert(not ChatFrame3.clickAnywhereButton:IsShown(), 'active chat click target not disabled')
        -- Exercise the selected XML consumer, not just a direct utility call.
        ChatFrame1.clickAnywhereButton:Click()
        assert(LAST_ACTIVE_CHAT_EDIT_BOX == first)
        assert(not ChatFrame1.clickAnywhereButton:IsShown())
        assert(ChatFrame3.clickAnywhereButton:IsShown())
    "#).expect("original SetLastActiveWindow and XML click consumer");
}

#[test]
fn invalid_previous_hide_is_not_silently_skipped_retirement_red() {
    let env = chat_owner();
    env.exec(
        r#"
        SetCVar('chatStyle', 'im')
        local previous = ChatFrame1.editBox
        LAST_ACTIVE_CHAT_EDIT_BOX = previous
        local saved = rawget(previous, 'Hide')
        previous.Hide = false -- invalid receiver method, never an alternative implementation
        local ok = pcall(ChatFrameUtil.SetLastActiveWindow, nil)
        previous.Hide = saved
        assert(not ok, 'SP Hide type guard silently skipped malformed previous editbox')
    "#,
    )
    .unwrap();
}

#[test]
fn missing_previous_click_button_is_not_silently_skipped_retirement_red() {
    let env = chat_owner();
    env.exec(
        r#"
        SetCVar('chatStyle', 'classic')
        local previous = ChatFrame1.editBox
        LAST_ACTIVE_CHAT_EDIT_BOX = previous
        local saved = previous.chatFrame.clickAnywhereButton
        previous.chatFrame.clickAnywhereButton = nil
        local ok = pcall(ChatFrameUtil.SetLastActiveWindow, nil)
        previous.chatFrame.clickAnywhereButton = saved
        assert(not ok, 'SP previous clickAnywhereButton guard silently skipped missing XML child')
    "#,
    )
    .unwrap();
}

#[test]
fn missing_new_click_button_is_not_silently_skipped_retirement_red() {
    let env = chat_owner();
    env.exec(
        r#"
        SetCVar('chatStyle', 'im')
        SetCVar('wholeChatWindowClickable', '1')
        local nextBox = ChatFrame3.editBox
        LAST_ACTIVE_CHAT_EDIT_BOX = nil
        ACTIVE_CHAT_EDIT_BOX = nextBox -- isolate button update from the show/deactivate branch
        local saved = nextBox.chatFrame.clickAnywhereButton
        nextBox.chatFrame.clickAnywhereButton = nil
        local ok = pcall(ChatFrameUtil.SetLastActiveWindow, nextBox)
        nextBox.chatFrame.clickAnywhereButton = saved
        assert(not ok, 'SP new clickAnywhereButton guard silently skipped missing XML child')
    "#,
    )
    .unwrap();
}
