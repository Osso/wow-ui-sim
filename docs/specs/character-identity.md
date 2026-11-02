# Explicit imported character identity

An explicit `WOW_SIM_WTF_CHARACTER` selects the imported character whose identity
addons observe before loading. The same selection is reapplied by the existing
startup loader during ReloadUI. The WoW WTF input remains read-only.

## Selection and startup

When the character variable is nonempty, startup requires a valid WTF root and
the exact selected character directory. Explicit `WOW_SIM_WTF_PATH`,
`WOW_SIM_WTF_ACCOUNT` and `WOW_SIM_WTF_REALM` values cannot fall back to another
root, account or realm. Missing or non-directory components return an error with
the failing component and path before addons load. Omitted account/realm values
retain existing discovery. An absent or empty character variable retains
optional discovery and the simulator's default identity.

The persistence manager and identity initializer use the same resolved
`WtfConfig`. With `--no-saved-vars`, the read-only EditMode-cache source still
supplies identity, without enabling SavedVariables persistence.

Identity is assigned before EditMode/snapshot imports and Blizzard/third-party
addon loading. ReloadUI reconstructs through that startup loader and reapplies
the selected name and realm, replacing later scripted identity edits. The
existing player model continues to preserve current class, race and specialization.

## Supplied query behavior

Player state stores a name plus optional surname and realm. Forever splits the
selected directory name at the first `-` when both parts are nonempty. Other
profiles preserve the entire character component. Empty halves are not split.
The directory name used for character SavedVariables remains intact.

- Forever `UnitName("player")` and `UnitNameUnmodified("player")` return the first
  name and surname as two values. Unsupplied surname state projects nil as the
  second value. Absent units return two nils.
- Forever `UnitFullName("player")` also returns first name and supplied surname.
  Other profiles, and the existing unsurnamed simulator fixture, retain the
  modeled name and realm behavior.
- `GetRealmName` returns the supplied realm. `GetNormalizedRealmName` removes
  whitespace and hyphens from it.
- Local-player `GetPlayerInfoByGUID` and, where published, `UnitNameFromGUID`
  return the supplied realm. The modeled GUID itself is unchanged.
- Unsupplied realms retain each API's previous fallback. The unit-token surname
  and realm overrides apply to `player`; existing name resolution for other unit
  tokens is reused.

The native Forever 1.60.1 build 70450 full-probe recording confirms separate
first-name/surname values and exact arity two for `UnitName`, `UnitNameUnmodified`
and `UnitFullName`. Absent units return two nils. Its recorder preserves trailing
nils with `select("#", ...)`. A separate build 70009 firsthand report confirms
the player name/surname tuple. These observations supersede the earlier
consumer-only assumption that `UnitFullName` must return the realm.

The captures do not establish a valid unsurnamed player's second-value payload,
native name-policy defaults or separator value. Unsupplied surname projection
remains simulator policy; no blanket native parity is claimed for those states.

## Focused coverage and limits

- `src/paths.rs` tests exact Unicode selection, missing root/account/realm/character,
  non-directory inputs and discovery for unspecified account/realm values.
- `src/bin/wow_sim/startup_identity_tests.rs` uses subprocess-isolated WTF inputs
  and an actual addon. It checks identity/profile values during file execution
  and `ADDON_LOADED`, fresh-VM reload, preserved current class/race/spec, unchanged
  imported bytes, disabled persistence, missing-selection failures and absent/empty
  opt-in behavior.
  These full startup cases require `casc` for CASC-backed profiles; Wrath uses
  its existing pinned source loader.
- `saved_var_config.rs` tests profile-specific splitting and empty/multiple-hyphen
  boundaries. Player-identity unit tests check realm/GUID query values, arity,
  isolation and fallback restoration.
- `tests/wowforever_name_policy.rs` covers the original Camelot display,
  unmodified-full-name and player-matching consumers with split surname input,
  plus the existing combined-name regional-policy case.

The runtime-selected Forever `Camelot/NameUtil.lua` still has an inconsistent
`GetUnitFirstName` branch: with regional names enabled it pattern-matches a
combined name in the first API return. Separate first-name/surname input can
therefore return nil in that branch. The new split-input consumer test covers the
default disabled regional policy and the display/matching helpers; it does not
establish full regional-policy compatibility. Blizzard Lua is unchanged.
The same native capture records regional names enabled alongside the separate
name/surname tuple, so the API is not changed to satisfy that Lua branch.

## Sources

- [SavedVariables contract](saved-variables.md)
- [Startup selection](../../src/bin/wow_sim/saved_var_config.rs)
- [WTF discovery](../../src/paths.rs)
- [Regional name policy](regional-unique-names.md)
- [ReloadUI](reload-ui.md)
- [Native name-query capture, build 70450](https://github.com/DomenikIrrgang/wowlua/blob/bc4d518ce84193336055b61f3e1597b8760ef82b/crates/wowlua-engine/tests/client/forever-camelot-fullprobe.lua#L32406-L32427)
- [Native UnitFullName capture](https://github.com/DomenikIrrgang/wowlua/blob/bc4d518ce84193336055b61f3e1597b8760ef82b/crates/wowlua-engine/tests/client/forever-camelot-fullprobe.lua#L31179-L31185)
- [Arity-preserving recorder](https://github.com/DomenikIrrgang/wowlua/blob/bc4d518ce84193336055b61f3e1597b8760ef82b/tests/ingame/src/fullprobe.wowlua#L47-L49)
- [Recorded regional flag and player tuple](https://github.com/DomenikIrrgang/wowlua/blob/bc4d518ce84193336055b61f3e1597b8760ef82b/crates/wowlua-engine/tests/client/forever-camelot-engine.lua#L12764)
- [Firsthand player observation, build 70009](https://github.com/xIGBClutchIx/ForeverPlusPlus/blob/bcebbf95ab97013498464645e5239945aa7b3870/docs/forever-api.md#L61-L66)
