# Patch 12.0.5 enum additions

Publish three documented numeric enum additions from [retained patch notes](../../data/patch-api/sources/12.0.5-api-changes.txt). This contract covers Lua-visible numbers and metadata, not currency rewards, transmog eligibility, or housing suggestion behavior.

## What it must do

- [ ] With `retail-12-0-5` enabled, publish `Enum.CurrencyFlagsB.CurrencyBNoBonusXP = 2048`, retaining all eleven earlier members; metadata is `MinValue=1`, `MaxValue=2048`, `NumValues=12`.
- [ ] Publish `Enum.TransmogIllusionFlags.AllowedRangedShieldsHoldables = 4`, retaining values `HideUntilCollected=1` and `PlayerConditionGrantsOnLogin=2`; current retail metadata is `1/4/3` (minimum/maximum/count).
- [ ] Publish `Enum.HouseFinderSuggestionReason.HomeOwner = 64`, retaining seven earlier members; current retail metadata is `0/64/8`. Do not introduce `Relinquished` through this patch.
- [ ] Metadata reflects actual published numeric members rather than replacing later-epoch bounds/counts with 12.0.5 values.

Documented values: cached official `CurrencyConstantsDocumentation.lua:95–113`, `TransmogSharedDocumentation.lua:46–57`, and `PlayerHousingConstantsDocumentation.lua:52–69` under `~/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/`. The housing cache includes later `Relinquished=128`; that addition is excluded. Patch-note names occur at retained-source lines 573–574, 590–591, and 606–607.

## How it works

- [Lua API architecture](../lua-api.md)

## Implementation inventory

- `src/c_api/patch_12_0_5_enums.rs`: numeric publication and metadata.
- `src/c_api/mod.rs`: cumulative 12.0.5 feature gate.
- `src/lua_api/env_init/enums.rs`: registration after base missing/compat enums.

## Tests asserting this spec

- `tests/patch_12_0_5_enum_additions.rs`: public values, retained members, and metadata coherence in the grouped integration target. Default-retail execution is the bounded proof; later-profile branches are not claimed as executed.

## Known gaps (current cycle)

- [ ] Focused default-retail RED/GREEN proof pending.

## Out of scope

- Domain semantic parity: publishing a flag does not implement its downstream domain behavior.
- Earlier profiles/epochs without `retail-12-0-5`; later `Relinquished` publication and PTR housing compatibility repairs.
- Vendor changes, broad checks, native-client probes, deployment, and push.
