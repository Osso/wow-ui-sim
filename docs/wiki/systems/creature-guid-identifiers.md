# Creature GUID identifiers

`C_CreatureInfo.GetCreatureID` and legacy `UnitCreatureID` share one parser for simulator Creature GUID strings.

## Behavior

`src/c_api/c_creature_info.rs` splits a GUID on `-`, requires the `Creature` prefix, then parses the existing sixth segment policy as an integer. `GetCreatureID` exposes that result only on retail 12.0.0+; `UnitCreatureID` reuses it for unit-derived GUIDs.

Focused tests cover two Creature IDs and exact-one nil results for Player and empty inputs. Retail `creature` verification passes 44/44 on each 12.0.0/12.0.5/12.0.7. Mists broad `creature` is 33/41 because eight unrelated AccountStore tests fail setup at `tests/common/mod.rs:183` without a compatible `Blizzard_Colors` TOC; relevant 18 namespace helpers and legacy `UnitCreatureID` pass, but the new retail-gated tests do not run there. This is not overall Mists compatibility proof. Malformed or alternate GUIDs, coercion/errors, native identity/database behavior, secret arguments, lifecycle, and consumers are unverified.

## Sources

- [Creature GUID identifier spec](../../specs/creature-id.md) — required bounded behavior.
- [12.0.0 API audit](../investigations/patch-12-0-0-api-audit.md) — audit evidence boundary.

## See Also

- [[patch-12-0-0-api-audit]] — 12.0.0 audit status.
