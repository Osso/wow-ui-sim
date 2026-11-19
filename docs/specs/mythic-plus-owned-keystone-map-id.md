# C_MythicPlus.GetOwnedKeystoneChallengeMapID and GetOwnedKeystoneLevel

Status: implemented simulator policy backed by native decomp evidence.
Authority: pinned declarations plus the clean-room decomp answers, round 3 Q3
(`pr46-49-decomp-answers.md`, lane handoff).

Evidence base: Forever 1.60.1.70707 for the getter path. Retail 12.1.5.70688 was
reconstructed for the `GetActiveKeystoneInfo` globals only. PTR 12.1.7.70668 was
**not analysed**, and Retail 12.1.5.**70575 is not held**, so neither build proves
anything here. The simulator applies the contract below to Retail, PTR and
Forever as it does today; Mists, Era and Anniversary carry the same pinned
declaration and get the same treatment.

## What it must do

- [ ] `GetOwnedKeystoneChallengeMapID()` searches the player's cached keystone
      context for the entry that carries the keystone field marker and publishes
      that entry's challenge-mode map ID.
- [ ] `GetOwnedKeystoneLevel()` searches the same context for the field with the
      level tag and publishes that value.
- [ ] Neither getter scans bag items or the item cache, and neither allocates a
      context.
- [ ] With a context present and no field carrying the wanted tag, the getter
      publishes the integer `0`.
- [ ] With the context absent — no player, or a player that owns no qualifying
      entry — the getter publishes **nothing** (zero results), not `0`.
- [ ] `GetOwnedKeystoneMapID()` returns nothing without an owned keystone. With
      one, it maps the owned `challenge_map_id` through the generated
      `MapChallengeMode` table and returns that row's `MapID`; a missing row
      returns nothing. This mapping is **INFERRED (best-guess, pending R88bp)**;
      the table data is **VERIFIED** from MapChallengeMode build 12.1.5.70391.
- [ ] Publication is every profile with a pinned declaration: Retail, PTR,
      Forever, Mists, Era and Anniversary. Wrath pins no generated documentation.

## Context existence (Q3)

The client asks for the current player. A player record is searched for the first
candidate whose record resolves in the challenge-mode record table, whose first
field equals one fixed value, whose second field is true, and which carries the
keystone field marker in its own field table. No candidate passes, or no player
exists, so the search yields nothing at all — not an empty context.

| situation | published result | proof |
|---|---|---|
| context present, field with the wanted tag found | that field's value | inferred |
| context present, no field with the wanted tag | integer `0` | proved |
| context absent | nothing (nil) | proved |

Both absence causes are ordinary, reachable states and neither is an error.

## Simulator model

`MythicPlusState::owned_keystone: Option<OwnedKeystone>` in
`src/lua_api/state_types/mythic_plus_scenario.rs`.

- `None` is the absent context. It is the default: a fresh simulator has no
  player-owned qualifying entry, so both getters publish nothing.
- `Some(OwnedKeystone { .. })` is a present context. A zero `level` or
  `challenge_map_id` field models the proved "present, no matching field"
  publisher arm, so the `0` arm stays reachable from current inputs without a
  second flag.

No admin setter or saved-variable path writes this state; tests are the only
writer today.

## Consumers in the pinned Blizzard sources

`GetOwnedKeystoneLevel` has three consumers, all in
`Blizzard_ChallengesUI/Mainline/Blizzard_ChallengesUI.lua`, identical in Retail,
PTR and Forever at `246`, `312`, `383`, and in Mists, Era and Anniversary at
`235`, `301`, `372`:

- `246`: `sortedMaps[1].level == 0 and not C_MythicPlus.GetOwnedKeystoneLevel()`
- `312`: `elseif C_MythicPlus.GetOwnedKeystoneLevel() or (dungeonScore and dungeonScore > 0) then`
- `383`: `self.ownedKeystoneLevel = C_MythicPlus.GetOwnedKeystoneLevel();` and
  `:395 elseif self.ownedKeystoneLevel then`

All three test the result for truthiness, so `nil` and `0` are not equivalent in
Lua: only `nil` and `false` are falsy. Publishing `0` where the client publishes
nothing makes the weekly chest take the `INCOMPLETE` branch at `:312` for a
player who owns no key. `tests/blizzard_challenges_ui_loads.rs` drives that
consumer and asserts the wall-of-text state.

`GetOwnedKeystoneChallengeMapID` has **no** consumer in the pinned Blizzard
addons; only its declaration exists.

`GetOwnedKeystoneMapID` has no pinned Blizzard-addon consumer. Its declaration
allows no return or one number (`MythicPlusInfoDocumentation.lua:96-103`). The
simulator maps the owned challenge-mode ID through
`sim_data::map_challenge_modes::MAPS`. The mapping from an owned key to the
MapChallengeMode row's `MapID` is **INFERRED (best-guess, pending R88bp)**; the
table rows are **VERIFIED** from MapChallengeMode build 12.1.5.70391. If the
context is absent or the row is missing, the simulator returns nothing
(**INFERRED (simulator mapping)** for the absent-data default).

## Declarations

`MythicPlusInfoDocumentation.lua`: `GetOwnedKeystoneChallengeMapID` and
`GetOwnedKeystoneLevel` with `MayReturnNothing = true` at Retail/PTR/Forever
`76-94` and Mists/Era/Anniversary `75-93`; `GetOwnedKeystoneMapID` with
`MayReturnNothing = true` and one numeric `mapID` at Retail `96-103` (same
pinned declaration across the documented profiles).

## Not proved

- [ ] Whether the client uses this exact MapChallengeMode lookup for
      `GetOwnedKeystoneMapID` remains unknown pending R88bp; current simulator
      mapping is an inferred best guess.
- [ ] The value actually carried by a found field (inferred, not proved).
- [ ] The publication bridge itself was only partially admitted in the
      reconstruction.
- [ ] PTR 12.1.7 and Retail 12.1.5.70575: no reconstruction, contract applied.

## Implementation inventory

- `src/c_api/c_mythic_plus.rs` — `get_owned_keystone_level`,
  `get_owned_keystone_challenge_map_id`, `get_owned_keystone_map_id`.
- `src/lua_api/state_types/mythic_plus_scenario.rs` — `OwnedKeystone`,
  `MythicPlusState::owned_keystone`.
- `src/lua_api/state.rs` — re-exports `OwnedKeystone` for tests and callers.

## Tests asserting this spec

`tests/c_mythic_plus_probes.rs`: existing level/challenge-map getter cases, plus
`GetOwnedKeystoneMapID` with no key (nothing), challenge ID 499 (map ID 2649),
and an unmapped challenge ID (nothing).

`tests/blizzard_challenges_ui_loads.rs`:
`challenges_weekly_chest_reads_the_owned_keystone_context` runs the original
`ChallengeModeWeeklyChestMixin:Update` consumer for the no-key and owned-key
cases.
