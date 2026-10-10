# Independent artifact verification — 2026-10-10

**Completion: PASS. Errorless full-third-party startup: FAIL.** Exit 0 and the completion marker coexist with Lua errors. This report inspects existing artifacts only; no simulator rerun, build, operational change, delegation, or commit was performed.

## Receipt and byte association

Current receipt: `/home/osso/.local/state/wow-ui-sim/verification/full-addon-startup-current/20261010T173544Z`.

Original source/build/normal-startup receipt: `/home/osso/.local/state/wow-ui-sim/verification/count-api-startup-current/20261010T172719Z`.

- Original submission revision: `9635468d994be53d1f3571a8005115a49864c18a`. Full-addon submission revision and independently read current HEAD: `d6f767b11b11a072a3bc3e8a1dd3395e7c4d97df`.
- Both submissions identify the same sealed executable: `/home/osso/.local/state/wow-ui-sim/verification/count-api-startup-current/20261010T172719Z/wow-sim-sealed`. Independently rehashed SHA-256: `c9dd3209fbc3c10e8c40dc930cdf3890e8725b4bbe7a0ff8704fe0168956c552`; matches both artifact records. Full-addon outcome records `artifact_unchanged: true`.
- Original `source-before.json` equals `source-after.json`. Independently checked all 3,853 recorded paths against current on-disk SHA-256: **0 changed, 0 missing**. Scope: 1,653 `src/` files, 2,187 tests, `.cargo/config.toml`, Cargo manifest/lock, `build.rs`, seven Blizzard manifests and two listfile inputs. Current tracked `src/` plus root Cargo/build files have **0 paths absent from that snapshot**. Revision movement therefore does not invalidate this recorded code-byte scope.
- Original `compile.stdout` contains the `wow-sim` compiler-artifact record with `fresh: true`, executable `/home/osso/Projects/wow/wow-ui-sim/target/debug/wow-sim`, default/client-retail features, and `build-finished: success=true`. Original compile outcome is exit 0; this full-addon receipt reused the seal, not a new build. Original normal-startup execution used `--no-addons`, exited 0 and recorded `[CountApiStartupComplete]`; it cannot establish third-party success.

Association is bounded by the original receipt: external dependency/source-to-artifact provenance, untracked files/index, inherited environment, and runtime assets outside fixture inputs were excluded. Neither receipt proves all runtime/addon/cache inputs are unchanged or validates native behavior.

## Complete stream review and actual third-party loading

Inspected complete full-addon streams, including trailing suppression summaries:

| Stream | Bytes | Lines | SHA-256 |
|---|---:|---:|---|
| `startup.stdout` | 2,876 | 49 | `761c9192d17a878d870478e0d36d2731afe4d63c8fd56a257fb7932c5c46171c` |
| `startup.stderr` | 19,870 | 311 | `d9a7e958df61de9744e21c73ceb735db9c23b874caef258b7a00af8b0142e6b9` |

All stream paths above are under `/home/osso/.local/state/wow-ui-sim/verification/full-addon-startup-current/20261010T173544Z/`. Also inspected submission/outcome, request, queue submission and controller stdout; controller stderr is empty (0 bytes).

The recorded command used `timeout 90`, `--no-saved-vars`, the marker probe and filtered `dump-tree`. Request explicitly set `WOW_SIM_NO_SOUND=1` but did **not** record/unset inherited `WOW_SIM_NO_ADDONS`. Absence of `--no-addons` alone is insufficient. Actual loading is independently demonstrated by stdout and stderr:

- Stdout: `Loading 9 Blizzard addon dependencies for third-party addons...`, `Loading 79 addons...`, `Loaded: 49/79 addons` and concrete third-party timings.
- Loading summary: `Failed during loading: 1`; `Load failures: 0`; `Loaded with Lua errors during loading: 1`; `Total: 3008 Lua files, 151 XML files, 4 warnings`. These are separate reported categories, not proof that all 79 loaded or that every nonloaded addon failed.
- Stderr lines 120–136 bracket third-party loading and contain EnhanceQoL errors. AllTheThings fails later during login.
- Warm-cache evidence: Blizzard `1569/1569 hits`; third-party summary `3031/3031 hits`; both report lookup/replay/store failures 0. Cache content provenance is not established.

Sound and SavedVariables loading were disabled. An existing read-only EditMode cache was still read; private cache identity is deliberately omitted.

## Exact errors, wrappers and counts

`startup.stderr` has **7 first-occurrence `Lua error:` headers**, **2 suppression summaries representing 98 additional occurrences**, hence **105 recorded occurrences across 7 displayed error signatures**. Of those, **104 are owner instruction-budget exhaustion** (EnhanceQoL 100; AllTheThings 4); **1 is a nil-call error**. Counts derive from full output, not exit status.

| Stderr line | Exact first-occurrence error header | Occurrences including suppression |
|---:|---|---:|
| 121 | `Lua error: @Interface/AddOns/EnhanceQoL/Settings/GroupTools.lua: Lua error: instruction budget exhausted for owner 'EnhanceQoL'` | 1 |
| 124 | `Lua error: @Interface/AddOns/EnhanceQoL/Modules/Aura/FocusInterruptTracker.lua: Lua error: instruction budget exhausted for owner 'EnhanceQoL'` | 1 |
| 127 | `Lua error: @Interface/AddOns/EnhanceQoL/Modules/Mouse/Init.lua: Lua error: instruction budget exhausted for owner 'EnhanceQoL'` | 1 |
| 130 | `Lua error: @Interface/AddOns/EnhanceQoL/Modules/Food/Init.lua: Lua error: instruction budget exhausted for owner 'EnhanceQoL'` | 1 |
| 133 | `Lua error: [OnEvent] frame=#69142 addon=EnhanceQoL source=@Interface/AddOns/EnhanceQoL/Core/DynamicAnchors.lua:997: instruction budget exhausted for owner 'EnhanceQoL'` | 96 |
| 222 | `Lua error: [OnEvent] frame=#48610 addon=AllTheThings source=@Interface/AddOns/AllTheThings/lib/EventRegistration.lua:26: instruction budget exhausted for owner 'AllTheThings'` | 4 |
| 269 | `Lua error: Interface/AddOns/EllesmereUI/EllesmereUI_UICore.lua:1224: attempt to call a nil value` | 1 |

Every first occurrence and both summaries include exactly this traceback tail:

```text
stack traceback:
	(string):1: in main chunk
```

Stderr line 306 reports `Lua error suppressed 95 additional times:` for the EnhanceQoL OnEvent signature; line 309 reports `Lua error suppressed 3 additional times:` for AllTheThings. Current byte-associated `/home/osso/Projects/wow/wow-ui-sim/src/lua_errors.rs:241–256` formats these as total count minus one, confirming the arithmetic.

There are **4 nested `Lua error:` file-load wrappers**, **2 OnEvent first-occurrence wrappers**, and **0 `[handler-budget-error]` / `[file-budget-error]` diagnostic records**. No measured limit/used-before/used-after counters exist in these streams. Budget exhaustion is the reported cause; these artifacts do not distinguish exhausted callback entry from callback consumption or establish native quota parity.

No fatal/panic/timeout diagnostic was observed in either complete runtime stream. The runtime summary reports **4 warnings**, but neither stream provides four individually identifiable warning messages; their exact contents cannot be recovered from this receipt. Separately, original build stderr contains six deprecated Clippy manifest-key warnings and the aggregate `iced_wgpu` six-warning line; these are original build warnings, not four runtime warning details.

## Root nil call, marker and member shapes

Stderr line 272, after startup events and the EllesmereUI error:

```text
[FullAddonStartupComplete]	nil	table	nil
```

The recorded probe evaluates `type(IsUsingGamepad)`, `type(C_GamePad)`, and `type(C_GamePad and rawget(C_GamePad,'IsEnabled'))`. Observed shapes are therefore: **IsUsingGamepad nil; C_GamePad table; raw IsEnabled member nil**. This does not probe metamethod-resolved `C_GamePad.IsEnabled`, execute either input function, enumerate other raw members, or establish native input semantics.

Current local EllesmereUI source at line 1224 corroborates IsUsingGamepad as the first callee. Its nil shape explains the first-call failure; short-circuit evaluation does not establish that C_GamePad.IsEnabled was reached. This source was not a receipt-hashed runtime asset. No vendor source content or private install identity is retained. Native active-device selection and gamepad enablement semantics remain unknown; no default or repair is inferred.

`outcome.json` records exit 0, completion marker seen and unchanged artifact. Stderr independently shows login/world-enter/post-login/first-frame event progression, the exact marker, subsequent font/layout diagnostics, frame-tree header and final suppression summaries. Completion is supported; errorless startup is explicitly contradicted. No runtime repair or native-parity claim is made. No raw private profile, realm, character, account/cache identity or private keys are reproduced in this report.
