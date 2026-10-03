# Verification: 92e4ea045

Read-only artifact verification; no Cargo or test execution. Caller reports 8/8 quest and 2/2 equipset passing; not independently rerun. Review uses commit blobs, not changing working tree.

## Preliminary evidence
Quest query authenticates declared and extra arguments before validation; nil lookup reads explicit host context/map. Eight tests exercise concrete state and security. Equipment tests exercise actual inventory and event transitions, including recovery and live rename/delete. Cached caller and refactor comparison pending.

## Caller compatibility finding
Quest favor miss errors are exposed to cached Blizzard reward helpers: Blizzard_FrameXMLUtil/Mainline/QuestUtils.lua:778 unconditionally queries favor; Blizzard_GameTooltip/Mainline/GameTooltip.lua:198 queries it in the reward-presence OR chain. Default map/context is empty and only tests seed it. Thus reaching the former helper without host records raises, rather than allowing its numeric >0 branch. No cached-helper integration test covers this boundary. Native miss policy is not established; this is a concrete simulator compatibility gap, not proof of a newly introduced native regression.

## Refactor finding
Parent/current comparison shows UseEquipmentSet preserves i32 argument conversion, apply transition, event ordering/payloads/error propagation, boolean return and one-result arity. Both new command tests require the dispatch arm: inventory assertions at tests/equipment_set_command.rs:62/64 and event-count assertion at :100 fail under the prior unknown-command no-op.

## Final report

Scope: commit `92e4ea045224cd41e1d5deff92afc5488932acb3`, inspected with `git show` and commit-qualified `git grep`; cached retail Lua inspected directly. No repo edits, git mutation, builds, test execution, agents or model CLIs. Reported 8/8 and 2/2 passes belong to caller; independent proof here is source/test inspection.

### 1. Quest favor — REJECT for closing audit row 293

Implementation is substantive, state-backed, not a wrapped constant: `src/c_api/c_quest_info_system.rs:15–24,81–98` stores independent quest records and reads raw/capped amounts. Nil selects explicit `context_quest_id`; changing context/records changes results. Registration is reachable at `src/lua_api/globals/register.rs:227`; empty default/state field at `src/lua_api/state.rs:345` and `src/lua_api/state/sim_state.rs:375`.

**Compatibility defect:** `src/c_api/c_quest_info_system.rs:84–93` errors on absent context/record. Default state has neither, and commit-qualified search finds no production seeding. Cached `Blizzard_FrameXMLUtil/Mainline/QuestUtils.lua:778` calls this API unconditionally inside `QuestUtils_AddQuestRewardsToTooltip`; `Blizzard_GameTooltip/Mainline/GameTooltip.lua:198` also calls it when preceding reward-presence alternatives are false. Therefore these real reward-tooltip paths raise when they reach favor lookup without host fixtures. The tests intentionally assert these errors (`tests/quest_reward_favor.rs:81–93`) but never exercise the cached caller boundary. This is a concrete compatibility gap; prior implementation/native miss behavior is not proven, so not labeled a demonstrated new regression. Fix requires a supported state/miss contract, not an invented placeholder zero.

Security ordering is correct by inspection: `src/c_api/c_quest_info_system.rs:46–52` unwraps both declared arguments and all extras before validation or lookup. Cached `Blizzard_APIDocumentationGenerated/QuestInfoSystemDocumentation.lua:44,48–54` declares AllowedWhenUntainted, nullable quest/flag and nonnil numeric output. Tests use authentic host-created secrets, tainted callers, wrong public IDs, missing records, extra secrets, recovery and GC (`tests/quest_reward_favor.rs:60–77,192–248`). No blanket secret rejection or declassification substitute.

**Earned bounded-model checkboxes:** `docs/specs/quest-reward-favor.md:9–13,17–21`. Behavioral evidence: defaults/isolation :81–94,179–189; context/default flag :97–113; exact values/replacement/removal :116–158; representation validation :161–175; authentic-secret/public caller/security precedence :192–248. These assertions observe returned values/arity, errors, caller taint and state, not source shape. Native claims remain unearned: spec :40 explicitly says “Do not close audit row 293 based solely on inferred simulator tests or registration.” Nullable selection, cap policy, strict numeric domain, public outputs and miss semantics are inferred rather than native-verified.

Documentation defect: spec :31 still says integration pending although wired. Checkbox :39 is only partly established here: integration exists; formatter/build/startup verification is outside this read-only proof. No cached-UI compatibility test.

### 2. /equipset — ACCEPT WITH QUALIFICATIONS

Both prose rows (`data/patch-api/sources/12.0.5-api-changes.txt:121,178`) say unknown/invalid set names no longer cause Lua errors. The implementation performs real catalog lookup (`src/c_api/equipment_set_command.rs:10–21`) and dispatches from `C_Macro.RunMacroText` (`src/lua_api/globals/spell_macro_verbs.rs:176–177,319`). Unknown names return without swap mutation/events; known names use the existing equipment transition. Empty/whitespace arguments are already discarded by parser :183–189, not by the new arm.

Parent/current comparison establishes exact preservation of `C_EquipmentSet.UseEquipmentSet` at `src/c_api/item_spell/c_equipment_set.rs:275–293`: same i32 conversion, apply operation, pending/finished/changed ordering and payloads, propagated errors, boolean result and one-value arity. Only extraction changed; no observed refactor regression.

Both new tests would fail without the dispatch arm: `tests/equipment_set_command.rs:62,64` require changed inventory, and :100 requires three swap events. Invalid no-op assertions alone would pass the old dispatcher, but valid-control/recovery assertions prevent false credit. Fixtures seed concrete catalog/inventory; tests assert ignored-slot preservation, last-used state, live rename/delete and ordered notifications.

**Earned checkboxes:** `docs/specs/equipment-set-command.md:7–10`, supported by `tests/equipment_set_command.rs:48–81,89–119`. Checkbox :11 partially earned: exact casing and names containing spaces are tested; trimming a valid name and quoted/conditional syntax are not explicitly tested. No dedicated new direct-UseEquipmentSet return/error regression test; preservation proof is the source comparison, not execution.

Documentation defects: spec :21 and :27 still describe pending integration/no passing-test claim; :31 combines integration and execution evidence and cannot be wholly independently credited here. These stale statements do not invalidate the two substantive behavior tests. Accept bounded prose-row behavior, not whole macro-engine/native security parity.

### Artifact gate / merge risk

[EXIST] PASS — all six primary files retrieved from the exact commit; source/test lengths: quest 99/249 lines, equipset 23/120 lines.
[SUBSTANTIVE] PASS — real host maps, inventory transitions and behavioral fixtures.
[WIRED] PASS — quest registration :227; macro registration :319 → dispatch :176 → lookup/transition :21.
[ANTI-PATTERN] PASS — TODO/FIXME/HACK/XXX counts each zero in the two new implementation and two test files; no empty implementation bodies or commented-out implementations.

OVERALL: FAIL for claiming both audit slices complete. Merge risk is quest reward-tooltip failures with ordinary empty host state; equipset has no identified functional regression within its declared bounded scope. No changes made beyond this report.
