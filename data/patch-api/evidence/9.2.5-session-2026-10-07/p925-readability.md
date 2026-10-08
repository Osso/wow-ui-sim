# Changed Rust readability audit

Scope: src/c_api/patch_retired_members.rs and tests/patch_9_2_5_{publication_sweep,publication_fixes,cached_surfaces,classic_surfaces}.rs.

Changed lines manually audited: data-only retirement const and one existing registration call; flat sweep spec; repeated lookup assertions in a single Lua loop; explicit env creation/exec/error-count reads. No new nested Rust domain logic, opaque conditions, speculative abstraction or lint/warning suppression. Existing retail feature gate excludes Mists/Wrath/Era/Anniversary; Mists compilation and legacy-lookup test pass. No changed-line readability violations. Existing unchanged retirement function length is not expanded beyond one call.

Metrics: five rust-code-analysis-cli runs exit 0. Cognitive complexity of retirement registration is 0; existing helpers max 1. Registration cyclomatic count is 23 (22 at base), reflecting fallible registration calls rather than nested domain branches; retained as an existing registry-shape finding, not an authorization to refactor unrelated retirement lists. Macro-contained prefork tests need manual review because the analyzer reports no function spaces.
