# Method Dispatch Refactor

Refactor of frame method dispatch to fix runtime pollution (wrong-type methods exposed) and align `__index` lookup order with real WoW behavior.

## Current State (Fixed)

Verified by source inspection on 2026-10-09: `env_init/frames.rs` registers method groups on a base frame metatable, then builds a shallow shared `__index` without `__*` keys. `methods/frame_metatable.rs` clones that index into cached per-widget metatables, excluding explicitly named scroll/message methods and `SetStatusBarAtlas` for StatusBar. That inspected revision gave WorldFrame the base metatable; `b003ec183` now gives it a separately cached clone, described below. No `is_method_allowed` registry or positive per-type allowlist exists.

`methods.rs::attach_frame_metatable` attaches the selected metatable to each Lua frame table. Its filtered `__index` therefore participates in ordinary lookup, not just introspection. Shared registration does not prove all-type dispatch. These inspected files do not support the former introspection-only filter claim; no invocation tests or native parity proof were run for this correction.

The reported historical **491-row delta across 31 names** and the **11,985-row baseline file total** measure different scopes, not contradictory totals. The historical delta was not independently reproduced here. Neither count alone justifies snapshot replacement, a native allowlist, or a new publication contract.

## Current Investigation — Saved469 and Alpha Receiver (2026-10-10)

Saved suite revision `469a92d27` (not 469 tests) remains **FAIL**; parent goal **OPEN**. Whole extra panic lines 6871–7463 report 96 removals and 491 additions. The earlier lines 6874–6957 excerpt omitted 12 removals and all 491 additions; it was not complete row coverage. The 96 removals are six retired minimap setters × 16 widget types (`SetBlipTexture`, `SetCorpsePOIArrowTexture`, `SetIconTexture`, `SetPOIArrowTexture`, `SetPlayerTexture`, `SetStaticPOIArrowTexture`). Lines 7494–7518 identify five stale missing-snapshot rows: FontString `GetAlphaGradient`, `GetScaleAnimationMode`, `SetScaleAnimationMode`, `SetVertexColorFromBoolean`, and Texture `SetVertexColorFromBoolean`; plus 16 newly missing `SetPreventSecretValues` rows. “Remove stale entries” means those five keys are now exposed, not that they remain absent; line 7495 specifically names **FontString**, not Frame.

Epoch gates intentionally retire the six minimap setters from `retail-12-0-7` and `SetPreventSecretValues` from `retail-12-0-5`. Main source inspection places the minimap gates in `map_frames.rs:62–83`, epoch lookup checks in `tests/global_frames.rs:605–615`, and absence checks in `tests/patch_12_0_7_removed_native_surface.rs:165–168`. Do not restore retired methods or treat snapshot synchronization as a semantic repair. Valid native Minimap invocation on the current epoch is not proven: the setters are absent there. Native discovery README date **2026-02-23** supplies no build/profile identity; snapshot consistency is not native parity.

**Independent saved-capture audit PASS, bounded scope:** diagnostic `20261011T023000Z` (October 10 local work date) reused a sealed normal Retail artifact. Six FontString states assert exactly two values; Frame lookup returned a function and explicit invocation returned `true, 0, 0` (Frame arity not asserted); plain-table `pcall` returned false. Exit 0, unique completion marker and trailing JSON `[]` were verified. Getter resolves frame backing and generic accessibility, without FontString-specific validation. Native receiver compatibility remains unknown; this is not a semantic repair. Shared source representation is `Val::Table`, not userdata. Unsupported 3D, saved suite FAIL and parent OPEN remain unchanged.

[Prior behavioral PASS lines](https://github.com/Osso/wow-ui-sim/blob/183edc381/data/patch-api/evidence/narrow-repairs-2026-10-09/integrated/method-alpha-receiver/saved469-behavior-excerpts.txt) reuse eight scale-mode/vertex-boolean tests plus two minimap-absence tests already inside saved469 totals. They prove simulator stored-state/arity/isolation and lookup absence, not animation, rendering, native coercion or new executions. Setter retirement and FontString gradient controls also passed at saved469 lines8158/8101. No tests were replayed to classify these deltas.

Privacy custody incident: a read-only agent altered private stderr/submission. Main preserved its sanitized copies and restored originals: stderr matches the original pre-mutation SHA; submission fields and canonical serialization match the verifier’s pre-incident report, but no original submission byte hash existed. [Audit](https://github.com/Osso/wow-ui-sim/blob/183edc381/data/patch-api/evidence/narrow-repairs-2026-10-09/integrated/method-alpha-receiver/independent-report.md) discloses this limit, non-hermetic EditMode/cache inputs, endpoint-only seals and requested—not independently proven effective—resource limits.

[Method/alpha/receiver packet](https://github.com/Osso/wow-ui-sim/tree/183edc381/data/patch-api/evidence/narrow-repairs-2026-10-09/integrated/method-alpha-receiver) retains stdout, privacy-redacted stderr, probe and small receipts/hashes; no binary or private identity is copied. [Saved469 evidence](https://github.com/Osso/wow-ui-sim/blob/183edc381/data/patch-api/evidence/narrow-repairs-2026-10-09/integrated/fullsuite-469/evidence.json) records original raw-log location and suite failure boundaries.

## Complete Method Result Files — 2026-10-11 UTC

Correct complete comparison counts: **12,380 extra + 33 missing = 12,413 rows**, from original **11,985 extra + 22 missing = 12,007**. Extra delta is **96 removals + 491 additions**; missing delta is **five removals + 16 additions**: **608 changed rows**. Native discovery's **625 names / 3,522 type pairs / 16 types** is a different scope from comparison rows.

Main4528's interim **11,901 extra + 33 missing = 11,934** reconstruction and 105-change completeness claim are incorrect: the bounded excerpt omitted 12 extra removals and all 491 extra additions. [Older packet](https://github.com/Osso/wow-ui-sim/tree/183edc381/data/patch-api/evidence/narrow-repairs-2026-10-09/integrated/method-full-snapshots) and older receipts remain preserved as superseded, not full PASS. Fresh original assertions `030002` report extra **FAIL101** (12 removals + 491 additions), missing **PASS0**. Prior **21/21 documentation gate** proved bounded-excerpt coverage, not full-row coverage; independent diagnostic023000's six FontString states remain PASS within their separate scope. Earlier generation025508 was resource-blocked across five admissions/zero executions; that receipt is not replaced.

[Corrected retained packet](https://github.com/Osso/wow-ui-sim/tree/183edc381/data/patch-api/evidence/narrow-repairs-2026-10-09/integrated/method-full-snapshots-complete) includes full original/complete files, complete saved469 panic and interim FAIL receipts. Full generator `030142` completed **one named helper case, exit0**, independently audited, producing **12,380 extra + 33 missing = 12,413 rows**; original discovery and source artifact seal unchanged. Main annotated the older packet as superseded. Generation is not assertion proof. Corrected extra assertion `030409` passes after fifth-attempt resource admission; unchanged missing assertion reuses `030002` PASS without replay. Independent audit covers both original assertions: one fresh plus one reused saved case, not two new executions. No runtime/native behavior repair follows from these counts. Native epoch/parity unknown; original saved469 FAIL and parent OPEN remain.

## Cross-profile Comparisons — 2026-10-11 UTC

[Retained packet](https://github.com/Osso/wow-ui-sim/tree/183edc381/data/patch-api/evidence/narrow-repairs-2026-10-09/integrated/method-profile-comparisons) (`7d4019472`): saved `031711` **runtime capture audit PASS**, separately from comparator outcomes **1 PASS / 11 FAIL across 12 actual cases**. Complete diagnostic parsing and all 12 full profile comparison files are retained; independent retention audit PASS:64hashes/50exactcopies/six sanitized receipts/12argv records. Capture PASS does not change comparator FAIL outcomes.

| Profile | Extra assertion / full rows | Missing assertion / full rows |
|---|---|---|
| PTR | FAIL / 12,412 | PASS / 33 |
| Wrath, Mists, Era, Anniversary — each | FAIL / 11,633 | FAIL / 38 |
| Forever | FAIL / 11,825 | FAIL / 38 |

Prior Retail's two covered PASS cases—fresh extra `030409` and saved missing `030002`—are excluded from these 12. Source inputs stayed unchanged; no rebuild, cached Blizzard UI load or acquisition occurred. Shared default-Retail expected files are **not a native/profile oracle**: these are fixture disagreements, not 11 runtime implementation bugs or all-profile PASS. Main SHA256 comparison confirms both complete files are byte-identical across the four classic builds; this is method-set equality only, not behavioral/native profile equivalence.

PTR's 32 extra additions are `SetRoundLayoutToNearestPixel` and `GetRoundLayoutToNearestPixel` × 16 enumerated types. Actual [source gate](../../../src/lua_api/frame/methods/core_state/mod.rs) is `retail-12-1-5` **OR** `client-wowforever`; the [versioned pixel-rounding contract](../../specs/pixel-layout-rounding.md) links native PTR proof limited to Frame/Texture/FontString, not all 16 types and not merely source-only evidence for those three. Enumeration does not prove dispatch or native semantics. Native build/product remains unknown; source lanes **KEEP BLOCKED**, historical full suite **FAIL**, parent **OPEN**.

## MovieFrame Receiver and Return Arity — 2026-10-11 UTC

`EnableSubtitles` appears among the491extra exposure rows, but its body requires `object_type_name == Some("MovieFrame")`. Existing saved469 test7781 passes valid XML-frame state transitions/isolation and ordinary-Frame/bad-argument rejection; nil-expression checks alone did not prove zero returns. [New arity packet](https://github.com/Osso/wow-ui-sim/tree/183edc381/data/patch-api/evidence/narrow-repairs-2026-10-09/integrated/movie-subtitle-arity) independently passes four exact-zero-return calls on two Lua-created MovieFrames and two explicit receiver rejections, plus public hidden-state preservation. One CLI capture, not six test executions; native receiver/default/playback semantics unproved. See [subtitle contract](../../specs/movie-subtitles.md). Exposure, invocation, state and return arity are separate coverage levels; unsupported3D and full handoff remain unchanged.

## Texture receiver containment — 2026-10-12

`b003ec183` filters `GetFileWidth`/`GetFileHeight` from every non-Texture index before existing widget-specific exclusions. WorldFrame now uses the same clone/cache mechanism with distinct key `WorldFrame` (its reported type name is Frame), preserving all other base-index entries. Frame values remain native-backed Lua tables, not userdata; index enumeration and method invocation are separate proof levels.

Main reports the preceding receiver-stage method comparison **1 PASS / 1 FAIL**, with **32 file-getter extra rows**. The default comparison fixture now retains only the two intended Texture extras; no native capture was modified. Current Texture7/VertexColor7/method2 bounded PASS; graph consistency is not native parity. Linked integrated SSOT owns startup, pending check and unrun gates; packet retention awaits main publication. [Texture contract](../../specs/texture-file-dimensions.md) owns metadata/receiver gaps; [integrated checkpoint](integrated-source-and-factory-proof-2026-10-09.md#five-producer-and-receiver-source-checkpoint--2026-10-12) owns stage reconciliation. Earlier historical comparison totals remain historical, not a refreshed complete inventory.

## Current Source Representation

- Shared registration/index construction: `src/lua_api/env_init/frames.rs`
- Named per-widget exclusions/index cloning: `src/lua_api/methods/frame_metatable.rs`
- Frame-table metatable attachment: `src/lua_api/methods.rs`

## Historical Target Architecture

```
__index(ud, key):
  1. mixin_overrides[frame_id][key]    — Mixin() shadows
  2. children_keys[key]                — child frame lookup
  3. custom_fields[frame_id][key]      — script handlers, properties
  4. rust_dispatch(widget_type, key)   — direct Rust match, no Lua table
```

Step 4 resolves methods directly from widget type + name via a `HashMap<(WidgetType, &str), mlua::Function>`. No intermediate Lua table, no shared-method fallback.

## Historical Proposed Work (Not Current Requirements)

- Trim `diff_methods_extra.txt` (types still exposing too many methods in discovery)
- Add diff-driven coverage so unintentional method-surface changes fail tests
- Optionally collapse runtime dispatch and metatable exposure onto one direct Rust lookup

## Sources

- [frame_metatable.rs](../../../src/lua_api/methods/frame_metatable.rs) — named exclusions and cached metatable clones
- [frames.rs](../../../src/lua_api/env_init/frames.rs) — shared registration and method-index construction
- [methods.rs](../../../src/lua_api/methods.rs) — attaches the selected metatable to frame tables
- [method-dispatch-refactor.md](../../method-dispatch-refactor.md) — historical design

## See Also

- [[editmode-layout]] — regression caused by the Lua-fields-first `__index` change
- [[global-frame-index]] — related `__index` on `_G` for lazy frame lookup
