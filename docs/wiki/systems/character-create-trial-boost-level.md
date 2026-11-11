# Character-create trial boost level

`C_CharacterCreation.GetTrialBoostStartingLevel` reads the selected boost level from a host-supplied `CharacterServiceInfo` row. Its public contract is in the [feature spec](../../specs/character-create-trial-boost-level.md).

## Content

### State and lookup

- **VERIFIED (R61):** the integer selector starts at `2`; its host setter always stores the supplied value and emits no event.
- **VERIFIED (R61):** the model stores `{ServiceType, BoostType, BoostLevel}` rows and the getter returns the first match in table iteration order: `ServiceType == 1` and `BoostType == selector`, or a `ServiceType == 0`, `BoostType == 0` row for selector `0`. No match returns `0`. The simulator preserves supplied vector order.
- **VERIFIED (local export):** Retail state uses the generated rows in `src/c_api/c_character_creation_trial_boost_data.rs`; the selector-2 row has level `100`.
- WowForever starts with an empty row table. **VERIFIED:** the checked local Forever payload is encrypted and its key is missing. Native Forever row values remain **UNKNOWN**; the getter's no-match result is `0`.

### Registration

- **VERIFIED (R61):** Retail and WowForever register the getter through the Glue catalog. **INFERRED:** native availability is Glue-only because the checked Game catalogs do not install the namespace. The simulator removes the member on Game screens.
- PTR keeps its existing zero fallback. Native PTR availability and result behavior remain **UNKNOWN**.

### Runtime evidence and boundaries

- **VERIFIED (Retail 12.1.5.70783 simulator):** the screen-registration, getter/arity, host-selector, original tooltip, zero-selector, no-match and exact-export tests pass.
- **VERIFIED (Forever simulator at `091aac3ce`):** the Glue/Game registration and empty-row tests pass. The three broader CharacterCreate filters fail before the trial-level tooltip consumer at nil `C_CharacterCreation.SetInCharacterCreate` in `Blizzard_CharacterCreate/Camelot/Blizzard_CharacterCreate.lua:221`. The action-update and transition filters also log nil `GetSelectedSuperDistrictID` and `HasQueuedUpgrade` calls; those APIs are outside this contract. The direct original tooltip consumer test is added, but its run is pending.
- **UNKNOWN:** Forever native rows and PTR native getter availability remain unknown; see the [feature contract](../../specs/character-create-trial-boost-level.md#known-gaps-current-cycle).

### Retail data regeneration

`data/character-service-info/retail-12.1.5.70783.csv` is the exact CSV exported from local Retail build `12.1.5.70783`. The adjacent provenance file records the FDID, WDC5 layout, scanned WoWDBDefs hash, exporter hash and CSV hash. `python3 tools/gen_character_creation_trial_boost.py` verifies the CSV and definition hashes, then writes the Rust tuple table without reordering the CSV. **VERIFIED (exporter source and CSV):** the exporter sorts records by ID, and the 13 `(ServiceType, BoostType)` pairs are unique. Thus current lookup results do not depend on this export order; future duplicate keys require the host's actual table iteration order.

**VERIFIED:** the selector-2 row agrees with the separate R61 observation from Retail `12.1.5.70688`; the installed `70783` data is labeled with its own build and was not adjusted to match the older observation.

## Sources

- [Feature contract](../../specs/character-create-trial-boost-level.md)
- [Retail CSV](../../../data/character-service-info/retail-12.1.5.70783.csv)
- [Retail provenance](../../../data/character-service-info/retail-12.1.5.70783.provenance.json)
- [C API model](../../../src/c_api/c_character_creation_trial_boost.rs)
- [Data generator](../../../tools/gen_character_creation_trial_boost.py)

## See Also

- [[lua-api]]
