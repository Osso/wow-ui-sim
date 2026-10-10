# Independent Forever aura-block event RED verification

Date: 2026-10-10. Repo: `/home/osso/Projects/wow/wow-ui-sim`. Inspected HEAD: `20e808b945188ce2b952f3dd81f2aa6547bc8587`.

**OVERALL: PENDING — resource-blocked; no behavioral RED receipt.** Existing producer terminated before compiling. No wait needed for this terminal outcome. Verifier performed read-only receipt/source inspection; no builds, tests, check tools, reruns, operational changes, or delegation. Only this requested report was written.

## Receipt result

Epoch `20261010T190145Z/outcome.json` contains exactly:

```json
{"build_performed": false, "status": "resource-blocked"}
```

`resources.json` records load1 **41.09**, available memory **32.26543045043945 GiB**, free disk **1882998263808 bytes**, shared-container lock used **false**. Worker admission rejects load >=24 or available memory <6 GiB; recorded load explains the block. Controller stdout records completion of the worker, not completion of compilation/testing. Root `submission.json` exit 0 proves queue submission only.

Epoch has only `resources.json` and `outcome.json`: **zero executed integration tests**, no compile result, no source-before/after snapshots, no sealed artifact, no execution receipts. Compile exit is **unavailable**, not 0 or 101. Resource blocking and compile exit 101 would not constitute behavioral RED.

| Required proof | Observed status |
| --- | --- |
| Compile exit 0, current-source binding, sealed integration artifact | PENDING; absent |
| Filter selects exactly three actual cases | PENDING runtime; source defines three matching cases |
| RegisterEvent positive fails at unknown UNIT_AURA_BLOCK_LIST_CLEARED | PENDING; not executed |
| RegisterUnitEvent positive fails at same unknown event | PENDING; not executed |
| Separate unrelated-unknown control passes | PENDING; not executed |

Worker's intended selector is `wowforever_finite_constants::forever_aura_block_list_cleared_`, with expected count 3. Intended compile uses `--test integration --no-default-features --features sound,gui,casc,client-wowforever --offline --locked`; these are worker instructions, not observed compiler/artifact evidence. This blocked epoch did not record its own HEAD; inspected current HEAD matches the caller's supplied revision, but that is not execution binding.

## Test fidelity and original preservation

- `tests/wowforever_finite_constants.rs:1` gates the module on `client-wowforever`. Three additions begin at lines **91, 120, 152**. `tests/integration.rs:1` includes the generated harness; `build.rs::generate_integration_test_harness` discovers top-level test modules and emits their module declarations. Source wiring exists; actual executable membership remains unproven.
- Positive cases call real `WowLuaEnv::new()` and public frame methods, without registry manipulation or vendor patches. Registration occurs at lines **103** and **132**, before simulated dispatch. Both methods reach `ensure_registerable_event` in `src/lua_api/frame/methods/text_attribute_event/events.rs:103–127`; rejection identifies the unknown event. RegisterUnitEvent shares the diagnostic formatted as RegisterEvent, so that diagnostic alone does not distinguish the invoked method.
- Forever's finite list in `src/event/valid_events.rs:31–65` omits the cleared event. Its occurrence at line 195 belongs to the retail-12-1-0-gated list, not the Forever additions. This supports the expected registration boundary, not an observed result from these new tests.
- Tests assert externally observable registration, callback identity/name, supplied one-unit payload delivery, unit filtering, unregister behavior, and absence of collected callback errors. Separate control invokes both registration methods on an invented event and asserts rejection identifies the name and retains no registration. These are behavioral tests, not source-shape assertions.
- Commit `20e808b94` changes only this test file: **80 insertions, zero deletions**. Independently removed the new block in memory and compared the entire remaining file byte-for-byte with `20e808b94^`: **equal**. Existing Forever tests are unchanged. Commit has no `src` diff, and current `src`/test file have no diff against that commit: no production registry change in this scope. Unrelated dirty wiki/untracked files do not establish a clean checkout.

## Highest-practical original boundary retained

Independently parsed the preserved normal stock baseline `/home/osso/.local/state/wow-ui-sim/verification/forever-stock-startup-current/20261010T184613Z/lua-errors.stdout`: **19 records, 36 total occurrences, 16 occurrences containing UNIT_AURA_BLOCK_LIST_CLEARED**. SHA-256 matches the retained baseline summary: `56ee89fd73d7740bd32951cf5610078f0ae8838d7dea99fec97eba8e68e8fefa`.

Preserved traces show unknown-event registration failures in actual Blizzard CompactRaidFrameManager/Container and CompactPartyFrame startup/event paths. This is the highest-practical original failure boundary supplied here; new isolated tests target its shared registration rejection but do not reproduce full stock load order or private-aura lifecycle. Baseline predates these additions (recorded revision `98f625741d01df768af6c3c97324a3cfbc544573`); it is original-failure evidence, not current-test RED proof.

## Rust readability / artifact-mode review

Read and followed `verify` and `rust-readability`; manually inspected all 80 added lines without executing audit/check tools. No reportable readability violations in the added Rust tests. Bodies are short, explicitly named, shallow, and expose state inspection at the call site; intentional test setup similarity does not warrant abstraction. No TODO/FIXME/HACK/XXX, suppression attributes, commented-out code, or placeholder bodies were added. Substantive test bodies and source harness wiring pass static inspection; execution remains PENDING.

Current test-file SHA-256: `1c97e9a1064969e694f649bf65611e7e1ea7d23a605f563ae5d5ded5e9b9c355`.
Epoch outcome SHA-256: `89b6c59b397a69d9b12a3083e71683e05a56f40620af7de552938a2375cb393f`.

**Limits:** Payload assertions concern only explicitly supplied `A_Admin.FireEvent` simulator dispatch. No native event production, native payload parity, successful control execution, compile success, or completed behavioral RED claimed. No retries or remediation attempted.
