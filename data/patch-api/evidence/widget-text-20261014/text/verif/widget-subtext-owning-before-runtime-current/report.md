# Owning-before runtime verification

**FAIL — expected-gap-set mismatch, exit 1. Exact observed delta confirmed.**

Source: `62f115d262613a848180143d6b30e2a0b3eb63b0`. Saved artifacts: `<private-state>/verification/widget-discrete-progress-current/widget-subtext-retail-owning-before-20261014T182701792315Z/runtime-owning-10-0-2-before-gap-reconcile`. Verification read complete stdout (4,437 bytes), stderr (24,714 bytes), and all 416 JSON rows. Full logs, JSON and exact sets retained in `proof.json`; originals unchanged.

| Evidence | Result |
|---|---|
| Owning normal Retail prefork case | 0 passed, 1 failed, 1 total; exit 1 |
| Auxiliary chat/cast-bar/spellbook groups | Three zero-case summaries; nonadditive |
| Publication rows | 416 unique archived-register IDs; 268 OK, 148 non-OK |
| Archived expected gaps | 149 unique IDs; unexpected non-OK: 0 |
| Only missing expected gap | `wt-global-api-C_UIWidgetManager.GetTextWithSubtextWidgetVisualizationInfo-309` |

Target row: `ok=true`, `kind=member`, `detail="raw=function; lookup=function"`, expected published, no supersession. Stderr left/right sets exactly match all 148 observed / 149 expected IDs. Failure location: `tests/common/publication_sweep.rs:315:5`. No permission/setup denial: startup reaches FIRST_FRAME_RENDERED and test-body gap assertion.

## Invocation, setup and process

Exact argv retained in proof. Sealed `prefork_full_ui` executable receives `patch_10_0_2_publication_sweep --nocapture --test-threads=1`; default Retail features and prefork target match reused receipt. Archived `tests/prefork_full_ui.rs:301-350` selects normal full-UI setup, invokes conformance then actual normal preload. Conformance implementation at lines 495-518 runs same executable with `PREFORK_CONFORMANCE_SUITE=1`, requires success and discards successful output; individual conformance counts unavailable. This is not a direct conformance-mode invocation or bare-factory sweep.

Preload loads discovered Blizzard Game startup addons/bootstrap nodes through profile cache resolver and fires normal startup events. Parent bytecode-cache bypass and child read-only mode are explicit. Default cache root: `<private-cache>/wow-ui-sim/blizzard-ui/retail/AddOns`; historical inherited XDG_CACHE_HOME/full environment not captured, so absolute root and cache bytes are not independently authenticated. No legacy Interface/BlizzardUI runtime path.

Read-only source mounted onto canonical path; runtime output directory explicitly writable-bound. `P1002_SWEEP_OUT` points to complete publication JSON outside read-only source. Saved launcher: exit-code, status 1/FAILURE, 8.391s, 1.4G peak. Current systemd observation: failed/exit-code, ExecMainStatus=1, MainPID=0, empty ControlGroup.

## Authentication and boundaries

Exact auth receipt/seal reused: SHA-256 `100f6cd42d60d966f4ef074afbcfb5fb8453cb35a2a4f49f433cee1cdaa9da6d`. Producer hash reads: 1; auth receipt hash reads: 1; verifier binary reads/hash reads: 0. Current stat equals before/after auth and runtime reused stat. No build/test/rerun/delegation or canonical edits. Receipt authenticates executable only, not complete source/dependencies/cache/settings.

Separate saved cached-behavior evidence: Retail 11/11, exit 0 (9.46s); PTR 11/11, exit 0 (11.95s), both source62. Each has eleven nested one-case summaries plus one outer eleven-case summary: **not 22 cases**. They do not turn this failing publication sweep green or establish native WoW parity. Publication alone proves neither behavioral output nor security semantics.

Parent may reconcile **only** exact expected-gap array by removing row309 following this actual observed delta. No whole-parent completion claim.
