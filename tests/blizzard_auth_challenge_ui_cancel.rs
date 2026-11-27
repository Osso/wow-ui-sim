use crate::common;

use common::blizzard_addon_harness::with_blizzard_addon_smoke_shape;
use common::panel_fixtures::{clear_recorded_lua_errors, recorded_lua_errors};

const ROOT: &str = "Blizzard_AuthChallengeUI";
const CANCEL_PROBE_LUA: &str = r##"
-- INFERRED (simulator mapping: secure namespace distinct per R88g.2, applied by
-- e93399f5e3). Same probe route as blizzard_auth_challenge_ui_secure_env.rs.
local authNamespace = __secureenv.C_AuthChallenge
local callCount = 0
local argCounts = {}
local originalCancel = authNamespace.Cancel

authNamespace.Cancel = function(...)
  callCount = callCount + 1
  table.insert(argCounts, select("#", ...))
end

local function okayButtonFor(frame)
  for _, child in ipairs({ frame:GetChildren() }) do
    if child:GetObjectType() == "Button" and child:GetText() == BLIZZARD_CHALLENGE_OKAY then
      return child
    end
  end
end

local deniedOkay = okayButtonFor(AuthChallengeFrame.DeniedFrame)
local errorOkay = okayButtonFor(AuthChallengeFrame.ErrorFrame)
local deniedOnClick = deniedOkay and deniedOkay:GetScript("OnClick")
local errorOnClick = errorOkay and errorOkay:GetScript("OnClick")

AuthChallengeUI_Cancel()
deniedOnClick(deniedOkay)
errorOnClick(errorOkay)
C_AuthChallenge.Cancel = originalCancel

return callCount,
       argCounts[1],
       argCounts[2],
       argCounts[3],
       deniedOkay ~= nil,
       errorOkay ~= nil,
       deniedOnClick == AuthChallengeUI_Cancel,
       errorOnClick == AuthChallengeUI_Cancel
"##;

post_wrath_ui_fixture! {
#[test]
fn auth_challenge_ui_cancel_calls_auth_challenge_cancel_without_args() {
    common::with_perf_lock(|| {
        common::with_timeout(240, || {
            with_blizzard_addon_smoke_shape(&[], &[], |env, _loaded| {
                clear_recorded_lua_errors(env);

                let (loaded, reason): (bool, Option<String>) = env
                    .eval(r#"return C_AddOns.LoadAddOn("Blizzard_AuthChallengeUI")"#)
                    .expect("C_AddOns.LoadAddOn should return");
                assert!(loaded, "`{ROOT}` should load: {reason:?}");

                let result: (i32, i32, i32, i32, bool, bool, bool, bool) = env
                    .eval(CANCEL_PROBE_LUA)
                    .expect("AuthChallengeUI_Cancel probe should run");
                let (
                    call_count,
                    direct_arg_count,
                    denied_arg_count,
                    error_arg_count,
                    has_denied_okay,
                    has_error_okay,
                    denied_handler_matches,
                    error_handler_matches,
                ) = result;

                assert!(has_denied_okay, "DeniedFrame should expose an Okay button");
                assert!(has_error_okay, "ErrorFrame should expose an Okay button");
                assert!(
                    denied_handler_matches,
                    "DeniedFrame Okay button should use `AuthChallengeUI_Cancel`"
                );
                assert!(
                    error_handler_matches,
                    "ErrorFrame Okay button should use `AuthChallengeUI_Cancel`"
                );
                assert_eq!(
                    call_count, 3,
                    "`AuthChallengeUI_Cancel` and both Okay buttons should call Cancel once each"
                );
                assert_eq!(
                    direct_arg_count, 0,
                    "`AuthChallengeUI_Cancel` should pass no args to `Cancel`"
                );
                assert_eq!(
                    denied_arg_count, 0,
                    "DeniedFrame Okay button should pass no args to `Cancel`"
                );
                assert_eq!(
                    error_arg_count, 0,
                    "ErrorFrame Okay button should pass no args to `Cancel`"
                );

                let errors = recorded_lua_errors(env);
                assert!(
                    errors.is_empty(),
                    "`{ROOT}` Cancel probe emitted Lua errors:\n{}",
                    errors.join("\n")
                );
            });
        });
    });
}
}
