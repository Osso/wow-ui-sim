# Combat State

### A_Admin.SetSpellBookOverrideObservation(spellID, overrideSpellID)

Retail and Forever: supply a known catalog result for `C_SpellBook.FindSpellOverrideByID`. Both IDs must be positive signed-32-bit integers; the result must exist in the selected spell catalog. The empty model keeps the prior no-link record check. This observation is independent of `C_Spell.GetOverrideSpell`; no event is emitted. VERIFIED declarations: Retail `SpellBookDocumentation.lua:85–98`, Forever `:95–108`; both `Blizzard_TutorialManager.toc:16` select `Blizzard_TutorialHelper.lua:190–211`, which resolves action buttons by base and override spell IDs.

### A_Admin.SetCallPetSpellInfo(spellID, slot, petName)

Supply one call-pet spell and the current player pet name for its slot (0–4, from Retail R88m.4). The spell ID must be a positive signed-32-bit integer; the pet name must be string or nil (an empty string is valid). The selected `Blizzard_ActionBar/Shared/SpellFlyout.lua:270` (Retail), `:329` (Forever) reads two values; the Forever shape is inferred from this consumer, not a native probe. No event is emitted. Retail's absent observation returns zero values; Forever preserves the prior two nils until native return-count evidence is available.

### A_Admin.SetSpellOverrideObservation(spellID, specID, onlyKnown, ignoreOverrideSpellID, overrideSpellID)

Retail and WoW Forever: provide the exact result for a `C_Spell.GetOverrideSpell` query. `specID=0` resolves the active player's specialization when read or supplied. The input identifier must resolve to a positive spell ID; the result must be a positive integer. Spec and ignored spell ID must be nonnegative integers (nil uses zero), and onlyKnown must be boolean or nil (nil uses true). The empty model returns the resolved input spell ID when no observation matches, as documented. No event is emitted; this is a supplied observation, not native eligible-override discovery.

VERIFIED shapes: Retail `SpellDocumentation.lua:165–181`; Forever `:206–222`. Both `Blizzard_SharedTalentUI.toc:5` select `Blizzard_SharedTalentUtil.lua:110–120`; its replacement-name utility reads the result via `GetSpellName`.

### A_Admin.SetSpellUiFlags(spellID, pressHoldRelease, priorityAura)

Retail and WoW Forever: supply the two observed boolean flags for a positive integer spell ID (maximum `i32::MAX`). Both arguments must be booleans; invalid input raises an error before mutation. The empty model reports false. A call replaces that spell's row and emits no event. Supply rows before the original UI caches them, or send the normal action-slot update event after changing action assignments.

VERIFIED declaration shapes: Retail `SpellDocumentation.lua:704-734` and Forever `SpellDocumentation.lua:773-803`. The selected `Blizzard_ActionBar/Shared/ActionButton.lua` stores the press-and-hold flag in its action-button attribute; the selected `Blizzard_FrameXMLUtil/AuraUtil.lua:449-455` caches aura-priority observations by spell ID. This is host-supplied classification, not native discovery of spell attribute masks.

### Pet-battle runtime inputs (Retail and WoW Forever)

These inputs supply simulator state; they do not model a server reply. The predicate rules below are **INFERRED R88g9 (native evidence pending)** where the cited R88f checks stop short of the complete handler.

- `A_Admin.SetPetBattleAvailability(playerTrapState, actionAvailable, trapError)` supplies the local trap-state byte, action-availability check, and failure code used by `C_PetBattles.IsTrapAvailable()`.
- `A_Admin.SetPetBattleTeamState(localTeamPresent, skipBlocking, shouldSelect, opponentIsNPC)` supplies the local team flags and opposite-team NPC marker used by `IsTrapAvailable()`, `ShouldShowPetSelect()` and `IsPlayerNPC()`.
- `A_Admin.SetPetBattleEffectName(name)` appends one supplied label returned by `C_PetBattles.GetAllEffectNames()`.
- `A_Admin.SetPetBattleMatchmakingInfo(status, estimatedTime, queuedTimeMilliseconds, canAccept)` replaces the matchmaking snapshot consumed by `C_PetBattles.GetPVPMatchmakingInfo()` and `CanAcceptQueuedPVPMatch()`. `status` is nil, `"queued"`, `"suspended"`, `"proposal"`, or `"entry"`; the times must be nonnegative integers; `canAccept` must be boolean. The setter emits no event. To refresh the original `QueueStatusFrame`, call `A_Admin.FireEvent("PET_BATTLE_QUEUE_STATUS")` after supplying the state.

The Retail `SharedPetBattleTemplates.lua:381` consumer collects each `GetAllEffectNames()` return into an array, so **INFERRED R88g9 (native evidence pending)** the simulator returns supplied labels as separate string results; native ordering and contents remain unknown. `Blizzard_PetBattleUI.lua:667` consumes `IsTrapAvailable()`, `:155` and `:195` consume `ShouldShowPetSelect()`, and `:300` consumes `IsPlayerNPC()`; these call sites establish consumer use, not the exact native state mapping.

The current predicate projection uses battle phases 1 or 3 for pet selection, and phase 2 for trap availability. Trap availability also requires trap state 1, an NPC opponent, available action state, and no skip-blocking flag for a present local team. `GetPetInfoByPetID` is modeled by the existing `C_PetJournal.GetPetInfoByPetID` handler against supplied journal data; **VERIFIED R88f.D.43**, the checked Retail registration census has no `C_PetBattles.GetPetInfoByPetID` entry and publishes that member under `C_PetJournal`.

### A_Admin.LearnSpell(spellID) / A_Admin.UnlearnSpell(spellID)

Adds or removes a supplied player-known spell ID. The known-spell queries include this supplied membership as well as the simulator's fixed spellbook data. These inputs do not add spellbook slots or fire events.

### A_Admin.SetSpellTargeting(spellID, classification) (Retail and WoWForever)

Supplies whether a spell is classified as helpful or harmful. Pass `"helpful"`, `"harmful"`, or `nil`; each call replaces that spell's prior classification, and `nil` clears both sets. `IsHelpfulSpell`, `IsHarmfulSpell`, `C_Item.IsHelpfulItem`, and `C_Item.IsHarmfulItem` read this scenario input. It does not create spell or item data.

### A_Admin.SetSpellBookItemLowRank(spellID, lowRank)

Sets or clears a scenario low-rank flag for a spell ID. The scenario starts with no flagged spells; this is not native rank data. On PTR and WoWForever, `C_SpellBook.IsSpellBookItemLowRank` reads this input.

### A_Admin.SetActiveRaidMarkers(indexes)

Replaces the supplied active world-marker IDs. `indexes` must be a dense array of whole numbers from `1` through `8`; invalid entries reject the call before state changes. `IsRaidMarkerActive(index)` reads this state. This setup input does not fire events. The simulator models active marker IDs, not their 3D positions.

### A_Admin.SetActionRepeatActive(slot, active)

Supplies positive action-slot repeat membership independently of casting/current
state, attack classification and assigned spell. `active` must be a literal boolean.
Returns no values; emits no events. Supply state before explicitly firing
`START_AUTOREPEAT_SPELL`, `STOP_AUTOREPEAT_SPELL` or `ACTIONBAR_UPDATE_STATE` through
`A_Admin.FireEvent`. Membership lasts until explicitly cleared; this input does not
infer action replacement, spell auto-fire or multiple-repeat event arbitration.
See the [G2 contract](../specs/action-repeat-active.md). Staged; runtime unverified.

### A_Admin.SetInCombat(inCombat)

Toggles the player's combat state.

- **inCombat** `boolean` -- `true` to enter combat, `false` to leave
- **Affects:** `InCombatLockdown()`, `UnitAffectingCombat("player")`
- **Fires:** `PLAYER_REGEN_DISABLED` for true and `PLAYER_REGEN_ENABLED` for false, after the state change. This is simulator ordering; native combat-state timing is unverified.
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

### A_Admin.SetItemCooldown(itemID, duration)

Sets the supplied cooldown for an item ID. The start time is the current simulator `GetTime()` value. A duration of `0` clears that item's cooldown input.

- **itemID** `number` -- Item ID to put on cooldown
- **duration** `number` -- Cooldown duration in seconds; `0` clears the input
- **Affects:** `C_Item.GetItemCooldown(itemID)` and the original quest item button when that item is the quest special item

```lua
A_Admin.SetItemCooldown(19001, 31)
local start, duration, enabled = C_Item.GetItemCooldown(19001)
-- duration=31, enabled=true
A_Admin.SetItemCooldown(19001, 0) -- clear
```

### A_Admin.SetCombatLogEntries(entries)

Replaces the host-supplied combat-log history of the Retail and WoW Forever
model (`src/c_api/combat_log_history.rs`). The simulator invents no combat
events; this is the only input of that history.

- **entries** `table` -- Array of entries, oldest first. Each entry is an array
  of the values that `GetCurrentEventInfo` returns. Values must be booleans,
  finite numbers or strings; other values raise an error.
- The current entry resets to the newest entry.
- **Affects:** `C_CombatLog.GetEntryCount`, `C_CombatLogSecure.GetCurrentEventInfo`,
  `SeekToNewestEntry`/`SeekToPreviousEntry`, `ShouldShowCurrentEntry` and,
  on WoW Forever, `C_CombatLogInternal.GetCurrentEventInfo`.

```lua
A_Admin.SetCombatLogEntries({
    { "SPELL_DAMAGE", 19750, 150 },
    { "SPELL_HEAL", 19750, 275 },
})
local event, spell, amount = C_CombatLogSecure.GetCurrentEventInfo()
-- "SPELL_HEAL", 19750, 275
```
