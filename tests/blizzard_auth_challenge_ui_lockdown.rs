use crate::common;

use common::blizzard_addon_harness::with_blizzard_addon_smoke_shape;
use common::panel_fixtures::{clear_recorded_lua_errors, recorded_lua_errors};

const ROOT: &str = "Blizzard_AuthChallengeUI";
const LOCKDOWN_PROBE_LUA: &str = r#"
local child = CreateFrame("Frame", nil, AuthChallengeFrame)
return AuthChallengeFrame:IsForbidden(), child:IsForbidden()
"#;

post_wrath_ui_fixture! {
#[test]
fn auth_challenge_ui_full_lockdown_is_reported_and_inherited() {
    common::with_perf_lock(|| {
        common::with_timeout(240, || {
            with_blizzard_addon_smoke_shape(&[], &[], |env, _loaded| {
                clear_recorded_lua_errors(env);

                let (loaded, reason): (bool, Option<String>) = env
                    .eval(r#"return C_AddOns.LoadAddOn("Blizzard_AuthChallengeUI")"#)
                    .expect("C_AddOns.LoadAddOn should return");
                assert!(loaded, "`{ROOT}` should load: {reason:?}");

                let (parent_forbidden, child_forbidden): (bool, bool) = env
                    .eval(LOCKDOWN_PROBE_LUA)
                    .expect("AuthChallengeUI lockdown probe should run");

                assert!(
                    parent_forbidden && child_forbidden,
                    "fullLockdown must be reported and inherited by children"
                );

                let errors = recorded_lua_errors(env);
                assert!(
                    errors.is_empty(),
                    "`{ROOT}` lockdown probe emitted Lua errors:\n{}",
                    errors.join("\n")
                );
            });
        });
    });
}
}
