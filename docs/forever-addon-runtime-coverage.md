# Downloaded Forever addon runtime coverage

Overall compatibility remains **unverified**. The [per-project matrix](../data/forever-addon-audit/runtime-coverage.json) covers all 269 projects in the [cached archive inventory](../data/forever-addon-audit/cached-comparisons.json), not just previously selected candidates. The earlier [comparison audit](forever-addon-comparison.md) records static evidence; it is not runtime acceptance.

## Inventory — 2026-09-22

All 268 successfully downloaded Forever packages exist and match their indexed sizes and SHA-256 hashes. ConsumableTracker's selected Forever file `8924598` is unavailable; its older comparison archive is not silently substituted. Missing cached dependencies remain separate blockers. No further downloads are authorized by this audit.

The matrix separates current startup and interaction results from `priorEvidence`. A `not-run` current result does not erase an earlier scoped proof: it means that proof has not yet been reconciled or rerun for this inventory-wide pass. Current bounded batches add Abattis PugBoard open/close, Abgesattelt DB/event initialization, three clean startup observations, one intentional LoadOnDemand-not-requested module, one profile-excluded package, and one concrete ActionBarAuras startup failure. Ellesmere's existing six-group proof is retained. EpicDamageMeter also has historical bounded interaction proof; CooldownMaster and DragonGuildMaster have startup-only evidence. Carbonite and Baganator have recorded failures or dependency blockers.

## Baseline isolation correction — 2026-09-22

All 268 available packages were attempted in package-only batches. Those runs omitted `XDG_DATA_HOME`: `--no-saved-vars` and isolated WTF paths do not isolate simulator CVar overrides. Addons wrote to shared `/home/osso/.local/share/wow-sim/cvars.json`; enabling combo-point UI exposed missing `GetComboPoints`, after which unrelated packages repeated the native error. A no-addons shared-data control fails; the same control with fresh `XDG_DATA_HOME` returns `[]`.

The polluted baseline remains historical evidence only. The corrected harness uses per-run data storage and explicit cached-provider composition. Cross-process tests prove writer persistence, independent reader defaults, and an unchanged shared host CVar hash. The post-batch shared CVar file is snapshotted, not reset: no exact pre-batch snapshot exists, so original values cannot be claimed restored.

## Isolated startup checkpoint

At binary build `c1e830ffa`, all 268 available project entries have dispositions: 161 clean startups, 32 runtime failures, 72 unloaded/partial loads, and three unresolved dependency cases. The unavailable archive is the 269th entry. Every isolated batch preserved the shared host CVar hash. These are startup observations, not full compatibility acceptance; 14 of the 72 unloaded cases contain some successfully loaded roots, and intentionally excluded or deferred modules must not be mistaken for failures.

`GetComboPoints` and whitespace enum/configuration regressions pass 6/6 and 1/1 respectively; format/default compile checks pass. Classic UI Forever now starts without its prior combo-point failure. The later `55ee20d7` build passes formatter-focused tests (native 7/7, configuration 7/7, binding 11/11, numeric 8/8). ActionBarAuras advances past its formatter/container-initialization failures but still fails: `C_Spell.GetBaseSpell` returns nil for its action spells. Its real duration-text workflow remains unproven. Full per-addon major workflows remain mostly untested.

A separate DBM-GUI replay at `55ee20d7` observes metadata, dependency and LoD status before execution, then explicitly loads the module and observes its table/options frame with zero Lua errors. This is loader evidence only—not DBM encounter, timer, alert or rendered-GUI acceptance. The replay requires the explicit out-of-date interface switch; its ledger is `/tmp/forever-addon-runtime/dbm-lod-native-e4hvi488/ledger.json`.

## Explicit out-of-date replay of unloaded packages

Replayed exactly the 72 previously unloaded/partial projects with `WOW_SIM_LOAD_OUT_OF_DATE_ADDONS=1` and the `55ee20d7` binary, which includes deferred-addon metadata registration. Result: 48 clean startups, 12 runtime failures, 12 still unloaded. Per-run data/WTF isolation preserved the shared host CVar hash. Default-interface attempts remain in each project's `priorDefaultInterfaceAttempt`.

The latest per-project matrix therefore totals **209 clean startups, 44 failed, 12 unloaded, three dependency-blocked, one unavailable archive**. These mixed-revision/configuration observations are not a single current-build certification. Batches: `/tmp/forever-addon-runtime/unloaded-ood-55ee20d7-batch0.json` and `batch1.json`.

Ellesmere's expanded startup replay exposes a taint error in `AuraKit.RunJob`; an identical isolated replay with frozen `c1e830ffa` reproduces the same 56 occurrences. Thus the error is not introduced by the new formatter/LoD commits. Prior six-workflow evidence remains scoped historical proof, not acceptance of this expanded configuration.

## Cached LibStub composition and bounded library interactions

C_Everywhere `8912546`, CustomSearch `8912549`, and CustomTutorials `8912550` start with the unchanged, TOC-bearing LibStub subtree from cached DBM `8925922`. The explicit `WOW_SIM_LOAD_OUT_OF_DATE_ADDONS=1` switch is required by these observations; it does not waive other loader restrictions. Original dependency-only attempts remain in the matrix. Provider provenance: `/tmp/forever-addon-audit/cached-libstub-subtree-provenance.json`.

At the frozen `c1e830ffa` binary, isolated interaction probes observed:

| Package | Covered behavior | Still untested |
| --- | --- | --- |
| CustomSearch | 16 positive/negative accent, AND/OR/NOT, tag and numeric query checks | Consumer integration; native-client comparison |
| CustomTutorials | Progress 1→2→3, callback delivery, already-seen no-op, reset/hide, retrigger1 | Button clicks, rendered appearance, images/shine, restart persistence |
| C_Everywhere | CVar namespace/call parity; occupied item6948 stack3, five empty slots, removal | Legacy scalar packing; remaining namespaces |

Each probe returned its completion marker and zero recorded Lua errors; shared host CVars remained unchanged. Fixtures live in `tools/forever-addon-fixtures/`; exact commands, binary hashes, and outputs are linked per project. These add bounded evidence, not whole-addon certification. The matrix now records five bounded interaction passes, one initialization-only result, and 263 not-run entries; historical evidence remains separate.

## Acceptance discipline

- Verify that intended addon roots actually load; empty error JSON with nothing loaded is not a pass.
- Separate clean startup, LoadOnDemand/excluded roots, missing dependencies, runtime failures, and incomplete observations.
- Define addon-specific major actions and assert resulting state. Merely finding a slash command, frame, or library is not interaction proof.
- Preserve exact package hashes, simulator revision/binary identity, commands, complete outputs, and proof scope. Reuse prior evidence only when relevant behavior is unchanged.
- Keep goal open while required projects or major workflows fail or remain untested. No full native-conformance claim follows from bounded simulator tests.

Initial archive proof: `/tmp/forever-addon-audit/selected-archive-hash-proof.json`. Prior runtime evidence reconciliation: `/tmp/forever-addon-audit/runtime-proof-inventory.json`. Detailed per-run artifacts remain under `/tmp/forever-addon-runtime/`; the tracked matrix records their paths and dispositions.
