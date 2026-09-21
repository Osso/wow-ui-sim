//! Preserve native Edit Mode geometry initialization before dependent systems update.
#![cfg(feature = "client-wowforever")]

#[test]
fn editmode_initial_anchors_precede_queue_status_consumer() {
    let ui = wow_ui_sim::paths::default_blizzard_ui_addons_path().unwrap();
    let (env, _) = crate::common::blizzard_addon_harness::build_blizzard_addon_closure_env(
        &ui,
        &["Blizzard_QueueStatusFrame", "Blizzard_MicroMenu"],
        &[],
    );
    env.set_screen_size(1600.0, 1200.0);
    env.exec(include_str!(
        "../src/lua_api/workarounds/editmode/sync_set_point_overrides.lua"
    ))
    .unwrap();
    env.exec(
        r#"
        local manager = EditModeManagerFrame
        local micro, queue = MicroMenuContainer, QueueStatusButton
        assert(micro and queue and queue.UpdateDefaultAnchor)
        manager.registeredSystemFrames = {micro, queue}
        manager.layoutInfo = {layouts = {}, activeLayout = 1}
        local layouts = {
            [micro.system] = {
                system = micro.system, isInDefaultPosition = false, settings = {},
                anchorInfo = {point = "CENTER", relativeTo = "UIParent",
                    relativePoint = "CENTER", offsetX = -200, offsetY = -250},
            },
            [queue.system] = {
                system = queue.system, isInDefaultPosition = false, settings = {},
                anchorInfo = {point = "CENTER", relativeTo = "UIParent",
                    relativePoint = "CENTER", offsetX = 120, offsetY = -90},
            },
        }
        function manager:GetActiveLayoutSystemInfo(system) return layouts[system] end
        queue.systemInfo = nil
        queue:ClearAllPoints()
        queue:SetSize(45, 45)
        queue:SetScale(1)
        assert(queue:GetNumPoints() == 0)
        assert(queue:GetCenter() == nil)

        QueueQuadrants, QueueQuadrantErrors = {}, {}
        local getQuadrant = FrameUtil.GetScreenQuadrant
        FrameUtil.GetScreenQuadrant = function(frame)
            if frame ~= queue then return getQuadrant(frame) end
            local ok, quadrant = pcall(getQuadrant, frame)
            if not ok then
                QueueQuadrantErrors[#QueueQuadrantErrors + 1] = quadrant
                error(quadrant)
            end
            local x, y = frame:GetCenter()
            QueueQuadrants[#QueueQuadrants + 1] = {quadrant, x, y}
            return quadrant
        end
        "#,
    )
    .unwrap();
    env.exec(include_str!(
        "../src/lua_api/workarounds/editmode/apply_system_anchors.lua"
    ))
    .unwrap();
    env.exec(
        r#"
        assert(#QueueQuadrantErrors == 0, table.concat(QueueQuadrantErrors, "\n"))
        assert(#QueueQuadrants > 0, "native QueueStatus consumer was not exercised")
        local first = QueueQuadrants[1]
        assert(first[1] == FrameUtilQuadrantEnum.TopLeft)
        assert(first[2] == 22.5 and first[3] == 1177.5,
            "earlier MicroMenu update must see the native initialized queue rect")
        local point, relative, relativePoint, x, y = QueueStatusButton:GetPoint(1)
        assert(point == "CENTER" and relative == UIParent and relativePoint == "CENTER")
        assert(x == 120 and y == -90, "saved queue anchor must survive initialization")
        local centerX, centerY = QueueStatusButton:GetCenter()
        assert(centerX == 920 and centerY == 510)
        "#,
    )
    .unwrap();
}
