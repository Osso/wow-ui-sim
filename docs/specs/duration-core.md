# Duration core

Clock-driven duration state for the existing Lua table proxy in `src/lua_api/globals/lua_duration_object.rs`. Formulas and the bounded secret-duration policy below are simulator choices, not confirmed native WoW semantics.

## What it must do

- [x] Preserve table-proxy identity, hidden metatable, method names, and current profile publication; instances hold independent timing state.
- [x] Configure start/base-duration/rate with `SetTimeFromStart`; configure the equivalent interval from its end with `SetTimeFromEnd`; configure an unmodified interval with `SetTimeSpan`.
- [x] Query endpoints, total, elapsed, remaining, rate, and zero/activity status consistently while a bound manual clock advances or rewinds.
- [x] Use simulator elapsed time (the `GetTime` time source) for an unbound clock and `C_DurationUtil.GetCurrentTime`.
- [x] Reset timing/rate without changing the selected clock; `SetToDefaults` also clears that clock binding.
- [x] Reject tested nonfinite endpoints/durations, negative durations, reversed spans, and zero rates before changing timing state. Additional overflow/nonfinite-rate rejection is implemented but not separately proven.

- [x] Return elapsed/remaining fractions from the same timing state, with clock-boundary clamping, rewind, and modifier validation. Focused proof passed 7/7 on retail 12.0.0 (reused matching hashes), 12.0.5, and 12.0.7; see `/tmp/verify-duration-percent-ledger.json`.

### Copy and assignment

- [x] `Copy` returns one independent duration with the source's configured start, base duration, and rate; later timing changes on either object do not affect the other.
- [x] `Assign(other)` transfers configured timing into the existing receiver and returns no values; self-assignment leaves state unchanged.
- [x] Copy the optional clock reference, not the clock object. Clock advancement is shared; rebinding either duration is independent. An unbound source clears the receiver's previous clock binding.
- [x] Preserve receiver identity and custom fields during assignment. Reject missing or non-duration sources before changing receiver state.

Cached `LuaDurationObjectAPIDocumentation.lua` describes copying a duration and assigning another duration into the receiver. Clock-reference handling, custom-field retention, and validation above are simulator policies, not native historical-client evidence. Only modeled timing and clock state transfer; arbitrary source fields are not part of duration state.

### Bounded secret timing — explicit simulator guesses

Forever's cached `LuaDurationObjectAPIDocumentation.lua` declares the three timing setters as `SecretArguments = "AllowedWhenUntainted"`, describes `HasSecretValues` as non-secret metadata, and explicitly says `SetToDefaults` clears secret state. Native `Blizzard_AuraButton.lua:167–169` passes `secretwrap(...)` into timing setters. `/tmp/ellesmere-forever/duration-secret-input-red.stderr` reproduces all three setters rejecting those wrappers while the caller is untainted. User-run Forever probes are unavailable.

The following choices are **informed simulator guesses**, not native-verified lifecycle or output semantics:

- [x] Timing setters accept authenticated rilua wrappers through `unwrap_secret`; arbitrary userdata still fails numeric validation. A wrapped nil optional rate means the ordinary default rate of one. All arguments validate before timing changes.
- [x] Any wrapped timing argument makes all three stored timing slots secret. Slots `-1`, `-2`, and `-3` contain VM-owned wrappers, never plaintext plus a public flag; `rawget` therefore does not disclose their numeric payloads. `HasSecretValues` derives its non-secret boolean from these wrappers.
- [x] Secret configuration persists through ordinary timing reconfiguration, `Reset`, and assignment from a plain source. Only `SetToDefaults` clears it through the duration-method surface. Reset retains the selected clock; defaults clears it.
- [x] `Copy` can copy opaque wrapped slot values and the clock reference without unwrapping, including from a tainted caller. `Assign` requires an untainted caller when either source or destination contains secret timing, and retains existing destination secrecy.
- [x] Timing getters, including zero/activity predicates, reject tainted callers for secret-configured durations. Untainted callers receive ordinary computed values after authenticated unwrapping. No plaintext query cache bypasses this check. `GetClock` retains its ordinary clock-reference behavior; it does not read timing payloads.
- [x] Timing mutations, resets/defaults, assignment, and clock rebinding require an untainted caller when existing secret timing is involved. Failed authorization or argument validation leaves stored timing and clock unchanged.

`/tmp/ellesmere-forever/secret-handoffs-tests-ledger.json` records 28/28 duration-core cases at the secret-handoff checkpoint; `/tmp/ellesmere-forever/secret-layout-tests-ledger.json` adds 3/3 duration-secret cases. The final 90-second trusted GUI acceptance also paints and removes the secret-duration aura. These prove the documented simulator policy on Forever only; plain duration behavior and other profiles retain their existing surface.

This policy does not claim native secret-return tagging. Widget and text-binding handoffs must preserve the access boundary separately; a successful core setter is not proof of complete aura rendering. Numeric-slot write immutability and general VM secret arithmetic remain outside this slice.

### Chosen formulas — native-unverified

Store start `s`, base duration `D >= 0`, and finite rate `r > 0`. Real span `T = D / r`; end `e = s + T`. `SetTimeFromEnd(e,D,r)` derives `s=e-D/r`. `SetTimeSpan(s,e)` stores `D=e-s`, `r=1`.

`RealTime=0` (default) returns total `T`, elapsed `clamp(now-s,0,T)`, remaining `T-elapsed`. `BaseTime=1` multiplies these duration results by `r`. Endpoint queries remain clock coordinates regardless of modifier. Unknown duration-query modifiers error; native modifier/coercion behavior is unverified.

Percentage queries return dimensionless fractions in `[0,1]`: `GetElapsedPercent` is elapsed divided by `T`, and `GetRemainingPercent` is remaining divided by `T`. Both return zero when `T==0`. Omitted/nil, `RealTime`, and `BaseTime` give the same fractions because numerator and denominator share the time scale; invalid modifiers and invalid bound clocks still error. Division uses real-time quantities before scaling. Fraction scale, clamping, zero-span results, validation, and rewind behavior are simulator policies, not native conformance claims.

`IsZero` means configured `D==0`, not expiration. Zero durations report false for `HasStarted`, `HasExpired`, and `IsActive`, preserving existing default behavior. For nonzero durations, `HasStarted` means `now>=s`; `HasExpired` means `now>=e`; `IsActive` means `s<=now<e`. Before start, elapsed is zero and remaining is the full interval. After end, elapsed is the full interval and remaining is zero. Clock rewind reverses these observations without modifying configured timing.

`Reset` produces `s=0,D=0,r=1` and retains the clock. `SetToDefaults` additionally selects the default simulator clock. Existing manual clocks use their mutable `time` field; invalid/nonfinite bound clock time errors on clock-dependent queries. Clock setter validation remains unchanged.

## Common numeric formatting

Retail 12.0.5 source `data/patch-api/sources/12.0.5-api-changes.txt:19` names AbbreviatedNumberFormatter, NumericRuleFormatter, and SecondsFormatter as duration Format consumers. Cached `NumericFormatterAPIDocumentation.lua` defines `FormatNumber(number) -> string` with `SecretArguments = "AllowedWhenUntainted"`; duration documentation declares all three Format methods with NumericFormatter arguments, RealTime default modifiers, and `SecretWhenNumericFormatterSecret`.

- [x] Accept real instances of all three formatter types in `FormatElapsedDuration`, `FormatRemainingDuration`, and `FormatTotalDuration`. Reuse each existing formatter's numeric contract, not an arbitrary Lua callback or a duck-typed table.
- [x] Apply the existing duration getter's omitted/nil, RealTime, or BaseTime modifier exactly once, returning one string. Reject unknown formatters and invalid modifiers without changing duration timing or formatter configuration.
- [x] Preserve secret duration, modifier, wrapped formatter, and formatter-derived output provenance. Retain existing untainted-caller requirements for decoded secret timing and arguments; do not clear caller or callback closure taint.
- [x] Keep derived secret time opaque at the SecondsFormatter Lua/curve boundary. Native abbreviated/numeric-rule dispatch and core duration queries must not deliver decoded time, modifiers, or formatter configuration to arbitrary Lua callbacks, including replaced getters, public FormatNumber methods, or string.format.

Conservative output propagation from secret duration/modifier/wrapped receiver is a **simulator guess**, not native-verified parity. Existing rendering bounds remain: abbreviated formatting models enUS/enGB; numeric rules retain their finite numeric printf policies; native SecondsFormatter uses the existing private localized backend, while profiles without that capability retain numeric-text output through a checked common FormatNumber entry. Adding duration consumption does not establish localized SecondsFormatter rendering on those profiles.

Future native probes: use all three real factories with a manual duration of base 2468/rate 2 at elapsed real time 300; compare total/elapsed/remaining outputs for omitted/nil, RealTime, and BaseTime. Repeat with secret timing, modifier, formatter references, and formatter-derived secrets in secure and tainted closures; record result secrecy, exact strings, callback input secrecy/closure taint, errors, and unchanged duration/configuration after rejection. Exact native coercions, error strings, secret-configuration policies, and callback evaluation ordering remain unverified.

## How it works

- [Lua API architecture](../lua-api.md)
- [Duration text binding](duration-text-binding.md)

## Implementation inventory

- `src/lua_api/globals/lua_duration_object.rs`: existing proxy factory, clock binding, method registration.
- `src/lua_api/globals/lua_duration_object/core.rs`: timing configuration, authenticated wrapped-slot storage, access checks, and queries.
- `duration_has_secret_values(&LuaState, Val) -> bool`: crate-private metadata helper for consumer handoffs; no payload exposure.

## Tests asserting this spec

- `tests/duration_core.rs`: manual progression/rewind, rate modifiers, end/span configuration, reset, atomic validation, default time source, independent instances; percentage boundary/rewind, zero-span, invalid modifier, invalid clock, and seven Copy/Assign simulator-policy cases.
- `tests/duration_numeric_formatters.rs`: four default-profile cases cover all three actual formatter objects, concrete total/elapsed/remaining values, modifiers, secret provenance/caller guards, receiver/contract validation, captured numeric printf, trusted core getter dispatch despite Lua overrides, and unchanged state after errors. A fifth native-backend case covers opaque curve input and retained callback taint; it was feature-disabled in the default proof below and remains pending.
- Existing `tests/cooldown_widget.rs` and duration-text-binding tests: bounded consumer regression checks; these do not establish native core formulas.

Common formatting implementation `8f716ae99` / `789b6b91b` has focused default GREEN at parent build `9a50d8a5cc20d0adf0b7c529d237fc043ef57532`: all four `duration_numeric_formatters::` cases pass, including number-`__tostring` and overwritten-getter boundaries. Existing abbreviated formatter controls pass 4/4 and numeric-rule controls pass 7/7. Logs are `/tmp/patch-12.0.5-batch4-{duration-common-green,abbreviated-control,numeric-rule-control}.log`. Earlier build `a3ba2a23a` records 0/4 RED in `/tmp/patch-12.0.5-duration-common-red.log`: three incompatible abbreviated receivers and one decoded modifier sent to an overwritten Lua getter. The later number-`__tostring` assertions were not part of that RED binary. `/tmp/duration-numeric-formatters-proof.json` records the scope.

Batch5 compiled `e0a46d691` supersedes the prior primitive-text/raw-93 boundary: common format 5/5 including curve/closure taint, Seconds format 7/7 including cached garden consumer, configuration 7/7 and native-method controls 3/3 PASS. Exact argv/logs: `/tmp/patch-12.0.5-batch5-runs.json`; chronology: `/tmp/duration-numeric-formatters-proof.json`. These are bounded development GREEN for native-enabled simulator output, not native-client parity or final acceptance; independent verifier 76 and final Rust gates remain pending. Private bootstrap/registry integrity is trusted; no privileged debug/upvalue confidentiality claim.

`/tmp/verify-duration-percent-ledger.json` records the earlier seven-case percentage proof. The first Copy/Assign GREEN attempt at `19416ff84` did not compile: `/tmp/duration-copy-green-ledger.json` records E0308 before tests ran. `acb86ceda` corrects that mutability mismatch; `/tmp/duration-copy-green-retry-ledger.json` retains durable 7/7 output but lost its numeric exit code in a post-execution wrapper error. Final independent proof is `/tmp/verify-duration-copy-ledger.json`: all 15 `duration_core::` tests pass with exit 0 on 12.0.0, 12.0.5, 12.0.7, and Mists; format, default check, both default binaries, no-addons/no-saved-vars startup (`[]`), both affected validators, and all-manifest hash scanning pass. Historical warnings remain 6/6/1/6; default verification has none. This proves ordinary simulator policies only. Focused proof at `89a71308d`: `duration_core::` has three passing tests on PTR and retail; `test_patch_12_0_7_duration_objects_and_text_binding` passes on PTR. Bounded consumer run at `9aa4a1eb7` passed eight tests and failed one forbidden-object AuraContainer fixture; these are not a clean full consumer acceptance result.

## Cooldown zero-duration option

`SetCooldownFromDurationObject(duration, clearIfZero)` defaults `clearIfZero` to `true` in both pinned base and PTR declarations. The simulator interprets explicit `false` with a zero duration as preserving the cooldown's current start, duration, display duration, and rate. Omitted/true clears timing; nonzero duration objects update timing regardless of the flag. `tests/cooldown_widget.rs` covers these cases with real duration proxies and independent frames. This interpretation does not establish native, protected-call, or type semantics.

## Known gaps (current cycle)

- [ ] Native rate/modifier, endpoint, zero-state, reset, validation, and clock-binding semantics require real-client evidence.
- [ ] `SetCooldownFromDurationObject` now resolves duration-proxy methods through Lua indexing and ordinary proxy transfer/reset behavior is covered. Its new PTR protected-function contract, secret handling, and forbidden-object behavior remain unproven.
- [ ] Existing `test_patch_12_1_duration_binding_reference_lifetime_and_identity` expects a table from the separate duration-text-binding factory and fails with `type`; the core preserves its own table identity. Existing AuraContainer binding integration fails with `expected forbidden object reference`. These are separate consumer boundaries; this slice establishes no cause beyond their observed failures.

## Out of scope

- Curve evaluation is specified separately in [duration curve evaluation](duration-curve-evaluation.md); rendering remains outside this slice. Native copy/assignment identity, clock, coercion/error, custom-field, and lifecycle/GC semantics remain unverified.
- General secret-value propagation, native getter-return secrecy, protected/forbidden enforcement, and immutable proxy internals are not inferred from this bounded core policy.
- Consumer redesign or changes to duration-text-binding identity: preserve current proxy representation.
