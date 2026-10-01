# Duration core

`LuaDurationObject` is an existing Lua table proxy with simulator-owned start, base-duration, rate, and optional manual-clock state. It makes ordinary duration queries and cooldown transfer observable while retaining native timing, modifier, and security behavior as open questions.

## Behavior

`SetTimeFromStart`, `SetTimeFromEnd`, and `SetTimeSpan` configure independent table proxies. Queries derive endpoints, total, elapsed, remaining, rate, zero, started, expired, and active state from the selected clock. Manual-clock advance and rewind update later observations without mutating configured timing.

The core stores `start`, base duration, and rate. Its rate/modifier formulas, zero-state rules, validation, default clock, reset behavior, and percent semantics are simulator choices documented in [duration core](../../specs/duration-core.md), not native WoW confirmation.

`24fe9d746` replaces the ordinary zero placeholders for `GetElapsedPercent` and `GetRemainingPercent`. Both derive fractions in `[0,1]` from the existing clamped elapsed/remaining clock state: before start and after end clamp to the boundary, and clock rewind changes later observations without mutating configured timing. Both return zero for a zero span. `RealTime`, `BaseTime`, omitted, and nil modifiers produce the same fraction because scaling cancels. These behaviors, including invalid-modifier and invalid-bound-clock errors through existing validation, are simulator policies. `/tmp/verify-duration-percent-ledger.json` records seven focused tests passing on retail 12.0.0 (reused matching hashes), 12.0.5, and 12.0.7, plus passing format, check, build, and standalone startup; `/tmp/verify-duration-percent-reconciled-ledger.json` completes its metadata reconciliation.

`5f7d70fe6` adds a nonzero `SetToDefaults` transition: configured timing and a bound manual clock reset to zero endpoints, rate one, and no clock binding. `/tmp/verify-duration-defaults-ledger.json` records all eight duration-core tests passing on the same three profiles, with format/readability passing; production check/build/startup proof is unchanged and reused. This does not establish native compatibility, consumer loading, or security behavior.

`19416ff84` makes `Copy` and `Assign` transfer only modeled duration timing and optional clock binding. The RED record has 1/7 pass and six failures; the first GREEN attempt failed to compile before tests ran with E0308 at `lua_duration_object.rs:339`. `acb86ceda` corrects that mutability mismatch. Final independent proof at `/tmp/verify-duration-copy-ledger.json` passes all 15 `duration_core::` cases with exit 0 on 12.0.0, 12.0.5, 12.0.7, and Mists, plus format, default check, both binaries, zero-error startup, both affected validators, and all-manifest hash scanning. Historical warnings remain 6/6/1/6; default verification has none. Copying a clock reference, retaining receiver identity/custom fields, and all binding policies are simulator choices; no native copy/assignment, identity, coercion, security, lifecycle/GC, or consumer behavior is inferred.

`aae5996eb` replaces four duration curve-evaluation stubs that always returned numeric zero. `EvaluateElapsedDuration`, `EvaluateRemainingDuration`, `EvaluateElapsedPercent`, and `EvaluateRemainingPercent` now reuse their matching duration getter and the existing registered scalar/color curve evaluator. `/tmp/duration-curve-red-ledger.json` records ten cases failing before that replacement; `/tmp/duration-curve-green-ledger.json` records 10/10 passing on retail 12.0.0. Final proof `/tmp/verify-duration-curve-ledger.json` passes all 25 `duration_core::` cases with exit 0 on 12.0.0, 12.0.5, 12.0.7, and Mists; 11 PTR curve/userdata cases; format, default check, both binaries, zero-error startup, both validators, and all-manifest hash scanning. Historical warnings remain 6/6/1/6; PTR and default verification emit none. This does not establish a separate `LuaCurveEvaluatedResult` object: cached scalar and color curve declarations return numbers and ColorMixin values. Native curve, modifier, clock, coercion/error, secret, lifecycle, and consumer behavior remain open. See [[duration-curve-evaluation]].

`FrameAPICooldown.SetCooldownFromDurationObject` resolves proxy methods through Lua indexing. A zero duration preserves existing cooldown timing when `clearIfZero = false`; omitted/true clears it. This is tested simulator behavior, not native confirmation. Protected, secret, and forbidden semantics remain unresolved.

## Common numeric formatting

Retail 12.0.5 duration `FormatElapsedDuration`, `FormatRemainingDuration`, and `FormatTotalDuration` dispatch the three source-named NumericFormatter objects. Abbreviated and numeric-rule formatters use typed host-owned rows; numeric-rule printf must remain native. SecondsFormatter uses captured private identity/FormatNumber closures rather than public property lookup. Its configuration store is not exported by the handoff, and secret-derived duration input remains wrapped at the Lua/curve boundary.

Core getters come from the native methods registry, not replaceable fields on the duration table. Focused default RED at parent build `a3ba2a23a` records three wrong-abbreviated-receiver failures plus a decoded secret-modifier disclosure to a replaced Lua getter in `/tmp/patch-12.0.5-duration-common-red.log`. Parent build `9a50d8a5c` passes all four default common-format cases, four abbreviated controls, and seven numeric-rule controls. That common run includes the later number-`__tostring` boundary. Typed host number/string producers mint derived opaque values without clearing taint; existing secret reads retain their caller guards. The newly appended number-`__tostring` regression was not included in that RED build.

The [duration contract](../../specs/duration-core.md#common-numeric-formatting) owns modifier/secrecy guesses and profile bounds. That GREEN used the pre-promotion default Seconds primitive-text path. Separate actual unit-rendering RED returned `93` instead of units; `e0ff01ef1` now promotes the native capability into Retail 12.0.5. Batch5 `e0a46d691` passes common 5/5 including curve/closure taint, Seconds format 7/7 including cached garden consumer, configuration 7/7 and native-method controls 3/3. Exact argv/logs: `/tmp/patch-12.0.5-batch5-runs.json`; chronology: `/tmp/duration-numeric-formatters-proof.json`. This is development GREEN only; independent verifier 76 and final Rust gates remain pending. Profiles outside that capability retain checked primitive numeric text. Duration-text-binding behavior is unchanged. The [abbreviated formatter spec](../../specs/abbreviated-number-formatter.md) owns locale/breakpoint guesses. No native parity, general debug/upvalue confidentiality, vendor change, or caller-taint bypass is claimed.

## Cooldown duration selection

Action and spell duration producers now use a shared duration-only selector. Retail 12.0.5 true `ignoreGCD` selects the active individual cooldown; omitted/false reuses the unchanged GCD-aware later-end selector. Earlier epochs ignore the new argument. The new retail 12.0.0+ spellbook producer resolves integral player-bank-0 slots through existing entries and uses the same duration factory; absent entries and unsupported banks return nil, without pet or macro mappings.

These are inferred simulator selection/input policies, not native semantics. The retained 12.0.5 source only names the added arguments. Grouped ordinary-value RED at `9a50d8a5c` fails 6/6; batch5 `e0a46d691` passes the six default cases; earlier-profile and independent final proof remain pending. Contracts: [spell duration](../../specs/spell-cooldown-duration.md#1205-ignoregcd-contract-green-pending), [action duration](../../specs/action-cooldown-duration.md#1205-ignoregcd-contract-green-pending), [spellbook duration](../../specs/spellbook-cooldown-duration.md). No secrecy or consumer-parity claim.

## Player cast duration queries

`180d08b69` reuses the core factory for `UnitCastingDuration`, `UnitChannelDuration`, and `UnitEmpoweredChannelDuration` under the narrow `player-cast-durations` capability shared by Retail 12.1+ and Forever. The queries snapshot simulator-owned player cast/channel timestamps; idle and unmodeled units return no result. Empower defaults to hold-at-max inclusion, while explicit `false` uses the base empowered end. The latter boundary is an inference from the pinned Forever CastingBar consumer, which separately adds hold to `UnitChannelInfo` endpoints.

The same capability exposes the existing player channel lifecycle and uses numeric cast-bar IDs in cast/channel query tuples consistent with update and stop payloads. `tests/unit_cast_durations.rs` recorded RED 0/5 before the producer work; integrated GREEN and real Ellesmere acceptance remain pending. This establishes simulator behavior, not native timing, secrecy, other-unit state, or complete castbar conformance.

## Automatic duration-text binding boundary

A real ActionBarAuras replay on September 22, 2026 reached an assigned, shown CustomAuraButton for a modeled player aura but observed no duration text. This falsified the earlier event/filter/container hypotheses: `UNIT_AURA` dispatch, candidate filtering, slot assignment, and aura-button visibility all occurred. The missing boundary is automatic duration-binding updates on engine ticks, distinct from earlier manual `UpdateFontString()` binding proof.

`bf073db38` and follow-up `d6a3859e4` introduce scheduler-side binding work during engine OnUpdate processing. At frozen build `748e3668`, an isolated ActionBarAuras replay observes the real helpful player-buff path: `7s` becomes `6s`, `RemoveBuff` hides the assigned aura button, and the probe completes with no collected Lua errors. The 20-second process timeout occurs after completion; the shared host CVar hash is unchanged. Evidence: `/tmp/forever-addon-runtime/aba-duration-automatic-4rgzhxxk/{ledger.json,stdout}`.

This is bounded simulator evidence only. Final scheduler proof at `71d73ed81` reuses the exact `a61c6080d` source build and records binding-copy 11/11, native formatter 7/7, numeric formatter 8/8, visibility and event error continuation 1/1 each, format/default checks, PTR integration compilation, PTR binding-copy 7/7, and PTR native formatter 3/3: `/tmp/forever-addon-audit/verify-binding-tick-final-ledger.json`. PTR automatic scheduler runtime remains unproven because the scheduler test is Forever-gated. Target-debuff and rendering paths, native timing parity, first-update/expiry behavior, and native output equivalence remain open. The active implementation contract and tests belong in [duration text binding](../../specs/duration-text-binding.md).

## Sources

- [Abbreviated number formatter](../../specs/abbreviated-number-formatter.md) — scoped producer/consumer contract and unresolved native policies.

- [Unit cast duration queries](../../specs/unit-cast-durations.md) — modeled query contract and explicit inference boundary.
- [Duration core spec](../../specs/duration-core.md) — modeled contract and explicit assumptions.
- [`core.rs`](../../../src/lua_api/globals/lua_duration_object/core.rs) — state and query implementation.
- [`duration_core.rs`](../../../tests/duration_core.rs) — focused ordinary behavior proof.
- [`cooldown_widget.rs`](../../../tests/cooldown_widget.rs) — bounded proxy consumer proof.

## See Also

- [[patch-12-1-5-api-audit]] — exact changed API occurrences and evidence boundary.
- [Duration text binding](../../specs/duration-text-binding.md) — separate consumer path, bounded automatic-update proof, and pending final scheduler verification.
