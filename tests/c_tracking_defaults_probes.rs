use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::screen::ScreenKind;

fn env() -> WowLuaEnv {
    WowLuaEnv::new().expect("Failed to create Lua environment")
}

#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
#[test]
fn content_tracking_defaults_are_empty_and_inactive() {
    let env = env();
    let result: String = env
        .eval(
            r#"
            local tracked = C_ContentTracking.GetTrackedIDs(0)
            if type(tracked) ~= "table" or #tracked ~= 0 then return "tracked" end
            if C_ContentTracking.IsTracking(0, 42) ~= false then return "tracking" end
            return "ok"
            "#,
        )
        .expect("content tracking defaults should be callable");

    assert_eq!(result, "ok");
}

#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
#[test]
fn tracking_consumers_reflect_supplied_tracking_entries() {
    let env = consumer_env();
    assert_tracking_consumers_loaded(&env);
    assert!(collections_shows_disabled_prompt(&env));
    seed_tracking_inputs(&env);
    env.exec("A_Admin.SetContentTrackingEnabled(true)")
        .expect("enable content tracking through the admin setter");

    let consumer_states = consumer_tracking_states(&env);
    let enabled: bool = env
        .eval("return C_ContentTracking.GetCollectableSourceTrackingEnabled()")
        .expect("content-tracking capability query should run");
    assert_eq!(consumer_states, (true, true, false));
    assert!(
        enabled,
        "the C API must return the supplied capability state"
    );
}

#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
fn assert_tracking_consumers_loaded(env: &WowLuaEnv) {
    for addon in ["Blizzard_ContentTracking", "Blizzard_FrameXMLUtil"] {
        let loaded: bool = env
            .eval(&format!("return C_AddOns.IsAddOnLoaded({addon:?})"))
            .expect("tracking consumer load state should be available");
        assert!(loaded, "selected consumer addon {addon} must be loaded");
    }
}

#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
fn seed_tracking_inputs(env: &WowLuaEnv) {
    let appearance_type: i32 = env
        .eval("return Enum.ContentTrackingType.Appearance")
        .expect("Appearance tracking type should be available");
    env.state()
        .borrow_mut()
        .content_tracking
        .tracked_ids_by_type
        .insert(appearance_type as u32, vec![301, 302]);
}

#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
fn collections_shows_disabled_prompt(env: &WowLuaEnv) -> bool {
    env.eval(
        r#"
        local tooltip = { lines = {} }
        function tooltip:AddLine(text) table.insert(self.lines, text) end
        CollectionWardrobeUtil.AddTrackingTooltipLine(tooltip, 301)
        return tooltip.lines[1] == CONTENT_TRACKING_DISABLED_TOOLTIP_PROMPT
        "#,
    )
    .expect("CollectionsUtil should show the disabled-tracking prompt")
}

#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
fn consumer_env() -> WowLuaEnv {
    let env = WowLuaEnv::new().expect("Lua environment should initialize");
    env.set_screen_size(1024.0, 768.0);
    env.set_screen_mode(ScreenKind::Game);
    let ui = wow_ui_sim::paths::default_blizzard_ui_addons_path()
        .expect("profile Blizzard UI cache should exist");
    env.state().borrow_mut().addon_base_paths = vec![ui.clone()];
    wow_ui_sim::xml::register_intrinsic_templates();
    for (name, toc) in
        wow_ui_sim::loader::discover_blizzard_addons_for_screen(&ui, ScreenKind::Game)
    {
        wow_ui_sim::loader::load_addon(&env.loader_env(), &toc)
            .unwrap_or_else(|error| panic!("load {name}: {error}"));
    }
    env.apply_post_load_workarounds();
    wow_ui_sim::startup::fire_startup_events_for_screen(&env, ScreenKind::Game);
    env
}

#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
fn consumer_tracking_states(env: &WowLuaEnv) -> (bool, bool, bool) {
    env.eval(
        r#"
        local trackableType = Enum.ContentTrackingType.Appearance
        local function isShownFor(trackableID)
            local element = CreateFrame("Frame")
            Mixin(element, ContentTrackingElementMixin)
            element.ContentTrackingCheckmark = element:CreateTexture(nil, "OVERLAY")
            element:AddTrackable(trackableType, trackableID)
            element:UpdateTrackingCheckmark()
            return element.ContentTrackingCheckmark:IsShown()
        end
        return isShownFor(301), isShownFor(302), isShownFor(303)
        "#,
    )
    .expect("ContentTrackingElement should render the supplied tracked state")
}

post_wrath_ui_fixture! {
#[test]
fn neighborhood_initiative_defaults_have_empty_tracked_ids_shape() {
    let env = env();
    let result: String = env
        .eval(
            r#"
            if type(C_NeighborhoodInitiative.IsInitiativeEnabled) ~= "function" then return "enabled" end
            local tracked = C_NeighborhoodInitiative.GetTrackedInitiativeTasks()
            if type(tracked) ~= "table" then return "tracked-table" end
            if type(tracked.trackedIDs) ~= "table" or #tracked.trackedIDs ~= 0 then return "ids" end
            if type(C_NeighborhoodInitiative.GetInitiativeTaskInfo) ~= "function" then return "info" end
            if C_NeighborhoodInitiative.RemoveTrackedInitiativeTask(1) ~= nil then return "remove" end
            if C_NeighborhoodInitiative.AddTrackedInitiativeTask(1) ~= nil then return "add" end
            return "ok"
            "#,
        )
        .expect("neighborhood initiative defaults should be callable");

    assert_eq!(result, "ok");
}
}
