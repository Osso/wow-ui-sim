# Independent 3.4.3 source-accounting verification

**PASS — bounded source accounting only.** Verified 2026-10-09 in `/home/osso/.worktrees/wow-ui-sim-p343-source`, clean branch `p343-source`, HEAD `2213f81f85d7e6008c254b7b434ed53d985b10ce`. Followed `/home/osso/AgentConfig/skills/verify/SKILL.md`; no delegation, cwd-switch, Bash, repository edits, commits, builds, Cargo, profile checks, fixture reruns or shared acceptance gate.

## Fresh replay

Command (absolute cwd above; `PYTHONDONTWRITEBYTECODE=1`):

```text
python3 /home/osso/.worktrees/wow-ui-sim-p343-source/data/patch-api/evidence/3.4.3-session-2026-10-09/validate.py
```

Exit **0**, stderr empty; full stdout:

```json
{"enumerated_api_occurrences": 0, "header_counts": [], "removals": 0, "runtime_observations": 0, "sealed_inputs": 16, "source_rows": 7, "statuses": {"UNPROVEN": 2, "metadata-only": 5}}
```

Exact archived extractor/generator are the proof boundary, not current shared parsers. `validate.py` imports the two historical copies, reproduces plaintext with canonical navigation, derives source-row identities/statuses and inventory counts, and checks all 16 seals. This is not a simulator/runtime check.

## Source, historical evidence and rebase

- Frozen response confirms page **152751**, revision **5983024**, timestamp **2024-03-07T08:49:48Z**. Literal source TOC **30403**. Wikitext 463 bytes, SHA-256 `1849b20140e6c83b62c4a1b5a14adba2143de8ce01d9a368b7a52d166d0f5e6e`; response 747 bytes, SHA-256 `594cacde59ac51e99109e28fea49ea153a1969994032064f83b8e3dfad5b3b05`.
- All **16/16** sealed source/evidence files match their seals AND original `48ab8e1c3112427d25f5604e4b2f048517447051` blobs. `seals.json` itself is byte-identical before/after rebase. Scoped `git diff --stat 48ab8e1c3 HEAD -- <three source files> <own evidence directory>` produced empty stdout, exit 0.
- The **10 substantive original tested input/tool/fixture/validator files** checked against tested revision `84274cf675b7e9cf7f8048d758ad07297beb5023` are byte-identical. `red.log` does not exist at that implementation revision (git exit 128); it was a later retained receipt, not tested implementation. This matches the docs' receipts-added-later explanation and does not invalidate GREEN.
- Retained GREEN log: **7 tests**, **OK**, 0.023s. Fixture inspection independently counts **22 mutation cases** (2 revision/source, 7 row omissions, 4 prose credit, 4 client/successor, 1 inventory, 4 profile/native/runtime). No unchanged fixture rerun. RED log is one real derived-count assertion failure, not import/setup failure. Tamper log ends `AssertionError: seal: data/patch-api/evidence/3.4.3-session-2026-10-09/green.log`; retained ledger records exit 1. Fresh positive replay and green.log hash validate restored bytes; tamper operation was not repeated.
- Historical profile/code and Wrath manifest reference hashes match Git blobs at `23c930d837530e197d5f728e07336f81847e0710`. Profile **Wrath**, feature **client-wrath**, configured interface **38001**, source TOC **30403**, native/runtime observations **0**. Cache observation has **42 entries**, all within API-documentation directories; one is an empty `BattleNetDocumentation.lua.missing` marker (41 non-marker paths). Thus 42 is accurate as an observed file-entry count, not 42 usable documentation payloads. No present-day cache/native observation was made.

## Claim boundaries

| Contract | Evidence | Result |
|---|---|---|
| Complete nonblank source accounting | Lines 1,3,4,5,7,8,9; five metadata/two UNPROVEN | PASS |
| Retail 10.1.7 summary | Unspecified subset; no expanded linked page or member inventory | UNPROVEN, correctly retained |
| Collections summary | No named APIs, signatures, outputs, events or security rules | UNPROVEN, correctly retained |
| Empty inventory | Zero explicit APIs/headers/removals; no register/positive capability credit | PASS source count; NO runtime/native parity |
| Supported Wrath versus historical build/cache | 38001 separately recorded from 30403; documentation-only historical cache | PASS boundary; publisher-loaded native proof absent |

Inspected spec, investigation wiki, own index/log entries, source ledger, validator, fixtures, retained logs and relevant historical parser functions. No inaccurate seven-row/five-metadata/two-UNPROVEN/7-test/22-mutation claim found. Historical validator receipt reports **11 seals at its historical run**, not the current 16; wiki explicitly says the seal total grows and is not fixed acceptance data. Retained tests are historical proof with unchanged bytes, not a claim of a fresh seven-test execution at rebased HEAD. No retail/Cata/Mists successor substitution, linked-page reconstruction, current-runtime parity, native parity or shared final-gate completion claim found.

Main still owns integration after 4.0.1 and all broader gates. This source-only page requires no Rust fmt/readability gate; upcoming 3.4.2 shared-classifier work is separate.

## Receipt

`/tmp/p343-independent-receipt.json` contains exact replay argv/cwd/env/exit/stdout/stderr, full original Git blob outputs, per-file before/after/seal SHA-256 comparisons, historical revision/reference checks and full retained log contents.

Receipt SHA-256: `aa96a8949f44c51c1a19c9e8893045dd284d0ad9f052a168b63144a2a07cc71c`.
