use wow_ui_sim::lua_api::WowLuaEnv;

pub(crate) fn world_map_provider_sets_and_refreshes_user_waypoint(env: &WowLuaEnv) {
    env.exec(
        r#"
        local loaded, reason = C_AddOns.LoadAddOn("Blizzard_SharedMapDataProviders")
        assert(loaded, "could not load Blizzard_SharedMapDataProviders: " .. tostring(reason))
        assert(type(WaypointLocationDataProviderMixin) == "table")
        assert(type(WaypointLocationDataProviderMixin.RefreshAllData) == "function")
        assert(type(WaypointLocationDataProviderMixin.HandleClick) == "function")

        local function assert_point(point, mapID, expectedX, expectedY, expectedZ)
            assert(point and point.uiMapID == mapID)
            local x, y = point.position:GetXY()
            assert(math.abs(x - expectedX) < 0.000001)
            assert(math.abs(y - expectedY) < 0.000001)
            assert(point.z == expectedZ)
        end

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
                    SetPosition = function(_, x, y)
                        pin.x, pin.y = x, y
                    end,
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

        provider:RefreshAllData()
        assert(provider.pin == nil, "a map without a user waypoint must not get a pin")

        provider:HandleClick()
        local point = C_Map.GetUserWaypoint()
        assert_point(point, 2248, 0.25, 0.75, nil)
        assert(C_Map.HasUserWaypoint() == true)

        provider:RefreshAllData()
        assert(provider.pin ~= nil, "refresh must acquire the waypoint pin")
        assert(math.abs(provider.pin.x - 0.25) < 0.000001)
        assert(math.abs(provider.pin.y - 0.75) < 0.000001)

        C_Map.SetUserWaypoint({
            uiMapID = 2248.9,
            position = { x = "0.6", y = 0.4 },
            z = 12,
        })
        point = C_Map.GetUserWaypoint()
        assert_point(point, 2248, 0.6, 0.4, 12)

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
            assert_point(point, 2248, 0.6, 0.4, 12)
        end

        C_Map.SetUserWaypoint({
            uiMapID = 2248,
            position = { x = 0.45, y = 0.35 },
            z = "not a number",
        })
        point = C_Map.GetUserWaypoint()
        assert_point(point, 2248, 0.45, 0.35, nil)

        local mapPosition = C_Map.GetUserWaypointPositionForMap(2248)
        local positionX, positionY = mapPosition:GetXY()
        assert(math.abs(positionX - 0.45) < 0.000001)
        assert(math.abs(positionY - 0.35) < 0.000001)

        provider:RefreshAllData()
        assert(provider.pin ~= nil)
        assert(math.abs(provider.pin.x - 0.45) < 0.000001)
        assert(math.abs(provider.pin.y - 0.35) < 0.000001)
        provider.pin.IsMouseOver = function() return true end
        provider:HandleClick()
        assert(C_Map.HasUserWaypoint() == false)
        assert(C_Map.GetUserWaypoint() == nil)
        provider:RefreshAllData()
        assert(provider.pin == nil)
        "#,
    )
    .expect("the original waypoint provider must set, refresh, and clear a user waypoint");
}
