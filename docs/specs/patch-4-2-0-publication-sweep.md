# Patch 4.2.0 publication accounting

Account for the pinned historical retail page without inventing missing signatures or behavior. [Audit](../wiki/investigations/patch-4-2-0-api-audit.md).

## What it must do

- [x] Retail `BNGetFriendIndex(accountID)` reads the existing ordered Battle.net friend list and tracks reorder/removal; unknown IDs return nil (inferred, native parity unproven).

- [x] Reproduce all 65 inventory IDs and both source counts using recorded existing flags.
- [x] Account for the sole navigation metadata row and zero prose/signatures.
- [x] Probe every inventory occurrence; require the exact retained gap set.
- [ ] Apply actual later retail registers, with explicit pending 4.3.0/4.3.4 placeholders; exclude Classic histories.

## How it works

[Source boundary and publication scope](../wiki/investigations/patch-4-2-0-api-audit.md).

## Implementation inventory

- `data/patch-api/sources/4.2.0-*` — pinned inventory, provenance, extract and per-ID ledger.
- `tests/patch_4_2_0_publication_sweep.rs` — own retail prefork case.
- `tests/patch_4_2_0_behavior.rs` — concrete friend-list mutation tests.
- `src/lua_api/globals/real/bnet_friend_index.rs` — modeled legacy global, retail-only registration via `real/mod.rs` and `globals/register.rs`.
- `tests/data/patch_4_2_0_sweep_known_gaps.json` — exact development-observed mismatch set.
- `data/patch-api/evidence/4.2.0-session-2026-10-09/` — compact source and development receipts.

## Tests asserting this spec

Own prefork filter `patch_4_2_0_publication_sweep`; `tools/test_patch_4_2_0_accounting.py`; integration filter `patch_4_2_0_friend_index`.

## Known gaps (current cycle)

- [ ] Reconcile pending 4.3.0/4.3.4 successor registers; 31 exact retained publication gaps remain.

## Out of scope

Native behavior parity, invented linked-page signatures, Classic changes, new shims/fallbacks, vendor changes, broad/final acceptance, push/merge/deploy/delegation.
