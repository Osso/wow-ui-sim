# Quest log reward favor

`C_QuestInfoSystem.GetQuestLogRewardFavor(questID, clampFavorToCycleCap)` queries explicit host-seeded quest favor amounts. Retail 12.0.5 [GlobalAPI source](../../data/patch-api/sources/12.0.5-api-changes.txt) line 293 changes `arg1.Nilable false -> true`; source ID `global api-C_QuestInfoSystem-GetQuestLogRewardFavor-293`. Cached retail `Blizzard_APIDocumentationGenerated/QuestInfoSystemDocumentation.lua:41–56` declares nullable numeric `questID`, nullable boolean `clampFavorToCycleCap`, `SecretArguments = "AllowedWhenUntainted"`, and one nonnil numeric `amount`. This declaration is not native behavioral evidence. See [Lua API architecture](../lua-api.md).

## What it must do

### Explicit inputs and numeric output

- [ ] Keep an independent, empty-by-default quest-keyed map per environment; each host-supplied record contains raw and already-cycle-capped numeric amounts. Do not synthesize rewards, cap arithmetic or catalog data.
- [ ] Return exactly one numeric amount from the requested quest's record: raw for false, already-capped for true. Reflect replacements/removals immediately; preserve fractional amounts and explicitly seeded zero amounts.
- [ ] **INFERRED** omitted/nil quest ID selects only an explicit host `context_quest_id`, never `selected_quest_log_id`, a quest log index or an arbitrary map entry. Explicit IDs override that context; absent explicit records never fall back to it.
- [ ] **INFERRED** omitted/nil clamping means false. Nil context without a host-selected quest, unknown quests and removed records return one public zero: a quest with no host record rewards no favor, which is what the cached reward tooltips (`QuestUtils.lua:778`, `GameTooltip.lua:198`) compare numerically. An explicit unknown ID never falls back to the context reward. Native miss behavior is not established.
- [ ] Queries leave host records/context unchanged; environments remain isolated. **INFERRED** outputs are public numbers, not native output-security proof.

### Arguments and caller security

- [ ] **INFERRED** require nil or a finite integral u32 quest ID, including zero and u32 maximum; reject numeric strings, booleans, tables, functions, threads, frame objects, fractional/negative/nonfinite/out-of-range numbers before state lookup. Require nil or an actual boolean for clamping, not Lua truthiness.
- [ ] Authenticate both declared arguments and every supplied extra argument with `rilua::table_security::unwrap_secret` before validating either declared argument or consulting host inputs. This models declared `AllowedWhenUntainted`, not blanket secret rejection.
- [ ] Accept authentic secret quest numbers, boolean flags and nil wrappers from untainted callers. Preserve rooted wrapper identity and secrecy across repeated queries/GC; do not replace/declassify caller values or change caller taint.
- [ ] Deny tainted callers' secrets in either position or combinations, including secret nil and secrets accompanying invalid public quest IDs or absent records. A second-position secret denial must precede first-position public validation; extra-position secrets must not bypass authentication.
- [ ] Public calls work from tainted and untainted callers without changing caller taint. **INFERRED** ignore public extra arguments after authentication; native extra-argument semantics remain unverified.

## How it works

- [Lua API architecture](../lua-api.md)
- [State-backed Maw-power precedent](spell-maw-powers.md)

## Implementation inventory

- `src/c_api/c_quest_info_system.rs` — explicit `QuestFavorState` / `QuestRewardFavor`, argument authentication and state-backed namespace query.
- Wired in `src/c_api/mod.rs`, `src/lua_api/state/sim_state.rs`, `src/lua_api/state.rs`, `src/lua_api/globals/register.rs`, all under `retail-12-0-5`.

## Tests asserting this spec

`tests/quest_reward_favor.rs` — eight bounded tests: empty-state zeros; omitted/nil quest and explicit-context selection; distinct raw/capped records and live replacement/removal; zero/u32 endpoint keys; invalid public representations; read-only inputs/environment isolation; untainted authentic secrets with GC; tainted public calls and secret rejection ordering/recovery. No API replacement fixtures; `env.eval` numeric results use f64.

## Development proof and independent bounded acceptance — 2026-10-03

Commit `a4cce2db1`. RED: 0 PASS / 8 FAIL at 92e4ea045's parent; the miss policy changed in a4cce2db1 after a first review rejected raising. GREEN: 8/8 inside a 404/404 run with control suites; `cargo fmt --check` exit0; startup `lua-errors` `[]`. This section supersedes any wording above that describes the slice as staged, unapplied or unrun.

Main accepts an independent GPT-6.1-sol source review (no test rerun): **ACCEPT WITH QUALIFICATIONS**, [report](../../data/patch-api/evidence/12.0.5-session-2026-10-03/b98-verify-events-commands.md) SHA256 `72c3dbad682b576a045792d498f279ba54e7ef7b6094d5364da3d8a5817b4ac3`. First review (b97-verify-quest-equipset.md) rejected the raising miss; zero-on-miss is inferred policy. Requirement checkboxes are left as authored; the report lists which are earned and to what bound. Bounded simulator proof, not native parity.

[Page accounting](../../data/patch-api/sources/12.0.5-page-coverage.json): global api-C_QuestInfoSystem-GetQuestLogRewardFavor-293 bounded-coverage under capability `quest-reward-favor`; **119 capabilities/362 IDs; 42 pending /262 bounded /23 partial /35 metadata**.

## Known gaps (current cycle)

- [ ] Cached reward-tooltip callers were inspected, not executed with loaded UI.
- [ ] Obtain native evidence for nil selection, clamp defaults, cap semantics, unknown-quest behavior, numeric domain, malformed/extra arguments and output secrecy. Do not close audit row 293 based solely on inferred simulator tests or registration.

## Out of scope

- Native favor acquisition, currency/cycle-cap calculation, quest selection synchronization and production reward catalogs: host supplies inputs explicitly.
- Additional `C_QuestInfoSystem` APIs, legacy globals, unrelated quest reward paths, events and vendor/cache edits.
- Older API epochs: model/state/registration and tests gated to `retail-12-0-5`; no compatibility fallback.
- Native error wording or full native parity: declaration does not establish either.
