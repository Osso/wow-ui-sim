# Party loot method

Bounded Retail 12.0.5 `C_PartyInfo.GetLootMethod` / `SetLootMethod` contract over existing `SimState.loot_method`, preserving legacy `GetLootMethod()` and `GetMasterLooterThreshold()`. Epoch-gated producer has saved bounded parent GREEN; independent verifier288 acceptance remains pending. See [Lua API state architecture](../lua-api.md).

## What it must do

### Shared getter and defaults

- [ ] Under `retail-12-0-5`, return exactly three C results: required numeric `Enum.LootMethod`, nullable party master index, nullable raid master index. Read existing shared state, not a separate static Group seed. Positive existing indices remain numbers; zero/negative indices become nil in C only (**inferred normalization policy**).
- [ ] Preserve fresh legacy `personalloot`, indices 0/0 and threshold 2; C reports Personal=5, nil/nil. Preserve legacy three-result string/raw-index representation, including explicit input values; neither getter mutates state or queues events.
- [ ] Translate verified existing tokens: Freeforall=0→`freeforall`, Roundrobin=1→`roundrobin`, Masterlooter=2→`master`, Group=3→`group`, Needbeforegreed=4→`needbeforegreed`, Personal=5→`personalloot`. Earlier feature epochs retain their existing static C Group expectation.

### Bounded setter policy

- [ ] Accept ordinary integral numeric enum values 0–5, plus an optional ordinary string or nil `lootMaster`; return exactly one boolean on accepted/unresolved requests. **Strict simulator validation policy, not native characterization:** reject missing/nil enum, unknown/fractional/nonfinite numbers, numeric strings, booleans, tables and functions with an explicit nonempty runtime error; validate optional master type even for non-master methods. Reject before mutation/event, without coercion.
- [ ] **Inferred non-master policy:** all five non-master values succeed (`true`), store the mapped legacy token, clear both indices and preserve threshold. An ordinary string supplied for a non-master method is validated but does not require identity resolution. Personal roundtrips even though unchanged availability currently excludes Personal.
- [ ] **Inferred master resolution policy:** resolve an ordinary name against active existing modeled identity, never invent names or indices. Concrete party member `LootPartnerTwo` at `party_members[1]` resolves to party index 2 / raid index 0; existing player `LootPlayer` at modeled raid roster position 1 resolves to party index 0 / raid index 1. Fixtures use ordinary literals matching explicit model inputs, not declassified `UnitName` results. Threshold remains unchanged even for UnitPopup's third argument `2`; no new third-argument threshold contract.
- [ ] **Inferred unresolved policy:** missing/nil/empty, unknown name, unsupported realm-qualified name or inactive group returns exactly one `false`, preserving token, indices, threshold and event queue. This is a simulator missing-identity policy, not native denial/permission parity.
- [ ] **Inferred event/repetition policy:** an accepted change of token or resolved indices queues one payload-free `PARTY_LOOT_METHOD_CHANGED`, dispatched after shared state is committed so Lua listeners observe both coherent getters and threshold. An unchanged accepted selection returns `true` without a duplicate change event; repeating the same resolved master is unchanged. Public state/events are isolated per environment.

### Restrictions and secrets

- [ ] Reuse shared [chat messaging lockdown guard](chat-messaging-lockdown.md) before effects. Explicit lockdown true rejects setter with a nonempty runtime error; false allows ordinary requests regardless of combat. Both getters remain readable in all four combat/lockdown combinations. Do not change either restriction input.
- [ ] **Conservative inferred security policy:** reject actual VM secret enum or secret master string before effects in both untainted and tainted callers, including secret optional string on a non-master request. Preserve secret values, shared state, event queue and caller taint. Do not unwrap or declassify. This is stricter than cached `AllowedWhenUntainted`; no annotation parity claim.

Boxes describe the contract, not native acceptance. Saved bounded compiled proof is recorded below; independent verifier288 remains pending.

## How it works

- [Lua API state architecture](../lua-api.md)
- [Chat messaging lockdown](chat-messaging-lockdown.md)

## Implementation inventory

- `src/lua_api/sim_substates/mod.rs:485–509` — existing six-token `LootMethodState`, default personal/0/0/2; no new input struct required.
- `src/lua_api/state/sim_state.rs` — existing `loot_method`, player/group identity and chat-lockdown input owner.
- `src/lua_api/globals/real/loot_method.rs:20–60` — unchanged legacy state-backed getters and refresh event.
- `src/lua_api/globals/enum_data/missing_enums.lua:9190–9198` — verified public numeric mapping.
- `src/c_api/c_party_info.rs` — namespace registration preserves earlier static C getter and existing availability.
- `src/c_api/c_party_info/loot_method.rs` — epoch-gated shared getter/setter, strict public argument parsing, bounded identity resolution and change-only event publication. Unknown shared tokens error explicitly rather than inventing a numeric method.
- `src/c_api/c_chat_info.rs:20–29` — existing restriction guard to reuse; unchanged here.
- `src/lua_api/globals/group_queries.rs:253–315,469–496` — existing group/raid classification, player-at-roster-1 and modeled party identity evidence.
- `tests/party_loot_method.rs` — twelve epoch-gated fixtures, autodiscovered by existing grouped integration harness.
- `tests/party_info_loot.rs` — availability unchanged; default expectation Personal under epoch, Group before epoch.

## Tests asserting this spec

`tests/party_loot_method.rs` covers fresh defaults/arity, six explicit shared tokens, positive/nullable indices with unchanged legacy outputs, five non-master roundtrips, idempotent repetition, concrete party/player roster resolution, unresolved/missing/inactive master atomicity, environment isolation, combat×lockdown, strict enum validation, optional master validation, and real VM number/string secrets with tainted callers. Events are drained and dispatched to a real Lua listener that observes committed state.

`tests/party_info_loot.rs` retains the five seeded available choices/Personal unavailable, with epoch-sensitive getter default. Existing `tests/loot_method.rs` remains unchanged as legacy controls.

Parent intended filter: `cargo test --test integration party_loot_method::`; availability/legacy controls additionally needed. No new Cargo target or manifest changes.

## Reconciled batch34 parent proof — 2026-10-01

- Initial inputs `42e15e675` failed compilation (exit101, 110.49s): three unsupported u8 result conversions, not behavioral RED. Corrected fixtures `1c7af9c03` compiled exit0 in175.01s; saved selected execution exit101, **0 PASS / 12 FAIL**. Evidence: `/tmp/patch-12.0.5-batch34-red{,-fixed}-build-result.json`, corrected `red-fixed-run.json` / `run.log`.
- Producer `f2e85fcb6` first GREEN build attempt exited101 after3.40s in `build.rs` against a concurrently incomplete countdown fixture. No runtime execution or loot regression established by that failure. Stable fixture revision `676e4c25a` compiled exit0 in151.54s, as saved in `/tmp/patch-12.0.5-batch34-green{,-fixed}-build-result.json` and corresponding logs/JSONL.
- `/tmp/patch-12.0.5-batch34-green-fixed-runs.json` binds runs0–2 to integration SHA256 `7fd860d9023a6b3a3643a6e3a90caad5442042e98fb401d834fc6d5fd0c5aac6`: loot12 + availability1 + broad legacy filter18 PASS. The last filter includes the same12 loot tests plus six legacy controls: **19 unique PASS / 31 passing executions**, not31 distinct cases. Saved logs establish bounded shared defaults/arity, six tokens, nullable indices, setters, threshold preservation, concrete identity resolution, atomic invalid/unresolved/secret/lockdown handling, isolation and change-only coherent listener state; availability and legacy controls remain intact.
- Same binary run3 (`party_countdown::`) exits101, **0 PASS / 10 FAIL**: behavioral RED for the next slice, not loot acceptance or a compilation failure. Countdown remains open; its spec/wiki ownership is excluded here.
- Saved startup at `676e4c25a` exits0 with `[]` and CLEAN0 unique/occurrences; `/tmp/patch-12.0.5-batch34-green-fixed-startup-run.json` binds wow-sim SHA256 `770a9a5522645f8cba951e30070406dbb8c14c1888ee9bff7284dd2527e42ba7`. This is parent snapshot proof, not a fresh rerun or broader compatibility gate.

**Verifier288 pending.** No fresh fmt/check/readability, native security/permissions, all-profile or full-row acceptance inferred. Final `prose-2026-03-31-169` remains pending for broader limits/countdown; **264 pending / 84 bounded / 14 partial = 362**, retained IDs/source hash and unrelated statuses unchanged. Docs/accounting inspection only; no code/tests/builds/delegation.

## Exact evidence and inference boundary

- Final retained source `data/patch-api/sources/12.0.5-register.json:1079–1085`, ID `prose-2026-03-31-169`: restrictions on countdown, ready check, ping and loot-method APIs apply to chat messaging lockdown rather than all combat. Superseded March25 prose is not evidence for this slice.
- Active Retail runtime cache `~/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/PartyInfoDocumentation.lua:287–295`: getter required enum + two nullable numbers. Lines 562–576: setter required enum, optional nullable string, `HasRestrictions=true`, `SecretArguments="AllowedWhenUntainted"`, one required bool. Lines 794–798: unique `PARTY_LOOT_METHOD_CHANGED`. These do not establish default, accepted enum handling, identity resolution, success/error policy or event timing.
- Cached `Blizzard_UnitPopup/Standard/UnitPopupButtons.lua:597–770`: enum-valued selection/checks; Masterlooter calls setter with player name and third argument `2` at 712–728. The third argument is caller evidence, not part of generated setter documentation. Ignoring it rather than changing threshold is bounded simulator policy.
- `src/lua_api/sim_substates/mod.rs:485–509` explicitly lists all six spellings; `tests/loot_method.rs:37–59` asserts personal default and direct shared master indices. `src/lua_api/globals/enum_data/missing_enums.lua:9190–9198` supplies all six numeric assignments; no inference from list order.
- Existing `group_queries.rs` models party names through `party_members` and classifies six or more other members as a raid; `raid_roster_member` explicitly places the player at roster index 1. These support concrete fixture identities/positions only, not native master eligibility.
- Strict validation, nonpositive normalization, secret rejection, unresolved false, non-master success/index clearing, ignored ordinary non-master name/third arg, payload-free event and repeat-no-event policy are **inferred simulator decisions**. No native permission enforcement or exact malformed/blocked semantics has been established.

## Known gaps (current cycle)

- [x] Parent saved bounded GREEN and legacy/availability controls: 19 unique PASS, provenance and duplicate accounting above.
- [ ] Independent verifier288 acceptance and broader gates remain pending.
- [ ] Cached `AllowedWhenUntainted` acceptance is not implemented by conservative rejection; native taint/security parity remains unknown.
- [ ] General raid member resolution is unclaimed: existing raid `UnitName`/existence indexing differs from `GetRaidRosterInfo`'s player-first roster. Only explicit player roster index 1 is selected here; do not fabricate a general mapping.
- [ ] Solo/party-player master encoding, native missing/ambiguous/realm-name policies, native permissions and native event repetition/timing remain unknown. Existing zero-index state cannot distinguish an assigned party player from no assignment without a separate contract.

## Out of scope

- Availability redesign, eligibility/leadership/native permissions, loot distribution and threshold mutation; existing availability set is deliberately unchanged.
- Native lockdown activation/reset, countdown/other named party actions, all-profile/native parity and closure of final prose row169.
- Lua/vendor patches, new Cargo targets, PLAN/audit promotion, compiled verification and delegation in this production slice; parent owns runtime proof; independent acceptance remains pending.
