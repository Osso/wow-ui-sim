# Round 100 result

Both slices integrated on `round-100` in `/home/osso-test/.worktrees/wow-ui-sim-b100`, based on `ce418cbe46049c28ed7798c67a929a0dbb6e4354`. Worktree clean. No push, merge, agents/models, vendor edits, or other-worktree modifications.

## Commits and files

**A — `c9b0aa636f49866be2c6eda3316421d666db0b71`** — Route roster names through shared identity secrecy.

- `docs/specs/instanced-identity.md`
- `docs/specs/model-unit-identity-guard.md`
- `src/lua_api/globals/group_queries.rs`
- `tests/model_set_unit_identity_followups.rs`
- `tests/raid_roster_identity_followups.rs`

**B — `f3a9457d15e9b7225def14ad0e06c61da7b633e4`** — Return selected macro units and model viewed outfit state.

- `docs/specs/outfit-action-command.md`
- `docs/specs/target-marker-macro-command.md`
- `src/c_api/c_transmog_outfit_info.rs`
- `src/c_api/c_transmog_outfit_info/viewed.rs`
- `src/lua_api/globals/security/cmd_option.rs`
- `src/lua_api/globals/security/mod.rs`
- `src/lua_api/globals/unit_misc.rs`
- `src/lua_api/state.rs`
- `src/lua_api/state/sim_state.rs`
- `src/lua_api/workarounds/temporary/transmog_outfit_slot_defaults.rs`
- `tests/cmd_option_selected_unit.rs`
- `tests/pending_transmog_cost.rs`
- `tests/transmog_outfit_info.rs`
- `tests/viewed_outfit_selection.rs`

Both messages end with:
`Co-Authored-By: Claude Opus 5.5 (1M context) <noreply@anthropic.com>`

## Commands and proof

All commands below ran with explicit isolated-worktree cwd. Exact test command, substituting the literal filter from each row, one invocation per filter:

```text
CARGO_BUILD_JOBS=6 BUILD_HOST_SCRIPTS=/home/osso-test/Projects/world-of-osso/game-engine/scripts python3 scripts/build-host.py --build-host local --test --target-dir /home/osso-test/.cache/wow-ui-sim-target-b100 --test integration FILTER -- --test-threads=1
```

A RED/GREEN used `identity_followups::`, covering the two identity modules together. B RED used `cmd_option_selected_unit::` and `viewed_outfit_selection::` separately. GREEN controls used each module filter below separately. Counts are **passed/failed**, not inferred expectations:

| Module/filter | RED | A GREEN | Final integrated GREEN |
|---|---:|---:|---:|
| `model_set_unit_identity_followups::` | 6/0 | 6/0 | 6/0 |
| `raid_roster_identity_followups::` | 0/5 | 5/0 | 5/0 |
| `cmd_option_selected_unit::` | 0/2 | — | 2/0 |
| `viewed_outfit_selection::` | 0/3 | — | 3/0 |
| `mouse_tm_commands::` | — | — | 5/0 |
| `outfit_action_command::` | — | — | 7/0 |
| `patch_12_0_5_outfit_catalog::` | — | — | 5/0 |
| `transmog_outfit_info::` | — | — | 1/0 |
| `pending_transmog_cost::` | — | — | 10/0 |
| `blizzard_map_lane::` | — | — | 13/0 |
| `blizzard_restricted_addon_environment_loads::` | — | — | 10/0 |
| `security_api::` | — | 47/0 | 47/0 |
| `security_state_drivers::` | — | — | 13/0 |
| `cast_events_identity::` | — | 10/0 | 10/0 |
| `instanced_identity::` | — | 6/0 | 6/0 |
| `unit_token_identity_secrecy::` | — | 5/0 | 5/0 |
| `raid_roster_unknown_name::` | — | 4/0 | 4/0 |
| `admin_party_api::` | — | 28/0 | 28/0 |
| `tooltip_basic::` | — | 62/0 | 62/0 |
| `tooltip_item_spell::` | — | 88/0 | 88/0 |
| `widget_methods_model::` | — | 17/0 | 17/0 |

Final unique selected tests: **347 passed, 0 failed**. The repeated parser control adds no unique-test credit. No unfiltered suite run.

RED scope: A tests only on base; B tests + viewed-state field/default on A commit. Producers withheld until observed behavioral failures. A failures demonstrated unconditional secrecy, equal-literal poisoning and stale cached secrecy after classification clearing. B failures demonstrated missing selected-unit return and no-op/Lua-backed viewed selection.

GREEN scope: all implemented Rust/test changes; only spec prose changed before commits. Final `cargo fmt --check` and `CARGO_BUILD_JOBS=6 cargo check --target-dir /home/osso-test/.cache/wow-ui-sim-target-b100` both exited **0**, no final compiler warnings. `cargo fmt` ran before each commit. Manual changed-line readability audit completed.

Expanded exact commands, revision/scope ledger and historical results: `round-100-proof-ledger.md` beside this report. Logs: `a-red.log`, `a-green-*.log`, `b-red-*.log`, `b-green-*.log`, `cargo-check.log` in the requested scratch directory.

## Anchors and existing expectations

- All **14 A / 25 B** old anchors matched exactly once against the newer base. Existing files changed by exact replacements, never staged whole-file overwrite. B99 SimState additions, UnitTreatAsPlayerForDisplay changes and ce418cbe4 profile secrecy/security expectations preserved.
- A roster index 1 resolves `player`; later indices resolve existing `party(n-1)` identities. Shared predicate and trusted host-secret wrapper replace interned-string marking. Public UNKNOWN and twelve-result tuple retained. Legacy marker path preserved, not runtime-tested here.
- B selected-unit nil/default policy stays explicitly inferred; Rust macro/state-driver resolver retains implicit target. Real entire cached SlashCommands file loaded unchanged, registered `/tm` handler called with distinct target/focus identities.
- Staged taint test corrected before RED: create secret in secure code, then pass it from tainted closure. Otherwise rejection would occur at secretwrap instead of the setter. Setter authenticates its sole declared argument before validation/mutation.
- A introduced an unused marker re-export warning, exposed by B RED build. B removes that alias and redirects the sole legacy importer through parent-visible secret_values; behavior unchanged, no suppression.

**Existing assertions weakened: none.** A changed no existing tests. `tests/transmog_outfit_info.rs`: old fixture rawset viewed ID 7 → `ChangeViewedOutfit(7)`; active toggle/clear independence assertions unchanged. `tests/pending_transmog_cost.rs`: old rawset viewed ID 91 → `ChangeViewedOutfit(91)`; message now says API-selected outfit, preserving cost-read-only, slot, lock, catalog and sheathe assertions. Existing first-return parser/security expectations unchanged.

Additional parser/state-driver modules discovered by local Python file scanning (rg unavailable): blizzard_map_lane, blizzard_restricted_addon_environment_loads, security_api, security_state_drivers; all included above.

## Startup and deviations

Attempted exactly once:

```text
CARGO_BUILD_JOBS=6 BUILD_HOST_SCRIPTS=/home/osso-test/Projects/world-of-osso/game-engine/scripts WOW_SIM_CASC=0 python3 scripts/build-host.py --build-host local --run --target-dir /home/osso-test/.cache/wow-ui-sim-target-b100 -- --no-addons --no-saved-vars lua-errors
```

Exit **2**, exact error:
```text
build-host.py: error: unrecognized arguments: --target-dir /home/osso-test/.cache/wow-ui-sim-target-b100
```
Startup skipped as explicitly permitted. No `[]` startup proof claimed; full stderr saved in `startup.log`.

Initial two-filter test invocation was rejected: `error: unexpected argument 'raid_roster_identity_followups::' found`; corrected to shared `identity_followups::` before build. Initial Pyrun run helper used canonical cwd despite session cwd: read-only Git calls and unsuccessful `git switch -c round-100` / `git switch round-100` attempts ran there. No changes succeeded. Every subsequent command used explicit isolated cwd.

**No slice left out.** No native/GUI/older-profile/full-suite proof, page-accounting promotion or acceptance-review claim added.

## Source-claim coverage

| Source row/claim | Proven | Not proven |
|---|---|---|
| widgets-PlayerModel-SetUnit-534 | Actual PlayerModel, DressUpModel, CinematicModel, TabardModel and ModelScene secret denial, exact nil/false/true results, binding preservation/recovery; tooltip content/callback control | 3D/native model-load success, optional-argument policy, older profiles; row accounting remains partial |
| prose 04-10-197..199, roster part | Cached roster uses shared player/party GUID policy, instance group/player exemptions, explicit classification override, equal-literal isolation, retained-secret lifetime, cache/UNKNOWN transitions and exact tuple | Native exemption precedence; raid/pet/vehicle GUID producers; automatic world acquisition. Existing instanced-identity controls cover supported visitor/mind-control context, not missing categories |
| prose 03-25-098, vendor /tm path | Explicit selected-unit second return feeds unchanged cached registered handler; focus/target destinations, default target, marker mutations and notifications | Full chat/addon-loading path, native grammar/arity certification, permission matrix |
| prose 03-25-111/122 and 03-31-165/179, viewed-outfit part | Host-owned viewed ID/query, real ID-based setter, environment isolation, authentication/taint, synchronous payload-free refresh, repeat/miss behavior, active/pending independence and updated fixtures | Native miss/repeat/disabled eligibility/pending-discard policies, persistence, appearance application, protected authority or full UI path; these policies remain inferred |
