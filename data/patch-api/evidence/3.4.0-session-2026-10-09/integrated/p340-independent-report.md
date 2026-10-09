# Bounded 3.4.0 independent source proof — 2026-10-09

Read-only `/home/osso/.worktrees/wow-ui-sim-p340-source` at HEAD `9b932d3f89d249d3c1e6a17b3ad0e1284bea84f5`; original `74b101115`. Parent `4e3eee3b85fc6a459ca141b17f784b3a442069d8` ancestor confirmed. Initial/final porcelain empty.

| Claim | Evidence | Limit |
|---|---|---|
| Retained 8/8 applicable | Historical GREEN eight serialized cases OK; all 21 seal hashes equal expected/original/HEAD; scoped evidence/source diff exit 0 | Reused historical proof, no unchanged fixture rerun |
| Source rows retained | Fresh own validator exit 0: 331 inventory, four summaries, 386 source rows; 335 UNPROVEN, 51 metadata-only | Literal source/model-contract accounting |
| Signatures bounded | 266 signature limits: 265 identity-only callable occurrences plus one partial UnitAura contract; zero complete signatures | No argument/result parity |
| Runtime boundary | Runtime/native observations zero; configured Wrath38001 distinct from source30400 | No factory, loadedUI, runtime or native closure |

Inspected own validator/tests, GREEN/source-proof and historical model context. Tests consume serialized owned inputs and frozen historical tools, not current Rust/UI. Retained omission/tamper controls are applicable by unchanged bytes; not executed again. Historical queue/context revisions remain historical.

Precise UNPROVEN contracts:

- `_Wrath` and `-WOTLKC` support retained as literal source contract. Historical loader candidate omission of `-WOTLKC` is static context only; no execution/native support or precedence credit.
- `UnitAura.shouldConsolidate` position remains **immediately before variable returns**. Numeric slot, type and value derivation remain null; full arguments/returns remain null. No inferred boolean or tuple position.
- Retail9.2.5 subset/UnitPopup details unexpanded; CURSOR_UPDATE→CURSOR_CHANGED does not establish triggers/payload/dispatch.

Fresh explicit-cwd command:
`PYTHONDONTWRITEBYTECODE=1 python3 data/patch-api/evidence/3.4.0-session-2026-10-09/validate.py`

Exit 0; stderr empty; complete stdout:

```json
{"cvar_defaults": 18, "directions": {"added": 300, "removed": 31}, "explicit_signatures": 0, "header_counts": [{"direction": "added", "header_count": 228, "parsed_count": 228, "section": "global-api"}, {"direction": "removed", "header_count": 27, "parsed_count": 27, "section": "global-api"}, {"direction": "added", "header_count": 10, "parsed_count": 10, "section": "widgets"}, {"direction": "removed", "header_count": 0, "parsed_count": 0, "section": "widgets"}, {"direction": "added", "header_count": 43, "parsed_count": 43, "section": "events"}, {"direction": "removed", "header_count": 3, "parsed_count": 3, "section": "events"}, {"direction": "added", "header_count": 19, "parsed_count": 19, "section": "cvars"}, {"direction": "removed", "header_count": 1, "parsed_count": 1, "section": "cvars"}], "inventory_occurrences": 331, "kinds": {"command": 1, "cvars": 19, "events": 46, "global-api": 255, "widgets": 10}, "native_observations": 0, "partial_signatures": 1, "prose_limits": 4, "removal_occurrences": 31, "runtime_observations": 0, "sealed_inputs": 21, "signature_limits": 266, "source_rows": 386, "statuses": {"UNPROVEN": 335, "metadata-only": 51}}
```

Complete revision, argv/cwd/env/exit/full stdout/stderr/hashes, original blob read outputs, three-way seals and retained GREEN/source-proof: `/tmp/p340-independent-receipt.json`.

No edits, Bash, cwd switch, delegation, commits/push/merge, Cargo or broad/portable fixtures/suites. Only `/tmp` reports written. Main owns final integrated portable gate. No completion/native parity claim.
