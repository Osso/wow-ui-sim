use crate::common;

use common::blizzard_addon_harness::with_blizzard_addon_smoke_shape;
use common::panel_fixtures::{clear_recorded_lua_errors, recorded_lua_errors};

const ROOT: &str = "Blizzard_AuthChallengeUI";
const SUBMIT_PROBE_LUA: &str = r##"
-- INFERRED (simulator mapping: secure namespace distinct per R88g.2, applied by
-- e93399f5e3). Same probe route as blizzard_auth_challenge_ui_secure_env.rs.
local authNamespace = __secureenv.C_AuthChallenge
local callCount = 0
local argCount = nil
local originalSubmit = authNamespace.Submit

authNamespace.Submit = function(...)
  callCount = callCount + 1
  argCount = select("#", ...)
end

AuthChallengeUI_Submit()
C_AuthChallenge.Submit = originalSubmit

return callCount, argCount
"##;

post_wrath_ui_fixture! {
#[test]
fn auth_challenge_ui_submit_calls_auth_challenge_submit_without_args() {
    common::with_perf_lock(|| {
        common::with_timeout(240, || {
            with_blizzard_addon_smoke_shape(&[], &[], |env, _loaded| {
                clear_recorded_lua_errors(env);

                let (loaded, reason): (bool, Option<String>) = env
                    .eval(r#"return C_AddOns.LoadAddOn("Blizzard_AuthChallengeUI")"#)
                    .expect("C_AddOns.LoadAddOn should return");
                assert!(loaded, "`{ROOT}` should load: {reason:?}");

                let (call_count, arg_count): (i32, i32) = env
                    .eval(SUBMIT_PROBE_LUA)
                    .expect("AuthChallengeUI_Submit probe should run");
                assert_eq!(
                    call_count, 1,
                    "`AuthChallengeUI_Submit` should call `C_AuthChallenge.Submit` once"
                );
                assert_eq!(
                    arg_count, 0,
                    "`AuthChallengeUI_Submit` should pass no args to `Submit`"
                );

                let errors = recorded_lua_errors(env);
                assert!(
                    errors.is_empty(),
                    "`{ROOT}` Submit probe emitted Lua errors:\n{}",
                    errors.join("\n")
                );
            });
        });
    });
}
}
