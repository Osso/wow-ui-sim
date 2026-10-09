# Patch-audit validator portability

Merged audit validators prove historical executions, not new executions against every subsequently added patch. Base `7fab847eb` had thirteen failing validators and one passing validator across fourteen retained `validate.py` files.

## Root causes and fixes

| Validator | Root cause | Fix |
|---|---|---|
| 10.0.0 | Expanded register set; absolute receipt cwd; result rewrite deletes saved revision | Pin complete historical register set; remove path gate; print without rewriting result |
| 10.0.2 | Expanded register set; absolute receipt cwd/target | Pin complete historical register set; remove path gates |
| 10.0.5 | Expanded register set; absolute receipt cwd/target | Pin complete historical register set; remove path gates |
| 8.2.5 | Absolute receipt cwd/target | Remove path gates; existing exact preservation checks remain |
| 8.3.0 | Later 9.2.5 ledger/fixture closure; expanded register set; absolute receipt paths | Exact merged replacement allowance; historical accounting and register scope; remove path gates |
| 8.3.7 | Later 9.2.5 ledger/fixture closure; expanded register set; absolute receipt paths | Same |
| 9.0.1 | Later 9.2.5 ledger/fixture closure; expanded register and sweep-source sets; absolute receipt paths | Same, plus pin source-defined historical sweep set |
| 9.0.2 | Later 9.2.5 ledger/fixture closure; expanded register set; absolute receipt paths | Exact merged replacement allowance; historical accounting and register scope; remove path gates |
| 9.0.5 | Later 9.2.5 ledger/fixture closure; expanded register set; absolute receipt paths | Same |
| 9.1.0 | Later 9.2.5 ledger closure violates frozen hash; changed fixture disagrees with historical sweep | Exact merged replacement allowance; historical fixture accounting |
| 9.1.5 | Later 9.2.5 ledger closure; absolute receipt paths | Exact merged replacement allowance; remove path gates |
| 9.2.0 | None; already passing | Unchanged |
| 9.2.5 | Later ledger counts and gap fixture disagree with historical observations; absolute receipt paths | Validate current replacement provenance, then check historical ledger/fixture; remove path gates |
| 9.2.7 | Absolute receipt cwd | Remove path gate |
| 8.1.5 | Refreshed 8.2.0 register set; absolute receipt cwd/target; validator rewrites evidence; command scope compared with moving HEAD | Pin integrated register and proof comparison scope at `ee2315426`; use shared exact preservation allowances; validate read-only |
| 8.1.0 | Merged 8.1.5/8.2.0 registers; formerly accepted arbitrary protected-file drift | Pin integrated register/sweep scope at `ba7e46ad6`; shared exact preservation checks; refresh own receipts and two attributable supersessions |
| 8.2.0 | Later 8.1.5 expands register and sweep sets; absolute receipt cwd/target | Pin merged 8.2.0/8.2.5 scope at `ea9e5995e`; remove path gates |

| 7.0.3 integrated | Live shared-runtime hashes reject the later 6.2.0 cost-policy module despite fixed Git-scope proof | Remove redundant live-runtime equality; retain recorded revision/scope, receipts, historical artifacts and validator/log hashes |

## Historical inputs remain protected

[Shared helper](../../../tools/patch_audit_validation.py) allows only the exact original-to-final SHA-256 pairs for the 9.2.5 ledger and known-gap fixture. Their replacement bytes are pinned to the merged [8.2.5 audit](patch-8-2-5-api-audit.md) endpoint `127aa3724035d9e668143d9b28aaa4cf6e176fa1`; original bytes are from `7ff3dc540^`. The sole closure is `wt-global-api-C_ClubFinder.ReportPosting-117`, listed in [later-gap closures](../../../data/patch-api/evidence/8.2.5-session-2026-10-08/p825-later-gap-closures.json).

Original before-hashes stay unchanged. Arbitrary row edits, unrelated file replacements and even whitespace additions to the approved replacement fail. Historical accounting reads the old Git blob only after checking current bytes against that exact allowance. The 9.2.5 audit still proves its original 33 bounded rows and 34 gaps; current follow-up accounting remains 34 bounded rows and 33 gaps.

Register and sweep-source sets come from fixed audit revisions, not receipt-derived subsets. Every historically required file must still exist. New earlier-page audits do not retroactively expand historical proof scope. Full Git history containing those revisions is required; missing history fails explicitly, with no bypass.

Only absolute cwd/target equality assertions were removed. Receipt commands, revisions, log/compressed-log hashes, exits, invalidation labels, warning checks, negative controls and source accounting retain their previous checks. The original portability repair did not rerun runtime proof or regenerate logs, results, receipts or before-hashes. [8.1.0 integration](patch-8-1-0-api-audit.md) separately refreshes its own runtime/reproduction receipts; other audits retain historical proof.

## Verification

Run `python3 -B tools/test_patch_audit_validation.py` for original/replacement bytes, tamper rejection, historical accounting, complete snapshot scope and read-only result regression. Final acceptance runs all fourteen local validator copies in both the implementation checkout and a second detached checkout of the same branch revision; protected-input tampering must fail before restoration. Neither validation path writes retained evidence.

## Fresh-checkout and later-audit gate

Run `python3 tools/check_patch_validators.py [revision]` (default `HEAD`) before merging. The [gate](../../../tools/check_patch_validators.py) creates a clean detached temporary worktree, runs every retained `validate.py`, commits unrelated changes to a C API source, generator, new `9.9.9` register/evidence and wiki log, then reruns every validator. It removes the worktree, prints per-validator JSON and fails if either phase fails. Existing source registers are never reformatted or mutated. [Fixture tests](../../../tools/test_check_patch_validators.py) cover ignored scratch dependencies, moving shared inputs, pinned proof success and cleanup.

Evidence storage rule: never inline directory inventories; use Git tree IDs per pinned revision/directory and individual blob IDs for file-specific exceptions. Recorded pre-rebase IDs are content proofs, not dependencies on unreachable commits. Every tracked file under `data/patch-api/evidence` must stay under 5 MB (5,000,000 bytes); the gate reports paths exceeding that limit in both phases.

Rule: no live-file comparisons outside the audit's own session directory; prove shared files at recorded Git revisions. Historical register sets use `historical_registers`, not today's global glob. Own immutable session records retain their seals; missing history remains an error.

At `45a64bef3`, 27 of 28 validators passed both phases. Integrated 6.2.4 failed on ignored scratch `PLAN.md` and also compared shared runtime/tool/source files and prior validators with live bytes. The repair pins those checks to recorded revisions, removes the two uncommitted scratch entries, and preserves counts, gap sets, negative control, receipts/log hashes and rebase mapping. Its [note](../../../data/patch-api/evidence/6.2.4-session-2026-10-08/integrated/validator-portability-note.md) explains preservation of the original historical-manifest seal.

## Compact rebase evidence

The seven mappings with `pinned_inputs` use [tree proofs](../../../tools/patch_audit_pin_trees.py): full rebased directory tree IDs, projected recorded-input tree IDs, sparse exclusions and blob overrides. Standalone files and historical byte exceptions retain blob IDs and SHA-256 seals. Historical replay reconstructs the exact original input dictionaries, directory names and diff blob maps before running the existing checks. Conversion compared every expanded structure with its original and checked every original content seal.

Five sessions also had stale *rebased* commit references after subsequent rebases. Historical labels now resolve through verified root-tree snapshots at master-ancestor commits. Missing wiki blobs and three changed patches are gzip-preserved and hash-checked; recorded trees are recomputed without opening historical tree objects. The reachable counterpart's patch ID is independently pinned. Both entrypoints hash-pin the helper; only changed artifact seals were refreshed. Original mappings remain at `a82b8eb1c` under their session paths, named in each affected `integrated/evidence-compaction-note.md`.

The four remaining mappings have no `pinned_inputs`: 6.1.0 retains individual blob checks and source/sweep inventory allowances; 6.2.0 retains referenced source blobs and exact permitted inventory additions; 6.2.4 retains raw changed-blob and patch/tree checks; 7.0.3 retains its original-history and runtime equality checks. Their small proofs are unchanged.

## Sources

- [Shared helper and tests](../../../tools/test_patch_audit_validation.py).
- [8.2.5 input preservation](../../../data/patch-api/evidence/8.2.5-session-2026-10-08/p825-input-preservation.json).
- [9.2.5 historical validator](../../../data/patch-api/evidence/9.2.5-session-2026-10-07/validate.py).

## See Also

- [[patch-8-2-5-api-audit]] — authorizes the exact later ReportPosting closure.
- [[patch-9-2-5-api-audit]] — original proof remains historical.
