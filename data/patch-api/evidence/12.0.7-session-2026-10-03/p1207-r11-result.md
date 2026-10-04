
## Step 1: active goal
Implement only row 048 with inferred contract, host state and behavioral tests in p1207-r11. Completion: RED unchanged producer, GREEN, intersecting quest-hub tests, formatting/check, one commit. Excludes cache/vendor/wiki/coverage edits and push/merge/deploy/PR/agents.

Step 2: worktree clean at a4643e9d3; reading sibling, caller and related declarations. Tool grep unavailable; filesystem text scan used.

## Step 3: evidence / inferred choices
1. QuestHubInfoDocumentation.lua: `Name = "IsAreaPOICurrentlyRelatedToHub"`, `SecretArguments = "AllowedWhenUntainted"`, `{ Name = "areaPoiID", Type = "number", Nilable = false }`. INFERRED required numeric selector and secret policy; authenticate all extras before validating.
2. Entire profile cache *.lua scan: zero exact getter occurrences, no caller or exact declaration; no caller field shape to honor.
3. Retail AreaPoiInfoDocumentation.lua:GetDragonridingRacesForMap: `Documentation = { "Returns all area POIInfos flagged as dragonriding races for the given map." }`; `{ Name = "areaPoiIDs", Type = "table", InnerType = "number", Nilable = false }`. INFERRED race elements are numeric race POI identifiers, not invented tables/fields. Per-POI ordered numeric race records are Vec<i32>.
4. Handoff row048: "replace the probe with a per-environment POI/content input and live Rust getter once schema is known"; "No cached consumer of this exact getter was located." Owner explicitly authorizes inferred contract instead of prior block. INFERRED empty/unknown defaults, host order, fresh snapshots, and environment isolation are simulator policies, not native proof.

Step 4: seven behavioral tests written before producer change. Added only host-map scaffold so RED compiles. Producer still unchanged. RED command: build-host.py --build-host local --test --target-dir b100 --test integration p1207_dragonriding_races -- --test-threads=1; base a4643e9d3 + test/scaffold.
RED finished exit 101; log r11-RED-p1207_dragonriding_races.log.

Step 5: RED 1 PASS / 6 FAIL, expected missing content/validation. Implemented live numeric race POI snapshots and full pre-validation authentication; spec quotes evidence. GREEN proof scope: a4643e9d3 + current four Rust files/spec.
Step 6: GREEN invocation finished exit 101; log r11-GREEN-p1207_dragonriding_races.log.
Step 7: initial GREEN 6 PASS / 1 FAIL in secret test. Added assertion diagnostics only; producer unchanged. Narrow diagnostic filter secure_secrets_are_accepted_and_all_tainted_secrets_precede_validation.
Step 8: diagnostic exit 101; captured once.
Step 9: diagnostic proves VM denial text is "table security operation requires an untainted caller" (not "secret"). Corrected test expectation; producer unchanged. Previous six passing cases still valid; rerun seven because test changed.
Step 10: final GREEN exit 0, log r11-GREEN-p1207_dragonriding_races-final.log.
Step 11: GREEN 7 PASS / 0 FAIL, no warnings. Spec behavioral bullets marked proven (still INFERRED). cargo fmt completed; existing QuestHub probes located in loader startup_globals. Readability manually audited changed Rust: small getter/selector, no new suppressions or nested control; no violations.
Step 12: formatted coherent implementation committed dc697d7f5d1d65695989d60ed6cc440838c1a357; exactly five authorized files. GREEN proof remains valid after formatting (no semantic change). Final formatting/check and narrow integration map probes follow. Existing exact QuestHub probes are src/loader unit tests, outside prescribed --test integration command; report as unexecuted rather than widening runner.
Step 13: proof ledger checked; no earlier proof for intersecting filter c_area_poi_probes, revision dc697d7f5.
Intersecting c_area_poi_probes: exit 0, r11-GREEN-c_area_poi_probes.log.
Step 13: proof ledger checked; no earlier proof for intersecting filter startup_world_map_provider_list_apis_are_iterable, revision dc697d7f5.
Intersecting startup_world_map_provider_list_apis_are_iterable: exit 0, r11-GREEN-startup_world_map_provider_list_apis_are_iterable.log.
Step 14: cargo fmt --check exit 0, revision dc697d7f5.
Step 15: final check, no prior cargo check proof; revision dc697d7f5, default feature set, four jobs, exclusive b100 target.
Step 16: cargo check exit 0; r11-GREEN-cargo-check.log.

## Final result — 2026-10-04
Commit: `dc697d7f5d1d65695989d60ed6cc440838c1a357`. Branch p1207-r11; clean worktree `/home/osso-test/.worktrees/wow-ui-sim-p1207-r11`. One commit; no push/merge/deploy/PR/model/agent invocation.

Changed files:
docs/specs/quest-hub-dragonriding-races-12-0-7.md
src/c_api/c_quest_hub.rs
src/lua_api/state.rs
src/lua_api/state/sim_state.rs
tests/p1207_dragonriding_races.rs


### Proof ledger
All test invocations used explicit cwd `/home/osso-test/.worktrees/wow-ui-sim-p1207-r11`, `CARGO_BUILD_JOBS=4`, `BUILD_HOST_SCRIPTS=/home/osso-test/Projects/world-of-osso/game-engine/scripts`, and argv:
`python3 scripts/build-host.py --build-host local --test --target-dir /home/osso-test/.cache/wow-ui-sim-target-b100 --test integration FILTER -- --test-threads=1`.
- RED `p1207_dragonriding_races`: unchanged producer at a4643e9d3 plus tests/host-map scaffold, 1 PASS / 6 FAIL. Expected missing content/validation; empty default already passes.
- Initial GREEN: 6 PASS / 1 FAIL, test expected wrong VM error substring. Diagnostic: 0 PASS / 1 FAIL, message "table security operation requires an untainted caller". Corrected expectation; no producer change.
- Final GREEN `p1207_dragonriding_races`: 7 PASS / 0 FAIL. Current commit differs only by formatting/spec checkbox marking; behavior proof valid.
- `c_area_poi_probes`: 8 PASS / 0 FAIL at current commit.
- `startup_world_map_provider_list_apis_are_iterable`: 1 PASS / 0 FAIL at current commit.
- `cargo fmt`: exit0 before commit; `cargo fmt --check`: exit0 after commit.
- `CARGO_BUILD_JOBS=4 cargo check --target-dir /home/osso-test/.cache/wow-ui-sim-target-b100 --jobs 4`: exit0 at current commit.
- No warnings in recorded test/check logs. Readability manual audit: no violations in changed Rust.
- Exact existing QuestHub smoke cases `test_patch_12_0_7_safe_global_bridges` and `test_patch_12_1_safe_global_bridges` are library unit tests in src/loader/tests/wow_api_globals/startup_globals.rs, not tests/integration; not executed because mandated runner restricts invocations to integration. No unfiltered, alternate-profile or startup execution performed.

### Coverage and recommendation
| Behavior | Proof |
|---|---|
| One empty default result; ordered independent POI inputs; unknown POI | New behavioral tests PASS |
| Live replacement/removal; detached snapshots; environment isolation | New behavioral tests PASS |
| Missing/invalid selector rejected; secure secret accepted; tainted secret/extra denied before validation | New behavioral tests PASS |
| Native schema/security parity | Unverified; INFERRED |

Recommended row status: **bounded-coverage under explicitly INFERRED contract**, not native parity. No coverage JSON edited.
Merge risk: localized implementation risk low; compatibility risk moderate because exact getter declaration/caller/native populated output is absent. Numeric race POI elements follow related map getter, not an authoritative exact schema. Signed-i32 numeric domain and ignored public extras are also inferred; real getter could differ. Exact library smoke probes and older-profile execution remain unproved. No incompatible cached caller found.
