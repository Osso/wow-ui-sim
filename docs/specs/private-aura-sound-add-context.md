# Private-aura sound Add context

Bounded simulator model for the remaining Add clause in `data/patch-api/sources/12.0.5-api-changes.txt:168` (prose-2026-03-31-168). The source limits restrictions to encounters/M+/PvP matches. Existing [sound removal](private-aura-sound-removal.md) is a separate accepted slice. Cached later-client `UnitAuraDocumentation.lua`, `UnitConstantsDocumentation.lua` and the complete actual deprecated 12.1 chunk establish the modern signature, sound fields and legacy delegation; not native 12.0.5 parity. See [Lua API architecture](../wiki/systems/lua-api.md).

All registration, allocator, input-validation, error, secure-caller and context-mapping policies below are **INFERRED**. No native probe or compiled proof has been run by this authoring slice.

## What it must do

### INFERRED inputs and observable state

- [ ] Start empty, allocate monotonic nonzero u32 IDs from host `next_id`, avoiding host-declared live IDs. Host sets `next_id` to a nonzero candidate or None; None is exhaustion, returning exactly one nil.
- [ ] Copy trigger, unitToken, spellID, soundFileName, soundFileID and outputChannel into per-environment Rust registrations. Later Lua mutation must not change prior registrations.
- [ ] Accept actual nonempty string fields and finite integral inclusive-u32 numeric fields; no coercion, alias lookup or fallback. Require exactly one file name or file ID; outputChannel optional. Invalid input leaves registrations, live IDs and allocator unchanged.
- [ ] Use an allocated public numeric handle with the existing remover; removal deletes the payload and live ID immediately, preserving other registrations even in restricted contexts.
- [ ] Isolate environments and observe host context/allocator changes immediately.

### INFERRED caller-context restriction

- [ ] Public arguments under actual addon taint register outside encounter/M+/PvP, including ordinary combat. Preserve caller taint and outer secure context.
- [ ] Use existing `world.encounter_in_progress`, existing `mythic_plus.is_active`, and explicit host `pvp_match_active`. Do not substitute combat, PvP queue, arena instance type or aura secrecy flags.
- [ ] Any active context denies an insecure caller with a public contextual error; consume no ID. Context exit permits the next Add immediately.
- [ ] Secure callers remain permitted during active contexts. This interpretation of HasRestrictions is INFERRED, not native permission proof.

### VM AllowedWhenUntainted and publication

- [ ] Call actual `unwrap_secret` on every declared top-level argument before validating any argument; authenticate every declared sound field before validating any field. Secure callers accept correctly typed real secret payloads; addon callers reject inaccessible secrets without declassification.
- [ ] Rooted actual secret table/number survive GC and denial; malformed earlier public values must not mask denial of a later secret argument/field. Preserve allocator/state and permit subsequent public recovery.
- [ ] Publish legacy Add from retail-12-0-5; modern Add from retail-12-1-0 solely to preserve complete cached deprecated-file delegation. Modern triggers 0/1/2 share the same registration model and context guard. Removal remains unrestricted with its accepted conservative secret-ID rejection unchanged.

## How it works

- [Lua API architecture](../wiki/systems/lua-api.md)
- [Addon loading pipeline](../addon-loading-pipeline.md)

## Implementation inventory

- `src/c_api/private_aura_sounds/inputs.rs` — empty host state plus owned registration payload.
- `src/c_api/private_aura_sounds/add.rs` — authentication, validation, context restriction and registration producer.
- `src/c_api/private_aura_sounds.rs` — publication and shared removal transition.

## Tests asserting this spec

`tests/private_aura_sound_add_context.rs`: six common fixtures plus two modern1210 fixtures, automatically included in the existing integration binary. Parent applies state/tests first with producers withheld, captures compiled RED, then applies producers. No proof claimed here.

## Known gaps (current cycle)

- [ ] Parent-owned compiled RED/GREEN, current controls, warning/format/readability and acceptance evidence.
- [ ] Audit accounting remains partial until parent accepts this bounded model and its proof.

## Out of scope

Audio/playback, aura-trigger dispatch, resource existence, unit/spell catalog membership, native IDs/error wording, native secret permissions, native context/secure-caller parity and whole-page acceptance. Later-client documentation does not prove legacy 12.0.5 native behavior. This model registers owned host records and enforces a stated INFERRED permission policy, not a synthetic audio engine.
