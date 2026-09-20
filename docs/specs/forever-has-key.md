# Forever keyring inventory query

## Contract

Forever exposes the zero-argument boolean `HasKey()` global declared in build 1.60.1.69913 `PlayerScriptDocumentation.lua`. `BagIndexConstantsDocumentation.lua` assigns `Enum.BagIndex.Keyring = -1`.

The simulator returns true when `SimState.bag_items` contains a positive-slot, nonzero-item, positive-count entry in container -1. Removing that entry or reducing its count to zero restores false. Ordinary bags do not count: item-class detection outside the keyring is not modeled by this predicate. This is an explicit inventory-membership policy, not native proof of every key-carrying location.

Other profiles retain their existing surface. No Admin API, server mechanics, or vendor changes are introduced. `C_ActionBar.ShouldShowKeyring` remains a separate query.

## Proof

`tests/wowforever_has_key.rs` covers empty/populated/removed inventory, zero stacks, ordinary-bag exclusion, return arity, environment isolation, and the actual KeyRing tutorial consumer with the tutorial already acknowledged. Tutorial triggering/pulsing is not claimed by that consumer test.
