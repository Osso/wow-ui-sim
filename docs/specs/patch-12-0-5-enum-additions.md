# Patch 12.0.5 enum additions

Publish three documented numeric enum additions from [retained patch notes](../../data/patch-api/sources/12.0.5-api-changes.txt). This contract covers Lua-visible numbers and metadata, not currency rewards, transmog eligibility, or housing suggestion behavior.

## What it must do

- [x] With `retail-12-0-5` enabled, publish `Enum.CurrencyFlagsB.CurrencyBNoBonusXP = 2048`, retaining all eleven earlier members; metadata is `MinValue=1`, `MaxValue=2048`, `NumValues=12`.
- [x] Publish `Enum.TransmogIllusionFlags.AllowedRangedShieldsHoldables = 4`, retaining values `HideUntilCollected=1` and `PlayerConditionGrantsOnLogin=2`; current retail metadata is `1/4/3` (minimum/maximum/count).
- [x] Publish `Enum.HouseFinderSuggestionReason.HomeOwner = 64`, retaining seven earlier members. For 12.0.5/12.0.7, metadata is `0/64/8`. Shared 12.1 compatibility already adds `Relinquished=65`; preserve it unchanged and refresh metadata to actual members (`0/65/9` in current retail), including after post-load restoration. Do not introduce or correct `Relinquished` through this patch.
- [x] Metadata reflects actual published numeric members rather than replacing later-epoch bounds/counts with 12.0.5 values.

Documented values: cached official `CurrencyConstantsDocumentation.lua:95–113`, `TransmogSharedDocumentation.lua:46–57`, and `PlayerHousingConstantsDocumentation.lua:52–69` under `~/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/`. The housing cache includes later `Relinquished=128`; that addition is excluded. Patch-note names occur at retained-source lines 573–574, 590–591, and 606–607.

## How it works

- [Lua API architecture](../lua-api.md)

## Implementation inventory

- `src/c_api/patch_12_0_5_enums.rs`: numeric publication and metadata.
- `src/c_api/mod.rs`: cumulative 12.0.5 feature gate.
- `src/lua_api/env_init/enums.rs`: registration after base missing/compat enums.
- `src/ptr/compat_bootstrap.rs`: refresh housing metadata after shared 12.1 member publication, initially and after load.

## Tests asserting this spec

- `tests/patch_12_0_5_enum_additions.rs`: public values, retained members, and metadata coherence in the grouped integration target, including shared housing post-load restoration. Focused default-retail execution passes all three tests; historical/PTR branches are not claimed as executed.
- `src/loader/tests/wow_api_globals/patch_12_1_5_transmog_illusion_flags.rs`: existing default-retail publication/post-load control passes. Numerical publication is not native-client/domain parity.

Revision-scoped commands, artifact hashes, and provenance limitations are recorded in `/tmp/patch-12.0.5-enums-ledger.md`.

## Known gaps (current cycle)

No remaining default-retail numerical-publication failure. Historical/PTR execution and full-build provenance reconciliation remain outside the bounded behavioral proof.

## Out of scope

- Domain semantic parity: publishing a flag does not implement its downstream domain behavior.
- Earlier profiles/epochs without `retail-12-0-5`; later `Relinquished` publication and PTR housing compatibility repairs.
- Vendor changes, broad checks, native-client probes, deployment, and push.
