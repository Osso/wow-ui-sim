# Encounter warning preview

`C_EncounterWarnings.GetEditModeWarningInfo(severity)` constructs synthetic Edit Mode preview records in the PTR API epoch. Source contracts are the pinned Gethe revision `49b69918fcdc77e109813281e4f537d45ec7dcbf` (`EncounterWarningsDocumentation.lua` and `DungeonEncounterConstantsDocumentation.lua`) and the [two changed warning-info occurrences](../../data/patch-api/sources/12.1.5-register.json). No gameplay-warning store or trigger is introduced.

## What it must do

- [x] Return exactly one fresh table containing all fourteen documented fields for each severity: Low `0`, Medium `1`, High `2`.
- [x] Return independent mutable ColorMixin-compatible colors with numeric RGBA channels; modifying a previous record/color must not affect later previews.
- [x] Supply finite numeric seconds for `duration`; preserve the requested severity and distinguish high-severity deadly presentation.
- [x] Reject missing, nonnumeric, nonintegral, and out-of-range severities without persistent state changes.
- [ ] Feed the actual Blizzard warning system's editing path, text/icons/color, and existing `C_Timer.NewTimer` expiration, cancellation, replacement, and reuse lifecycle without modifying vendor code.
- [x] Preserve the actual earlier-retail record, legacy severity mapping (including severity `3`), and 30-second duration before/after bootstrap. This is baseline preservation, not pinned-base conformance.

## Simulator preview policy

All content is synthetic. Text is `Simulated Low/Medium/High Warning`; caster/target names are `Simulator Caster` and `Simulator Target`, with opaque `Sim-Warning-Caster` / `Sim-Warning-Target` identifiers. These identifiers are not native WoW GUIDs. The chosen icon is file ID `136122`, tooltip spell is `0`, and duration is five seconds. Colors are white, amber `(1, 0.75, 0.1)`, and red `(1, 0.15, 0.05)`, all alpha one. Only High is deadly. Warnings are shown, but sound and chat output are disabled.

The source establishes fields/types, ColorMixin shape, and severity values—not these preview values, validation errors, native secret handling, or exact expiration behavior. Ordinary strings are used even for source fields annotated secret. Existing legacy `PlaySound` behavior is untouched. No new preview setting API is needed. The constructor uses the existing `CreateColor` factory, so the loaded Blizzard ColorMixin is retained. The initialization-only color fallback has getters and mutable RGBA fields but lacks `SetRGBA`; that unrelated fallback is not expanded here.

## How it works

- [Widget API](../widget-system.md)
- [Event/timer system](../event-system.md)
- [PTR API audit](../wiki/investigations/patch-12-1-5-api-audit.md)

## Implementation inventory

- `src/c_api/c_encounter_warnings.rs`: PTR synthetic record constructor and strict severity validation.
- `src/c_api/mod.rs`: profile-scoped module declaration.
- `src/lua_api/globals/missing_surface/encounter_warnings.rs`: registration routes PTR previews to the model and preserves the earlier-retail implementation.

## Tests asserting this spec

- `tests/encounter_warning_preview.rs`: complete records, independence, validation, retail baseline, and actual Blizzard preview/timer lifecycle. Timer deadlines are advanced through existing simulator timer state; callbacks are not replaced or injected.
- Focused development proof at `ae642f1e4`: `cargo test --test integration --offline --no-default-features --features sound,gui,client-<profile> encounter_warning_preview:: -- --nocapture`. PTR: three passed, one failed at the real `AnimationGroup:Play()` visibility boundary. Retail: one passed. The later expiration/cancel/reuse assertions remain unexecuted, not passing evidence. No final gates were run.

## Known gaps (current cycle)

- [ ] Complete the real Blizzard preview display/expiration/cancel/reuse test. Current `AnimationGroup:Play()` changes playback flags without dispatching `OnPlay`; Blizzard's installed handler is what shows the warning view. The regression remains failing rather than calling `Show()` or replacing vendor callbacks. Animation-engine correction requires separate scope approval.
- [ ] Native secret/taint behavior, preview content, GUID semantics, exact duration policy, and validation/error compatibility remain unverified.

## Out of scope

Gameplay warning storage/dispatch, encounter/timeline coupling, new triggers, sound/chat models, native security enforcement, artifact credit, and final verification gates belong outside this bounded preview slice.
