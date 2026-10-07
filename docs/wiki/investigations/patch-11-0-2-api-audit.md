# Patch 11.0.2 API page audit

Page 595347, revision 6862982 (September 6, 2026, 02:25:37 UTC), retrieved October 7, 2026 UTC. Page exists; no earlier-page substitution. Branch p1102-page starts from p1105-page revision 0286a243d. Default retail carries 12.1.0, not reconstructed 11.0.2.

## Source and accounting

All six inventory header counts match: 26 added, four removed, four changed occurrences. Thirteen later registers, 11.0.5 through 12.1.0, supersede chronologically. Three historical added CVars now expect absence: UseOldVolumeFog, minimapTrackedInfov2 and useCompactPartyFrames. Source hashes/revision and line IDs remain retained. Non-inventory text retains Settings_API transclusion as unexpanded external context.

263 unique source IDs: 34 inventory + 229 extract. Ledger: 15 partial-development-green, seven bounded-coverage, 213 audit-pending, 28 metadata-only. Every extract ID assigned once across eight settings/macro statements, 20 enum rows, 42 structure rows, 63 deprecation/removal rows, 68 retained-deprecation rows and 28 editorial rows. Source candidates do not confer runtime credit. Changed annotations, including event payload names/types, receive publication-only proof.

## Bounded fixes and retained gaps

Initial sweep: 18 OK / 16 gaps. Namespace autostub fabricated three removed members: C_GameModeManager.GetFeatureSetting, C_GameModeManager.IsFeatureEnabled and C_WeeklyRewards.GetWeeklyRewardTextureKit. Read-only qualified/bare searches found no applicable current cached consumer. Retirement markers now preserve absence through repeated ordinary/raw lookup without deleting cached deprecation wrappers; neighbor queries remain callable.

C_Item.IsItemBindToAccountUntilEquip lacked an explicit producer. It now reads intrinsic generated bonding metadata. Cached ItemConstantsDocumentation.lua:179–195 names ItemBind 9 ToBnetAccountUntilEquipped. Concrete Veilroot Fountain ID 253451, numeric string, bare link and hyperlink return true; Thunderfury ID 19019, unknown ID and invalid string return false. This is intrinsic metadata, not inventory-instance binding/refund/ownership or native security parity.

Four closures leave 12 exact publication gaps. [Per-ID review](../../../data/patch-api/evidence/11.0.2-session-2026-10-06/p1102-gap-review.json) explains missing layout-name policy, guild event access, item refund/reward selection, map-label geometry, minimap inset/render model, quest reputation/first-time bonus data, trait edit permission, delve visualization DTO and voice settings access. No placeholder publication added.

[Exhaustive extract scout](../../../data/patch-api/evidence/11.0.2-session-2026-10-06/p1102-extract-scout.json) retains each statement, candidate file:line and precise proof boundary. Macro dispatcher has no text length limit and omits /click; neither establishes the secure macrotext/client chaining contract. Settings saved-table/proxy callbacks/navigation need full addon-origin interaction proof. Enum numeric values, populated DTO serialization and historical deprecated successor behavior remain pending. Deprecated rows explicitly retained on the historical beta are not recast as removal requirements.

## Verification

Runtime revision aefaa9605. Two behavioral tests RED before fixes, GREEN afterward. Fourteen isolated publication sweeps pass unchanged later fixtures; [table](../../specs/patch-11-0-2-publication-sweep.md#local-proof). Negative control changes only the new item query added → removed: exactly one new failure, no resolved failures, 12 → 13 gaps and expected exit 101. Two isolated one-filter prefork runs pass: new retirement/item surfaces and existing weekly-reward addon load. Eighteen extractor/register fixtures pass. Fourteen registers regenerate byte-identically; 78 later source/register/fixture/ledger inputs match branch base unchanged.

Formatting passes. Mists test check passes with zero non-vendor warnings; six existing iced vendor manifest deprecations plus summary remain unsuppressed. Separate retail build and bounded startup exit 0 with JSON `[]`. Changed Rust lines reviewed manually; no readability findings. Commands/revisions/outcomes retained in [proof ledger](../../../data/patch-api/evidence/11.0.2-session-2026-10-06/p1102-proof.json). [Artifact validator](../../../data/patch-api/evidence/11.0.2-session-2026-10-06/p1102-validate.py) checks source hashes, supersession, exact gaps and all source IDs. Artifact accounting validation passes at b0cf82ee3; subsequent edits only retain this result and documentation, not validated source/runtime scopes. [Validation result](../../../data/patch-api/evidence/11.0.2-session-2026-10-06/p1102-validation-result.json) preserves the command and output. No independent/native acceptance claim.

Every command uses explicit cwd p1102-page and its own target directory. Worktree creation uses the prescribed canonical Git metadata operation with cwd in the empty destination. No canonical working files, siblings, cache/vendor Lua or Wowless edited; no full suite, agents/models, push or merge.

## Sources

- [Source provenance](../../../data/patch-api/sources/11.0.2-api-changes.provenance.json)
- [Page coverage ledger](../../../data/patch-api/sources/11.0.2-page-coverage.json)
- [Publication contract](../../specs/patch-11-0-2-publication-sweep.md)
- [Retirement searches](../../../data/patch-api/evidence/11.0.2-session-2026-10-06/p1102-retirement-consumers.json)

## See Also

- [[patch-11-0-5-api-audit]] — read-only branch base and first supersession register.
- [[patch-11-0-7-api-audit]] — exhaustive accounting/proof conventions.
- [[client-profiles]] — current supported retail/classic boundaries.
