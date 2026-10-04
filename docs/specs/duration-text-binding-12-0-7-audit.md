# Retail 12.0.7 duration text bindings — B08/B09/B10/B29/B30

The retained [12.0.7 source](../../data/patch-api/sources/12.0.7-api-changes.txt) adds the duration-text-binding factory, ScriptObject and eighteen binding methods. The existing provider in `src/c_api/duration_text_binding.rs` remains the only provider. This slice also audits `DurationObject:HasExpired` without replacing the existing [duration-core contract](duration-core.md). Cached declarations are later than the historical source epoch: they establish present declaration text, not native build 68182 behavior.

| Source rows | Contract boundary | Public proof |
|---|---|---|
| prose-undated-022, global api-C_DurationUtil-CreateDurationTextBinding-031 | One factory, distinct userdata handles, default configuration | defaults, state roundtrips |
| scriptobjects-DurationTextBinding-CanUpdateFontString-094, Disable-095, Enable-096, IsEnabled-104 | Eligibility and host-enabled automatic updates | defaults, automatic updates |
| scriptobjects-DurationTextBinding-GetDuration-097, GetExpiredText-098, GetFontString-099, GetTimeModifier-101, GetUpdateInterval-102, GetZeroDurationText-103 | One live result from independent configured state | defaults, roundtrips, environment locality |
| scriptobjects-DurationTextBinding-SetDuration-105, SetExpiredText-106, SetFontString-107, SetTimeModifier-108, SetUpdateInterval-109, SetZeroDurationText-110 | Authenticated declared input shapes; atomic host/configuration changes | invalid setters, authenticated inputs |
| scriptobjects-DurationTextBinding-CanFormatText-093, GetFormattedText-100 | Live host duration/clock, formatter, zero/expired text | live formatting, secret timing |
| scriptobjects-DurationObject-HasExpired-089 | Inherited zero-span rule plus authenticated modifier/extras | HasExpired fixture |

## What it must do

### State and lifetime

- [x] Keep the existing userdata binding factory, weak tick registry and Copy/Assign/SetClock methods. Never install a second provider or retry another formatter after failure.
- [x] Keep enabled, update interval and duration time modifier in independent host-owned state. Rust reads current settings when getters or formatting run; changing one binding never changes another or another environment.
- [ ] Keep duration, font string, formatter, clock and text references rooted by the existing VM-owned configuration closures. No Rust payload may hold an untraced Lua `Val`.
- [x] Preserve identity of configured duration/font string references and shared clock references during Copy/Assign. Scalar settings and tick schedules are independent.
- [x] INFERRED: newly created 12.0.7 bindings have nil duration/font string/expired/zero text; enabled=true, interval=1 and modifier=RealTime. Reset clears duration/font string/format/formatter/text and restores scalar defaults. Later declaration says reset clears these fields; exact historical defaults remain unverified.

### Live production

- [ ] Sample the existing duration object and host-owned clock live through the Rust duration producer, with no mutable Lua clock substituted by the binding.
- [x] Read RealTime/BaseTime modifier from host settings. Concrete interval [10,18) at rate 2 returns 6s at clock=12 in RealTime and 12s in BaseTime; advancing, rewinding, changing rate/span or rebinding the duration clock affects subsequent formatting without replacing the binding.
- [x] Use configured zero text for unconfigured/zero duration and configured expired text after expiration. INFERRED: zero text wins over expired text; otherwise a valid NumericFormatter is required. Missing formatting configuration raises an explicit error; it never returns a fabricated "0".
- [x] Use the configured NumericFormatter for remaining-duration text. Preserve the existing percent-format SetTextFormat extension; do not substitute another formatter when the configured formatter errors or returns nil.
- [x] CanFormatText reflects eligibility; CanUpdateFontString additionally requires a configured font string. Eligibility does not require automatic updates to be enabled.
- [x] Disable suppresses automatic updates, Enable resumes them, and zero interval updates each engine tick. INFERRED: first update/configuration changes invalidate cadence immediately; elapsed cadence uses engine ticks while numeric duration sampling uses its bound clock.
- [x] Return exactly one value from getters/predicates/GetFormattedText and no results from setters/Enable/Disable. Getter nils remain explicit single nil results.

### Input and secrecy

- [x] Every selected setter with cached `SecretArguments = "AllowedWhenUntainted"` calls `rilua::table_security::unwrap_secret` on ALL supplied inputs, receiver and ignored extras before any receiver/type/range validation. Tainted rejection leaves configured state unchanged; public plain inputs still work without clearing caller taint.
- [x] SetDuration accepts the existing LuaDurationObject, not a numeric pseudo-duration. SetFontString requires a FontString. Expired/zero text accept string or nil. Time modifier accepts RealTime/BaseTime. Update interval requires a finite nonnegative number. INFERRED: strict noncoercing validation and negative/nonfinite rejection; later cache gives types, not native coercion/range behavior.
- [x] Keep authenticated secret text as VM wrappers, not decoded public strings. INFERRED: secret timing sampled into text (including zero/expired text selection) yields a VM secret text result; tainted callers cannot derive text from secret timing, and the font-string handoff preserves the wrapper.
- [x] INFERRED: authenticated wrapped references and scalar settings normalize to their decoded values; wrapped text remains wrapped, and duration timing secrecy remains the existing core policy. No native secret-return parity is claimed beyond the conditional-secret declaration.
- [ ] No selected input declaration says NeverSecret/NotAllowed. Do not invent an authentication policy from absent annotations. If historical declarations add such a policy, reject secrets for every caller before validating anything.
- [x] DurationObject.HasExpired authenticates all inputs/extras before modifier validation under `retail-12-0-7`; omitted/nil/RealTime/BaseTime are accepted. INFERRED: zero-span remains fully elapsed/expired via inherited 12.0.5 semantics. The page only lists the method addition and the cache says "reached its end time"; neither establishes a contrary zero-span exception.
- [ ] New defaults/formatting/validation/HasExpired authentication are gated on `retail-12-0-7`. Earlier profile implementations receive shared host-scalar storage without claiming historical API availability. Earlier profile execution must be checked by integration.

## How it works

- [Duration core](duration-core.md) — existing timing and host-owned clocks.
- [Lua API](../lua-api.md) — VM values and public API environment.
- [Event system](../event-system.md) — engine tick dispatch.

## Implementation inventory

- `src/c_api/duration_text_binding.rs` — sole binding provider, VM reference configuration and weak engine-tick scheduler; feature-gated formatting/input contract.
- `src/c_api/duration_text_binding/state.rs` — host scalar settings, argument authentication and live Rust duration sampling; no alternate binding factory.
- `src/lua_api/globals/lua_duration_object.rs` — existing duration identity validation exposed crate-internally.
- `src/lua_api/globals/lua_duration_object/core.rs` — existing duration producer plus gated HasExpired authentication, inherited zero-span semantics unchanged.

## Tests asserting this spec

- `tests/patch_12_0_7_duration_text_binding.rs` — nine new public-API behavioral tests: defaults/arity, Copy/Assign/reset/surface, live host clock and modifier, engine cadence, environment isolation, invalid-input atomicity, all-input authentication ordering, secret text/timing handoff, HasExpired boundaries.
- `src/loader/tests/wow_api_globals/startup_globals.rs::test_patch_12_0_7_duration_objects_and_text_binding` — corrected zero-span expectation and documented duration/formatter fixtures replacing numeric pseudo-durations.
- `tests/patch_12_0_7_duration_clocks.rs` — existing host clock proof; unchanged.

Observed integration on default Retail, 2026-10-04: new module 9/9; duration core 27/27; duration clocks 9/9; startup globals 26/26; binding lib 1/1; retained copy 7/7, including cached CustomAuraButton execution; common numeric formatting 5/5. Exact revisions, argv and logs: `/home/osso-test/.cache/wow-ui-sim-audit/p1207-r8-result.md`. Proof is bounded simulator behavior, not native historical conformance. Binding tick controls select zero tests on Retail and earn no proof credit.

## Known gaps (current cycle)

- [x] Targeted RED compiled: new module 1/9 passed before producers; GREEN 9/9 after producers and correcting the FontString readout fixture. Existing lib fixtures and retained copy controls use real durations/formatters, preserving their assertions.
- [ ] Historical cache/build 68182 and native probes are unavailable. Formatting options/component composition, exact default texts, rounding and expiry precedence are not authenticated for 12.0.7.
- [ ] NumericFormatter and FontString checks currently validate public protocol/type information; exact native handle identity is not independently proven.
- [ ] Exact native formatter behavior and full secret-output policy need historical/native reconciliation; staged code rejects non-string formatter results and sampled output policy is explicitly INFERRED.
- [ ] Earlier profile regression proof, weak-scheduler collection and exhaustive scheduler error-handler behavior remain unproved. Retained Copy resource lifetime and handle identity are covered by the passing Retail copy controls.

## Out of scope

- B06/B07 clock redesign: existing host clocks are reused, never replaced.
- B31 DurationTextFormattingOptions methods and later text-color/component behavior: separate rows and historical epoch evidence required; existing methods are retained but this slice does not claim their complete conformance.
- Page-ledger/capability promotion and native-client execution: owned by the main audit, not this integration.
- Earlier-profile execution: excluded by this integration task; no earlier-profile availability or regression claim.
- FontString GetText secret-return tagging: existing secure readout returns plain text; the binding test observes the wrapped public SetText input instead of changing widget policy.
- Overridden duration-query methods: live sampling currently resolves the existing duration getter through its Lua proxy; trusted dispatch under method replacement is not established by these tests.
