# Bounded independent proof — 2026-10-09

Scope: read-only `/home/osso/.worktrees/wow-ui-sim-p342-source`, HEAD `4137b59948f6b7b9cd996286f8ea211e2993b3cd`, original `870dc6a12df433f8db208df9292019e9633527d8`. `master80a3fb104` ancestor confirmed (exit 0); start/end porcelain empty. No checkout mutation, delegation, Cargo test/build/check, full suite or integration gate performed. Only `/tmp` report/receipt written.

## Proof coverage

| Claim | Evidence | Limit |
|---|---|---|
| Retained 6/6 targeted proof applicable | All five ledger SHA256 values equal original and HEAD; original/current evidence, test/common, Cargo.toml/Cargo.lock diff exit 0 | Reused original execution, not fresh HEAD execution |
| Runtime changes do not invalidate Wrath scope | Complete original→HEAD src/Cargo.lock diff retained in receipt; BNGetFriendIndex gated client-retail; Collapse/Expand retirement gated retail-12-0-0; happiness remains registered with identical body under not(client-retail). Transitive selected feature closure contains neither retail flag | Static cfg applicability, not default/mists compilation proof |
| 155 factory observations | Independently counted 72 matches, 83 strict mismatches; globals 34/60, widgets 13/5, events 4/4, CVars 21/14; no probe-error/unprobeable kinds | Publication direction, not argument/results/security/native behavior |
| CVar default difference | TargetAutoLock published value/default 0 vs source default 1; 14 absent added registrations, 11 equal published defaults, one unequal; 15 total recorded default differences | Not 83 absent/native APIs |
| Cross-line controls retained | Original GREEN includes retail rejection and retail/Mists foreign successor exclusion plus empty Wrath successor control; byte-identical tests/classifier | No same-line supersession needed: historical 3.4.3 has zero explicit occurrences |
| Retail alias-provider preservation | `run_publication_sweep` now delegates to `run_sweep(env, spec, || read_deprecated_aliases(env))`; provider evaluated at old call position; old classifier/alias parsing untouched. Empty alias map is Wrath-factory-only | Static inspection only; main owns current retail runtime gate |
| Historical source seals | Fresh own validate.py exit 0; all 23 expected SHA256 values checked; original→HEAD evidence diff empty | Source accounting remains historical; two prose rows UNPROVEN |
| Formatting | Fresh cargo fmt --check exit 0 | No compilation evidence |

Measured environment: `WowLuaEnv::new`, `client-wrath`, interface **38001**. NOT native Wrath Classic source TOC **30402**, NOT loaded Blizzard UI/Game. All 18 widget owners construct; native widget identity not established. Nonsense event and removed TWITTER_POST_RESULT both accepted; event existence/emission/payload/retirement unproven. Historical source ledger runtime/native observations remain zero and are separate from factory observations.

## Readability audit (manual; no additional commands)

Changed Rust: factory target plus shared Wrath enum/profile arm and provider extraction. No TODO/FIXME/HACK/XXX or warning suppression markers in either file. No new >3-level nesting, >15 cognitive-complexity estimate, parameter overload or hidden-I/O naming issue identified. Two maintainability findings, not functional counterexamples:

- [LENGTH] `tests/common/publication_sweep.rs:524` — `run_sweep`: non-test body exceeds 30 lines; reads inputs, probes, persists and compares. Mostly inherited body newly extracted. Separately authorized cleanup could split result collection/exact-gap comparison.
- [STATE] `tests/patch_3_4_2_factory.rs:50` — `cvar_defaults_remain_separate_from_publication`: mutable positional count tuple plus loop push. Separately authorized cleanup could name categories and collect differences independently.

No edits recommended as a required parity fix by this report. Main owns integration decisions and default/mists compilation/runtime gates, portable gate and CI.

## Exact fresh command outputs

All commands explicitly used cwd `/home/osso/.worktrees/wow-ui-sim-p342-source` at HEAD above.

1. `PYTHONDONTWRITEBYTECODE=1 python3 data/patch-api/evidence/3.4.2-session-2026-10-09/validate.py`: exit 0; stderr empty; stdout:

```json
{"added": 109, "cache_files": 42, "inventory_occurrences": 155, "native_observations": 0, "prose_limits": 2, "removed": 46, "runtime_observations": 0, "sealed_inputs": 23, "section_counts": {"cvars": 35, "events": 8, "global-api": 94, "widgets": 18}, "signature_occurrences": 0, "source_rows": 208, "statuses": {"UNPROVEN": 157, "metadata-only": 51}, "successor_inventory_occurrences": 0}
```

2. `cargo fmt --check`: exit 0; stdout/stderr empty.

Complete argv, environment, exit, untruncated captured stdout/stderr, stream hashes, original GREEN log/ledger, artifact hashes and seal/hash comparisons: `/tmp/p342-independent-receipt.json`. Original GREEN log is retained as its original combined artifact, not represented as fresh or reconstructed split streams. No unchanged 6/6 rerun. No completion or native parity claim.
