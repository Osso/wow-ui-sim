# Free Promotional Character Upgrade

`C_SharedCharacterServices.HasFreePromotionalUpgrade` reports whether a free promotional character upgrade is available. Native probes verify the shared namespace on Retail, PTR, and Forever, and verify Forever registration on both Game and Glue. The simulator does not model the upgrade inventory or promotion classifier, so its temporary default returns `false`.

## What it must do

- [ ] Retail, PTR, and Forever expose the no-argument query in `C_SharedCharacterServices`; it returns exactly one boolean.
- [ ] Return `false` when the upgrade list is empty or no qualifying promotion is available.
- [ ] Keep `C_StorePublic.HasFreePromotionalUpgrade` absent.
- [ ] Forever exposes the query on both Game and Glue.

Native evidence: R22 (Retail 12.1.5 and PTR 12.1.7) and R27 (Forever 1.60.1). The true-result promotion classifier and packet-to-record path are not modeled here.

## How it works

- [Shared character services](../wiki/systems/shared-character-services.md)

## Implementation inventory

- `src/lua_api/workarounds/temporary/shared_character_services_defaults.rs`: supplies the verified false result while upgrade inventory state is not modeled.
- `src/lua_api/workarounds/mod.rs`: installs the temporary defaults.

## Tests asserting this spec

- `tests/glue_character_select.rs::free_promotional_upgrade_uses_shared_character_services`
- `tests/glue_character_select.rs::forever_free_promotional_upgrade_is_available_on_glue`
- `tests/store_tree.rs::store_free_upgrade_check_handles_account_without_upgrade`

## Known gaps (current cycle)

- [ ] The shared upgrade inventory and true-result promotion classifier are not modeled. The simulator returns `false` until that backing state is supplied.

## Out of scope

- Native server-message parsing and the unverified upgrade-record fields used to select a promotional product.
