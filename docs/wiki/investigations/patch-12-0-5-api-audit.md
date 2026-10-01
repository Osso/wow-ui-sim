# Patch 12.0.5 API Audit

The expanded Patch 12.0.5 source audit is **IN PROGRESS** after source-retention commit `7ff275fd3`; full-page behavior coverage is not established. Earlier work was probe-driven rather than a full API-change-page audit. Retail `12.0.5.67823` live probes pinned core frame, event, attribute, identity, scale-event, and XML frame-level behavior; the simulator already models the safe findings with regression coverage. No dedicated `patch_12_0_5_inert_defaults` module exists.

## Content

### Source scope

The original audit used live-client probe addons under `docs/addons/` and corresponding wiki investigations, not the full patch page. Commit `7ff275fd3` retains the entire plaintext **Patch 12.0.5/API changes** extract in [12.0.5-api-changes.txt](../../../data/patch-api/sources/12.0.5-api-changes.txt), with [retrieval provenance](../../../data/patch-api/sources/12.0.5-api-changes.provenance.json): retrieved 2026-09-30, SHA-256 `4da3872aa566695f46e2dacd4e79992f5b06be9541f0d19cf0e8dba45cea8329`. This is plaintext, not raw wikitext or its link graph.

The source inventory `/tmp/patch-12.0.5-inventory.json` counts **244 consolidated delta rows / 186 distinct subjects**, plus **118 chronological prose rows**; some prose is narrative rather than an API. Consolidated sections cover Global API, ScriptObjects, Widgets, Events, Enums, and Structures. The source has **0 CVar entries**. These are source-coverage counts, not implemented API counts or behavioral proof. Chronological PTR proposals and future plans must remain distinct from the consolidated snapshot; 362 inventory rows do not imply 362 shipped APIs.

**Guessed implementation policy:** the user explicitly requests best-supported guesses for missing APIs rather than stalling for native evidence. Label each guessed contract as a guess, record its supporting evidence, and track a concrete future probe identifying the call/input scenario and observable results needed to resolve uncertainty. Guesses and simulator tests do not establish native semantics. Existing probe-register classifications below are preserved, not silently reclassified by this expanded policy.

Primary retained 12.0.5 probe sources (13 SavedVariables captures): `AnimScriptProbe`, `AttributeDispatchProbe`, `CoreBehaviorProbe`, `DevToolsDumpProbe`, `FrameIdentityProbe`, `HookScriptBindingProbe`, `IsProtectedProbe`, `JustifyProbe`, `ProtectedRetailProbe`, `ScaleEventProbe`, `SetAtlasProbe`, `StoreForbiddenProbe`, and `TextureSetTextureProbe`. `XmlFrameLevelProbe` findings are documented, but its raw capture was not retained.

The machine register is `data/patch-api/12.0.5-probes.json`, sourced from `data/patch-api/sources/12.0.5-probes.json`; [[patch-12-0-5-probe-inventory]] is its human-readable inventory. It preserves 38 probe subfindings. Current machine classification is **33 best-effort, 0 implemented, 4 evidence-required, 1 exception-requested, and 0 untriaged**: the exception is approved provenance-only, while four behavior gaps remain evidence-required—one impossible same-size input-boundary gap and three unsafe Store/security gaps.

### Expanded capability and future-probe coverage

The six committed slices below are a **documentation/source-inference inventory, not code proof**. Linked specs own implementation proof updates; their recorded targeted results are not independently checked here. Full patch audit remains **IN PROGRESS**; independent final gate is pending, and none of these slices establishes native equivalence or adds credit to the 38-subfinding register.

| Commit / contract | Bounded capability described by spec | Evidence boundary / future native probe |
|---|---|---|
| `326e571b8` — [Action text](../../specs/action-text.md) | `UsesActionText` / `GetActionText` read occupied macros' exact current nonempty names through rename, move, replacement and deletion. | Label qualification is inferred; spec reports targeted 5/5 GREEN, not final proof. Probe names/whitespace/body variants, populated item/spell/empty slots and lifecycle changes; real item slots and cross-profile execution remain unproven. |
| `5bfca2e16` — [UnitHasPowerType](../../specs/unit-has-power-type.md) | One non-secret boolean from present-unit primary and explicitly modeled player secondary capability; cumulative 12.0.5 publication. | Absent-unit, pet/vehicle alias and strict argument policies are inferred; spec records RED, GREEN pending. Probe zero-capacity pools, class/spec transitions, aliases, absent tokens, invalid/secret arguments and combat secrecy. |
| `f372687b6` — [Scenario unit criteria](../../specs/scenario-unit-criteria.md) | Supplied per-token credit/percentage/display triples; empty-default map and explicit row identity restriction, with guarded secret input/output. | Missing-row zero-return and row restriction policies are inferred; proof checkboxes remain open. Probe no scenario/unknown/zero-credit units, reassignment, localization, restriction transitions and tainted/secret calls. Host-result adoption `6eace49d5` now passes four bounded scenario cases, including tainted plain-token restricted output; secret-input guards remain. Native identity policy and independent final proof remain open. |
| `e8ebb47c1` — [Enum publication and source/proof matrix](../../specs/patch-12-0-5-enum-additions.md#retained-source--proof-matrix) | Current-retail additions, removals and rename; source-backed related shifts and actual-member metadata. | Parent RED observed; post-change GREEN pending. Spec owns exact source accounting and per-test proof. Historical/native numbering and downstream domain semantics are unclaimed; existing `Relinquished` remains unchanged. |
| `9d89c7021` — [Outfit catalog lookups](../../specs/outfit-catalog-lookups.md) | Four queries share an empty-default catalog; documented fields, case-insensitive names, explicit indices, fresh tables and guarded secret arguments. | Empty enumeration, Unicode/duplicate/order and invalid-input policies are inferred; focused GREEN/final verification pending in spec. Probe empty results, Unicode normalization/casing, sparse indices and exact invalid-input returns/errors. No mutation/persistence/lifecycle claim. |
| `50feedbfe` — [Cooldown decimal threshold](../../specs/cooldown-decimal-threshold.md) | Stored Lua threshold drives formatter: one decimal below threshold, whole-second ceiling otherwise; zero/reset and aura/abbreviation precedence. | “Below” is documented; zero-disable, ceiling and precedence are inferred. Spec lists formatter tests, not GPU/native proof; probe 19.9/20.0/20.1 at threshold 20, zero/changed thresholds, mode precedence and rollover. |

### Current-retail enum publication

`e8ebb47c1` corrects current-retail enum values in `c_api/patch_12_0_5_enums.rs`, after base publication. The `client-retail` gate preserves historical/PTR surfaces. Existing 12.1 compatibility fills additional members; refresh derives metadata from actual numeric members after initialization and post-load. Disabled photo-status insertion requires shifted existing values, and current unit-frame names/values are grounded in cached documentation and the existing strict removal, not old sequential positions. No unrelated enum import or vendor edits.

The [enum spec](../../specs/patch-12-0-5-enum-additions.md#retained-source--proof-matrix) is the single source for exact row/subject counts, literal values, removal/rename cases, doc lines and grouped test proof. Earlier 36/24 accounting is superseded by that retained-source matrix. Parent owns GREEN and final checks; numerical publication does not establish native/domain parity.

### Batch4 bounded development proof

Compiled revision `9a50d8a5cc20d0adf0b7c529d237fc043ef57532`: retained `/tmp/patch-12.0.5-batch4-integration-build.{json,log}` records successful integration compilation and executable `integration-0915b883f3757151`. The actual binary ran focused filters; these are **development GREEN**, not independent final acceptance, native proof, or API-coverage counts.

| Exact source row / capability | Executed scope | Remaining boundary |
|---|---|---|
| `prose-2026-03-25-091` — [FontString smooth scaling](../../specs/fontstring-smooth-scaling.md) | Six API cases PASS: isolated state, validation/secret caller guards, fractional and auto height, wrapping, ordinary/runtime XML. | This batch does not execute renderer tests; GPU/native metrics and earlier XML-profile support remain unproven. Source says XML predates the API; 12.0.5+ publication does not cover that earlier claim. |
| `prose-2026-03-12-019` — [Common duration formatting](../../specs/duration-core.md#common-numeric-formatting) | Four cases PASS: three real formatter instances, duration/modifier dispatch, invalid-input atomicity and secret provenance/caller guards. Abbreviated 4/4 and numeric-rule 7/7 controls PASS. | Native localized Seconds duration units remain RED in the separate batch4 regression (raw 93); source-wide formatter compatibility is not complete. Conservative secrecy is simulator policy, not native parity. |
| [Scenario unit criteria](../../specs/scenario-unit-criteria.md) — host-result adoption `6eace49d5` | Prior four-case host-secret development run PASS, including tainted plain-token restricted outputs without contaminating public values. | Explicit row classification remains supplied input; native identity policy and final acceptance remain open. |

Evidence: `/tmp/patch-12.0.5-batch4-{font-api-green,duration-common-green,abbreviated-control,numeric-rule-control}.log`, `/tmp/patch-12.0.5-scenario-host-secret-green.log`, and current `/tmp/patch-12.0.5-proof-ledger.md`. [Page coverage links](../../../data/patch-api/sources/12.0.5-page-coverage.json) attach only these exact supported source rows; other accounting remains audit-pending. Whole page remains **IN PROGRESS**. No new credit or reclassification enters the 38-subfinding native-probe register.

### Batch5 bounded development proof

Default integration build `e0a46d691` PASS: `/tmp/patch-12.0.5-batch5-integration-build.{json,log}`. Exact executable argv, filters, exits and log paths: `/tmp/patch-12.0.5-batch5-runs.json`. These runs supersede pending batch4 output claims only within the scopes below; independent verifier 76 is pending. No final Rust checks, native-client parity, or full-page completion.

| Exact source rows | Development result | Remaining boundary |
|---|---|---|
| `prose-2026-03-31-147` | Stat restriction 4 PASS covering 40 supported API outputs. | Explicit supplied restriction state; missing stats 10 FAIL; base models and native activation/parity pending. |
| `prose-2026-03-31-141`–`144` | UnitIsUnit final permission matrix 6 PASS, including nil denials. | Earlier chronology remains classified separately; no native parity. |
| `prose-2026-03-31-160`; consolidated `global api-C_ActionBar-GetActionCooldownDuration-235`, `global api-C_Spell-GetSpellCooldownDuration-307`, `global api-C_SpellBook-GetSpellBookItemCooldownDuration-324` | ignoreGCD 6 PASS. | Supplied cooldown/GCD selection only; no secrecy or native consumer parity. |
| `prose-2026-03-12-019` | Common format 5 PASS including curve/closure taint; Seconds format 7 PASS including cached garden consumer, configuration 7 PASS, native-method controls 3 PASS. | Native-enabled simulator output supersedes prior raw-93 RED/pending; not native-client parity or entire three-formatter statement completion. `/tmp/duration-numeric-formatters-proof.json` retains chronology. |
| `prose-2026-03-12-023` | Charge duration 1 PASS / 3 FAIL. Cooldown countdown formatter separately 8 FAIL. | Implementations pending agents 73/74/75; no new GREEN credit. `GetSpellChargeDuration` resolves through `runtime_surface_bootstrap.lua` lines 65–76 (`__wow_namespace_mt.__index` lazy nil closure), explaining no-data PASS; `C_Spell.GetSpellCharges` uses a temporary zero-table default. Models/producers are missing, not API globals. Other static C_* absence reports also require final-provider tracing. |

The compile-fixture FF failure gate fix `e0a46d691` is not Forever GREEN: other-main profile compilation remains pending. All 362 source row IDs and source-register chronology/classifications remain intact; linked rows can still contain unproven clauses. Counts are inventory, not capabilities. Source accounting and the entire 12.0.5 goal remain **IN PROGRESS**. The 38 native-probe subfinding statuses are unchanged.

### Bounded unit-stat output secrecy (verified 2026-10-01)

[Unit-stat restriction spec](../../specs/unit-stat-output-restriction.md) owns the exact 50-row coverage matrix and pending base-model names. `8da12a42c` adds the plain `SimState.unit_stats_restricted` boolean, default false, and matching `C_Secrets.ShouldUnitStatsBeSecret`; `f35d0293f` adds concrete grouped fixtures; `edab2563a` marks all numeric results of the 40 supported default-retail APIs while retaining existing values, arity and order. Ten unsupported base models remain separate pending work; secrecy support does not establish native stat-model parity.

`c_secrets::push_stat_number` accepts only a concrete Rust-computed `f64`, uses rilua's trusted host-number producer, and roots the result immediately without changing caller taint. The module compiles across profiles, but predicate and aura registrations retain their feature gates; profiles without `retail-12-0-5` push plain numbers. The existing zero-return producer was shared by source-covered PvP/resilience APIs and unrelated miss/enemy queries: only the source trio uses the restricted producer now.

Observed predicate RED at `a3ba2a23a` was missing publication. Output RED at `9a50d8a5c` passes the predicate and fails three output tests, including `GetAttackPowerForStat secrecy 1`; log `/tmp/patch-12.0.5-batch4-stats-output-red.log`. Batch5 at `e0a46d691` passes four stat-restriction cases covering the 40 supported API outputs; ten missing base-API cases fail. Independent verifier 76 and parent-owned final gates remain pending. Restriction activation is an explicit approved simulator input, not an aura/combat heuristic or native-verified policy. Future native probes and absent-unit limitations remain in the spec. Full patch audit remains **IN PROGRESS**; no register reclassification.

[Patch-page discovery index](../../../data/patch-api/patch-page-index.json) discovers **138 API pages / 98 retail-history candidate titles**. This is title classification only: neither retained page-content coverage, shipped API classification nor runtime/behavior proof.

### Itemized probe status

**Machine-classified with direct behavioral evidence:** the full Frame/AnimationGroup/nine-subtype script-handler matrix; repeated scalar/false attribute dispatch and the two-panel ShowUIPanel pulse; normal-frame forbidden behavior and absent retail forbidden constructor; valid/invalid unit-event filters; wildcard false/true/string attributes; Raise/Lower level boundaries and GUI mouse-focus ordering; frame identity slot, surrogate dispatch, duplicate-frame freshness, and DevTools frame-array dump metadata; normal HookScript chaining plus rejected explicit slots 0 and 2; absent legacy protection setters, the full plain-frame and XML-protected-frame sequences, and protected secure templates; frame-layer FontString default points, size variants, explicit anchors, implicit ButtonText anchors, EditBox backing regions/TextInsets, and MessageFrame/ScrollingMessageFrame owner-region behavior; complete observable display/UI-scale/CVAR ordering; the complete invalid-atlas argument matrix; texture path/FDID and clear behavior; and bare/fixed/parent/reparent XML frame-level semantics and flags.

**Evidence and exception state:** XML raw-capture provenance is the only approved exception because it concerns missing historical evidence while frame-level behavior is independently regression-tested. Same-size transitions remain an evidence-required impossible input-boundary gap. Secure Store behavior, Store dropdown population, and Store forbidden descendants remain evidence-required unsafe Store/security gaps; existing subsystem tests are not substituted for the missing probe behavior.

### Open probe gaps and evidence-required rows

A broad approval recorded on 2026-07-14 is superseded. The five rows below distinguish four item-specific evidence-required behavior gaps from one approved provenance-only exception-requested row. Evidence-required rows carry hashed repository evidence but need no approval, commit, or focused test; they await authoritative/live evidence or correct implementation.

1. **ProtectedRetailProbe.SecureStore — evidence-required unsafe:** Retained Store frames are forbidden, legacy setters are absent, and `IsProtected` errors; the current simulator returns normally, so exact forbidden/secret-return enforcement is unsafe to guess.
2. **ScaleEventProbe.SameSizeDuplicatePair — evidence-required impossible:** Retained live observations already establish that maximize/restore can produce another ordered display/scale pair without a dimension change. The missing boundary is the production window-transition signal, not screen-size evidence: the probe records dimensions and state, while the simulator receives only draw-time `iced::Size` and deliberately ignores equal sizes. Pinned iced 0.14.0 / winit 0.30.12 expose no maximize/restore/fullscreen transition notification. Their mode/maximize queries are not ordered transition events, so polling would be an approximation. Correct behavior remains unmodeled.
3. **XmlFrameLevelProbe.RawCaptureProvenance — approved impossible:** Behavior is regression-tested, but the raw SavedVariables capture does not exist and cannot be reconstructed locally.
4. **StoreForbiddenProbe.DropdownPopulation — evidence-required unsafe:** The retained capture has `StoreDropdown_SetDropdown == nil`, so population, reuse, text/check, callback, and protection behavior was never observed.
5. **StoreForbiddenProbe.ForbiddenDescendants — evidence-required unsafe:** The retained file lacks the `/sfp` manual descendant scan, so Store descendant forbidden/protected state is unknown; correct behavior remains unmodeled pending authoritative/live evidence.

The 38-row register is complete only for its explicit probe contract; generic fallbacks cannot be claimed as globally patch-complete without another concrete source.

### Completed modeled work

Retail `12.0.5.67823` probe results are modeled in these areas:

- `CreateForbiddenFrame` is absent on current retail, and `SetForbidden(true)` on addon-created normal frames succeeds without making the frame forbidden.
- `RegisterUnitEvent("UNIT_HEALTH", "not_a_unit")` registers the event but drops the invalid unit filter; `IsEventRegistered("UNIT_HEALTH")` returns registered with no unit filter.
- Wildcard `GetAttribute` preserves an explicit `false` stored with `SetAttribute("*type1", false)`.
- `Raise()` / `Lower()` only affect same-raw-level tie ordering and do not let a lower frame level overtake a higher one.
- Frame identity dispatch uses `frame[0]` userdata tokens; surrogate tables shaped with `[0] = frame[0]` dispatch shared frame methods, while `[1]`-only surrogates do not.
- Duplicate named `CreateFrame` calls produce fresh Lua objects and fresh identity tokens rather than copying stale custom fields from the prior global binding.
- XML bare `frameLevel` is an absolute initial value, not a parent-relative offset, but remains non-fixed so later parent level changes shift the child by the captured parent delta; `fixedFrameLevel="true"` pins the level.
- `DISPLAY_SIZE_CHANGED` and `UI_SCALE_CHANGED` fire as an ordered pair for observable size/scale recalculations, with startup pairs before `PLAYER_LOGIN`.

Key implementation locations:

- `src/lua_api/frame/methods/text_attribute_event/events.rs` — invalid `RegisterUnitEvent` filter fallback and animation handler validation.
- `src/lua_api/frame/methods/text_attribute_event/attributes.rs` — retail forbidden-frame and attribute behavior.
- `src/lua_api/methods.rs`, `src/lua_bridge/table_builder.rs`, `src/lua_api/globals/create_frame/helpers_shared.rs` — frame identity token dispatch and duplicate named-frame behavior.
- `src/lua_api/globals/template/direct/frame_level.rs` — XML frame-level resolution and fixed/non-fixed propagation.
- `src/lua_api/env_runtime.rs`, `src/startup.rs`, `src/iced_app/resize_event_tests.rs` — display/scale event pair behavior.

### Verification

Regression coverage exists in:

- `tests/admin_event_api.rs` — invalid unit-filter registration fallback.
- `tests/protected_frame_enforcement.rs` — retail `SetForbidden` no-op behavior.
- `tests/protected_attribute_enforcement.rs` — wildcard explicit-false lookup and repeated-false dispatch ordering.
- `tests/frame_level.rs` — Raise/Lower and raised-frame-level ordering.
- `src/iced_app/mouse_tests.rs` — GUI hover, `GetMouseFocus`/`GetMouseFoci`, and Raise/Lower focus ordering.
- `tests/security_api.rs`, `tests/frame_table_iteration.rs`, `tests/globals_legacy.rs` — frame identity slot, surrogate dispatch, opaque identity userdata, duplicate named-frame freshness.
- `tests/xml_frame_strata.rs` — XML `frameLevel` and `fixedFrameLevel` semantics.
- `src/iced_app/resize_event_tests.rs` — display/scale ordered-pair behavior.

### Same-size transition boundary

`ScaleEventProbe` captures screen and physical dimensions, UI-scale CVars, and `UIParent` scales with each ordered event pair; it does not need another dimensions capture. `App::sync_screen_size_to_state` receives only `iced::Size` during drawing and does nothing when both dimensions match. The pinned window stack has resize, move, focus, and scale-factor notifications but no mode/maximize/restore/fullscreen transition notification. There is therefore no production input that distinguishes an equal-size maximize/restore transition from no transition. Do not fire another pair merely because the same size is observed again, and do not add an admin event as a fidelity substitute.

### Remaining inert/default surface

There is no 12.0.5-specific inert-default module. Broad compatibility defaults still live in `src/lua_api/workarounds/temporary/` and permanent unsupported C API shims, but the 12.0.5 probe-backed findings listed above have modeled behavior and tests rather than patch-scoped inert stubs.

The remaining generic defaults are intentionally outside this 12.0.5 audit unless a probe or addon failure ties one to a 12.0.5 retail behavior contract. Examples include unsupported 3D/model domains, loose/placeholder namespace defaults, and compatibility fallbacks that are tracked by their own subsystem investigations.

### Audit state

The expanded source audit remains **IN PROGRESS**, with no completed full-page behavior claim. Separately, the historical 38-subfinding probe register remains open with 4 evidence-required rows and 1 approved provenance-only exception-requested row. The four behavior gaps are one impossible same-size input-boundary gap and three unsafe Store/security gaps; they are not exception or approval candidates. Authoritative/live evidence or correct behavior is still required before this audit can close. No 12.0.5-specific inert-default module remains, but absence of a patch shim is not proof that every retained probe result has exact regression coverage.

### Secret-string formatting bounded runtime proof

[Secret-string formatting spec](../../specs/secret-string-formatting.md) covers only retained line 40, `prose-2026-03-12-040`: secret `%s` ignores width/precision and preserves full payload; public formatting is unchanged. Allowed tainted opaque formatting is **inferred**, not native-verified permission; input/result unwrap guards and stack taint remain intact, with no arbitrary callback capability. `SetFormattedText`, display provenance and other formatting domains remain unproven.

MAIN published rilua `host-secret-bool` revision `6044544b960cd68b4b0c58bb3373412757c2caee` after explicit user approval and remote verification; simulator pin commit `c5ba89ae3` changes only Cargo/lock pin. Independent runtime report `/tmp/rilua-secret-format-independent-proof.md` records 6 formatter PASS, 2 host-guard PASS, fmt/check PASS with pre-existing `strlen` warning—not warning-free. Actual simulator old-pin RED at `eac08bda3` is 1 public PASS / 5 secret FAIL (`/tmp/patch-12.0.5-batch6-secret-format-red.log`). New-pin batch7 is pending compilation: **no simulator integration GREEN or native proof**. Concrete future probe and remaining guard/display boundaries live in the spec. Full page remains IN PROGRESS; 38-row probe classifications are unchanged.

## Sources

- [Unit-stat output restriction](../../specs/unit-stat-output-restriction.md) — exact supported/pending matrix, proof and future native probes.
- [Retained full plaintext patch page](../../../data/patch-api/sources/12.0.5-api-changes.txt) and [provenance](../../../data/patch-api/sources/12.0.5-api-changes.provenance.json) — expanded source audit, not behavior proof.
- `/tmp/patch-12.0.5-inventory.json` — working source inventory; temporary artifact, not a committed manifest.
- [Patch-page discovery index](../../../data/patch-api/patch-page-index.json) — title-only discovery; capability contracts and commit references are linked in the table above.

- [[retail-core-behavior-probes]] — core 12.0.5 live-client behavior findings.
- [[frame-surrogate-identity-slot]] — frame `[0]` identity-token behavior.
- [[display-size-ui-scale-events]] — display/scale event pair behavior.
- [XmlFrameLevelProbe](../../../docs/addons/XmlFrameLevelProbe/README.md) — live XML frame-level probe notes.
- [CoreBehaviorProbe](../../../docs/addons/CoreBehaviorProbe/README.md) — live core behavior probe notes.
- [FrameIdentityProbe](../../../docs/addons/FrameIdentityProbe/README.md) — live frame identity probe notes.
- [ScaleEventProbe](../../../docs/addons/ScaleEventProbe/README.md) — live display/scale event probe notes.
- [ScaleEventProbe source](../../../docs/addons/ScaleEventProbe/ScaleEventProbe.lua) — captured dimensions, CVars, and event order.
- [screen-size synchronization](../../../src/iced_app/update_runtime.rs) — simulator's size-only input boundary.
- [Cargo lockfile](../../../Cargo.lock) — pinned iced 0.14.0 and winit 0.30.12.

## See Also

- [[patch-12-0-5-probe-inventory]] — only 38 native-probe subfindings, not full patch-page coverage.

- [[patch-12-0-7-api-audit]] — later additive API bridge audit pattern.
- [[patch-12-1-api-audit]] — PTR API bridge audit pattern.
- [[lua-api]] — Lua runtime surface and frame method dispatch.
- [[retail-core-behavior-probes]] — retained 12.0.5 core probe evidence.
- [[event-system]] — event registration/dispatch behavior.
- [[xml-template-system]] — XML template and frame-level handling.
