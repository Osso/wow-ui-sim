# Crafted-quality Retail API RED — existing evidence

**Verdict: genuine first-call public-API RED in both actual waited Retail runs.** Each exits **101**, **0 PASS / 1 FAIL**, with 11,019 filtered tests. Verification launched nothing and rebuilt nothing.

Epoch: `verification/widget-discrete-progress-current/crafted-quality-retail-api-red-retry1-20261014T012534611408Z`  
Recorded source: `8b46433a4568e26458a15907ea79636e38ff65f9`. Current HEAD observed: `9cc2112267ea139e3e13408a16787edd53c232dd`.

## Scope and provenance

`tests/crafted_quality_api.rs` read fully, tracked, clean for that path, and unchanged against recorded source via `git -C ... diff 8b46433a4568e26458a15907ea79636e38ff65f9 -- tests/crafted_quality_api.rs` (empty output). HEAD differs; this proves unchanged test scope, not unchanged production/runtime source. File's historical “unexecuted” comment does not override actual run evidence.

Full stdout, stderr, argv, result, admission, and launcher records read for both retry1 directories; exact contents retained in `proof.json`. Both argv records use `systemd-run --user --wait`, direct sealed integration executable, 86-second timeout, read-only source/target bindings, network-isolated bwrap, Retail filter and one test thread. No commands replayed. Both admissions allow 2 GiB maximum / 1.5 GiB high; both launchers finish `exit-code`, `code=exited, status=101/n/a`, not timeout/OOM or admission denial.

Prior receipt reused by **stat only**: device 66306, inode 148374833, size 471375320, mtime_ns 1791941241640759151. Matches historic receipt and both runtime result records. No new hashes or binary/library/archive reads. External libraries/cache/settings and source archive remain independently unauthenticated, as prior receipt states.

## Concrete coverage matrix

| Case | Executed boundary / evidence | Unreached |
|---|---|---|
| `crafted_quality_required_item_info_errors_have_context` | First `requiredArgumentError()` omitted-input call reaches public `C_TradeSkillUI.GetItemCraftedQualityInfo`; `pcall` succeeds, then Lua assertion at test line 14 fails: `required ItemInfo must reject omitted/nil input`. Rust panic `tests/crafted_quality_api.rs:27:6`. 0 passed / 1 failed; launcher exit 101. | Explicit `nil` second call at line 24; message type, method/itemInfo context, secret-denial distinction. |
| `crafted_quality_secret_nil_authenticates_before_required_validation` | Untainted setup assertion at line 36 and genuine `secretwrap(nil)` / `issecretvalue` assertion at lines 37–38 precede first public call. That `pcall` succeeds; assertion at line 41 fails: `untainted secret nil must reach required-input validation`. Rust panic `tests/crafted_quality_api.rs:72:6`. 0 passed / 1 failed; launcher exit 101. | Message checks; subsequent tainted closure/denial/unwrap checks; restoration and recovery. No separately executed security-denial cause. |

Both environments reach `WowLuaEnv::new complete`. Failures are public required-input rejection assertions, not VM-secret construction, setup, compilation, or resource failures. Passing preceding secret/untainted assertions establishes this test's setup, not native parity or later security semantics.

**Return payload unknown:** neither failure prints public API return values. Successful `pcall` establishes no thrown error, not a nil return. Separately, recorded-source `src/lua_api/env_init/runtime_surface_bootstrap.lua:64–78` defines a missing-namespace-key auto-stub returning nil and caching it. That is source behavior, not payload output from these tests or independently proved dispatch attribution.

## Prior denials and compilation context

`runtime-crafted-required-api-red-2g` and `runtime-crafted-secret-api-red-2g` contain only `admission.json`, each `allowed: false`. Preserved unchanged: denial-only, zero launches; not runtime API RED.

Prior no-run compilation records success (741 JSON records; final `build-finished.success=true`; recorded terminal status 0). Existing stderr retains **6 manifest deprecation warnings** in `iced-wgpu-patched/Cargo.toml` and **3 package-cache lock-wait notices**. These do not replace runtime evidence or justify a broad rerun. No new compile or test invocation performed.

## Not proved

Native behavior/exact text; positive crafted-quality producer; full consumer; PTR. Also unproved: explicit-nil second case, contextual error messages, tainted denial/security ordering, recovery/restoration, actual API return payload.

Only requested local report artifacts written. No vendor/production changes, service actions, delegation, Bash, cwd tools, or new authentication.
