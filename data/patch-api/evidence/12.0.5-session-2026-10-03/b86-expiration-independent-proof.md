# B86 independent verification — 2026-10-03

## Scope and provenance

Producer: `0d2a98596d6bd6c78d20e3d5f39ce89d7bd5b6c6`. Inputs: `a342c6eec1fa758487082eb8a0653353d80441df`. Findings below refer to the producer, not later documentation.

**PASS — revision isolation.** Initial HEAD equaled producer; latest observed HEAD was `80aa64102adb04e60a174d1a9f4bb4014fbcde5a`. The later commit changes only `data/patch-api/sources/12.0.5-page-coverage.json`, `docs/specs/unit-stat-output-restriction.md`, and three wiki files. `git diff 0d2a98596 HEAD -- src tests Cargo.toml Cargo.lock build.rs` exited 0 with empty output. No later changes under `src` or `tests`. Working tree status was empty. Input-to-producer diff for `tests` and the expiration spec was also empty.

**PASS — artifacts and wiring.** Implementation has 194 lines, tests 298, spec 46. Registration is substantive: `src/c_api/aura_duration.rs:51–56` registers `"DoesAuraHaveExpirationTime"` to its Rust callback; `src/lua_api/globals/register.rs:92–93` invokes `aura_duration::register`. `build.rs:54–90` discovers top-level test modules and emits their declarations; `tests/integration.rs:1` includes the generated harness. Independent execution discovered all nine cases.

Read verify and rust-readability skills. No agents, model CLIs, cargo, build, formatter, full simulator, or repository writes used. Only the requested report file was written.

## 1. Source delta and declaration

**PASS.** `data/patch-api/sources/12.0.5-api-changes.txt:364–365` says:

```text
C_UnitAuras.DoesAuraHaveExpirationTime
  # SecretArguments AllowedWhenTainted -> AllowedWhenUntainted
```

`data/patch-api/sources/12.0.5-register.json:2414–2424` records ID `"global api-C_UnitAuras-DoesAuraHaveExpirationTime-365"`, subject `"C_UnitAuras.DoesAuraHaveExpirationTime"`, change `"# SecretArguments AllowedWhenTainted -> AllowedWhenUntainted"`, `source_lines: [365]`, chronology `"consolidated-final"`, status `"consolidated-delta"`.

Cached declaration at `/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/UnitAuraDocumentation.lua:107–125`:

- Function name at 107; `Type = "Function"` at 108.
- `RequiresUnitAuraAccess = true`, `RequiresValidUnitAuraInstance = true`, `SecretWhenUnitAuraRestricted = true` at 109–111.
- `SecretArguments = "AllowedWhenUntainted"` at 112.
- Documentation at 113: `"Returns true if an aura instance will expire after a certain amount of time."`
- Arguments at 117–118: nonnil `auraInstanceUnit: UnitToken`, nonnil `auraInstanceID: number`.
- Return at 123: one nonnil `hasExpirationTime: bool`.

Declaration/source evidence does not establish native runtime behavior.

## 2. Authentication ordering and preserved behavior

**FAIL — both arguments are not authenticated before validation.** Producer introduces `read_expiration_arguments`; its unit match starts at `src/c_api/aura_duration.rs:79` with `unwrap_secret(state, stack_val(state, 1))?`. It validates/converts the unit at 80–85, including `"C_UnitAuras: unit must be a string or nil"`, before invoking the instance-ID unwrap at 87. Lookup is later, at 69.

Concrete source-derived counterexample, not executed: a tainted caller invoking `DoesAuraHaveExpirationTime(12, SecretTimedID)` receives the unit-type error at 85; the authentic secret second argument never reaches its VM authentication at 87. With `('player', SecretTimedID)`, authentication instead produces the VM denial. This violates spec line 20, `"denied before validation or lookup"`, and the commit message's before-validation claim. It is not evidence of a leaked secret payload: the differing first argument is public.

**PASS — secret payloads do not reach validation or lookup for tainted callers.** Each argument that actually reaches its unwrap is guarded before its payload is consumed. With valid public/nil unit and secret ID, denial occurs before ID validation/lookup. Secret unit, including unknown secret unit, is denied at the first unwrap. Known versus unknown valid string units do not select different secret-denial errors. No path found where a tainted secret's payload itself reaches validation or lookup.

### Exact VM semantics

`Cargo.toml:26` and `Cargo.lock:4992–4994` pin rilua revision `6044544b960cd68b4b0c58bb3373412757c2caee`. Inspected `/home/osso-test/.cargo/git/checkouts/rilua-fd5a0715e46b5888/6044544/src/table_security.rs:231–256`:

- At 233–234, a non-userdata value is returned unchanged: `return Ok(value)`.
- At 236–242, userdata without a VM-owned secret payload is also returned unchanged. Arbitrary userdata is not treated as a secret.
- At 244–245, actual wrappers require `ensure_secure_caller(state)?` before `Ok(payload)`.
- At 248–255, insecure state errors with `"table security operation requires an untainted caller"`.
- `src/api.rs:403–412` checks every frame from `0..=state.ci`; any `frame.taint.is_some()` returns false. It does not merely inspect the immediate callback.

An untainted caller receives the original payload. A tainted caller is denied for a VM-owned secret; public values pass unchanged even when tainted. These functions accept `&LuaState` and do not alter wrappers or taint. `src/lua_api/methods.rs:383–387` decodes string bytes as UTF-8 without secret-policy logic.

**PASS — duration controls unchanged.** `git show 0d2a98596` changes only imports, the expiration callback's argument-reader call, and the new argument reader. `GetAuraBaseDuration`/`GetRefreshExtendedDuration` still route through `query_duration` at 100–115, which calls unchanged `read_public_arguments`. That helper still rejects secrets in arguments 1, 2, and 3 at 160–165. `find_public_aura` is unchanged. All 18 controls passed independently.

**PASS at helper level; qualification at profile level.** The new non-`retail-12-0-5` helper at 95–98 delegates to the exact old `read_public_arguments` call. However, `src/c_api/mod.rs:15–16` already gates the entire module on `retail-12-0-5`, and `globals/register.rs:92–93` gates registration likewise. The new negative-cfg branch is unreachable in existing builds. Its source preservation does not prove older profiles expose this producer or reject secrets.

Additional observed change: retail expiration calls no longer inspect argument 3. Previously the shared helper validated/rejected it (`aura_duration.rs:160,184–191`); the new reader consumes only the two declared arguments. Extra-argument behavior is untested and outside the declared two-argument contract. Public malformed-input error text also changes. Both readers reject invalid UTF-8; prior decoding is `src/lua_bridge/from_stack.rs:294–308`.

## 3. Nine tests and complete requirement mapping

All test references below are `tests/aura_expiration_time.rs`. `AssertExpires` at 49–57 asserts `select('#', ...) == 1`, boolean type, `not issecretvalue(value)`, and exact expected value. It calls the actual registered API. `RejectExpires` at 59–63 checks registration and a nonempty string error, but not its cause or ordering.

| Test / lines | Status and behavioral evidence | Vacuity / limits |
|---|---|---|
| `timed_and_permanent_player_auras_return_exactly_one_public_boolean`, 98–109 | PASS: 102–105 query helpful/harmful IDs 301–304 with true/false expectations. | Cannot pass with constant output; checks result shape and secrecy through helper. |
| `party_records_are_unit_and_instance_isolated`, 112–123 | PASS: 116–119 query party buff/debuff and cross-unit misses. | Concrete positive party result prevents universal-false vacuity; does not test blocking. |
| `stored_expiration_changes_are_read_live`, 126–136 | PASS: false → true → false after changing only stored expiration, 128–135. | Rejects cached results and duration-only implementations. |
| `unknown_units_instances_and_nil_unit_return_false`, 139–150 | PASS: 143–146 test unknown/negative ID, unknown unit, nil unit. | Alone accepts always-false implementations; other tests exclude them. Nil policy remains inferred. |
| `malformed_arguments_error_before_lookup`, 153–170 | PASS for errors: 157–165 cover numeric/table/bool unit; omitted/nil/string/NaN/infinite ID; malformed ID on unknown unit. Positive recovery at 166. | Any unrelated nonempty error passes each rejection. Does not directly assert validation-before-lookup ordering or mixed malformed-public/secret precedence. |
| `untainted_query_accepts_each_authentic_secret_argument_and_combination`, 173–196 | PASS: host-secret installation 71–94; 179–181 verify actual secret payloads; 182–188 test each position/both, true/false and secret unknown-unit miss. Taint and secrecy assertions 178–191. | Positive/negative results and authentic wrappers prevent mock-secret or universal-false vacuity. |
| `tainted_query_denies_each_secret_even_before_unknown_unit_lookup`, 199–228 | PASS for denial/recovery: stamped caller 221–222; 207–211 reject each position/both and known/unknown units; 212–219 preserve taint/secrecy and permit public calls; 223–224 restore outer untainted acceptance. | Any error passes rejection. No assertion compares VM denial errors or proves authentication-before-validation; no malformed first argument paired with secret ID. RED failure occurs during later untainted recovery, not necessarily during denial checks. |
| `secret_arguments_survive_gc_without_declassification`, 231–261 | PASS for rooted usability/secrecy: full collection 237–238 and 254; string-wrapper identity `rawequal(unit, SecretExpirationUnit)` at 239; ID payload/secrecy at 241; query and tainted denial at 242–256. | No ID-wrapper identity assertion. An ID wrapper replaced with an equivalent secret could evade the stated identity check. |
| `queries_leave_aura_records_unchanged_and_environments_isolated`, 264–298 | PASS for measured state: repeated player/party queries 277–280; before/after equality 293; unseeded second environment false at 294–297. | Snapshot only covers player `(aura_instance_id, expiration_time)` at 267–274/285–292. Other player fields and all party fields are unasserted. |

Every unchecked behavioral spec bullet is mapped:

| Spec line | Requirement | Supporting test(s) | Coverage |
|---|---|---|---|
| 11 | Exactly one public bool from nonzero expiration | 98, 126, plus shared helper in other cases | PASS for concrete fixtures/live changes. |
| 12 | Player polarities, party buffs/debuffs, unit/ID isolation via public enumeration | 98, 112 | PASS for results; collector routing is source proof, not implementation-shape assertion. |
| 13 | Live state, no aura mutation, environment isolation | 126, 264 | PARTIAL: live/isolation asserted; full-record and party immutability not asserted. |
| 14 | Unknown unit/instance/nil unit yield one public false | 139 | PASS. |
| 15 | Invalid unit/nonfinite-or-nonnumeric ID error before lookup | 153 | PARTIAL: error/recovery asserted; ordering source-proven for public inputs, not directly discriminated by test. |
| 19 | Untainted authentic secret unit/ID/both accepted, public result | 173 | PASS. |
| 20 | Tainted secret denied before validation/lookup, including unknown units | 199, 231 | FAIL for before-validation ordering; no assertion covers malformed unit plus secret ID. Other denial cases pass. |
| 21 | Taint unchanged, inputs not declassified, tainted public calls work | 173, 199, 231 | PASS for exercised wrappers/caller contexts. |
| 22 | Rooted secrets retain identity/secrecy across full GC | 231 | PARTIAL: unit identity and both secrets' usability/secrecy asserted; numeric-wrapper identity unasserted. |

No entire behavioral bullet lacks all assertions. Specific unasserted clauses are listed above. The separate unchecked spec line 39, `"Inputs only: no compiled RED, producer or GREEN recorded yet"`, is a stale evidence checkpoint, not a test requirement.

## 4. Saved and fresh execution evidence

**PASS — saved RED/GREEN counts and startup Lua-error claim.** Read all four supplied logs.

- `b86-red.log:3–12`: nine tests; the three secret-argument cases are marked FAILED. Lines 29–30, 46–47, and 62–63 contain `"C_UnitAuras: secret unit access is not modeled"`. Lines 66–71 name those three failures and say: `test result: FAILED. 6 passed; 3 failed; 0 ignored; 0 measured; 10056 filtered out; finished in 1.62s`.
- `b86-green.log:3–30`: nine expiration tests and 18 duration controls individually marked `ok`. Line 32: `test result: ok. 27 passed; 0 failed; 0 ignored; 0 measured; 10038 filtered out; finished in 4.10s`.
- `b86-startup.stdout:3`: `[]`. The whole stdout also contains two native-runner preamble lines; it is not exclusively JSON.
- `b86-startup.stderr:223–227`: `Status: CLEAN`, `Lua errors: 0 unique, 0 occurrence(s)`, zero attributed owners/unattributed occurrences. Lines 22–30 also contain ALSA configuration errors and `Sound: no audio device available`; zero Lua errors does not mean zero process diagnostics.

**NOT VERIFIED from saved logs — cargo exit 0, formatter exit 0, and exact historical revision binding.** These logs contain no captured exit-code record, formatter command/result, or producer/input SHA. Their test summaries support the counts, not those additional provenance claims. No cargo/formatter rerun performed.

### Binary preconditions — PASS

Both checked immediately before execution:

- Producer commit time: `1791056482`, `2026-10-03T14:41:22-05:00`.
- Binary mtime: `1791056602.1911116`, 120.1911116 seconds later.
- Required `git diff 0d2a98596 HEAD -- src tests Cargo.toml Cargo.lock build.rs`: exit 0, empty stdout/stderr.

Mtime plus empty diff meets the requested gate; it is not cryptographic proof of binary build provenance.

### Independent commands — PASS

Cwd for both: `/home/osso-test/Projects/wow/wow-ui-sim`.

```text
timeout 90 /home/osso-test/Projects/wow/wow-ui-sim/target/debug/deps/integration-8ea324359263a4d2 aura_expiration_time:: --test-threads=1
exit 0
running 9 tests
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 10056 filtered out; finished in 1.51s
stderr: empty

timeout 90 /home/osso-test/Projects/wow/wow-ui-sim/target/debug/deps/integration-8ea324359263a4d2 aura_refresh_duration:: --test-threads=1
exit 0
running 18 tests
test result: ok. 18 passed; 0 failed; 0 ignored; 0 measured; 10047 filtered out; finished in 2.91s
stderr: empty
```

All nine expiration cases listed in section 3 individually emitted `ok`. The 18 controls individually emitted `ok`: actual secret rejection; absent metadata; expired aura; explicit numeric/alias override; finite-sum overflow; immutable metadata/aliases/events across GC; invalid metadata; malformed arguments; environment-local metadata/aliases; explicit-only numeric alias; omitted/nil identifier; party polarities/blocking/isolation; permanent aura; saturated refresh cap; unknown alias/metadata; unknown unit/instance; elapsed-clock bounds; zero base/cap.

## 5. Regression search

**PASS for searched named callers and executed controls.** Searched producer tracked files with `git grep`; independently scanned readable `.rs/.lua/.md/.xml/.toc` files, including untracked files, in all four requested roots.

- `src`: seven matches, all in `aura_duration.rs`. Both helpers are private; no external callers found. Duration paths at 115/119 remain unchanged.
- `tests`: five direct-name matches, all in the new expiration file. Duration controls exercise the unchanged paths and passed 18/18.
- `Interface/AddOns`: zero named matches in scanned files.
- `docs/addons`: nine matches, all in ApiContractProbe code/readme/tests. Runtime probe at `ApiContractProbe.lua:2562–2576` calls exactly `("player", id)` via `observeAuraCall`; public two-argument behavior remains intact. `tests/aura_time.lua:7–12` installs `C_UnitAuras = ns`; tests use their own functions, including at 87, 161, and 172, rather than this Rust producer. Their mocked behavior is unaffected by the diff; they were not executed.

Existing duration tests do not acquire changed behavior. The three new secret tests intentionally move from RED to GREEN. No existing direct test found for extra third arguments or the mixed malformed-unit/secret-ID precedence defect. This named scan does not prove absence of dynamically constructed API names.

## 6. Readability and quality

**FAIL — misleading authentication comment.** `src/c_api/aura_duration.rs:75–76`: `"denies tainted ones before validation"` overstates the ordering, as established in section 2.

**Concrete dead-code issue.** Added negative-cfg helper at 95–98 cannot compile into a supported build because its enclosing module requires the positive feature (`src/c_api/mod.rs:15–16`). Positive cfg annotations newly added at 7, 11, and 77 also repeat the enclosing gate. This is not evidence of older-profile runtime preservation.

No additional concrete readability issue found: new reader has 15 body lines, shallow matches, no mutable accumulation, no warning suppression, no complex boolean condition, and descriptive naming. Added code contains zero TODO/FIXME/HACK/XXX markers, empty catches, or commented-out implementation blocks. Existing larger registration/query helpers were not changed and are not raised as new findings.

## 7. Spec honesty and compatibility limits

**PASS — native limits explicitly acknowledged.** Spec lines 3–5 distinguish source/declaration evidence from native execution. Lines 14–15 label miss and representation policies inferred. Line 43 explicitly excludes the three access/restricted-output annotations; line 44 excludes native nil/type/error-wording parity. Cached duration declarations at 149–154 and 392–397 confirm their `AllowedWhenTainted` policy; retaining conservative rejection does not claim that parity.

**FAIL — older-profile assertion overclaims available evidence.** Spec line 46 says `"those builds keep rejecting secret arguments"`. The producer is gated out below the feature, so its unreachable alternate helper cannot substantiate that statement. Older-profile API behavior was not executed or otherwise established.

**Missing stated limit — public collector blocking and fixtures.** `src/lua_api/globals/auras.rs:310–327` consults `C_UnitAuras._blockedAuras` and removes blocked records; 331–339 applies this visible collector before additional filtering. A blocked stored aura with nonzero expiration therefore yields false from this query. The spec's line-12 reference to public enumeration implies the dependency, but line 11's unconditional stored-record description omits this visibility qualification; the new suite never asserts blocked-expiration behavior. Existing duration blocking tests do not assert the expiration API's result.

The collector also returns hardcoded `target_fixture_auras()` for `target` at `auras.rs:295–300`, rather than per-environment stored target records. Player/party fixtures support the stated bounded tests; the opening stored-record description is not proof of an arbitrary-unit backing model.

Nil-unit false remains an explicitly inferred simulator policy, despite cached arguments being nonnil and `RequiresValidUnitAuraInstance = true`. Always-public output and misses must not be presented as native restricted-aura behavior. `_blockedAuras` visibility filtering is not an implementation of those excluded access/restricted-output annotations.

Spec line 39's inputs-only checkpoint is stale at the producer revision. This understates available evidence rather than overstating success, but should not be used as current proof state.

## Unverified scope and merge risk

Not verified: native WoW execution; actual restricted-aura access/output policy; older-profile runtime behavior; full suite/CI/build/check/formatter; saved commands' exit codes or exact SHA binding; live simulator/GUI/startup rerun; ApiContractProbe execution; numeric-secret wrapper identity; full player/party-record immutability; blocked expiration-query behavior; dynamic API-name callers; undocumented extra arguments. No new counterexample was executed because authorization permits only the two existing binary filters and forbids code changes.

Merge risk: narrow, concrete contract violation in authentication precedence, not an established payload-disclosure vulnerability. Passing 27/27 tests does not cover that violated ordering. Duration paths have unchanged source plus fresh 18/18 proof.

## Verdict: REJECT

1. Spec line 20 and producer comment require secret denial before validation. `read_expiration_arguments` validates argument 1 before authenticating argument 2; malformed public unit plus tainted secret ID reaches the wrong boundary. Existing rejection assertions cannot detect it.
2. Acceptance documentation must retain the listed coverage qualifications, particularly unasserted ID-wrapper identity/full-record immutability and unsupported older-profile claims. These are secondary to the ordering defect; no broader compatibility redesign is authorized or implied.
