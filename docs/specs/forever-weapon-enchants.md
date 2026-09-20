# Weapon enchant queries

Forever `C_Item.GetWeaponEnchantInfo(weaponSlot)` returns a fresh list for MainHand (0), OffHand (1), or Ranged (2), never nil. Each active snapshot supplies `hasEnchant`, `enchantType`, `timeLeft` (milliseconds), `charges`, `enchantID`, and `enchantIconID`. Invalid slots, including 3, fail explicitly. Forever publishes all three `Enum.WeaponSlot` values and matching metadata.

Source: pinned Forever 1.60.1.69913 `ItemDocumentation.lua` (`WeaponEnchantInfo`) and `ItemConstantsDocumentation.lua`. The unchanged BuffFrame consumer divides the returned time by 1000; returned records must not alias simulator state.

Three per-environment slot lists hold server-provided snapshots. State replacement, removal, and remaining-time updates are reflected immediately. The legacy eight-return `GetWeaponEnchantInfo()` reads the Temporary (2) entry of main-hand and off-hand only from the same state; ranged snapshots never add returns; empty slots preserve the established false/0/0/0 tuple. Its temporary empty-result shim is removed.

Automatic countdown, enchant application/cancellation, persistence, and secret-value enforcement are outside this bounded snapshot/query change. No native conformance claim follows from simulator tests.

Grouped tests cover empty slots, populated slots, updates/removal, result isolation, environment isolation, legacy agreement, and the actual cached BuffFrame `UpdateTemporaryEnchantmentBuffs` function with its authored texture mapping.
