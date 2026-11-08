# Mists Cast-Bar Startup Visibility

Verified: 2026-10-11. Historical Mists epoch `20261010T084353Z` failed the immediate-hidden assertion after addon load. Current settled-fade/corrected-parent proof below supersedes that epoch's fixture acceptance framing; historical failures and native uncertainty remain preserved.

<a id="current-fixture-phase--2026-10-11-unverified"></a>

## Current fixture phase — 2026-10-11, bounded proof

[Current acceptance authority](integrated-source-and-factory-proof-2026-10-09.md#acceptance-authority--2026-10-11) requires coherent addon-compatible simulator behavior, not native timing parity. Commit `4e3848a77` advanced the normal animation pipeline by 0.55 simulated seconds before every unchanged original assertion. [Retained epoch `20261011T155435Z`](https://github.com/Osso/wow-ui-sim/blob/183edc381/data/patch-api/evidence/mists-source-fade-phase-2026-10-11/README.md) compiled successfully and ran the exact original prefork cast-bar case: **0 PASS / 1 FAIL** at the next `UIParent` parent expectation, after the hidden assertion. Its trace records hidden state, parent equal to `UIParentBottomManagedFrameContainer`, unchanged layout-parent identity and absent showing-frame membership. The separate source-fade control reports **one PASS execution**; nested timeout parent/child summaries are not two executions. Source/frame effects are not proof that later original assertions ran.

Current follow-up corrects that single unsupported parent expectation to require the retained manager parent. Cached `CastingBarAnim_OnFadeOutFinish` calls `Hide`; `UIParentManagedFrameMixin:OnHide` removes manager membership, and `RemoveManagedFrame` does not restore `UIParent`. The strict visibility/layout-parent/membership and unlocked/locked anchor-transition checks remain; no production reparenting, unconditional `Hide` or vendor timing patch was added. [Authenticated epoch `20261011T171434Z`](https://github.com/Osso/wow-ui-sim/blob/183edc381/data/patch-api/evidence/mists-source-fade-phase-2026-10-11/parent-followup/README.md), revision `95e215f78` including correction `83acf239e`, passes the exact original prefork cast-bar case once and the separate fade control once: **two unique executions, zero failures**. Later strict unlocked player-anchor and locked cast-anchor transitions both execute successfully. Independent saved-artifact audit2262 authenticated source endpoints, features and both sealed executable hashes; no next assertion failure was observed. Raw outcome wording saying every original assertion was unchanged is stale: the parent expectation was explicitly corrected. The initial immediate-hidden and subsequent parent failures remain historical. Startup stderr retains self-anchor errors and early systemInfo READ_ERROR; this is not clean startup, whole Mists phase, exact historical vendor-cache identity, native timing parity or full-handoff acceptance.

## Exact execution boundary

Saved receipt root (private): `/home/osso/.local/state/wow-ui-sim/verification/mists-prefork-admission-fixed/20261010T084353Z/`. Submission revision: `845bea6d5e378a893e6d9945f46ce7b5163b6b19`; selected profile `client-mists`. `execution/results.json` records the original selector `spell_casting::cast_bar_respects_edit_mode_lock_setting_after_startup_fix`, one execution, exit 1. Its stdout records 0 passed / 1 failed; stderr identifies `tests/spell_casting.rs:489:13`, “Mists startup cast bar should remain hidden”. These are saved receipts, not a new run.

| Captured phase | IsShown | Parent / showingFrames | Proof |
|---|---|---|---|
| After addon load, before post-load workarounds | true | UIParent / absent | Exact 084353 stderr |
| After post-load workarounds, before startup events | true | UIParent / absent | Exact 084353 stderr |
| Before EDIT_MODE_LAYOUTS_UPDATED | true | UIParent / absent | Exact 084353 stderr |
| After EDIT_MODE_LAYOUTS_UPDATED | true | UIParentBottomManagedFrameContainer / present | Exact 084353 stderr |
| After startup events; before original assertions | true | UIParentBottomManagedFrameContainer / present | Exact 084353 stderr |

The first captured shown state precedes post-load workarounds and startup events. This bounds investigation toward addon-load creation/lifecycle initialization; it does **not** identify the operation responsible, prove a hidden→shown transition, or rule out uncaptured intermediate changes. The later parent/membership transition is observed, not a proven visibility root cause.

The preceding lock-zero and unattached assertions did not stop this execution. The Mists hidden assertion fails. Later original Mists parent, layout-parent, and showingFrames assertions are **unreached**, as are subsequent player-anchor and locked cast-anchor assertions. Their input values may appear in diagnostics, but that does not turn unreached assertions into passes or failures. The non-Mists parent assertion is inapplicable.

## Historical epoch is not current evidence

The older private diagnostic `mists-cast-diagnostic/20261009T221327Z/exact-run.stderr`, summarized in `mists-visibility-phase-final.md`, records `IsShown=false` across its captured phases, UIParent parenting, and absent showingFrames membership. Its recorded service exit 101 is not the 084353 original assertion receipt. Do not import its hidden observation or `firstIsShown=false` into the current failure, or infer cross-epoch source/cache identity. The earlier assertion-matrix report's contradictory assertion numbering does not override the exact current panic and source assertion order.

## Corrected OnLoad source interpretation

The private `mists-onload-main-errata.md` corrects the earlier nil-unit inference:

- Cached `Classic/CastingBarFrame.xml` supplies inherited inline `self:OnLoad(nil, true, false)` and concrete `PlayerCastingBarMixin` with a method OnLoad handler using `inherit="append"`.
- The inline expression dynamically looks up the final frame method; it is not a lexical call to `CastingBarMixin.OnLoad`. With the final player method installed and reached, it resolves `PlayerCastingBarMixin:OnLoad()`, whose extra arguments are ignored and whose explicit base delegation supplies `"player"`.
- Thus this expression proves no second `SetUnit(nil)` or player→nil transition. A repeated player initializer enters SetUnit's changed-unit body only if intervening state makes its guard true.

Local handler emission supports instance-before-inherited append ordering and preserves the inline lookup. This is static source interpretation, not proof of the actual runtime method table, successful handler completion, call count, or unit state.

## Conditional FinishSpell evidence

`mists-finishspell-final.md` reports the cached idle sequence: base OnLoad → changed-unit SetUnit → PLAYER_ENTERING_WORLD handler → no channel/cast → FinishSpell → PlayFadeAnim. FinishSpell does not synchronously hide on that described path. When the animation exists, alpha is positive, visibility permits it, and earlier operations succeed, PlayFadeAnim may start FadeOutAnim. Cached animation XML specifies **0.2-second start delay and 0.3-second duration**.

This supports a source-level deferred-fade candidate, not native timing/visibility parity or proof that 084353 reached FinishSpell or started the fade. The handoff leaves direct completion-callback inspection unresolved; no completion semantics or source-timed probe is credited here as acceptance of the original assertion. Earlier fill-update failures could prevent later calls. No assertion replacement follows from this candidate.

## Identity and acceptance limits

Direct comparison of saved `source-before.json` / `source-after.json` found all 3,851 captured entries equal. Saved `execution/cache-before.json` / `cache-after.json` found all 3,055 entries equal; `results.json` also records `cache_equal=true` and no changed paths. This establishes captured within-084353 stability only. Submission exclusions include external path dependencies, uncaptured data/runtime inputs, inherited environment, and untracked index. It establishes neither cross-epoch identity nor native parity.

| Feature / boundary | Status | Proof level |
|---|---|---|
| Original hidden-at-startup assertion | FAIL | Saved exact 084353 execution |
| Later original assertions | Unreached | Panic boundary plus assertion order |
| Final player OnLoad lookup; nil argument ignored | Conditional source interpretation | Private main erratum / cached source inspection |
| Idle FinishSpell deferred fade parameters | Source-supported candidate | Private Lua/XML inspection; executed root unknown |
| First visibility operation / native acceptance | UNKNOWN / BLOCKED | No call-level trace or matching native evidence |

Root call trace remains unknown: actual method lookup, handler entries/exits, query results, cast/edit-mode state, and first visibility-changing operation lack execution evidence. Native acceptance remains blocked. Call-level replay/instrumentation was not run for this audit.

## Temporary simulator trace — original assertion still FAIL

The temporary observer in diagnostic commit `5bc63e7da` emits unconditional stderr state records only for the exact stored name `PlayerCastingBarFrame`: stored visibility, alpha, numeric identity and parent ID. `xml_properties_before/after` bracket `apply_xml_properties_direct`; the after observation also occurs on a returned error, before propagation and before intrinsic property application. `xml_onload_before/after` bracket each Rust OnLoad dispatcher call, using the supplied Lua frame value's ID, not each internal binding handler. OnLoad `success=true|false` describes only the outer dispatcher result: internally caught handler failures remain possible. `show_before`/`hide_before` occur immediately before the no-op branch; `show_after`/`hide_after` follow `set_frame_visible`, before visibility-handler dispatch. No-op calls reaching that branch attempt only a before observation; earlier rejection/error paths are not covered. Fixed `observation_failed` records may lack exact-name confirmation when state/frame lookup is unavailable. Stored values do not establish effective ancestor visibility/alpha; direct state writes outside these hooks remain unobserved. Vendor source and original assertions are unchanged.

[Retained trace](https://github.com/Osso/wow-ui-sim/blob/183edc381/data/patch-api/evidence/narrow-repairs-2026-10-09/integrated/mists-visibility-trace/cast-trace.txt) and [receipt](https://github.com/Osso/wow-ui-sim/blob/183edc381/data/patch-api/evidence/narrow-repairs-2026-10-09/integrated/mists-visibility-trace/receipt.json) bind diagnostic source `5bc63e7da`, epoch `20261010T222248Z`, compiled exit0/source-equal. The unchanged exact original selector executed once: **0 PASS / 1 FAIL**, exit1 at the same hidden assertion. Four target records use one ID13136: XML properties before/after and outer OnLoad before/after all record stored `visible=true`, alpha1 and parentID1. Outer dispatch reports success=true, not per-binding success. No Show/Hide record or observation-failure record appears in this captured output. Cache membership/content and source endpoints match; these are captured bounds, not continuous immutability. [Independent saved-receipt audit](https://github.com/Osso/wow-ui-sim/blob/183edc381/data/patch-api/evidence/narrow-repairs-2026-10-09/integrated/mists-visibility-trace/final-report.md) admits these bounds: 3,854 source and 2,977 cache entries match; fmt/Mistscheckexit0. Diagnostic check retains six dependency deprecations and one simulator unused scalar re-export warning, not warning-free. The subsequent re-export capability gate is a separate compile-only correction.

The initial XML declaration and inspected ancestor templates contain no parent `hidden=true`; hidden selection children are separate widgets. The loader's initial-hidden resolver therefore selects false for this source chain, while the widget default is visible. This source interpretation agrees with the earliest trace observation; it does not prove native WoW defaults or all later call paths. The shared fade callback calls parent Hide after animation completion, but its execution was not observed here.

Temporary logging was removed after the sealed capture; three Rust files restore their pre-diagnostic contents. This is not a public feature or behavior repair. Stderr overhead prevents performance conclusions. Original assertion and later unreached assertions remain unchanged; native expectations, any need for startup settling, and root behavior contract remain unresolved. No new full suite is warranted by temporary instrumentation followed by exact source restoration.

## Related reduced ProductChoice diagnostic — 2026-10-11

[Separate packet](https://github.com/Osso/wow-ui-sim/blob/183edc381/data/patch-api/evidence/productchoice-declaration-probe-2026-10-11/README.md), epoch215919/independent2430, observes a returned self-anchor loader error for the copied Icon declaration and valid parent/sibling anchor identities in a separate control. Compile/run0 describe diagnostic completion, not successful declaration loading. This does not change the cast/fade fixture, repair Mists startup, identify its full-template error ordering or establish native semantics.

## Texture XML self-key producer checkpoint — 2026-10-13

[Owning policy and pending gates](../../specs/xml-self-relative-key.md) record source `480b9f934` after genuine Mists `a90f84c5d` RED (1 PASS / 1 FAIL, actual exit101). With real cached ModelScene/ProductChoice prerequisites, creation reaches the layer-child self-anchor failure; the separate direct Lua guard preserves two points and passes its distinct-target control. Texture XML published direct single-segment self-key normalization is approved simulator policy, not native parity. Shared Frame/FontString generation and missing-key handling remain unchanged.

Producer GREEN, normal startup and named prefork acceptance are pending. The [historical source844 packet](https://github.com/Osso/wow-ui-sim/blob/183edc381/data/patch-api/evidence/mists-source-fade-phase-2026-10-11/current-844-2026-10-13/README.md) retains one selected cast PASS after 0.55 seconds and separate normal CLI self-anchor FAIL; neither attributes the anonymous baseline frame to ProductChoice. Two earlier admission denials are separate zero-launch records, not behavioral RED.

## XML producer bounded runtime checkpoint — 2026-10-13

[Owning acceptance](../../specs/xml-self-relative-key.md#known-gaps-current-cycle) and [evidence index](https://github.com/Osso/wow-ui-sim/blob/183edc381/data/patch-api/evidence/xml-self-relative-key-2026-10-13/README.md) own current focused/normal/shared-anchor/named-prefork gates. Normal cached Mists startup is independently CLEAN; named cast control still settles0.55s before hidden assertions and retains captured diagnostic read errors. This does not attribute the original first visibility writer, erase immediate-hidden FAIL, prove native parity, or establish full-parent acceptance. Initial77-copy PASS and follow-up42 technical/corrected-metadata reuse PASS remain separate from runtime proof; historical process/metadata FAILs are preserved.

## Sources

- Private `handoff/mists-load-visibility-final.md` — earliest shown state, XML/loader observations, identity limits.
- Private `handoff/mists-finishspell-final.md` — conditional idle fade and unresolved callback/native limits.
- Private `handoff/mists-onload-main-errata.md` — dynamic method lookup correction; withdrawn SetUnit(nil) inference.
- Private 084353 receipts named above — submission, exact stdout/stderr, execution results, source/cache before/after captures. Private handoffs and receipts live under `/home/osso/.local/state/wow-ui-sim/`; raw vendor payloads are not copied here.
- [Original assertion source](../../../tests/spell_casting.rs) — assertion ordering; current source read is not a cross-epoch execution claim.

## See Also

- [[prefork-test-harness]] — profile admission differs from original fixture acceptance.
- [[integrated-source-and-factory-proof-2026-10-09]] — broader proof ledger; unchanged by this task.

## Current selected-body receipt — 2026-10-13

[Owning current packet](https://github.com/Osso/wow-ui-sim/blob/183edc381/data/patch-api/evidence/mists-source-fade-phase-2026-10-11/current-844-2026-10-13/README.md) separates phase-adjusted execution from historical immediate-hidden failure and unresolved acceptance.
