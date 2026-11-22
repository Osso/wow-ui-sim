#![cfg(any(feature = "client-retail", feature = "client-wowforever"))]

use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::startup::{fire_one_on_update_tick, fire_startup_events, process_pending_timers};

const IGNORE_LIST_SETUP: &str = r#"
    A_Admin.SetSocialIgnoreName(1, "R88eh-IgnoreAlpha")
    A_Admin.SetSocialIgnoreName(2, "R88eh-IgnoreBeta")
    A_Admin.SetSocialBlockedEntry(1, 92001, "R88eh-BlockedAlpha")
    assert(C_FriendList.GetNumIgnores() == 2, "both supplied ignores must be counted")
    assert(C_FriendList.GetIgnoreName(1) == "R88eh-IgnoreAlpha", "first supplied ignore")
    assert(C_FriendList.GetIgnoreName(2) == "R88eh-IgnoreBeta", "second supplied ignore")
    assert(C_SocialRestrictions.IsFriendsDisabled() == false,
        "Forever should expose the modeled default friends-disabled state")

    FriendsFrame:Show()
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
    assert(provider:GetSize() == 5, "provider must contain two headers and three rows")
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

    local selectedType, selectedIndex = IgnoreList_GetSelected()
    assert(selectedType == SQUELCH_TYPE_IGNORE and selectedIndex == 1,
        "original update must select the first ignore row")
    assert(firstIgnore:IsHighlightLocked(), "selected ignore row must be highlighted")

    FriendsFrame_SelectSquelched(SQUELCH_TYPE_BLOCK_INVITE, 1)
    selectedType, selectedIndex = IgnoreList_GetSelected()
    assert(selectedType == SQUELCH_TYPE_BLOCK_INVITE and selectedIndex == 1,
        "original selector must select the blocked row")
    assert(blocked:IsHighlightLocked(), "selected blocked row must be highlighted")
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
