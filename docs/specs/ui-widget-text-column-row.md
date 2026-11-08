# Supplied TextColumnRow visualization

`C_UIWidgetManager.GetTextColumnRowVisualizationInfo` is the cached Retail/PTR type-21 query. [Manager architecture](../wiki/systems/ui-widget-manager-state.md) owns state, feeds and consumer loading.

## Required behavior

- Accept complete explicitly supplied outer/nested records. No inferred fields, defaults, catalogs or demo seeds. Keep `text_column_rows` initially empty and environment-local; empty entries differ from an absent record.
- Preserve existing numeric-key policy and shared required-number/security boundary. Return one nil for an absent record, otherwise complete fresh outer/entries/entry tables. Caller mutation, replacement or deletion must not alter retained snapshots or another environment.
- Use normal type-21 feeds with existing type/set validation and state-before-synchronous-event semantics. Rejected updates leave records, membership and notifications unchanged. Membership removal retains independent visualization data.
- Exercise actual profile-cache registration, activation/update, nested entry styling/layout, Hidden release, Shown reacquisition, membership removal and live unregister through normal feeds/OnUpdate. No substitute provider, private handler invocation or vendor mutation.

## Cached declarations

Both Retail/PTR `Blizzard_APIDocumentationGenerated/UIWidgetManagerDocumentation.lua` declare these required, non-nil fields. Numeric/String Rust representations follow simulator conventions, not native ABI proof. No global field-count or declaration-order contract is introduced.

| Outer field | Declared type |
| --- | --- |
| `shownState` | `WidgetShownState` |
| `entries` | table of `TextColumnRowEntryInfo` |
| `textSizeType` | `UIWidgetTextSizeType` |
| `fontType` | `UIWidgetFontType` |
| `tooltip` | string |
| `tooltipLoc` | `UIWidgetTooltipLocation` |
| `bottomPadding` | number |
| `widgetSizeSetting` | number |
| `textureKit` | textureKit |
| `frameTextureKit` | textureKit |
| `hasTimer` | bool |
| `orderIndex` | number |
| `widgetTag` | string |
| `inAnimType` | `WidgetAnimationType` |
| `outAnimType` | `WidgetAnimationType` |
| `widgetScale` | `UIWidgetScale` |
| `layoutDirection` | `UIWidgetLayoutDirection` |
| `modelSceneLayer` | `UIWidgetModelSceneLayer` |
| `scriptedAnimationEffectID` | number |

| Entry field | Declared type |
| --- | --- |
| `text` | string |
| `enabledState` | `WidgetEnabledState` |
| `hAlign` | `WidgetTextHorizontalAlignmentType` |
| `columnWidth` | number |

## Current source checkpoint — 2026-10-14

Complete record structs, empty environment-owned map, normal type-21 storage routing and the getter/projector/registration are implemented under `src/c_api/ui_widget_manager/`. The getter uses the shared required-number/security selector and creates independent outer, entries and nested-entry tables without inferred data. The later bounded Retail producer proof is recorded below; generic callable/nil compatibility is not modeled-query evidence.

`tests/ui_widget_text_column_row.rs` discovers six base cases under Retail/PTR plus numeric/real-host-secret selector supplements; the secret case additionally requires `retail-12-1-0`. Base cases cover populated projection/deep freshness; replacement/empty entries/deletion/isolation; numeric keys; first-feed callback state and removal; atomic rejection; actual cached nested-pool lifecycle. Supplements use distinct integer/fraction records through normal feeds, complete recursive public snapshots, actual host-secret NUMBER wrappers, secure allowance/tainted denial/recovery and unchanged backing/membership. Shared helper assertions do not substitute a getter or API. Fixtures and observational Lua assertions live in `tests/widget_text_column/`. Each case owns its environment; populated preconditions are not a global test-order rule. First-feed setup removes its query seed before publication.

Nonincremental Retail integration compilation at `9a60ded79` completed with an independently verified copied executable. The eight-case selection then exited 101: all eight fail at the populated supplied-record query because the returned value is not a table. The consumer's nested child failure is not an additional ninth case. This establishes the missing populated-result boundary, not later lifecycle/security behavior. At that RED checkpoint, producer GREEN and consumer tag/style/reset assertions remained unproved. Query-only texture/effect/timer values are synthetic; consumer records neutralize them. Source/readability inspection is separate from compilation and behavior.

## Saved nonbehavioral follow-up — 2026-10-14

Root `cargo fmt --check` at `c66bfa263` passes with empty complete stdout/stderr. The earlier 512M formatter attempt timed out without a formatter result; the corrective 2G attempt passes. This proves formatting only, not compilation or behavior. Later `ed9fdba0e` changes six manifest Clippy-key spellings, preserving values, dependencies and Rust sources; one offline/locked/no-deps metadata invocation passes without those warnings. Historical warning streams remain unchanged; metadata does not prove compiler/linter cleanliness.

Six earlier TextColumn Retail integration `--no-run` attempts ended in guard-issued SIGTERM without Cargo `build-finished` or a completed integration artifact: the four at `c66bfa263` (host full PSI avg10 twice, host full PSI avg60, then worker full PSI avg10), followed by the corrected worker-PSI attempt and the host-reserve attempt. These are incomplete compilations, not compiler-error failures or behavioral RED. The earlier owning compile aborts remain separate history in the [Spacer contract](ui-widget-spacer.md).

A later Retail integration `--no-run` attempt at `055b57afd` reached linking and failed through clang/mold with 223 undefined-symbol diagnostics referencing the root simulator rlib. The saved terminal reports Cargo exit 101; the Cargo message stream ends with `build-finished` success false. No type/import/feature diagnostics were observed in that stream. This establishes partial source compilation reaching link failure, not compiler-warning cleanliness, a completed test executable, runtime behavior or populated API RED. At that failed-link checkpoint, all eight authored cases remained unexecuted and getter/projector/registration remained absent. Incremental-artifact involvement is only a hypothesis; the root cause is not established. Full raw diagnostics remain privately retained. Neither existing frozen historical public packet includes these new attempts.

The subsequent nonincremental compile/copy gate passes at `9a60ded79`: complete mixed stdout contains 742 Cargo JSON records and 2,654 verbose build-script lines; terminal exit 0 and `build-finished` success true establish compilation. A JSON-only parser failure was framing, not Cargo failure. One external `iced_widget@0.14.2` deprecated `std::f32::INFINITY` warning remains. This is not warning-clean or proof of default incremental/linker repair. The independent compile/copy report remains privately retained, not published in the historical packets. The later eight-case RED is retained separately under that build epoch's `runtime-text-column-eight-red-20261015T045002981007Z/`. Frozen public packets do not include these new results.

Saved guard checks establish bounded pure threshold decisions, not unconditional malformed-input fail-closed handling, successful live stopping or machine responsiveness. Resource observations do not establish workload causality.

The separate [frozen historical71 packet](https://github.com/Osso/wow-ui-sim/blob/183edc381/data/patch-api/evidence/spacer-text-column-late-text-2026-10-14/README.md) retains the third Spacer abort, formatter attempts and source-only TextColumn/shared-doc reports. Independent private exactness/privacy review passes after preserving earlier privacy FAIL25/FAIL4; independent canonical verification at `055b57afd` passes 71/71 exact payload comparisons, catalog/owner link, commit scope and bounded 73-file privacy review. It excludes newer compile attempts and current-source acceptance.

## Bounded Retail producer proof — 2026-10-15

Independent saved-stream review passes the `dcd2fbd12` Retail eight-case producer selection and separate nine-case shared-control selection, both exit 0, zero failures/ignored. Parent summaries are 8 PASS / 11081 filtered (2.68s) and 9 PASS / 11080 filtered (2.34s). Nested exact-child output is nonadditive: 17 distinct selected cases, not 20 and not full-parent acceptance. Complete stderr contains only informational environment-startup milestones; this is not ordinary startup diagnostics proof. Raw artifacts remain private; frozen historical packets are unchanged.

| Capability | Bounded proof |
| --- | --- |
| Complete supplied projection and snapshots | Three query cases PASS: all 19 outer/four entry fields, fresh nested tables, destructive caller-mutation isolation, whole replacement, empty arrays, deletion, separate environments, fractional/large/signed-zero keys and absent one-nil results. Empty-array query proof is not empty-entry consumer lifecycle proof. |
| Normal type-21 publication and rejection | Two feed cases PASS: first/replacement state visible inside synchronous events, membership removal retaining independent data; wrong type/unknown set leave records, membership and notifications unchanged. |
| Shared selector/security policy | Two supplements PASS: distinct integer/fraction normally-fed records; real host-secret NUMBER secure allowance, tainted denial before lookup and secure recovery, recursive public/nonalias snapshots and unchanged backing/sets. Simulator caller contract, not native parity. |
| Actual cached consumer | One case PASS: normal registration/activation, three-to-two nested pool update with text/font/color/alignment/width/layout assertions, Hidden release, Shown reacquisition, membership removal retaining query data and live unregister followed by feed without reacquisition. Incremental frame tag remains acquisition tag; restored acquisition receives updated tag. No restored-frame identity assertion or TextColumn-specific post-unregister container-hidden/zero-showing assertion. |
| Separate shared controls | Nine PASS: six manager-state cases (absence, feed atomicity, IconAndText freshness, exact quest mapping, set metadata, ordered listings), manager secret/security case, cached IconAndText/ButtonHeader lifecycle case and TextureAndText numeric-selector case. Not duplicate TextColumn coverage or new Spacer owning proof. |

The producer's separate nonincremental compile/copy gate passes, with one preserved external `iced_widget@0.14.2` deprecated `std::f32::INFINITY` warning. The pressure watcher **FAILED before monitoring**: unit-prefix validation rejected the producer unit. No successful continuous reserve/PSI/max/OOM guard operation is established; compile/runtime success does not repair that gap or prove default incremental/linker repair. Existing copied-executable receipt is reused, not new source/archive/external cache authentication. Scoped source/readability and root-format gates PASS separately; neither expands runtime coverage.

Consumer texture/frame kits, timer, animations, scripted effect and model layer are neutralized; nonneutral fields are query-tested only. Widget-state assertions are not rendered/native comparison. PTR execution, ordinary owning observation/publication reconciliation and full-parent acceptance remain pending. Earlier eight-case RED, six compile aborts, seventh linker failure and frozen packets remain historical evidence, not overwritten by this GREEN.

### Default-feature check and ordinary startup

Independent review passes the later default-feature Retail `cargo check --offline --locked --message-format=json -j 1` at `576555c67` (tracked code unchanged from `dcd2fbd12`): exit 0, 733 JSON records, zero compiler diagnostics. `CARGO_INCREMENTAL=0` was explicit; this is not default incremental linker-repair evidence. Its first attempt remains incomplete after guard SIGTERM15 at the provisional host-reserve boundary. Corrected-prefix guards actually sampled both check attempts; retry completion does not establish unconditional guard safety or erase the producer build's earlier monitoring failure.

The ordinary Retail `lua-errors` run independently exits 0 with `[]` plus LF and zero collected Lua errors; all 219 stderr lines were inspected. Loader warning/nil/requirement totals remain unknown in that invocation because diagnostics are gated and event-stage stderr is suppressed. It does not replace the separate 17-case behavior proof.

A separate verbose ordinary `dump-tree` invocation retains event-stage diagnostics: 291 selected addon success summaries report zero loader warnings, **185 nil observations and 88 strict missing-requirement records**, plus one intentional `EDIT_MODE_LAYOUTS_UPDATED` skip. Those are per-load deduplicated records, not necessarily globally unique symbols or 88 demonstrated call failures. Exact full/bootstrap and omitted-addon/file counts remain unknown. The raw TextColumn member reports function/C and an absent one-nil result; no populated state was supplied in this probe. Normal startup has no complete requirements-clean PASS. Font logs report `casc=true` despite the explicit `WOW_SIM_CASC=0` environment; actual extraction is not established. Inherited read-only WTF/cache and finite update windows remain limits. This dump uses different settling/layout/probe work from `lua-errors`, so their timings are not a performance comparison.

## Bounded PTR execution — 2026-10-18 UTC

Lane `textcolumn-ptr` executed the committed draft without changing assertions, providers, vendor files or simulator behavior. The first producer selection passed 8/8 at `a94dca4d3` on base `78577edc0`. After rebasing onto `f452d5a4b`, code revision `8e1100ad6` passed the same eight producer cases, two new PTR cases and nine separate shared controls. All selections exited 0, with no ignored, extra or cfg-missing selected cases; nested timeout-child executions are nonadditive. No missing-behavior RED or test-defect RED was observed, so no implementation fix was justified.

| Boundary | Bounded PTR proof at `8e1100ad6` |
| --- | --- |
| Supplied query, feeds, numeric/security selectors | Eight existing producer cases PASS, including actual host-secret numbers, deep isolated snapshots and the original cached type-21 consumer. |
| Original PTR lifecycle | New closure-loaded case PASS: registration, activation, three-to-two entry update/style/layout, Hidden release, Shown reacquisition, membership removal and live unregister. Container-hidden/zero-showing assertions pass immediately after unregister and after subsequent feed plus normal OnUpdate, without row/nested-entry reacquisition. |
| Owning observation | New ordinary Game discovery/bootstrap/startup case PASS, followed by the same populated original consumer and unchanged 179-row 9.1.0 sweep/later-register chain. Sweep has 131 successful rows and exactly the existing 48 known gaps; TextColumn publication row is successful (`raw=function; lookup=function`). `graphicsComputeEffects` (4 versus 1) and `RAIDVolumeFog` (1 versus 0) default mismatches remain recorded. This is expected-gap equality, not 179 implemented behaviors. |
| Separate preservation | Exact nine shared manager/lifecycle/security/TextureAndText controls PASS; these are not additional TextColumn producer cases. |

Commands used `--no-default-features --features sound,gui,casc,client-ptr`, slot-2 and async 9-GiB systemd units admitted only on capacity ADD. Separately built `wow-sim --no-saved-vars lua-errors` exited 0; stdout is byte-exact `[]` plus LF, matching `docs/baselines/ptr-lua-errors.json` (SHA256 `37517e5f3dc66819f61f5a7bb8ace1921282415f10551d2defa5c3eb0985b570`). Runtime used a child-only byte-copy of current persisted CVars; original stat/hash remained unchanged. Stderr retains unavailable-audio-device diagnostics. Collector success does not establish loader nil/strict-requirement cleanliness, rendered/native parity or full-parent acceptance.

Private exact argv, revision-scoped outputs, selected names/counts, hashes and receipts remain at `/home/osso/.local/state/wow-ui-sim/handoff/textcolumn-ptr-run/`. Historical Retail failures and evidence packets remain unchanged.

## Remaining proof

- [x] Compile tests and observe populated result-boundary RED before getter implementation: eight unique Retail cases fail querying explicitly supplied records; deeper assertions remain unreached.
- [x] Independently verify bounded Retail projection, selector/security, deep snapshots, feeds and actual cached lifecycle: eight producer cases plus nine separate controls.
- [x] Complete the scoped default-feature check and capture both limited collector and verbose ordinary Retail diagnostics; preserve the observed nil/requirement gaps and skip.
- [ ] Reconcile remaining PTR ordinary-startup nil/strict-requirement diagnostics; original cached lifecycle and the baseline-matched Lua-error collector are proven above, not complete requirements cleanliness.
- [x] Observe applicable 9.1.0 publication after ordinary PTR owning startup and populated consumer; retain known gaps, historical failures and broader acceptance gaps.

Native catalogs/publishers, native interpolation/security/return parity, rendered output, empty-entry consumer lifecycle and nonneutral texture/effect/timer activation are not established by this slice. Other profiles and full-parent acceptance remain separately scoped.
