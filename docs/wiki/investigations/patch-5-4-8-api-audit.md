# Patch 5.4.8 API audit

Pinned pageid 177816, revision 1736772 (2014-05-16). This is the 2014 retail Mists line, not Mists Classic. The API page itself has no literal interface number; its linked parent Patch 5.4.8, revision 6611068, states `toc = 50400` and `Release = May 20, 2014`. The API navigation leads to retail 6.0.1.

## Source and scope

The page contains 27 changed CVar identities and one `SetUIVisibility` combat restriction, not API additions/removals. Default extraction retains all prose and yields 32 supplemental occurrences. An opt-in `--combat-restriction-bullets` generator flag preserves the 28 inventory occurrences; prior defaults remain unchanged.

The prefork sweep applies current-retail publication/absence only. Two ordered integration placeholders remain: 6.0.1 then 6.0.2, followed by the merged 6.1.0 and later retail chain. Classic 5.5.x registers are excluded.

## Retirement and caller evidence

No removals on this page and no runtime retirements. All 29 changed/writer identities have untruncated whole-word `/usr/bin/grep` scans in the session directory: retail cached consumers excluding Documentation, plus all src/tests callers including `pcall(Name, ...)` and `and Name then`. Master and queued p602/p601 register sets are recorded using `git ls-tree` and pinned revisions, without reading their live worktrees.

## Proof status

Discovery and combat behavior proofs are in progress. Publication does not prove native failure messages, protection notifications, exact historical security policy or inactive CVar behavior. Final counts and gate results will be recorded here after verification.

## Sources

- [Pinned source/provenance](../../../data/patch-api/sources/5.4.8-api-changes.provenance.json).
- [Own session evidence](../../../data/patch-api/evidence/5.4.8-session-2026-10-08/).
- [Sweep spec](../../specs/patch-5-4-8-publication-sweep.md).

## See Also

- [[patch-6-1-0-api-audit]] — next merged retail register.
- [[patch-audit-validator-portability]] — historical clean/later-audit gate.
