//! Load smoke for `Blizzard_ArtifactUI`.

use crate::common::blizzard_addon_harness::with_blizzard_addon_smoke_shape;
use crate::common::panel_fixtures::{
    blizzard_ui_dir, clear_recorded_lua_errors, recorded_lua_errors,
};
use wow_ui_sim::loader::discover_blizzard_addon_closure_for_screen;
use wow_ui_sim::screen::ScreenKind;
use wow_ui_sim::startup::settle_headless_startup;
use wow_ui_sim::toc::TocFile;

const ROOT: &str = "Blizzard_ArtifactUI";
const COLORS: &str = "Blizzard_Colors";

post_wrath_ui_fixture! {
#[cfg(feature = "optional-tests")]
#[test]
fn artifact_ui_toc_dependency_loads_blizzard_colors_first() {
    let toc = TocFile::from_file(&artifact_ui_toc()).expect("Blizzard_ArtifactUI TOC should parse");
    assert_eq!(
        toc.dependencies(),
        [COLORS],
        "`{ROOT}` must declare its hard dependency on `{COLORS}`"
    );

    let closure =
        discover_blizzard_addon_closure_for_screen(&blizzard_ui_dir(), ScreenKind::Game, &[ROOT]);
    let loaded_names = closure
        .iter()
        .map(|(name, _toc)| name.as_str())
        .collect::<Vec<_>>();
    let colors_index = loaded_names
        .iter()
        .position(|name| *name == COLORS)
        .unwrap_or_else(|| panic!("dependency closure for `{ROOT}` must include `{COLORS}`"));
    let artifact_index = loaded_names
        .iter()
        .position(|name| *name == ROOT)
        .unwrap_or_else(|| panic!("dependency closure must include root addon `{ROOT}`"));

    assert!(
        colors_index < artifact_index,
        "`{COLORS}` must be loaded before `{ROOT}` in the dependency-aware loader closure: \
         {loaded_names:?}"
    );
}
}

// ArtifactUI vendor handlers/templates do not ship in Wrath; modeled API probes stay enabled.
post_wrath_ui_fixture! {
#[cfg(feature = "optional-tests")]
#[test]
fn artifact_ui_is_load_on_demand_until_explicitly_loaded() {
    with_blizzard_addon_smoke_shape(&[], &[], |env, _loaded| {
        let loaded_before: bool = env
            .eval(r#"return C_AddOns.IsAddOnLoaded("Blizzard_ArtifactUI")"#)
            .expect("pre-load IsAddOnLoaded probe should run cleanly");
        assert!(
            !loaded_before,
            "`{ROOT}` declares `LoadOnDemand: 1`, so startup must not load it eagerly"
        );

        let (loaded, error): (bool, Option<String>) = env
            .eval(r#"return C_AddOns.LoadAddOn("Blizzard_ArtifactUI")"#)
            .expect("C_AddOns.LoadAddOn probe should run cleanly");
        assert!(
            loaded,
            "`{ROOT}` must load after an explicit `C_AddOns.LoadAddOn`; error={error:?}"
        );

        let loaded_after: bool = env
            .eval(r#"return C_AddOns.IsAddOnLoaded("Blizzard_ArtifactUI")"#)
            .expect("post-load IsAddOnLoaded probe should run cleanly");
        assert!(
            loaded_after,
            "`{ROOT}` must report loaded after explicit `C_AddOns.LoadAddOn`"
        );
    });
}
}

#[cfg(feature = "optional-tests")]
#[test]
fn artifact_item_quality_color_surface_is_available_after_load() {
    with_blizzard_addon_smoke_shape(&[], &[], |env, _loaded| {
        let (loaded, error): (bool, Option<String>) = env
            .eval(r#"return C_AddOns.LoadAddOn("Blizzard_ArtifactUI")"#)
            .expect("C_AddOns.LoadAddOn probe should run cleanly");
        assert!(
            loaded,
            "`{ROOT}` must load before probing artifact quality colors; error={error:?}"
        );

        let failures: String = env
            .eval(
                r#"
                local failures = {}

                local function expect(condition, message)
                    if not condition then
                        table.insert(failures, message)
                    end
                end

                local artifactQuality = Enum.ItemQuality.Artifact
                expect(type(artifactQuality) == "number", "Enum.ItemQuality.Artifact must be numeric")

                local colorData = ColorManager.GetColorDataForItemQuality(artifactQuality)
                expect(type(colorData) == "table", "artifact item quality color data must be a table")

                if type(colorData) == "table" then
                    expect(type(colorData.r) == "number", "artifact colorData.r must be numeric")
                    expect(type(colorData.g) == "number", "artifact colorData.g must be numeric")
                    expect(type(colorData.b) == "number", "artifact colorData.b must be numeric")
                    expect(type(colorData.GetRGB) == "function", "artifact colorData must expose GetRGB")

                    if type(colorData.GetRGB) == "function" then
                        local r, g, b = colorData:GetRGB()
                        expect(r == colorData.r, "GetRGB red channel should match colorData.r")
                        expect(g == colorData.g, "GetRGB green channel should match colorData.g")
                        expect(b == colorData.b, "GetRGB blue channel should match colorData.b")
                    end
                end

                return table.concat(failures, "\n")
                "#,
            )
            .expect("artifact quality color probe should run cleanly");
        assert!(
            failures.is_empty(),
            "Artifact item-quality color surface mismatches:\n{failures}"
        );
    });
}

#[cfg(feature = "optional-tests")]
#[test]
fn artifact_appearance_tutorial_bitfield_constant_is_numeric() {
    with_blizzard_addon_smoke_shape(&[], &[], |env, _loaded| {
        let constant_type: String = env
            .eval(r#"return type(LE_FRAME_TUTORIAL_ARTIFACT_APPEARANCE_TAB)"#)
            .expect("tutorial bitfield constant probe should run cleanly");
        assert_eq!(
            constant_type, "number",
            "`LE_FRAME_TUTORIAL_ARTIFACT_APPEARANCE_TAB` must be numeric for ArtifactUI \
             GetCVarBitfield/SetCVarBitfield gates"
        );
    });
}

post_wrath_ui_fixture! {
#[cfg(feature = "optional-tests")]
#[test]
fn artifact_ui_loads_cleanly_via_c_addons_load_addon() {
    with_blizzard_addon_smoke_shape(&[], &[], |env, _loaded| {
        clear_recorded_lua_errors(env);

        let (loaded, error): (bool, Option<String>) = env
            .eval(r#"return C_AddOns.LoadAddOn("Blizzard_ArtifactUI")"#)
            .expect("C_AddOns.LoadAddOn probe should run cleanly");
        assert!(
            loaded,
            "`{ROOT}` must load via `C_AddOns.LoadAddOn`; error={error:?}"
        );

        ensure_player_frame_stub(env);
        env.apply_post_load_workarounds();
        settle_headless_startup(env);

        let errors = recorded_lua_errors(env);
        assert!(
            errors.is_empty(),
            "`{ROOT}` must load and settle with zero recorded Lua errors:\n{}",
            errors.join("\n")
        );
    });
}
}

fn ensure_player_frame_stub(env: &wow_ui_sim::lua_api::WowLuaEnv) {
    env.exec(
        r#"
        if not PlayerFrame then
            PlayerFrame = CreateFrame("Frame", "PlayerFrame", UIParent)
        end
        PlayerFrame.unit = "player"
        "#,
    )
    .expect("failed to create PlayerFrame stub before settling startup events");
}

fn artifact_ui_toc() -> std::path::PathBuf {
    blizzard_ui_dir().join(ROOT).join("Blizzard_ArtifactUI.toc")
}
