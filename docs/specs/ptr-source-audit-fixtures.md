# PTR pinned-source audit fixtures

## Goal

Reconcile the GuildBank directory fixture, QuestTimer XML fixture and the 24 failing `patch_12_1_audit` cases with the selected PTR 12.1.7 cache. Other integration failures, source manifest changes and vendor changes are excluded.

## Rules

- Use the active profile cache, not the Retail file layout or the historical 12.1.0 audit direction. Source-publication tests must assert the literal currently selected publisher and runtime surface, not equate references or comments with definitions.
- GuildBank: the unified PTR cache contains the bare TOC, bootstrap, Localization and Mainline directory. Preserve the Retail eight-entry expectation separately. Evidence: `Blizzard_GuildBankUI/Blizzard_GuildBankUI.toc:1-9`; the committed PTR manifest lists the five included files. Do not require excluded flavor directories.
- QuestTimer: the unified TOC selects family overrides, shared button template, core Lua, then family XML. PTR Mainline XML declares the container, not 25 named buttons. Its Lua acquires pooled rows. Preserve the old static-row expectation only on the source profiles that ship it. Evidence: `Blizzard_QuestTimer/Blizzard_QuestTimer.toc:8-11`, `Mainline/Blizzard_QuestTimer.xml:3-35`, `Blizzard_QuestTimer.lua:7-17,61-102`, `QuestTimerButtonTemplate.xml`.
- Symbol audits operate on source bytes. The cached `Blizzard_NewPlayerExperience/Camelot/Blizzard_TutorialData.lua` contains byte `0x92` in a comment. ASCII API names on both sides of such comments must remain searchable. Do not skip files, replace bytes or require UTF-8 for an ASCII search.
- Aura behavior tests use the pinned Lua interfaces: `ManagedAuraContainerPrivateMixin:GetAuraSources`, `AuraContainerUtil.CreateAuraGroupManager`, `RegisterAuraGroup`, `AddAura(unitToken, auraData)`, `RefreshFrameAssignments`, `GetFramesByAura` and `auraComparator`. Preserve source partition, ownership/release, filtering and configured-sort assertions. Supply all required unit/aura state; a missing target is not an aura-filter defect. Evidence: `Blizzard_AuraContainer/Blizzard_ManagedAuraContainer.lua:140-171` and `Blizzard_AuraContainerGroups.lua:32-580`.
- Utility publication assertions follow the pinned namespaces. `InputUtil.lua:1-17,207-241` and `InterfaceUtil.lua:19-27` own cursor/screen helpers. `Blizzard_UIParentUtil/UIParentUtil.lua:1-18` has a local notch helper and global `UpdateUIParentPosition`. `Blizzard_ShakeUtil.lua:1-15` also publishes legacy shake globals; these coexist with `ScriptAnimationUtil`.
- CombatAudio, FriendsList, GuildControl, Narration, SocialUI and utility method lists are independent publication slices. Assert the functions/tables actually declared by their selected Lua files. Load the owning LoD addon when required. Do not change simulator APIs just to satisfy old absence claims.
- Removed native namespace members require their pinned generated declarations and existing backing model to decide applicability. Lua reference absence alone is not native removal proof. Unknown C behavior stays unchanged and is recorded as a question for main/decomp in the lane report.
- In particular, do not invent post-world-handler retirement timing, event-policy changes or CVar wrapper identity. The old timing test remains parked until current native evidence establishes its contract.

## Acceptance

Existing master tests are RED acceptance for all 26 scoped failures. The new source-scanner fixture reproduces the non-UTF-8 input boundary with explicit symbols before and after the comment. Commit acceptance before implementing corrections. Run each changed test by exact selector with a fresh persisted CVar copy and bounded execution. Main owns the final PTR failure-set comparison (strict subset of master 74), six-profile zero-warning gate and Wrath compile; lanes supply targeted evidence, not a substitute completion claim.
