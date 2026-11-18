//! Layout lock for the Achievements panel.

use crate::common;

const ACHIEVEMENT_LAYOUT_ASSERTIONS_LUA: &str = r#"
    local EPS = 0.75

    local function approx(actual, expected, eps)
        if type(actual) ~= "number" or type(expected) ~= "number" then
            return false
        end
        return math.abs(actual - expected) <= (eps or EPS)
    end

    local function rect(frame, name)
        if type(frame) ~= "table" then
            return nil, name .. "_missing"
        end
        local l, b, w, h = frame:GetRect()
        if not (l and b and w and h) then
            return nil, name .. "_missing_rect"
        end
        return { l = l, b = b, w = w, h = h, r = l + w, t = b + h }, nil
    end

    local function has_point(frame, point, rel, rel_point, x, y, eps)
        for i = 1, frame:GetNumPoints() do
            local p, r, rp, ox, oy = frame:GetPoint(i)
            local rel_matches = (r == rel) or (r == nil and rel ~= nil and frame.GetParent and frame:GetParent() == rel)
            if p == point and rel_matches and rp == rel_point and approx(ox or 0, x, eps) and approx(oy or 0, y, eps) then
                return true
            end
        end
        return false
    end

    local function expect_rect(frame, name, left, bottom, width, height)
        local r, e = rect(frame, name)
        if not r then
            return nil, e
        end
        if not approx(r.l, left) then
            return nil, name .. "_left=" .. tostring(r.l)
        end
        if not approx(r.b, bottom) then
            return nil, name .. "_bottom=" .. tostring(r.b)
        end
        if not approx(r.w, width, 0.1) then
            return nil, name .. "_width=" .. tostring(r.w)
        end
        if not approx(r.h, height, 0.1) then
            return nil, name .. "_height=" .. tostring(r.h)
        end
        return r, nil
    end

    if not ToggleAchievementFrame then
        return "missing_toggle_achievement_frame"
    end

    ToggleAchievementFrame()
    if not AchievementFrame or not AchievementFrame:IsShown() then
        return "achievement_not_shown"
    end

    local frame_rect, frame_err = expect_rect(AchievementFrame, "achievement_frame", 96, 152, 768, 500)
    if not frame_rect then return frame_err end
    if not has_point(AchievementFrame, "TOPLEFT", UIParent, "TOPLEFT", 96, -116, 0.1) then
        return "achievement_frame_anchor_mismatch"
    end

    local close_rect, close_err = expect_rect(AchievementFrameCloseButton, "achievement_close_button", 840, 628, 24, 24)
    if not close_rect then return close_err end
    if not has_point(AchievementFrameCloseButton, "TOPRIGHT", AchievementFrame, "TOPRIGHT", 0, 0, 0.1) then
        return "achievement_close_button_anchor_mismatch"
    end

    if not AchievementFrameCategories or not AchievementFrameCategories:IsShown() then
        return "achievement_categories_missing_or_hidden"
    end
    local categories_rect, categories_err = expect_rect(
        AchievementFrameCategories,
        "achievement_categories",
        117,
        172,
        175,
        461
    )
    if not categories_rect then return categories_err end
    if not has_point(AchievementFrameCategories, "TOPLEFT", AchievementFrame, "TOPLEFT", 21, -19, 0.1) then
        return "achievement_categories_top_anchor_mismatch"
    end
    if not has_point(AchievementFrameCategories, "BOTTOMLEFT", AchievementFrame, "BOTTOMLEFT", 21, 20, 0.1) then
        return "achievement_categories_bottom_anchor_mismatch"
    end

    local categories_bg_rect, categories_bg_err = expect_rect(
        AchievementFrameCategoriesBG,
        "achievement_categories_bg",
        121,
        175,
        195,
        454
    )
    if not categories_bg_rect then return categories_bg_err end
    if not has_point(AchievementFrameCategoriesBG, "TOPLEFT", AchievementFrame, "TOPLEFT", 25, -23, 0.1) then
        return "achievement_categories_bg_top_anchor_mismatch"
    end
    if not has_point(AchievementFrameCategoriesBG, "BOTTOMLEFT", AchievementFrame, "BOTTOMLEFT", 25, 23, 0.1) then
        return "achievement_categories_bg_bottom_anchor_mismatch"
    end

    local background = AchievementFrame.Background
    local bg_rect, bg_err = expect_rect(background, "achievement_background", 112, 168, 736, 468)
    if not bg_rect then return bg_err end
    if not has_point(background, "TOPLEFT", AchievementFrame, "TOPLEFT", 16, -16, 0.1) then
        return "achievement_background_top_anchor_mismatch"
    end
    if not has_point(background, "BOTTOMRIGHT", AchievementFrame, "BOTTOMRIGHT", -16, 16, 0.1) then
        return "achievement_background_bottom_anchor_mismatch"
    end

    local header_rect, header_err = expect_rect(AchievementFrame.Header, "achievement_header", 122, 614, 726, 106)
    if not header_rect then return header_err end
    if not has_point(AchievementFrame.Header, "BOTTOMLEFT", AchievementFrame, "TOPLEFT", 26, -38, 0.1) then
        return "achievement_header_anchor_mismatch"
    end

    local scrollbox_rect, scrollbox_err = expect_rect(
        AchievementFrameCategories.ScrollBox,
        "achievement_categories_scrollbox",
        117,
        177,
        175,
        451
    )
    if not scrollbox_rect then return scrollbox_err end
    if not AchievementFrameCategories.ScrollBox:IsShown() then
        return "achievement_categories_scrollbox_hidden"
    end

    -- XML declares Summary at 461px, but its BOTTOM anchor to Categories resolves the runtime height to 425px.
    local summary_rect, summary_err = expect_rect(AchievementFrameSummary, "achievement_summary", 314, 172, 530, 425)
    if not summary_rect then return summary_err end
    if not AchievementFrameSummary:IsShown() then
        return "achievement_summary_hidden"
    end
    if AchievementFrameAchievements and AchievementFrameAchievements:IsShown() then
        return "achievement_list_should_start_hidden"
    end
    if AchievementFrameStats and AchievementFrameStats:IsShown() then
        return "achievement_stats_should_start_hidden"
    end
    if AchievementFrameComparison and AchievementFrameComparison:IsShown() then
        return "achievement_comparison_should_start_hidden"
    end

    local status_rect, status_err = expect_rect(
        AchievementFrameSummaryCategoriesStatusBar,
        "achievement_summary_categories_status_bar",
        335,
        380,
        488,
        21
    )
    if not status_rect then return status_err end
    if not AchievementFrameSummaryCategoriesStatusBar:IsShown() then
        return "achievement_summary_categories_status_bar_hidden"
    end

    if PanelTemplates_GetSelectedTab and PanelTemplates_GetSelectedTab(AchievementFrame) ~= 1 then
        return "achievement_selected_tab=" .. tostring(PanelTemplates_GetSelectedTab(AchievementFrame))
    end

    return "ok"
"#;

#[test]
#[cfg(feature = "gui")]
fn achievement_frame_layout_stays_locked() {
    if crate::common::wrath_achievement_fixtures::assert_native(
        r#"
        A_Admin.SetAchievementEarned(6, true)
        ToggleAchievementFrame()
        assert(AchievementFrame:IsShown())
        assert(AchievementFrame:GetWidth() == 768 and AchievementFrame:GetHeight() == 500)
        assert(AchievementFrameHeader:GetWidth() == 726 and AchievementFrameHeader:GetHeight() == 106)
        assert(AchievementFrameSummary:GetWidth() == 530)
        assert(AchievementFrameCategoriesContainer:GetObjectType() == "ScrollFrame")
        assert(AchievementFrameCategoriesContainer.buttons[1]:IsVisible())
        local point, relative, relativePoint, x, y = AchievementFrameHeader:GetPoint(1)
        assert(point == "BOTTOMLEFT" and relative == AchievementFrame and relativePoint == "TOPLEFT")
        assert(x == 26 and y == -39)
        assert(AchievementFrameSummary:IsVisible())
        assert(not AchievementFrameAchievements:IsShown())
        assert(not AchievementFrameStats:IsShown() and not AchievementFrameComparison:IsShown())
        assert(AchievementFrameBackground:GetWidth() == 736 and AchievementFrameBackground:GetHeight() == 468)
        assert(AchievementFrame.selectedTab == 1)
        "#,
    ) {
        return;
    }
    test_timeout! {
        let env = common::panel_fixtures::setup_env();
        let result: String = env.eval(ACHIEVEMENT_LAYOUT_ASSERTIONS_LUA).unwrap();
        assert_eq!(
            result, "ok",
            "AchievementFrame layout should remain locked after ToggleAchievementFrame(): {result}"
        );
    }
}

#[test]
#[cfg(feature = "gui")]
fn achievement_summary_empty_text_does_not_overlap_summary_entries() {
    if crate::common::wrath_achievement_fixtures::assert_native(
        r#"
        A_Admin.SetAchievementEarned(6, true)
        ToggleAchievementFrame()
        local empty = assert(AchievementFrameSummaryAchievementsEmptyText)
        local first = assert(AchievementFrameSummaryAchievement1)
        if empty:IsVisible() and first:IsVisible() then
            local overlapX = empty:GetLeft() < first:GetRight() and empty:GetRight() > first:GetLeft()
            local overlapY = empty:GetBottom() < first:GetTop() and empty:GetTop() > first:GetBottom()
            assert(not (overlapX and overlapY), "native empty text overlaps first summary entry")
        end
        AchievementFrameSummary_UpdateAchievements(6)
        assert(not empty:IsShown() and first:IsShown())
        "#,
    ) {
        return;
    }
    test_timeout! {
        let env = common::panel_fixtures::setup_env();
        let result: String = env
            .eval(
                r#"
                if not ToggleAchievementFrame then
                    return "missing_toggle_achievement_frame"
                end

                ToggleAchievementFrame()
                if not AchievementFrameSummary or not AchievementFrameSummary:IsShown() then
                    return "achievement_summary_hidden"
                end

                local emptyText = AchievementFrameSummaryAchievementsEmptyText
                if not emptyText then
                    return "summary_empty_text_missing"
                end

                -- VERIFIED: Retail 12.1.5.70783 Blizzard_AchievementUI.lua:2379-2506
                -- shows default incomplete rows and EmptyText together; test geometry.
                for i = 1, ACHIEVEMENTUI_MAX_SUMMARY_ACHIEVEMENTS do
                    local entry = _G["AchievementFrameSummaryAchievement" .. i]
                    if emptyText:IsShown() and entry and entry:IsShown() then
                        local overlapX = emptyText:GetLeft() < entry:GetRight()
                            and emptyText:GetRight() > entry:GetLeft()
                        local overlapY = emptyText:GetBottom() < entry:GetTop()
                            and emptyText:GetTop() > entry:GetBottom()
                        if overlapX and overlapY then
                            return "summary_empty_text_overlap_" .. i
                        end
                    end
                end

                return "ok"
                "#,
            )
            .unwrap();
        assert_eq!(
            result, "ok",
            "achievement summary empty text should not overlap summary rows: {result}"
        );
    }
}

#[test]
#[cfg(feature = "gui")]
fn achievement_frame_toggle_hides_visible_panel_tree() {
    if crate::common::wrath_achievement_fixtures::assert_native(
        r#"
        A_Admin.SetAchievementEarned(6, true)
        ToggleAchievementFrame()
        assert(AchievementFrame:IsShown() and AchievementFrameSummary:IsVisible())
        ToggleAchievementFrame()
        assert(not AchievementFrame:IsShown())
        local function assert_hidden_tree(frame)
            for _, child in ipairs({frame:GetChildren()}) do
                assert(not child:IsVisible(), child:GetName() or "anonymous visible child")
                assert_hidden_tree(child)
            end
        end
        assert_hidden_tree(AchievementFrame)
        "#,
    ) {
        return;
    }
    test_timeout! {
        let env = common::panel_fixtures::setup_env();
        let result: String = env
            .eval(
                r#"
                if not ToggleAchievementFrame then
                    return "missing_toggle_achievement_frame"
                end

                ToggleAchievementFrame()
                if not AchievementFrame or not AchievementFrame:IsShown() then
                    return "achievement_not_shown"
                end

                local hideCalls = 0
                local originalHideUIPanel = HideUIPanel
                HideUIPanel = function(frame, ...)
                    if frame == AchievementFrame then
                        hideCalls = hideCalls + 1
                    end
                    return originalHideUIPanel(frame, ...)
                end

                ToggleAchievementFrame()
                HideUIPanel = originalHideUIPanel
                if AchievementFrame:IsShown() then
                    return "achievement_still_shown"
                end
                if hideCalls ~= 1 then
                    return "hide_uipanel_calls=" .. tostring(hideCalls)
                end

                local leaked = {}
                local function visit(frame, path)
                    if (type(frame) ~= "table" and type(frame) ~= "userdata") or type(frame.GetChildren) ~= "function" then
                        return
                    end
                    for index, child in ipairs({ frame:GetChildren() }) do
                        local childPath = path .. "." .. tostring(child:GetName() or index)
                        if type(child.IsVisible) == "function" and child:IsVisible() then
                            table.insert(leaked, childPath)
                        end
                        visit(child, childPath)
                    end
                end

                visit(AchievementFrame, "AchievementFrame")
                if #leaked > 0 then
                    return "visible_child_after_hide=" .. table.concat(leaked, ",")
                end

                return "ok"
                "#,
            )
            .unwrap();
        assert_eq!(
            result, "ok",
            "AchievementFrame toggle should hide the full visible panel tree: {result}"
        );
    }
}
