# Patch 7.0.3 API audit

Pinned page 549091, revision 5295335 (2017-09-18T11:16:26Z), fetched 2026-10-08. Source has 123 wikitext lines and 134 explicit publication identities, including nested profession inventories and prose retirements. This is a bounded current-retail audit, not reconstruction of the Legion client.

## Active coverage matrix

| Boundary | Modeled/preserved | Problematic | Proof stage |
|---|---|---|---|
| Recipe name filter | Per-environment text, existing learned catalogue, nil clear, case-insensitive matching, list-update dispatch | Native locale collation, reagent searching and other filters excluded | Cached RED proves no-op setter; implementation awaiting GREEN |
| Mount renames | GetMountInfo/GetMountInfoExtra absence; current ByID successors untouched | Summon has live simulator callers; retain legacy member | Initial sweep proves synthesized lookup; complete grep/later-register review |
| Historical widgets | Correct existing region/animation factories | Native 3D UiCamera and Model rendering intentionally unsupported | Publication distinct from behavior |
| Remaining source | Every literal retained for exact accounting | Archive-only APIs, missing fixtures and unnamed domain migrations | Discovery: 57/134 publication gaps; final accounting pending |

## Retirement boundary

All nineteen explicitly removed identities have qualified and bare whole-word `/usr/bin/grep` scans, excluding `*Documentation*` in cached retail AddOns. Full src/tests scans have no syntax filter, so `pcall(Name, ...)` and `and Name then` cannot be missed. Outputs are untruncated. [Scan index](../../../data/patch-api/evidence/7.0.3-session-2026-10-08/p703-retirement-scans.json) records commands, exits, matches and hashes.

Only C_MountJournal.GetMountInfo and GetMountInfoExtra are newly retired: neither has cached or simulator callers, and no later register re-adds them. [Later scan](../../../data/patch-api/evidence/7.0.3-session-2026-10-08/p703-later-register-scan.json) reads fixed master and p710-page Git objects, never the parallel worktree. Other consumers remain: SetGlyph is used by cached InspectUI; Summon by collection tests; Alpha:SetChange, Model:GetModel and GameTooltip:SetTradeSkillItem by existing behavioral callers. ShowHelm/ShowCloak/ShowingHelm/ShowingCloak are explicitly re-added by 12.0.0. Already absent identities need no code change. Unnamed glyph `etc.`, Multistrike and Amplify members are not guessed.

## Verification policy

Long commands run asynchronously with complete log files and revision/scope receipts under the owned target. No full integration suite, push, merge, agents/model CLIs, working-directory switching, other-worktree edits or vendor changes. Master baseline is an immutable Git-archive snapshot inside the owned target, not a worktree. Addons-enabled startup comparison and touched profession/collection regressions are required before completion.

Opt-in `--legion-prepatch` preserves prior parser behavior. Initial reproduction: 46 registers and 43 extracts reproduce; inherited 12.0.5, 12.0.7 and 12.1.0 extract failures are unchanged. All 25 pre-existing historical/integrated validators pass. Final proof remains pending at this implementation checkpoint.

## Sources

- [Pinned provenance](../../../data/patch-api/sources/7.0.3-api-changes.provenance.json).
- [Publication register](../../../data/patch-api/sources/7.0.3-wikitext-register.json).
- [Spec](../../specs/patch-7-0-3-publication-sweep.md).
- [Evidence](../../../data/patch-api/evidence/7.0.3-session-2026-10-08/).

## See Also

- [[patch-7-2-0-api-audit]] — region factory and proof conventions.
- [[patch-8-0-1-api-audit]] — pre-patch accounting without native parity claims.
- [[patch-audit-validator-portability]] — fixed historical scope and exact input protection.
