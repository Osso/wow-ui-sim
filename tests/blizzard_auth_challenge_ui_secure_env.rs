use crate::common;

use common::blizzard_addon_harness::with_blizzard_addon_smoke_shape;
use common::panel_fixtures::{clear_recorded_lua_errors, recorded_lua_errors};

const ROOT: &str = "Blizzard_AuthChallengeUI";
const SECURE_ENV_PROBE_LUA: &str = r#"
local secureSubmit = _G.AuthChallengeUI_Submit
local secureCallCount = 0
local privateCallCount = 0
local originalSubmit = C_AuthChallenge.Submit

C_AuthChallenge.Submit = function()
  secureCallCount = secureCallCount + 1
end

local privateEnv = {
  privateOnlySentinel = "must-not-be-visible",
  C_AuthChallenge = {
    Submit = function()
      privateCallCount = privateCallCount + 1
    end,
  },
}
setmetatable(privateEnv, { __index = _G })

local caller = function()
  local sameClosure = AuthChallengeUI_Submit == secureSubmit
  local secureFenv = getfenv(AuthChallengeUI_Submit) == __secureenv
  local callOk = pcall(AuthChallengeUI_Submit)
  return sameClosure, secureFenv, callOk
end
setfenv(caller, privateEnv)

local sameClosure, secureFenv, callOk = caller()
C_AuthChallenge.Submit = originalSubmit

return sameClosure,
       secureFenv,
       callOk,
       secureCallCount,
       privateCallCount
"#;

timerunning_util_panel_fixture! {
post_wrath_ui_fixture! {
#[test]
fn auth_challenge_ui_submit_stays_in_secure_environment() {
    common::with_perf_lock(|| {
        common::with_timeout(240, || {
            with_blizzard_addon_smoke_shape(&[], &[], |env, _loaded| {
                clear_recorded_lua_errors(env);

                let (loaded, reason): (bool, Option<String>) = env
                    .eval(r#"return C_AddOns.LoadAddOn("Blizzard_AuthChallengeUI")"#)
                    .expect("C_AddOns.LoadAddOn should return");
                assert!(loaded, "`{ROOT}` should load: {reason:?}");

                let (same_closure, secure_fenv, call_ok, secure_calls, private_calls): (
                    bool,
                    bool,
                    bool,
                    i32,
                    i32,
                ) = env
                    .eval(SECURE_ENV_PROBE_LUA)
                    .expect("AuthChallengeUI secure-env probe should run");

                assert!(
                    same_closure,
                    "`AuthChallengeUI_Submit` should resolve to the same exported closure from a private env"
                );
                assert!(
                    secure_fenv,
                    "`AuthChallengeUI_Submit` should keep the secureenv function environment"
                );
                assert!(
                    call_ok,
                    "`AuthChallengeUI_Submit` should run from a private env"
                );
                assert_eq!(
                    secure_calls, 1,
                    "`AuthChallengeUI_Submit` should use the secure/global `C_AuthChallenge` table"
                );
                assert_eq!(
                    private_calls, 0,
                    "`AuthChallengeUI_Submit` should not see a caller-private `C_AuthChallenge`"
                );

                let errors = recorded_lua_errors(env);
                assert!(
                    errors.is_empty(),
                    "`{ROOT}` secure-env probe emitted Lua errors:\n{}",
                    errors.join("\n")
                );
            });
        });
    });
}
}
}
