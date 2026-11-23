#![cfg(any(feature = "client-retail", feature = "client-wowforever"))]

use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::startup::{fire_one_on_update_tick, fire_startup_events, process_pending_timers};

const IGNORE_LIST_SETUP: &str = r#"
    A_Admin.SetSocialIgnoreName(1, "R88eh-IgnoreAlpha")
    A_Admin.SetSocialIgnoreName(2, "R88eh-IgnoreBeta")
    A_Admin.SetSocialBlockedEntry(1, 92001, "R88eh-BlockedAlpha")
    A_Admin.SetSocialBlockedEntry(2, 92002, "R88eh-BlockedBeta")
    assert(C_FriendList.GetNumIgnores() == 2, "both supplied ignores must be counted")
    assert(BNGetNumBlocked() == 2, "both supplied block rows must be counted")
    assert(BNGetSelectedBlock() == 0, "no selected block row must return zero")
    local firstBlockID, firstBlockName = BNGetBlockedInfo(1)
    local secondBlockID, secondBlockName = BNGetBlockedInfo(2)
    assert(firstBlockID == 92001 and firstBlockName == "R88eh-BlockedAlpha",
        "first valid block row must return its identifier and name")
    assert(secondBlockID == 92002 and secondBlockName == "R88eh-BlockedBeta",
        "second valid block row must return its identifier and name")
    assert(C_FriendList.GetIgnoreName(1) == "R88eh-IgnoreAlpha", "first supplied ignore")
    assert(C_FriendList.GetIgnoreName(2) == "R88eh-IgnoreBeta", "second supplied ignore")
    assert(C_SocialRestrictions.IsFriendsDisabled() == false,
        "Forever should expose the modeled default friends-disabled state")

    -- The panel manager supplies FriendsFrame's anchors; direct Show() does not.
    ShowUIPanel(FriendsFrame)
    FriendsFrame.IgnoreListWindow:Show()
"#;

const IGNORE_LIST_ASSERTIONS: &str = r#"
    local scrollBox = FriendsFrame.IgnoreListWindow.ScrollBox
    local width, height = scrollBox:GetSize()
    local hasViewport = width > 0 and height > 0
    local windowWidth, windowHeight = FriendsFrame.IgnoreListWindow:GetSize()
    local insetWidth, insetHeight = FriendsFrame.IgnoreListWindow.Inset:GetSize()
    local parentWidth, parentHeight = FriendsFrame:GetSize()
    local point1, relativeTo1, relativePoint1, x1, y1 = scrollBox:GetPoint(1)
    local point2, relativeTo2, relativePoint2, x2, y2 = scrollBox:GetPoint(2)
    assert(hasViewport, string.format(
        "ignore ScrollBox size must be positive: scroll=%sx%s window=%sx%s inset=%sx%s parent=%sx%s anchors=%s/%s(%s,%s,%s),%s/%s(%s,%s,%s)",
        width, height, windowWidth, windowHeight, insetWidth, insetHeight, parentWidth, parentHeight,
        tostring(point1), tostring(relativeTo1 and relativeTo1:GetName()), tostring(relativePoint1), x1, y1,
        tostring(point2), tostring(relativeTo2 and relativeTo2:GetName()), tostring(relativePoint2), x2, y2))
    local provider = scrollBox:GetDataProvider()
    assert(provider:GetSize() == 6, "provider must contain two headers and four rows")
    local visibleFrames = 0
    for _ in scrollBox:EnumerateFrames() do
        visibleFrames = visibleFrames + 1
    end
    assert(scrollBox:IsShown() and scrollBox:IsVisible(), "ignore ScrollBox must be visible")

    local function check(squelchType, index, name)
        local row = assert(scrollBox:FindFrameByPredicate(function(_, elementData)
            return elementData.squelchType == squelchType and elementData.index == index
        end), "original ignore row not materialized: " .. index .. ", visible frames=" .. visibleFrames)
        assert(row.name:GetText() == name, "original row must display supplied name")
        assert(row.type == squelchType, "original row must retain its row type")
        return row
    end

    local firstIgnore = check(SQUELCH_TYPE_IGNORE, 1, "R88eh-IgnoreAlpha")
    check(SQUELCH_TYPE_IGNORE, 2, "R88eh-IgnoreBeta")
    local blocked = check(SQUELCH_TYPE_BLOCK_INVITE, 1, "R88eh-BlockedAlpha")
    local secondBlocked = check(SQUELCH_TYPE_BLOCK_INVITE, 2, "R88eh-BlockedBeta")

    local selectedType, selectedIndex = IgnoreList_GetSelected()
    assert(selectedType == SQUELCH_TYPE_IGNORE and selectedIndex == 1,
        "original update must select the first ignore row")
    assert(firstIgnore:IsHighlightLocked(), "selected ignore row must be highlighted")

    FriendsFrame_SelectSquelched(SQUELCH_TYPE_BLOCK_INVITE, 1)
    selectedType, selectedIndex = IgnoreList_GetSelected()
    assert(selectedType == SQUELCH_TYPE_BLOCK_INVITE and selectedIndex == 1,
        "original selector must select the blocked row")
    assert(blocked:IsHighlightLocked(), "selected blocked row must be highlighted")

    FriendsFrame_SelectSquelched(SQUELCH_TYPE_BLOCK_INVITE, 2)
    assert(BNGetSelectedBlock() == 2, "selected block identifier must resolve to its row")
    assert(secondBlocked:IsHighlightLocked(), "second blocked row must be highlighted")
    A_Admin.SetSocialBlockedEntry(1, 92002, "R88eh-BlockedBeta")
    A_Admin.SetSocialBlockedEntry(2, 92001, "R88eh-BlockedAlpha")
    assert(BNGetSelectedBlock() == 1, "selected block must follow its identifier after reordering")

    C_FriendList.SetSelectedIgnore(2)
    A_Admin.SetSocialIgnoreName(1, "R88eh-IgnoreBeta")
    A_Admin.SetSocialIgnoreName(2, "R88eh-IgnoreAlpha")
    assert(C_FriendList.GetSelectedIgnore() == 1,
        "selected ignore must follow its identifier after reordering")

    for index = 1, 51 do
        A_Admin.SetSocialIgnoreName(index, "R88eh-Ignore" .. index)
    end
    assert(C_FriendList.GetNumIgnores() == 50, "ignore count must stop at fifty entries")
"#;

fn load_friends_frame() -> WowLuaEnv {
    let env = WowLuaEnv::new().expect("friend-list fixture environment");
    env.set_screen_size(1024.0, 768.0);
    crate::common::load_game_startup_inventory(&env);
    env.apply_post_load_workarounds();
    fire_startup_events(&env);
    env.apply_post_event_workarounds();
    process_pending_timers(&env);
    fire_one_on_update_tick(&env);
    env
}

#[test]
fn real_friends_frame_ignore_scrollbox_uses_supplied_names_and_selection() {
    let env = load_friends_frame();
    env.exec(IGNORE_LIST_SETUP)
        .expect("original FriendsFrame ignore window should open");
    env.fire_on_update(0.016)
        .expect("post-open UI tick should resolve the original ScrollBox layout");
    env.exec(IGNORE_LIST_ASSERTIONS)
        .expect("original FriendsFrame ignore ScrollBox should display supplied rows");
}
