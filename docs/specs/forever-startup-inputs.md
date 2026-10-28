# Forever 70532 startup inputs

## Scope

`client_profile::ACTIVE == WowForever` publishes this additional surface, except
`C_Ping.IsPingSystemEnabled`, which Retail and PTR also publish, and
`C_PaperDollInfo.AmmoNeeded`, which PTR also publishes. Other profiles retain
their existing registrations and behavior. Forever contracts come from
the copied 1.60.1.70532 `Blizzard_APIDocumentationGenerated` cache. Retail/PTR
`PingManagerDocumentation.lua:44` declares the same non-null boolean result.

## Supplied state, not native policy

`SimState.forever_startup` contains host inputs for ammo requirement, cursor
visibility, ping enablement, bank-type purchase permission, match completion,
target spell, TTS settings-save state, temporary CVars, external-event URL state,
and group-buff visual alerts. Empty-world flags and collections are simulator
fixtures, not claims about native defaults. Tests must vary inputs through the
original Blizzard consumers. Native derivation and validation policies remain
unknown.

- `C_PaperDollInfo.AmmoNeeded()` reads the single environment-local
  `forever_startup.ammo_required: Option<bool>` on PTR and Forever. Explicit
  `Some(false)`/`Some(true)` return exactly one boolean. `None` raises
  `C_PaperDollInfo.AmmoNeeded: ammo requirement has not been supplied`; this is
  simulator missing-input policy, not a native error claim. Both pinned builds
  declare the no-argument non-null bool at
  `Blizzard_APIDocumentationGenerated/PaperDollInfoDocumentation.lua:11–18`
  (PTR12.1.7.70493, Forever1.60.1.70532). PaperDoll hides AmmoSlot only when
  false; true continues initialization but does not Show a previously hidden
  slot. The Forever stock scenario starts with `Some(false)` as a best guess;
  PTR starts with `Some(false)` as a user-decided scenario default. Neither is
  native proof. No equipment/class/weapon mapping,
  `UnitUsesAmmo` equivalence or automatic event is inferred.
- Group-buff rows contain non-null numeric `spellID` and `visualValue` fields;
  `visualValue` uses the documented `VisualAlertType` enum. Linked-rank expansion
  requires host-supplied rows; no spell-rank policy is inferred.
- External-event probes return non-null booleans from the supplied optional URL,
  availability and newness. No network launch is simulated without a modeled
  launch result.
- Bank purchase/max-tab flags and optional `BankLockedReason` values are supplied
  per documented `BankType`; absence of a lock row returns one nil. Housing
  presence reads the existing spatial `HousingArea`, independently of ownership.
  Optional per-bank-type `PurchasableBankTabData` rows supply `tabCost`,
  `canAfford`, `purchasePromptTitle`, `purchasePromptBody` and
  `purchasePromptConfirmation`; absent rows return one nil. The original
  Camelot first-slot consumer is covered for absent and paid offers. Free-offer
  purchase actions and native pricing/affordability policy are not modeled.
- Discord enablement reads the existing `discordClientEnabled` CVar. Ping
  availability is the same supplied `forever_startup.ping_enabled` flag on
  Retail, PTR and Forever. Its derived false default describes the empty
  simulator world, not native policy. True/false transitions must project
  through the original raid manager. The original OnLoad initializes its ping
  menu when true and does not remove that menu on a later false input.
  `RestrictPingsTo` remains a Forever host input; Retail's feature-gated
  restriction model is unchanged. TTS bool settings are supplied as
  enabled `TtsBoolSetting` values. Save marking sets a local saved flag, and its
  getter reads that same flag; no persistence policy is inferred.
- `C_CVar.RemoveTempCVar(name)` removes the host's lower-case temporary scalar
  entry without writing saved CVars. `GetCVar`/`GetCVarBool` and their namespace
  counterparts see the temporary scalar before the saved/base value. This does
  not establish native mutation events, validation or other temporary-CVar APIs.
- `C_Spell.GetTargetSpellID()` reads a supplied numeric identifier. If no
  target is supplied, the simulator returns nil as a best-guess scenario
  default, not a native observation. This deviates from the declaration's
  non-nilable return; native no-target behavior is unknown. See the
  [target-spell contract](spell-target-flyout.md).

## Events

Accept the documented `SECURE_TRANSFER_CONFIRM_TAKE_MAIL_MONEY`,
`SHARD_TRANSFER_CANCELLED`, `EXTERNAL_EVENT_LAUNCH_URL_RESULT` and
`SOCIAL_UI_SOCIAL_QUEUE_SYSTEM_STATUS_UPDATED` names for Forever only. Also accept
`EXTERNAL_EVENT_URL_CHANGED` and the subsequently exposed
`SOCIAL_UI_RAID_LIST_SYSTEM_STATUS_UPDATED`. URL launch result has one non-null
boolean payload; the other five have no payload. This
adds registration support, not native event-generation policy.

## Unknown residue

`C_AccountServices.GetPreferredDatacenterLocalities` is consumed by the selected
GuildPreferredPlaySettings source but has no declaration in this build's generated
API documentation. Its native contract remains unconfirmed. The stock scenario
returns two empty parallel arrays as a best guess, not a native observation.
No locality catalog is inferred.

The laptop aso source-only audit on 2026-10-25 classified locality returns and
idle target-spell return as native-unknown.

## Excluded housing UI

70532 `Blizzard_HousingControls.toc` declares `## AllowLoadGameType: standard`.
Forever's camelot profile therefore excludes its bootstrap, the sole publisher
of `HousingControls_LoadUI`. The original GameEvent handler's true housing branch
is blocked on that missing Lua publisher. Tests must retain both boolean
projection branches and assert the exact excluded-publisher failure/unloaded
state; they must not bypass the TOC or fabricate a loader. This test expectation
was approved on 2026-10-25 after the old load-success assertion was disproved.

## Acceptance

Original selected consumers must reject missing/constant producers and project
varying host inputs. Stock startup captures must preserve exact message bytes,
classify every residual error, and accept a baseline only for documented
native-unknown residue. Record profile checks and CPU evidence in the lane report.
