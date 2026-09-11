# Cast-bar identity returns

`UnitCastingInfo` and `UnitChannelInfo` expose existing simulator cast identity in their query tuples. The source contract is the committed [12.1.5 register](../../data/patch-api/sources/12.1.5-register.json), comparing Gethe `a89e9d0ceb7f6cd31e8fc5ca7df1a338ac0b1b58` with `49b69918fcdc77e109813281e4f537d45ec7dcbf`.

## What it must do

- [x] Both pinned profiles return eleven values for active casting and channeling, in the order below.
- [x] `castBarID` uses the existing allocated `CastingState.cast_id`; repeated queries and timing/empower-state updates retain identity.
- [x] New admin-created casts receive new allocated identities; casting and channeling queries read their respective states.
- [x] Inactive and unsupported-unit queries retain existing nil-observable behavior.
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
- `src/lua_api/game_data.rs`: `CastingState` identity, timing, and accumulated delay.
- `src/lua_api/globals/admin.rs`: cast allocator/start/stop operations.
- `src/lua_api/globals/admin/cast_inputs.rs`: explicit delay and failure inputs.
- `src/lua_api/spellcast_events.rs`: shared synthetic GUID formatting.

## Tests asserting this spec

- `tests/cast_bar_id.rs`: complete tuple assertions driven through `A_Admin.SetCasting`, `StartChannel`, `StartEmpower`, and timing-update inputs. Ordinary casts replace channels rather than retaining simultaneous active modes. See [channel/empower lifecycle](channel-empower-lifecycles.md) for separate producer and consumer proof.
- `tests/spell_casting.rs`: existing casting regressions.
- `tests/c_vehicle_possession_globals.rs`: existing casting/channel query regressions.

Focused proof at `3d4d483d3`: two new tests passed on each of `client-ptr` and `client-retail`; eighteen `spell_casting::` and seventeen `c_vehicle_possession_globals::unit_` regressions passed per profile. PTR RED observed casting arity 9 and channel arity 10 before the serialization fix. These are targeted development tests, not final conformance verification.

## Known gaps (current cycle)

- [ ] Numeric simulator `cast_id` mapping to native `UnitCastBarID` is unverified. Casting slot seven now uses the same synthetic string GUID as event payloads; its native encoding remains unverified.
- [x] `A_Admin.DelayCasting` accumulates delay seconds on cast state and returns milliseconds in slot eleven, preserving start and identity. See [delay/failure inputs](cast-delay-failure-inputs.md) for focused lifecycle and real cast-bar consumer proof.
- [ ] Existing texture paths remain strings rather than native `fileID` values; existing trade/interruptibility defaults are unchanged.
- [ ] Native identity lifetime, security/secret annotations and type semantics remain unverified.

## Out of scope

- This spec covers tuple serialization; event dispatch and channel scheduling are covered separately by [channel/empower lifecycle](channel-empower-lifecycles.md) and [spellcast payloads](spellcast-event-payloads.md).
- Audit-manifest credit, artifact generation and broad/final verification gates.
