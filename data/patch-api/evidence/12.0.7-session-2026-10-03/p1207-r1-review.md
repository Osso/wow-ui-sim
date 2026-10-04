# p1207-r1 independent source-read review

Status: complete (source-only). Source-only review; no builds, tests, simulator, agents, or repository writes authorized.

## Early findings

- 316973443 changes namespace ping authentication only; legacy slots remain Lua-table-backed, with no new host state. Spec explicitly excludes native invocation/restrictions.
- 447d45d7c adds tests/specs only; color spec explicitly admits cached trigger/ColorMixin signature mismatch.
- 97055bf28 replaces writable clock tables with host userdata; clock input authentication is added to mutators and duration binding/activity. Generic SetClock remains accepted, not strict declared clock typing.
- No executable proof has been run. Source-only predictions must not be called GREEN.

## Integration checkpoint

Master confirmed `7702befe8a567c225d9e8680594186e9689734f4`; branch confirmed `97055bf282a17d88a79af7d59ba10bd5cd532b63`. `git log 50d390688..master` contains five round-2 commits. Shared touched path is only `src/c_api/mod.rs`, with independent insertions. Read-only three-tree inspection used `git merge-tree --trivial-merge 50d390688 7702befe8 97055bf28` (exit 0); no checkout/index/repository writes or builds. Existing startup clock tests use methods, not raw clock table writes. Final conflict/behavior assessment follows below.

## Reconstructed commit record

All citations below refer to branch tip `97055bf28`, relative to `/home/osso-test/.worktrees/wow-ui-sim-p1207-r1`, unless explicitly prefixed `master:` or `cache:`. Cache paths resolve under `~/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/`. Row suffix numbers are literal lines of `data/patch-api/sources/12.0.7-api-changes.txt`, not invented coverage identifiers.

### 316973443 — Authenticate pending ping callback inputs

Changed files: `docs/specs/pending-secure-callback-storage.md`, `src/c_api/c_ping_secure.rs`, `tests/p1207_pending_callbacks.rs`. `git show` reports 471 insertions, 1 deletion. Runtime change is only the namespace setter and helper at `src/c_api/c_ping_secure.rs:121–142`; legacy accessors remain unchanged. No SimState fields added.

New tests below are in `tests/p1207_pending_callbacks.rs`. Mapping keys: P = namespace rows 046/047; B = button rows 055/059; G = legacy ping rows 056/060; R = toggle-run rows 057/061.

| Test (exact name, declaration line) | Source rows exercised | Can pre-change provider pass? |
|---|---|---|
| `slots_replace_identity_clear_and_have_exact_public_arities`:66 | B,G,R | Yes; storage already exists. |
| `namespace_and_legacy_ping_share_one_live_slot`:97 | P,G | Yes. |
| `clear_ping_preserves_button_and_run_slots_and_environment_isolation`:127 | P,B,G,R | Yes. |
| `all_slots_root_closures_and_upvalues_across_full_gc`:161 | B,G,R | Yes. |
| `storage_operations_preserve_secure_and_tainted_caller_context`:185 | P,B,G,R | Yes; notably allows tainted public namespace calls. |
| `retrieved_callback_keeps_its_taint_when_event_consumer_invokes_it`:216 | 047,056 | Yes. |
| `replacement_and_clear_inside_callback_take_effect_on_next_event`:237 | P,056 | Yes. |
| `nested_real_fire_event_observes_replacement_without_duplicate_old_call`:260 | G | Yes. |
| `callback_error_does_not_change_slot_or_break_later_dispatch`:280 | P,G | Yes; test consumer supplies pcall isolation. |
| `namespace_rejects_malformed_callback_atomically_and_recovers`:318 | 047,056 | No: old setter stores malformed/nil values. |
| `namespace_unwraps_authentic_function_for_secure_caller`:341 | 047,056 | No: old setter stores wrapper, not underlying closure. |
| `namespace_authenticates_extra_arguments_before_type_validation`:361 | 047,056 | No: old setter neither authenticates nor rejects extras. |

These are source predictions, not executed RED/GREEN. Constant nil/manufactured callbacks cannot satisfy identity/replacement/invocation assertions. They can all exercise the old Lua-table compatibility implementation: the consumer is expressly “Test-owned consumer, NOT a native secure-input producer” (`tests/p1207_pending_callbacks.rs:22`). The error test's pcall proves consumer policy, not provider error isolation (`:292–297`).

### 447d45d7c — Prove money lines and encounter snapshot/color subsets

Changed files: `docs/specs/p1207-b14-tooltip.md`, `docs/specs/p1207-b15-encounter-end.md`, `docs/specs/p1207-b16-colors.md`, `tests/p1207_b14_tooltip.rs`, `tests/p1207_b15_encounter_end.rs`, `tests/p1207_b16_colors.rs`. 292 insertions; no producer or state change.

| New test | Source rows | Pre-change / constant assessment |
|---|---|---|
| `loaded_money_helper_reflects_mail_amount_changes_and_append_order`, `tests/p1207_b14_tooltip.rs:4` | prose 007, global 058 | Pre-change can pass intentionally. One fixed output cannot satisfy 123/20000/zero, append order, highlight/red, and clear/reappend assertions (`:14–41`). Tests two mail amounts, not modification of an existing mail record. |
| `tainted_encounter_listener_observes_public_detached_multi_boss_snapshot`, `tests/p1207_b15_encounter_end.rs:6` | prose 009, event 163 | Pre-change can pass. Fixed/empty status cannot satisfy two bosses then changed health 62.5→11 and detached earlier snapshot (`:10–12,39–66`). No automatic engaged-boss producer is tested. |
| `warning_rgba_overrides_are_live_detached_and_environment_local`, `tests/p1207_b16_colors.rs:6` | prose 010; indirect input to 033 | Pre-change can pass. Constant color fails distinct IDs, new RGBA, alias edits, and empty independent environment (`:10–55`). No trigger or timeline notification tested. |
| `legacy_timeline_bridge_reflects_warning_override_alpha_changes`, `tests/p1207_b16_colors.rs:64` | prose 010, global 033 | Pre-change can pass. Fixed white fails changing alpha .375→.875 (`:69–76`). Test deliberately enforces old four-return tuple, not cached ColorMixin. Excluded under retail-12-1-5. |

### 97055bf28 — Model authenticated host-owned duration clocks

Changed files: `docs/specs/duration-core.md`, `src/c_api/duration_clock.rs`, `src/c_api/mod.rs`, `src/lua_api/globals/lua_duration_object.rs`, `src/lua_api/globals/lua_duration_object/core.rs`, `tests/duration_core.rs`, `tests/patch_12_0_7_duration_clocks.rs`. 476 insertions, 120 deletions.

New tests below are in `tests/patch_12_0_7_duration_clocks.rs`.

| Test (exact name, declaration line) | Source rows | Pre-change assessment |
|---|---|---|
| `manual_clock_fractional_transitions_independence_and_return_arity`:13 | 032,083–087 | Can pass old mutable-clock provider. Cannot pass one constant. |
| `manual_clock_state_is_host_owned_not_raw_writable_table_storage`:35 | 032,083,084 | Old provider fails userdata/nonwritable/receiver behavior. |
| `manual_clock_inferred_validation_rejects_before_mutating`:54 | 032 indirectly; 083,084,086,087 | Old provider fails missing/nil/nonfinite/overflow rejection. |
| `manual_clock_allowed_when_untainted_authenticates_inputs_before_receiver_validation`:84 | 083,084,086,087 | Old provider fails authenticated secret numeric acceptance and denial order. |
| `duration_clock_rebinding_reads_live_boundaries_and_roots_clock_identity`:115 | 032,083–088,090–092 | Can pass existing provider; clock methods, GC identity, before/start/end/rewind remain meaningful nonconstant proof. |
| `duration_set_clock_authenticates_wrapped_clock_and_nil_without_rebinding_on_denial`:163 | 088,092; 083 indirectly | Old provider fails decoded clock identity/unbinding/atomic denial. |
| `duration_activity_predicates_authenticate_and_validate_modifier_without_changing_state`:193 | 090,091; 083,087,088,092 controls | Old provider ignores invalid/secret modifiers, so fails. |
| `duration_clocks_and_bindings_are_independent_between_environments`:230 | 032,083–085,088,092 | Can pass old provider; concrete elapsed/time changes reject one constant/global clock. |
| `allowed_when_untainted_authenticates_ignored_extras_before_any_validation`:252 | 084,086,087,090–092; 083/088 atomicity observations | Old provider fails denied extra-argument behavior. |

Existing tests modified, not new: `duration_percent_rejects_invalid_bound_clock_without_mutation` (`tests/duration_core.rs:342–370`) now deliberately binds a generic writable table at :347; `duration_curve_invalid_bound_clock_preserves_duration_and_curve_state` (:884 onward) replaces manual clock with `{time=15}` before corrupting it. This preserves invalid-query behavior, but no longer proves a manual instance can be corrupted. New test :35 separately proves the intended prohibition.

Total: **25 new test functions**, two existing tests adjusted. No new source-substring assertions. Userdata `type`, raw writes, method replacement, return tuples, identities, taint, event payloads, and errors are observable public contracts, not source-structure assertions. Neither test name nor author prediction constitutes executed proof.

## Wiring / artifact checks

[EXIST] PASS: every changed file exists and was read via git show and/or branch files. [SUBSTANTIVE] PASS for manual clocks and authentication: `ManualClock { time: f64 }` (`src/c_api/duration_clock.rs:10–12`), finite mutation :124–139, namespace authentication :132–142. Qualification: callback storage explicitly remains “inert compatibility state” (`src/c_api/c_ping_secure.rs:4–6`), not the requested modeled native producer.

[WIRED] PASS: `src/lua_api/globals/register.rs:229` calls duration registration; `src/lua_api/globals/lua_duration_object.rs:104–109` installs the new Rust factory; `src/c_api/mod.rs:280` calls ping surface registration; `src/c_api/c_ping_secure.rs:81–85` installs authenticated namespace setter. `Cargo.toml:9` disables automatic Cargo tests, but `build.rs:463–497` discovers top-level .rs modules and does not exclude these new files; `tests/integration.rs:1` includes the generated harness. All new files are retail-12-0-7 gated.

[ANTI-PATTERN] Changed producer/test files contain no TODO/FIXME/HACK/XXX markers. This is only a marker scan, not proof against compatibility shims or fallbacks; concrete counterexamples follow.

## Source/declaration conformance by touched row

The source names below are verbatim retained-page entries. All current recommendations are **audit-pending**: row ledger has `capabilities: []` and no recovered execution evidence establishes this tip. `data/patch-api/sources/12.0.7-page-coverage.json:7` explicitly says “Development tests are not independent final acceptance.” No row is promoted merely because a new test is predicted to pass. Candidate bounded scopes are listed; full-row closure is not proposed.

### B06/B07: clock and duration rows

Cache declarations cited in this section are under `cache:Blizzard_APIDocumentationGenerated/`.

| Exact row ID | Contract comparison / bounded candidate | Recommended status and gap |
|---|---|---|
| `global api-C_DurationUtil-CreateManualClock-032` | Cache `DurationUtilDocumentation.lua:31–38`: one nonnil `LuaDurationManualClock`, no declared arguments. New factory returns one real userdata (:15–25 of duration_clock.rs), defaults time to zero. Optional initial time is an extension, not declared parity. | audit-pending; host-owned zero-clock candidate after execution; factory extension/secrecy/extra policy not native-proven. |
| `scriptobjects-DurationClock-GetTime-083` | Cache `LuaDurationClockAPIDocumentation.lua:11–22`: no inputs, one nonnil FrameTime. Producer :105–108 returns live f64, not constant; userdata type matches cache ObjectType at :5. | audit-pending; live finite timestamp bounded candidate. |
| `scriptobjects-DurationManualClock-AdvanceTime-084` | Cache `LuaDurationManualClockAPIDocumentation.lua:11–20`: required nonnil DurationSeconds, AllowedWhenUntainted, no returns. :124–147 computes live previous+delta only after auth/finite validation. | audit-pending; candidate authenticated fractional advancement. Signed/finite/overflow policy inferred. |
| `scriptobjects-DurationManualClock-ResetTime-085` | Cache :22–29 says “Resets the clock to a zero time value.” :155–158 writes zero and returns none; no explicit secret policy declared for ResetTime. | audit-pending; reset bounded candidate; extra/receiver secret policy unspecified. |
| `scriptobjects-DurationManualClock-RewindTime-086` | Cache :31–40: required nonnil DurationSeconds, AllowedWhenUntainted. :132–151 computes previous−delta with same authentication. | audit-pending; candidate authenticated rewind; signed/finite policy inferred. |
| `scriptobjects-DurationManualClock-SetTime-087` | Cache :42–50: required nonnil FrameTime, AllowedWhenUntainted. :124–143 validates finite number and writes it, zero returns. | audit-pending; candidate authenticated set with atomic denial. |
| `scriptobjects-DurationObject-GetClock-088` | Cache `LuaDurationObjectAPIDocumentation.lua:181–192`: one nullable LuaDurationClock, nil means internal GetTime-equivalent source. `lua_duration_object.rs:291–295` returns exactly stored value including nil. Normal bound clocks match; arbitrary values admitted by SetClock break declared return type. | audit-pending; identity/nil/default source candidate, generic-value typing remains partial. |
| `scriptobjects-DurationObject-HasStarted-090` | Cache :366–379: default RealTime, nonnil DurationTimeModifier, AllowedWhenUntainted, one nonnil bool. core.rs:315–328 authenticates before modifier validation; :337–339 tests base>0 and now>=start. Default and both known modifiers accepted. Zero-span rule remains explicit prior inference, not universal cache wording. | audit-pending; nonzero start-boundary/authentication candidate. Explicit nil/default semantics and zero-span inference not native-proven. |
| `scriptobjects-DurationObject-IsActive-091` | Cache :382–395: same modifier/auth policy, “at or after its start time and before its end time.” core.rs:348–351 matches nonzero interval boundaries; tests move across both. | audit-pending; nonzero active-window/authentication candidate. |
| `scriptobjects-DurationObject-SetClock-092` | Cache :421–429: one nullable LuaDurationClock, AllowedWhenUntainted, no returns. New code authenticates supplied values before receiver/timing checks but stores any decoded value (:298–306), with no LuaDurationClock validation. | audit-pending; authenticated identity/unbind candidate; cached type conformance FAIL. |

The concrete SetClock counterexample is source-derived: `d:SetClock(false)` stores false and `d:GetClock()` then returns false, violating “Type = "LuaDurationClock", Nilable = true”; `{time=15}` is explicitly accepted by adjusted tests (`tests/duration_core.rs:347–349`). No execution was used to claim this counterexample. Duration objects remain table proxies with timing slots and `clock` field (`core.rs:31–45,115–133`; `lua_duration_object.rs:306`), not host-owned duration state. Host-owned manual time does not by itself satisfy the host-state standard for every B07 row.

### B12/B13: pending callback rows

| Exact row ID | Source/declaration comparison | Status and gap |
|---|---|---|
| `global api-C_PingSecure-ClearPendingPingOffScreenCallback-046` | Source :46 lists name only; no exact current clear declaration found in generated cache. :170–171 / :211–215 clears shared slot, zero returns; nil/idempotence inferred in spec :12. | audit-pending; shared-slot clear candidate, historical arity/policy/native consumer absent. |
| `global api-C_PingSecure-SetPendingPingOffScreenCallback-047` | Cache `PingManagerSecureDocumentation.lua:159–167` says `HasRestrictions = true`, `SecretArguments = "AllowedWhenUntainted"`, nonnil callback; :308–310 defines zero-argument callback. New setter matches closure decoding/function requirement/zero returns and all-arg auth. Cache namespace :6 says `Environment = "SecureOnly"`; simulator deliberately accepts tainted public calls and enforces no restrictions. | audit-pending; authentication subset candidate only; restriction/environment conformance FAIL against cache, historical epoch unresolved. |
| `global api-GetSecurePendingButtonCallback-055` | No exact cached declaration located; source only publishes name. :150–158 and :219–224 returns live button closure/nil as one value. | audit-pending; inferred storage read, not native button action. |
| `global api-GetSecurePendingPingOffScreenCallback-056` | No exact cached declaration; :145–147 and :219–224 returns live shared ping closure/nil, one value. | audit-pending; inferred live read. |
| `global api-GetSecurePendingToggleRunCallback-057` | No exact cached declaration; :166–168 / :219–224 returns live run closure/nil, one value. | audit-pending; inferred live read. |
| `global api-SetSecurePendingButtonCallback-059` | No exact cached declaration; :150–152 / :205–208 stores any Lua value, including nil, zero returns. | audit-pending; inferred closure storage, malformed/security/native invocation unspecified. |
| `global api-SetSecurePendingPingOffScreenCallback-060` | No exact cached declaration; :116–118 uses unchanged generic storage, not new namespace validation/authentication. | audit-pending; inferred shared-slot write; namespace secret policy must not be imputed to this global. |
| `global api-SetSecurePendingToggleRunCallback-061` | No exact cached declaration; :160–162 / :205–208 generic storage, zero returns. | audit-pending; inferred closure storage. |

State is the addon-visible `_G.__wow_ping_secure_callbacks` table (`src/c_api/c_ping_secure.rs:14,227–245`), created/recreated when the current value is not a table. That is neither explicit host-owned callback state nor native pending-input production. Direct quote: “callbacks are accepted and retained as inert compatibility state” (:5). The proof suite rejects constants but **can pass this shim**. This is an in-scope proof-standard failure, not a demand to implement every unrelated ping-wheel feature. Existing Lua-state replacement behavior is a retained alternate/recovery path, not host-state proof.

### B14–B16: helper, event and colors

| Exact row ID | Source/declaration comparison | Status and gap |
|---|---|---|
| `prose-undated-007` | Source :7 says money API uses “embedded atlases/MoneyFormatter” and removed SetTooltipMoney usages. Actual cache helper `Blizzard_GameTooltip/Mainline/GameTooltip.lua:320–322` selects red/highlight and delegates; formatting/append tested. Removal of all old usages not tested. | audit-pending; coin text/color/append candidate, not whole prose. |
| `global api-GameTooltip_AddMoneyLine-058` | Source :58 name only. Cached Lua function has `(self, rawCopper, useRedLineColor)` and no explicit return; omitted third arg is ordinary Lua nil. No generated typed/nilability/secrecy declaration located. Test uses actual loaded vendor helper, not prefix bootstrap. | audit-pending; loaded-helper candidate after running; malformed/secret/localization/layout policies unproved. |
| `prose-undated-009` | Source :9 specifies status list for “all boss units engaged” with named nonsecret fields. Test proves public detached fixture rows in tainted listener, not automatic exhaustive engagement history. | audit-pending; explicit supplied-list snapshot candidate; automatic/exhaustive tracking incomplete. |
| `events-ENCOUNTER_END-163` | Source :163 adds status payload. Cache `EncounterInfoDocumentation.lua:38–50`: synchronous, six nonnil payloads, final table of EncounterUnitStatus; :100–107: number/string/number fields. `admin_encounter.rs:20–36` appends fresh validated list and :46–55 roots during dispatch. Test has exact six args and public fields. | audit-pending; success-only supplied snapshot candidate; no defeat lifecycle, engaged-state producer. |
| `prose-undated-010` | Source :10 includes configured text/timeline colors, alpha, 5-second color-change event. Tests cover RGBA copying/live edits only. No timer trigger, notification, or real catalog→timeline instance mapping. State remains Lua `C_EncounterEvents._state.colors` (`encounter_events.rs:231–259`). | audit-pending; alpha round-trip subset, not host-backed triggers/notification. |
| `global api-C_EncounterTimeline-GetEventColor-033` | Cache `EncounterTimelineDocumentation.lua:81–97`: valid timeline event required, `SecretArguments = "NotAllowed"`, encounter outputs secret, arguments eventID plus nullable overrideTrigger, one nonnil ColorMixin. Strict old producer takes catalog ID only, returns four plain numbers, ignores extras, defaults missing components to white (`encounter_events.rs:51–74`). | audit-pending; legacy tuple test is simulator regression evidence, NOT cached contract coverage. |

Related color declaration mismatch is explicit, not merely missing tests: cache `EncounterEventsDocumentation.lua:11–25` declares getter `(encounterEventID, trigger)` with AllowedWhenUntainted and one nullable ColorMixin; :106–116 declares setter `(encounterEventID, trigger, color)` with NotAllowed. Existing provider :112–145 has one-input getter and two-input setter; missing getter color returns **zero** values, not a demonstrated one-nil return. No ColorMixin is installed by `copy_color_table` (:296–304). No all-argument secret authentication/rejection exists there. The cache's spelling is **NotAllowed**, not NeverSecret; under the requested never-secret rejection standard it must reject wrapped arguments for every caller. Secret extras are simply ignored by the legacy timeline getter, a concrete missing rejection boundary. Cache may be later: do not silently retrofit its signature into 12.0.7, but also do not label contradictory tuple assertions cache conformance. Spec `docs/specs/p1207-b16-colors.md:28` appropriately records this gap.

## Authentication order: every changed producer

| Changed entry point | Observed ordering | Verdict |
|---|---|---|
| Namespace `C_PingSecure.SetPendingPingOffScreenCallback` | Read arg1 without validation; unwrap arg1; unwrap stack entries base+1 through top (all extras); require Function; only then access/create callback storage and write (:121–142). | PASS for AllowedWhenUntainted ordering under retail-12-0-7. No transient slot mutation before denial. No SecureOnly/restriction enforcement. |
| `C_DurationUtil.CreateManualClock` | Unwrap arg1, nil→zero or finite-number validation, then ensure metatable/allocate/push host object (`duration_clock.rs:15–25`). Extras ignored, not authenticated. | No factory SecretArguments policy in cached declaration; cannot claim all-argument auth or NeverSecret parity. Its secret initial-time acceptance is an inferred extension needing explicit policy labeling, not cache evidence. |
| Manual SetTime / AdvanceTime / RewindTime | Authenticate **every** stack argument first (:117–121); unwrap input and receiver again; validate number; validate/read ManualClock receiver; compute result; reject nonfinite result; write host time; zero returns (:124–139). | PASS for declared AllowedWhenUntainted before validation/mutation, including extra arguments. Input/receiver decoding occurs after complete authentication. |
| Manual GetTime | Downcast receiver, read live time, push number (:105–108); no secret/extras authentication. | No explicit secret policy declared; do not invent one. |
| Manual ResetTime | Validate/downcast receiver via write_time, write 0, return none (:155–158,68–81); no secret/extras authentication. | No explicit secret policy declared; resetting is not a declared AllowedWhenUntainted mutator. |
| Duration SetClock | Authenticate all supplied arguments; unwrap clock; validate duration receiver; authenticate existing secret timing; store decoded clock (:298–306). | PASS order; FAIL clock type validation against cached LuaDurationClock. Wrapped receiver is authenticated but original stack receiver is still passed to require_duration, so do not claim wrapped-receiver acceptance. |
| Duration HasStarted / IsActive | query enters authenticate_activity_modifier first; unwrap every argument; unwrap modifier; accept nil/0/1; only then read timing wrappers and clock (`core.rs:315–351`). | PASS all-arg authorization precedence. Receiver itself is not validated as a DurationObject by this query path; unchanged read_slot/read_number behavior supplies zeros for invalid proxies. No strict receiver-conformance claim. |
| New clock userdata `__index` | Read receiver/key; key “time” downcasts/reads host state, otherwise metatable method lookup (:89–103). | Internal read protocol, not a declared C API secret-input producer; no script-visible mutable clock storage. |

No producer changed in 447d45d7c. Its tests cannot establish the missing color authentication: `encounter_events.rs:112–145` validates IDs/types without unwrap_secret, and legacy timeline getter :51–55 never inspects extras. Encounter snapshot helper performs structural validation and plain-value copying (`admin_encounter/unit_status.rs:30–82`); it is an A_Admin fixture input, not an AllowedWhenUntainted native API declaration. Money helper is unmodified vendor Lua and must not be assigned an invented secret policy.

**INFERRED labeling gap:** `docs/specs/duration-core.md:54` clearly labels numeric clock choices “INFERRED”; pending spec :11–21 labels legacy nil/arity, function-only, clear idempotence and ignored extras. Changed producer code does not mirror all of these markers: `duration_clock.rs:1` says only “numeric policies are documented as inferred,” :14 describes the undocumented extension without the required INFERRED marker; ping authentication :132–142 has no INFERRED comment for extra-argument ignoring or function-only validation. Predicate nil acceptance and shared-profile validation changes are not specifically INFERRED-marked in code either. This matches the same spec-vs-producer labeling issue master explicitly corrected for its own producers; no new source policy may be credited from spec prose alone.

## Master integration and risks of merging today

Observed master log (base 50d390688 excluded):

1. `e4fde2ceb` — Gate retired 12.0.7 natives and prove cached forwarding.
2. `9d40163da` — Authenticate 12.0.7 assets and numeric invites; model delve titles.
3. `54d237a91` — Mark inferred argument and output policies on 12.0.7 producers.
4. `9ce93bb12` — Keep retired 12.0.7 namespace members absent on ordinary lookup.
5. `7702befe8` — Record 12.0.7 round 2 proof and reviews.

The changed-file intersection from base is **only `src/c_api/mod.rs`**: master's insertions register battle-net-invite/delve modules near :46/:68; r1 adds duration_clock near :183. No overlapping edit hunk identified. Read-only three-tree inspection exit 0 is recorded above, but exit 0 of legacy merge-tree alone is not a conflict-proof acceptance gate. No merge/cherry-pick/index changes were performed. Source comparison finds no textual collision; compilation/integrated behavior remain unproven.

- **startup_globals.rs:** r1 does not edit it; master's numeric InviteFriend/no-fabricated-friend assertions, nil delve miss, epoch-split removals, and old-profile control remain intact when commits are integrated rather than files replaced. Existing startup clock test :89–98 uses GetTime/SetTime/AdvanceTime/RewindTime/ResetTime; it does not expect a table or raw writable field, so the new userdata representation does not contradict those specific assertions.
- **Known existing failure persists:** startup test :111 expects `durationObject:HasExpired()` to return false for a new zero-span duration, whereas core.rs:340–345 returns `cfg!(feature = "retail-12-0-5")` for zero spans. r1 does not alter this branch. Master record `data/patch-api/evidence/12.0.7-session-2026-10-03/round-2-record.md:22–23` records the existing failure and missing strict/older-profile runs. Treat those as reconstructed prior reports, not recoverable current execution proof. Do not describe this branch as all-green or blame this pre-existing mismatch on r1.
- **State files:** r1 touches neither `src/lua_api/state.rs` nor `src/lua_api/state/sim_state.rs`; master's new invite/delve state fields and their initializers have no competing edits. Do not copy r1's older complete files over master: that would discard r2 behavior. There is no evidence the new duration module reads any of r2's changed state fields.
- **Duration core:** master has no duration-object/clock edits after base. New clocks preserve the read-only “time” protocol used by core's `state.gettable` (:152–167); method-based public clock consumers inspected here do not require a table. Two deliberately invalid clock tests are adjusted as documented. Shared, ungated provider now changes **all profiles**, not only 12.0.7: userdata/nonwritable state and required finite mutation input replace formerly writable/defaulting tables. Earlier-profile consumers permitting nil mutation args/raw writes or replacement of clock methods would change behavior. No cross-profile runtime proof exists here.
- **Namespace removal fallback:** r1 changes no removal producer/bootstrap path, so master ordinary-lookup removal correction is not reverted by these commits. Branch bootstrap `runtime_surface_bootstrap.lua:66–83` can fabricate fallback functions for non-retired missing names; none of these new tests prove API absence using rawget. Positive callback/clock behavior generally defeats a nil-returning fabricated function; it does not defeat the existing compatibility providers. Keep master removed-key work intact.

Current in-scope merge risks, ranked:

1. Treating proposed tests/specs as accepted row coverage would violate the requested standard: callbacks and warning colors retain Lua-state compatibility producers and fallback paths; encounter data is fixture-supplied, not engaged-host state. No current run proves the new tip compiles or behaves as predicted.
2. Cached type/signature/secrecy mismatches remain for SetClock/GetClock, secure ping restrictions, and timeline colors. Color tests lock in a tuple that is explicitly different from current cached declaration. Historical epoch pinning is required before calling that native parity.
3. Ungated clock representation/validation changes can affect existing Classic/earlier consumers. No strict/older-profile proof was run, and existing startup failure persists.
4. Inferred policies are not fully marked in changed producer code. This is not automatically covered by master 54d237a91, which touched different producers.
5. Textual merge risk is low: independent mod.rs additions, no competing state/startup/duration hunks. That does not prove compile or profile integration.

## Recommended row accounting and overall verdict

**REJECT against the stated audit/merge-ready proof standard.** This is not a claim that authentication implementation is wrong or that all development commits should be discarded. The namespace setter and the six declared duration mutation/binding/activity entry points have the required all-argument-before-validation ordering. New clock time is genuinely host-owned and public tests are behavior-based. These are useful development slices.

Acceptance is nevertheless unsupported: host-backed producers required by the task are missing in callback/color/snapshot coverage; important cached contracts deliberately remain unmatched; literal INFERRED producer markers are incomplete; no execution evidence was recovered for these 25 tests or integrated code. Specifications themselves repeatedly say “Source-reading predictions only; no test execution or accepted coverage” (three p1207 B14/B15/B16 specs, :23/:24/:23 respectively). That candor prevents interpreting them as false completion claims, but does not turn source predictions into proof.

**Per-row status:** retain `audit-pending` for all 24 touched source rows listed above. Do not assign `partial-development-green` without real targeted GREEN evidence. After execution, promote only narrowly named capabilities (host manual time; authenticated mutators; loaded money formatting; explicit public detached snapshots) where the agreed host-state/producer standard is actually met. Callback storage and legacy color round trips require a user-approved bounded exception or a modeled producer before coverage promotion; native engaged tracking, ping restrictions, 5-second notification, and historical cache signature differences remain open. No ledger edits were made.

## Proof ledger / limits

- Read `/home/osso-test/AgentConfig/skills/verify/SKILL.md` first; artifact-mode role followed directly, no delegation.
- `git show 316973443`, `git show 447d45d7c`, `git show 97055bf28`: source/commit inventory inspected; focused file reads completed truncated displays. These are artifact evidence only.
- `git log --oneline 50d390688..master`, `git diff --name-only 50d390688 master`, focused master startup/mod diffs: source integration comparison at master 7702befe8 and branch 97055bf28.
- `git merge-tree --trivial-merge 50d390688 7702befe8 97055bf28`: read-only tree inspection, exit 0; no merge/index/worktree mutation; not a compilation or runtime gate.
- Local generated declarations, retained source, all 24 ledger rows, B06/B07/B12–B16 plan entries and triage were read. Current cache is not historical 12.0.7 evidence.
- Executable tests/build/simulator verification: **NOT RUN**, as explicitly required. No RED/GREEN count invented; master reconstructed counts are attributed reports only.
- No repository writes, cargo/test/build/simulator invocations, agents, model CLIs, operational mutations, or online research. Only this authorized persistent report was written incrementally.
