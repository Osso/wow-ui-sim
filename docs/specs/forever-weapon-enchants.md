# Weapon enchant queries

Forever `C_Item.GetWeaponEnchantInfo(weaponSlot)` returns a fresh list for MainHand (0), OffHand (1), or Ranged (2), never nil. Each active snapshot supplies `hasEnchant`, `enchantType`, `timeLeft` (milliseconds), `charges`, `enchantID`, and `enchantIconID`. Invalid slots, including 3, fail explicitly. Forever publishes all three `Enum.WeaponSlot` values and matching metadata.

Source: pinned Forever 1.60.1.69913 `ItemDocumentation.lua` (`WeaponEnchantInfo`) and `ItemConstantsDocumentation.lua`. The unchanged BuffFrame consumer divides the returned time by 1000; returned records must not alias simulator state.

Three per-environment slot lists hold server-provided snapshots. State replacement, removal, and remaining-time updates are reflected immediately. The legacy eight-return `GetWeaponEnchantInfo()` reads the Temporary (2) entry of main-hand and off-hand only from the same state; ranged snapshots never add returns; empty slots preserve the established false/0/0/0 tuple. Its temporary empty-result shim is removed.

Automatic countdown, enchant application/cancellation, persistence, and secret-value enforcement are outside this bounded snapshot/query change. No native conformance claim follows from simulator tests.

Grouped tests cover empty slots, populated slots, updates/removal, result isolation, environment isolation, legacy agreement, and the actual cached BuffFrame `UpdateTemporaryEnchantmentBuffs` function with its authored texture mapping.

## Temporary enchantment display query

Forever also publishes the existing
`C_PaperDollInfo.GetTemporaryEnchantmentInfo(inventorySlot)` implementation used
by Retail 12.1+. Inventory slots 16, 17 and 18 select the first Temporary (2)
snapshot from the same weapon-enchant lists. A match returns one fresh table
with `enchantID`, `remainingTimeMs`, `chargesRemaining` and `hasExpirationTime`.
An absent enchant or unsupported slot returns zero values.

`remainingTimeMs` projects the supplied snapshot without automatic clock aging.
The simulator infers `hasExpirationTime` from a positive snapshot time.
`CancelTemporaryEnchantment` is available with `retail-12-1-0`.

[Forever 70707 documentation](https://github.com/Gethe/wow-ui-source/blob/aff0d8cd53bf4d9ef1c1dd2786b94461a276fb16/Interface/AddOns/Blizzard_APIDocumentationGenerated/PaperDollInfoDocumentation.lua)
declares the query and result shape. The focused regressions verify the
selected AuraContainer display consumes the supplied snapshot and clears when
it disappears. [Forever regressions](../../tests/forever_temporary_enchantment.rs)
