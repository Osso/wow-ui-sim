# Character creation submission

Initial slice: Retail and WoW Forever; the PTR addendum is below. Evidence is coordinator `agent-server-wow-ui-sim`
on tracker task `lt-a912`, especially R78 answers #205, #208, and #210 (Retail 12.1.5.70688 and
Forever 1.60.1.70707). VERIFIED contracts are distinguished from simulator host
inputs and UNKNOWN behavior below.

## Real consumers

The owner-hosted Retail 12.1.5.70575 UI calls
`C_CharacterCreation.CreateCharacter(name, useNPE, raceFaction)` from
`Blizzard_CharacterCreate/Blizzard_CharacterCreate.lua:696–709`.
Its result handler is at line 113 and name validation at line 2119.

Forever 1.60.1.70707 selects the **Camelot** consumer through its TOC `[Game]`
entry (`AllowLoadGameType camelot`); the `[Family]` Mainline entry excludes
Camelot. The active `Blizzard_CharacterCreate/Camelot/Blizzard_CharacterCreate.lua`
calls `CreateCharacter(name, surname, useNPE, raceFaction)` at lines 811–826
and handles results at line 137. Runtime traceback independently confirms this
selection. The tests must use these real edit boxes and inherited button scripts,
not direct mixin invocation or a replacement submit implementation.

## VERIFIED native contracts (R78 #205)

- `C_CharacterCreation.OnPlayerInteraction` reads no arguments, has no Usage
  check, returns zero values, and writes interaction byte `1` even without a
  model. Its body dispatches no named Lua event. The byte's meaning/readers are
  UNKNOWN. The simulator exposes the byte to the host without interpreting it.
- `StopAllSpellVisualKitsOnCharacter` reads no arguments, has no Usage check,
  returns zero values, and clears visual-kit state even without a model. Native
  stop/finish helpers run when a model exists; no Lua-visible effect is established.
- Retail Usage is
  `Usage: C_CharacterCreation.CreateCharacter(name, useNPE [, raceFaction])`.
  Name accepts string or number, with numeric text conversion. `useNPE` uses
  Lua truthiness; optional raceFaction accepts string or number. No surname.
  #241 clarifies useNPE is REQUIRED on both profiles: missing or nil raises Usage;
  truthiness applies only to present non-nil values.
- Forever Usage is
  `Usage: C_CharacterCreation.CreateCharacter(name, surname, useNPE [, raceFaction])`.
  Name and surname are required strings or numbers (numbers convert to text).
  Optional faction accepts strings or numbers; Forever numeric faction conversion
  is only INFERRED (#208), so the simulator must not assert it.
  Neither profile has an explicit exact-arity
  check. Tests match Usage only: the surrounding bad-argument wording is INFERRED.
- `CHARACTER_CREATION_RESULT` has exactly three arguments. Nonempty error text
  gives `(false, errorText, nil)`; null/empty error text gives
  `(true, nil, guidStringOrNil)`. Error text is a string key. A host delivers this
  input explicitly, and the actual Blizzard result handler updates the UI.

## Unknowns and host boundaries

- CreateCharacter return: **INFERRED zero returns on success (R78b Q2/#222)**.
  This preserves the explicit zero-return simulator policy authorized by #208;
  tests do not assert its count. This evidence-label update is a separate,
  droppable INFERRED commit. Request serialization and
  fields remain UNKNOWN; do not invent a network payload.
- Result timing, ordering relative to `CHARACTER_LIST_UPDATE`, and automatic
  roster refresh are UNKNOWN. The submit/result path must not emit that event or
  manufacture a roster entry. Host roster inputs are separate.
- The complete error key set and client name validation rules are UNKNOWN.
  #208 verifies the status-to-key table on both profiles but says the Lua return
  bridge remains UNKNOWN. **IsCharacterNameValid is out of scope for this slice**:
  retain its existing behavior on every profile with the comment
  `guessed; return bridge pending decomp (R78b)`. Do not create a validator or
  configurable validation response, and do not assert the existing guess.
  Submit tests drive the real controls from an allowed-submit UI state.
- Native initialization values and the meaning of interaction/model state are
  not inferred from the simulator's empty host scenario.

## Preservation and merge gate

For the first PR (#56), coordinator #194 required all non-Retail/non-Forever profiles to keep the old
guess and old test bodies unchanged behind explicit profile cfgs, labelled
`guessed; native evidence pending (R78 PTR part)`. PTR evidence is pending.
The old guessed-rename tests are `character_create_action_updates_player_name_without_lua_errors`
and `create_character_wrapper_seeds_state_and_admin_name`.

Coordinator #224 supplied m37 master `ab91e4f92c2fc01c5fc0c96418ee98c1e230fd4b`,
merged without rewriting history in `4ade986593b3269b731ca08775811a3938566c5c`.
The subsequent atomic change excludes the CreateCharacter guess and mixin wrapper
from Retail/Forever and cfg-gates the two old test bodies unchanged. Raw Lua
chunks are recomposed in their original order/scope for every other profile;
initial frame seeding and other workaround functions remain untouched.

## Host call observation (#210)

The last validated call is a **simulator call record; native request fields and
serialization UNKNOWN (R78 Q3)**. It is host-only, never exposed as a Lua API.
It records name and Forever surname after verified numeric-to-text conversion,
useNPE as Lua truthiness, and optional raceFaction as the original string/number
without conversion. Missing/nil faction is absent. It must not affect names,
roster, events, or result delivery. The pass-through spy independently observes
arguments from Blizzard's real Create button.

## Tests and current reachability boundary

- API tests in `tests/glue_character_select.rs` check argument Usage gates,
  numeric name conversion, Lua truthiness, zero-return interaction/model-state
  setters, and the host-only last-call observation. They do not assert the
  unknown CreateCharacter return count.
- `character_creation_host_result_handler_preservation_without_submit` delivers
  failure and both success payload variants to the real, already-loaded handler.
  It checks the actual localized popup text, handler CVar update, and all three
  event arguments. This is **handler preservation**, not end-to-end submit proof.
- After the R78b Q3 visibility prerequisite, the Retail real-button RED reaches
  customization categories whose guessed provider omits `cameraZoomLevel`.
  `Blizzard_CustomizationUI.lua:583` passes nil to the evidenced zoom setter;
  #232 confirms the categories themselves are invented and requires a native,
  DB2-backed GetAvailableCustomizations provider (R79, separate in-house lane).
  This slice must not modify that provider, add guessed camera values, weaken
  the setter, or tolerate the error. Full submit/result GREEN is blocked on R79.

## Model visibility prerequisite (R78b Q3, #222)

VERIFIED identically on Retail/Forever: Glue-only
`C_CharacterCreation.SetPlayerModelHiddenState(hiddenState)` requires one non-nil
argument, uses Lua truthiness, returns zero values, and emits no event. Missing
or nil raises `Usage: C_CharacterCreation.SetPlayerModelHiddenState(hiddenState)`.
With a native player model it sets alpha to zero when truthy and 1.0 otherwise.
Without a model it has no effect and stores no hidden flag. The simulator has
no character-create player-model object (the existing dress-state API likewise
models the no-model branch), so only argument validation/zero returns are
observable. No fake model, hidden flag, or alpha storage is introduced. Game
registration remains UNKNOWN and is not added.

#222 also offers an optional INFERRED name-validation bridge. This slice chooses
to leave IsCharacterNameValid unchanged, avoiding unrelated inferred behavior.

## PTR addendum (R78 PTR, #256/#261/#265/#268)

Native evidence targets PTR 12.1.7.70668; tests use the owner-hosted pinned
PTR 12.1.7.70493 UI. Its TOC selects Mainline through `[Family]` excluding Camelot.
The real consumer at `Mainline/Blizzard_CharacterCreate.lua:748` supplies four
arguments, including `GetSelectedSurname()`; the runtime traceback independently
confirms this selection.

VERIFIED required-input behavior (#265): name, surname and useNPE must each be
present and non-nil. Missing/nil raises the Usage substring
`Usage: C_CharacterCreation.CreateCharacter(name, surname, useNPE [, raceFaction])`.
The surrounding error wording is UNKNOWN and is not asserted. Missing/nil
raceFaction advances without error.

PTR uses its own neutral host-only call record, not the RF validated-reader
output. Name, surname and useNPE are owned observations of the supplied inputs;
no native numeric-to-text conversion or truthiness normalization is inferred.
These reader conversions are INFERRED and are not implemented or asserted here.
String/number faction values are preserved. Other non-nil faction types are
**UNKNOWN** and accepted without interpretation as an opaque marker, as explicitly
authorized by #268; there is no type-contract test. Opaque markers retain no VM
references. The record is not a native request, serialized packet, or result.

CreateCharacter retains the existing PTR simulator's no-return convention.
An ordinary native zero-result call is only **INFERRED** (#265 corrects #256);
only a zero-result tail is VERIFIED. No test asserts the public return count.
Neither result nor roster events, roster changes, or player renames are emitted.
The PTR result payload, timing and ordering are UNKNOWN, so the RF host result
adapter is not enabled on PTR.

The existing PTR CreateCharacter bootstrap availability is preserved, not taken
as native catalog evidence. OnPlayerInteraction, StopAllSpellVisualKitsOnCharacter
and SetPlayerModelHiddenState remain unregistered pending R78c. The real-control
RED asserts the exact Mainline line 287 OnPlayerInteraction nil-call and still
fails. No internal interaction byte, visual flags, hidden flag or pending-error
input is invented for PTR. IsCharacterNameValid remains unchanged per #265.

The PTR producer replacement and removal of its guessed namespace/mixin paths
are atomic with gating the two old guessed-rename tests out of PTR. Their bodies
and the guessed Lua chunks remain unchanged for the remaining legacy profiles;
initial-frame seeding remains unchanged everywhere.

### PTR R78c verified registrations (#282)

CreateCharacter and StopAllSpellVisualKitsOnCharacter are VERIFIED members of the
PTR Glue registrar. The Game aggregate does not invoke that registrar; both are
removed on the Game route and restored on Glue routes. StopAll accepts no
arguments and has no Lua-visible effect in the simulator's no-model/no-pending-error
case (#261). No pending-error input, visual flag, or model is invented. Ordinary
return count remains INFERRED and is not asserted. OnPlayerInteraction and
SetPlayerModelHiddenState remain absent at this verified-only stage; the exact
OnPlayerInteraction real-control RED is retained before the separate droppable
inferred-existence change.

### PTR R78c droppable inferred existence (#282)

OnPlayerInteraction and SetPlayerModelHiddenState exist by **INFERRED** unguarded
consumer calls; the native PTR catalog is **UNKNOWN**. Their registration is a
separate droppable commit, not a verified catalog claim. OnPlayerInteraction
VERIFIED writes one to an internal byte; the model exposes that write only as a
neutral host seam. No Lua reader or indirect gameplay effect is invented.
SetPlayerModelHiddenState VERIFIED requires a non-nil first argument, uses Lua
truthiness, and emits no event. With no owner/model it writes nothing. The
simulator has no character-create model, so it stores no hidden flag or alpha.
Its return count is UNKNOWN and is not asserted. Interaction's ordinary zero
return count remains INFERRED and is also not asserted. IsCharacterNameValid
remains unchanged.

The verified-only stage retained the exact OnPlayerInteraction RED (0 passed,
1 failed, 5.44s). After the inferred entries are applied, that boundary is passed;
the next exact first error is Mainline/Blizzard_CharacterCreate.lua:812 attempting
to call nil GetCurrentCameraZoom. No PTR camera behavior is inferred from Retail.
The real-control test remains explicitly RED with that exact-first-error assertion,
gated under optional-tests as directed in #289. Full PTR submit GREEN therefore
still awaits evidence for the newly exposed camera prerequisite.

Coordinator #292 confirms retaining the exact PTR `GetCurrentCameraZoom`
line 812 RED behind `optional-tests` for this PR. The native camera question
is filed as R83; no camera API or default is guessed in this addendum.

### PTR R83 verified zoom reader (#298)

GetCurrentCameraZoom is VERIFIED registered through Glue only. It reads no
arguments, has no Usage text, and reads stored zoom state rather than model
distance. The verified-only commit registers that route and reads the existing
shared character_create_camera_zoom storage, while deferring the Lua return
bridge to a separate INFERRED commit. This scaffold is not a claim of zero native
return values. Tests cover only Glue/Game routing and unused arguments at this
stage. SetCameraZoomLevel remains unchanged pending its argument-reader evidence.

## R79 host-seeded customization publisher (#275, #281)

This separate slice implements only publication for Retail and Forever. The
collector, selection, DB2 joins, requirement filters, eligibility/lock rules and
native source ordering remain **UNKNOWN**. No DB2 collector or sorting is added.

`SimState.character_customizations` is an optional host-owned input. `None`
leaves the existing temporary provider entirely unchanged; `Some(records)` uses
the publisher, including `Some([])` returning an empty table. The existing
SaveSeenChoices wrapper captures this provider and retains its existing semantics.
The exact optional-tests Create-button RED is unchanged. Removing the unseeded
temporary collector must wait for collector evidence.

VERIFIED publication contract (#275):

- Categories: `id`, `orderIndex`, `name`, `icon`, `selectedIcon`, `undressModel`,
  `subcategory`, `cameraZoomLevel`, `cameraDistanceOffset`, `spellShapeshiftFormID`,
  `chrModelID`, `options`, `hasNewChoices`, `needsNativeFormCategory`.
- Options: `id`, `name`, `orderIndex`, `optionType`, `choices`,
  `currentChoiceIndex`, `hasNewChoices`, `isSound`.
- Choices: `id`, `name`, `ineligibleChoice`, `isNew`, `swatchColor1`,
  `swatchColor2`, `soundKit`, `isLocked`, `lockedText`.
- Fields are published in these sequences. The publisher's literal field lists
  and loops are the insertion-order proof; tests do not assert Lua hash iteration
  order. Nested prepared lists are traversed without sorting by `orderIndex`.
- Icon ID zero publishes nil; a nonzero unresolved ID publishes an empty string.
  Resolved bytes come from the host, not an invented asset resolver.
- `cameraZoomLevel` publishes an integer host input; `cameraDistanceOffset`
  publishes an unscaled floating-point host input. No source truncation, native
  integer width, or implicit scaling is inferred.
- One table is returned, including for explicitly seeded empty input. #275
  separately labels the final native return bridge UNKNOWN.

All other field conversions and nil/omission rules remain **UNKNOWN**. Per #281,
these fields accept already-published owned host values: nil, numeric, boolean,
text, nested objects or arrays. The publisher does not coerce them, infer swatch
shape, or translate a source index for `currentChoiceIndex`. Host fixtures use
already-one-based values where appropriate. Nil fields are absent under ordinary
Lua table semantics. Unknown input field names are not added to the native field
set. Host values retain no VM references, and every table is rooted while nested
values allocate.

Tests: `tests/character_creation_publisher.rs` covers field sets, host-value
passthrough, reversed orderIndex list preservation, icon cases, zoom/distance,
seeded-empty versus unseeded behavior, and existing SaveSeenChoices semantics.
Arbitrary host-value shapes in tests are seam fixtures, not native type claims.

R79 delta #285 (Retail 12.1.5.70688, VERIFIED) applies a return policy to
finished options/choices tables, never to selection or filtering. Disabled
policy returns one result; nil/userdata and tables at policy value 1 are never
marked. Other marking depends on unmodeled frame-policy checks. Per #285 this
slice continues returning one plain table and adds no secret/marking behavior;
field order and unsorted record traversal are unchanged.

### PTR R83 droppable getter return bridge

A separate **INFERRED (R83)** commit enables the single-number Lua return and
labels the existing shared zero initialization as INFERRED for PTR. No test
asserts the initial value or return count. Tests seed nonzero host zoom values
and check only the VERIFIED stored-value identity. Existing RF initialization
and all setter behavior are unchanged; the PTR setter's reader/Usage behavior
still awaits clarification. This commit alone is not a modeled setter claim.


### PTR R83b/R83c shared setter core (#305, #318, #320)

The verified setter body is shared with Retail and Forever. Argument 1 is
required/non-nil signed int32, fractions truncate toward zero, and the stored
zoom clamps to 0..100. Its missing-input Usage and int32 range message are those
in #305. No camera-profile/model-distance effects are modeled.

R83c supersedes the interim #318 marker uncertainty: a changed zoom clears the
custom marker; an unchanged clamped zoom leaves it untouched. Explicit true for
keepCustomZoom skips the update while the marker is set. Tests cover all three
cases and the verified reader/clamp, not registration or return counts.

The existing shared body needs no semantic change. Its visibility is widened
only for the separately droppable PTR Lua registration. PTR installation is
UNKNOWN; ordinary zero returns and non-boolean second-argument truthiness are
INFERRED, not verified by these tests.


### PTR R83b droppable setter registration

The separately droppable **INFERRED** commit installs SetCameraZoomLevel only
in Glue, replacing its temporary no-op atomically. The pinned PTR Mainline
Blizzard_CharacterCreate.lua:548 calls the wrapper at :791 with zoom 0;
Blizzard_CustomizationUI.lua:664 calls it with category.cameraZoomLevel.
The native Glue catalog is UNKNOWN (#305/#320), not asserted by tests.

This bridge reuses the existing RF body: ordinary zero returns and truthiness
for non-boolean non-nil keepCustomZoom are INFERRED on PTR. Tests assert neither.
The R83b reader and R83c changed-only marker clearing remain VERIFIED. Wider
camera-profile effects are unmodeled. Getter initial zero and single numeric
return remain the earlier separate R83 inferred policy, never count/default
assertions. Unseeded category data remains unchanged pending the collector.
