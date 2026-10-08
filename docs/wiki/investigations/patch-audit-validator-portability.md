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

## Historical inputs remain protected

[Shared helper](../../../tools/patch_audit_validation.py) allows only the exact original-to-final SHA-256 pairs for the 9.2.5 ledger and known-gap fixture. Their replacement bytes are pinned to the merged [8.2.5 audit](patch-8-2-5-api-audit.md) endpoint `127aa3724035d9e668143d9b28aaa4cf6e176fa1`; original bytes are from `7ff3dc540^`. The sole closure is `wt-global-api-C_ClubFinder.ReportPosting-117`, listed in [later-gap closures](../../../data/patch-api/evidence/8.2.5-session-2026-10-08/p825-later-gap-closures.json).

Original before-hashes stay unchanged. Arbitrary row edits, unrelated file replacements and even whitespace additions to the approved replacement fail. Historical accounting reads the old Git blob only after checking current bytes against that exact allowance. The 9.2.5 audit still proves its original 33 bounded rows and 34 gaps; current follow-up accounting remains 34 bounded rows and 33 gaps.

Register and sweep-source sets come from fixed audit revisions, not receipt-derived subsets. Every historically required file must still exist. New earlier-page audits do not retroactively expand historical proof scope. Full Git history containing those revisions is required; missing history fails explicitly, with no bypass.

Only absolute cwd/target equality assertions were removed. Receipt commands, revisions, log/compressed-log hashes, exits, invalidation labels, warning checks, negative controls and source accounting retain their previous checks. No runtime proof was rerun or claimed fresh. Logs, results, receipts and before-hashes were not regenerated.

## Verification

Run `python3 -B tools/test_patch_audit_validation.py` for original/replacement bytes, tamper rejection, historical accounting, complete snapshot scope and read-only result regression. Final acceptance runs all fourteen local validator copies in both the implementation checkout and a second detached checkout of the same branch revision; protected-input tampering must fail before restoration. Neither validation path writes retained evidence.

## Sources

- [Shared helper and tests](../../../tools/test_patch_audit_validation.py).
- [8.2.5 input preservation](../../../data/patch-api/evidence/8.2.5-session-2026-10-08/p825-input-preservation.json).
- [9.2.5 historical validator](../../../data/patch-api/evidence/9.2.5-session-2026-10-07/validate.py).

## See Also

- [[patch-8-2-5-api-audit]] — authorizes the exact later ReportPosting closure.
- [[patch-9-2-5-api-audit]] — original proof remains historical.
