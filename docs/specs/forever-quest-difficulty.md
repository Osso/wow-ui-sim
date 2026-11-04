# Quest difficulty query

Retail, PTR and Forever's `C_QuestLog.GetQuestDifficultyLevel` must expose the supplied rich quest-log row's `level`. Implementation belongs to `src/c_api/c_quest_log.rs`; [quest loading context](../addon-loading-pipeline.md). The named **UNKNOWN/EMULATED quest-difficulty-level** seam retains the existing equality and fail-fast missing-input policies; native tuning, unknown-ID, error and secrecy behavior are not established.

## Declared-profile publication — 2026-11-04

Pinned Retail, PTR and Forever `Blizzard_APIDocumentationGenerated/QuestLogDocumentation.lua:380–392` declare `questID -> level`. The four Classic profiles do not declare this endpoint in their pinned generated documentation. Publication now follows that declared set and reuses the existing model without changing Forever behavior. PTR's original `Blizzard_FrameXMLUtil/Mainline/DifficultyUtil.lua:84–88` requires it during startup. WM89 difficulty-band evidence does not prove that the stored quest level equals a native tuning-adjusted difficulty level.

## Native trivial ranges

WM89 found that Forever 1.60.1.70707 prepares 5 for both unit trivial-range APIs. `C_QuestLog.GetTrivialRange` prepares 0 without a player and evaluates GlobalCurve selection keys 47 and 0 with a player; an absent configuration row selects 5. The simulator leaves these APIs unchanged and has no configured curve input.

For `C_PlayerInfo.GetContentDifficultyQuestForPlayer`, WM89 establishes Impossible at difference ≥5, Difficult at 3–4, and Fair at −2 through 2. Below −2, Forever uses Easy through the inclusive negative curve range and Trivial below it. Populated curve values are unknown. The simulator emulates an absent configuration row with range 5; this is not a normal-client guarantee. Retail/PTR use fixed Easy −4 through −3 and Trivial at −5 or lower. Both paths currently use stored quest and player levels as emulated substitutes for ContentTuning and effective-level inputs.

## What it must do

- [ ] Publish a raw callable `GetQuestDifficultyLevel` through the modeled registration on Retail, PTR and Forever; preserve absence on the four Classic profiles.
- [x] Return exactly one number from the exact current environment's `quest_log_entries.entries` row matching `quest_id`; observe non-80 live mutations and never consult a static catalog or fabricate a level.
- [x] Fail explicitly with API/questID context for nonnumeric, nonfinite, fractional, out-of-i32-range or unrepresented identities and for removed rows. Missing valid integer identities include that ID in the error.
- [x] Support unchanged whole cached Mainline DifficultyUtil plus Camelot QuestUtilsOverrides with explicit test fixtures for elite status, CVar selection and color dependencies; produce concrete ordinary/elite, colored/plain title prefixes.
- [ ] Preserve other profiles' existing surface and Retail/PTR `QuestCanHaveWarModeBonus` behavior in runtime verification.

At source `9c2eea67f`, the shared live-row/isolation/error controls pass2/2 on
both PTR and Retail. PTR's full `edit_mode` filter passes46/46, including the
original subprocess startup consumer that failed before publication. Retail's
filter passes56/56. Classic absence is source-gated but has not been runtime
verified in this lane; the cross-profile requirement above remains open.

## Ordinary-watch membership follow-up — bounded model execution

The Forever membership implementation replaces the all-ordinary-rows watch policy with explicit per-environment `quest_watch_types: HashMap<i32, QuestWatchType>` metadata. The contracts below are bounded simulator policies; current model execution is recorded below, not native parity or normal-startup acceptance.

- **Membership and seed:** seed current non-world rich-row IDs once as `Manual=1`; `Automatic=0` is representable metadata, not selected by `AddQuestWatch`. Later row insertion or identity changes do not auto-enroll IDs. Counts and enumeration intersect metadata keys with current non-world rich rows in rich-row order, not insertion or hash order. Removed/world rows are hidden without deleting their metadata; restoring an eligible row can expose retained membership. Fresh state gets its own seed/map.
- **Real mutators and type:** `C_QuestLog.AddQuestWatch(questID)` inserts Manual only for a currently represented ordinary ID without existing metadata; duplicate, world-only and absent IDs return one `false`. `RemoveQuestWatch` deletes existing metadata even when the rich row is absent/world, returning one boolean indicating deletion. `GetQuestWatchType` returns exactly one numeric type only for a currently ordinary row with metadata, otherwise one nil. These three endpoints reject nonnumeric, nonfinite, fractional and out-of-i32-range IDs with API context; extra arguments are ignored. Add does not change an existing type.
- **Count and index:** `C_QuestLog.GetNumQuestWatches` and global `GetNumQuestWatches` return the same intersection count. `GetQuestIDForQuestWatchIndex` returns one ID for a valid one-based watch position, otherwise zero results. Global `IsQuestWatched` accepts a log index and returns one boolean. `GetQuestIndexForWatch` maps watch position to rich-row offset + 2; `GetQuestWatchIndex` and `GetQuestSortIndex` map that log index back to one-based watch position. These numeric mapping globals return one zero on misses. Legacy index conversion remains missing/nil → zero, numeric truncation/saturation then bounds checking, nonnumber error, extra arguments ignored. Row removal, reorder and world flags immediately change the visible intersection and mappings; no static catalog fallback or fabricated rich row.
- **Synchronous event policy:** a successful metadata insertion/deletion commits state, releases its mutable borrow, then dispatches `QUEST_WATCH_LIST_CHANGED(questID, added)` immediately, before the mutator returns. No-op mutations emit no event. Handlers can observe changed state and reenter mutators; nested changes dispatch synchronously. Dispatch errors propagate after mutation, without rollback. Direct host row/map edits do not use this notification path. This is bounded simulator policy, not established native event timing or payload parity.

### Evidence boundary

Prior runtime source40a has **28 PASS** (difficulty3 + watch-projection4 + finite21), plus the separately recorded normal watch/global-alias/voice checks; [prior receipt](../baselines/README.md#current-watchtalent-execution--2026-10-15) owns attribution. Those results cover the prior read-only projection, not this new explicit-membership implementation or revised tests. Source704's original normal Lua RED included static-only80002; source40a removed that stale producer. A separate source40a normal RED still observed zero-result add/remove, unchanged count2, nil type and zero events. Preserve both historical boundaries; no authored Rust RED is inferred.

Current source `f91eb7aab` has **18 quest model PASS**: membership8 + revised projection4 + getter3 + whole cached header-consumer3. The [current central receipt](../baselines/README.md#current-watch-membership-model-execution--2026-10-15) owns build/seals, the separate finite21 controls and independent saved model-runtime PASS. Whole-header tests use explicit fixtures and replace the unrelated refresh tail; they are not normal-process header execution. Actual normal original membership and actual `QuestMapFrame` header probes now show bounded GREEN markers, exit0 and no execLua error; the central receipt owns observations and retained stream-only checker correction. Independent saved normal-runtime reconciliation PASS; no commands were rerun. Fresh startup **FAIL:10 records /14 occurrences, exit1** is byte-identical to the committed baseline; no baseline JSON update. Other-profile, native, full UI lifecycle and full-parent acceptance remain open.

## How it works

- [C API boundary](../../AGENTS.md#c-api-boundary)
- [Lua API system](../wiki/systems/lua-api.md)

## Implementation inventory

- `src/c_api/c_quest_log.rs`: shared declared-profile supplied-state difficulty lookup; Forever elite and existing Retail/PTR eligibility queries remain separate.
- `src/c_api/mod.rs`: narrowly extends the module and utility call gate to Forever.
- `src/lua_api/globals/quest_surface/{register,handlers}.rs`: omit replaced static watch handlers only on Forever, preserving initial modeled publication.
- `tests/wowforever_quest_difficulty.rs`: shared declared-profile live-row/isolation/error assertions and Forever-only whole-consumer assertions.
- `tests/edit_mode_cache_startup.rs`: unchanged PTR/Retail process startup reaches the original DifficultyUtil title consumer; truncated account-cache warning remains observable.
- `src/c_api/c_quest_log_watches.rs`: Forever membership mutators, type/count/index queries, legacy global mappings and synchronous notifications; seeded map stored in `SimState`.
- `tests/wowforever_quest_watch_projection.rs`: revised live-row/explicit-membership projection assertions, four model cases PASS at source `f91eb7aab`.
- `tests/wowforever_quest_watch_membership.rs` and `.lua`: membership, restore-call preservation and reentrant-event coverage, eight model cases PASS at source `f91eb7aab`; `tests/fixtures/quest_watch_membership_original.lua` retains the original probe.
- `tests/wowforever_quest_header_consumer.rs` and `.lua`: three model cases PASS at source `f91eb7aab`; load whole cached QuestUtils and QuestMapFrame unchanged, exercise header-one track/untrack, world interleaving and the real NPE removal gate. An observer replaces only the unrelated UI-refresh tail; actual refresh/menu/frame-event lifecycle is not covered. Rich rows support one synthetic header, so second-header behavior is not fabricated. Actual normal header execution now has bounded GREEN evidence in the central receipt; full refresh/UI parity remains unverified.

## Tests asserting this spec

`tests/wowforever_quest_difficulty.rs`: all three PASS at runtime source `70459ec5a`; [central receipt](../baselines/README.md#current-providerquest-execution--2026-10-15) owns build/seals, normal probes and failing startup. Original sealed Lua REDs precede this producer; no authored Rust RED is claimed.

- `forever_quest_difficulty_raw_publication_reads_seed_and_live_state`: all four seed IDs, exact one-result raw endpoint, level37 mutation, new identity456789/level23 and independent fresh seed.
- `forever_quest_difficulty_rejects_invalid_and_unrepresented_ids_explicitly`: invalid input shapes, absent IDs, removed row, useful errors; 80002 is deliberately absent from rich state despite the older static quest catalog.
- `forever_quest_difficulty_whole_vendor_title_consumer_uses_live_level`: exact `[37] ` / `[37+] ` prefixes, `|cff336699` fixture wrapping, CVar disabled plain output.

## Known gaps (current cycle)

- [x] Independently reconcile all 39 distinct saved model passes at source `f91eb7aab`; discovery and filtered cases are not passes.
- [x] Independently reconcile normal original/header probes with bounded GREEN observations; full refresh/UI/native acceptance remains open.
- [ ] Verify POI toggle, tracker and actual refresh/menu/frame-event lifecycle beyond bounded consumer cases.
- [x] Independently accept [default-profile compiler preservation](../baselines/README.md#default-profile-compiler-preservation--2026-10-15) at source `70112e418`; production metadata check only, not `cfg(test)` compilation or runtime.
- [ ] Verify other-profile preservation; world-watch projection and broader query reconciliation remain open.
- [ ] Native parity and whole-startup acceptance remain open, including the fresh Gamepad startup FAIL. Prior source40a28 PASS and startup FAIL remain separate from current bounded membership proof.

## Out of scope

Native unknown-ID/error/secrecy parity; defining a separate difficulty-level model; changing vendor code; new globals, missing-surface shims, retail epochs or build configuration. Forever follows ordinary numeric validation; the Retail/PTR-only secret-unwrapping branch is unchanged, not promoted into a new Forever security policy.
