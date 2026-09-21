# Combat State

### A_Admin.SetInCombat(inCombat)

Toggles the player's combat state.

- **inCombat** `boolean` -- `true` to enter combat, `false` to leave
- **Affects:** `InCombatLockdown()`, `UnitAffectingCombat("player")`
- **Fires:** `PLAYER_REGEN_DISABLED` when entering combat, `PLAYER_REGEN_ENABLED` when leaving
- **Example:**
```lua
A_Admin.SetInCombat(true)
print(InCombatLockdown())   -- true
A_Admin.SetInCombat(false)
print(InCombatLockdown())   -- false
```

### A_Admin.SetCasting(spellId, spellName, iconPath, duration)

Starts a simulated cast bar on the player.

- **spellId** `number` -- Spell ID for the cast
- **spellName** `string` -- Display name shown in the cast bar
- **iconPath** `string` -- Texture path for the cast bar icon
- **duration** `number` -- Cast time in seconds
- **Affects:** `UnitCastingInfo("player")`
- **Fires:** no ordinary cast-start event; this is the existing state-only initializer.
- **Note:** On retail 12.1+, replacing a channel emits its cancellation event. The existing timed-cast completion path can consume this state at its deadline. `StopCasting()` clears it without cast events; use `SpellStopCasting()` for modeled self-cancel notifications.
- **Example:**
```lua
A_Admin.SetCasting(19750, "Flash of Light", "Interface\\Icons\\Spell_Holy_FlashHeal", 1.5)
local name, text, texture, startTime, endTime = UnitCastingInfo("player")
-- name="Flash of Light"
```

### A_Admin.DelayCasting(seconds)

Extend the current timed cast by finite, nonnegative seconds. Returns `true` and emits `UNIT_SPELLCAST_DELAYED` after updating the deadline and cumulative `UnitCastingInfo` delay in milliseconds. Zero is accepted and still notifies. Returns `false` when no cast exists; invalid/overflowing input errors without mutation.

### A_Admin.FailCasting(quiet = false)

Fail the current timed cast. Returns `true`, clears the old cast before callbacks, and emits `UNIT_SPELLCAST_FAILED` (or `UNIT_SPELLCAST_FAILED_QUIET`), then `UNIT_SPELLCAST_STOP`, with the old identity. Returns `false` when inactive. A callback-created replacement survives. A failed specialization cast discards its pending specialization before callbacks.

These are explicit simulator inputs, not native failure detection. Failure order and quiet behavior are model policies; GCD/cooldown state is unchanged. See [delay/failure contract](../specs/cast-delay-failure-inputs.md).

### A_Admin.StopCasting()

Cancels the current simulated cast.

- **Affects:** `UnitCastingInfo("player")` returns `nil`
- **Fires:** none; this existing state-only API clears ordinary casting only. Use `StopChannel()` for channel/empower state.
- **Example:**
```lua
A_Admin.SetCasting(19750, "Flash of Light", "", 1.5)
A_Admin.StopCasting()
print(UnitCastingInfo("player"))    -- nil
```

### Channel and empower inputs (Retail 12.1+ and Forever)

- `A_Admin.StartChannel(spellID, name, icon, durationSeconds)` starts a channel.
- `A_Admin.StartEmpower(spellID, name, icon, stageSeconds, holdSeconds)` starts empower from a dense array of positive stage durations.
- `A_Admin.UpdateChannel(durationSeconds)` / `UpdateEmpower(stageSeconds, holdSeconds)` replace total timing from the original start and return whether the matching mode was active.
- `A_Admin.StopChannel(complete=false)` cancels either mode, or models successful early release with `true`; returns whether one existed.

All inputs use **seconds**. `UnitChannelInfo` endpoints and empower stage/hold queries use **milliseconds**: the query endpoint excludes empower hold, while the natural-completion deadline includes it. START/UPDATE payloads are `(unit, castGUID, spellID, castBarID)`. CHANNEL_STOP is `(unit, castGUID, spellID, interruptedBy, castBarID)`; EMPOWER_STOP is `(unit, castGUID, spellID, complete, interruptedBy, castBarID)`. `castBarID` is numeric and matches the matching `UnitCastingInfo`/`UnitChannelInfo` tuple, so unchanged consumers can correlate update and stop events. Natural completion and early release use nil `interruptedBy`; cancellation uses the current player GUID. This consumer-required convention conflicts with generated non-nil metadata and is simulator policy. `UpdateEmpower` changes modeled timing/counts, but the current Blizzard UPDATE handler does not rebuild pips or add hold to its display maximum; that unmodified-consumer limitation means no full native UI correctness is claimed. New starts replace the old active mode; callbacks can replace the incoming operation. Natural completion occurs on OnUpdate. These are simulator inputs—not native spell data, damage ticks, security, or timing semantics. See [channel/empower contract](../specs/channel-empower-lifecycles.md) and [duration-query contract](../specs/unit-cast-durations.md) for boundaries.

### Player cast duration queries (Retail 12.1+ and Forever)

`UnitCastingDuration(unit)`, `UnitChannelDuration(unit)`, and `UnitEmpoweredChannelDuration(unit[, includeHoldAtMaxTime])` return a `LuaDurationObject` only for matching modeled player state. They return no value while idle or for unmodeled units.

- Cast duration uses the stored cast start and end.
- Ordinary channel duration ends at the stored base channel end.
- Empowered duration includes hold-at-max by default; pass `false` to exclude it.

These are state-backed simulator queries. The pinned Forever `UnitDocumentation.lua` establishes their nullable public shape; separate hold handling in the pinned CastingBar consumer supports the ordinary-channel boundary. Native timing, secrecy, and other-unit behavior remain unproven. See [unit cast durations](../specs/unit-cast-durations.md).

### A_Admin.SetGCD(duration)

Triggers the global cooldown.

- **duration** `number` -- GCD duration in seconds (typically 1.5)
- **Affects:** `GetSpellCooldown(61304)` -- spell ID 61304 is the GCD sentinel
- **Example:**
```lua
A_Admin.SetGCD(1.5)
local start, dur, enabled = GetSpellCooldown(61304)
-- start=current_time, dur=1.5, enabled=1
```

### A_Admin.SetSpellCooldown(spellId, duration)

Sets a cooldown on a specific spell.

- **spellId** `number` -- Spell ID to put on cooldown
- **duration** `number` -- Cooldown duration in seconds
- **Affects:** `GetSpellCooldown(spellId)`
- **Example:**
```lua
A_Admin.SetSpellCooldown(31935, 15.0)   -- Avenger's Shield on 15s CD
local start, dur, enabled = GetSpellCooldown(31935)
-- dur=15.0
```
