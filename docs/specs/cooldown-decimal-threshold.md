# Cooldown decimal threshold

`Cooldown:SetCountdownMillisecondsThreshold(seconds)` controls the countdown text emitted by the simulator renderer. The [12.0.5 notes](../../data/patch-api/sources/12.0.5-api-changes.txt) specify one decimal place when remaining time is below the supplied seconds threshold.

## Required behavior

- [ ] Lua threshold updates affect rendered countdown text, not only the getter.
- [ ] Below the threshold, emit one decimal place, including thresholds above ten seconds.
- [ ] At or above the threshold, retain whole-second ceiling formatting.
- [ ] Zero disables fractional display; changing or resetting the threshold takes effect on the next rendering calculation.
- [ ] Existing aura-display and abbreviation modes retain precedence.

## Evidence and inference

The setter already stores seconds. Rendering previously ignored that state and hardcoded ten seconds. Tests configure a real Cooldown through Lua, then assert the same text formatter used by rendering. They do not establish GPU appearance or native-client equivalence.

Strict comparison follows “below” in the notes. Zero-disable, ceiling outside the threshold, and aura/abbreviation precedence are inferred simulator policies, not native-verified behavior. No ten-second fallback remains.

## Future native probe

Record client build, initial getter, and visible text at remaining 19.9, 20.0, and 20.1 with threshold 20; repeat 2.4 seconds with thresholds 0, 5, 2, and 0. Repeat with aura-display mode and abbreviation threshold 5 to establish precedence and default behavior. Capture rounding near a decimal rollover.

## Behavioral tests

`src/iced_app/quad_builders_cooldown.rs`:

- `cooldown_decimal_threshold_changes_rendered_text_at_exact_boundary`
- `cooldown_decimal_threshold_zero_and_updates_control_fractional_text`
- Existing `cooldown_countdown_text_uses_aura_and_abbrev_modes` control uses an explicit decimal threshold.

## Related contracts

- [Cooldown numeric methods](cooldown-numeric-methods.md)
- [Rendering pipeline](../rendering-pipeline.md)
