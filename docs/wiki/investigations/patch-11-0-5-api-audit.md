# Patch 11.0.5 API page audit

Page 601519, revision 6726778 (May 25, 2026, 20:32:26 UTC), retrieved October 7, 2026 UTC. Evidence directory retains the requested October 6 session label. Audit accounts for 48 inventory occurrences and 34 non-inventory rows. Default retail carries 12.1.0, not a reconstructed 11.0.5 client. Branch p1105-page starts at p1107-page revision ff1d68d24; later audit inputs remain unchanged.

## Source and supersession

All eight added/removed header counts match: 30 added, nine removed and nine changed inventory occurrences. Twelve later registers, 11.0.7 through 12.1.0, apply chronologically; latest add/remove wins. No effective reversal occurs for these 48 rows. Wikitext is authoritative; external resource links are not expanded. Read-only browser MediaWiki query retained exact revision content and provenance.

Every command ran with explicit cwd p1105-page. Worktree creation used the prescribed canonical Git worktree operation, with command cwd in the newly created empty destination. All build artifacts use this worktree's own target directory; no sibling targets or copied binaries. No cached/vendor Lua, Wowless sources, canonical working files or other worktrees were edited.

## Root causes and bounded fixes

Initial cached sweep: 33 OK / 15 gaps. Two removed members were fabricated by namespace autostub lookup: C_AuctionHouse.RequestFavorites and C_MajorFactions.GetCovenantIDForMajorFaction. Full current retail-cache searches for both qualified and bare member names found no matches. Existing retirement policy now blocks repeated ordinary/raw lookup on supported retail epochs. Neighbor GetBrowseResults/GetMajorFactionData remains callable; classic registration unchanged.

C_BarberShop.HasAlteredForm lacked an explicit producer despite the existing current-character alternate-race snapshot. It now derives availability from that snapshot, independently of which form is viewed. Tests cover absent character, populated Worgen alternate race, both viewing selections, removing alternate race and clearing character. Cached BarberShopDocumentation.lua:81–87 declares a nonnil bool but not eligibility policy. Snapshot derivation is bounded simulator policy, not proof of native form eligibility.

ChromaEffectsEnable and ChromaEffectsFactionColor lacked CVar registry entries despite page default 1. Both now use existing mutable CVar storage, case-insensitive setters and immutable defaults. Tests cover global/C_CVar queries, mutation and restore. This models configuration, not physical peripheral lighting/faction colors.

Three new behavior tests fail before their respective fixes and pass afterward. Five publication closures leave ten exact gaps. [Per-ID review](../../../data/patch-api/evidence/11.0.5-session-2026-10-06/p1105-gap-review.json) retains each initial defect, observation and precise closure/boundary.

**Preserved deprecation surfaces:** cached Blizzard_DeprecatedGlue/Deprecated_Glue.lua:9 assigns `IsOnGlueScreen = C_Glue.IsOnGlueScreen();`, a boolean snapshot. Strict absence therefore remains an exact compatibility gap; neither vendor assignment nor classifier was changed to hide it. Cached Blizzard_DeprecatedSpecialization/Deprecated_Specialization_Standard.lua aliases GetNumSpecializationsForClassID to C_SpecializationInfo.GetNumSpecializationsForClassID. Shared sweep accepts that loaded alias identity, not strict absence or successor behavior. Both isolated prefork tests pass. A third prefork case proves new retirements, barber query and Chroma storage survive unmodified full cached Game preload without new Lua errors.

Remaining publication gaps: chat/combat logging lifecycle, four currency filter/backpack/account-data queries or mutations, outgoing-chat permission, transmog valid-class selection, retail ClassicExpansionAtMost, and retained deprecated IsOnGlueScreen boolean. Currency getters still use static currency_data lists; a disconnected setter would not model filtering/backpack behavior. Transmog class_mask exists but does not establish which class to return for zero/multiple classes. No placeholder publication added.

## Coverage matrix

| Statements/features | Count | Proof / boundary |
|---|---:|---|
| Unsuperseded add/change publication/defaults | 30 | Partial-development-green; not full signature/output/security/behavior proof |
| Unsuperseded removals | 8 | Seven strict bounded absence probes; one loaded deprecated alias acceptance |
| Inventory gaps | 10 | Exact producer/model/policy or retained-compatibility boundaries |
| Non-inventory enum/structure contracts | 18 / 6 | Exhaustive ranked scout; candidates only, no runtime credit |
| Non-inventory editorial context | 10 | Metadata-only |

82 unique IDs: 30 partial-development-green, eight bounded-coverage, 34 audit-pending, ten metadata-only. Publication accounting is complete; ledger remains in-progress because behavior/producer gaps remain. Every extract ID is assigned once. Current enum tables and DTO serializers are candidates, not page-specific behavioral proof. Inventory changed annotations—including StatusBar success return, attack-power returns and event payload changes—retain publication-only scope and receive no inferred semantic credit.

## Development proof

[Proof ledger](../../../data/patch-api/evidence/11.0.5-session-2026-10-06/p1105-proof.json) records exact command/cwd/environment/revision/outcome and failed/superseded scopes. Runtime revision 5fb610fa3; earlier retirement/barber revision 9ebd57076. Later sweeps ran at b3113d0f7; subsequent change only adds two Chroma CVar defaults and corresponding tests/fixture, none of which occurs in later registers. Their publication proof remains valid without redundant reruns. Later evidence/docs edits do not invalidate runtime scope.

All thirteen sweeps run alone and pass, matching unchanged later fixtures. [Sweep table](../../specs/patch-11-0-5-publication-sweep.md#local-proof) retains row/OK/gap counts. Negative control changes only C_BarberShop.HasAlteredForm added → removed: exactly one new gap, none resolved, 10 → 11, expected exit 101. Three new behavior tests GREEN after RED; three one-filter prefork cases GREEN. Seventeen extractor/register fixtures pass. All thirteen byte-identical register regenerations, deterministic extract reproduction, source hashes, exact fixture and exhaustive artifact accounting validate at 2bf022deb with newly generated reproduction/proof inputs. Later evidence/docs/result records do not change validated capability/source scopes.

Formatting passes. Mists test check has zero non-vendor warnings; six pre-existing iced manifest deprecations plus vendor summary remain unsuppressed. Separate retail build and bounded exit-0 startup return []. Changed Rust lines manually reviewed for readability. Cargo output retained once in ignored local logs and inspected with rg; tracked summaries preserve results without depending on local logs. No full suite, agents/models, independent/native acceptance, push or merge.

## Sources

- [Publication contract and sweep table](../../specs/patch-11-0-5-publication-sweep.md)
- [Source provenance](../../../data/patch-api/sources/11.0.5-api-changes.provenance.json)
- [Page ledger](../../../data/patch-api/sources/11.0.5-page-coverage.json)
- [Extract scout](../../../data/patch-api/evidence/11.0.5-session-2026-10-06/p1105-extract-scout.md)
- [Retirement consumer searches](../../../data/patch-api/evidence/11.0.5-session-2026-10-06/p1105-retirement-consumers.json)
- [Artifact validator](../../../data/patch-api/evidence/11.0.5-session-2026-10-06/p1105-validate.py)

## See Also

- [[patch-11-0-7-api-audit]] — branch base and immediate supersession boundary.
- [[patch-11-1-0-api-audit]] — exact-gap/page-ledger conventions.
- [[client-profiles]] — supported profiles and current retail epoch.
