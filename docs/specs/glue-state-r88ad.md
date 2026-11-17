# Glue saved-account, connection, background and audio state (R88ad.7-10)

## Scope and evidence

Task [lt-fb68](https://tasks.sakuin.org/tasks/lt-fb68) quotes the R88ad.7-10
ANSWER for Retail **12.1.5.70688**. The runtime cache is Retail **12.1.5.70783**,
so consumer behaviour is a compatibility observation, not native conformance
proof. The producers are registered only for `client-retail`. PTR, WoW Forever,
Mists and Wrath keep the shared bootstrap defaults because this evidence does not
cover them. The globals stay exposed on Game screens as before; native Glue-only
scope is not claimed.

Labels: **VERIFIED** facts come from the ANSWER and are the only facts the tests
assert. **INFERRED** items are listed separately. **UNKNOWN** values are empty host
inputs in `GlueHostState` (`src/lua_api/globals/real/glue_state/mod.rs`).

The ANSWER labels publication blocks VERIFIED and the complete traversal of the
intervening native guards UNKNOWN. The simulator models the decoded publication
blocks as the normal result.

## What it must do

### R88ad.7 saved account and token login

- [x] VERIFIED: `GetSavedAccountName` and `GetSavedAccountList` read no argument and
  publish one string: the `accountName` / `accountList` setting text, or the empty
  fallback when the setting does not exist.
- [x] VERIFIED: `SetSavedAccountName` requires a string or number (numbers use Lua
  string conversion) and fails with `Usage: SetSavedAccountName(accountName)`
  otherwise. It copies the text through its first NUL into a present
  `accountName` setting and skips an absent one. It returns zero values.
- [x] VERIFIED: `ClearSavedAccountList` reads no argument, submits an empty string
  to a present `accountList` setting, does not clear the saved name and returns
  zero values.
- [x] VERIFIED: `SetUsesToken` requires one nonnil value (`Usage: SetUsesToken(usesToken)`).
  False writes `0` and every other value writes `1` to a present
  `g_accountUsesToken` setting. It returns zero values.
- [x] VERIFIED: No getter or setter dispatches a named event.

Simulator boundary: "setting exists" means the CVar store has a value for that
name (catalogue default, runtime registration, persisted override or imported
WTF value). The store has none of the three names by default, so on a fresh
profile the getters return `""` and the setters write nothing. The native
setter's runtime callback and its decline paths are not modelled. Non-UTF-8
bytes are stored lossily because the CVar store holds UTF-8 text.

### R88ad.8 connection, realm and expansion state

- [x] VERIFIED: `GetServerName` reads no argument. While the connection-availability
  flag is clear it publishes zero values. While it is set it publishes the name
  (or nil), the first and second `Cfg_Configs` Booleans, the additional connection
  Boolean, and the realm number of a present connection object (or nil). A row
  miss leaves all three Booleans false.
- [x] VERIFIED: `IsConnectedToServer` publishes the same flag as one Boolean. It is
  not `C_Login.GetState`'s `connectedToWoW`.
- [x] VERIFIED: `GetMinimumExpansionLevel` publishes the shared state as one number.
  The update routine stores a differing value and then fires
  `MIN_EXPANSION_LEVEL_UPDATED` with no payload; an equal value does nothing
  (`update_minimum_expansion_level`).
- [x] VERIFIED: `WasScreenFirstDisplayed` reads no argument and is true exactly at
  display state 2 (`SimState::screen_first_display_state`). Glue startup sets 2
  before it fires `SCREEN_FIRST_DISPLAYED`.

UNKNOWN host inputs: the connection snapshot (`GlueServerState`, unavailable by
default), its writers and event order; the minimum-expansion value Lua first
observes (empty input `0`, the zero-filled image storage). INFERRED and not
modelled: the native initializer value 10 is observable after initialization.
The simulator does not seed it, because the session's later writer is UNKNOWN.
The 0-to-1 display transition is collapsed by the host into the 2 write.

### R88ad.9 background selection

- [x] VERIFIED: `GetLoginScreenBackground(high, low)` requires two signed 32-bit
  numbers (numeric strings convert, range checked before truncation) and fails with
  `Usage: local result = GetLoginScreenBackground(highResFileID, lowResFileID)`.
  It publishes `high` when a native background object accepts it, otherwise `low`;
  an empty object list selects `low`.

UNKNOWN host input: `GlueHostState::login_background_file_ids`, the high IDs that
a native background object's file/model helper accepts. Empty by default, so the
low ID is selected.

### R88ad.10 audio controls

- [x] VERIFIED: `PlayGlueMusic(kit)` and `PlayGlueAmbience(kit [, fade])` use the
  signed 32-bit kit parser and their Usage texts; the fade must be finite
  (`must be a finite number`) and within the single-precision range
  (`outside of expected range -3.402823e+38 to 3.402823e+38`). All four calls
  return zero values.
- [x] VERIFIED: `PlayGlueMusic` does nothing when both control flags are set or the
  active current handle has the requested kit. Otherwise it stops an active current
  handle, always marks and stops the other handle, and for a positive kit retains
  it, submits a request and clears the current stop marker.
- [x] VERIFIED: `PlayGlueAmbience` skips an active handle with the same kit, stops
  another active handle, and for a positive kit clears the marker, retains the kit
  and submits a request with fade `trunc(f32(fade) * 1000)` (overflow gives
  `i32::MIN`; absent gives `-1`).
- [x] VERIFIED: `StopGlueMusic` marks both music handles and submits a stop for each;
  `StopGlueAmbience` marks and stops the ambience handle. Neither clears the
  retained kit.

Simulator boundary: submitted operations are recorded in
`GlueAudioState::requests`; there is no Glue audio backend. Handle activity and
the two music control flags are host inputs (inactive / clear by default) because
the backend result is UNKNOWN. Not modelled: the native helper that resubmits a
retained kit, audible results, the fade curve and later backend notifications.

## Original consumers and tests

`tests/glue_state_r88ad.rs` (Retail) drives the original 70783 consumers:
`AccountLogin_UpdateSavedData`, `AccountLogin_Login`,
`AccountLoginEditBox_OnTextChanged`, `AccountLoginDropdown_SetupList`,
`CharacterSelectUtil.GetFormattedCurrentRealmName` and `IsSameRealmAsCurrent`,
`TimerunningChoicePopupMixin:OnEvent`, `AccountUpgradePanel_GetDisplayExpansionLevel`,
`KoreanRatingsMixin:OnLoad`, `SetLoginScreenModel`, `ResetModel`,
`GlueParent_SetScreen` and `PhotosensitivityWarningFrameMixin:TryShow`.
`SetLoginScreenModel` uses a fixture expansion display record, because
`GetExpansionDisplayInfo` is an unmodelled temporary default.


## Retired defaults

On Retail, `shared_bootstrap.lua` no longer installs its defaults for the
fourteen R88ad.7, .8, .9 (`GetLoginScreenBackground`) and .10 globals
(`retireGlueStateDefaults`). Removal is behaviour-neutral: the shared bootstrap
runs before `register_globals`, whose Retail `glue_state::register` replaced each
default, and `restore_post_cleanup_globals` only fills nil globals. The unread
`SimState` fields `saved_account_name`, `saved_account_list` and `uses_token`
were removed. Other profiles keep the defaults.
