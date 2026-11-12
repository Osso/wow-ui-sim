# Character creation submission

Scope: Retail and WoW Forever. Evidence is coordinator `agent-server-wow-ui-sim`
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

Coordinator #194 requires all non-Retail/non-Forever profiles to keep the old
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
