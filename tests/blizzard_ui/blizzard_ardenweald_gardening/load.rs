//! Load smoke for `Blizzard_ArdenwealdGardening`.

use crate::common::blizzard_addon_harness::with_blizzard_addon_smoke_shape;
use crate::common::panel_fixtures::blizzard_ui_dir;
use wow_ui_sim::startup::settle_headless_startup;

const ROOT: &str = "Blizzard_ArdenwealdGardening";
const NATURAL_CALLER: &str = "Blizzard_GarrisonUI";

#[cfg(feature = "optional-tests")]
#[test]
fn ardenweald_gardening_loads_cleanly_with_no_recorded_lua_errors() {
    with_blizzard_addon_smoke_shape(&[ROOT], &[], |env, loaded| {
        assert!(
            loaded.iter().any(|name| name == ROOT),
            "`{ROOT}` must be loaded as the requested LoadOnDemand root"
        );

        ensure_player_frame_stub(env);
        env.apply_post_load_workarounds();
        settle_headless_startup(env);

        let errors = env.state().borrow().lua_errors.clone();
        assert!(
            errors.is_empty(),
            "`{ROOT}` must load and settle with zero recorded Lua errors:\n{}",
            errors.join("\n")
        );
    });
}

#[cfg(feature = "optional-tests")]
#[test]
fn ardenweald_gardening_caller_shape_loads_garrison_ui() {
    assert_ardenweald_toc_has_no_dependencies();

    with_blizzard_addon_smoke_shape(&[NATURAL_CALLER, ROOT], &[], |_env, loaded| {
        assert!(
            loaded.iter().any(|name| name == NATURAL_CALLER),
            "`{NATURAL_CALLER}` must be present when the harness models the natural \
             `UIParentLoadAddOn(\"{ROOT}\")` caller path. Loaded set: {loaded:?}"
        );
        assert!(
            loaded.iter().any(|name| name == ROOT),
            "`{ROOT}` must still be loaded as the on-demand addon requested by the \
             Garrison landing page. Loaded set: {loaded:?}"
        );
    });
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

fn assert_ardenweald_toc_has_no_dependencies() {
    let toc_path = blizzard_ui_dir()
        .join(ROOT)
        .join("Blizzard_ArdenwealdGardening.toc");
    let toc = std::fs::read_to_string(&toc_path)
        .unwrap_or_else(|error| panic!("failed to read `{}`: {error}", toc_path.display()));

    for dependency_key in ["Dependencies", "RequiredDep", "RequiredDeps"] {
        let prefix = format!("## {dependency_key}:");
        assert!(
            !toc.lines()
                .any(|line| line.trim_start().starts_with(&prefix)),
            "`{ROOT}` TOC must not grow an explicit `{dependency_key}` edge; \
             `{NATURAL_CALLER}` is a caller-path root, not an addon dependency"
        );
    }
}
