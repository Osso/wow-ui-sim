# Patch 1.15.1 literal source and SeasonID alias audit

Frozen page577687/revision5998991/timestamp `2024-04-03T08:43:49Z`, verified2026-10-09. Exact legacy response/raw/manifest/registry identity retained. No linked retail or deprecated-file body imported into the frozen ledger. [Spec](../../specs/patch-1-15-1-source-accounting.md).

## Literal coverage matrix

| Source feature | Accounting | Behavior proof |
|---|---|---|
| Dragonflight10.2.5 unspecified subset, line4 | Prose limitation and separate unexpanded retail link | UNPROVEN; no subset membership imported |
| SeasonOfDiscovery addition, line5 | Named enum member; source numeric value null | UNPROVEN native numeric value |
| Placeholder deprecation, line6 | Concrete alias continuity to SeasonOfDiscovery, not generic placeholder publication | UNPROVEN pending direct existing-state proof |
| Diffs line10 / deprecated API line11 | Three unexpanded external link contracts | Bodies not frozen-page contracts |
| TOC11501, two headers, navigation | Metadata; template unexpanded | No native/runtime credit |

Nine nonblank rows: four metadata/five UNPROVEN; seven contract records, two API occurrences, zero signatures/event/CVar/widget/command declarations. No count-bearing headers. Literal expansion Dragonflight does not establish native client identity. Same-Era context comes from task, not template expansion.

## Existing enum and loading/public-build boundary

At base `7ad66791e64f01dd6dc61d29d3aa48faccdce3b8`, `missing_enums.lua:12378–12386` publishes SeasonOfDiscovery=2 and no Placeholder. `env_init/enums.rs` executes that initializer. Existing `utility_system_spell/mod.rs:380–384` returns true for IsPublicBuild, registered at689. No modeled C_Seasons implementation established; interned namespace alone is not a model.

Era manifest lines1315–1316 includes Blizzard_Deprecated transition guide/TOC, not Deprecated_1_15_1.lua. Profile cache `/home/osso/.cache/wow-ui-sim/blizzard-ui/era/AddOns` absent here; current official definitions and full loading unresolved. **Bare alias absence is a loading/public-build boundary, not established enum defect.** No production edit.

Main separately pinned unchanged mirrored Blizzard Lua at Gethe commit `967711a33ee9d3db2e7262e0bc0b49f4ef5a0013`:267bytes, SHA256 `aa34e2aa7b244efde499f8eb4767cb3b29bc7cbb4a583947c90996256ce512b3`. It returns early when not IsPublicBuild; otherwise assigns Placeholder from SeasonOfDiscovery. This independent file supports direct public-build assignment testing, not numeric/native parity, negative public-build branch or full addon loading. Frozen page remains unexpanded.

## Successors and configuration

Own snapshot Era/Anniversary11507 is not native1.15.1. Frozen1.15.2 in flight,1.15.3 queued,1.15.4–9 integrated-canonical inputs retained but not applied here. Retail10.2.5 attribution is not Era supersession. Main owns actual ordered integration.

## Proof ledger

Own SOURCE tests: RED8 assertion failures against empty-accounting scaffold retained. GREEN and portable proof pending. No broad/check/lint/type/final acceptance performed.

## Remaining gaps

Numeric/native correspondence; real realm seasonal state/C_Seasons; current Era cache and complete deprecated-addon loading; non-public-build branch; linked subset membership/diff/body contracts. Main owns native/integration/final gates.

## Sources

- [Frozen source pin](../../../data/patch-api/evidence/1.15.1-session-2026-10-09/source-pin.json) — Warcraft Wiki CC BY-SA4.0; exact request/retrieval/hash.
- [Literal ledger](../../../data/patch-api/evidence/1.15.1-session-2026-10-09/ledger.json).
- [Model investigation](../../../data/patch-api/evidence/1.15.1-session-2026-10-09/model-investigation.json) — snapshot observations and limitations.
- [Separate external source pin](../../../data/patch-api/evidence/1.15.1-session-2026-10-09/external-primary-source/source-pin.json) — unchanged Blizzard Lua distribution mirrored by Gethe; supplied by main.

## See Also

- [[patch-1-15-4-api-audit]] — later same-Era source scope.
- [[client-profiles]] — configured profile identity.
