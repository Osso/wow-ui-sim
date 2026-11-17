# Retail Glue host state (R88ad.7-10)

**VERIFIED (simulator source):** Retail registers the saved-account, connection,
minimum-expansion, first-display, login-background and Glue-audio globals from
`src/lua_api/globals/real/glue_state/`. Their inputs live in `SimState::glue_state`
(`GlueHostState`), the CVar store and `SimState::screen_first_display_state`.
The [contract](../../specs/glue-state-r88ad.md) carries the per-fact evidence labels.

## Content

### State sources

- **Saved account (R88ad.7):** the `accountName`, `accountList` and
  `g_accountUsesToken` CVars. A name with no value in the store is an absent
  setting: getters publish `""` and setters skip the write. Seed one with
  `CVarStorage::set_imported` (the WTF import path).
- **Connection (R88ad.8):** `GlueServerState`. `available` gates both
  `GetServerName` (zero values when clear) and `IsConnectedToServer`. It is
  independent of `C_Login.GetState`'s `connectedToWoW`.
- **Minimum expansion (R88ad.8):** `minimum_expansion_level`, written by
  `update_minimum_expansion_level`, which fires `MIN_EXPANSION_LEVEL_UPDATED`
  only for a differing value. The empty host input is 0; the native initializer
  value 10 is not seeded because the session writer is UNKNOWN.
- **First display (R88ad.8):** `screen_first_display_state` (0, 1, 2). Glue startup
  writes 2 before it fires `SCREEN_FIRST_DISPLAYED`; screen changes and UI reload
  reset it to 0 (host policy).
- **Login background (R88ad.9):** `login_background_file_ids`, the high-resolution
  IDs a native background object accepts. Empty selects the low-resolution ID.
- **Audio (R88ad.10):** `GlueAudioState`. Submitted start/stop operations are
  appended to `requests`; handle activity and the two music control flags are host
  inputs because there is no Glue audio backend.

### Profiles

Only `client-retail` registers these producers and retires the matching
`shared_bootstrap.lua` defaults (`retireGlueStateDefaults`). PTR, WoW Forever,
Mists and Wrath keep the defaults until evidence for those builds exists.
`IsTestBuild` returning false is an INFERRED Retail change with no test assertion.

### Consumer tests

`tests/glue_state_r88ad.rs` drives the original 70783 Glue consumers. Two
harness details matter: `AccountLogin` edit-box `OnTextChanged` is delivered at
the next update flush (`fire_on_update`), not inside `SetText`; and
`LoginBackgroundModel:SetModel(fileID)` does not record numeric IDs, so the test
hooks the call to read the consumer's argument.

## Sources

- R88ad.7-10 ANSWER quoted in lt-fb68 (Retail 12.1.5.70688)
- `src/lua_api/globals/real/glue_state/`
- `src/lua_api/env_init/shared_bootstrap.lua`
- `tests/glue_state_r88ad.rs`

## See Also

- [Retail Glue login lifecycle](glue-login-lifecycle.md)
- [Consumer selection and RED proof](../investigations/consumer-selection-and-red-proof.md)
