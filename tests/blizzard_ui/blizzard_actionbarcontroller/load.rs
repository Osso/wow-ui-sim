//! Load smoke for `Blizzard_ActionBarController`.
//!
//! TOC reference (`Interface/BlizzardUI/Blizzard_ActionBarController/
//! Blizzard_ActionBarController.toc`):
//!
//! ```text
//! ## Title: Blizzard_ActionBarController
//! ## DefaultState: enabled
//! ## Dependencies: Blizzard_ActionBar, Blizzard_OverrideActionBar
//! ## AllowLoad: Game
//! ```

use crate::common::blizzard_addon_harness::with_blizzard_addon_smoke_shape;
use crate::common::panel_fixtures::recorded_lua_errors;

const ROOT: &str = "Blizzard_ActionBarController";
const REQUIRED_DEPS: &[&str] = &["Blizzard_ActionBar", "Blizzard_OverrideActionBar"];

#[cfg(feature = "optional-tests")]
#[test]
fn action_bar_controller_loads_with_dependency_closure_and_no_lua_errors() {
    with_blizzard_addon_smoke_shape(&[ROOT], &[], |env, loaded| {
        assert_loaded(loaded, ROOT);
        for dep in REQUIRED_DEPS {
            assert_loaded(loaded, dep);
        }

        let errors = recorded_lua_errors(env);
        assert!(
            errors.is_empty(),
            "Blizzard_ActionBarController dependency-closure load must emit zero recorded Lua \
             errors after the smoke-shape harness clears the panel baseline. Got:\n  {}",
            errors.join("\n  ")
        );
    });
}

fn assert_loaded(loaded: &[String], addon: &str) {
    assert!(
        loaded.iter().any(|name| name == addon),
        "`{addon}` must appear in the loaded addon set for the closure rooted at \
         `Blizzard_ActionBarController`. Loaded set: {loaded:?}"
    );
}
