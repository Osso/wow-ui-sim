# Three immutable retention manifests — independent verification

Verified: 2026-10-09. **OVERALL: PASS — bounded retention and SSOT claims.** Zero artifact defects found. Not runtime acceptance, clean-startup certification, native-client proof, or external-publication approval. Parent remains OPEN.

Followed `/home/osso/AgentConfig/skills/verify/SKILL.md` as the verifier; performed checks directly without delegation. Scope: immutable Git objects, selected retained evidence, available selected originals, and changed wiki claims. No tests, builds, broad checks, network, executable/runtime invocation, services, packages, reboot, repository edits, commits, pushes or deployments. Only requested local verification outputs written.

## Artifact coverage

All counts exclude each manifest itself.

| Commit / bundle | Files | Bytes | Commit hash + size | Working-copy identity | Available original identity |
|---|---:|---:|---|---|---|
| `ff4585f30badb39687245046d263b3e2c56e503c` / controlled linker | 70 | 2,490,261 | 70/70 PASS | 70/70 PASS | 70/70 PASS |
| `5e4e6677d657bcc44de84d4b23aa6e50942fc43a` / fixture preconditions | 11 | 61,758 | 11/11 PASS | 11/11 PASS | 11/11 PASS |
| `287d3eb243c7566eb43bfcb0efbf2c93e19178ea` / startup audit | 7 | 123,339 | 7/7 PASS | 7/7 PASS | 7/7 PASS |
| Total | **88** | **2,675,358** | **88/88 PASS** | **88/88 PASS** | **88/88 PASS** |

Each immutable bundle tree contains exactly its manifest and listed files: 71, 12, and 8 tree entries. All three working manifests match their commit blobs. Commit changes are restricted to the respective evidence directory and wiki; no runtime/test implementation changes in these retention commits.

- [EXIST] PASS: all manifest entries present in immutable trees and working copies; full path/size/hash ledger in `proof.json`.
- [SUBSTANTIVE] PASS: complete selected bytes consumed; nonempty receipts, diagnostics, patches and source snapshots, not placeholders. Twelve controlled-audit source snapshots match their named Git revisions. Both fixture patches match exact immutable Git diff/show output.
- [WIRED] PASS: changed SSOT sections link to existing immutable artifacts/specs. Retained-source intersections of audit hash ledgers match: controlled 71/76 rows, startup 6/12 rows. Remaining rows refer outside the selected retention set and were not recertified.
- [ANTI-PATTERN] N/A to implementation: these commits retain historical evidence and documentation, not new runnable code. Historical source snippets/diagnostics are not treated as newly introduced TODO/stub implementations.

## SSOT claim boundaries

**Controlled linker:** `docs/wiki/investigations/integrated-source-and-factory-proof-2026-10-09.md:303–318` correctly separates compile success from runtime failure. Retained Cargo stdout has 739 JSON records, ending in `build-finished success=true`; Cargo receipt exit0. The 3,845 before/after source entries are equal. Exact saved rows: case1 `1 passed; 0 failed`, case2 `1 passed; 0 failed`, case3 `0 passed; 1 failed`; corresponding service main statuses 0, 0, 101. Eight same-chunk probe records stop after `after_editbox_metatable`; no getter-call or after-secretwrap credit. No precise linker cause, durable runtime repair, or follow-up execution inferred.

**Fixture supplement:** SSOT `:320–334` matches immutable `611e10d2d` and `21b513f3e` patches: enUS grouping with unchanged icon markup; saved/disabled/restored OnTextChanged; Big21/Buff22 and metadata max22/count23; owned temporary directory; explicit GameTooltip Show and visible-order preconditions. Historical two-case execution remains linked to the controlled epoch. Four later exact cases remain unexecuted in this evidence; no new native or broader execution claim.

**Standalone startup:** SSOT `:336–350` agrees with retained diagnostics/report: 4,636 + 359 = 4,995 classified lines; category totals reconcile, zero unclassified; seven signatures, 89 additional suppressed, 96 occurrences; eight status-row warnings; one selected completion marker. Recorded service main status0 is not handler success. Clean-startup FAIL remains explicit. EditMode cache use remains explicit despite ordinary SavedVariables disabled. The 4,044 cache-entry and binary matches are historical audit observations, not newly verified cache/binary state or continuous immutability. No current-test, other-profile, sustained-runtime or native-client acceptance granted.

## Privacy and exclusions

Full selected UTF-8 credential-pattern scan across all 88 files: zero private-key, Bearer, credential-assignment or common-token candidates. This bounded scan is not exhaustive privacy certification.

Startup tree retains seven selected audit/technical receipts only. Raw stdout/stderr, binary, cache-entry inventory and private profiles are absent from that tree and were not opened or copied by this verifier. Selected diagnostics contain 74 selected lines, not the 4,995-line raw capture. Redactions are retained at stdout lines 3 and 3745: account/realm/character identity and private filesystem prefix. Structured startup JSON contains no matching sensitive identity/layout payload keys. The sole WTF/Account path-pattern hit is the generic exclusion label `WTF/account/realm/character profiles` in privacy.json, not a real identity or profile payload. Aggregate excluded-input hashes do not publish their inputs.

Local paths, usernames, revision IDs, process/service/boot metadata and technical source remain in these bundles. Controlled audit privacy explicitly states local-only unsanitized evidence; redaction is required before external publication. SSOT caveats preserve this boundary. PASS does not authorize publication of raw startup payload or blanket release of these bundles.

## Proof method and limitations

Read-only commands: `git show <commit>:<path>`, `git ls-tree -r --name-only <commit> -- <bundle>`, `git diff-tree --no-commit-id --name-only -r <commit>`, targeted `git cat-file -e`, fixture `git diff`/`git show --format=fuller`. Command exits checked. Python SHA-256, byte-size and direct equality comparisons operate on full selected data; no truncation used for comparisons/scans. Per-file results and semantic consistency receipts are in compact `proof.json`.

Verifier bookkeeping correction: initial JSON construction reused size/hash keys for booleans, producing false aggregate/snapshot alarms. Individual artifact comparisons had passed. Corrected serialization and recomputed dependent comparisons; no artifact changes or runtime reruns. A generic privacy exclusion label was likewise examined and rejected as a payload finding. No unresolved findings.
