# Forever spell item-cooldown publication

## Evidence and scope

**VERIFIED declaration shape:** owner-provided Forever70707 `SpellDocumentation.lua:122–136` and PTR70493 `SpellDocumentation.lua:139–153` are byte-identical for `C_Spell.GetItemCooldown`: required numeric `itemID`, `MayReturnNothing`, and `SpellCooldownInfo` return. Both `SpellSharedDocumentation.lua:19–31` blocks match, including required `startTime`, `duration`, `isEnabled`, `isActive`, `modRate`, and optional `activeCategory`, `timeUntilEndOfStartRecovery`, `isOnGCD`. Retail's pinned SpellDocumentation does not declare this API; Retail remains absent.

**INFERRED behavior reuse:** an explicit `client_profile::ACTIVE` gate publishes the existing PTR state-backed implementation for Forever. No producer arithmetic, return construction, secret access, events, or eligibility policy changes. Existing `item_cooldowns` rows supply start/duration; absent rows return nil. Enabled=true, modRate=1 and active-from-nonzero-pair remain the existing bounded simulator policy, not newly verified native behavior. Optional fields stay absent. Native eligibility and secret/restriction rules remain unproved. See [the PTR bounded contract](ptr-i4-first-green.md).

## Original consumer proof

Forever Collections is load-on-demand. Its Game TOC selects `Shared/Blizzard_CollectionTemplates.lua:37` via line17 and `Mainline/Blizzard_ToyBox.lua` via `[Family]` at line34 (line33 is the Camelot progress-tracker XML). The original ToyBox callsite at403 invokes `CollectionsSpellButton_UpdateCooldown`. Forever's active game types admit mainline, its Game is Camelot and Family is Mainline.

`forever_spell_item_cooldown::forever_original_collections_cooldown_observes_two_records_and_absence_without_errors` is optional-only. It checks selected TOC files, loaded closure, original file hashes and function source identity, then directly exercises that original shared function using real Button/Cooldown frames. Two distinct supplied item records6948=(11,31) and140192=(17,43) must produce matching millisecond cooldown times; absent880001 must hide. Each constant/no-op producer fails the selector independently. No API or vendor function is replaced.

The full LoD closure may contain unrelated baseline errors. The test preserves its error messages, records and occurrence counts, and asserts the exercised original cooldown path adds none; successful `env.exec` also excludes uncaught path errors. It does not claim all Collections features are error-free.

The RED against base5d815a7e8 fails exactly at CollectionTemplates.lua:37 because GetItemCooldown is nil. GREEN uses the identical selector. The existing PTR record test retains all fixture values and assertions, with its applicability guard extended to declared Forever and its long body split. Retail absence and other-profile boundaries remain intact.

## Verification

Exact compiler-artifact hashes bind each profile's list/test/startup invocations before and after runtime, immediately following that profile build. Full transcripts, hashes, five zero-warning compile receipts, PTR preservation, and Retail/Forever startup counts are recorded on lt-f3ef, not committed as evidence files.
