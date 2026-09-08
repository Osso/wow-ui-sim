# Talents panel

The Blizzard talents UI loaded from `Interface/BlizzardUI/Blizzard_PlayerSpells/ClassTalents/` (and `PvPTalents/`). This spec describes the contract the simulator must meet, not the implementation. For implementation details see the wiki.

## What it must do

### Load + lifecycle

- [ ] Talents panel loads without Lua errors when default Blizzard addons are enabled (`wow-sim --no-saved-vars lua-errors`)
- [ ] Talents panel loads without Lua errors with `--no-addons` (Blizzard UI only)
- [ ] Opening the panel via `/run TogglePlayerSpells()` (or equivalent) does not raise a Lua error
- [ ] Closing the panel does not leak frames or leave dangling pool entries
- [x] With the actual addon/SavedVariables configuration at the user-accepted 1906-unit canvas, native GUI Escape closes specialization without opening GameMenu.

### Taint / security model

- [ ] `AcquirePipFrame` returns a frame whose taint state is acceptable to the caller's `SecureMap`
- [ ] `Pools.AddObject` does not reject the frame on store
- [ ] Pool release path does not raise "attempted to release a secret value into a pool"
- [ ] Blizzard UI code in `Interface/BlizzardUI/` is NOT taint-stamped by the loader
- [ ] `SecureTypes.SecureMap.SetValue` enforces `issecure()` on keys and values per WoW semantics

### C_ClassTalents API surface

- [ ] `C_ClassTalents.GetActiveConfigID` returns a valid config ID for the active spec
- [ ] `C_ClassTalents.GetConfigIDsBySpecID` returns the configs for a given spec
- [ ] Talent flag queries used by the UI return without error (covered by `tests/class_talents_flags.rs`)
- [ ] Spec change via admin API updates the active config (covered by `tests/admin_spec_talent_api.rs`)

### Rendering

- [ ] All three talent trees (Class / Spec / Hero) render when the panel is opened on a class+spec that has them populated
- [ ] Talent node icons resolve via the texture atlas (no missing-texture placeholder for known nodes)
- [ ] Tier track templates render their pip frames at the correct anchor positions

## How it works

- → `docs/wiki/systems/` — system pages (talent panel page TBD)
- → `docs/layout-system.md` — anchor resolution used by tier track templates
- → `docs/lua-api.md` — `C_ClassTalents` namespace status

## Implementation inventory

- `src/lua_api/talent_state.rs` — Rust-side talent config state
- `src/lua_api/globals/missing_surface/traits.rs` — surface gap tracking incl. talent APIs
- `Interface/BlizzardUI/Blizzard_PlayerSpells/ClassTalents/` — Blizzard Lua (read-only, vendored)
- `Interface/BlizzardUI/Blizzard_PlayerSpells/PvPTalents/` — PvP variant (read-only, vendored)

## Tests asserting this spec

- `tests/class_talents_flags.rs`
- `tests/class_talents_config.rs`
- `tests/admin_spec_talent_api.rs`
- `tests/pool_api.rs` — covers the SecureMap/Pools contract this panel depends on

## Known gaps (current cycle)

- [ ] At the narrower 1266-unit canvas, PlayerSpells exceeds the 1186-unit center-panel capacity. Blizzard clears its center slot, so Escape opens GameMenu; this configuration remains unsupported.
- [ ] Hero talents subtree not yet rendered
- [ ] PvP talent variants untested in CI
- [ ] Taint model bug: `Pools.AddObject` rejects talent frames as tainted (see PLAN.md root cause analysis)

## Evidence

- `/tmp/pi-gui-accepted-panels.json` — current real addon/SavedVariables GUI proof at the accepted 1906-unit canvas: native Escape closes specialization without showing GameMenu; SpellBook has 43 visible rows and 41 valid spells.
- `/tmp/pi-gui-escape-fit-hook.json` — narrow-canvas fit failure; observes BlizzMove and EnhanceQoLMover fit callbacks without assigning sole cause.

## Out of scope

- Item-grants-talent tooltip integration (separate spec, item tooltips)
- Talent loadout sharing / import-export strings (deferred)
