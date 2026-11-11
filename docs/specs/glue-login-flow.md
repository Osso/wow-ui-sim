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

ANSWER **51** supersedes the R40 Forever preset policy: the absent-setting
fallback is now false (PTR remains true), with no fabricated setting override
before R47. Consequently, the original Forever GlueParent may select
`experiencepreset` until that host setting is supplied; tests must not force
charselect around it.

The same answer supplies PTR70668 field projections. `PtrConnectionState` keeps
the connection/pending/WoW-queue inputs separate from Battle.net queue snapshots,
requires a present and available realm-list object, gates rule-set and
superdistrict flags independently, and suppresses realm waiting during an active
superdistrict choice. Connected-to-WoW suppresses the reported Aurora state to
NONE. Product gates are host inputs, not guessed product configuration. Explicit
screen boot seeds the existing connected/disconnected simulator fixture; it is
not a native reset/disconnect callback. Host field changes emit no invented
notifications. These PTR formulas are not silently extended to Forever; only
its explicitly superseded preset fallback changes. Tests cover connection
precedence, object availability, independent flags/gates, suppression and all
nine idle fields. Native reset/disconnect details remain documented evidence,
not new callable native functions.

ANSWERS **52/54** (R45, Retail70688 / Forever70707) resolve selection. The host
roster now contains slots with a character ID and an optional resolved record,
so empty and unresolved slots are retained and counted. Resolved records must
match their slot ID. Capacity counts nonzero IDs independently of resolution.
The direct-screen Battle.net fixture is converted into slots without becoming
submitted-login account data.

`SelectCharacter` converts a number, truncates finite supported values toward
zero, then converts one-based to zero-based; an invalid numeric slot falls back
to slot zero. Bad argument types raise exactly `Usage: SelectCharacter(index)`.
It returns no values. Empty rosters, unchanged selection, or a normalized slot
without a resolved record produce no selection/event. A changed selection is
stored before `UPDATE_SELECTED_CHARACTER` with one one-based integer. The
getter returns the stored zero-based index plus one, initially zero. 3D effects
and the inferred CVar write are omitted.

Plain numeric-string acceptance is INFERRED; full string grammar is UNKNOWN.
Native conversion width, NaN, infinity and overflow behavior are UNKNOWN and
untested. A defensive signed-32-bit finite input envelope returns an explicit
simulator-coverage error outside that envelope. This is not the native numeric
range or native error contract, and no native conformance is claimed there.
No saturating Rust conversion defines selection for those unsupported inputs.

Full-list processing resets selection to -1 before `CHARACTER_LIST_UPDATE`.
The event's count includes empty and unresolved slots. A separate zero-based
host saved-candidate input is unset by default because its native initial value
and storage are UNKNOWN. Unset/invalid input starts at the first slot, searches
forward and wraps for a resolved record, and falls back to the first slot when
none resolve. Empty lists produce no candidate. The computed host output does
not restore selection and is not written into a native setting. Original Lua
continues to read its existing lastCharacterIndex and call SelectCharacter.
Tests cover fractions 2.9/0.5, plain integer strings, bad arguments, invalid-slot
fallback, no-op cases, state-before-event, full-list reset/count, and the separate
candidate search. They do not assert unsupported native conversion boundaries.

ANSWER **56** (R46, Retail70688 / Forever70707, independently verified) resolves
the loader gap. Native Glue startup submits every eligible non-LoD addon across
three passes, including an early LoadFirst pass. CharacterCreate therefore
belongs to the initial Glue load set. Connection-independent loading and keeping
the addon set during login-to-charselect are INFERRED. Required dependencies
load first, including LoD dependencies; requesting an already loaded addon does
not reload it. Ordering against GlueParent OnLoad or the first login-state event
is UNKNOWN. The simulator removes the three CharacterCreate/Customize/
TimerunningCharacterCreate Login exclusions and uses its existing TOC and
dependency ordering. No separate preload, teardown, timing fixture, or vendor
patch is added. Synthetic loader tests assert eligibility and dependency order;
real cache tests include CharacterCreate on Login. Any newly exposed missing
native API remains a contract question rather than a reason to restore exclusions.

The R45 selection event exposes missing `GetCharacterTimerunningSeasonID` at
Retail CharacterSelect.lua:483/455 and Forever:487/460. QUESTION **57** requests
its contract. It is not replaced by the unrelated active-player season getter.

For Retail/PTR/Forever, pre-submission Login also uses the empty-by-default host
roster, not social-account fixtures. Explicit CharacterSelect/Create/Game boots
retain the historical fixture path. This is the simulator data-input boundary,
not a claim about native roster-arrival timing. A regression supplies social
accounts while asserting an empty Login roster, preserved host holes/indexes,
and the separate direct-screen fixtures.

ANSWER **59** (R47, PTR70668 / Forever70707, independently verified) identifies
`hasSelectedExperiencePreset` as a real registered local CVar, not a host-only
flag. Its PTR/Forever defaults are respectively `1`/`0`; missing records retain
the corresponding declaration fallback. GetState reads the integer-prefix
projection: nonzero is true, words such as `true`/`false`/`junk` are false,
`1junk` is true, and hexadecimal prefixes are accepted. The model reads the real
CVar store without changing generic GetCVarBool semantics. Overflow and locale
behavior remain UNKNOWN. Generic SetCVar does not dispatch LOGIN_STATE_CHANGED;
its existing value-update/CVAR_UPDATE path is retained. The real UI fixtures set
this CVar to `1` when selecting the ordinary Forever character-screen scenario.

The native pre-login preset writer's documented event order is value/cache
update, CVAR_UPDATE(name, "1") if changed, then unconditional no-payload
LOGIN_STATE_CHANGED, including unchanged or failed writes. QUESTION **60** asks
for the writer's required preset-argument contract and other backing effects,
plus remaining coercion details; no setter argument semantics or incoming native
callback source is invented. That writer remains pending.

Per ANSWER61, signed, whitespace-prefixed and leading-zero forms remain
unmodelled pending R50. They currently produce an explicit simulator-coverage
diagnostic rather than a guessed native value/error contract. Tests use only
R47's verified ordinary decimal/hex cases. This temporary coverage limit is not
native validation behavior.

ANSWER **64** (R49, Retail70688 / Forever70707, independently verified) resolves
GetCharacterTimerunningSeasonID. It accepts one required GUID string or number
(string conversion, not slot indexing), ignores extra arguments, and returns
exactly one nonzero season integer or nil for unknown/unresolved/zero-season
records. Bad/missing/nil/boolean/table arguments use the supplied usage error.
CharacterRecord now owns an optional season, defaulting to native construction
zero. The getter uses requested-GUID record resolution and never the selected
character or active season. Glue-only exposure is INFERRED and profile-scoped.
No extra events, GUID grammar, secret handling or other writers are invented.

### R50 preset writer and cached-integer projection

Tracker lt-a667 comment 66 supplies Forever 70707 / PTR 70668 evidence.
The setter records changed/classic-chosen state, writes the real CVar through a
native path (not Lua SetCVar access checks), emits CVAR_UPDATE only on a changed
value, then always emits payload-free LOGIN_STATE_CHANGED. It returns no values.
There is no GAME_RULES_CHANGED, fabricated validation callback, server response,
or implementation of the unknown native refresh. Optional outgoing requests are
omitted. Glue and Game registration is an inference. Required numeric conversion
uses signed int32, truncation and the low byte, accepting only resulting 0 or 1.
The shared CVar truth projection skips ASCII whitespace and an optional sign,
but selects hex only when the original string starts 0x/0X. Leading zero decimal
and trailing junk are accepted. Overflow and non-ASCII locale parsing remain
outside verified coverage. Generic GetCVarBool behavior is not broadened.

R50 follow-up (comment 71): fractional int32 boundary range-check timing,
low-byte rejection's exact core error text, and table/function argument handling
are UNKNOWN. No conformance assertions cover those cases; ordinary shared
conversion/error behavior is not promoted to native evidence. R52 is pending
for GetSelectBackgroundModel and SetCharSelectBackground, reached by the original
Retail CharacterSelectUI.lua:374 consumer after the R49 season getter.
