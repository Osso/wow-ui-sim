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

ANSWER **38** (R42, Retail70688 / Forever70707) resolves QUESTION32.
`C_Login.GetLogonQueueInfo` takes no arguments, has no usage error, and returns
exactly `(isQueued, queuePosition, estimatedSeconds)`. Default/no queue is
`(false, 0, 0)`. A supplied active Battle.net queue record produces server
position minus one and the last supplied estimate; it does not count down.
This record is independent of the WoW connection state. Every active host update,
including an identical one, publishes no-payload `LOGIN_STATE_CHANGED`.
`update_logon_queue` is the simulator host-input producer; clearing the record
changes getter output without inventing an unspecified inactive-update event.
Glue-only registration is INFERRED. The exact original getter consumers are
Retail GlueParent.lua **248** and Forever **282**. A focused test covers the
empty/default case, repeated active updates, snapshot behavior and independence
from WoW connection state.

The coordinator owns the five existing Forever character-select/create failures
on `forever-int-clean` (lt-95ac ANSWER 28). Integrate master after its merge; do
not duplicate those fixes here. No acceptance claim is made before final tests.

ANSWER **42** (R43, Retail70688 / Forever70707) resolves QUESTIONS34/35.
`CanCreateCharacter` has no arguments or usage error. Registration is verified;
Glue-only scope and the one-boolean return are INFERRED. It returns false while
connection readiness is clear, otherwise compares the number of nonzero roster
GUIDs with the stored signed limit. It does not test character-list arrival.
`ConnectionInfo` is host input; default readiness is false and no realm capacity
is invented. The offline policy delivers the supplied readiness and limit with
the roster transition. Empty host GUID strings represent absent/zero entries for
this comparison. Tests cover unready, zero/negative limits, absent GUIDs, equality
and spare capacity. The nonempty UI fixture explicitly supplies readiness and
limit 2; neither value is a native realm default.

`GetGameModeGlueScreenName` is registered in Glue and Game. Native selection is
override record, then selected-realm record, then no-mode sentinel; nonempty
record names are returned, otherwise nil. Ordinary-play nil and the absence of a
native charselect fallback are INFERRED. The simulator stores the resolved name
as optional host input, default None; empty names also return nil. It does not
fabricate record-selection data. The original GlueParent supplies `charselect`
at Retail GlueParent.lua:220 / Forever:254. Updated tests assert nil for default
and empty inputs, preserve explicit nonempty names, and require the original
GlueParent fallback in the real login-submission tests.

The R42 Forever submission run also reaches absent `IsNameReservationOnly` at
`Blizzard_GlueXML/Mainline/CharacterSelect.lua:1003`. Its existing approved false
default is PTR-only. A separate tracker question requests the Forever contract
or confirmation that coordinator cleanup already owns it; no return is guessed.

After R43, both Retail submission paths reach visible charselect and hidden
login through the original GlueParent fallback. Their strict no-error assertion
then exposes missing `SelectCharacter` at Retail CharacterSelect.lua:860
(Forever:875). QUESTION43 requests that native setter and its relationship to
`GetCharacterSelection`; it is not replaced by a guessed selection shortcut.
An additional real-input test covers the empty host roster's charcreate path.

ANSWER **46** identifies the initial Login addon exclusion as simulator policy,
not the native load set (`Blizzard_CharacterCreate` allows Glue and is not LoD).
An experimental separate cached preload was reverted in `8ce6bd300` as directed.
Native initial-load timing and ordering await R46. The empty-roster charcreate
test remains RED on Retail; no separate preload or startup replay is retained.

ANSWER **47** (R44-Q18, PTR70668 / Forever70707) resolves name reservation.
`IsNameReservationOnly` is a Glue global on PTR and Forever, absent on Retail70688.
It takes no arguments and returns one boolean from a feature-system status byte,
independent of realm and character. Its verified initial value is false; an
ordinary server-sent value is UNKNOWN. `GlueFeatureStatus` starts false and only
`deliver_status` replaces it. The host action emits no-payload
`TIMERUNNING_SEASON_UPDATE`, writes the status, then emits no-payload
`AVAILABLE_GAME_MODES_UPDATED`, preserving the supplied posting order. Host
notifications dispatch synchronously; tests do not claim native callback latency
or handler-time visibility. Tests reject a constant-false getter, cover repeated
records, exact event order/count/payload and independence from screen resets.
The getter is unpublished in Game and restored on a transition back to Glue,
without replacing the feature status; tests cover both directions and fresh Game.
The old PTR constant-false workaround is removed, not layered over the model.

ANSWER **48** additionally specifies PTR GetState's nine-field table and native
hasSelectedExperiencePreset declaration defaults: true on PTR, false on Forever,
overridden by a persisted CVar. QUESTION **50** requests that CVar's exact name
and read/coercion rule, plus remaining PTR flag defaults/backing writers. The
previous R40 Forever true value is explicitly simulator policy, not a native
default; this distinction remains until the verified replacement is available.
