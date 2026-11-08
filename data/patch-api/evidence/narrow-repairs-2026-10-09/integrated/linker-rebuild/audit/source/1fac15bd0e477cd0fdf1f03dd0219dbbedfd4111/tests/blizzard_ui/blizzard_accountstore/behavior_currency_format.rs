//! Behavior pin for `AccountStoreUtil.FormatCurrencyDisplay`.
//!
//! Spec/source mismatch finding (PLAN.md task for
//! `AccountStoreUtil.FormatCurrencyDisplayWithIcon(currencyID, amount)`):
//! the plan describes a function named `FormatCurrencyDisplayWithIcon`
//! taking `(currencyID, amount)` and returning `"<amount> |T<icon>:0|t"`.
//! The actual seven-line function at
//! `Blizzard_AccountStoreUtil.lua:56-63` differs in four ways:
//!
//! ```lua
//! function AccountStoreUtil.FormatCurrencyDisplay(currencyAmount, accountStoreCurrencyID)
//!     local currencyInfo = C_AccountStore.GetCurrencyInfo(accountStoreCurrencyID);
//!     if not currencyInfo.icon then
//!         return "";
//!     end
//!
//!     return BreakUpLargeNumbers(currencyAmount) .. " " .. CreateSimpleTextureMarkup(currencyInfo.icon, 12, 12);
//! end
//! ```
//!
//! 1. **Function name mismatch.** `AccountStoreUtil.FormatCurrencyDisplayWithIcon`
//!    does NOT exist anywhere in `Blizzard_AccountStoreUtil.lua`. The
//!    actual exported function is `AccountStoreUtil.FormatCurrencyDisplay`
//!    (no `WithIcon` suffix). The lane has a sibling
//!    `FormatCurrencyDisplayWithWarning` at lines 74-106, but the
//!    PLAN-shaped `WithIcon` variant is absent — callers under
//!    `Blizzard_AccountStoreCardTemplates.lua:178` and
//!    `Blizzard_AccountStoreUtil.lua:140` invoke `FormatCurrencyDisplay`
//!    directly.
//!
//! 2. **Parameter order mismatch.** The PLAN signature
//!    `(currencyID, amount)` is REVERSED from the actual
//!    `(currencyAmount, accountStoreCurrencyID)`. A caller passing
//!    arguments in the PLAN order would (a) feed the currency-id integer
//!    into `BreakUpLargeNumbers` (which would format the id like a
//!    money amount) and (b) feed the money amount as a currency-id
//!    lookup into `C_AccountStore.GetCurrencyInfo` — almost certainly
//!    returning nil and triggering the `not currencyInfo.icon` branch
//!    (returning the empty string). Real callers in the lane use
//!    `(itemInfo.price, itemInfo.currencyID)` (price first, id second),
//!    matching the actual declaration.
//!
//! 3. **Output format mismatch.** The PLAN shape `"<amount> |T<icon>:0|t"`
//!    has only ONE numeric field after the icon; the actual output uses
//!    `CreateSimpleTextureMarkup(currencyInfo.icon, 12, 12)` from
//!    `Blizzard_SharedXMLBase/TextureUtil.lua:247-255`, which produces
//!    `"|T%s:%d:%d:%d:%d|t"` with FIVE numeric fields: `height||width`,
//!    `width`, `xOffset||0`, `yOffset||0`. With width=12 and height=12
//!    (no offsets supplied), the actual markup is
//!    `|T<icon>:12:12:0:0|t` — four numbers, not one zero. Additionally
//!    the amount is passed through `BreakUpLargeNumbers` (a thousands-
//!    separator helper modeled locally in
//!    `src/lua_api/globals/real/break_up_large_numbers.rs` using locale-aware
//!    Decimal formatting). With the fixture locale asserted as `enUS`, the
//!    local model returns `"1,234 |T999:12:12:0:0|t"` for
//!    `FormatCurrencyDisplay(1234, 5)` with `icon = 999`. This pins local
//!    modeled behavior, not a native-client capture.
//!
//! 4. **PLAN-omitted nil-icon fallback.** When
//!    `C_AccountStore.GetCurrencyInfo(currencyID).icon` is nil
//!    (currency record exists but no icon field was populated), the
//!    body short-circuits at line 58-60 and returns the empty string,
//!    NOT a partial format like `"<amount> "` or a localized
//!    placeholder. The PLAN spec ignores this branch entirely; a
//!    regression that started returning a partial string would NOT be
//!    caught by tests that only assert the happy-path concatenation.
//!
//! Four tests pin the contract:
//!
//! - `format_currency_display_with_icon_plan_named_function_does_not_exist`
//!   asserts `AccountStoreUtil.FormatCurrencyDisplayWithIcon` is nil
//!   (the structural-absence tripwire that flips if Blizzard adds the
//!   PLAN-shaped variant alongside the actual function).
//! - `format_currency_display_passes_second_positional_arg_as_currency_id_to_get_currency_info`
//!   replaces `C_AccountStore.GetCurrencyInfo` with a tracker that
//!   captures its first positional arg and returns a stub table; calls
//!   `FormatCurrencyDisplay(SENTINEL_AMOUNT, SENTINEL_CURRENCY_ID)`;
//!   asserts the tracker received exactly one call with
//!   `SENTINEL_CURRENCY_ID` (proving the SECOND positional arg is the
//!   currency id, NOT the first as PLAN claims).
//! - `format_currency_display_returns_amount_space_texture_markup_with_size_twelve_twelve_zero_zero`
//!   stubs GetCurrencyInfo to return `{icon = 9_999_777}`; calls
//!   `FormatCurrencyDisplay(1234, ANY_CURRENCY_ID)`; asserts the result
//!   is exactly `"1,234 |T9999777:12:12:0:0|t"` in the asserted `enUS`
//!   fixture (proving width=12, height=12, xOffset=0, yOffset=0, NOT
//!   the PLAN-shaped single-`:0` shape).
//! - `format_currency_display_returns_empty_string_when_currency_info_icon_is_nil`
//!   stubs GetCurrencyInfo to return `{icon = nil}`; calls
//!   `FormatCurrencyDisplay(SOME_AMOUNT, ANY_CURRENCY_ID)`; asserts the
//!   result is exactly `""` (the PLAN-omitted short-circuit branch).

use crate::common::blizzard_addon_harness::with_blizzard_addon_smoke_shape;
use wow_ui_sim::lua_api::WowLuaEnv;

const ROOT: &str = "Blizzard_AccountStore";

#[test]
fn format_currency_display_with_icon_plan_named_function_does_not_exist() {
    with_blizzard_addon_smoke_shape(&[ROOT], &[], |env, _loaded| {
        let plan_named_type: String = env
            .eval("return type(AccountStoreUtil.FormatCurrencyDisplayWithIcon)")
            .expect(
                "type probe for AccountStoreUtil.FormatCurrencyDisplayWithIcon must run cleanly",
            );

        assert_eq!(
            plan_named_type, "nil",
            "Expected `AccountStoreUtil.FormatCurrencyDisplayWithIcon` to be NIL — the PLAN-named \
             function does not exist anywhere in `Blizzard_AccountStoreUtil.lua`. The actual \
             exported function is `AccountStoreUtil.FormatCurrencyDisplay` (no `WithIcon` suffix) \
             at lines 56-63; the lane also has `FormatCurrencyDisplayWithWarning` at lines 74-106 \
             but no `WithIcon` variant. A non-nil reading would mean Blizzard added the \
             PLAN-shaped function (forcing a re-pin to the new contract)."
        );

        let actual_type: String = env
            .eval("return type(AccountStoreUtil.FormatCurrencyDisplay)")
            .expect("type probe for AccountStoreUtil.FormatCurrencyDisplay must run cleanly");

        assert_eq!(
            actual_type, "function",
            "Expected `AccountStoreUtil.FormatCurrencyDisplay` to be a function — the actual \
             entry point at `Blizzard_AccountStoreUtil.lua:56`. A non-function reading would \
             indicate the function was renamed or removed, breaking real callers in \
             `Blizzard_AccountStoreCardTemplates.lua:178` and \
             `Blizzard_AccountStoreUtil.lua:140`."
        );
    });
}

#[test]
fn format_currency_display_passes_second_positional_arg_as_currency_id_to_get_currency_info() {
    with_blizzard_addon_smoke_shape(&[ROOT], &[], |env, _loaded| {
        const SENTINEL_AMOUNT: i64 = 1_234_001;
        const SENTINEL_CURRENCY_ID: i64 = 5_678_002;

        seed_get_currency_info_arg_tracker(env);

        env.eval::<()>(&format!(
            r#"
            _G.__behavior_currency_format_result = AccountStoreUtil.FormatCurrencyDisplay(
                {SENTINEL_AMOUNT},
                {SENTINEL_CURRENCY_ID}
            )
            return
            "#
        ))
        .expect(
            "FormatCurrencyDisplay(amount, currencyID) call must run cleanly under the stubbed \
             GetCurrencyInfo tracker — the body reads currencyInfo.icon from the tracker's \
             returned stub and concatenates the icon markup",
        );

        let (call_count, captured_arg): (i64, i64) = env
            .eval(
                r#"
                return _G.__behavior_currency_format_get_currency_info_call_count,
                       _G.__behavior_currency_format_get_currency_info_arg or -1
                "#,
            )
            .expect("post-call GetCurrencyInfo tracker probe must run cleanly");

        assert_eq!(
            call_count, 1,
            "Expected `C_AccountStore.GetCurrencyInfo` to have been invoked exactly once after \
             `FormatCurrencyDisplay(amount, currencyID)`. The body at line 57 calls \
             `C_AccountStore.GetCurrencyInfo(accountStoreCurrencyID)` unconditionally as the \
             first statement. A zero count means the body errored before reaching the call; a \
             count > 1 means the body now calls GetCurrencyInfo multiple times (worth \
             investigating because the actual implementation caches the result in the local \
             `currencyInfo` and reads `.icon` only once)."
        );

        assert_eq!(
            captured_arg, SENTINEL_CURRENCY_ID,
            "Expected the SECOND positional arg ({SENTINEL_CURRENCY_ID}) to have been forwarded \
             to GetCurrencyInfo, NOT the first ({SENTINEL_AMOUNT}). The body at line 56 declares \
             `function AccountStoreUtil.FormatCurrencyDisplay(currencyAmount, accountStoreCurrencyID)` \
             — currency id is the SECOND parameter. The PLAN-shaped `(currencyID, amount)` order \
             is REVERSED. A captured arg of {SENTINEL_AMOUNT} would prove the parameter order \
             matches PLAN (forcing a re-pin against the new signature and likely breaking real \
             callers `(itemInfo.price, itemInfo.currencyID)` in the lane)."
        );

        teardown_get_currency_info_arg_tracker(env);
    });
}

#[test]
fn format_currency_display_returns_amount_space_texture_markup_with_size_twelve_twelve_zero_zero() {
    with_blizzard_addon_smoke_shape(&[ROOT], &[], |env, _loaded| {
        const AMOUNT: i64 = 1234;
        const CURRENCY_ID: i64 = 7;
        const ICON_FILE_ID: i64 = 9_999_777;

        let locale: String = env
            .eval("return GetLocale()")
            .expect("fixture locale probe");
        assert_eq!(locale, "enUS", "currency format fixture requires enUS");

        seed_get_currency_info_with_icon_stub(env, ICON_FILE_ID);

        let result: String = env
            .eval(&format!(
                r#"
                return AccountStoreUtil.FormatCurrencyDisplay({AMOUNT}, {CURRENCY_ID})
                "#
            ))
            .expect(
                "FormatCurrencyDisplay must concatenate the locally formatted amount and stub icon",
            );

        let expected = format!("1,234 |T{ICON_FILE_ID}:12:12:0:0|t");

        assert_eq!(
            result, expected,
            "Expected the literal output `{expected}` for amount={AMOUNT}, icon={ICON_FILE_ID}. \
             The body at line 62 returns \
             `BreakUpLargeNumbers(currencyAmount) .. \" \" .. CreateSimpleTextureMarkup(\
             currencyInfo.icon, 12, 12)`. The local `BreakUpLargeNumbers` model in \
             `src/lua_api/globals/real/break_up_large_numbers.rs` uses locale-aware Decimal \
             formatting and produces \"1,234\" for the asserted enUS fixture; this is not \
             native-client capture evidence. `CreateSimpleTextureMarkup` from \
             `Blizzard_SharedXMLBase/TextureUtil.lua:247-255` formats \
             `\"|T%s:%d:%d:%d:%d|t\"` with the args (file, height||width, width, xOffset||0, \
             yOffset||0) — an icon id plus four size/offset fields, NOT the PLAN-shaped \
             single-`:0` shape. Different output indicates changed markup or local \
             locale-aware amount formatting."
        );

        teardown_get_currency_info_arg_tracker(env);
    });
}

#[test]
fn format_currency_display_returns_empty_string_when_currency_info_icon_is_nil() {
    with_blizzard_addon_smoke_shape(&[ROOT], &[], |env, _loaded| {
        const SOME_AMOUNT: i64 = 5_555;
        const ANY_CURRENCY_ID: i64 = 8;

        seed_get_currency_info_with_nil_icon_stub(env);

        let result: String = env
            .eval(&format!(
                r#"
                return AccountStoreUtil.FormatCurrencyDisplay({SOME_AMOUNT}, {ANY_CURRENCY_ID})
                "#
            ))
            .expect("FormatCurrencyDisplay must return cleanly when stubbed icon is nil");

        assert_eq!(
            result, "",
            "Expected the empty string `\"\"` when `currencyInfo.icon` is nil — the body at \
             lines 58-60 short-circuits with `if not currencyInfo.icon then return \"\" end` \
             BEFORE reaching the concatenation at line 62. A non-empty reading proves the \
             short-circuit branch was removed (forcing the concatenation to attempt \
             `CreateSimpleTextureMarkup(nil, 12, 12)` which would format \"|Tnil:12:12:0:0|t\" \
             via Lua's tostring coercion of nil to \"nil\") — a regression the PLAN spec ignores."
        );

        teardown_get_currency_info_arg_tracker(env);
    });
}

fn seed_get_currency_info_arg_tracker(env: &WowLuaEnv) {
    env.eval::<()>(
        r#"
        _G.__behavior_currency_format_get_currency_info_call_count = 0
        _G.__behavior_currency_format_get_currency_info_arg = nil
        _G.__behavior_currency_format_original_get_currency_info = C_AccountStore.GetCurrencyInfo
        C_AccountStore.GetCurrencyInfo = function(currency_id)
            _G.__behavior_currency_format_get_currency_info_call_count =
                _G.__behavior_currency_format_get_currency_info_call_count + 1
            _G.__behavior_currency_format_get_currency_info_arg = currency_id
            return { icon = 0 }
        end
        return
        "#,
    )
    .expect("seeding C_AccountStore.GetCurrencyInfo arg tracker must run cleanly");
}

fn seed_get_currency_info_with_icon_stub(env: &WowLuaEnv, icon_file_id: i64) {
    env.eval::<()>(&format!(
        r#"
        _G.__behavior_currency_format_original_get_currency_info = C_AccountStore.GetCurrencyInfo
        C_AccountStore.GetCurrencyInfo = function(_currency_id)
            return {{ icon = {icon_file_id} }}
        end
        return
        "#
    ))
    .expect("seeding C_AccountStore.GetCurrencyInfo stub returning explicit icon must run cleanly");
}

fn seed_get_currency_info_with_nil_icon_stub(env: &WowLuaEnv) {
    env.eval::<()>(
        r#"
        _G.__behavior_currency_format_original_get_currency_info = C_AccountStore.GetCurrencyInfo
        C_AccountStore.GetCurrencyInfo = function(_currency_id)
            return { icon = nil }
        end
        return
        "#,
    )
    .expect("seeding C_AccountStore.GetCurrencyInfo stub returning nil icon must run cleanly");
}

fn teardown_get_currency_info_arg_tracker(env: &WowLuaEnv) {
    env.eval::<()>(
        r#"
        C_AccountStore.GetCurrencyInfo = _G.__behavior_currency_format_original_get_currency_info
        _G.__behavior_currency_format_original_get_currency_info = nil
        _G.__behavior_currency_format_get_currency_info_call_count = nil
        _G.__behavior_currency_format_get_currency_info_arg = nil
        _G.__behavior_currency_format_result = nil
        return
        "#,
    )
    .expect("GetCurrencyInfo tracker tear-down must run cleanly");
}
