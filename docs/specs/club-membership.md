# Club membership

`src/c_api/` owns opaque member identity and community membership management. The cached retail `ClubDocumentation.lua` defines string-like opaque member selectors, privilege-gated management and synchronous member events. [Implementation](../wiki/systems/club-membership.md).

## What it must do

- [x] Return stable opaque IDs from member lists, member info and message authors; reject numeric member selectors.
- [x] Update roles/notes and remove members before synchronous callbacks observe state.
- [x] Preserve required/unique owner, enforce assignable and kickable role lists.
- [x] Store pending invitations separately from members; honor invite, revoke and query privileges.
- [x] Deny restricted/uninitialized mutations without state changes or events.
- [x] Accept explicit host member changes and publish ADDED, PRESENCE_UPDATED, REMOVED, ROLE_UPDATED and UPDATED with the same ID used by getters.

## How it works

- [Club model and inferred policies](../wiki/systems/club-membership.md).

## Implementation inventory

- `src/c_api/club_model.rs`: club/member/invitation state and role policy.
- `src/c_api/club_members.rs`: Lua getters and management/event boundary.
- `src/c_api/c_club.rs`: registration and guild streams/messages.
- `src/c_api/club_inputs.rs`: synchronous host member snapshots and departures.
- `src/lua_api/state{,/sim_state}.rs`: stores club state.

## Tests asserting this spec

- `tests/club_membership.rs`: opaque identity, creation and management behavior.
- `tests/c_club_probes.rs`: guild-club getters and message behavior.
- `tests/p1207_gap_closures.rs`: BattleTag requests use returned member IDs.

## Known gaps (current cycle)

- [ ] Unrelated baseline chat filter failure remains: `c_api_surface::chat_info_no_state_defaults_are_not_c_api_temporary_shims` (`tests/c_api_surface.rs:602`). It fails on untouched master `a55b2509c` and on this branch; excluded parallel chat work is not modified.

## Out of scope

- Live community service/network transport, persistence and native server validation are absent.
- Coverage JSON reconciliation and unrelated API work are excluded by task authorization.

## Development proof — 2026-10-05

Baseline filters ran before edits with HEAD and master both `a55b2509c`. Final simulator code `ce3dd9170`; invitation restriction fixture strengthened at `5d2bcc729`. Later documentation changes do not invalidate code proof.

All Rust commands ran in this worktree with `python3 scripts/build-host.py --build-host local --test --test integration <filter> -- --nocapture` unless noted.

| Filter | Master baseline | Final code | Result |
|---|---:|---:|---|
| `club` | 33 passed | 45 passed | PASS; includes all 12 new member-model tests |
| `guild` | 191 passed | 192 passed | PASS |
| `communities` | 11 passed | 11 passed | PASS |
| `chat` | 157 passed, 1 failed | 157 passed, same 1 failed | Pre-existing failure above |
| `bnet` | 3 passed | 3 passed | PASS |
| `patch_12_0_7_publication_sweep` | Not rerun on master | 1 passed; 174-row publication inventory | PASS at `91f02ce4e`, `--test-threads=1`; later GUID/test changes do not alter publication registration |

`club_membership` independently passed 12/12 at `91f02ce4e`; the final club filter reran these after the GUID change. The strengthened `club_restrictions_and_initialization_deny_privileged_mutations` passed 1/1 at `5d2bcc729`. Initial opaque-ID/creation RED: 0/2 passed on the original implementation. An intermediate Guild rank-menu regression was reproduced by guild/communities filters, traced to absent `guid`, and fixed in the model; the specific cached-UI test passed before final filters.

`cargo fmt --check`, `python3 scripts/build-host.py --build-host local --check` (Cargo check), and separate local debug build passed. Startup ran after that build using `--build-host local --run --no-build -- --no-addons --no-saved-vars lua-errors`, capped at 90 seconds: exit 0, `[]`, 0 unique / 0 occurrences. The binary timestamp was newer than the final model source edits. No simulator compiler warnings remain; six pre-existing vendored iced manifest lint-name deprecations remain untouched.

Readability metrics for the four model/boundary files: maximum function cognitive complexity 8, cyclomatic complexity 9; no warning suppressions. Boundary extraction and name-keyed guild projection avoid duplicated management decisions and quadratic roster matching.

### Row outcomes (coverage ledger deliberately unchanged)

| Source ID | Observable proof |
|---|---|
| `events-CLUB_MEMBER_ADDED-158` | Host membership arrival publishes the returned opaque ID; callback sees member and cleared pending invitation. |
| `events-CLUB_MEMBER_PRESENCE_UPDATED-159` | Host presence 2 publishes `(clubId, memberId, 2)`; callback getter agrees, including guild projection. |
| `events-CLUB_MEMBER_REMOVED-160` | Kick/host departure publishes opaque ID; callback lookup is nil and surviving IDs/history remain stable. |
| `events-CLUB_MEMBER_ROLE_UPDATED-161` | Assign/host role change publishes opaque ID and new role; callback getter agrees; owner transfer is atomic. |
| `events-CLUB_MEMBER_UPDATED-162` | Changed note/name publishes opaque ID; callback sees updated fields. |

These are bounded simulator/host-input behavior claims, not observations from a live community service. String representation, guild name identity/GUID synthesis, role hierarchy/owner transfer, local synchronous completion, change-only events and explicit restriction inputs are INFERRED in code and documented in the linked wiki.
