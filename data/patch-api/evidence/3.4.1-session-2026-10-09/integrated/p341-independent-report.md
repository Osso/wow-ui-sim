# Bounded 3.4.1 independent source proof — 2026-10-09

Read-only worktree `/home/osso/.worktrees/wow-ui-sim-p341-source`. HEAD `4e3eee3b85fc6a459ca141b17f784b3a442069d8`; original `61fd58823`; parent `4137b59948f6b7b9cd996286f8ea211e2993b3cd` ancestor check exit 0. Initial/final status clean.

| Claim | Proof | Scope |
|---|---|---|
| Retained 7/7 applicable | Original GREEN log shows seven targeted serialized tests OK. All 19 sealed SHA256 values equal expected/original/rebased HEAD; entire own source/evidence diff exit 0 | Reused execution; no unchanged test rerun |
| Source coverage | Fresh own validate.py exit 0: 333 inventory occurrences, 376 source rows | Literal source accounting only |
| Limits retained | 335 substantive rows UNPROVEN, 41 metadata-only; two prose limits, 211 signature limits, zero explicit signatures | No linked reconstruction or contract credit |
| Profile boundary | Historical profile interface 38001 vs source TOC 30401; runtime/native observations zero | No factory, loaded UI, runtime or native closure |

Inspected validator and all seven test cases. They consume owned serialized response/raw/pin/ledger/plaintext/profile plus frozen historical generator/extractor, not live Rust or cached UI. Sealed dependencies unchanged; unrelated master runtime changes cannot alter these recorded source-accounting assertions. Historical successor queue and historical revision fields remain historical, not descriptions of current integration state.

Fresh command, explicit cwd above, HEAD above:

`PYTHONDONTWRITEBYTECODE=1 python3 data/patch-api/evidence/3.4.1-session-2026-10-09/validate.py`

Exit 0; stderr empty; complete stdout:

```json
{"cvar_defaults": 62, "directions": {"added": 244, "removed": 89}, "explicit_signatures": 0, "header_counts": [{"direction": "added", "header_count": 146, "parsed_count": 146, "section": "global-api"}, {"direction": "removed", "header_count": 65, "parsed_count": 65, "section": "global-api"}, {"direction": "added", "header_count": 35, "parsed_count": 35, "section": "events"}, {"direction": "removed", "header_count": 9, "parsed_count": 9, "section": "events"}, {"direction": "added", "header_count": 63, "parsed_count": 63, "section": "cvars"}, {"direction": "removed", "header_count": 15, "parsed_count": 15, "section": "cvars"}], "inventory_occurrences": 333, "kinds": {"command": 1, "cvars": 77, "events": 44, "global-api": 211}, "native_observations": 0, "prose_limits": 2, "removal_occurrences": 89, "runtime_observations": 0, "sealed_inputs": 19, "signature_limits": 211, "source_rows": 376, "statuses": {"UNPROVEN": 335, "metadata-only": 41}}
```

`/tmp/p341-independent-receipt.json` retains command argv/env/cwd/revision/exit/full captured outputs and stream hashes, all 19 three-way input hashes, original GREEN log and source-proof ledger. Original blob hash reads used `git show 61fd58823:<sealed-path>` in explicit cwd, exit 0 for every path; resulting digests retained. Original GREEN log remains its historical combined artifact.

No edits, commits, pushes, merge, cwd switch, Bash, delegation, Cargo or broad/portable fixtures/suites. Only `/tmp` outputs written. Main owns source-only integration order and final portable gate. No completion/native parity claim.
