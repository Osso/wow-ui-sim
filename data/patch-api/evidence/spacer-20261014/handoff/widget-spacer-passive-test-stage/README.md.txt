# Private Spacer passive/test stage

Status: source-authored, inert, unapplied, unexecuted. Parent contract: `[PRIVATE_EVIDENCE]/handoff/widget-spacer-background-parent-completion.json`. User-reported canonical checkpoint `220ec`; no Git commands used to authenticate it. Scoped canonical hashes are in `source-contract.json`.

## Delivery

Complete fifteen-field `SpacerRecord`, empty environment-owned `spacers` map proposal, `WidgetVisualization::Spacer` type22 and storage wiring, six authored cases, complete explicit fixtures, and spec. `wiring-edits.json` contains seven exact unique row replacements across three canonical files; none applied. Preserve current Row/Subtext producers. No getter, projector, namespace registration, default DTO, demo seed, or catalog.

Stage paths mirror their proposed repository destination. Metadata files are not repository changes. No integration.rs edit: existing `crates/test-harness/build.rs` discovers top-level test modules and `tests/integration.rs` invokes its macro. `tests/ui_widget_spacer.rs` reuses `tests/widget_two_records/helpers.rs`; shared helper remains canonical and unchanged, not copied or replaced. Snapshot helpers alias only the actual getter.

## Capability / proof matrix

| Authored distinct case | Exact scope | Proof |
|---|---|---|
| `spacer_absent_returns_exactly_one_nil` | Two absent IDs, exact one nil | Source only; unexecuted |
| `spacer_full15_fresh_mutation_whole_replacement_and_environment_isolation` | All fifteen fields, fresh tables, caller mutation, every field changes on replacement, retained snapshots, two environments and reverse replacements | Source only; unexecuted |
| `spacer_normal_feed_before_event_and_atomic_rejection` | Ordinary subscriber reads complete record/membership during synchronous event; replacement; remove retains record; wrong type/missing set preserve maps and event counts | Source only; unexecuted |
| `spacer_required_ordinary_number_and_fraction_selectors` | Shared missing/invalid/ordinary number policy; fractional supplied lookup without truncation | Source only; unexecuted |
| `spacers_actual_secret_selectors_allow_secure_deny_tainted_preserve_state` | Real host secret-number wrappers via existing shared helper; secure present/absent, tainted denial before lookup, public-number access, wrapper/state retention and recovery | Source only; Retail/PTR plus retail-12-1-0 gate; unexecuted |
| `spacer_cached_normal_activation_dimensions_hidden_reacquire_remove_live_unregister` | Actual profile cache/provider/container/template; normal feeds, width/height update, bounds, hidden release, reacquisition, removal and live unregister retain records/membership | Source only; unexecuted |

## Consumer evidence boundaries

Both cached generated documents declare the same fifteen non-nil fields, required numeric widgetID, AllowedWhenUntainted, and one nullable outer record. Both cached Spacer XMLs inherit UIWidgetBaseTemplate and supply template size10x10; tests do not treat template size as API defaults.

Retail Setup uses `Clamp(widgetInfo.widgetWidth, 1, widgetInfo.widgetWidth)` and the analogous height call. Cached Retail `MathUtil.lua` Clamp checks `value > max` before `value < min`; with ordinary finite value and max=value, the former is false and values below1 become1. PTR Setup uses `math.max(value, 1)`. Authored boundaries: normal173.25/41.5, updated206.25/73.75, zero/negative crossed pairs0/-11.5 and-7.25/0, positive-below-one/exact-one crossed pairs. No nonfinite policy invented. Raw query dimensions remain supplied, never clamped in the DTO.

Consumer records use explicit empty kits, false hasTimer, None in/out animation and model layer. Effect ID731.25 is supplied, but None model layer excludes effect activation. All-field query/feed replacement uses valid Fade=1 in both animation fields, Front=1 model layer, timer=true and synthetic kit strings; it is never activated in a cached container. Empty-kit safety is bounded to actual Spacer/base paths, not a universal atlas-helper claim.

Acquisition initializes widgetTag in cached CreateWidget. Ordinary Setup does not replace that tag; getter returns the updated tag while active frame keeps the old one. Hidden release followed by normal shown feed reacquires the new tag. No fake provider, forced cached handler, vendor mutation, timer/animation/effect acceptance, native/render evidence, or full-parent acceptance.

## Hold and ledger

Housing hold20:47:19 remains active. Only parent-authorized filesystem/text operations performed: read, source parsing/hash, private mkdir/chmod/write/fsync/readback. No subprocess/CLI/commands, runner imports, builds, formatter/compiler, tests, simulator runtime, network, delegation, canonical writes, commits, or deployment. Source-authored cases are not RED; compile errors are not behavioral evidence. `proof-ledger.json` records exact proof scope and exclusions. `manifest.json` records final payload hashes and private filesystem readback.
