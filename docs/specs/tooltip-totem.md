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
`GameTooltip:SetTotem` accessor. `tests/tooltip_totem_getter.rs` runs that
consumer, rejects constant/spell-name producers with two distinct stored names,
and covers fractional nearest-even boundaries, the three unit transitions,
pre-format f32 loss, expiry, zero end, clock wrap, slots 8/9 and unsigned-domain
handling. Native protected-return arity, invalid-rate behavior and complete
exception decoration are not claimed.
