# Forever unit interactions

Forever exposes read-only `UnitIsGameObject`, `UnitHasLootInteraction`, `UnitIsInInteractRange`, and `UnitIsInteractable` boolean queries, matching cached build 1.60.1.69913 `UnitDocumentation.lua` signatures and `MainActionBarFrame.lua` consumers.

Interaction capabilities live on existing `TargetInfo` snapshots: game-object identity, available loot, interactability, and range. Initial capabilities are false until simulator input supplies them. Queries resolve existing target/focus/enemy/player/party tokens; unknown or absent selections return false. `softinteract` aliases an existing selection token, not a second unit database; changing or clearing that selection affects its queries immediately. A resolved soft game object is detectable through `UnitIsGameObject` but is not a living unit for `UnitExists`.

`UnitIsGameObject` replaces the existing cross-profile false stub with modeled behavior. The other three new registrations are Forever-only. No actions, loot mutations, range simulation, or native security semantics are added.

Tests exercise unchanged vendor `UpdateInteractIcons`: absent/attackable targets, loot in/out of range, interactable targets, soft game objects and loot priority. Icon paths and preferred target outputs are asserted. Fixture hooks supply cursor-rendering and input-binding context; interaction decisions and selection queries remain real. This is simulator state coverage, not native conformance.
