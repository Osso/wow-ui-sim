# Forever 70707 addon startup compatibility

A Forever 70707 game startup with SavedVariables enabled reached the first
frame and exposed missing queries and duplicate secure state dispatch.

## Content

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

## See Also

- [[forever-login-startup]] — graphics initialization before game startup.
- [[consumer-selection-and-red-proof]] — selecting the original consumer.
