# Retail secure dependency XML replay

**INFERRED (R88g.1, R88t.t1/t2):** Retail uses a shared secure environment and separate ordinary/secure loaded states, so an ordinary load does not satisfy a later secure dependency request. The evidence does not establish whether that secure request replays XML from an already ordinary-loaded dependency. R88h4 is pending on that exact question.

## Symptoms

**VERIFIED (Retail simulator logs before the fix):** `GlueParent`'s `FRAMES_LOADED` handler read nil `AccountLogin`, and `ScriptErrorsFrame`'s `OnLoad` called nil `AddLuaErrorHandler`. **INFERRED (loader source and trace):** the Retail `SecureDependency` pass replayed those XML files, created separate frame instances and ran their lifecycle handlers.

## Root cause and current conflict

**VERIFIED (commit `5a7104d29b` and the prior lane report):** replaying dependency XML made the AutoComplete secure-dependency test pass. **VERIFIED (Retail simulator trace):** the same broad replay produced duplicate Glue XML lifecycle errors, including nil `AccountLogin` in `GlueParent` and nil `AddLuaErrorHandler` in `ScriptErrorsFrame`.

**VERIFIED (commit `4a972b832`):** the current loader skips all XML in `SecureDependency` when that addon is already ordinarily loaded; eligible Lua files still load securely, and the special Retail `RaidWarning.xml` guard remains. **VERIFIED (exact Retail runs at `e260b5c21c8d302934f443f0c37b69b0cccbffd0` and final code `373c970dddbbd6f836ae9bb7cdc9eee18c69e64d`):** six selected Glue tests pass, while `retail_catalog_shop_secure_dependency_publishes_autocomplete_names_without_overwrite` still fails after the CreateFrame change. Its final probe returns `(false, false, true, true, true)` instead of `(true, true, true, true, true)`.

**INFERRED (R88g.1, R88t.t1/t2):** these results expose opposing simulator outcomes, but the cited native evidence does not justify either broad XML replay or a TOC/`addToSecureEnv` exception. Keep the current loader rule and leave the AutoComplete test failing until R88h4 answers whether the native secure pass replays XML for an already ordinary-loaded dependency.

## Proof

**VERIFIED (exact final Retail run at `373c970dddbbd6f836ae9bb7cdc9eee18c69e64d`):** the AutoComplete secure-dependency test still fails with the missing secure names above after the CreateFrame change. **VERIFIED (exact Retail runs at the same final code):** all six selected Glue tests pass. The empty-roster test still logs its explicitly non-asserted post-transition CharacterCreate setup diagnostics; its transition assertion passes. See the lane report for receipts and exact test logs.

## Pending native question

**UNKNOWN (R88h4 pending):** when a dependency is already loaded ordinarily, does the secure dependency pass execute its XML files again? Until this is answered, the evidence does not support a file-level exception for AutoComplete XML or a general replay rule for Glue screen XML.

## Sources

- [Add-on load phases](../../specs/addon-load-phases.md#r88t-secure-dependency-and-named-publication-routes) — native secure dependency evidence and its simulator boundary
- [Loader implementation](../../../src/loader/addon.rs) — secure dependency file selection
- [GlueParent load tests](../../../tests/blizzard_glue_parent_loads.rs) and [Glue login tests](../../../tests/glue_login.rs) — original consumers

## See Also

- [[addon-loading]] — dependency loading and secure-library replay
- [[glue-login-lifecycle]] — login-state transition behavior
