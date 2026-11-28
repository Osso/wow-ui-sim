# Retail secure dependency XML replay

**INFERRED (R88g.1, R88t.t1/t2):** a secure dependency request should load eligible Lua files into shared `__secureenv` without replaying XML from an ordinary-loaded dependency. The native evidence establishes secure dependency mode and separate loaded states, but does not state whether XML is replayed.

## Symptoms

**VERIFIED (Retail simulator logs before the fix):** `GlueParent`'s `FRAMES_LOADED` handler read nil `AccountLogin`, and `ScriptErrorsFrame`'s `OnLoad` called nil `AddLuaErrorHandler`. **INFERRED (loader source and trace):** the Retail `SecureDependency` pass replayed those XML files, created separate frame instances and ran their lifecycle handlers.

## Root cause and fix

**VERIFIED (commit `5a7104d29b`):** the Retail branch of `skips_loaded_dependency_xml()` skipped only `RaidWarning.xml`; the generic skip for already ordinary-loaded dependencies no longer applied. **INFERRED (loader source and trace):** the secure pass then created duplicate frames and ran their lifecycle handlers.

**VERIFIED (commit `4a972b832`):** the generic skip was restored for `SecureDependency` XML when the dependency is already loaded ordinarily; eligible Lua files still load securely, and the special Retail `RaidWarning.xml` guard remains. **VERIFIED (branch diff):** no vendor file or test assertion changed.

**INFERRED (R88g.1, R88t.t1/t2):** the native evidence supports shared secure-environment dependency loading and separate ordinary/secure loaded states; it does not prove XML replay for a dependency already loaded ordinarily. This simulator rule keeps XML frame creation single-instance without claiming native parity for that detail.

## Proof

**VERIFIED (merged simulator):** all six targeted Retail Glue tests pass. The five available Forever Glue equivalents also pass. The empty-roster test still logs its intentionally non-asserted post-transition CharacterCreate setup diagnostics; its error-free transition assertion passes. See the lane report for receipts and exact test logs.

## Sources

- [Add-on load phases](../../specs/addon-load-phases.md#r88t-secure-dependency-and-named-publication-routes) — native secure dependency evidence and its simulator boundary
- [Loader implementation](../../../src/loader/addon.rs) — secure dependency file selection
- [GlueParent load tests](../../../tests/blizzard_glue_parent_loads.rs) and [Glue login tests](../../../tests/glue_login.rs) — original consumers

## See Also

- [[addon-loading]] — dependency loading and secure-library replay
- [[glue-login-lifecycle]] — login-state transition behavior
