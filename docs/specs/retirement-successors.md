# Retired 11.x API successors

Retail 12.0.0+ keeps all 21 symbols in `tests/data/patch_12_0_0_deprecated.json` absent natively and after cached Blizzard loading. Successor probes must pass without a known-gap allowance. Earlier legacy publication tests remain epoch-gated; model tests use current namespaces.

| Successor | Contract and modeled scope |
|---|---|
| C_SpellBook.GetSpellBookItemName | Name and subName from seeded spellbook/spell catalog; invalid slot has no name. |
| C_Spell.GetSpellTexture | iconID and originalIconID are file data IDs, never paths; unknown spell returns nothing. 12.1.0 also returns nil conditionalIconID until conditional icons are modeled. |
| C_MerchantFrame.GetItemInfo | Ordered `merchant_items` selects catalog identity. Missing slot/item returns nothing. ID-only host offers mean free, single-unit, unlimited stock, purchasable/usable, no extended cost or quest starter; merchant economics are not modeled. |
| C_ChallengeMode.GetChallengeCompletionInfo | Host completion snapshot with all documented fields and members, independent of weekly run history. Empty state yields documented zero/false/nil/empty-list record; time is milliseconds. |
| C_Log.LogMessage | Appends message to simulator console sink, returns zero values. |
| C_SpellActivationOverlay.IsSpellOverlayed | Boolean host proc membership; absent membership is false. No automatic proc generation. |

`C_Spell.IsSpellUsable` identifies usable known spells independently of cooldown readiness; cooldown model tests observe `C_Spell.GetSpellCooldown().isActive`. Removed weekly-reward API has no listed successor, so its legacy state-surface tests are gated rather than assigning a new contract to another namespace.

Proof: native behavior tests in `tests/retirement_successors.rs`, model migrations in existing integration tests, and cached retirement/successor proof in `tests/patch_12_0_0_deprecated.rs`. Cached generated API docs and unmodified consumer Lua are the compatibility target. No page-coverage ledger changes.
