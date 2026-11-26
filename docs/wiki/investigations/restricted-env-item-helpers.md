# Restricted environment item helper bindings

The item-targeting handoff reports that `IsHelpfulItem` was nil in a Retail secure snippet, but the same original restricted-environment path succeeds in the current Retail and WoW Forever simulator fixtures. **The earlier failure's root cause is not established.** No simulator producer change is justified until the difference between the reported run and these fixtures is known.

## Evidence

**VERIFIED (Retail 12.1.5 cache):** `Blizzard_RestrictedAddOnEnvironment.toc` selects `RestrictedEnvironment.lua [LoadIntoEnvironment secure]`; that file assigns `C_Item.IsHelpfulItem`, `C_Item.IsHarmfulItem`, and `C_ActionBar.HasAction` into `ENV`, wraps the functions into `RESTRICTED_FUNCTIONS_SCOPE`, and exports that scope through the addon's second-vararg table. `RestrictedExecution.lua` imports that table into `LOCAL_Restricted_Global_Functions` before it creates the restricted closure environment.

**VERIFIED (Retail simulator, exact full-UI case):** `blizzard_restricted_addon_environment_loads::restricted_environment_secure_snippet_imports_item_and_action_apis` passes with the full UI loaded. The original secure snippet runs from a tainted Lua caller and writes six results to the secure frame's attributes: helpful/harmful classifications for items 5512 and 6948, plus true/false `HasAction` results for populated and empty slots.

**VERIFIED (WoW Forever simulator, exact selected-addon closure case):** `restricted_environment_imports::restricted_environment_secure_snippet_imports_item_and_action_apis` passes. The loaded dependency closure includes `Blizzard_RestrictedAddOnEnvironment`; its Forever TOC admits `camelot` and retains the same secure file override.

**VERIFIED (handoff artifact):** the item-targeting report records an earlier Retail nil-call failure and states that `__secureenv.C_Item.IsHelpfulItem` existed in that run. **UNVERIFIED:** why that run differs from the passing current fixtures; its exact test setup and binary/source identity are not included in that report section.

## Root-cause status

The current Rust loader path applies the selected secure replay environment to secure Lua files. The loaded Blizzard source transfers `RESTRICTED_FUNCTIONS_SCOPE` into `RestrictedExecution.lua`, and the exact snippet tests observe all three requested bindings. These results rule out a persistent failure in the tested loader and consumer path. They do not explain or invalidate the earlier report.

No code change was made to Rust secure-environment construction or API publication. Do not describe the earlier failure as fixed: the root cause remains open pending a reproduction of its specific runtime context.

## Sources

- Item-targeting lane handoff, `REPORT.md` § Restricted-environment (local, untracked evidence) — records the earlier failure.
- `Blizzard_RestrictedAddOnEnvironment.toc` — selected Retail file and environment annotation.
- `Blizzard_RestrictedAddOnEnvironment/RestrictedEnvironment.lua` — `ENV` bindings and exported function scope.
- `Blizzard_RestrictedAddOnEnvironment/RestrictedExecution.lua` — restricted environment construction and scope import.
- [Restricted environment probe](../../tests/common/restricted_environment_import_probe.lua) — snippet shared by Retail and Forever tests.
- [Consumer selection and RED proof](consumer-selection-and-red-proof.md) — runtime selection requirements.

## See Also

- [[addon-loading]] — TOC selection and secure per-file loading.
- [[taint-system]] — execution environments and taint boundaries.
