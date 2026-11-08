# UI Widget Manager State

Environment-local supplied UI widget sets, visualization snapshots, exact quest associations and synchronous host feeds form the core model. Production `0745fe84f` adds supplied bars and HUD selectors. The [contract](../../specs/ui-widget-manager-state.md) owns requirements; the [retained attempts](https://github.com/Osso/wow-ui-sim/blob/183edc381/data/patch-api/evidence/ui-widget-manager-state-2026-10-13/README.md#later-consumer-attempts) own current outcomes and historical failures. Historical test-only `b28a45f91` has HUD OnLoad2 PASS and bar initial visibility FAIL; later `5bd208dba` closes the bounded Retail consumer assertions below. No native or whole-goal acceptance follows.

## State and representation

`UiWidgetInputs` stores set records keyed by `WidgetId`, independent IconAndText records, and exact `(questID, setType)` associations. A set contains layout direction, vertical padding, and an ordered vector of explicit widget IDs, types, and optional unit tokens. Numeric keys preserve floating-point bits except equal signed zeroes; availability is not inferred from ID ranges.

Queries project fresh Lua tables from supplied state: unknown set metadata and IconAndText records return nil; unknown membership returns an empty table. IconAndText projects the complete supplied record, including tooltip, texture kits, timer, ordering, animation, scale, layout, and model-layer fields, without automatic field fillers. Existing complete ButtonHeader and Prey records remain separate backing inputs; core feeds support IconAndText type0, ButtonHeader type30, and feature-gated PreyHuntProgress type31. New feed integration adds `WidgetVisualization` StatusBar type2 and CaptureBar type1. This is not coverage of every visualization type.

A C_API widget record is data, **not a Rust `Frame`**. Blizzard's cached Lua manager consumes that data and owns actual template selection, frame pools, activation, updates, and removal. Resulting ordinary frames participate in the existing [widget architecture](../../widget-system.md), [loading pipeline](../../addon-loading-pipeline.md), and [event dispatch](../../event-system.md). The producer does not replace the manager with Rust-rendered widget consumers.

## Exact demo associations

Initialization preserves the existing twelve demo world quests: three explicitly stored set types per quest (Tooltip, BehindIcon, AdventureMapDetails), totaling 36 sets and associations. The initializer derives the previous fixture IDs and supplies one IconAndText member per set with the `Worldquest` texture kit. Query-time lookup uses stored associations, not numeric thresholds or synthesized ranges. These are demo fixtures, not a native quest/widget catalog.

## Synchronous host feeds

`WowLuaEnv` exposes three source-implemented operations:

- `replace_ui_widget_set`: replace or remove a set, then synchronously publish `UPDATE_ALL_UI_WIDGETS`; independent visualization records survive.
- `update_ui_widget`: validate type and supplied-set existence, update an existing membership in place or append a new member, store the visualization, then synchronously publish `UPDATE_UI_WIDGET` with supplied identity/type/unit-token fields. Type mismatches and unknown sets fail before mutation or notification.
- `remove_ui_widget`: remove matching membership and publish `UPDATE_ALL_UI_WIDGETS` only when membership changed; return false for absent sets/members. Visualization snapshots remain independently queryable.

State is committed before notification. Full reconciliation and incremental consumer behavior belong to the cached Blizzard manager; source wiring alone does not prove their runtime lifecycle.

## Proof boundary and remaining surface

The [contract proof ledger](../../specs/ui-widget-manager-state.md#tests-asserting-this-spec) retains historical query, cached-container, security, deferred-layout, and unit-notification failures. Prior core proof remains separate: two actual lifecycle cases PASS at `e887`, plus 23 unchanged cases reused from `67775404d`; not a fresh 25-case run and not proof of the new bars/HUD slice.

### Bars, HUD, and feed implementation scope

- `bars.rs` supplies full 35-field StatusBar and 22-field CaptureBar DTO snapshots, including a fresh ordered StatusBar partition array.
- `hud.rs` exposes four live environment-local `f64` selectors. Each query returns exactly one number; default zero is simulator model policy, not a proven native default.
- `queries::register` calls both modules. Feed integration supplies StatusBar type2 and CaptureBar type1; the actual bar-consumer adaptation uses host feeds rather than direct state mutation/manual events.
- The old PowerBar bootstrap is removed without fallback. Query5/core19 and normal-startup/default-check proofs are reused within their original source scopes, not rerun for later fixture corrections.

### Historical RED and current consumer boundary

The [eight-case RED ledger](../../specs/ui-widget-manager-state.md#next-slice-tests-and-proof-gaps) at `87fecfd31` separates two wrong-type bar snapshot failures, three TopCenter exactly-one-return failures, one supplied-record query failure after a cached closure, and two numeric-query failures before cached HUD OnLoad. Later bar lifecycle and HUD consumer assertions were not reached. Earlier four bwrap missing-target launch failures executed zero cases and remain separate.

Later `39ce8969a` and `174b86505` each retain three consumer failures. At `b28a45f91`, HUD2 pass actual cached OnLoad; bar assertion59 fails initial neutral-glow visibility. Subsequent incremental bar updates, hidden/restore, membership removal and unregister remain unreached. With the explicit [atlas application-result policy](../../specs/texture-atlas-application-result.md), later `5bd208dba` actual bar1/HUD2 pass, including incremental updates, zero-zone geometry, hidden/restore, removal and unregister. The linked [packet](https://github.com/Osso/wow-ui-sim/blob/183edc381/data/patch-api/evidence/ui-widget-manager-state-2026-10-13/README.md#atlas-application-result-red-and-retail-green) owns exact current receipts; preceding failures remain historical. Later actual PTR consumers pass separately at `1804d1c49`; the [PTR and owning checkpoint](../investigations/integrated-source-and-factory-proof-2026-10-09.md#separate-ptr-and-owning-checkpoints--2026-10-13) owns exact runtime and reuse boundaries. Native return parity, other-profile and full-parent acceptance remain unestablished.

No native catalog, server transport, interpolation parity, dynamic HUD rebinding or unit-context proof is established. Query freshness does not prove rebinding of loaded containers. Consumer and broader parent acceptance remain OPEN.

### DiscreteProgressSteps producer checkpoint — 2026-10-13

[Discrete contract](../../specs/ui-widget-discrete-progress-steps.md) owns current producer and exact proof ledger. Historical42 input stage physically fails five cases on missing supplied record before cache/atlas. Current source registers complete fresh snapshots and type19 host-feed storage; query/security direct inputs and actual cached consumer typed feeds remain distinct. New feed tests observe committed state inside real callbacks. [Current retained proof](https://github.com/Osso/wow-ui-sim/blob/183edc381/data/patch-api/evidence/ui-widget-discrete-progress-2026-10-13/README.md#retail-and-ptr-current-proof) separates actual Retail/PTR seven-case Discrete GREEN, five preservation cases per profile, fresh normal startup and default check. Source/root compatibility policy is not catalog, rendered/native/tutorial or full-parent acceptance; bounded GREEN text-retention audit PASS at `a6723200be40145ec352666b538998bf4035a608`, scoped by current packet; original RED and other packets not rechecked.

### DiscreteProgressSteps input-only checkpoint — 2026-10-13

[Discrete contract](../../specs/ui-widget-discrete-progress-steps.md) owns the complete19-field supplied-record/type19 slice. Historical `42f206f66` stage added DTO, empty keyed map and five direct-seeded query/security/actual-cache cases only. Getter, registration and typed feed were absent at that stage; initial compilation/RED were unrun. Later genuine RED and producer source are recorded above. Initial supplied-record assertions precede cache/atlas prerequisites; downstream lifecycle assertions are not evidence until reached. Existing `eyeofthejailer` atlas metadata grounds fixture names, not asset extraction/render/native parity. No vendor or demo catalog changes.

### TextWithState input-only checkpoint — 2026-10-13

Historical input-only stage `64a2a57b2` is superseded by producer `2bb6cdde1`. The [contract](../../specs/ui-widget-text-with-state.md) owns complete21-field projection, existing numeric/secret validation, Retail/PTR publication and type8 feeds. [Bounded saved GREEN](https://github.com/Osso/wow-ui-sim/blob/183edc381/data/patch-api/evidence/ui-widget-text-with-state-2026-10-13/README.md#bounded-saved-green) owns independent Retail/PTR supplied-record and actual cached lifecycle PASS, separate startup/check and selected publication integrity PASS. Historical RED/source-gate FAIL remain retained; native/render/full-parent acceptance remains unproved.

### SpellDisplay input-only checkpoint — 2026-10-13

[Current bounded gate](https://github.com/Osso/wow-ui-sim/blob/183edc381/data/patch-api/evidence/ui-widget-spell-display-2026-10-13/README.md#current-bounded-runtime-gate--2026-10-13) owns independent Retail PASS, PTR independently accepted and separate retention PASS. [Spell contract](../../specs/ui-widget-spell-display.md) and [removal contract](../../specs/frame-event-removal.md) own requirements. Following input-stage checkpoint is historical, not current status; native/world-loot/render/full-parent unproved.

At `85b5494d2`, the [SpellDisplay contract](../../specs/ui-widget-spell-display.md) owns passive supplied DTOs, environment-local storage, type13 feed and nine tests. [Retail result-boundary RED](https://github.com/Osso/wow-ui-sim/blob/183edc381/data/patch-api/evidence/ui-widget-spell-display-2026-10-13/README.md) is independently verified; selected-copy integrity and PTR RED remain pending; getter/query/serializer registration and GREEN/consumer acceptance remain absent. Parent OPEN.

### BulletTextList input-only checkpoint — 2026-10-13

[Owning contract](../../specs/ui-widget-bullet-text-list.md) records supplied type10 fifteen-field records, empty environment-local backing and the existing typed feed extension. Authored direct/snapshot/security/callback/unchanged-cache lifecycle tests remain unexecuted; getter projection is deliberately absent until genuine supplied-record RED. No new seeds or native/render/full-parent acceptance.

### BulletTextList producer checkpoint — 2026-10-13

[Owning contract](../../specs/ui-widget-bullet-text-list.md) retains actual Retail/PTR supplied-record/callback RED and the independent Retail/PTR split between six feature failures and one seeded-set fixture defect. The native getter now authenticates numeric selectors and projects complete cloned records into rooted fresh row/line arrays. Corrected rejection tests and GREEN remain pending; no native/render/full-parent acceptance.

### Bullet / dressability runtime checkpoint

[Whole host records](https://github.com/Osso/wow-ui-sim/blob/183edc381/data/patch-api/evidence/bullet-dress-host-records-2026-10-13/README.md) retain the Retail/PTR supplied-model matrix, historical fixture failures, complete compatibility diagnostics and separate startup/check/format gates. Public-copy and retained-gate copy checks pass at their recorded scopes; native/render and parent acceptance remain open. Owning contracts: [BulletTextList](../../specs/ui-widget-bullet-text-list.md) and [item dressability](../../specs/item-dressability.md).

### DoubleStatusBar / TextureAndText scaffold checkpoint — 2026-10-14

Owning contracts: [DoubleStatusBar](../../specs/ui-widget-double-status-bar.md) and [TextureAndText](../../specs/ui-widget-texture-and-text.md). Source `2b13a7c57` supplies explicit 26/19-field DTOs; `bd48d6ce8` adds initially empty environment-local maps and type3/type12 typed-feed storage. Those scaffold revisions did not implement getter projection/serialization or modeled registration; `8645aaf0a` adds full26/19-field projections and `a62c81768` registers both. Shared numeric selector policy is reused without new validation/defaults. Bounded Retail/PTR producer, lifecycle and secret-wrapper behavior independently verified; separate Retail owning sweep and final format/default-check gates also pass. No inferred catalog or demo seeds were added.

The generic namespace `__index` in `runtime_surface_bootstrap.lua:64–79` fabricates and caches a function returning nil; namespace attachment is at lines112–129. Thus initial callable/nil getters are not evidence of modeled getters or native availability. Private saved-outcome verification (`widget-two-api-initial-outcome-current`) confirms four distinct Retail PASS only: absent IDs and actual cached-provider skipping with no visualization. No genuine RED or populated behavior was observed.

`fb7a1c243` authors twelve cases (six per module). Genuine observed Retail RED remains **4 primary PASS / 8 FAIL**, independently classified; later lifecycle assertions were unreached. Initial four PASS were existing behavior, not RED. Private receipt `runtime-two-widget-populated-red` remains preserved.

Bounded saved first GREEN at `a62c81768` (`widget-two-retail-first-green-current`) independently confirms **15 primary PASS: twelve two-widget cases plus three existing lifecycle controls**. Complete supplied snapshots, isolation, selectors and actual cached activation/update/hide/restore/removal/unregister pass. Requested owning sweep used the wrong integration target: zero executed cases, zero coverage credit.

Current bounded Retail/PTR runtime, secret-wrapper behavior, normal diagnostics and final format/default-check proof are recorded in the owning contracts above:17 primary PASS per profile (widget14 including security2 + controls3), separate Retail owning sweep1, no PTR owning-sweep credit. Shared registration is not native/earlier-profile availability. Historical first GREEN15 and zero-launch failures remain separate; no all-seven/render/native/full-parent closure.

[8.2.0 owner](../../specs/patch-8-2-0-publication-sweep.md#widget-producer-source-checkpoint--2026-10-14) retains known publication-array reconciliation **53→52, row135 only**. Row134 was unchanged at that checkpoint; its later owning observation is linked below. DoubleStatusBar has zero evidenced publication rows, none invented.

### TextWithSubtext scaffold checkpoint — 2026-10-14

The [manager checkpoint](../../specs/ui-widget-manager-state.md#textwithsubtext-scaffold-checkpoint--2026-10-14) records the complete27-field DTO, empty environment-owned map and normal type25 feed at `66514bf90` / `39a454d09`. Those revisions contain no modeled getter or projection, and have no populated runtime or compile proof. Generic callable/nil fallback is not modeled availability. Completed two-widget evidence above does not cover TextWithSubtext.

### TextWithSubtext producer checkpoint — 2026-10-14

[Manager checkpoint](../../specs/ui-widget-manager-state.md#textwithsubtext-producer-checkpoint--2026-10-14) and [owning contract](../../specs/ui-widget-text-with-subtext.md) record source `e37f5426a` and independently accepted preproduction Retail RED: control1 PASS / six populated-getter and one selector FAIL. Consumer stopped before cached setup in that RED. The [current bounded manager gate](../../specs/ui-widget-manager-state.md#current-bounded-textwithsubtext-verification--2026-10-14) now records independent Retail/PTR11 each, complete supplied queries and actual cached two-line lifecycle, normal-startup limits and format/default check. Acquisition tag is retained during ordinary setup and refreshed on reacquisition; no model/vendor patch. Historical scaffold/failures remain; [owning publication](../../specs/patch-10-0-2-publication-sweep.md#owning-after-checkpoint--2026-10-14) and broader acceptance stay separate.

### TextureAndTextRow passive checkpoint — 2026-10-14

[Owning row contract](../../specs/texture-and-text-row.md) records `89dca152d`: complete supplied 18-field DTO with two-field ordered entries, initially empty environment-local map and public type15 typed feed. Only fixedWidth is nullable. Eight authored cases include live row/nested-pool unregister; modeled getter/projector and behavioral RED/GREEN remain unproved; source audit and no-run compilation do not prove behavior. Callable nil compatibility is not populated API or native catalog evidence.

### TextureAndTextRow current bounded checkpoint — 2026-10-14

[Owning row contract](../../specs/texture-and-text-row.md) is the single-source contract and detailed proof ledger; the passive checkpoint above preserves historical input-stage wording. Getter source `398c427fd` projects complete fresh snapshots through shared registration. Independent source/readability gate PASS separately covers that producer and diagnostic-only `46f33b1c8`, which distinguishes lifecycle phases without changing feeds or expected behavior.

Genuine pre-projector Retail RED at `89dca152d` remains **1 PASS / 7 FAIL**; ambiguous historical container-flush failure does not prove injection or activation. Independent saved Retail runtime at `46f33b1c8` confirms **8/8 unique PASS, zero FAIL, exit 0**; nested child summaries are nonadditive. Bounded evidence reaches actual cached normal-feed activation/update/hidden release/reacquisition/membership removal/live unregister, fresh nested snapshots and isolation, exact fractional/required-number selectors, and actual host-secret denial/preservation/secure recovery. No substituted vendor handlers; bounded-out texture kits, animations, effects and timers gain no coverage.

Three unchanged Retail sibling controls independently PASS separately from the eight row cases. Historical PTR CPU/memory-pressure admission denial and successful no-run compile remain retained. Later independent saved PTR first-runtime gate at `7f44eb773` confirms **8/8 unique PASS, zero FAIL, exit 0**, including actual profile-cached consumer lifecycle/live nested-pool unregister and host-secret behavior. Nested child summaries are nonadditive; earlier Retail8+3 is separate, not added to PTR totals.

[Owning 8.2.0 observation](../../specs/patch-8-2-0-publication-sweep.md#row134-owning-observation--2026-10-14) owns independent final saved owning-after PASS: one primary case, exact equality of all193 before/after objects, and exact51-gap fixture reconciliation. Historical before FAIL remains retained; runtime behavior is unchanged. Publication presence is separate from the bounded row behavioral gates, not broader publication acceptance.

The [row ledger](../../specs/texture-and-text-row.md#known-gaps-current-cycle) owns separate independent Retail normal-before/PTR normal diagnostic PASS, PTR feature/cache-selection semantics, non-cold compile evidence, and final format/default-check PASS with six warnings and the high-threshold crossing retained. Source LuaErrors collection suppresses startup-event stderr; complete saved logs cannot recover suppressed messages or establish universal startup-clean behavior. Read-only WTF/layout-cache dependencies persist despite SavedVariables disabled. Compile is not runtime proof. Public [whole-TEXT packet](https://github.com/Osso/wow-ui-sim/blob/183edc381/data/patch-api/evidence/ui-widget-texture-and-text-row-2026-10-14/README.md) is committed at `33cbb6baee0e8665770d1d678062eb37b92e6f09`; the row ledger owns counts, accepted-candidate byte equality, opaque `.txt` payload semantics and the Retail stderr metadata correction. Candidate/canonical public-copy integrity, bounded privacy and generated navigation independently PASS. Publication adds no code/runtime proof. Native/render parity, all-profile/broad/provider/performance/credit acceptance, cache-byte authentication, independent executable-byte authentication and full-parent closure remain unproved.

### Spacer passive-input checkpoint — 2026-10-14

[Spacer contract](../../specs/ui-widget-spacer.md) preserves the historical passive-input and selector-RED checkpoint.

### Spacer bounded producer checkpoint — 2026-10-14

[Owning implementation/proof ledger](../../specs/ui-widget-spacer.md#current-implementation-and-proof) records current modeled projection and independently verified profile-scoped behavior; remaining acceptance and provenance limits live there.

### TextColumnRow passive checkpoint — 2026-10-14

[Owning contract](../../specs/ui-widget-text-column-row.md) retains supplied records/type-21 feeds and historical eight-case populated-result RED at `9a60ded79`, six guard aborts and seventh linker failure. Getter/projector/registration implement fresh complete nested snapshots.

### TextColumnRow bounded Retail proof — 2026-10-15

[Capability/proof matrix](../../specs/ui-widget-text-column-row.md#bounded-retail-producer-proof--2026-10-15) owns independent `dcd2fbd12` Retail eight-case PASS and separate nine-control PASS, nonadditive nested output, actual cached nested-pool lifecycle and exact exclusions. Compile/copy PASS retains one external dependency warning; pressure guard FAILED unit-prefix validation before monitoring. Scoped source/root-format PASS is separate. No new Spacer owning proof, PTR/default-check/ordinary-startup/owning/native/render/full-parent acceptance or frozen-packet update.

### DoubleIconAndText passive scaffold and selector RED — 2026-10-15

[Owning contract](../../specs/ui-widget-double-icon-and-text.md) is the field/type and behavior SSOT. Source `4cb2b5e98` adds an explicit supplied record, initially empty environment-local map and type5 normal typed-feed storage. This passive scaffold is **UNCOMPILED / UNEXECUTED** at this checkpoint: no modeled projection or getter registration, demo seeds or native defaults. Callable/nil compatibility is not modeled getter evidence.

The first selector regression actually executed at `a56e11267`: **one unique case FAIL, exit101**, after environment setup and the existing manager required-number control passed. The missing-selector target call succeeded instead of rejecting; the failing assertion does not independently establish its returned value. Later target error-message assertions were unreached. Duplicated nested libtest summaries are parent/child propagation of the same case, not two cases. Independent saved audit: `/home/osso/.local/state/wow-ui-sim/verification/doubleicon-selector-next/runtime-red-report.md`. Main's private erratum `/home/osso/.local/state/wow-ui-sim/handoff/doubleicon-selector-red-main-disposition.md` corrects the report's permission annotation: decimal33216 is a regular file with mode0700, not0440; no execution-permission failure occurred. Frozen report and RED streams remain unchanged.

Selector RED is not populated-result, isolation or cached-lifecycle acceptance. Broader C_UIWidgetManager lifecycle, popup, performance and full-parent acceptance remain OPEN; no native/render/profile parity follows. Later authored or integrated tests gain no execution credit from this checkpoint.

### DoubleIconAndText producer source after populated RED — 2026-10-15

[Owning contract](../../specs/ui-widget-double-icon-and-text.md) inventories getter/registration source `c3096cd6b`: all 19 explicit supplied fields, shared `read_numeric_id`, record clone before allocation, rooted fresh Lua result and one nil for absent IDs. No defaults or vendor changes. At that source-only checkpoint producer execution was pending; later bounded Retail acceptance is recorded below, not inferred from inspection.

The separate source1571253e9 Retail baseline executed nine unique cases: **one unknown-ID control PASS / eight FAIL, exit101** (seven supplied-table failures, one invalid-selector failure). Cached lifecycle was unreached before consumer loading. The owning contract links the exact private epoch; independent saved-runtime reconciliation confirms all nine identities, full streams and build/seal receipts. Authored assertions target projection, isolation, feeds and cached lifecycle; none gains producer acceptance from this RED. This RED alone closes no behavior checkbox. Later bounded acceptance follows; whole-parent acceptance remains OPEN.

### DoubleIconAndText bounded Retail producer proof — 2026-10-15

[Owning acceptance matrix](../../specs/ui-widget-double-icon-and-text.md#bounded-retail-producer-acceptance--2026-10-15) records getter `c3096cd6b` executed at saved source `5dfa94330`: **nine unique PASS, exit0**, with independent build/seal/full-stream runtime reconciliation PASS. Complete supplied snapshots, selector/actual-secret policy, feed ordering/atomic rejection and the unchanged cached type5 lifecycle all executed, including live unregister and later inactive feed. Nested parent/child appearances are not additional cases. Historical selector/populated REDs remain separate. Remaining profile preservation, format/check, ordinary startup and full-parent gates stay OPEN; no native/render parity or unrelated current-HEAD/C3 proof.

## IconTextAndBackground producer source — 2026-10-16

[Owning contract](../../specs/ui-widget-icon-text-and-background.md#bounded-retail-producer-proof--2026-10-16) records independently accepted combined Retail/PTR supplied-record requirements: ten cases per profile (the same ten logical cases), actual cached type4 lifecycle and secret access, plus separate clean normal startups with original CVars unchanged. Retail normal source `84da38a3b`, test checkout `13423aae4` and getter `2a422cb86` remain distinct; the [PTR receipt](../../specs/ui-widget-icon-text-and-background.md#bounded-ptr-producer-proof--2026-10-16) retains its own attribution. Historical REDs and separate fmt/default-check proofs are preserved. Standalone common-helper execution is not whole-integration proof; broader/full-parent acceptance remains OPEN. No native/catalog/assets/render parity or activation of projected effects follows; different CPU allocations preclude performance comparison.

## IconTextAndCurrencies passive source and selector RED — 2026-10-16

[Owning contract](../../specs/ui-widget-icon-text-and-currencies.md#bounded-combined-retailptr-acceptance--2026-10-16) owns historical RED, combined bounded behavior acceptance, startup/CVar qualifications and separate source-check proof; broader acceptance stays separate.

## ItemDisplay historical selector RED — 2026-10-16

[Owning selector receipt](../../specs/ui-widget-item-display.md#historical-retail-selector-red--2026-10-16) owns the independently reconciled bounded Retail RED, separate test/library attribution and reached/unreached assertions. At that historical checkpoint producer, consumer and broader acceptance remained open; [bounded Retail acceptance](../../specs/ui-widget-item-display.md#bounded-retail-acceptance--2026-10-16) now records Retail status; [combined acceptance](../../specs/ui-widget-item-display.md#bounded-combined-retailptr-acceptance--2026-10-16) separately records actual PTR.

### ItemDisplay passive supplied-state checkpoint — 2026-10-16

[Owning contract](../../specs/ui-widget-item-display.md#implementation-inventory) inventories source `417f1badd`: complete supplied records, initially empty environment-owned storage, type27 typed-feed storage and public Rust exports. No getter or Lua projection was added. The owner maintains the field schema; unchecked requirements described this historical passive checkpoint, not current combined acceptance.

[Populated fixtures](../../specs/ui-widget-item-display.md#tests-asserting-this-spec) are uncompiled/unexecuted at this checkpoint. Historical selector RED remains separate; no populated, cached-consumer, native, asynchronous, asset/pixel or broader acceptance follows.

### ItemDisplay getter source after populated RED — 2026-10-16

[Owning inventory](../../specs/ui-widget-item-display.md#implementation-inventory) records getter source `64a5576b1`: complete fresh rooted 15-outer/16-nested projection, four `Option` omissions, exactly one nil for absent IDs, shared `read_numeric_id` policy and unconditional namespace registration matching neighbors. Getter was **UNCOMPILED / UNEXECUTED** at this historical source checkpoint, superseding the passive-only source status above, not its historical evidence. [Bounded Retail acceptance](../../specs/ui-widget-item-display.md#bounded-retail-acceptance--2026-10-16) and [combined acceptance](../../specs/ui-widget-item-display.md#bounded-combined-retailptr-acceptance--2026-10-16) now supersede those pending profile statuses.

[Populated Retail receipt](../../specs/ui-widget-item-display.md#populated-retail-red-before-getter--2026-10-16) owns the separate pre-getter compile0 and eight unique FAIL at the populated-result boundary. Later projection, feed, secret and cached-lifecycle assertions were unreached; the owner now links independent saved-evidence reconciliation of the exact pre-getter RED boundary. No GREEN, PTR, native or lifecycle acceptance follows from that pre-getter RED; the later bounded Retail receipt is separate.

## ItemDisplay bounded Retail acceptance — 2026-10-16

[Owning receipt](../../specs/ui-widget-item-display.md#bounded-retail-acceptance--2026-10-16) records accepted saved normal Retail build/seals, ten unique PASS (eight populated plus two selectors), actual cached completed synchronous type27/BaseItem lifecycle and actual secret wrappers. Separate clean startup/CVar preservation and source/schema/readability audit retain their own scope. Historical REDs remain; [combined receipt](../../specs/ui-widget-item-display.md#bounded-combined-retailptr-acceptance--2026-10-16) now owns separate actual PTR acceptance and remaining gates.

## ItemDisplay combined bounded acceptance — 2026-10-16

[Owning SSOT](../../specs/ui-widget-item-display.md#bounded-combined-retailptr-acceptance--2026-10-16) owns separately attributed bounded Retail/PTR behavior acceptance, separate startup/CVar evidence and accepted saved default Retail check/whole-workspace formatting. Historical checkpoints stay historical. PTR/all-profile type-check, native/assets/fonts/pixels, alternate styles/effect/timer activation, async/pending/cancellation races, whole integration and parent completion remain excluded.

## Sources

- [ItemDisplay contract](../../specs/ui-widget-item-display.md) — schema and combined bounded acceptance SSOT.

- [IconTextAndCurrencies contract](../../specs/ui-widget-icon-text-and-currencies.md) — supplied-record contract and owning evidence/gate reconciliation.

- [DoubleIconAndText contract](../../specs/ui-widget-double-icon-and-text.md) — supplied-record/consumer requirements, bounded Retail acceptance matrix and separate historical selector/populated RED boundaries.

- [TextureAndTextRow contract](../../specs/texture-and-text-row.md) — single-source supplied-row contract, historical RED, bounded independent Retail/PTR/source gates, independent final owning/check/format PASS, separate Retail/PTR diagnostic audits and suppression limits; committed public packet with bounded candidate/canonical integrity/privacy/navigation PASS; broader acceptance remains open.

- [TextWithSubtext projection](../../../src/c_api/ui_widget_manager/text_with_subtext.rs) — complete supplied producer; bounded profile evidence and remaining limits owned by manager checkpoint.

- [BulletTextList](../../specs/ui-widget-bullet-text-list.md) — supplied-record contract and revision-scoped proof limits.

- [UIWidgetManager contract](../../specs/ui-widget-manager-state.md) — required behavior, staged tests, and historical RED boundary.
- [Manager module](../../../src/c_api/ui_widget_manager.rs) and [inputs](../../../src/c_api/ui_widget_manager/inputs.rs) — numeric identities, set/membership records, and complete supplied DTOs.
- [Queries](../../../src/c_api/ui_widget_manager/queries.rs), [feed](../../../src/c_api/ui_widget_manager/feed.rs), and [fixtures](../../../src/c_api/ui_widget_manager/fixtures.rs) — snapshot registration, synchronous producers, and bounded demo initialization.
- `src/c_api/ui_widget_manager/bars.rs` and `src/c_api/ui_widget_manager/hud.rs` — implemented producer paths; exact query/consumer proof boundaries remain separate.
- [Task quest lookup](../../../src/lua_api/globals/quest_surface/task_quest.rs) — stored quest/set association lookup.
- [Widget architecture](../../widget-system.md), [loading pipeline](../../addon-loading-pipeline.md), and [events](../../event-system.md) — existing frame, cached Lua consumer, and dispatch boundaries.

## See Also

- [[widget-system]] — ordinary frames are distinct from C_API records.
- [[addon-loading]] — cached Blizzard consumer loading.
- [[event-system]] — synchronous host notification boundary.
- [[lua-api]] — exposed API inventory.
