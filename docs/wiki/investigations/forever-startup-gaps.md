# Forever 70532 startup gaps

Verified 2026-10-25. The locally authenticated 1.60.1.70532 UI reaches simulator
registration gaps. Fixes publish documented shapes from explicit host inputs,
not guessed native character/account policy.

## Reproduction and selection

The lane's copied runtime cache retains all source bytes. Stock
`wow-sim --no-addons --no-saved-vars lua-errors` reproduced 76 raw messages:
38 underlying errors and 38 Blizzard_ScriptErrors reports. PaperDoll's
`[Game]` entry is allowed for camelot; the excluded Mainline alternative is not
its consumer. `PaperDollItemSlotButton_OnLoad` uses `AmmoNeeded() == false` to
hide AmmoSlot. Returning nil would not implement this condition.

The runtime's API documentation confirms all initial symbols/events except
`C_AccountServices.GetPreferredDatacenterLocalities`. The selected
GuildPreferredPlaySettings source calls that undeclared method, but that alone
does not establish its native signature or catalog.

## Supplied input boundary

The [contract](../../specs/forever-startup-inputs.md) distinguishes fixture state
from native policy. Publication checks `client_profile::ACTIVE`; registration
on other profiles is untouched. Tests use selected cached Blizzard functions
and vary host state. Ammo requirement is an explicit character input, not an
inferred class/item rule. Group-buff rows are supplied already expanded; no
native rank-expansion rule is claimed.

Eight original-consumer selectors reached GREEN at `4429dde8b`: ammo visibility,
event registration, group-buff rows, external URL state, cursor visibility,
bank purchase permissions, PvP completion and target spell. Their exact RED and
GREEN listings/executions are in the lane report. Subsequent source changes can
invalidate intersecting proof.

## Shared PTR/Forever ammo observation

Verified 2026-10-26.

The [input contract](../../specs/forever-startup-inputs.md#supplied-state-not-native-policy)
requires an optional supplied boolean, not the former Forever default false.
Only PTR and Forever publish AmmoNeeded. PTR's selected TOC entries52/59 load
Mainline PaperDoll; Forever's53/60 load Camelot and exclude Mainline. Their
original checks are Mainline1702 and Camelot1952. Forever's actual AmmoSlot XML
binds `PaperDollAmmoSlotButton_OnLoad`, which delegates to the item-slot consumer;
PTR binds an inline XML wrapper. Tests must invoke those real slot scripts,
explicitly Show before each OnLoad replay, and observe hiding versus inventory
initialization, isolation, and named unset/recovery behavior.

No native ammo derivation or default is established. Unsupplied PTR startup
replaces nil-call keys with named errors. Unsupplied Forever startup can gain
named errors because the removed false previously masked missing observations;
earlier baseline counts below are historical, not acceptance of this change.
The current paired stock packet is PTR122 keys/138 occurrences before and after,
with40 nil-call raw keys replaced by40 named missing-input keys. Forever changes
from26/26 to44/44:40 named ammo errors are added and22 downstream target-spell
reports disappear because earlier slot/BagSlotCluster initialization stops. Those
removed reports are masked, not repaired. Each raw addition/removal is attributed
in the private ammo-needed handoff; neither startup is clean.

Both original XML-bound tests pass false/true/false, two-environment isolation,
exact unset failure and recovery. PTR proves Mainline1702; Forever proves both
Camelot1952 and the XML delegate3654. Identical predecessor tests fail at PTR's
missing member and Forever's silent unset success respectively. Unfiltered PTR
closure diagnostics remain visible; this proves the slot boundary, not complete
closure initialization. Both existing Forever inventory-slot preservation tests
also pass. All seven full-feature all-target checks pass with zero Cargo JSON/plain
compiler warnings. All 23 launch CVar guards and 10 pinned-input hashes are unchanged;
formatting and manual changed-line readability review pass.

The loader propagates the failed BagSlotCluster child creation and stops later
XML elements (`xml_frame/setup.rs`, `xml_frame/finalize.rs`, `xml_file.rs`). This
explains the lost bag event registrations and later GamepadBackpackButton reports;
the target-spell API itself was not changed.

The unset probe follows the existing `mists_scenario_minlevel` precedent:
`env.exec` alone does not retain a native-error traceback. An asserted-failure
`xpcall` captures status, exact error message and `debug.traceback` separately.
The test rejects successful calls and non-exact messages; Forever additionally
requires both original1952 and XML-delegate3654 frames. No production error
handling or vendor callback is changed.

## Historical startup packet and unknown consumers

`GetTargetSpellID` documents a non-null number but no absent-target sentinel.
At this historical boundary, absent supplied input raised a native-unknown error.
The later [target-spell contract](../../specs/spell-target-flyout.md) replaces that
error with a nil scenario default; native no-target behavior remains unknown.
Since 2026-11-07 the Forever stock idle scenario supplies `0` (researched best
guess for the unknown sentinel, matching the non-nilable declaration) while an
explicitly unset input still returns nil. Repairing earlier
PaperDoll/BagSlot errors lets more bag consumers reach this query; do not hide
these reports by returning an invented zero. Native facts were requested from
the laptop. Locality declaration/behavior is also pending that evidence.

The first repaired stock capture had 40 raw messages. It exposed
`C_Bank.FetchBankLockedReason` (nullable documented enum) and
`SOCIAL_UI_RAID_LIST_SYSTEM_STATUS_UPDATED` (documented payload-free event).
The completed supplied models include Discord, ping/restriction, spatial
housing, TTS save/bool settings, temporary scalar CVars and optional bank offers.
All 21 exact selectors passed at `1fcbddf67`. That packet's [baseline and per-record
classification](../../baselines/README.md#forever-lua-errors) retained only two
native-unknown boundaries; the shared optional ammo observation adds another
explicit missing-input boundary. Startup is not clean; no native idle value
or locality catalog was invented. Profile checks and release evidence belong to
the lane report.

## Forever 1.60.1.70707 residuals

Verified 2026-11-07 from the `ptr-toc-pins` Forever `toc` run (457 passed, 3
failed). Each failure is a missing publication, not a consumer bug.

| Symptom | Root cause | Fix |
|---|---|---|
| `LoadAddOn("Blizzard_AuctionHouseUI")` aborts at `Camelot/Blizzard_AuctionData.lua:128` | `Enum.ItemContainerSubclass` absent; `Enum.ItemQuiverSubclass` would fail next at :133 | Publish both enums and their `*Meta` bounds from `ItemConstantsDocumentation.lua:325-346` and `:493-506` |
| `forever_stock_target_spell_reaches_original_item_context` panics with `assertion failed!` | Stock `target_spell_id` unset, so `C_Spell.GetTargetSpellID()` returned nil against a non-nilable declaration | Seed the Forever stock idle scenario with `0` |
| `quest_readiness` QL2 autocomplete swallows `attempt to call field 'CanAbandonQuest' (a nil value)` | `C_QuestLog.CanAbandonQuest` registered for Retail only, but Forever declares it at `QuestLogDocumentation.lua:45-58` and `Mainline/QuestMapFrame.lua:1147` calls it | Extend the registration gate to Forever |

All three remain Forever-only. Publication gates are `client_profile::ACTIVE`
or the existing `client-wowforever` feature; Retail and PTR surfaces are
unchanged. Contract details live in
[forever finite constants](../../specs/forever-finite-constants.md),
[target spell and flyout membership](../../specs/spell-target-flyout.md) and
[quest abandonability](../../specs/quest-abandonability.md).

## Forever 70707 integration fixture differences

VERIFIED (2026-11-12): the `forever-touched-15` exact replay reproduced all
15 failures from base `54acfb8e0`. Most were Retail fixture assumptions, not
simulator runtime defects:

| Consumer | Selected Forever evidence | Required fixture |
|---|---|---|
| Modified clicks | `Blizzard_FrameXML/Bindings_Camelot.xml:1533` supplies SELFCAST ALT; `src/loader/addon.rs::load_modified_click_defaults` installs it when FrameXML loads | Load the FrameXML closure, not a bare environment |
| SharedTalentUI dependency | `Blizzard_SharedTalentUI.toc:3` lists only SpellSearch | Do not use PTR's additional SpellTypeIconOverlay edge |
| Tracking bars | `Camelot/StatusTrackingBarConstants.lua:21-28` gives Exp=0, Rep=2, Honor=3, HouseFavor=5; shared manager `:61` sorts descending | Cold Rep+Exp; battlefield Honor+Rep; tracked house HouseFavor+Rep |
| Endcaps | `Camelot/EditModePresetLayoutConstants.lua:11` disables parent management; `EditModeSystemTemplates.lua:3786-3802` applies each cap's Hidden setting | Use per-cap Hidden settings and `Camelot/MainMenuBarEndCaps.lua:16-22` Edit Mode visibility override; atlas is on `.Texture` |
| Flyout | `Shared/SpellFlyout.lua:214-215,338-339` creates unnamed pooled buttons | Observe `SpellFlyout.buttons`, not numbered globals |
| Spellbook art | `Camelot/SpellBook/Blizzard_SpellBookTemplates.xml:37,43,49` selects C60 page atlases | Resolve those atlas files and compare emitted texture requests, including renderer crop suffixes; retain the draw-order assertion |
| Specialization | `Camelot/Blizzard_PlayerSpellsFrame.lua:58-63` disables the top tab system | Use the separate TalentMicroButton and public specialization helper, with the same Escape assertions; explicitly supply empty outfits/unavailable transmog |

VERIFIED: the selected `Bindings_Camelot.xml` has no TOGGLEACHIEVEMENT
binding; the Retail test references `Bindings_Standard.xml:1251`. The Forever
gate has coordinator approval and makes no claim about the A key's behavior.
VERIFIED: the partial panel closure also needed eager QueueStatusFrame because
`EditModeSystemTemplates.lua:2467` reaches QueueStatusButton during transitions.

VERIFIED: the runtime defect was the Forever publication gate for the existing
[supplied item-overlay classifications](../../specs/retail-corrupted-azerite-items.md).
`ItemDocumentation.lua:1232` declares IsCorruptedItem and the selected
`ItemButtonTemplate.lua:271` calls it. The original consumer test distinguishes
marked and unmarked item IDs; no native classification default is inferred.

VERIFIED: the dedicated 211-entry eager-order fixture was independently generated
from the pin's selected Game TOCs, with file hashes and header citations in the
private lane report. Its required/manager dependency graph has no cycles or
reversed edges. A test validates those edges separately from snapshot equality.
INFERRED: implicit startup publisher edges remain simulator policy; native total
ordering of unrelated addons is UNKNOWN.

## Excluded and unmodeled branches

The user approved correcting the positive housing expectation after proving
`Blizzard_HousingControls.toc` declares `## AllowLoadGameType: standard`. That
excludes camelot and the sole `HousingControls_LoadUI` bootstrap publisher. The
test retains both boolean projection branches and asserts the exact missing
publisher error and unloaded state; no loading or vendor changes are made.

Optional bank offers are covered as absent and supplied paid five-field rows.
Native pricing, affordability and free-offer purchase actions are not modeled.
External URL network launch, native buff-rank expansion and TTS persistence also
remain outside this startup/shape boundary. Laptop evidence disposition for
both retained baseline symbols: **laptop aso source-only audit 2026-10-25:
native-unknown**.

## Sources

- [Startup input contract](../../specs/forever-startup-inputs.md).
- [Profile selection](../systems/client-profiles.md).
- [XML loading pipeline](../../addon-loading-pipeline.md) and [loader error propagation](../../../src/loader/xml_file.rs).
- [Consumer-selection proof rules](consumer-selection-and-red-proof.md).
- Private Forever startup slot-run report and capture receipts.

## See Also

- [[c-factory-startup-followup]] — historical frozen startup member accounting.
- [[casc-asset-cache]] — byte-preserving source-cache provenance.
