# SOUNDKIT constants

`SOUNDKIT` is the Blizzard sound-kit id table. Each profile's pinned
`Blizzard_SharedXML/<flavor>/SoundKitConstants.lua` assigns the whole table, and
the addon loader publishes it for the full UI (see
[addon loading](../addon-loading-pipeline.md)).

## What it must do

- [x] The pinned table is published by the addon loader, selected through the
      profile TOC: Retail `Blizzard_SharedXML.toc:157`
      `[Family]\SoundKitConstants.lua`, PTR/WowForever `:163`, Mists
      `Blizzard_SharedXML_Mists.toc:5` (`## AllowLoadGameType: mists`) and `:45`
      `Wrath\SoundKitConstants.lua`, Era/Anniversary
      `Blizzard_SharedXML_Vanilla.toc:5` and `:44`
      `Vanilla\SoundKitConstants.lua`.
- [x] A simulator fixture that runs a Blizzard consumer without the full UI loads
      that same pinned file first, so ambient globals the real client always has
      are present.
- [x] The runtime surface also publishes the nine ids that secure-environment
      addons read before Blizzard UI loads: eight `CATALOG_SHOP_*` values and
      `IG_INVENTORY_ROTATE_CHARACTER`. Each value must equal the pinned table.
- [ ] No id outside that secure-environment set is published by the runtime
      surface; every other id comes from the pinned file.

## How it works

- [addon loading pipeline](../addon-loading-pipeline.md)
- `src/lua_api/globals/security/secure_env.rs` — `__secureenv` is a snapshot of
  `_G` taken during env init, before Blizzard addons load. Secure addons
  (`## UseSecureEnvironment: 1`) resolve globals through it, so they cannot see
  the Blizzard `SOUNDKIT` table.

## Implementation inventory

- `src/lua_api/env_init/shared_bootstrap.lua` — publishes the secure-environment
  ids with the pinned values and documents the snapshot constraint.
- `src/iced_app/quad_builders_alpha_gradient_tests.rs` — `consumer` fixture loads
  the TOC-selected sound-kit file before running the original QuestFrame code.

## Tests asserting this spec

- `src/iced_app/quad_builders_alpha_gradient_tests.rs`
  (`consumer::cached_quest_consumer_*`, Mists/Wrath) — the Blizzard QuestFrame
  consumer reaches its gradient branch, which requires
  `PlaySound(SOUNDKIT.IG_WRITE_QUEST)`.
- `src/lua_api/globals/security/secure_env.rs`
  (`secure_environment_includes_soundkit_table`) — secureenv carries `SOUNDKIT`.

## Known gaps (current cycle)

- [ ] `__secureenv` never tracks `_G` after Blizzard UI load, so every Blizzard
      global defined in Lua after env init is invisible to secure code. The
      `SOUNDKIT` seed covers one consumer; the root fix is a post-load secureenv
      synchronization.
- [ ] The nine seeded values duplicate pinned table entries and can drift on the
      next pin; no test compares them with the pinned file.
- [ ] No test asserts that the full-UI loader publishes the TOC-selected file.

## Out of scope

- Modelling sound kits as a `C_Sound` catalog: `SOUNDKIT` is plain Blizzard Lua
  data, not an engine query.
