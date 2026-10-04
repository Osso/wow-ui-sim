# Retail 12.1.0 publication sweep handoff

## Staged artifacts

Root: `/home/osso-test/.cache/wow-ui-sim-audit/staging/p1210-sweep/`.

- `tests/patch_12_1_0_publication_sweep.rs`
- `tests/data/patch_12_1_0_sweep_known_gaps.json` — initially `[]`.
- `docs/specs/patch-12-1-0-publication-sweep.md`

Author-only handoff. Repository and git state were not modified. No cargo, test, build, simulator, agent or model CLI was run. Static inspection and standalone rustfmt only; compilation/runtime proof remains outstanding. Cargo.toml already supplies serde with derive and serde_json as normal dependencies; no dependency changes needed.

## Integration and run

Integrator copies the three staged files into their matching repository paths when authorized. `build.rs` auto-includes top-level tests in the single integration binary; do not add a separate Cargo test target or alter build.rs.

Filter: `patch_12_1_0_publication_sweep`. From the repository, after memory pressure clears:

```sh
P1210_SWEEP_OUT=/home/osso-test/.cache/wow-ui-sim-audit/p1210-sweep-results.json python3 scripts/build-host.py --build-host local --test --test integration patch_12_1_0_publication_sweep -- --nocapture --test-threads=1
```

Retail default features select the 12.1.0 surface; the module is gated to `client-retail`, not PTR. Ensure the synced retail cache matches the audit's intended source before interpreting results. The cache itself is not native build authentication.

One test creates ONE environment, using the existing `tests/common/prefork_full_ui_preload.rs::preload_full_game_ui` via a path module. It loads the full cached **Game startup** set, fires startup events, restores cleanup globals and applies existing post-load workarounds. It does not inject panel-fixture stubs. Optional LoD/Glue-only publication may remain non-ok; this is startup breadth, not proof that every optional addon is loaded. The helper fails explicitly on load/startup errors; such an infrastructure failure produces no complete sweep and must not become a known-gap baseline.

Full Game startup dominates memory and time; all 778 probes share that load. Exclusive workload gating avoids competing gated test workloads, but does not stop unrelated processes or another build. No measured runtime estimate is available. Use the single filter and one test thread, not the full suite. Existing helper has prefork bytecode-cache lifecycle effects, so use an isolated filtered test process. No UI/compositor or GPU screenshot is needed.

## Exact known-gap mechanism

Each source ID gets `{expected, observed, ok}`. `expected` preserves section/direction/symbol plus the publication expectation. `observed` records kind/detail, CVar value/default and `default_mismatch`. Per-row Lua errors are non-ok observations, not early exits. Output is written BEFORE the exact-set assertion, so the first failing run remains usable.

Only the **set of non-ok IDs** is baselined. A new failure AND an unexpectedly resolved/stale known gap fail the comparison. Baseline membership does not make a row green, and the test cannot write its own known-gap fixture. Malformed inventory, cache startup failure and output I/O failure are infrastructure/schema failures, not accepted gaps.

1. Capture the first run with the empty fixture; expect RED, at least for unsupported RadialProgress and CHAT_MSG_*.
2. Review every non-ok row, especially `probe-error`/`unprobeable`, before accepting the baseline. Do not baseline an incomplete run or a wrong-profile/wrong-cache result.
3. Generate a candidate list with the snippet below, then explicitly copy/review it into `tests/data/patch_12_1_0_sweep_known_gaps.json` and commit with the test/spec.
4. Rerun after integration when authorized. An unchanged accepted gap set passes; individual gap rows still earn no proof credit.

```python
import json
from pathlib import Path

root = Path('/home/osso-test/.cache/wow-ui-sim-audit')
results = json.loads((root / 'p1210-sweep-results.json').read_text())
assert len(results) == 778
assert all(type(row['ok']) is bool for row in results.values())
gaps = sorted(source_id for source_id, row in results.items() if not row['ok'])
candidate = root / 'p1210-sweep-known-gaps-candidate.json'
candidate.write_text(json.dumps(gaps, indent=2) + '\n')
print('Reviewed candidate only:', candidate)
for source_id, row in sorted(results.items()):
    direction = row['expected']['direction']
    if not row['ok']:
        status = 'audit-pending'
    elif direction == 'removed':
        status = 'bounded-coverage'
    else:
        status = 'partial-development-green'
    print(source_id, status, row['observed']['kind'], sep='\t')
    if row['observed']['default_mismatch']:
        print('DEFAULT-MISMATCH', source_id, row['expected']['page_default'],
              row['observed']['default'], sep='\t')
```

These are recommendations, not automatic ledger edits. Preserve stronger independent evidence. For removed symbols confirm the whole row contract is absence before assigning `bounded-coverage`; added/changed rows get at most `partial-development-green`, never behavioral parity. Event changes prove registerability only, not new payloads. Attach the output artifact, test filter, revision, profile and cache provenance to ledger evidence; do not count an expected gap as accepted coverage.

## Object-kind probes and limitations

| Owner in register | Probe |
|---|---|
| Frame, FrameScriptObject, ScriptRegion | CreateFrame('Frame'); latter two are base interfaces, not concrete CreateFrame types |
| FontString | Frame:CreateFontString() |
| TextureBase | Frame:CreateTexture(); abstract base interface |
| VectorGraphics | Frame:CreateVectorGraphics() |
| Minimap, StatusBar | CreateFrame(owner) |
| AnimationGroup, Animation | Frame:CreateAnimationGroup(), then group:CreateAnimation() |
| RadialProgress | group:CreateAnimation('RadialProgress'), with exact GetObjectType guard |
| DurationTextBinding, SecondsFormatter | C_DurationUtil.CreateDurationTextBinding(), C_StringUtil.CreateSecondsFormatter() |

RadialProgress's four rows cannot currently earn method-publication credit: `src/lua_api/animation.rs::AnimationType::from_str` has no RadialProgress and returns generic Animation for unknown types. The guard makes this an explicit non-ok factory observation, rather than probing unrelated shared methods. Factory/lookup failures for any other type are also recorded, not skipped. Runtime creatability of the other kinds was not exercised here.

`CHAT_MSG_*` (one changed event occurrence) cannot be registered as an event name; no authoritative expansion is supplied by the register. It remains explicitly unprobeable/non-ok; no per-event ad hoc list was invented. All 778 IDs still appear in a completed result file.

Namespace/mixin probes use raw resolution and raw member lookup. Removed members also require ordinary lookup nil, including when raw parent resolution failed; fabricated __index functions therefore cannot prove absence. Widget methods use ordinary lookup on factory-created objects because FrameRef methods are shared Rust userdata registrations, not raw Lua fields. This proves reachability only, not owner-specific restrictions or callable behavior.

Current register has no CVar page defaults: every CVar annotation is empty and rows contain no default field. The staged parser accepts an optional `page_default` (alias `default`) string when supplied; mismatches are reported in stderr/JSON but never change `ok`. This run can establish current/default nonnil or nil only, not page-default equality.

## Authoring proof ledger

- Read-only register inspection: 778 rows; serde/serde_json availability confirmed; all register object owners mapped; no runtime observations.
- Read-only helper/API inspection: cached Game preload, real factories, RegisterEvent return/state/error rules and finite event validation examined.
- Standalone rustfmt on staged Rust file: first attempt failed because the unstaged helper path was absent; second used `--edition 2024 --config skip_children=true` and succeeded. This skips formatting the existing helper, not compiler warnings.
- Manual staged Rust/Lua readability inspection: named probe helpers, one centralized classifier, no warning suppressions, no per-symbol cases. Lua strings contain no premature raw-string delimiter; eval returns strings/bool/optional strings, never u32.
- Not compiled, not run, not integrated. Spec requirements intentionally remain unchecked.
