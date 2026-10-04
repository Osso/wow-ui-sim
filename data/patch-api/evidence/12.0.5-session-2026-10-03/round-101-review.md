# Round 101 independent read-only verification

**Overall: ACCEPT WITH QUALIFICATIONS. Merge risk: medium for existing bare API callers; low for the GUI completion relocation. Not full-row acceptance.**

Scope: commits a9f6b0a26 and d62a972eb, artifact/source inspection only; inspected master HEAD 4ed44718e, clean. Read verify skill; no cargo, tests, repo edits, git mutation, agents or model CLIs. Reported execution evidence is attributed, not independently rerun.

## 1. Cached aura ordering — ACCEPT WITH QUALIFICATIONS

`tests/secure_aura_header_helpers.rs:37–58` appends unannotated SecureAuraHeader.lua/xml entries to the parsed cached restricted-environment TOC before its first load. Actual cached Lua/XML, not a replacement comparator, supplies OnShow/OnEvent and child management. `:88–128` requires the Lua function, XML OnShow script, hidden template and live aura-index resolution; `:144–211` asserts short < long < permanent under TIME ascending, shuffled input, finite-to-permanent transition, removal/readdition and stale-child hiding. Permanent tie order intentionally unspecified.

These assertions are sensitive to wrong vendor sorting and wrong API data insofar as those faults alter the tested observable order, IDs or child lifecycle. No producer change is necessary for a useful regression test of already-correct behavior. They are not exhaustive expiration/GetUnitAuras tests: e.g. a monotonic distortion of finite expiration values could preserve order and pass.

Deployment qualification: cached `Blizzard_RestrictedAddOnEnvironment/Blizzard_RestrictedAddOnEnvironment.toc:16–17` marks BOTH files `[AllowLoadGameType classic]`; scanning cached TOC/XML found no alternate include of these two files. `src/toc/mod.rs:84–90` enforces those filters. The fixture deliberately bypasses that eligibility; it does not prove retail normally loads or exposes this template. Cached `SecureAuraHeader.lua:460–466` reads GetUnitAuras and replaces zero expiration with math.huge, matching source prose lines 112/151. This is legitimate isolated compatibility-source testing, not proof of retail production reachability. Keep applicability explicit; do not count normal retail runtime support from these tests alone.

## 2. Helper commands and unloaded UI — ACCEPT WITH QUALIFICATIONS

Important distinction: the callbacks live in cached `Blizzard_ChatFrame/Shared/ClassTalentHelper.lua:15–33`, NOT in Blizzard_PlayerSpells. That file's `:1–4` lazy-loads PlayerSpells when necessary. Absence of PlayerSpells alone therefore does not imply a no-op.

A truly bare `WowLuaEnv::new()` installs simulator APIs but no Blizzard helper callbacks (`src/lua_api/env.rs:75–105`). Valid SwitchTo* arguments enter `src/c_api/class_talent_commands.rs:52–73`, dispatch an event, then return zero Lua results. The empty callback snapshot returns Ok (`src/lua_api/globals/real/event_callbacks.rs:174–220`); no state mutation, missing-handler error, load attempt or queued retry occurs. All four commands are silent no-ops without callbacks. Bad argument conversions can error; registered callback errors propagate normally.

This changes existing bare callers: former functions immediately mutated specialization/talent/loadout state, now retained only under `cfg(not(retail-12-0-5))` (`src/lua_api/globals/missing_surface/traits/class_talents.rs:246–304`). Availability/semantics break is real, although native vendor's callback contract supports the intentional delegation. Do not describe the rewrite as preserving bare API behavior. Third-party direct C_* calls after ChatFrame registration use the same real callbacks and lazy-load branch; calls before that registration are lost. GUI before PlayerSpells loads is a different case from a bare environment. Cold loading is source-reachable, not behaviorally proven by preloaded fixtures (`tests/secure_aura_header_helpers.rs:217–227`).

Registration and added completion behavior are retail-12-0-5-gated; earlier epochs/classic/Forever preserve old command mutation and completion semantics. Retail/PTR newer epochs inherit 12.0.5 via Cargo features. No old-profile execution proof supplied.

## 3. Shared cast completion — ACCEPT

Compared parent `src/iced_app/casting.rs` with committed `src/lua_api/cast_completion.rs`: existing extraction, STOP/SUCCEEDED/tradeskill publication and spell-effect bodies unchanged; visibility expands and test paths move. `src/iced_app/update.rs:565–568` calls the new shared wrapper at the same GUI tick point.

Order retained: clear casting state → UNIT_SPELLCAST_STOP → UNIT_SPELLCAST_SUCCEEDED → UPDATE_TRADESKILL_CAST_STOPPED(false) for recipes → spell effects/UNIT_HEALTH or UNIT_AURA → apply pending specialization → PLAYER_SPECIALIZATION_CHANGED(player). Retail additions synchronize talent/config/hero state alongside player specialization before the latter event, then publish no-payload ACTIVE_PLAYER_SPECIALIZATION_CHANGED (`src/lua_api/cast_completion.rs:75–100`). No other event-order change found. Added notification chronology is simulator policy, not native chronology proof. Existing callback-reentrancy policy/error handling unchanged; relocation does not repair or worsen it.

All four original lib files remain present and included via `#[path]` with original feature guards (`src/lua_api/cast_completion.rs:5–16`); previously GUI-only parent no longer required. Inspected `round-101-green-lib-cast-completion.log`: 12 passed/0 failed, split 1 duration + 5 input + 3 interrupted + 3 identity/completion. This is existing worker execution evidence, not a verifier rerun.

## 4. Existing tests — ACCEPT WITH QUALIFICATIONS

- `tests/hero_talents.rs:207–214`: loaded real-helper lifecycle replaces bare immediate command assertions. `tests/secure_aura_header_helpers.rs:452–580` retains concrete 202/102/301 loadout mapping, names, last-selection and hero-subtree checks, adds deferred state, identity/event ordering and no repeated completion. Legitimate new asynchronous contract test, but bare-environment command availability and immediate mutation are no longer protected. That lost guarantee is the item-2 compatibility change, not equivalent old behavior.
- `tests/admin_spec_talent_api.rs:277–278`: LoadConfig(configs[2], true) + Ready assertion is legitimate setup for tree/system mapping checks; unchanged mapping assertions. It intentionally no longer covers SwitchToLoadoutByIndex in a bare environment.
- `tests/hero_talents.rs:22–30,414` and `tests/hero_talents/rendering.rs:5,154,231`: coherent Protection player/talent state seeded before UI loading. Glow, edge ordering/releveling and anchoring assertions unchanged. Legitimate rendering fixtures, not tests of transition/lazy loading.
- `tests/unit_auras_private.rs:45–73`: changed observer matches actual event payload `(unit, castGUID, spellID, castBarID)` from `src/lua_api/spellcast_events.rs:12–23`. Original spell-ID/unit-filter/count checks retained; query agreement added. This test is already gated retail-12-1-0.
- `tests/combat_verbs.rs:235–247` and `tests/click_targeting.rs:117–130`: numeric ID slot 10 for current retail, slot 7 for historical tuples; strict monotonicity and blocked-second-cast equality retained. Actual producer `src/lua_api/globals/utility_system_spell/spell_api.rs:475–502` gates the new tuple on player-cast-durations, not retail-12-1-0. Their epoch predicate is correct for standard retail/older classic bundles, but not capability-only builds or Forever, which also enables player-cast-durations. This broader-profile mismatch already existed with unconditional slot 7; adaptations do not fix it. No claim of all-profile correctness.

Pre-existing failures: inspected worker logs `round-101-green-unit_auras_private.log` (4 pass/1 fail), `round-101-green-combat_verbs.log` (15/1) and `round-101-green-click_targeting.log` (21/1). Last two report “expected numeric result, got string”; first fails the obsolete payload assertion. These are pre-adaptation worktree executions, not runs of untouched canonical master. Git comparison confirms both relevant producer files unchanged from a9f6b0a26's parent through d62a972eb; old test observers therefore already contradicted parent master under default retail features. No evidence those three passed in main's earlier runs; they were excluded. No new helper regression inferred from these failures.

## 5. Prose accounting — ACCEPT WITH QUALIFICATIONS

| Prose row | Recommendation | Proven clause / unproven clause |
|---|---|---|
| prose-2026-03-25-112 | PARTIAL | Bounded TIME-ascending short < long < permanent behavior of explicitly loaded cached template; ordinary retail applicability/exposure pending. |
| prose-2026-03-31-151 | PARTIAL | Same concrete ordering proof, finalized wording. Do not count forced classic-only file inclusion as retail deployment proof. |
| prose-2026-03-25-120 | PARTIAL | Four helper commands drive real callbacks with clean actual talent UI writes; specialization additionally proves clean player/overlay castbar writes through completion. Loadout Ready-path only; delayed commit/castbar clause unproven. |
| prose-2026-03-31-177 | PARTIAL | Same finalized taint-neutrality coverage; LoadInProgress loadout commit/player-castbar path remains unmodeled and uncredited. |

Proposal and finalized rows should share each capability's evidence, not count as four independent implementations. Aura ordering itself merits bounded component credit; retail applicability remains pending, not established merely by appearing in a mainline patch prose page. Helper lifecycle/native argument/combat/cold-load parity remains bounded, not universal. Author explicitly excludes delayed loadout commit/castbar flow, cold lazy loading and native-client comparison; those exclusions are real limitations, not reasons to mark full prose clauses complete.

### Defects / merge qualifications with locations

1. **New compatibility regression:** valid bare SwitchTo* calls silently do nothing (`src/c_api/class_talent_commands.rs:52–73`; retail registration `src/lua_api/globals/missing_surface/traits/class_talents.rs:107–113`). Existing direct-mutation callers lose behavior. Source supports intentional native callback delegation, so this is not evidence to restore a prohibited fallback. Accept only with the loaded-callback precondition made explicit; preloaded tests do not prove cold behavior.
2. **Existing broader-profile fixture defect remains:** numeric query decoding keys on retail-12-1-0 (`tests/combat_verbs.rs:236`, `tests/click_targeting.rs:118`) instead of producer capability player-cast-durations (`src/lua_api/globals/utility_system_spell/spell_api.rs:477–494`). Forever/capability-only builds still select the wrong slot. Default retail repairs are valid; do not claim all-profile repair.
3. **Accounting risk, not production-code defect:** forced file eligibility (`tests/secure_aura_header_helpers.rs:49–58`) bypasses classic-only cached TOC entries. This must stay isolated-source proof, not retail runtime support.

### Verification protocol / proof ledger

[EXIST] PASS — all added/changed implementation and test files inspected; four retained casting test files present (44/170/155/118 lines); old parent casting module intentionally removed.

[SUBSTANTIVE] PASS — actual secure native-event dispatch, real cached helper callbacks, shared timed completion, and observable state/taint/child-order assertions. Not placeholder implementations.

[WIRED] PASS WITH QUALIFICATIONS — retail registration calls class_talent_commands::register; GUI update calls cast_completion::tick_casting; generated integration harness discovers top-level test files (`build.rs:55–95`); four lib modules included via path attributes. No helper consumer in a bare environment; forced aura inclusion is fixture-only.

[ANTI-PATTERN] PASS — added diff lines contain zero TODO/FIXME/HACK/XXX markers; no new empty catch/silent exception placeholder or commented-out implementation found. Existing event-error handling retained.

Execution ledger: no verifier test/check invocations. Caller reports 144/144 master tests, startup lua-errors [], fmt clean. Worker report attributes 381 integration + 12 lib tests and cargo check to staged worktree revisions, not canonical master; inspected 12/12 lib log and three pre-adaptation failing logs. Source comparison proves unchanged ordinary completion/event producers. No full suite, live GUI, native-client, cold-load or other-profile acceptance claim.
