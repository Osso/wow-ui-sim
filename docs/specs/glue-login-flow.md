# Glue login submission

## Scope and evidence

Task [lt-a667](https://tasks.sakuin.org/tasks/lt-a667), ANSWER **30** (round R40),
is the native evidence and separately labeled simulator-policy authorization.
Native evidence covers Retail **12.1.5.70688** and Forever **1.60.1.70707**.
The runtime Retail pin is **12.1.5.70575**: build-specific native facts for that
pin remain **UNKNOWN**. Tests of its cached Lua are compatibility observations,
not native conformance proof. New `C_Login.Login` publication is limited to
Retail and Forever. Glue-only native registration is **INFERRED**, not verified.
Existing namespace publication in other profiles is unchanged. Native Glue-only
scope is not enforced by the current simulator registration of Login,
GetLogonQueueInfo or CanCreateCharacter: those remain exposed in Game. This is
an exposure limitation, not a native scope claim. PTR evidence below is 70668;
its committed runtime pin is 70493, another build mismatch with unknown specifics.

## Native contract

R40 verifies string/number account arguments and an EditBox frame-table argument.
Invalid account types, non-table password arguments, and unresolved
objects fail with `Usage: C_Login.Login("accountName", AccountLoginPasswordEdit)`.
A resolved non-EditBox instead raises the object-resolver error
`Wrong object type for function` (review132 correction).
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
- Forever's current nothing-pending defaults are `hasSelectedExperiencePreset=false`,
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
Review132 specifies signed-int32 inputs: positions <=1 clear the record, return
(false,0,0), and emit no login-state event.
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

`GetGameModeGlueScreenName` is registered in Glue and Game. The original R43
selected-realm interpretation is superseded by R60 below: both getters share
temporary-override/currentGameMode-CVar selection and host rows. Nonempty row
names are returned, otherwise nil. The display getter alone excludes record0;
the screen getter may read its supplied string. GlueParent supplies charselect
at Retail GlueParent.lua:220 / Forever:254; no fallback is invented in the getter.

The historical R42 Forever submission run reached absent `IsNameReservationOnly` at
`Blizzard_GlueXML/Mainline/CharacterSelect.lua:1003`. The R44 answer and implementation below resolve that gap.

The historical R43 submission tests reached visible charselect and hidden login,
then exposed missing SelectCharacter at Retail CharacterSelect.lua:860
(Forever:875). R45 below resolves QUESTION43 and its selection relationship.
An additional real-input test covers the empty host roster's charcreate path.

ANSWER **46** identifies the initial Login addon exclusion as simulator policy,
not the native load set (`Blizzard_CharacterCreate` allows Glue and is not LoD).
An experimental separate cached preload was reverted in `8ce6bd300` as directed.
R46 below resolves the eligible startup load set. The Retail empty-roster
charcreate regression now passes; no separate preload or startup replay is retained.

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
overridden by a persisted CVar. R47/R50 below resolve the name and coercion,
replacing the initial R40 Forever true simulator policy.

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
is UNKNOWN. Review132 assigns all shared-loader eligibility/phases work to the
separate R62/R64 lane. This branch restores master discovery, bootstrap and
screen-selection tests, including its existing Login exclusions. No preload,
timing fixture, vendor override or local eligibility change substitutes for
that lane. Any resulting real-consumer blocker must remain explicit.

The R45 selection event exposed missing GetCharacterTimerunningSeasonID at
Retail CharacterSelect.lua:483/455 and Forever:487/460. R49 below resolves
QUESTION57 without using the unrelated active-player season getter.

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

R47 identified the pre-login writer's changed CVAR_UPDATE followed by
unconditional LOGIN_STATE_CHANGED. R50 below resolves its argument contract,
backing effects, and the integer-prefix forms previously left unmodelled.

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
conversion/error behavior is not promoted to native evidence. R52 below resolves
GetSelectBackgroundModel and SetCharSelectBackground, reached by the original
Retail CharacterSelectUI.lua:374 consumer after the R49 season getter.

The Login-only headless refresh guard is approved in comment 73. Direct
CharacterSelect startup retains its compatibility refresh. The legacy
GetCharacterListUpdate workaround remains necessary: original
CharacterSelectListUtil.lua calls it, and R45/R49 do not register a replacement.
Its synchronous direct-handler behavior is not newly claimed as native evidence.
Login also skips the simulator's post-event chat activation fixture, preserving
GlueParent's existing visibility decision instead of showing ChatFrame1 again.

R55 (comment 86) identifies a cached-source split: Forever and PTR put the
always-true AllowChatFramesToShow in FloatingChatFrameGame.lua, selected only by
[AllowLoad game]. At Glue this file is skipped, preserving GlueParent's policy.
The Forever plain-login regression loads original addons with no post-load,
post-event, or settling fixtures and verifies the original GlueParent function
source, false policy, shown AccountLogin, and hidden ChatFrame1. It passes.
The complete authorized PTR cache is now verified. Its unassisted test also
passes every chat-policy/source/visibility assertion, but the final zero-Lua-error
guard fails on unrelated plain-load GetCurrentGameModeDisplayInfo and
AreHighResTexturesAvailable calls. These are reported for coordinator ownership
or native contracts; the test does not apply post-load workarounds to mask them.
At 72037ef24, the PTR glue run is 130 passed/3 failed with zero compiler warnings:
the two other failures are PTR feedback discovery/file-list expectations.

Retail 70575 defines the always-true function in unfiltered shared
FloatingChatFrame.lua:40, replacing GlueParent.lua:909's screen-aware policy.
Comment 99 retracts the earlier stale-pin diagnosis: the installed 70783 CASC
root also lacks FloatingChatFrameGame.lua, so a pin refresh will not fix this.
The native ordering/filtering/visibility explanation is UNKNOWN pending R62.
R62 below narrows the unsupported Retail visibility assertion without changing runtime behavior.


### R52 background lookup and supplied row boundary

Comments 80/83 provide Retail 70688 and Forever 70707 evidence and the host-data
policy. Existing simulator identity catalogs have no select-screen fields.
`BackgroundRows` therefore has explicit class-ID and race-ID maps, empty by
approved default. This is not a statement about the real DB2 table contents;
real data population belongs to the coordinator's separate task. A roster record
has a host race ID (default 0, not guessed from its social race-name string) and
an independent background override (native construction 0). Its class ID uses
the existing supplied account class ID. Timerunning does not select backgrounds.

The getter takes a one-based index, truncates toward zero, and resolves the
requested record independently of selection. Nonzero override, class select
field, race select field, then race low-res field form the cascade. Missing final
race row means 651555; present final zero means one nil. Invalid/empty/unresolved
slots use only key-zero race low-res, not a clamped first character. Tests supply
every row/value so they distinguish absent rows and zero fields. Ordinary numeric
strings are inferred; full grammar, nonfinite values and conversion overflow
remain outside verified coverage.

Both globals are Glue-only on Retail/Forever (normal scope is inferred from
native installation). PTR is unchanged and UNKNOWN. The setter validates numeric
file-ID or string input and returns zero values, but stores no fake observable
background/selection state. Retail 3D work is omitted. Forever readiness-event
payload, timing, repeat count and failures remain UNKNOWN; no synthetic
CHAR_SELECT_BACKGROUND_MODEL_READY is emitted. Filename resolution and all
indirect resource effects remain outside this bounded non-3D model.

### R54 independent automatic-boost session values

Comment 82 supplies Forever 70707 and Retail 70688 evidence. Two optional session
values, boost type and GUID, start absent and are separate from active purchase
and trial-service inputs. The getters/writers are installed in Game and Glue on
Retail/Forever before the existing protected-global wrappers. PTR is unchanged.
Tainted callers keep the existing report-and-return refusal without a combat
condition; no new access rule is added.

Present type returns one number, including zero. Absent type returns no values
(INFERRED), so tests assert assignment nil without asserting absence return count.
The GUID getter returns one string or nil. The ordinary int32, valid GUID-string
and explicit nil writer cases use existing shared readers; nil clears only its
own value. Full native writer conversions, malformed GUID handling and detailed
writer errors are UNKNOWN and not claimed by tests. No boost-change event or
synthetic CHARACTER_LIST_UPDATE is introduced.

The accepted CancelLogout path clears only boost type. An explicit Rust host
realm-disconnect notification has the same field clear, preserving GUID. There
is no native realm-disconnect callback wiring or invented Lua endpoint. Logout
and simulator screen boot must not clear these fields: shipped class-trial/store
Lua writes boost state before Logout. Broader logout/disconnect ordering remains
UNKNOWN.


### R58 character-model moves: explicit 3D gap

Comment 91 provides Retail 70688 and Forever 70707 evidence. Both
MoveCharactersToModelFFXFrame and MoveCharactersToMapSceneFrame read no Lua
arguments, perform no supplied-frame type checks, and return zero values.
Ordinary ignored extra arguments and normal Glue-only scope are inferred from
those handlers and registration. PTR is uninspected and unchanged.

The coordinator explicitly permits registered Glue-only zero-result no-ops for
these two model-transfer operations in the simulator without 3D rendering.
This is a bounded 3D gap, not a universal no-effect native contract: downstream
frame parenting, shown state, getters, events, and especially MapScene's final
indirect update remain unknown. No selected-index store or named event dispatch
was found in the checked direct native code.

ResetModel is original Blizzard_GlueParent Lua (cached Retail lines 779–785,
Forever 770–776), not a native API. It is not registered or replaced here.
Failures in its frame-method calls require separate evidence.

### R60 shared GameMode display selection and high-resolution capability

Comment 100 corrects R43 using PTR70668/Retail70688/Forever70707 evidence.
Both display and glue-screen getters select a bounded host temporary override
when enabled, otherwise the real currentGameMode CVar (registered default -1).
Host GameMode rows default empty. Missing rows, record0, and no-override -1 have
no display table. Empty screen fields return nil; GlueParent owns its fallback.
The old independent screen-name input and Forever display placeholder are gone.
A_Admin.SetActiveGameMode's optional screen remains an explicit administrative
host-row fixture, not native row data or a native writer.

The seven display fields are logo, logoHeight, logoVerticalOffset,
logoShrunkenHeight, logoUsesDarkBackdrop, characterCreateExtraHeight, and
characterCreateOuterBorder. The two file identifiers are omitted when zero.
The capability and display registrations apply in both Game and Glue; display
is Retail/PTR/Forever, while high-resolution availability is PTR/Forever only
and VERIFIED absent on Retail. The latter projects read-only byte0; its normal
false result and completed return tail are INFERRED. Review132 removes result
type/value assertions; only verified publication is asserted.

The bounded Rust host override setter models auto-connect selection without
inventing a Lua AutoConnectToGameModeRealm contract. The host clear_realm_list
lifecycle method clears the override; no ClearRealmList Lua API existed here.
Accepted currentGameMode CVar writes clear it and emit
GAME_MODE_DISPLAY_INFO_UPDATED through the existing synchronous dispatch path.
That callback timing relative to assignment and other events is simulator
scheduling, not native ordering evidence. Direct raw storage writes are not
callback-producing API calls. The native setter path and Lua CVar setter both
invoke this callback. Tests exercise original GameLogoDarkBackdrop Lua against
supplied rows, CVar changes, and the host clear operation.

UNKNOWN: full writer set, exact native callback timing/guards, loaded GameMode
rows, first login snapshot, computed writes to the capability byte, and complete
error lines. No row download or inferred nil-return arity assertion is added.

Cached-integer overflow is an explicit unsupported simulator input diagnostic,
not silently treated as no mode and not claimed as a native error contract.

### R62 Retail chat evidence boundary

Comment 104 permits narrowing only the Retail ChatFrame1 post-startup visibility
assertion: live AccountLogin chat visibility remains UNKNOWN. The simulator is
not changed to hide or show chat, and neither the original Lua nor its winning
function is overridden. PTR/Forever visibility checks, AccountLogin and
customization checks, other-profile chat-dock assertions, strict submission error checks,
and original input/transition tests remain intact. A Retail source regression
retains VERIFIED XML initial-hidden declaration, GlueParent's own login-false
policy, and the chat Lua's policy consultation. These source facts are not a
claim about the final live native visibility state.

R62 verifies phase1 LoadFirst/UseSecureEnvironment completes before native
non-insecure phase2 and remaining phase3; optional then required dependencies
precede an addon's own files. Correction109 identifies LoadManagers, not
LoadWith, as the list attempted before the own load request. Comment115 later
verifies Retail70688 LoadWith: X declaring Y joins Y's follower list without
loading Y. After Y's own file phase, startup and runtime paths submit followers
to the general loader (optional dependencies, required dependencies, own files).
An early-exit Y load submits no followers. An LOD target receiving a startup
request another way remains UNKNOWN, as do Forever/PTR LoadWith semantics.
Files use TOC line order,
and unannotated lines are accepted at Glue. Shared-loader deviations are
reported to the coordinator, not repaired in this lane. Case-insensitive name
order is only inferred and equal-name tie order unknown; neither is asserted.

Comment107 explicitly extends the same Retail-only UNKNOWN boundary to the
GeneralDockManager chat-role dock assertion. Both Retail post-startup chat
visibility expectations are omitted, without changing runtime behavior; other
profiles' dock checks and every non-chat visibility/error/input assertion remain.
The coordinator owns shared-loader ordering deviations in a separate lane.


### R63 credits inputs and capability evidence

Comment110 supplies PTR70668/Retail70688/Forever70707 evidence. PTR and Forever
publish GetCreditsDisplayInfo in Game and Glue, and the file-data-ID text API
only in the direct Glue catalog. Retail requested-name registration remains
unknown beyond absent exact image literals; its separate GetCreditsText is not
substituted. Row publication preserves creditsFileID0 and empty strings, omits
logo0, and distinguishes absent soundKit from present0. Native row data/order,
fresh PTR list and exceptional paths remain UNKNOWN. Comment112 approves
explicit host rows empty-by-default on both profiles, not fresh native state.
Empty-list table shape is verified for Forever and inferred for PTR. No vanilla
row is copied. R65 below supersedes the temporary text-map coverage boundary
and neutral result with verified input validation and Forever publication.


The five video capability bytes are separate from R60 high-resolution input.
Verified image bytes project to inferred normal Boolean results. A host seam
models those bytes explicitly, without treating the projections as native
observations or GPU/CVar state. Retail registers only spell density; PTR and
Forever register all five in Game and Glue. MultiSample remains a separate
runtime/device query, not a static byte; its PTR registration is UNKNOWN.

R63 implementation uses distinct per-environment host bytes with image-derived
inferred defaults: PTR linear0/outlineDisabled0/PBR0/secondary0/density1;
Forever all five1; Retail density1 only. Review132 removes inferred projection
assertions even for supplied bytes. Tests retain verified registration and
Forever return shape, plus original consumer loading/error checks.
The removed Forever credits notice/catalog and mutable video placeholders no
longer supply native-facing answers. The separate renderer MSAA workaround is
not relabeled as a static native capability or widened to PTR.

The original credits consumer fixture supplies rows/text before addon loading,
without replaying OnLoad. Opening credits with an empty host catalog would call
the text getter with nil at CreditsFrame.lua:112; R65 identifies this as a
required-argument error. Native fresh row catalogs remain unknown.


### R65 credits text input and publication

Comment116 resolves PTR/Forever input validation: numbers and numeric strings
must be finite and fit signed int32, with fractions truncated toward zero.
Missing/nil and invalid type/string/nonfinite/out-of-range errors share the
verified Usage core. The simulator uses that core without claiming a universal
full decorated Lua error line or boundary conversion details beyond the answer.

IDs resolve directly against the explicit host file-text map, without looking
up credits rows. Signed keys preserve the converted input. Return is one string
(including empty) or one nil for an absent file. Publication is VERIFIED for
Forever; PTR follows the coordinator-approved model without asserting its
unrecovered full publisher as native fact. Synchronous copy and no later event
are verified. Client-file origin and load-failure equivalence are inferred;
storage waiting and the complete publisher route remain unknown.

This supersedes earlier pending-R65 and neutral-zero-return descriptions.
Opening credits with an empty host catalog still passes nil from original Lua;
R65 now identifies that as a required-argument error rather than a simulator
coverage diagnostic. No fabricated default row is added to hide that condition.


## m35 integration validation (2026-11-12)

Master 926aadc0c (v0.1.41) was merged with --no-ff as 2ffbe6d36.
Integration fixes removed a duplicate numeric-parser export and its orphaned
profile gate; current code is 4d16f9ca7. The coordinator's moved character-service
state retains both queued-upgrade and automatic-boost inputs. Its character-create
cleanup remains intact; the obsolete name-reservation fallback does not replace
the R44 model.

Cached full glue tests: Retail 272 passed/0 failed (63.11s), PTR 144/0 (44.19s),
and Forever 168/4 (51.46s). Forever's run preceded the profile-neutral export-gate
fix; its original graphics/credits consumer suite subsequently passed 4/4
(4.39s) at 4d16f9ca7. All successful builds emitted zero warnings. Real Enter and
button submission to character select pass on Retail and Forever.

Three Forever character-create failures are confirmed master baseline by
comment122. Missing GetTrialBoostStartingLevel belongs to main's pending m36;
the raceData.racialAbilities contract belongs to main's R67. Neither is filled
or masked here. The fourth failure was this lane's stronger empty-roster test
asserting all downstream character-create setup is error-free. Comment124
explicitly scopes it to a passive observer of Blizzard EventRegistry's
GlueParent.ScreenChanged. The snapshot records screen, authenticated login state,
empty roster and Lua error count before original CharacterCreate OnShow. Only
that snapshot is asserted; original setup still executes and its later errors
remain recorded and printed. No vendor callback is replaced or suppressed.
This does not claim character-create setup succeeds; main will extend coverage
after R67.

At 28c142d6d the revised Forever full glue run passes 169 tests with only the
three confirmed master-baseline character-create failures (50.52s). Retail's
three real submission tests pass (4.20s). Both builds emit zero warnings.
The earlier full Retail 272/0 and PTR 144/0 results remain applicable: changes
since those runs are only the Retail/Forever empty-roster observer test and
documentation. Read-only review found no remaining integration or observer
defects. All validation jobs completed.

Comment120 supersedes the earlier seven-profile all-target lane gate: targeted
PTR/Retail/Forever proof is required; final all-profile integration and GitHub CI
belong to main. No shared-loader changes, native-data fetches or Blizzard CDN
requests were made.


## PR54 review corrections (comment132)

Login chat activation and headless character-select refresh skips apply only to
Retail/PTR/Forever. Era/Anniversary retain the old game-mode screen-name state
and CharacterSelect default, outside R43/R60 evidence. Shared-loader edits are
removed rather than repaired in this lane.

R65 now distinguishes required-argument errors (missing/nil) from signed-int32
range errors (bad type/string/nonfinite/out-of-range), retaining the verified
Usage core. Tests check categories, not the full decorated diagnostic.
Unsupported numeric-string selection/background and extra-argument assertions,
Game absence assertions for R44, PTR empty-catalog/publisher claims, capability
projection tails and high-resolution return-type claims are removed. Verified
publication, field serialization and unrelated consumer error checks remain.


### Review validation after m36

Master b4a4a14a5 was merged with --no-ff as fd5e2a995. Its native trial-boost
registration and screen hooks remain unchanged. At 6b1454c53, the changed-test
batch passes Retail 48/48 (8.40s) and Forever 60/60 (21.18s). At 1bae720ff,
PTR passes 38/38 (9.44s). The only intervening source change scopes the PTR
strict-startup diagnostic check as explicitly approved by comment136.

That check retains and prints exactly two known R64 loader residues:
CharCustomizeFrame nil at Blizzard_PTRFeedback.lua:729 and
Blizzard_PTRFeedback_Events_Glue.lua:71. Review140 replaces the earlier suffix matcher with exact cached-source
first-line matching, allowing only the known Lua Error wrapper on PTR; other errors and all Forever errors
remain failures. No loader workaround or vendor override is introduced.

The runtime batch uses integration-test filters glue_login, glue_credits,
glue_video, glue_startup_scope, game_rules and forever_login_catalog_defaults.
The new legacy cfg paths also pass cargo check --test integration for Era,
Anniversary and Wrath (154s, 50.72s and 48.50s), with incremental compilation
disabled for these checks. All six logs have zero compiler warnings.
These are targeted checks, not a new all-profile/full-suite acceptance claim.
The shared-loader files exactly match m36 master. All jobs completed.


### Second review corrections (comment140)

Login now resolves frame arguments through the existing fallible frame helpers
and applies the existing forbidden-frame/tainted-caller guard used by
CreateFontString. Plain unresolved tables still raise Usage; wrong resolved
types still raise Wrong object type for function. A forbidden resolved EditBox
called from AddOn-tainted code raises the supplied forbidden-object error
before login state or password changes. A regression uses the real forbidden
loading flag and tainted Lua function, without replacing the login provider.

Remaining unsupported argument-conversion and Game-absence assertions were
removed, including the inferred R58 move-global absence check. Production
registration is unchanged. Retail video tests assert only the verified spell
density publication, not absence of other methods.

The PTR R64 allowance now matches only the two exact cached-source first lines,
optionally prefixed once by the known Lua Error wrapper. It no longer accepts
arbitrary prefixes or suffix matches. Negative tests reject other paths,
line numbers, frames, preceding errors and arbitrary text prefixes.

Changed-test filters glue_login, glue_credits and glue_video pass Retail32/32
(8.05s) and Forever41/41 (15.36s) at e58b80af1, and PTR22/22 (6.22s) at
46d273b2e. Changes after the PTR run only fix the Retail/Forever forbidden test
fixture and remove a Retail/Forever inferred assertion. All three builds have
zero compiler warnings; no jobs remain. Master remains m36 b4a4a14a5.
These targeted receipts supersede earlier failed fixture iterations, without
claiming a new full-suite or all-profile matrix run.
