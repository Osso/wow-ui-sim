# Native API contract probes

Scalar curve point, `UnitSexBase`, unit-name/realm, finite numeric-formatting, cast/channel identity, selected action-slot, scalar resource scale, plain callback lifecycle, and residual `StripHyperlinks` investigations share one manual [ApiContractProbe](../../addons/ApiContractProbe/README.md). The existing [AuraDispelCurveProbe](../../addons/AuraDispelCurveProbe/README.md) covers dispel-ID input. Runtime `3f7934e27` adds `names`; `41bcd89f0` adds numeric capture; both are included in `all`. These recorder experiments are prepared; none was installed or executed in a native client by this task. They do **not** cover all remaining probe needs.

## Evidence boundaries

Scalar point declarations specify vectors but do not establish indexing, ordering or returned-object identity. The recorder observes empty/populated curves, explicit index queries, accessible fields/GetXY, repeated equality and an isolated mutation attempt. Cached `Blizzard_SharedXML/Vector2D.lua` provides vector fields but does not prove curve return behavior.

`/apicontract curve-state <label>` is manual-only and excluded from `all`; it leaves `curves` unchanged. Runtime `411edf39e` captures empty and populated unsorted duplicate-point scalar curves, accessible type/count/points/secret-flag, evaluations at seven fixed inputs, and results before/after reset. It separately records a copied curve before and after original mutation and clearing, without classifying defaults, duplicate handling, interpolation or copy isolation.

Runtime `9b7fdb43e` adds independent manual `/apicontract curve-edit <label>`, also excluded from `all`. Fresh scalar curves receive `RemovePoint` indices `-1, 0, 1, 2, 3, 4`; separate fresh curves receive `SetPoints` with empty and unsorted duplicate inputs. Replacement vectors come only from accessible `CreateVector2D`, and each case retains definitions, bounded point/count state, `Evaluate(15)`, mutation arity and opaque errors. The cumulative harness has 41 fixtures; that is not 41 new native cases. Native curve index base, ordering/duplicates, returned-copy behavior, `SetPoints` coercion and secret semantics remain capture work.

Runtime `ffd3b08ca` adds manual `/apicontract color-curves <label>`, also excluded from `all`. It constructs one owned curve with four color points, then a copy, records raw state, point lookups and packed/unpacked evaluation, and observes reset/copy mutation without classifying defaults or isolation. Returned tables expose only guarded raw RGBA fields; userdata stays opaque and no returned methods or metamethods run. A separate color harness has four fixtures; the existing harness retains 41. Neither suite is native execution. Native userdata field representation, identity, coercion, security, removal/replacement and alternate interpolation behavior remain unexecuted.

`UnitSexBase` returns `Enum.UnitSex`; legacy `UnitSex` numbering cannot be copied blindly. The recorder stores both raw returns and named enum values for real existing units, with scenario labels for independently observed transformations. It does not infer what “Base” means.

Name/realm capture invokes `UnitName` and `UnitNameUnmodified` without existence gating for fixed player, party, target, unknown and invalid-token inputs. It preserves exact arity and positional nils, so nil and empty realms remain distinct from explicit realm strings. Scenario labels identify independently assembled same-realm and cross-realm party slots; returned names never classify the session. Native same-realm and cross-realm sessions remain pending. Actual populated-slot, recharge, consumable transition, and duration-semantics captures remain pending.

`/apicontract numbers <label>` records raw `GetLocale` provenance plus exact arity, positional nils, and bounded accessible result bytes from both `C_StringUtil.FloorToNearestString` and `C_StringUtil.RoundToNearestString`. Its ordered 25 finite inputs cover signed ties and close neighbors, integers, fractions, and large finite magnitudes. It records literal observations only; helper names do not establish rounding or grouping semantics. Nonfinite inputs, coercion, security behavior, values outside this corpus, and actual build/locale captures remain unknown.

`/apicontract casts <label>` records `UnitCastingInfo` and `UnitChannelInfo` for fixed player, target, focus, party, unknown, invalid and empty-unit controls. It preserves exact arity and nil positions through sixteen scalar-only return slots; values beyond that bound are marked truncated. Returned objects retain only status/kind: the recorder never invokes methods, indexing, equality or string conversion. `all` includes casts. The matching client and manual sequences for ordinary casts, cancellation, replacement/consecutive casts, non-player channels, and empowered/non-empowered channels remain pending; spell IDs and fixture actions are intentionally not invented.

`/apicontract actions <slot> <label>` records one manually selected action slot; `all` deliberately excludes it. It captures `GetActionInfo`, display count with default and three explicit thresholds, and `GetActionCharges` arity plus raw `currentCharges`, `maxCharges`, `cooldownStartTime`, `cooldownDuration`, and `chargeModRate` fields. `GetActionChargeDuration` retains only accessible kind/status and arity, never values or methods. It does not call `GetActionCooldown`, infer slot roles, or execute actions. Actual populated-slot, recharge, consumable, before/use/restoration transition, and duration-semantics captures remain pending.

`/apicontract resources <label>` is an explicit residual best-effort recorder; `all` excludes it. It records player, target, focus, pet and nonexistent-unit raw health/max, power/max/type, omitted/nil curve queries, and explicit `unmodified=false`/`true` power queries. A six-point scalar linear curve spans `0`, `.5`, `1`, `25`, `50` and `100`; observations record its inputs and raw results rather than classifying native input scale. It neither invents power IDs nor changes resources. Color curves, prediction, secondary power, actual partial-resource fixtures, native event ordering and security behavior remain pending. Its 29 local fixtures prove recorder mechanics only; this remains outside the snapshot and earns no native credit.

`/apicontract hyperlinks <label>` is an explicit residual best-effort recorder; `all` excludes it. It submits 16 literal inputs across nine optional-flag variants (144 calls), recording input bytes, flags, argument count and bounded raw results. The corpus includes balanced and incomplete markup, Unicode, literal `|n`, newline, escaped pipes and an orphan suffix. It makes no expected-output claim. Native execution remains pending; malformed/nested semantics, coercion, arbitrary bytes, string-view lifetime and security behavior remain unknown. This residual recorder is neither a new audit credit nor completion of an active snapshot plan.

`/apicontract callbacks-start <label>` registers only this addon's plain Lua callbacks for global `UNIT_HEALTH` and player-filtered `UNIT_HEALTH`; `callbacks-stop` removes the retained identities. It records registration/removal results and up to 128 passive deliveries with exact arity and up to sixteen safe scalar positions. Partial registration and cleanup errors remain recorded rather than treated as stopped. It neither fires events nor examines wrapper containers. The 33 local fixtures prove recorder mechanics only. A native producer, post-removal delivery boundary, `Event.lua` FunctionContainers, duplicates, ordering, aliases, mutation/recursion and security behavior remain pending. This residual best-effort work does not complete the earlier simulator callback credits or a snapshot plan.

Dispel curve numeric IDs remain pending the separate prepared capture. See [[aura-dispel-curve-probe]].

## Local proof

Tests `e2adbb57d` initially failed because the new addon did not exist. The scoped recorder proof has since grown by focused commits; scalar-curve edit runtime `9b7fdb43e` brought the harness to 41 local fixtures, and color-curve runtime `ffd3b08ca` adds four scoped fixtures to that existing base. These counts are cumulative harness size, not per-slice native evidence. Callback fixtures cover start/stop, identity cleanup, bounded payloads, redaction and failure retention. Fixtures remain recorder proof only, not native contracts. Cast fixtures cover scalar tenth/eleventh positions, nil positions, empowerment values, repeated/consecutive snapshots, zero returns, opaque errors, restricted values, hostile objects, the sixteen-position bound, and `all` inclusion. Action fixtures cover selected-slot command parsing, five raw charge fields, display observations and opaque duration handling; they do not establish action semantics. Actual TOC/slash/SavedVariables wiring remains locally exercised.

The name recorder's prior 13 fixtures plus 12 supplemental checks remain valid within their earlier scoped commit. Numeric, cast, action, and hyperlink fixtures prove literal recording only, not native behavior. Fixture outputs are not native contracts. No audit credits or totals changed. Actual native captures still require matching-client same-/cross-realm party sessions, build/locale numeric sessions, and controlled cast/channel/empower fixture sequences before assigning compatibility evidence.

## Complete preparation inventory

[Preparation inventory](../../baselines/native-probe-preparation.json) retains all 282 snapshot plans: 275 active, covering 1,083 rows, and seven without active rows. Snapshot membership excludes best-effort residual gaps and exceptions; their separate reconciliation remains pending.

At `81f0c2ac1`, 112 plans have shared publication/event recording components, seven have partially reviewed dedicated observations, 154 still need design/recorder review, and two are explicitly deferred security plans. No whole-plan completion is claimed from these counts. Existing older probes may supply more coverage but must be inspected before credit.

`AuditTargets.lua` supplies 199 publication/CVar and 88 event-registration targets. These capture parent/member/CVar observations and real event tuples, not missing producers, historical phases, transitions or semantic contracts. Runtime `e2af1e8e1` initially invoked payload methods during passive event inspection. Regression `5870cfa46` reproduces that error; `d959372a3` uses non-executing raw table inspection for passive payloads. Independent proof reuses 10/10 main fixtures and passes 21/21 supplemental checks, including zero payload `GetXY`/`__index` calls. Ledger: `/tmp/verify-all-probes-recorders-corrected-ledger.json`.

All-probe preparation remains open. No installation or native execution is authorized by recording-code preparation.

## Sources

- [Recorder spec](../../specs/api-contract-probe.md)
- [Manual capture protocol](../../addons/ApiContractProbe/README.md)
- [Dispel probe spec](../../specs/aura-dispel-curve-probe.md)

## See Also

- [[patch-12-0-0-api-audit]] — unresolved native contracts
- [[aura-dispel-curve-probe]] — companion experiment
