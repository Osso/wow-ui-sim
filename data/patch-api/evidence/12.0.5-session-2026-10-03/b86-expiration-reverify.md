# B86 independent re-verification — 2026-10-03

## Verdict: ACCEPT WITH QUALIFICATIONS

Authentication-ordering defect and dead-cfg finding are resolved. Fresh execution passed 9/9 expiration tests and 18/18 duration controls. Acceptance is bounded to this implementation and these fixtures, not native WoW parity or complete spec coverage. No remaining blocking defect found in the requested delta.

Qualifications: blocked/target expiration behavior and full-record/party immutability remain unasserted; older-profile runtime absence was not executed; formatting/build/CI/startup after the reorder remain independently unverified.

Reviewed HEAD `c9940ece9a02f352a94e5022b1e0e328c20fa0c6`, fixes `08c9e8bc8`, `d1bbdc8e8`, and the prior report's Verdict and sections 2, 6, 7. Read verify skill first. No repository writes, cargo, builds, formatters, agents, model CLIs, Bash, or operational actions.

## 1. Producer — PASS

`git show d1bbdc8e8` and HEAD `src/c_api/aura_duration.rs:77–93` now start:

```rust
let unit = unwrap_secret(state, stack_val(state, 1))?;
let instance_id = unwrap_secret(state, stack_val(state, 2))?;
let unit = match unit {
```

Both authentication calls precede unit matching, UTF-8 conversion, finite-number validation and aura lookup. If the first secret is denied, the second call is naturally not reached; no validation runs first. With public malformed unit and secret ID, the second authentication now produces VM denial rather than the unit-type error.

Pinned rilua `6044544/src/table_security.rs:232–256` calls `ensure_secure_caller(state)?` before `Ok(payload)`, with denial text `"table security operation requires an untainted caller"`. Public/non-secret values pass unchanged. No remaining two-declared-argument path found where a tainted caller's authentic secret reaches validation or lookup instead of VM denial. This excludes undocumented extra arguments and invalid/non-authentic wrappers.

Dead cfg: PASS. `src/c_api/mod.rs:15–16` has `#[cfg(feature = "retail-12-0-5")] pub mod aura_duration;`; `src/lua_api/globals/register.rs:92–93` likewise gates registration. Redundant positive annotations and unreachable negative reader were removed.

Duration behavior: PASS. The fix changes only expiration reader/import annotations; `GetAuraBaseDuration` and `GetRefreshExtendedDuration` still call `query_duration` → unchanged `read_public_arguments`, rejecting secrets in positions 1–3. Fresh 18/18 controls passed.

## 2. Regression assertions — PASS

`git show 08c9e8bc8` adds a valid-unit/secret-ID denial baseline, then compares malformed public units `12`, `true`, `{}`:

```lua
assert(not ok and err == denial, 'secret denial precedes unit validation')
assert(malformed ~= denial, 'public malformed unit keeps its own error')
```

Against `0d2a98596`, unit validation precedes ID authentication: malformed unit returns `"C_UnitAuras: unit must be a string or nil"`, differing from the baseline VM denial. Thus this assertion detects the old defect. Saved RED confirms that failure; fresh GREEN confirms the fixed boundary.

Comparing error values is sound for this VM and regression: `RejectExpires` first establishes nonempty string errors, VM denial has fixed text, and the public malformed-input control must differ. It avoids hardcoding denial wording and asserts externally observable precedence. It is not a universal error-origin proof: another implementation producing identical strings for distinct causes could evade it; inspected producer/VM source establishes the cause here.

Numeric GC identity: PASS. Test now asserts `"rawequal(unit, SecretExpirationUnit) and rawequal(id, SecretTimedID)"` after two full collections, plus numeric secrecy and `secretunwrap(id) == 301`. Existing tainted GC probe and public recovery remain.

## 3. Logs and fresh proof — PASS

Saved `b86-red2.log`: `"test result: FAILED. 8 passed; 1 failed; 0 ignored; 0 measured; 10056 filtered out; finished in 2.30s"`; sole failure is `tainted_query_denies_each_secret_even_before_unknown_unit_lookup`, at `"secret denial precedes unit validation"`.

Saved `b86-green2.log`: `"test result: ok. 27 passed; 0 failed; 0 ignored; 0 measured; 10038 filtered out; finished in 5.24s"`. Read full logs. Saved logs alone do not authenticate exact build revision or process exit status.

Direct-execution preconditions passed: `git diff d1bbdc8e8 HEAD -- src tests Cargo.toml Cargo.lock build.rs` returned empty stdout, exit 0. Commit timestamp `1791057599`; binary mtime `1791057703.603091` (104.603091 seconds later). Working tree was clean. These satisfy the supplied eligibility rule, not cryptographic binary provenance.

Ran, with explicit repo cwd:

```text
timeout 90 /home/osso-test/Projects/wow/wow-ui-sim/target/debug/deps/integration-8ea324359263a4d2 aura_expiration_time:: --test-threads=1
timeout 90 /home/osso-test/Projects/wow/wow-ui-sim/target/debug/deps/integration-8ea324359263a4d2 aura_refresh_duration:: --test-threads=1
```

Full outputs read; each named test emitted `ok`; stderr empty; both exit 0:

```text
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 10056 filtered out; finished in 1.80s
test result: ok. 18 passed; 0 failed; 0 ignored; 0 measured; 10047 filtered out; finished in 4.14s
```

Proof ledger: both commands cover the eligible prebuilt binary with HEAD source scope unchanged from `d1bbdc8e8`; no changes made during verification. No reruns.

## 4. Spec honesty and unchecked requirements

PASS: earlier issues are explicitly bounded. Older-profile text now says the module is `"compiled and registered only under that feature"`, rather than claiming older builds reject secrets. Blocking and `target` are named: `"a blocked record reports false; target resolves a fixed fixture list, not per-environment state. Neither is asserted by this spec's tests."` Current collector source confirms both mechanisms. Proof table records rejection, ordering RED and corrected GREEN; stale inputs-only checkpoint is removed. Known gaps retain full-record/party limitations and ignored extra arguments. Native access/restricted-output policies remain explicitly unmodeled.

No new unqualified compatibility overclaim found when requirements are read together with these limits. Opening stored-record wording is shorthand, not proof of an arbitrary-unit model. The table's cargo/warnings/formatter claims are historical reported evidence, not independently certified here. Re-verification is no longer pending after this report, but this read-only task does not update that line.

All behavioral requirements remain unchecked. Mapping below uses test names in `tests/aura_expiration_time.rs` (module prefix omitted):

| Requirement | Asserting tests | May be checked? |
|---|---|---|
| One public bool from nonzero/zero expiration | `timed_and_permanent_player_auras_return_exactly_one_public_boolean`; `AssertExpires` checks arity/type/secrecy/value | Yes, bounded to fixtures and supported public enumeration |
| Player helpful/harmful and party isolation; blocking/target qualifications | Previous test; `party_records_are_unit_and_instance_isolated` | Only player/party subsection. Whole bullet cannot: no expiration assertions for blocking/target |
| Live reads, no mutation, environment isolation | `stored_expiration_changes_are_read_live`; `queries_leave_aura_records_unchanged_and_environments_isolated` | Only live reads, player `(instance, expiration)` immutability and fixture isolation. Whole bullet cannot: other fields/party snapshots absent |
| Unknown unit/instance/nil returns one public false | `unknown_units_instances_and_nil_unit_return_false` | Yes, bounded inferred policy |
| Invalid unit/nonfinite ID errors before lookup | `malformed_arguments_error_before_lookup` | Yes for enumerated representations and malformed ID on unknown unit; ordering also established by reader source. Not exhaustive/native error parity |
| Untainted authentic secret unit/ID/both acceptance | `untainted_query_accepts_each_authentic_secret_argument_and_combination` | Yes, bounded authentic string/number wrappers and fixtures |
| Tainted secret denial before validation/lookup | `tainted_query_denies_each_secret_even_before_unknown_unit_lookup` | Yes for tested positions/unknown units/malformed public units; source establishes both-authenticate-first generally |
| Taint unchanged, no declassification, public recovery | Both caller-policy tests; GC test | Yes, bounded inspected wrappers/caller contexts |
| Rooted secret identity/secrecy survives full GC | `secret_arguments_survive_gc_without_declassification` | Yes for rooted unit and numeric ID, including newly asserted numeric identity |

Known-gaps checkbox is not a behavioral requirement: independent re-verification is now supplied; its listed unasserted cases remain gaps. Do not mark the entire composite gap resolved.

## 5. Unverified scope and merge risk

`cargo fmt`/`cargo fmt --check` reportedly exited 0 in the main session: unverified by me; not rerun. Also not verified: builds/checks/warnings, full suite/CI, native WoW behavior, older-profile runtime, blocked/target expiration outputs, full player/party-record immutability, undocumented extra arguments, or startup `lua-errors` after the reorder.

Merge risk for the reviewed delta: narrow authentication reorder with a defect-specific RED/GREEN assertion and unchanged duration paths backed by 18 controls. Remaining risk is incomplete compatibility coverage, not an observed remaining precedence defect or established secret-payload disclosure.
