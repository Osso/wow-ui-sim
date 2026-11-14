use wow_ui_sim::lua_api::WowLuaEnv;

pub(crate) fn world_map_provider_sets_and_refreshes_user_waypoint(env: &WowLuaEnv) {
    env.exec(
        r#"
        local loaded, reason = C_AddOns.LoadAddOn("Blizzard_SharedMapDataProviders")
        assert(loaded, "could not load Blizzard_SharedMapDataProviders: " .. tostring(reason))
        assert(type(WaypointLocationDataProviderMixin) == "table")
        assert(type(WaypointLocationDataProviderMixin.RefreshAllData) == "function")
        assert(type(WaypointLocationDataProviderMixin.HandleClick) == "function")

        local pin
        local cursorX, cursorY = 0.25, 0.75
        local map = {
            GetMapID = function() return 2248 end,
            RemoveAllPinsByTemplate = function(_, template)
                assert(template == "WaypointLocationPinTemplate")
                pin = nil
            end,
            AcquirePin = function(_, template)
                assert(template == "WaypointLocationPinTemplate")
                pin = {
                    Icon = {
                        SetAtlas = function(_, atlas)
                            pin.atlas = atlas
                        end,
                    },
                    SetPosition = function(_, x, y)
                        pin.x, pin.y = x, y
                    end,
                    UseFrameLevelType = function() end,
                    IsMouseOver = function() return false end,
                }
                return pin
            end,
            ScrollContainer = {
                GetCursorPosition = function() return cursorX, cursorY end,
                NormalizeUIPosition = function(_, x, y) return x, y end,
            },
        }
        local provider = CreateFromMixins(WaypointLocationDataProviderMixin)
        provider.GetMap = function() return map end
        provider.toggleActive = true
        _G.__c_map_user_waypoint_provider = provider
        _G.__assert_user_waypoint = function(point, mapID, expectedX, expectedY, expectedZ)
            assert(point and point.uiMapID == mapID)
            local x, y = point.position:GetXY()
            assert(math.abs(x - expectedX) < 0.000001)
            assert(math.abs(y - expectedY) < 0.000001)
            assert(point.z == expectedZ)
        end

        provider:RefreshAllData()
        assert(provider.pin == nil, "a map without a user waypoint must not get a pin")
        "#,
    )
    .expect("the original waypoint provider must refresh an empty map without error");

    {
        let mut state = env.state().borrow_mut();
        state.super_track.quest_id = Some(77);
        state.super_track.quest_tracking = true;
        state.super_track.content = Some((0, 101));
        state.super_track.content_saved = (0, 101);
        state.super_track.content_tracking = true;
        state.super_track.map_pin = Some((0, 201));
        state.super_track.map_pin_saved = (0, 201);
        state.super_track.map_pin_tracking = true;
        state.super_track.vignette = Some("Vignette-fixture-1".into());
        state.super_track.corpse = true;
    }

    env.exec("__c_map_user_waypoint_provider:HandleClick()")
        .expect("original HandleClick must pass its C_SuperTrack call at line 101");

    env.exec(
        r#"
        local provider = __c_map_user_waypoint_provider
        local point = C_Map.GetUserWaypoint()
        __assert_user_waypoint(point, 2248, 0.25, 0.75, nil)
        assert(C_Map.HasUserWaypoint() == true)
        assert(C_SuperTrack.IsSuperTrackingUserWaypoint() == false)
        assert(select('#', C_SuperTrack.IsSuperTrackingUserWaypoint()) == 1)
        assert(select('#', C_SuperTrack.SetSuperTrackedUserWaypoint(false)) == 0)

        C_Map.ClearUserWaypoint()
        C_SuperTrack.SetSuperTrackedUserWaypoint(true)
        assert(C_SuperTrack.IsSuperTrackingUserWaypoint() == false,
            "true without a valid waypoint must leave the flag false")
        assert(C_SuperTrack.IsSuperTrackingQuest() == true)
        assert(C_SuperTrack.IsSuperTrackingContent() == true)
        assert(C_SuperTrack.IsSuperTrackingMapPin() == true)

        provider:HandleClick()
        point = C_Map.GetUserWaypoint()
        __assert_user_waypoint(point, 2248, 0.25, 0.75, nil)
        provider:RefreshAllData()
        assert(provider.pin ~= nil, "refresh must acquire the waypoint pin")
        assert(math.abs(provider.pin.x - 0.25) < 0.000001)
        assert(math.abs(provider.pin.y - 0.75) < 0.000001)
        WaypointLocationPinMixin.OnAcquired(provider.pin)
        assert(provider.pin.atlas == "Waypoint-MapPin-Untracked")

        local changes = 0
        local listener = CreateFrame("Frame")
        listener:RegisterEvent("SUPER_TRACKING_CHANGED")
        listener:SetScript("OnEvent", function() changes = changes + 1 end)
        local changesBefore = changes
        assert(select('#', C_SuperTrack.SetSuperTrackedUserWaypoint(0)) == 0)
        assert(changes == changesBefore, "the setter must not fire SUPER_TRACKING_CHANGED")
        assert(C_SuperTrack.IsSuperTrackingUserWaypoint() == true)
        assert(C_SuperTrack.GetSuperTrackedQuestID() == nil)
        assert(select('#', C_SuperTrack.GetSuperTrackedContent()) == 0)
        assert(select('#', C_SuperTrack.GetSuperTrackedMapPin()) == 0)
        assert(C_SuperTrack.GetSuperTrackedVignette() == nil)
        assert(C_SuperTrack.IsSuperTrackingQuest() == false)
        assert(C_SuperTrack.IsSuperTrackingContent() == false)
        assert(C_SuperTrack.IsSuperTrackingMapPin() == false)
        assert(C_SuperTrack.IsSuperTrackingCorpse() == true,
            "the original corpse-tracking flag must survive waypoint selection")
        assert(C_SuperTrack.IsSuperTrackingAnything() == true)
        WaypointLocationPinMixin.OnAcquired(provider.pin)
        assert(provider.pin.atlas == "Waypoint-MapPin-Tracked",
            "the original pin consumer must read the true user-waypoint flag")
        point = C_Map.GetUserWaypoint()
        __assert_user_waypoint(point, 2248, 0.25, 0.75, nil)

        C_SuperTrack.SetSuperTrackedQuestID(78)
        assert(C_SuperTrack.IsSuperTrackingQuest() == true)
        assert(C_SuperTrack.IsSuperTrackingContent() == false)
        assert(C_SuperTrack.IsSuperTrackingMapPin() == false)
        local changesBeforeNoOp = changes
        assert(select('#', C_SuperTrack.SetSuperTrackedUserWaypoint(true)) == 0)
        assert(changes == changesBeforeNoOp)
        assert(C_SuperTrack.GetSuperTrackedQuestID() == 78,
            "an unchanged true request must not clear another target")
        local changesBeforeDisable = changes
        C_SuperTrack.SetSuperTrackedUserWaypoint(false)
        assert(changes == changesBeforeDisable)
        assert(C_SuperTrack.IsSuperTrackingUserWaypoint() == false)
        assert(C_SuperTrack.IsSuperTrackingQuest() == true)
        assert(C_SuperTrack.GetSuperTrackedQuestID() == 78)
        C_SuperTrack.SetSuperTrackedUserWaypoint("")
        assert(C_SuperTrack.IsSuperTrackingUserWaypoint() == true,
            "a non-nil non-false value must be true")

        local usage = "Usage: C_SuperTrack.SetSuperTrackedUserWaypoint(superTracked)"
        local missingOK, missingError = pcall(C_SuperTrack.SetSuperTrackedUserWaypoint)
        assert(not missingOK and missingError:find(usage, 1, true))
        local nilOK, nilError = pcall(C_SuperTrack.SetSuperTrackedUserWaypoint, nil)
        assert(not nilOK and nilError:find(usage, 1, true))
        C_SuperTrack.SetSuperTrackedUserWaypoint(false)
        assert(C_SuperTrack.IsSuperTrackingUserWaypoint() == false)
        __assert_user_waypoint(C_Map.GetUserWaypoint(), 2248, 0.25, 0.75, nil)

        C_SuperTrack.ClearAllSuperTracked()
        assert(C_SuperTrack.IsSuperTrackingUserWaypoint() == false)
        assert(C_SuperTrack.IsSuperTrackingQuest() == false)
        assert(C_SuperTrack.IsSuperTrackingContent() == false)
        assert(C_SuperTrack.IsSuperTrackingMapPin() == false)
        assert(C_SuperTrack.IsSuperTrackingCorpse() == false)
        assert(C_SuperTrack.IsSuperTrackingAnything() == false)
        __assert_user_waypoint(C_Map.GetUserWaypoint(), 2248, 0.25, 0.75, nil)

        C_SuperTrack.SetSuperTrackedUserWaypoint(true)
        assert(C_SuperTrack.IsSuperTrackingUserWaypoint() == true)
        assert(C_SuperTrack.IsSuperTrackingCorpse() == false)
        assert(C_SuperTrack.IsSuperTrackingAnything() == true,
            "IsSuperTrackingAnything must include only user-waypoint tracking")
        C_SuperTrack.ClearAllSuperTracked()
        assert(C_SuperTrack.IsSuperTrackingUserWaypoint() == false)
        assert(C_SuperTrack.IsSuperTrackingAnything() == false)
        __assert_user_waypoint(C_Map.GetUserWaypoint(), 2248, 0.25, 0.75, nil)

        C_Map.SetUserWaypoint({
            uiMapID = 2248.9,
            position = { x = "0.6", y = 0.4 },
            z = 12,
        })
        point = C_Map.GetUserWaypoint()
        __assert_user_waypoint(point, 2248, 0.6, 0.4, 12)

        local rejectedPoints = {
            { uiMapID = 0, position = { x = 0.6, y = 0.4 } },
            { uiMapID = 2248, position = { x = -0.01, y = 0.4 } },
            { uiMapID = 2248, position = { x = 0.6, y = 1.01 } },
            { uiMapID = 2248, position = { x = math.huge, y = 0.4 } },
            { uiMapID = 2248, position = { x = 0.6, y = 0 / 0 } },
        }
        for _, rejected in ipairs(rejectedPoints) do
            C_Map.SetUserWaypoint(rejected)
            point = C_Map.GetUserWaypoint()
            __assert_user_waypoint(point, 2248, 0.6, 0.4, 12)
        end

        C_Map.SetUserWaypoint({
            uiMapID = 2248,
            position = { x = 0.45, y = 0.35 },
            z = "not a number",
        })
        point = C_Map.GetUserWaypoint()
        __assert_user_waypoint(point, 2248, 0.45, 0.35, nil)

        local mapPosition = C_Map.GetUserWaypointPositionForMap(2248)
        local positionX, positionY = mapPosition:GetXY()
        assert(math.abs(positionX - 0.45) < 0.000001)
        assert(math.abs(positionY - 0.35) < 0.000001)

        provider:RefreshAllData()
        assert(provider.pin ~= nil)
        assert(math.abs(provider.pin.x - 0.45) < 0.000001)
        assert(math.abs(provider.pin.y - 0.35) < 0.000001)

        C_Map.ClearUserWaypoint()
        assert(C_Map.HasUserWaypoint() == false)
        assert(C_Map.GetUserWaypoint() == nil)
        provider:RefreshAllData()
        assert(provider.pin == nil)
        "#,
    )
    .expect("the original provider must observe the stored, replaced, and cleared waypoint");
}
