# Patch 7.2.0 API audit

Pinned page 412195, revision 6767100 (2026-07-09T22:13:57Z), fetched 2026-10-08. Three explicit publication identities; prose separately records addon additions, namespace migration and two domain-level removals. Source links are not expanded into fabricated member requirements.

## Concrete coverage matrix

| Boundary | Existing backing behavior | Missing/problematic | Proof |
|---|---|---|---|
| MaskTexture | Region creation, attachment/deduplication, retrieval/removal | Native historical visual parity | Cached behavioral case; final acceptance pending |
| Texture:SetVertexOffset | Independent per-corner state, getter/reset | No renderer consumer of vertex offsets | Complete grep; cached behavior and existing corner tests selected |
| C_EquipmentSet | Saved-set create/rename/spec assignment/delete | No old-member catalog or exact deprecated-wrapper contract in source | Cached lifecycle and existing equipment tests selected |
| New addons | Current APIDocumentation, Contribution, Deprecated load | Historical 2017 addon semantics not inferred | Cached load case selected |
| Voice chat / Mac movie recording | No named members in source | Domain removals lack member/platform contracts | No retirements inferred |

## Discovery and probe correction

Initial prefork sweep reports exactly one gap: `wt-widgets-MaskTexture-5`. Runtime already implements CreateMaskTexture; the shared publication probe incorrectly tries `CreateFrame('MaskTexture')`. Added region factories for MaskTexture and Texture to the test helper. No simulator runtime or Blizzard Lua changes. Cached behavior tests exercise actual attachment/state/loadout transitions rather than mere function existence. Discovery is RED evidence only; final proof follows the committed correction.

## Retirement review

**No retirements.** Migration prose explicitly preserves old APIs through Blizzard_Deprecated. Removal prose names domains, not members. [Untruncated scans](../../../data/patch-api/evidence/7.2.0-session-2026-10-08/p720-scans.json) use `/usr/bin/grep -R -n -w -F` in retail cached AddOns, excluding `*Documentation*`, and src/tests. Bare-name scanning includes `pcall(Name, ...)` and `and Name then` without syntax filters. Current MaskTexture, SetVertexOffset and C_EquipmentSet consumers remain intact. [Later-register snapshots](../../../data/patch-api/evidence/7.2.0-session-2026-10-08/p720-later-register-scan.json) read master and p725-page Git objects only, never the parallel worktree. No named-identity intersection; voice-domain removal does not disable later voice APIs. 7.2.5 placeholder remains first in `later_registers`, followed by 7.3.0 and later pages.

## Verification

Targeted acceptance is in progress. No full integration suite, agents/model CLIs, push or merge. Own target: `/home/osso/.cache/wow-ui-sim-targets/p720-page`. Commands use explicit owned cwd; long Cargo runs are detached with complete logs and scope/revision receipts. The host lacks a WoW install; CASC visual tests are not claimed.

Current reproduction: 44/44 registers and 41/44 extracts; inherited 12.0.5, 12.0.7 and 12.1.0 extract failures unchanged. Parser fixtures pass 28/28; extractor and historical validator fixtures pass. New parsing is opt-in (`--legacy-widget-summaries`, `--prose-namespace-migrations`) and leaves earlier defaults unchanged. Exact source/prose accounting and final historical validator evidence remain to be sealed.

## Sources

- [Pinned provenance](../../../data/patch-api/sources/7.2.0-api-changes.provenance.json).
- [Register](../../../data/patch-api/sources/7.2.0-wikitext-register.json).
- [Publication spec](../../specs/patch-7-2-0-publication-sweep.md).
- [Evidence](../../../data/patch-api/evidence/7.2.0-session-2026-10-08/).

## See Also

- [[patch-7-3-0-api-audit]] — prior-page evidence format.
- [[patch-audit-validator-portability]] — fixed historical scope, checkout-independent receipts.
