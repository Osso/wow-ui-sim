# Existing PTR actual FIRST-call RED

Source `9cc2112267ea139e3e13408a16787edd53c232dd`; test delta `8b46433a4`; current observed `b03eaee384d9b90e2dd675a16572a588c0bc465f`. Minimal git -C file comparisons: three diff --exit-code results 0, empty stdout/stderr. Runtime snapshot and current tests/crafted_quality_api.rs match exactly; unchanged across 8b/9cc/current. Full 8b source inspected. Stale source header saying unexecuted does not override actual runtime evidence.

| Run | Actual result | Executed boundary | Unreached |
|---|---|---|---|
| runtime-crafted-required-api-red-2g | --wait 101; 0 PASS, 1 FAIL; 10196 filtered | First omitted call pcall succeeds; required rejection assertion fails. tests/crafted_quality_api.rs:27:6, assertion at line 14. | Explicit nil call and every context check. |
| runtime-crafted-secret-api-red-2g | --wait 101; 0 PASS, 1 FAIL; 10196 filtered | Genuine VM secret-nil setup passes; first untainted call pcall succeeds; rejection assertion fails. tests/crafted_quality_api.rs:72:6, assertion at line 41. | Context checks, tainted denial, wrapper preservation, taint recovery, subsequent validation and unwrap. |

Exact failures: `required ItemInfo must reject omitted/nil input`; `untainted secret nil must reach required-input validation`.

Both runtime admissions allowed. Both launchers report actual exited status 101. Startup completed. RED credit belongs only to the two first-call rejection assertion failures. No setup, compile, or admission RED credit. Secret test is NOT an executed security-denial counterexample: tainted closure never runs.

Actual successful-call payload not printed: unknown. Nil autostub is only a distinct source hypothesis, not runtime payload evidence.

Initial PTR `crafted-quality-ptr-api-red-20261014T012904034210Z`: only admission.json, allowed=false, zero launches; no-run admission denial preserved unchanged, zero RED credit.

Receipt/stat reuse only: recorded stat before/after agrees with both runtime result tuples. No binary/library/archive/source-tree hashes or fresh authentication performed. External inputs and current executable not freshly authenticated.

Positive producer, whole consumer, native parity unproved. No changed production. No rerun/build/auth/tests/check/fmt/services/delegation/Bash/cwd operations or repository edits. Full stdout/stderr/argv/result/admission/launcher, source, comparisons, existing receipt and initial admission embedded in proof.json.
