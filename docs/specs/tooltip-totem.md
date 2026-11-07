# Totem tooltip

## Publication and selection

Retail/PTR/Forever pinned `TooltipInfoDocumentation.lua:1084` declares
`C_TooltipInfo.GetTotem(slot)`. Publish on these three profiles only. WM94
sections 3/9 establish the Retail 70688 producer; applying that producer to PTR
and Forever is **INFERRED** shared-model reuse, not separate native proof.

- Authenticate the required unsigned slot. Public slots 1–8 select the current
  supplied `totem_slots` entry. An absent entry, zero or out-of-range index uses
  [the shared UNKNOWN failure seam](tooltip-appearance-item.md): zero return
  values are **EMULATED** simulator policy, not measured native failure arity.
- An occupied entry produces tooltip type 18 with native identity ID zero. The
  shared [D56 builder](tooltip-data-serialization.md) omits that zero ID on
  Retail/PTR and stamps a fresh instance. Other profiles keep their existing
  publication rules.
- Add the stored name with default color and one highlight-colored remaining-time
  line. Do not substitute a spell name or add a spell-description block. Occupied
  expired and zero-ended timers retain the remaining-time line with count zero.

### Slot reader (Round 6a)

1. **VERIFIED:** the argument must be a number inside the closed range 0 to
   4294967295 and passes the secret-access check; it is then converted as
   `truncate(value − 1)` with **no integral test**, and the low 32 bits are read
   as unsigned. `1.9`, `2.9` and `8.9` therefore select slots 1, 2 and 8, and Lua
   `0` becomes all-ones.
2. **VERIFIED:** the producer keeps a fixed table of eight slot records and
   rejects a negative converted slot and any slot of 8 or more, so the valid Lua
   range is 1–8 and an above-range slot takes the same miss arm as an empty slot.
3. **INFERRED:** the same truncation rule applied below 1 makes `0.5` select
   entry 0. The native conversion of a negative fractional value is not decoded;
   this follows from applying the verified conversion uniformly.
4. **VERIFIED (native wording):** a missing, nil, non-number, negative,
   out-of-domain or non-finite argument raises, with the usage literal
   `Usage: local data = C_TooltipInfo.GetTotem(slot)` for a missing/nil/wrong-type
   value and `outside of expected range 0 to 4294967295 - <usage text>` for a
   range or non-finite failure. **EMULATED:** the simulator raises its shared
   reader message instead of reproducing those native strings.

### Not established

- **UNKNOWN:** the obfuscated publication tail, so "an empty slot returns
  nothing" is INFERRED from the unset result count and emulated as zero return
  values.
- **UNKNOWN:** whether a plain miss queues a pending error message. Native raises
  only when one is queued, so the simulator's unconditional miss arm is a
  documented emulation.
- **INFERRED:** Forever 70707 registers the member and its decoded body prefix
  has the same shape, but its line composition is not decoded, so the two lines
  on Forever are shared-model reuse rather than separate native proof.

## Remaining time (WM94 §9)

1. Convert supplied seconds to unsigned stored milliseconds. Add start and
   duration modulo 2^32. Read the cached game-clock millisecond base, not a
   separate wall-clock sample.
2. A zero end, equality, or nonnegative signed wrapping `now - end` yields zero.
   Otherwise subtract now from end as wrapping u32, convert that difference to
   f32 and divide by the f32 rate. Elapsed time is not rate-scaled.
3. Convert the division result to integer milliseconds using one named
   `emulated_nearest_even_milliseconds` seam. Nearest-even control is **INFERRED**;
   the live thread's FP control is **UNKNOWN**. The same explicit **EMULATED**
   seam converts host seconds into millisecond storage. Unsupported host time
   domains and nonpositive/nonfinite rates raise an explicit simulator error;
   they are not silently clamped or treated as proved native expiry behavior.
4. Round integer milliseconds upward to seconds. Select days at 129601 seconds,
   hours at 5401, minutes at 91, otherwise seconds. Round the selected unit's
   count upward. Float conversion precedes these thresholds.
5. Format the integer count with the existing localized
   `SPELL_TIME_REMAINING_DAYS/HOURS/MIN/SEC` global. Do not call `SecondsToTime` or
   synthesize a clock string. Runtime localization-variant selection remains
   **UNKNOWN**; the simulator uses its supplied locale's existing key.

## Evidence and proof

Clean-room native source: WM94 `native-evidence-70688/WM94-tooltipinfo-getters.md`,
sections 3/9. The native fragment emulation is separate from live-client proof.

`Blizzard_SharedXMLGame/Tooltip/TooltipDataHandler.lua:579` supplies the original
`GameTooltip:SetTotem` accessor (`SetTotem = "GetTotem"`), reached from
`Blizzard_UnitFrame/Mainline/TotemFrame.lua:73,79`. `tests/tooltip_totem_getter.rs`
runs that consumer and covers the slot reader (`1.9`, `2.9`, `8.9`, `0.5`, `9`,
`9.9` and the six raising argument forms), rejects constant/spell-name producers
with two distinct stored names, and covers fractional nearest-even boundaries,
the three unit transitions, pre-format f32 loss, expiry, zero end, clock wrap,
slots 8/9 and unsigned-domain handling. Native protected-return arity,
invalid-rate behavior and complete exception decoration are not claimed.
