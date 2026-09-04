use crate::common;

use wow_ui_sim::lua_api::WowLuaEnv;

prefork_full_ui_case! {
fn compact_raid_manager_stays_hidden_and_stops_visible_onupdate_when_solo(env: &WowLuaEnv) {
        env.exec("A_Admin.SetPartySize(0)").unwrap();

        let (button_name, manager_shown, manager_visible, button_visible, in_group): (
            String,
            bool,
            bool,
            bool,
            bool,
        ) = env
            .eval(
                r#"
                local manager = CompactRaidFrameManager
                if not manager then error("missing_manager") end

                local bottom = manager.BottomButtons
                if not bottom then error("missing_bottom_buttons") end

                local button
                for _, child in ipairs({ bottom:GetChildren() }) do
                    local name = child and child.GetName and child:GetName()
                    if name and name:find("LeaveInstanceGroupButton", 1, true) then
                        button = child
                        break
                    end
                end

                if not button then
                    error("missing_leave_instance_group_button")
                end

                return button:GetName() or "",
                    manager:IsShown(),
                    manager:IsVisible(),
                    button:IsVisible(),
                    IsInGroup()
                "#,
            )
            .unwrap();

        assert!(
            !in_group,
            "A_Admin.SetPartySize(0) should make the simulated player solo"
        );
        assert!(
            !manager_shown,
            "CompactRaidFrameManager should hide after switching the simulated player to solo"
        );
        assert!(
            !manager_visible,
            "CompactRaidFrameManager should not stay visible after switching the simulated player to solo"
        );
        assert!(
            !button_visible,
            "leave-instance button should not stay visible after switching the simulated player to solo"
        );

        env.fire_on_update(0.016).unwrap();

        let state = env.state();
        let state = state.borrow();
        let button_id = state
            .widgets
            .get_id_by_name(&button_name)
            .expect("leave-instance button should have a runtime name");

        assert!(
            state.on_update_frames.contains(&button_id),
            "leave-instance button should still register an OnUpdate handler so this test checks the visibility gate"
        );

        let visible_ids = state.visible_on_update_cache.clone().unwrap_or_default();
        assert!(
            !visible_ids.contains(&button_id),
            "leave-instance button should not stay in visible OnUpdate cache when solo"
        );
}
}

prefork_full_ui_case! {
fn compact_raid_manager_layout_stays_locked_when_collapsed_and_expanded(env: &WowLuaEnv) {
        env.exec(
            r#"
            A_Admin.SetPartySize(4)
            CompactRaidFrameManager_UpdateShown()
            CompactRaidFrameManager_UpdateOptionsFlowContainer()
            CompactRaidFrameManager_UpdateContainerVisibility()
            CompactRaidFrameManager_Collapse()
            "#,
        )
        .unwrap();

        let result: String = env
            .eval(
                r#"
                local EPS = 0.75

                local function approx(actual, expected, eps)
                    if type(actual) ~= "number" or type(expected) ~= "number" then
                        return false
                    end
                    return math.abs(actual - expected) <= (eps or EPS)
                end

                local function rect(path, frame)
                    if type(frame) ~= "table" then
                        return nil, path .. "_missing"
                    end
                    local l, b, w, h = frame:GetRect()
                    if not (l and b and w and h) then
                        return nil, path .. "_missing_rect"
                    end
                    return { l = l, b = b, w = w, h = h, r = l + w, t = b + h }, nil
                end

                local manager = CompactRaidFrameManager
                if not manager then
                    return "manager_missing"
                end

                local display = manager.displayFrame
                local forward = manager.toggleButtonForward
                local back = manager.toggleButtonBack
                local bottom = manager.BottomButtons
                local container = CompactRaidFrameContainer
                if not display then return "display_missing" end
                if not forward then return "forward_toggle_missing" end
                if not back then return "back_toggle_missing" end
                if not bottom then return "bottom_buttons_missing" end
                if not container then return "container_missing" end

                -- Collapsed state lock.
                CompactRaidFrameManager_UpdateShown()
                CompactRaidFrameManager_UpdateOptionsFlowContainer()
                CompactRaidFrameManager_Collapse()

                if not manager:IsShown() then
                    return "manager_hidden_collapsed"
                end
                if not manager.collapsed then
                    return "manager_not_collapsed"
                end

                local managerCollapsed, managerCollapsedErr = rect("manager_collapsed", manager)
                if not managerCollapsed then return managerCollapsedErr end
                if not approx(managerCollapsed.l, -200) then
                    return "collapsed_left=" .. tostring(managerCollapsed.l)
                end
                if not approx(managerCollapsed.t, 628) then
                    return "collapsed_top=" .. tostring(managerCollapsed.t)
                end
                if not approx(managerCollapsed.w, 222, 0.1) then
                    return "collapsed_width=" .. tostring(managerCollapsed.w)
                end

                if display:IsShown() then
                    return "display_should_be_hidden_when_collapsed"
                end
                if back:IsShown() then
                    return "back_toggle_should_be_hidden_when_collapsed"
                end
                if not forward:IsShown() then
                    return "forward_toggle_should_be_visible_when_collapsed"
                end
                if bottom:IsShown() then
                    return "bottom_buttons_should_be_hidden_when_collapsed"
                end

                local forwardRect, forwardErr = rect("forward_toggle", forward)
                if not forwardRect then return forwardErr end
                if not approx(forwardRect.w, 16, 0.1) or not approx(forwardRect.h, 35, 0.1) then
                    return "forward_toggle_size=" .. tostring(forwardRect.w) .. "x" .. tostring(forwardRect.h)
                end
                if not approx(forwardRect.r, managerCollapsed.r - 7) then
                    return "forward_toggle_right=" .. tostring(forwardRect.r)
                end
                local managerCollapsedCenterY = managerCollapsed.b + (managerCollapsed.h / 2)
                local forwardCenterY = forwardRect.b + (forwardRect.h / 2)
                if not approx(forwardCenterY, managerCollapsedCenterY) then
                    return "forward_toggle_center_y=" .. tostring(forwardCenterY)
                end

                -- Expanded state lock.
                CompactRaidFrameManager_Expand()
                CompactRaidFrameManager_UpdateShown()
                CompactRaidFrameManager_UpdateOptionsFlowContainer()

                if not manager:IsShown() then
                    return "manager_hidden_expanded"
                end
                if manager.collapsed then
                    return "manager_still_collapsed_after_expand"
                end

                local managerExpanded, managerExpandedErr = rect("manager_expanded", manager)
                if not managerExpanded then return managerExpandedErr end
                if not approx(managerExpanded.l, 0) then
                    return "expanded_left=" .. tostring(managerExpanded.l)
                end
                if not approx(managerExpanded.t, 628) then
                    return "expanded_top=" .. tostring(managerExpanded.t)
                end
                if not approx(managerExpanded.w, 222, 0.1) then
                    return "expanded_width=" .. tostring(managerExpanded.w)
                end

                if not display:IsShown() then
                    return "display_hidden_when_expanded"
                end
                if not back:IsShown() then
                    return "back_toggle_hidden_when_expanded"
                end
                if forward:IsShown() then
                    return "forward_toggle_visible_when_expanded"
                end
                if not bottom:IsShown() then
                    return "bottom_buttons_hidden_when_expanded"
                end
                local backRect, backErr = rect("back_toggle", back)
                if not backRect then return backErr end
                if not approx(backRect.w, 16, 0.1) or not approx(backRect.h, 35, 0.1) then
                    return "back_toggle_size=" .. tostring(backRect.w) .. "x" .. tostring(backRect.h)
                end
                if not approx(backRect.r, managerExpanded.r - 7) then
                    return "back_toggle_right=" .. tostring(backRect.r)
                end
                local managerExpandedCenterY = managerExpanded.b + (managerExpanded.h / 2)
                local backCenterY = backRect.b + (backRect.h / 2)
                if not approx(backCenterY, managerExpandedCenterY - 20) then
                    return "back_toggle_center_y=" .. tostring(backCenterY)
                end

                local label = display.label
                if not label then
                    return "label_missing"
                end
                local labelText = label:GetText() or ""
                if labelText == "" then
                    return "label_text_empty"
                end

                return "ok"
            "#,
            )
            .unwrap();

        assert_eq!(
            result, "ok",
            "CompactRaidFrameManager collapsed/expanded layout should remain locked: {result}"
        );
}
}
