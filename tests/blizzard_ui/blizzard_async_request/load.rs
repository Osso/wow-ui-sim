//! Load smoke for `Blizzard_AsyncRequest`.

use crate::common::blizzard_addon_harness::{
    with_blizzard_addon_glue_smoke_shape, with_blizzard_addon_smoke_shape,
};
use crate::common::panel_fixtures::{
    blizzard_ui_dir, clear_recorded_lua_errors, recorded_lua_errors,
};
use wow_ui_sim::loader::load_addon;
use wow_ui_sim::startup::settle_headless_startup;
use wow_ui_sim::toc::TocFile;

const ROOT: &str = "Blizzard_AsyncRequest";

post_wrath_ui_fixture! {
#[cfg(feature = "optional-tests")]
#[test]
fn async_request_loads_cleanly_with_no_recorded_lua_errors() {
    with_blizzard_addon_smoke_shape(&[], &[], |env, _loaded| {
        clear_recorded_lua_errors(env);

        load_addon(&env.loader_env(), &async_request_toc())
            .expect("Blizzard_AsyncRequest should load directly from its TOC");

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

#[cfg(feature = "optional-tests")]
#[test]
fn async_request_toc_declares_no_dependencies_and_loads_as_leaf() {
    let toc =
        TocFile::from_file(&async_request_toc()).expect("Blizzard_AsyncRequest TOC should parse");
    assert!(
        toc.dependencies().is_empty(),
        "`{ROOT}` must not declare required dependencies"
    );
    assert!(
        toc.optional_deps().is_empty(),
        "`{ROOT}` must not declare optional dependencies"
    );

    with_blizzard_addon_smoke_shape(&[ROOT], &[], |_env, loaded| {
        assert_eq!(
            loaded,
            [ROOT],
            "`{ROOT}` must load as a leaf addon in the smoke harness; shared XML panel setup is \
             provided before the dependency-closure load. Loaded closure: {loaded:?}"
        );
    });
}

#[cfg(feature = "optional-tests")]
#[test]
fn async_request_allow_load_both_works_in_game_and_glue_contexts() {
    let toc =
        TocFile::from_file(&async_request_toc()).expect("Blizzard_AsyncRequest TOC should parse");
    assert_eq!(
        toc.metadata.get("AllowLoad").map(String::as_str),
        Some("Both"),
        "`{ROOT}` must remain loadable in both game and glue contexts"
    );

    with_blizzard_addon_smoke_shape(&[ROOT], &[], |_env, loaded| {
        assert_eq!(
            loaded,
            [ROOT],
            "`{ROOT}` must be accepted by the game-screen closure walker"
        );
    });

    with_blizzard_addon_glue_smoke_shape(&[ROOT], &[], |_env, loaded| {
        assert_eq!(
            loaded,
            [ROOT],
            "`{ROOT}` must be accepted by the glue-screen closure walker"
        );
    });
}

fn async_request_toc() -> std::path::PathBuf {
    blizzard_ui_dir()
        .join(ROOT)
        .join("Blizzard_AsyncRequest.toc")
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
