# Bounded independent verification — PASS

Requested scope: canonical `/home/osso/Projects/wow/wow-ui-sim`, `ce40cfb899bcc342457fc1cd03c2b23b26969406`. This is NOT an overall handoff/final acceptance gate.

## Coverage matrix

| Claim | Result | Concrete proof / remaining boundary |
|---|---|---|
| Integration identity | PASS | `merge-integration.stdout`: merge `3b3769d3b` has parent `29ed6fc8a`; merge `ce40cfb89` has parent `fba1c90cc`. |
| 1.14.1 frozen SOURCE | PASS | Copied fixture 8/8, 353 omission controls; 114 nonblank rows, 59 inventory occurrences, 59 unspecified signatures. Exact malformed `FontStringSetTextScale` retained, not aliased. |
| 1.14.0 frozen SOURCE | PASS | Copied fixture 10/10; 776 nonblank rows, 723 inventory occurrences; literal malformed identities, script label, foreign-baseline boundary retained. |
| 1.14.1 seals | PASS | 160 original + 10 receipt entries all match; original map SHA256 `9918f7611435b9f3a5711e8b4259b333e25627e3f1fb65c16b1b024f9d0e5a78`. |
| 1.14.0 seals | PASS | 92 original + 5 receipt entries all match; original map SHA256 `95ba4fff0eeff9b31cec31c9fb5a912a132ab0bbcde00d3f59ecbc402f9bb797`. |
| Copied portable replay | PASS | 3/3 each, exit 0; 161/93 archive members. Both ledger omission and fabricated log rejected, restored byte-identically, seals revalidated. |
| Historical defaults | PASS | 1.14.1 register/extractor byte-identical. 1.14.0 expected default generator exit 1 retained (`no symbol reference in: ': Scripts'`); existing `--skip-plain-scripts-label` register and default extractor exact. Expected negative controls are not verification failures. |
| Existing getter state | PASS, bounded | Standalone `patch_1_14_1_text_scale`: 1 passed, 0 failed, exit 0. First FontString 1.5→2.25; second remains 0.75. No native/signature/default/alias inference. |
| Actual same-Era successors | PASS, input inspection only | Canonical ledgers and pinned raw/response identities exist for 1.14.1–4 and 1.15.0–9; pins/raw match frozen successor inputs. Frozen queued/in-flight metadata was NOT rewritten into actual current state. |
| Native, loaded UI/security, full historical compatibility | UNPROVEN | 59 / 723 full inventory contracts remain unresolved. No linked foreign-history bodies, imported member contracts, native signatures/defaults, security parity or final acceptance credit. |

## Replay execution and isolation

Read original proof ledgers/current receipts before commands. Copied `test_portable.py` and archive into this directory, then executed scripts unchanged. Both literal source-worktree CWD directories existed. **No in-memory path adjustment**, no file patching. Top-level CLI CWD was always the explicit canonical absolute path; nested scripts retain explicit absolute source-worktree CWD. Each archive extracts into fresh temporary scratch; no copied Git, target or current tools, child PATH empty, `PYTHONDONTWRITEBYTECODE=1`.

`1.14.1-portable.json` and `1.14.0-portable.json` contain exact argv, CWD, UTC start/end and exit status. Each version's `portable-receipts.json` retains automatically captured child stdout/stderr and original argv/CWD. `annotated-receipts.json` adds enclosing UTC time bounds, revision/hash scope and environment overrides; original scripts do not record individual child timestamps, so none were invented. Separate `child-*.stdout/.stderr` retain full child output. Effective relevant inherited environment is in `replay-effective-environment.json`.

## Targeted standalone test

Only originating 1.14.1 current-proof receipt existed; independent Era-helper evidence covers other targets, not this getter. Source/runtime bytes unchanged since the originating proof, but no exact independent getter execution was credited. Therefore ran exactly once:

```
cargo test --offline --locked --manifest-path /home/osso/Projects/wow/wow-ui-sim/Cargo.toml --no-default-features --features client-era --test patch_1_14_1_text_scale -- --nocapture
```

CWD `/home/osso/Projects/wow/wow-ui-sim`; no environment overrides. UTC `2026-10-09T16:12:31.189848+00:00` through `2026-10-09T16:12:31.809814+00:00`; exit 0, 1/1, 0.09s test time. Full output: `text-scale.stdout`, `text-scale.stderr`. Six existing library warnings, one existing binary warning and six vendor manifest deprecations remain; no warning-free claim. Scope hashes: `runtime-scope-before.json`; relevant inherited environment: `runtime-inherited-env.json`.

## Actual successors, separately inspected

`actual-successors.json` retains current canonical paths/hashes, source pins, raw bytes, counts, frozen comparisons and limits. Actual 1.14.2 has 36 rows/7 inventory; 1.14.3 has 463 rows/412 inventory. 1.14.4 and 1.15.0–9 contain their own literal linkage/prose/API/metadata limits, not wholesale foreign-history compatibility. `successor-literal-comparison.json` compares exact API template identities only; it does not normalize aliases, infer removal, import linked bodies or claim behavioral supersession. This inspection gives zero foreign-history credit and does not apply successor semantics to historical ledgers.

## Preservation and concurrent checkout drift

All own evidence files, snapshots, original seal maps, receipt maps and archives are unchanged byte-for-byte (`before-evidence-hashes.json`, `after-evidence-hashes.json`, `preservation.json`). No edits by verifier inside repo, no delegation, operations, model/runtime changes, full suite or check runs. Initial and final status both `?? .code-index.db` (pre-existing).

Canonical HEAD advanced externally during verification to `045e396b0c6f717c2d962b97cdf46b736a3328ce`. Full delta retained in `concurrent-all-names.stdout`; code delta is an unrelated client-wowforever-gated test entry and its new target, plus other evidence/docs integrations. All `src`, pre-existing tests, own frozen evidence and lockfile bytes remain unchanged. Runtime pre-command Cargo hash matches requested `ce40cfb89`. Post-scope hash comparison detected Cargo change explicitly rather than silently asserting fixed HEAD. No rerun: change does not intersect the selected headless Era getter behavior. Do not read this report as overall acceptance at the newer HEAD. See `concurrent-scope-note.json` and final revision/status receipts.

## Receipt limitations

Initial git status/revision capture was displayed automatically, but evaluation-local variables were unavailable later; `initial.json` records their exact observed output manually and discloses this. All subsequent CLI output is automatically captured to files. Individual nested replay timestamps are unavailable in unchanged originals; enclosing bounds retained. No suppressed failure or fabricated timestamps.
