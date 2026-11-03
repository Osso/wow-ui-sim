# Forever 70707 addon startup compatibility

A Forever 70707 game startup with SavedVariables enabled reached the first
frame and exposed missing queries and duplicate secure state dispatch.

## Content

### Cooldown Viewer in Edit Mode

The Game Menu's Edit Mode button refreshes the Forever Cooldown Viewer.
`CooldownViewerItemData.lua` enumerates totem slots with `GetNumTotemSlots()`.
Forever now reads that count from the same slot state as `GetTotemInfo()`.
Active slots include the supplied spell ID and rate. When two totems share a
spell ID, the Blizzard cache keeps the slot with the latest expiration.

The next refresh step scans legacy talent slots for icons. Forever exposes
`C_SpecializationInfo.GetTalentInfo()` with the existing query validation.
The query returns nil for the unmodeled tier-and-column grid, so the icon
provider skips those rows and continues collecting icons.

The Edit Mode regression enters through the Game Menu and checks the Blizzard
cache with populated totem slots. In the Windows 70707 check, Edit Mode became
active, the default slot count returned four, and the `lua-errors` report was
empty.

### Startup failures

- AuraContainer needed the shared caster, carryover and enchantment queries.
  RXPGuides needed the predicate for Forever's public aura policy.
  [Carryover](../../specs/aura-container-carryover-private-caster.md) ·
  [Weapon enchantments](../../specs/forever-weapon-enchants.md#temporary-enchantment-display-query) ·
  [Aura secrecy](../../specs/spell-aura-secrecy.md#current-aura-output-predicate)
- In the startup capture, a missing surname separator interrupted
  `CompactRaidFrameManager` before its divider pool initialized. The focused
  test covers the name formatter. [Name formatter test](../../../tests/wowforever_character_name_constants.rs)
- The partial `C_Widget` namespace bypassed MoveAny's fallback. AuraContainer
  also needed registered subtypes to report `Animation` ancestry.
  [Widget recognition](../../specs/widget-recognition.md) ·
  [Animation ancestry](../../specs/animation-factory.md#animation-ancestry)
- The secure setter replayed Blizzard's state snippet with a raw frame after
  the restricted callback ran, breaking `RunAttribute` and frame references.
  [State snippet ownership](../../specs/secure-attribute-delegation.md#state-snippet-ownership)
- The gamepad action bar needed pending pet orders, LockInTools needed local
  channel membership, and EllesmereUIRaidFrames needed Blizzard's encounter
  wrapper to read live state.

### Verification

The focused tests vary simulator state and exercise selected consumers.
Set `MOVEANY_UI_PATH` and `LOCKINTOOLS_UI_PATH` for their installed-addon cases.
For a startup comparison, use `lua-errors` with matching caches, addons and
read-only WTF imports in separate writable sessions. Compare failure causes
because Blizzard can report the same error twice. [Runtime inputs](../../runtime-inputs.md)

## Sources

- [Name constants](../../../src/c_api/forever_finite_constants.rs) and
  [caster GUID tests](../../../tests/forever_aura_caster_guid.rs).
- [Pet-order tests](../../../tests/forever_pet_move_order.rs),
  [channel tests](../../../tests/forever_permanent_channels.rs), and
  [encounter tests](../../../src/c_api/c_instance_encounter.rs).
- [Totem queries](../../../src/lua_api/globals/real/totems.rs),
  [talent query](../../../src/c_api/c_spec.rs), and
  [Edit Mode regression](../../../tests/edit_mode_api.rs).

## See Also

- [[forever-login-startup]] — graphics initialization before game startup.
- [[consumer-selection-and-red-proof]] — selecting the original consumer.
