# Glue login submission

## Scope and evidence

Task [lt-a667](https://tasks.sakuin.org/tasks/lt-a667), ANSWER **30** (round R40),
is the native evidence and separately labeled simulator-policy authorization.
Native evidence covers Retail **12.1.5.70688** and Forever **1.60.1.70707**.
The runtime Retail pin is **12.1.5.70575**: build-specific native facts for that
pin remain **UNKNOWN**. Tests of its cached Lua are compatibility observations,
not native conformance proof. New `C_Login.Login` publication is limited to
Retail and Forever. Glue-only native registration is **INFERRED**, not verified.
Existing namespace publication in other profiles is unchanged.

## Native contract

R40 verifies string/number account arguments and an EditBox frame-table argument.
Invalid account types, non-table password arguments, and unresolved/non-EditBox
objects fail with `Usage: C_Login.Login("accountName", AccountLoginPasswordEdit)`.
The password is read and cleared synchronously; the call returns zero values.
Native Login does not reject empty strings; the shipped UI does so before calling.
The simulator does not retain or log credentials and clears the EditBox through
its normal text mutation path. This is not a guarantee of memory zeroization.

`LOGIN_STATE_CHANGED` has no payload. CONNECTING (3) can be posted synchronously
inside Login. Later native changes come from network callbacks; universal order
and frame counts are **UNKNOWN**. CONNECTING -> CONNECTED -> authenticated NONE is
**INFERRED** normal success. A WoW-authenticated connection reports NONE (1), and
its state event causes the original GlueParent to choose the next screen.

Retail `GetState` returns five values: aurora state, WoW connected flag, WoW
connection state, realm-list availability, and waiting-for-realm-list flag.
Forever returns one table with those five fields plus `waitingForRuleSets`,
`superDistrictChoicePending`, `hasSelectedExperiencePreset`, and
`keepingCharacterListOnRealmJoin`. The initial Retail tuple
`(1, false, 1, false, false)` is **INFERRED**. Existing global Aurora constants
1–9 and WoW-connection constants 1–3 already match R40; no Enum is introduced.

`CHARACTER_LIST_UPDATE` takes one optional integer: full-list updates may supply
0, while other sources may supply nil. Its native ordering relative to login
state events is **UNKNOWN**. Realm-list availability is not a record count.
An empty roster normally causes the original CharacterSelect to show charcreate,
subject to its game-mode, undelete, return-navigation and kiosk gates. There is
no native charcreate event.

## Explicit offline simulator policy

- Any credentials are accepted. No network authentication is attempted.
- Login publishes CONNECTING synchronously. Later update ticks publish CONNECTED,
  then authenticated NONE, then the host roster count as `CHARACTER_LIST_UPDATE`.
  This ordering and update cadence are policy, not a native scheduling claim.
- `SimState.glue_login.characters` is the host-supplied roster, empty by default.
  It reuses the existing character-query record representation. Submitted login
  queries use this roster, never unrelated seeded Battle.net friends. Existing
  direct-screen boot compatibility stays unchanged.
- `realm_list_available` is the host's realm-list availability input, false by
  default. Discovery and populated realm record schemas are not fabricated.
- Forever's nothing-pending defaults are `hasSelectedExperiencePreset=true`,
  `superDistrictChoicePending=false`, `waitingForRuleSets=false`, and
  `keepingCharacterListOnRealmJoin=false`. Existing kiosk policy is unchanged.
- A new explicit screen boot cancels pending login progression, preserving host
  inputs. Repeated Login replaces the pending offline progression.

## Original consumers and tests

Paths are relative to each profile's pinned AddOns cache. Both select Mainline
`AccountLogin.lua` through `[Family]`; Forever's Camelot override only changes
an edit-box anchor. Retail `AccountLogin.lua:204` and Forever `:213` call Login
with the actual password EditBox. Retail XML `:104,137–141,150–155` connects Enter
and the login button to the original handlers.

`Blizzard_GlueParent/Mainline/GlueParent.lua:157` (Retail) / `:172` (Forever)
handles the state event. GetState consumers are Retail `:203,216,243` and Forever
`:221,242,277`. CharacterSelect handles list/state updates at Retail `:401,594`
and Forever `:407,609`.

`tests/glue_login.rs` contains RED-first Enter and button tests that hit-test
visible edit boxes, type through key dispatch, verify text, and submit through
the real handlers. A nonempty host fixture is required for charselect. Tests do
not replay startup, call a screen-switch shortcut, or assert native frame counts.
Separate API tests cover validation, password clearing, zero returns, both
GetState shapes, state-event payloads, empty roster update payloads, supplied
realm availability, replacement of pending attempts, and explicit-boot
cancellation with host inputs preserved. These are offline policy tests, not
proof of native cancellation or retry semantics.

## Remaining dependencies

QUESTION **32** requests the missing `C_Login.GetLogonQueueInfo` contract. Exact
consumer call sites are Retail GlueParent.lua **248** and Forever **282** (the
question's initial line numbers were off by one). No implementation is inferred
from the Lua assignment alone.

The coordinator owns the five existing Forever character-select/create failures
on `forever-int-clean` (lt-95ac ANSWER 28). Integrate master after its merge; do
not duplicate those fixes here. No acceptance claim is made before final tests.

QUESTION **34** requests `CanCreateCharacter`, reached by full list updates at
Retail `CharacterSelect.lua:788` and Forever `:803`. QUESTION **35** requests the
ordinary/no-override `C_GameRules.GetGameModeGlueScreenName` contract. Its current
`"CharacterSelect"` scenario default is not a key in the original `GLUE_SCREENS`.
The getter's exact screen-selection consumers are Retail GlueParent.lua **220**
and Forever **254**. The declaration says nonnil cstring, but Lua's fallback and
empty-roster gate depend on absence. No return is guessed to bypass those gates.
