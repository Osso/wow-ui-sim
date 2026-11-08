# Crafted-item quality GREEN

| Bounded scope | Retail (`cd9f42756`) | PTR (`a5397a2d6`) |
|---|---:|---:|
| Initial required-input/security cases | 2 PASS | 2 PASS |
| Supplied fields/aliases/lifecycle/isolation/secret selectors | 5 PASS | 5 PASS |
| Whole original ItemButton overlay consumer | 1 PASS | 1 PASS |
| Recipe/item/PlayerChoice preservation | 4 PASS | 4 PASS |
| Separate normal startup | PASS | PASS |

Twelve unique test names × two profiles = 24 passing profile-target outcomes, plus two separate startup observations. Implementation is `05665ddad`; later recorded revisions contain documentation changes, not different getter behavior. [Final runtime matrix and full diagnostics](verification/crafted-quality-final-runtime-current/report.md) establishes this bounded proof. [Default lib/bin check and full format](verification/crafted-quality-default-check-format-current/report.md) pass separately; six manifest deprecations remain.

ItemButton runs retain nineteen nil observations and one missing `C_Reveal`; PlayerChoice runs retain seventy-seven nil observations and eighteen missing requirements. These are distinct from zero loader warnings/collected Lua errors. Startup is headless, CASC/sound off, third-party addons/SavedVariables disabled; read-only EditMode cache and existing workarounds still participate. No globally clean runtime, native catalog/name validation, rendered overlay, original professions panel, callback delivery/unregistration, earlier-profile or parent acceptance follows.

[Source gate](verification/crafted-quality-producer-source-current/report.md) retains its historical SOURCE PASS / stale-spec FAIL; [documentation reconciliation](handoff/crafted-producer-doc-reconcile-current.md) is additive. [Initial RED](../../crafted-item-quality-initial-api-red-2026-10-13/README.md) and [supplied-record RED](../red-current/README.md) remain unchanged. Earlier Retail GREEN compile denials are zero-launch admissions, not test failures.

Historical positive-query payloads were not printed. The retained final-runtime report's phrase “Positive RED returned nil” is a source-inferred explanation, not an additional observed payload; the RED packets preserve the exact reached assertion boundaries. This qualification does not change current GREEN results.

All 151 selected texts are complete UTF-8, changed only by ordered literal privacy substitutions and terminal LF when missing. Selected JSON was not reserialized or clipped. Standard tool paths, Cargo versions and public source URLs remain visible. Account/realm/character values, private-home prefixes, unit names and invocation IDs are redacted. Public-copy gate remains pending.
