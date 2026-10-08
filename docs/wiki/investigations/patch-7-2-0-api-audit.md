# Patch 7.2.0 API audit

Pinned page 412195, revision 6767100 (2026-07-09T22:13:57Z), fetched 2026-10-08. Three explicit publication identities; prose separately records addon additions, namespace migration and two domain-level removals. Source links are not expanded into fabricated member requirements.

## Concrete coverage matrix

| Boundary | Existing backing behavior | Missing/problematic | Proof |
|---|---|---|---|
| MaskTexture | Region creation, attachment/deduplication, retrieval/removal | Native historical visual parity | Cached behavioral case passes |
| Texture:SetVertexOffset | Independent per-corner state, getter/reset | No renderer consumer of vertex offsets | Complete grep; cached state test and 25 texture regressions pass |
| C_EquipmentSet | Saved-set create/rename/spec assignment/delete | No old-member catalog or exact deprecated-wrapper contract in source | Cached lifecycle, six integration and seven library regressions pass |
| New addons | Current APIDocumentation, Contribution, Deprecated load | Historical 2017 addon semantics not inferred | Cached load case passes |
| Voice chat / Mac movie recording | No named members in source | Domain removals lack member/platform contracts | No retirements inferred |

## Discovery and probe correction

Initial prefork sweep reports exactly one gap: `wt-widgets-MaskTexture-5`. Runtime already implements CreateMaskTexture; the shared publication probe incorrectly tries `CreateFrame('MaskTexture')`. Added region factories for MaskTexture and Texture to the test helper. No simulator runtime or Blizzard Lua changes. Cached behavior tests exercise actual attachment/state/loadout transitions rather than mere function existence. Discovery is RED evidence only; final proof follows the committed correction.

## Retirement review

**No retirements.** Migration prose explicitly preserves old APIs through Blizzard_Deprecated. Removal prose names domains, not members. [Untruncated scans](../../../data/patch-api/evidence/7.2.0-session-2026-10-08/p720-scans.json) use `/usr/bin/grep -R -n -w -F` in retail cached AddOns, excluding `*Documentation*`, and src/tests. Bare-name scanning includes `pcall(Name, ...)` and `and Name then` without syntax filters. Current MaskTexture, SetVertexOffset and C_EquipmentSet consumers remain intact. [Later-register snapshots](../../../data/patch-api/evidence/7.2.0-session-2026-10-08/p720-later-register-scan.json) read master and p725-page Git objects only, never the parallel worktree. No named-identity intersection; voice-domain removal does not disable later voice APIs. 7.2.5 placeholder remains first in `later_registers`, followed by 7.3.0 and later pages.

## Verification

Targeted acceptance passes. No full integration suite, agents/model CLIs, push or merge. Own target: `/home/osso/.cache/wow-ui-sim-targets/p720-page`. Commands use explicit owned cwd; long Cargo runs were detached with complete logs and scope/revision receipts. The host lacks a WoW install; CASC visual tests are not claimed.

[Occurrence ledger](../../../data/patch-api/sources/7.2.0-page-coverage.json): three publication + twelve extract IDs = fifteen total. Five bounded, six metadata-only, four substantive prose contracts pending. Publication gaps: zero. New parsing is opt-in (`--legacy-widget-summaries`, `--prose-namespace-migrations`) and leaves earlier defaults unchanged.

| Command / filter | Result |
|---|---|
| `cargo test --test prefork_full_ui -- patch_7_2_0` | 4/4 cached cases |
| `cargo test --test prefork_full_ui -- publication_sweep` | 45/45: 44 pages plus factory |
| Integration `texture_methods_port::`, `methods_texture::masks_and_misc::`, `equipment_set` | 25/25, 10/10, 6/6 |
| Lib `wow_api_equipment_set::` | 7/7 |
| Python extractor / generator / validator fixtures | 34/34, 28/28, 8/8 |
| Retained register / extract reproduction | 44/44 registers; 41/44 extracts |
| Mists `cargo check --no-default-features --features sound,gui,casc,client-mists --tests` | Exit 0; zero non-vendor warnings |
| `cargo fmt --check` | Exit 0 |
| Separate branch/master builds and addons-enabled bounded `lua-errors` | Both exit 0; exact error arrays `[]` |
| Negative namespace substitution | Expected exit 1; precisely 0 → 1 failed IDs; no resolved/stale IDs |

[Proof ledger](../../../data/patch-api/evidence/7.2.0-session-2026-10-08/p720-proof-ledger.md) records exact commands, revisions, scope hashes and invalidations. Runtime/accounting snapshot is `8ae333df4`; source input preservation is pinned to master `1ade15b52`. Three inherited extract failures (12.0.5, 12.0.7, 12.1.0) remain unchanged. Vendor iced manifest deprecations remain unsuppressed. Rust readability audit found no new suppressions or deeply nested logic.

[Startup comparison](../../../data/patch-api/evidence/7.2.0-session-2026-10-08/p720-startup-comparison.json) uses a separately built master Git-archive snapshot inside the owned target. Neither execution disables addons. Production src/Cargo/build/Interface/profile manifests are unchanged; shared probe changes have no runtime deployment. No other worktree was modified.

## Historical validator

[Validator](../../../data/patch-api/evidence/7.2.0-session-2026-10-08/validate.py) passes read-only, without absolute checkout/target equality gates. Historical register/sweep scope comes from `historical_registers` and the fixed runtime revision, never the moving checkout or receipt-derived subsets. Counts derive from retained files; every historical input must exist, and artifact hashes reject arbitrary edits. All sealed logs are force-tracked despite the repository's log ignore rule.

All twenty prior validators pass; together with this page, 21/21 pass. [Relocation proof](../../../data/patch-api/evidence/7.2.0-session-2026-10-08/p720-validator-portability.json) validates a complete Git-archive snapshot at a different root, then adds a hypothetical later-audit register/sweep: historical counts remain 44/45 and validation still passes. Tampered own ledger bytes fail. Original sealed artifacts remain byte-identical after validation/restoration. [All twenty prior validators also pass in that augmented relocated snapshot](../../../data/patch-api/evidence/7.2.0-session-2026-10-08/p720-relocated-prior-validator-matrix.json). Full Git history is required; no fallback or missing-history bypass was added.

## Sources

- [Pinned provenance](../../../data/patch-api/sources/7.2.0-api-changes.provenance.json).
- [Register](../../../data/patch-api/sources/7.2.0-wikitext-register.json).
- [Publication spec](../../specs/patch-7-2-0-publication-sweep.md).
- [Evidence](../../../data/patch-api/evidence/7.2.0-session-2026-10-08/).

## See Also

- [[patch-7-3-0-api-audit]] — prior-page evidence format.
- [[patch-audit-validator-portability]] — fixed historical scope, checkout-independent receipts.
