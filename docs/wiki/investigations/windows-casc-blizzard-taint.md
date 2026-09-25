# Windows CASC Blizzard Taint

Windows startup against the CASC-synced Blizzard UI cache failed because loader taint classification still assumed repo-style `Interface/BlizzardUI` paths.

## Content

After Blizzard UI source moved to the CASC cache under `%LOCALAPPDATA%/wow-ui-sim/blizzard-ui`, path-based taint detection no longer recognized cached Blizzard addons as first-party code. Some Blizzard menu closures were stamped with addon taint, so `SecureTypes.CreateSecureFunction():SetFunction(...)` rejected them as secret values during startup.

The first fix was to classify cached Blizzard UI by TOC metadata instead of path shape. `AllowLoad` identifies most Blizzard UI TOCs, while `UseSecureEnvironment` identifies secure helper TOCs. A second failure during `TimeManager_LoadUI()` showed that some internal Blizzard addons, such as `Blizzard_PersonalResourceDisplay`, have no `AllowLoad` but still use the signed `Blizzard_` folder-name convention used by `C_AddOns.GetAddOnSecurity`.

Runtime `C_AddOns.LoadAddOn()` has an extra boundary: it can be called from a tainted Lua stack. Blizzard/secure TOCs must load with active stack taint cleared and restored immediately after loading, otherwise closures created by XML `OnLoad` handlers can inherit caller taint even though the source addon itself is first-party Blizzard UI.

A separate `CreateFrame(..., template)` boundary persists after XML load: generated method-only/fallback script closures were created under the constructor's active addon taint. `/tmp/retail-regression/taint-provenance.stdout` records BetterBlizzFrames/DandersFrames taint on generated handlers even when the bound Blizzard method is untainted. The template registry now records declaration taint at XML registration; script generation temporarily uses that origin and restores the constructor's stack. Addon overrides remain independently tainted. The grouped behavioral regressions and current startup replay await the sole Cargo build owner; this is not a GREEN claim.

Verification on Windows:

```powershell
cargo test --lib loader::addon::tests --no-default-features --features sound,gui,client-retail
cargo build --bin wow-sim --no-default-features --features sound,gui,client-retail
$env:WOW_INSTALL_PATH='C:\World of Warcraft'
$env:WOW_SIM_NO_ADDONS='1'
$env:WOW_SIM_NO_SAVED_VARS='1'
.\target\debug\wow-sim.exe --no-addons --no-saved-vars lua-errors
```

Expected `lua-errors` output is `[]`.

## Sources

- [src/toc.rs](../../../src/toc.rs) — TOC-level Blizzard/secure code classification.
- [src/loader/addon.rs](../../../src/loader/addon.rs) — addon context taint stamping.
- [src/c_api/c_addons.rs](../../../src/c_api/c_addons.rs) — runtime `C_AddOns.LoadAddOn()` stack-taint boundary.
- [src/lua_api/taint.rs](../../../src/lua_api/taint.rs) — active stack taint save/restore helpers.
- [template declaration taint](../../specs/template-script-declaration-taint.md) — source-origin contract and grouped regressions.
- `/tmp/retail-regression/taint-provenance.stdout` — pre-fix generated-handler runtime provenance.

## See Also

- [[taint-system]] — secure/insecure execution model.
- [[casc-asset-cache]] — CASC-backed Blizzard UI source cache.
- [[casc-fdid-1579624-root-debug]] — related Windows CASC FDID resolution work.
