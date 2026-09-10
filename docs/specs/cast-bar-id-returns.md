# Cast-bar identity returns

`UnitCastingInfo` and `UnitChannelInfo` expose existing simulator cast identity in their query tuples. The source contract is the committed [12.1.5 register](../../data/patch-api/sources/12.1.5-register.json), comparing Gethe `a89e9d0ceb7f6cd31e8fc5ca7df1a338ac0b1b58` with `49b69918fcdc77e109813281e4f537d45ec7dcbf`.

## What it must do

- [ ] Both pinned profiles return eleven values for active casting and channeling, in the order below.
- [ ] `castBarID` uses the existing allocated `CastingState.cast_id`; repeated queries and timing/empower-state updates retain identity.
- [ ] New admin-created casts receive new allocated identities; casting and channeling queries read their respective states.
- [ ] Inactive and unsupported-unit queries retain existing nil-observable behavior (zero returned values).
- [ ] Profiles before `retail-12-1-0` retain their existing nine casting and ten channeling returns.

### Complete pinned tuples

Both source function `Returns` arrays and matching result-structure `Fields` arrays have the same order and types:

| Position | UnitCastingInfo / UnitCastingInfoResult | UnitChannelInfo / UnitChannelInfoResult |
|---|---|---|
| 1 | name: cstring | name: cstring |
| 2 | displayName: string | displayName: cstring |
| 3 | textureID: fileID | textureID: fileID |
| 4 | startTimeMs: number | startTimeMs: number |
| 5 | endTimeMs: number | endTimeMs: number |
| 6 | isTradeskill: bool | isTradeskill: bool |
| 7 | castID: WOWGUID | notInterruptible: bool? |
| 8 | notInterruptible: bool? | spellID: number |
| 9 | castingSpellID: number | isEmpowered: bool |
| 10 | castBarID: number? → UnitCastBarID? | numEmpowerStages: number |
| 11 | delayTimeMs: number | castBarID: number? → UnitCastBarID? |

The arrows are the only tuple field type differences between the pinned base and target. Optionality is unchanged. `castBarID` is `NeverSecret` in both; casting `isTradeskill`/`delayTimeMs` and channel `isTradeskill`/`isEmpowered`/`numEmpowerStages` are also `NeverSecret`. These annotations do not establish simulator enforcement.

## How it works

- [Lua API architecture](../lua-api.md)
- [Event system](../event-system.md) — unchanged by this slice.

## Implementation inventory

- `src/lua_api/globals/utility_system_spell/spell_api.rs`: state-backed tuple serialization and profile-dependent arity.
- `src/lua_api/game_data.rs`: existing `CastingState` identity and timing fields; unchanged.
- `src/lua_api/globals/admin.rs`: existing cast allocator/start/stop operations; unchanged.

## Tests asserting this spec

- `tests/cast_bar_id.rs`: grouped integration tests driven through `A_Admin.SetCasting`/`StopCasting`, with allocated state transferred to the channel slot because no channel-start API exists. Timing and empowerment updates use existing state fields; no channel-event lifecycle is claimed.
- `tests/spell_casting.rs`: existing casting regressions.
- `tests/c_vehicle_possession_globals.rs`: existing casting/channel query regressions.

## Known gaps (current cycle)

- [ ] Numeric simulator `cast_id` mapping to native `UnitCastBarID` is unverified. The native `WOWGUID` at casting slot seven remains modeled as the existing numeric ID.
- [ ] Casting `delayTimeMs` is an explicit zero placeholder: there is no accumulated-delay field. End-time changes do not update it; this slice does not introduce delay tracking or change timing.
- [ ] Existing texture paths remain strings rather than native `fileID` values; existing trade/interruptibility defaults are unchanged.
- [ ] Native identity lifetime, security/secret annotations and type semantics remain unverified.

## Out of scope

- Changes to the thirteen cast-event payloads, dispatch ordering, timing, cast/channel scheduling or identity allocation.
- Audit-manifest credit, artifact generation and broad/final verification gates.
