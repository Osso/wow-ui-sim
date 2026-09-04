//! Temporary Housing dashboard preload workaround.
//!
//! The real client receives housing/tutorial state from services we do not
//! model yet. Seed enough tutorial, neighborhood, and house-finder data for
//! the Blizzard housing dashboard frames to open without pretending these
//! service-backed values are complete `C_Housing` implementations.

use crate::lua_api::LoaderEnv;

const HOUSING_DASHBOARD_PRELOAD_WORKAROUND_LUA: &str = r#"
HousingTutorialUtil = HousingTutorialUtil or {}
if type(HousingTutorialUtil.BoughtHouseQuestComplete) ~= "function" then
    function HousingTutorialUtil.BoughtHouseQuestComplete()
        return true
    end
end

if type(C_Housing) == "table" then
    local function set_default(key, fn)
        if rawget(C_Housing, key) == nil then
            C_Housing[key] = fn
        end
    end

    set_default("GetPlayerOwnedHouses", function()
        local function dispatchOwnedHouses()
            if type(FireEvent) == "function" then
                FireEvent("PLAYER_HOUSE_LIST_UPDATED", {})
            end
        end
        if type(C_Timer) == "table" and type(C_Timer.After) == "function" then
            C_Timer.After(0, dispatchOwnedHouses)
        else
            dispatchOwnedHouses()
        end
    end)

    set_default("HouseFinderIgnoreNeighborhood", function()
    end)

    set_default("IsInsideOwnedHouse", function()
        return false
    end)

    set_default("IsInsideOwnedHouseOrPlot", function()
        return false
    end)

    set_default("IsInsideOwnedPlot", function()
        return false
    end)

    if type(ClearCachedActivitiesForPlayer) ~= "function" then
        function ClearCachedActivitiesForPlayer() end
    end

    local HOUSING_SIM_NEIGHBORHOODS = {
        {
            neighborhoodGUID = "wow-ui-sim-neighborhood-dawnmeadow",
            neighborhoodName = "Dawnmeadow",
            neighborhoodType = Enum.NeighborhoodType.Public,
            neighborhoodOwnerType = Enum.NeighborhoodOwnerType.None,
            suggestionReason = Enum.HouseFinderSuggestionReason.None,
        },
        {
            neighborhoodGUID = "wow-ui-sim-neighborhood-umber-grove",
            neighborhoodName = "Umber Grove",
            neighborhoodType = Enum.NeighborhoodType.Public,
            neighborhoodOwnerType = Enum.NeighborhoodOwnerType.None,
            suggestionReason = Enum.HouseFinderSuggestionReason.None,
        },
    }

    local HOUSING_SIM_MAP_IDS = {
        ["wow-ui-sim-neighborhood-dawnmeadow"] = 1,
        ["wow-ui-sim-neighborhood-umber-grove"] = 2248,
    }

    local HOUSING_SIM_TEXTURE_SUFFIXES = {
        ["wow-ui-sim-neighborhood-dawnmeadow"] = "elwynn",
        ["wow-ui-sim-neighborhood-umber-grove"] = "durotar",
    }

    function C_Housing.HouseFinderRequestNeighborhoods()
        if HouseFinderFrame and type(HouseFinderFrame.OnEvent) == "function" then
            HouseFinderFrame:OnEvent("NEIGHBORHOOD_LIST_UPDATED", Enum.HousingResult.Success, HOUSING_SIM_NEIGHBORHOODS)
        end
        if type(FireEvent) == "function" then
            FireEvent("NEIGHBORHOOD_LIST_UPDATED", Enum.HousingResult.Success, HOUSING_SIM_NEIGHBORHOODS)
        end
        local firstNeighborhood = HOUSING_SIM_NEIGHBORHOODS[1]
        if firstNeighborhood then
            C_Housing.RequestHouseFinderNeighborhoodData(firstNeighborhood.neighborhoodGUID, firstNeighborhood.neighborhoodName)
        end
    end

    function C_Housing.GetUIMapIDForNeighborhood(neighborhoodGUID)
        return HOUSING_SIM_MAP_IDS[neighborhoodGUID]
    end

    function C_Housing.GetNeighborhoodTextureSuffix(neighborhoodGUID)
        return HOUSING_SIM_TEXTURE_SUFFIXES[neighborhoodGUID]
    end

    function C_Housing.DoesFactionMatchNeighborhood(neighborhoodGUID)
        return true
    end

    function C_Housing.RequestHouseFinderNeighborhoodData(neighborhoodGUID, neighborhoodName)
        local mapPlotData = {
            {
                mapPosition = { x = 0.35, y = 0.46 },
                ownerType = Enum.HousingPlotOwnerType.None,
                plotID = 1,
                plotCost = 100000,
            },
            {
                mapPosition = { x = 0.62, y = 0.52 },
                ownerName = "Simfriend",
                ownerType = Enum.HousingPlotOwnerType.Friend,
                plotID = 2,
            },
        }
        local function dispatchNeighborhoodData()
            if HouseFinderFrame and type(HouseFinderFrame.OnEvent) == "function" then
                HouseFinderFrame:OnEvent("HOUSE_FINDER_NEIGHBORHOOD_DATA_RECIEVED", mapPlotData)
            end
            if type(FireEvent) == "function" then
                FireEvent("HOUSE_FINDER_NEIGHBORHOOD_DATA_RECIEVED", mapPlotData)
            end
        end
        if type(C_Timer) == "table" and type(C_Timer.After) == "function" then
            C_Timer.After(0, dispatchNeighborhoodData)
        else
            dispatchNeighborhoodData()
        end
    end

    function C_Housing.StartTutorial()
        if type(C_AddOns) == "table" and type(C_AddOns.LoadAddOn) == "function" then
            if not AUTOCOMPLETE_LIST or not AUTOCOMPLETE_LIST.HOUSE_FINDER then
                C_AddOns.LoadAddOn("Blizzard_AutoComplete")
            end
        end
        if not HouseFinderFrame and type(C_AddOns) == "table" and type(C_AddOns.LoadAddOn) == "function" then
            C_AddOns.LoadAddOn("Blizzard_HousingHouseFinder")
        end
        if HouseFinderFrame and type(ShowUIPanel) == "function" then
            ShowUIPanel(HouseFinderFrame)
        end
        if HouseFinderFrame and not HouseFinderFrame.hasNeighborhoodList then
            C_Housing.HouseFinderRequestNeighborhoods()
        end
        if HousingDashboardFrame and type(HideUIPanel) == "function" then
            HideUIPanel(HousingDashboardFrame)
        end
        return true
    end
end
"#;

pub(crate) fn patch(env: &LoaderEnv<'_>) {
    let _ = env.exec(HOUSING_DASHBOARD_PRELOAD_WORKAROUND_LUA);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lua_api::WowLuaEnv;

    #[test]
    fn seeds_house_finder_data_and_tutorial_flow() {
        let env = WowLuaEnv::new().expect("env should initialize");
        env.exec(
            r#"
            Enum = {
                HouseFinderSuggestionReason = { None = 0 },
                HousingPlotOwnerType = { Friend = 1, None = 0 },
                HousingResult = { Success = 0 },
                NeighborhoodOwnerType = { None = 0 },
                NeighborhoodType = { Public = 0 },
            }
            C_Housing = {}
            C_Timer = {
                After = function(_, callback)
                    callback()
                end,
            }
            C_AddOns = {
                loaded = {},
                LoadAddOn = function(addonName)
                    C_AddOns.loaded[addonName] = true
                end,
            }
            eventLog = {}
            function FireEvent(eventName, ...)
                table.insert(eventLog, { eventName, ... })
            end
            HouseFinderFrame = {
                hasNeighborhoodList = false,
                events = {},
                OnEvent = function(self, eventName, ...)
                    table.insert(self.events, { eventName, ... })
                    if eventName == "NEIGHBORHOOD_LIST_UPDATED" then
                        self.hasNeighborhoodList = true
                    end
                end,
            }
            HousingDashboardFrame = { hidden = false }
            function HideUIPanel(frame)
                frame.hidden = true
            end
            function ShowUIPanel(frame)
                frame.shown = true
            end
            "#,
        )
        .expect("housing test surface should install");

        patch(&env.loader_env());

        let (
            bought_house_complete,
            map_id,
            texture_suffix,
            faction_match,
            inside_house,
            inside_house_or_plot,
            inside_plot,
            started,
            hidden_dashboard,
            shown_finder,
            loaded_house_finder,
            neighborhood_events,
            event_count,
        ): (
            bool,
            i64,
            String,
            bool,
            bool,
            bool,
            bool,
            bool,
            bool,
            bool,
            bool,
            i64,
            i64,
        ) = env
            .eval(
                r#"
                local started = C_Housing.StartTutorial()
                return HousingTutorialUtil.BoughtHouseQuestComplete(),
                    C_Housing.GetUIMapIDForNeighborhood("wow-ui-sim-neighborhood-umber-grove"),
                    C_Housing.GetNeighborhoodTextureSuffix("wow-ui-sim-neighborhood-dawnmeadow"),
                    C_Housing.DoesFactionMatchNeighborhood("ignored"),
                    C_Housing.IsInsideOwnedHouse(),
                    C_Housing.IsInsideOwnedHouseOrPlot(),
                    C_Housing.IsInsideOwnedPlot(),
                    started,
                    HousingDashboardFrame.hidden,
                    HouseFinderFrame.shown,
                    C_AddOns.loaded.Blizzard_HousingHouseFinder == true,
                    #HouseFinderFrame.events,
                    #eventLog
                "#,
            )
            .expect("patched housing state should be readable");

        assert!(bought_house_complete);
        assert_eq!(map_id, 2248);
        assert_eq!(texture_suffix, "elwynn");
        assert!(faction_match);
        assert!(!inside_house);
        assert!(!inside_house_or_plot);
        assert!(!inside_plot);
        assert!(started);
        assert!(hidden_dashboard);
        assert!(shown_finder);
        assert!(!loaded_house_finder);
        assert_eq!(neighborhood_events, 2);
        assert_eq!(event_count, 2);
    }

    #[test]
    fn delays_owned_house_event_until_timer_callback() {
        let env = WowLuaEnv::new().expect("env should initialize");
        env.exec(
            r#"
            C_Housing = {}
            callbacks = {}
            C_Timer = {
                After = function(_, callback)
                    table.insert(callbacks, callback)
                end,
            }
            events = {}
            function FireEvent(eventName, ...)
                table.insert(events, { eventName, ... })
            end
            "#,
        )
        .expect("housing timer test surface should install");

        patch(&env.loader_env());

        let (events_before_tick, callbacks_scheduled): (i64, i64) = env
            .eval(
                r#"
                C_Housing.GetPlayerOwnedHouses()
                return #events, #callbacks
                "#,
            )
            .expect("owned house request should schedule its response");
        assert_eq!(events_before_tick, 0);
        assert_eq!(callbacks_scheduled, 1);

        let events_after_tick: i64 = env
            .eval(
                r#"
                callbacks[1]()
                return #events
                "#,
            )
            .expect("owned house response should dispatch after timer callback");
        assert_eq!(events_after_tick, 1);
    }
}
