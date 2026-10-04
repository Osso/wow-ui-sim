# Classic secrets integration result

**Commit:** `0cfcaf4cbad769693171796114ba6905eb2e6119`
**Branch:** `classic-secrets`; clean isolated worktree `/home/osso-test/.worktrees/wow-ui-sim-classic-secrets`.
**Base:** `aa29d7d7f06b14f33990c36ae736b98579b00b78`.
No push, merge, agents, model CLIs, retail build, vendor modifications, or canonical-checkout writes.

## Files changed

| Path | Change |
|---|---|
| `src/client_profile.rs` | Runtime `uses_secret_values()` predicate: false for Wrath/Mists/Era/Anniversary; true for Retail/PTR/Forever. |
| `src/lua_api/globals/security/secret_values.rs` | Guard string-marker production and loadstring closure-secret classification. |
| `src/lua_api/state.rs` | Ungate false stat-policy initializer; state-only adaptation. |
| `src/lua_api/state/sim_state.rs` | Ungate `unit_stats_restricted` field so Classic fixture can set explicit host input. |
| `tests/classic_secret_policy.rs` | Seven new behavioral tests. |
| `tests/security_api.rs` | Seven profile-aware existing expectations. |

## Anchors and adaptations

All staged changes applied through unique minimal replacements, not whole-file snapshots. Current staged anchors matched. Read current `unit_misc.rs`, `group_queries.rs`, and `real/instanced_identity.rs`: retained retail 12.0.5 explicit host GUID/map/group classification, its wrapper path, and `create_explicit_secret_party_env()` fixture. Older token-heuristic paths reach the centrally guarded marker; cached raid roster also reaches that marker. No direct identity-module changes needed.

Initial RED compile exposed a slice-related fixture issue: `unit_stats_restricted` field and initializer were epoch-gated out of Mists. Ungated only state storage/default; stat-output epoch gates unchanged. No unrelated compiler fix.

Protected-action fixture initially read addon-written event slots before testing secure recovery. Those reads propagated addon taint. Moved secure recovery before event inspection; retained exact blocked width, event payload/count, caller taint, and secure-write assertions. Diagnostic message also corrected: secure `debug.getstacktaint()` returns zero values, so eager `tostring(debug.getstacktaint())` errored. No production taint changes.

New tests ran before predicate/producers. Existing expectation edits reference the withheld predicate, so applied those dependent assertions with GREEN rather than manufacturing a compile-only RED.

## Exact commands and proof ledger

All commands ran from the isolated worktree via argv-style Pyrun helpers, without build timeouts. Captured stdout/stderr saved under the handoff scratchpad; logs inspected.

Build/test environment for every helper invocation:

```text
CARGO_BUILD_JOBS=6
CARGO_TARGET_DIR=/home/osso-test/.cache/wow-ui-sim-target-classic
BUILD_HOST_SCRIPTS=/home/osso-test/Projects/world-of-osso/game-engine/scripts
```

Initial RED command, run twice (compile error, then tests after state-only correction):

```text
python3 scripts/build-host.py --build-host local --no-default-features --features sound,gui,casc,client-mists --test --test integration classic_secret_policy:: -- --test-threads=1
```

Observed helper overrides environment target directory to isolated worktree `target`. Neither build shared canonical artifacts. Relocated those artifacts to requested cache; all subsequent test commands explicitly supplied Cargo `--target-dir`:

```text
python3 scripts/build-host.py --build-host local --no-default-features --features sound,gui,casc,client-mists --test --target-dir /home/osso-test/.cache/wow-ui-sim-target-classic --test integration FILTER -- --test-threads=1
```

`FILTER` substitutions below are exact, one invocation per row unless noted:

| Phase / log | FILTER | Passed / failed | Exit |
|---|---|---:|---:|
| Initial compile; `classic-secrets-red-attempt1-full.log` | `classic_secret_policy::` | No tests; two E0609 field errors | 101 |
| State corrected; `classic-secrets-red.log` | `classic_secret_policy::` | 3 / 4 | 101 |
| Diagnostic1; `classic-secrets-red-protected-diagnostic.log` | `classic_secret_policy::classic_secret_policy_tainted_protected_action_still_denied` | 0 / 1 | 101 |
| Diagnostic2; `classic-secrets-red-protected-diagnostic2.log` | Same exact protected-action filter | 0 / 1 | 101 |
| Event-read reordered; `classic-secrets-red-final.log` | `classic_secret_policy::` | 3 / 4; eager diagnostic error | 101 |
| **Corrected RED**; `classic-secrets-red-corrected.log` | `classic_secret_policy::` | **4 / 3** | 101 |
| GREEN; `classic-secrets-green-classic_secret_policy.log` | `classic_secret_policy::` | **7 / 0** | 0 |
| GREEN; `classic-secrets-green-security_api.log` | `security_api::` | **46 / 0** | 0 |
| GREEN; `classic-secrets-green-unit_name_secret_tokens.log` | `unit_name_secret_tokens::` | 0 / 0; compiled out | 0 |
| GREEN; `classic-secrets-green-unit_token_identity_secrecy.log` | `unit_token_identity_secrecy::` | 0 / 0; compiled out | 0 |
| GREEN; `classic-secrets-green-instanced_identity.log` | `instanced_identity::` | 0 / 0; compiled out | 0 |
| GREEN; `classic-secrets-green-admin_identity_api.log` | `admin_identity_api::` | **15 / 0** | 0 |

Corrected RED failures were exactly UnitName marker, cached roster marker, and tainted loadstring closure secrecy assertions. Stat, charge, duration/curve, and protected-action controls already passed. GREEN totals: **68 executed tests passed**. Retail-gated zero-test modules are exclusions, not behavioral proof.

Formatting/check commands:

```text
cargo fmt
cargo fmt --check
CARGO_BUILD_JOBS=6 CARGO_TARGET_DIR=/home/osso-test/.cache/wow-ui-sim-target-classic cargo check --no-default-features --features sound,gui,casc,client-mists --target-dir /home/osso-test/.cache/wow-ui-sim-target-classic
```

All exit 0. Formatting preceded GREEN and commit; format-check and Mists check ran after commit. No warnings in recorded compiler logs. Manual changed-line Rust readability audit found no violations. GREEN source equals committed source; no subsequent code changes invalidate proof. No broad unfiltered suite or redundant suite rerun.

Commit subject: `Disable secret markers and closure secrecy on Classic profiles`.
Final attribution: `Co-Authored-By: Claude Opus 5.5 (1M context) <noreply@anthropic.com>`.

## Existing expectations changed

| Security test | Classic expectation and reason |
|---|---|
| `test_secure_map_rejects_secret_keys_and_values` | Tainted loadstring function keys/values accepted: taint alone is not Classic secrecy. |
| `test_issecretvalue_tainted` | False for tainted loadstring closure. |
| `test_canaccessvalue_tainted` | True for same closure. |
| `test_canaccessallvalues_one_tainted` | True for ordinary values plus tainted closure. |
| `test_party_roster_name_is_secret_value` | Party name non-secret, accessible. |
| `test_party_full_name_marks_name_and_realm_secret` | Name/realm non-secret, bulk access allowed. |
| `test_table_containing_party_identity_is_not_accessible` | Nested ordinary identity accessible. |

Retail/PTR/Forever expectations remain their original values because predicate returns true. Retail identity wrappers/classifier, native secret checks, loadstring taint stamping, and stat/cooldown wrapping gates unchanged. Retail preservation established by source inspection only; no retail execution claimed.

## Clause coverage and remaining gaps

| Audit row | Proven | Not proven / unchanged |
|---|---|---|
| `prose-2026-04-17-213` | Mists ordinary party name/full-name/realm/GUID and cached roster payload/access; ordinary stat/charge outputs under explicit restriction flags; loadstring closure accessible but still tainted. Central marker/closure guards cover all four Classic profiles by exhaustive predicate inspection. | Entire secret system/all APIs, Wrath/Era/Anniversary execution, native-client parity, host-injected authentic wrappers, frame secret/aspect state, blanket Midnight-security disablement. |
| `prose-2026-04-17-214` | Mists pre-Midnight taint persists; insecure combat protected width write denied with exact blocked event, secure write succeeds. Manual-clock duration lifecycle, numeric/color curve interpolation, and duration curve evaluation work. | Chat/guild restrictions, combat-log payload/lifecycle, every restricted action, all associated duration/curve APIs, boolean-selection helpers, duration factories/formatters/bindings, native parity. |
| `prose-2026-04-17-215` | Same bounded runtime secrecy correction as row213 despite documentation references. | Upstream documentation correction/feedback/sweep are narrative, not completed repo work; vendor docs untouched. |

No unresolved failure within executed scoped tests/checks. Initial compiler/fixture failures corrected and retained in logs. No unrelated code repaired. Zero-test retail identity suites remain explicitly unproven under Mists; whole-row acceptance is not claimed.
