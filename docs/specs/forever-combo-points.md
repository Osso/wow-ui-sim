# Forever combo points

`GetComboPoints(unit, target)` exposes a target-bound view of the existing player combo-point pool. Implementation lives under `src/lua_api/globals/real/`; [the investigation](../wiki/investigations/forever-combo-points.md) records why this must not simply alias `UnitPower`.

## What it must do

- [x] Publish the legacy global on Forever only, requiring both unit-token arguments and returning one number for modeled queries. Leave other profiles and their existing compatibility behavior unchanged.
- [x] Read the existing `PlayerState.secondary_powers[4]` count. `A_Admin.SetPlayerPower(count, max, 4)` also assigns that snapshot to the selected target GUID; unrelated power inputs do not alter the assignment.
- [x] Return the pool count only for its assigned target. During `PLAYER_TARGET_CHANGED`, a different target must already report zero even when `UnitPower("player", 4)` still reports the earlier snapshot.
- [x] Resolve owner/target aliases through existing unit snapshots and GUID identity. An absent owner or target has no combo points and returns zero.
- [x] Preserve read-only queries, clear target assignment on a zero input, and leave a nonzero input made without a target unassigned. A new nonzero input replaces the single assignment.
- [x] Report unsupported resolved nonplayer ownership explicitly instead of fabricating its count. Exercise unchanged native ComboFrame initialization/update with `comboPointLocation=1`, including nonzero display and zero hiding.

### Evidence and explicit guesses

Pinned Forever `UnitDocumentation.lua` declares the two unit-token parameters and a non-nil numeric result. Native `Mainline/ComboFrame.lua:65` calls `GetComboPoints(PlayerFrame.unit, "target")` when its CVar-enabled initialization has set `maxComboPoints`.

Cached EllesmereUI 9.2.2's changelog reports `UnitPower=3` while `GetComboPoints=0` during a target swap on Forever. This is addon-author evidence, not a simulator-run native probe. User-run Forever probes are unavailable.

**Simulator guesses:** binding the existing power input to one target GUID, retaining that assignment across selection changes until another power input, unassigned/no-target handling, player GUID aliases, and the unsupported-owner error policy. No multi-target point accumulation or combat-generated gains are inferred. Nonplayer/vehicle-owned combo pools are not modeled. These policies are not native conformance claims.

## How it works

- [Target ownership investigation](../wiki/investigations/forever-combo-points.md)
- [Existing admin health/power inputs](../admin-api/health-power.md)
- [Existing power-query boundaries](unit-power-missing.md)

## Implementation inventory

- `src/lua_api/globals/real/combo_points.rs`: modeled query, registration, and target assignment from power inputs.
- `src/lua_api/state_types/character_world.rs`: Forever-only optional target GUID; count/max remain in the existing secondary pool.
- `src/lua_api/globals/admin.rs`: bind accepted combo-point inputs to their target.
- `src/lua_api/globals/real/mod.rs`, `src/lua_api/globals/register.rs`: Forever-only wiring.

## Tests asserting this spec

`tests/combo_points.rs` joins the existing grouped integration target. It covers changing/zero inputs, primary-power independence, callback ordering across a target swap before a power update, GUID aliases, no-target input, explicit unsupported ownership, and unchanged native ComboFrame with its CVar enabled.

Runtime RED exists in `/tmp/forever-addon-audit/batch-combo-error-transition.json` and the parent's no-addons shared-data control. At `c1e830ffa`, the isolated Forever integration build and `combo_points::` filter pass 6/6, including unchanged native ComboFrame updates with its CVar enabled. This proves the bounded simulator model, not native client behavior.

## Known gaps (current cycle)

- [ ] Verify affected addon paths under per-process CVar isolation; the shared-CVar baseline is invalidated and cannot provide this proof.
- [ ] Native PvP/secret-return semantics, nonplayer ownership, and combat generation/consumption remain unmodeled; affected workflows must remain explicit gaps.

## Out of scope

Addon/vendor/cache edits, constant-zero shims, changes to UnitPower snapshots or other profiles, arbitrary combo caps, multi-target accumulation, new targeting/event behavior, and inventory-wide compatibility claims.
