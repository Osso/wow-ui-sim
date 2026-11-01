# Store promotional upgrade default

Retail's `Blizzard_StoreUI.toc:24` loads `Blizzard_Shared_StoreUI.xml:6`, which loads `Blizzard_Shared_StoreUISecure.lua`. Its `StoreFrame_CheckForFree` calls `C_SharedCharacterServices.HasFreePromotionalUpgrade` at line 1833. The pinned Retail generated API documentation does not declare this function.

## Best-guess basis (best-guess, native UNKNOWN)

The simulator does not model account promotional upgrades. On Retail only, an account has no free promotional upgrade by default, so this temporary query returns false. Replace the default when account/store upgrade state is modeled.

## Test

`store_tree::store_free_upgrade_check_handles_account_without_upgrade` asserts the query returns `false` (not nil), then loads the original StoreUI and calls its secure check after both relevant events.

## Known gaps

- Native default and argument handling are unknown; the function is absent from the pinned Retail generated docs.
- Account promotional-upgrade state, the free-upgrade dialog branch of `StoreFrame_CheckForFree`, and PTR/other profiles are not modeled.
