//! Garrison landing-page gate for `C_ArdenwealdGardening.IsGardenAccessible`.

use wow_ui_sim::loader::BlizzardAddonOverride;
use wow_ui_sim::lua_api::WowLuaEnv;

use crate::common::blizzard_addon_harness::with_blizzard_addon_smoke_shape;

const NATURAL_CALLER: &str = "Blizzard_GarrisonUI";
const ROOT: &str = "Blizzard_ArdenwealdGardening";
const GARDENING_STARTUP_ROOT: BlizzardAddonOverride<'static> = BlizzardAddonOverride {
    addon: NATURAL_CALLER,
    extra_roots: &[ROOT],
};

// Forever has no natural caller: Blizzard_GarrisonUI.toc:11 declares
// `ExcludeLoadGameType: camelot` (see docs/specs/garrison-mission-lists.md).
#[cfg(not(feature = "client-wowforever"))]
#[test]
fn garden_accessibility_gates_landing_page_panel_load() {
    with_blizzard_addon_smoke_shape(
        &[NATURAL_CALLER],
        &[GARDENING_STARTUP_ROOT],
        |env, loaded| {
            assert!(
                loaded.iter().any(|name| name == NATURAL_CALLER),
                "`{NATURAL_CALLER}` must load before probing its garden section"
            );
            assert!(
                loaded.iter().any(|name| name == ROOT),
                "`{ROOT}` bootstrap must load before GarrisonUI calls its publisher"
            );
            let publisher_type: String = env
                .eval("return type(ArdenwealdGardening_LoadUI)")
                .expect("garden publisher type probe must run cleanly");
            assert_eq!(
                publisher_type, "function",
                "`{ROOT}` bootstrap must publish ArdenwealdGardening_LoadUI before the gate"
            );

            seed_garden_accessibility(env, false);
            let inaccessible = run_landing_page_garden_probe(env);
            assert_inaccessible_probe(inaccessible);

            seed_garden_accessibility(env, true);
            let accessible = run_landing_page_garden_probe(env);
            assert_accessible_probe(accessible);
        },
    );
}

type GardenGateProbe = (f64, f64, String, bool, bool, bool);

fn seed_garden_accessibility(env: &WowLuaEnv, accessible: bool) {
    env.state().borrow_mut().gardenweald.accessible = accessible;
}

fn run_landing_page_garden_probe(env: &WowLuaEnv) -> GardenGateProbe {
    env.eval(
        r#"
        local publisherCalls = 0
        local loadRequests = {}
        local originalPublisher = ArdenwealdGardening_LoadUI
        local originalLoadAddOn = LoadAddOnWithErrorHandling

        ArdenwealdGardening_LoadUI = function()
            publisherCalls = publisherCalls + 1
            return originalPublisher()
        end
        LoadAddOnWithErrorHandling = function(name)
            loadRequests[#loadRequests + 1] = name
            return originalLoadAddOn(name)
        end

        GarrisonLandingPage:SetupGardenweald()
        ArdenwealdGardening_LoadUI = originalPublisher
        LoadAddOnWithErrorHandling = originalLoadAddOn

        local panel = GarrisonLandingPage.ArdenwealdGardeningPanel
        return publisherCalls,
               #loadRequests,
               loadRequests[1] or "",
               panel ~= nil,
               panel and panel:GetParent() == GarrisonLandingPage.Report.Sections or false,
               panel and panel:IsShown() or false
        "#,
    )
    .expect("Garrison landing-page garden gate probe must run cleanly")
}

fn assert_inaccessible_probe(probe: GardenGateProbe) {
    let (
        publisher_calls,
        load_request_count,
        loaded_name,
        panel_exists,
        panel_parent_matches,
        panel_shown,
    ) = probe;

    assert_eq!(
        publisher_calls, 0.0,
        "inaccessible garden must not call ArdenwealdGardening_LoadUI"
    );
    assert_eq!(
        load_request_count, 0.0,
        "inaccessible garden must not request an addon"
    );
    assert_eq!(loaded_name, "", "inaccessible garden must not request an addon");
    assert!(
        !panel_exists,
        "inaccessible garden must not instantiate ArdenwealdGardeningPanel"
    );
    assert!(
        !panel_parent_matches,
        "inaccessible garden must not attach a garden panel to Report.Sections"
    );
    assert!(!panel_shown, "inaccessible garden must not show a panel");
}

fn assert_accessible_probe(probe: GardenGateProbe) {
    let (
        publisher_calls,
        load_request_count,
        loaded_name,
        panel_exists,
        panel_parent_matches,
        panel_shown,
    ) = probe;

    assert_eq!(
        publisher_calls, 1.0,
        "accessible garden must call the Ardenweald Gardening publisher exactly once"
    );
    assert_eq!(
        load_request_count, 1.0,
        "accessible garden publisher must request the Ardenweald Gardening addon exactly once"
    );
    assert_eq!(
        loaded_name, ROOT,
        "accessible garden must load Blizzard_ArdenwealdGardening"
    );
    assert!(panel_exists, "accessible garden must instantiate the panel");
    assert!(
        panel_parent_matches,
        "created garden panel must be attached to GarrisonLandingPage.Report.Sections"
    );
    assert!(panel_shown, "created garden panel must be shown");
}
