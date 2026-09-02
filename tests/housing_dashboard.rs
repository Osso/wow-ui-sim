#![cfg(feature = "gui")]

use crate::common;

use iced::{Point, Rectangle};
use std::collections::HashMap;
use wow_ui_sim::iced_app::{build_hittable_rects, frame_collect::collect_hittable_frames};
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::startup::process_pending_timers;
use wow_ui_sim::widget::{Frame, WidgetRegistry};

fn hit_test_like_gui(env: &WowLuaEnv, pos: Point) -> Option<u64> {
    let mut state = env.state().borrow_mut();
    state.ensure_layout_rects();
    let strata_buckets = state
        .get_strata_buckets()
        .expect("visible strata buckets should exist")
        .clone();
    let collected = collect_hittable_frames(&state.widgets, &strata_buckets);
    let hittable = build_hittable_rects(&collected, &state.widgets);
    let hittable_by_id: HashMap<u64, Rectangle> =
        hittable.iter().map(|&(id, rect, _)| (id, rect)).collect();

    // Phase 1: topmost hittable frame whose subtree contains a hittable target at pos.
    let initial_id = hittable.iter().rev().find_map(|(id, rect, _)| {
        (rect.contains(pos) && deepest_hover(&state.widgets, &hittable_by_id, *id, pos).is_some())
            .then_some(*id)
    })?;
    deepest_hover(&state.widgets, &hittable_by_id, initial_id, pos)
}

fn deepest_hover(
    widgets: &WidgetRegistry,
    grid: &HashMap<u64, Rectangle>,
    frame_id: u64,
    pos: Point,
) -> Option<u64> {
    let frame = widgets.get(frame_id)?;
    for child_id in test_children_at_point_by_z_order(widgets, frame, pos) {
        if let Some(found) = deepest_hover(widgets, grid, child_id, pos) {
            return Some(found);
        }
    }
    grid.get(&frame_id)
        .and_then(|rect| rect.contains(pos).then_some(frame_id))
}

fn test_children_at_point_by_z_order(
    widgets: &WidgetRegistry,
    frame: &Frame,
    pos: Point,
) -> Vec<u64> {
    let mut child_ids: Vec<u64> = frame
        .children
        .iter()
        .copied()
        .filter(|cid| {
            widgets
                .get(*cid)
                .is_some_and(|c| test_child_visually_contains(c, pos))
        })
        .collect();
    child_ids.sort_by_key(|cid| {
        let f = widgets.get(*cid);
        let level = f.map(|f| f.frame_level + f.raise_order).unwrap_or(0);
        let strata = f.map(|f| f.frame_strata as i32).unwrap_or(0);
        (strata, level, *cid)
    });
    child_ids.reverse();
    child_ids
}

fn test_child_visually_contains(child: &Frame, pos: Point) -> bool {
    if !child.visible || child.effective_alpha <= 0.0 {
        return false;
    }
    let Some(rect) = child.layout_rect else {
        return false;
    };
    pos.x >= rect.x
        && pos.x < rect.x + rect.width
        && pos.y >= rect.y
        && pos.y < rect.y + rect.height
}

fn setup_env() -> WowLuaEnv {
    common::panel_fixtures::setup_env()
}

#[test]
fn start_tutorial_button_opens_house_finder() {
    test_timeout! {
        let env = setup_env();
        let result: String = env
            .eval(
                r#"
                    local tutorialsLoaded, tutorialsReason = C_AddOns.LoadAddOn("Blizzard_HousingTutorials")
                    if not tutorialsLoaded then
                        return "tutorials_load_failed:" .. tostring(tutorialsReason)
                    end

                    local loaded, reason = C_AddOns.LoadAddOn("Blizzard_HousingDashboard")
                    if not loaded then
                        return "dashboard_load_failed:" .. tostring(reason)
                    end

                    ShowUIPanel(HousingDashboardFrame)
                    local button = HousingDashboardFrame.HouseInfoContent.DashboardNoHousesFrame.NoHouseButton
                    if not button or button:GetText() ~= HOUSING_DASHBOARD_START_TUTORIAL_BUTTON_TEXT then
                        return "missing_start_tutorial_button"
                    end

                    local onclick = button:GetScript("OnClick")
                    if not onclick then
                        return "missing_onclick"
                    end

                    local ok, err = pcall(function()
                        onclick(button, "LeftButton", false)
                    end)
                    if not ok then
                        return "click_failed:" .. tostring(err)
                    end

                    if not HouseFinderFrame or not HouseFinderFrame:IsShown() then
                        return "house_finder_not_shown"
                    end

                    if HousingDashboardFrame:IsShown() then
                        return "dashboard_still_shown"
                    end

                    return "ok"
                "#,
            )
            .unwrap();
        assert_eq!(
            result, "ok",
            "Housing tutorial button should advance to the house finder: {result}"
        );
    }
}

#[test]
fn dashboard_house_list_request_hides_main_spinner() {
    test_timeout! {
        let env = setup_env();
        let result: String = env
            .eval(
                r#"
                    local loaded, reason = C_AddOns.LoadAddOn("Blizzard_HousingDashboard")
                    if not loaded then
                        return "dashboard_load_failed:" .. tostring(reason)
                    end

                    C_Housing.GetPlayerOwnedHouses = function()
                        HousingDashboardFrame.HouseDropdown:OnHouseListUpdated({})
                    end
                    ShowUIPanel(HousingDashboardFrame)
                    HousingDashboardFrame.HouseDropdown:LoadHouses()

                    if HousingDashboardFrame.HouseInfoContent.LoadingSpinner:IsShown() then
                        return "spinner_still_shown"
                    end
                    if not HousingDashboardFrame.HouseInfoContent.DashboardNoHousesFrame:IsShown() then
                        return "empty_dashboard_not_shown"
                    end

                    return "ok"
                "#,
            )
            .unwrap();
        assert_eq!(
            result, "ok",
            "Housing dashboard should resolve the owned-house request: {result}"
        );
    }
}

#[test]
fn house_finder_initial_neighborhood_loads_initial_map() {
    test_timeout! {
        let env = setup_env();
        let open_result: String = env
            .eval(
                r#"
                    local tutorialsLoaded, tutorialsReason = C_AddOns.LoadAddOn("Blizzard_HousingTutorials")
                    if not tutorialsLoaded then
                        return "tutorials_load_failed:" .. tostring(tutorialsReason)
                    end

                    local loaded, reason = C_AddOns.LoadAddOn("Blizzard_HousingDashboard")
                    if not loaded then
                        return "dashboard_load_failed:" .. tostring(reason)
                    end

                    ShowUIPanel(HousingDashboardFrame)
                    local startButton = HousingDashboardFrame.HouseInfoContent.DashboardNoHousesFrame.NoHouseButton
                    local onclick = startButton and startButton:GetScript("OnClick")
                    if not onclick then
                        return "missing_start_tutorial_onclick"
                    end
                    local ok, err = pcall(function()
                        onclick(startButton, "LeftButton", false)
                    end)
                    if not ok then
                        return "start_tutorial_click_failed:" .. tostring(err)
                    end

                    if not HouseFinderFrame.selectedNeighborhoodButton then
                        return "missing_selected_neighborhood"
                    end
                    if not HouseFinderFrame.LoadingSpinnerMap:IsShown() then
                        return "spinner_not_shown_before_data"
                    end

                    return "opened"
                "#,
            )
            .unwrap();
        assert_eq!(
            open_result, "opened",
            "House finder should request initial map data after opening: {open_result}"
        );

        process_pending_timers(&env);

        let map_result: String = env
            .eval(
                r#"
                    if not HouseFinderFrame.HouseFinderMapCanvasFrame:IsShown() then
                        return "map_not_shown"
                    end
                    if HouseFinderFrame.LoadingSpinnerMap:IsShown() then
                        return "map_spinner_still_shown"
                    end
                    return "ok"
                "#,
            )
            .unwrap();
        assert_eq!(
            map_result, "ok",
            "House finder should load the initially selected neighborhood map: {map_result}"
        );
    }
}

#[test]
fn neighborhood_selector_populates_and_click_loads_selected_map() {
    test_timeout! {
        let env = setup_env();
        let click_result: String = env
            .eval(
                r#"
                    local tutorialsLoaded, tutorialsReason = C_AddOns.LoadAddOn("Blizzard_HousingTutorials")
                    if not tutorialsLoaded then
                        return "tutorials_load_failed:" .. tostring(tutorialsReason)
                    end

                    local loaded, reason = C_AddOns.LoadAddOn("Blizzard_HousingDashboard")
                    if not loaded then
                        return "dashboard_load_failed:" .. tostring(reason)
                    end

                    ShowUIPanel(HousingDashboardFrame)
                    local startButton = HousingDashboardFrame.HouseInfoContent.DashboardNoHousesFrame.NoHouseButton
                    local onclick = startButton and startButton:GetScript("OnClick")
                    if not onclick then
                        return "missing_start_tutorial_onclick"
                    end
                    local ok, err = pcall(function()
                        onclick(startButton, "LeftButton", false)
                    end)
                    if not ok then
                        return "start_tutorial_click_failed:" .. tostring(err)
                    end

                    local firstButton = nil
                    local secondButton = nil
                    for button in HouseFinderFrame.neighborhoodButtonPool:EnumerateActive() do
                        if button.layoutIndex == 1 then
                            firstButton = button
                        elseif button.layoutIndex == 2 then
                            secondButton = button
                        end
                    end

                    if not firstButton or not secondButton then
                        return "missing_neighborhood_buttons"
                    end
                    if firstButton.neighborhoodInfo.neighborhoodName ~= "Dawnmeadow" then
                        return "wrong_first_neighborhood:" .. tostring(firstButton.neighborhoodInfo.neighborhoodName)
                    end
                    if secondButton.neighborhoodInfo.neighborhoodName ~= "Umber Grove" then
                        return "wrong_second_neighborhood:" .. tostring(secondButton.neighborhoodInfo.neighborhoodName)
                    end

                    secondButton:OnClick()

                    if HouseFinderFrame.selectedNeighborhoodButton ~= secondButton then
                        return "second_neighborhood_not_selected"
                    end
                    if HouseFinderFrame.LoadingSpinnerMap:IsShown() then
                        return "clicked"
                    end

                    return "spinner_not_shown_after_click"
                "#,
            )
            .unwrap();
        assert_eq!(
            click_result, "clicked",
            "Neighborhood selector should request selected map data: {click_result}"
        );

        process_pending_timers(&env);

        let map_result: String = env
            .eval(
                r#"
                    if not HouseFinderFrame.HouseFinderMapCanvasFrame:IsShown() then
                        return "map_not_shown"
                    end
                    if HouseFinderFrame.LoadingSpinnerMap:IsShown() then
                        return "map_spinner_still_shown"
                    end
                    return "ok"
                "#,
            )
            .unwrap();
        assert_eq!(
            map_result, "ok",
            "Neighborhood selector should load selected map data: {map_result}"
        );
    }
}

#[test]
fn plot_pin_click_selects_plot_and_shows_info() {
    test_timeout! {
        let env = setup_env();
        let open_result: String = env
            .eval(
                r#"
                    local tutorialsLoaded, tutorialsReason = C_AddOns.LoadAddOn("Blizzard_HousingTutorials")
                    if not tutorialsLoaded then
                        return "tutorials_load_failed:" .. tostring(tutorialsReason)
                    end

                    local loaded, reason = C_AddOns.LoadAddOn("Blizzard_HousingDashboard")
                    if not loaded then
                        return "dashboard_load_failed:" .. tostring(reason)
                    end

                    ShowUIPanel(HousingDashboardFrame)
                    local startButton = HousingDashboardFrame.HouseInfoContent.DashboardNoHousesFrame.NoHouseButton
                    local onclick = startButton and startButton:GetScript("OnClick")
                    if not onclick then
                        return "missing_start_tutorial_onclick"
                    end
                    local ok, err = pcall(function()
                        onclick(startButton, "LeftButton", false)
                    end)
                    if not ok then
                        return "start_tutorial_click_failed:" .. tostring(err)
                    end

                    return "opened"
                "#,
            )
            .unwrap();
        assert_eq!(open_result, "opened", "House finder should open: {open_result}");

        process_pending_timers(&env);

        let click_result: String = env
            .eval(
                r#"
                    local map = HouseFinderFrame and HouseFinderFrame.HouseFinderMapCanvasFrame
                    if not map then return "no_map" end
                    if not map.pinPools then return "no_pin_pools" end

                    local pool = map.pinPools["HouseFinderPlotForSalePinTemplate"]
                    if not pool then return "no_for_sale_pool" end

                    local pin
                    for active in pool:EnumerateActive() do
                        pin = active
                        break
                    end
                    if not pin then return "no_active_for_sale_pin" end

                    if not pin:IsShown() then return "pin_hidden" end

                    local ok, err = pcall(function()
                        pin:OnClick("LeftButton")
                    end)
                    if not ok then return "click_err:" .. tostring(err) end

                    if not HouseFinderFrame.PlotInfoFrame or not HouseFinderFrame.PlotInfoFrame:IsShown() then
                        return "plot_info_not_shown"
                    end
                    if not HouseFinderFrame.SelectedPlotTooltip or not HouseFinderFrame.SelectedPlotTooltip:IsShown() then
                        return "selected_tooltip_not_shown"
                    end

                    return "ok"
                "#,
            )
            .unwrap();
        assert_eq!(
            click_result, "ok",
            "Clicking a for-sale plot pin should select it: {click_result}"
        );
    }
}

#[test]
fn plot_pin_is_hit_testable_at_screen_position() {
    test_timeout! {
        let env = setup_env();
        let opened: String = env
            .eval(
                r#"
                    local tutorialsLoaded = C_AddOns.LoadAddOn("Blizzard_HousingTutorials")
                    if not tutorialsLoaded then return "tutorials_load_failed" end
                    local loaded = C_AddOns.LoadAddOn("Blizzard_HousingDashboard")
                    if not loaded then return "dashboard_load_failed" end
                    ShowUIPanel(HousingDashboardFrame)
                    local startButton = HousingDashboardFrame.HouseInfoContent.DashboardNoHousesFrame.NoHouseButton
                    local onclick = startButton:GetScript("OnClick")
                    onclick(startButton, "LeftButton", false)
                    return "opened"
                "#,
            )
            .unwrap();
        assert_eq!(opened, "opened");

        process_pending_timers(&env);

        // Find the pin's screen-space rect via the registry and aim the hit-test there.
        let (pin_id_from_registry, pos) = {
            let mut state = env.state().borrow_mut();
            state.ensure_layout_rects();
            let mut pin_id: Option<u64> = None;
            let mut pin_rect = None;
            for id in state.widgets.iter_ids() {
                let Some(f) = state.widgets.get(id) else { continue };
                if f.children_keys.contains_key("SelectedUnderlay")
                    && f.children_keys.contains_key("HighlightTexture")
                    && f.visible
                {
                    pin_id = Some(id);
                    pin_rect = f.layout_rect;
                    break;
                }
            }
            let pin_id = pin_id.expect("for-sale pin frame should exist");
            let rect = pin_rect.expect("pin should have layout rect");
            (
                pin_id,
                Point::new(rect.x + rect.width / 2.0, rect.y + rect.height / 2.0),
            )
        };

        let hit_id = hit_test_like_gui(&env, pos);
        assert!(
            hit_id.is_some(),
            "hit-test at pin center {pos:?} returned nothing — pin is not reachable by mouse"
        );

        let pin_frame_id = pin_id_from_registry;
        let pin_in_chain = {
            let state = env.state().borrow();
            let mut cur = hit_id;
            let mut found = false;
            while let Some(id) = cur {
                if id == pin_frame_id { found = true; break; }
                let Some(frame) = state.widgets.get(id) else { break };
                cur = frame.parent_id;
            }
            found
        };

        if !pin_in_chain {
            let (chain, pin_in_grid, pin_strata, pin_level, hit_strata, hit_level, pin_grid_rect, pin_contains, last_hittable) = {
                let mut state = env.state().borrow_mut();
                state.ensure_layout_rects();
                let strata_buckets = state
                    .get_strata_buckets()
                    .expect("visible strata buckets")
                    .clone();
                let collected = collect_hittable_frames(&state.widgets, &strata_buckets);
                let pin_in_grid = collected.hittable.iter().any(|(id, _, _)| *id == pin_frame_id);
                let pin_grid_rect = collected
                    .hittable
                    .iter()
                    .find(|(id, _, _)| *id == pin_frame_id)
                    .map(|(_, _, r)| *r);
                let pin_contains = pin_grid_rect.map(|r| {
                    pos.x >= r.x
                        && pos.x < r.x + r.width
                        && pos.y >= r.y
                        && pos.y < r.y + r.height
                });
                let last_hittable: Vec<String> = collected
                    .hittable
                    .iter()
                    .rev()
                    .take(8)
                    .map(|(id, _, r)| {
                        let f = state.widgets.get(*id);
                        let raise = f.map(|f| f.raise_order).unwrap_or(0);
                        let level = f.map(|f| f.frame_level).unwrap_or(0);
                        let strata = f.map(|f| f.frame_strata);
                        let name = f.and_then(|f| f.name.clone());
                        format!(
                            "id={id} name={name:?} strata={strata:?} level={level} raise={raise} effective={} rect={r:?}",
                            level + raise
                        )
                    })
                    .collect();
                let mut chain: Vec<String> = Vec::new();
                let mut cur = hit_id;
                while let Some(id) = cur {
                    let Some(frame) = state.widgets.get(id) else { break };
                    chain.push(format!(
                        "id={id} name={:?} type={:?} mouse={} alpha={} strata={:?} level={} rect={:?} keys={:?}",
                        frame.name,
                        frame.widget_type,
                        frame.mouse_enabled,
                        frame.effective_alpha,
                        frame.frame_strata,
                        frame.frame_level,
                        frame.layout_rect,
                        frame.children_keys.keys().collect::<Vec<_>>(),
                    ));
                    cur = frame.parent_id;
                }
                let pin = state.widgets.get(pin_frame_id).expect("pin frame");
                let hit = hit_id.and_then(|id| state.widgets.get(id));
                (
                    chain,
                    pin_in_grid,
                    pin.frame_strata,
                    pin.frame_level,
                    hit.map(|f| f.frame_strata),
                    hit.map(|f| f.frame_level),
                    pin_grid_rect,
                    pin_contains,
                    last_hittable,
                )
            };
            // Walk pin's parent chain to see ancestor structure.
            let pin_chain: Vec<String> = {
                let state = env.state().borrow();
                let mut chain: Vec<String> = Vec::new();
                let mut cur = Some(pin_frame_id);
                while let Some(id) = cur {
                    let Some(frame) = state.widgets.get(id) else { break };
                    chain.push(format!(
                        "id={id} name={:?} type={:?} mouse={} alpha={} strata={:?} level={} raise={} rect={:?}",
                        frame.name,
                        frame.widget_type,
                        frame.mouse_enabled,
                        frame.effective_alpha,
                        frame.frame_strata,
                        frame.frame_level,
                        frame.raise_order,
                        frame.layout_rect,
                    ));
                    cur = frame.parent_id;
                }
                chain
            };
            panic!(
                "hit at pin center {pos:?} hit frame {hit_id:?} (NOT pin id {pin_frame_id}).\n\
                 pin in hit grid? {pin_in_grid}\n\
                 pin grid rect: {pin_grid_rect:?}\n\
                 pin grid rect contains pos? {pin_contains:?}\n\
                 pin strata={pin_strata:?} level={pin_level}\n\
                 hit strata={hit_strata:?} level={hit_level:?}\n\
                 last 8 hittable (top of stack):\n{}\n\
                 pin → root chain:\n{}\n\
                 hit chain:\n{}",
                last_hittable.join("\n"),
                pin_chain.join("\n"),
                chain.join("\n")
            );
        }
    }
}
