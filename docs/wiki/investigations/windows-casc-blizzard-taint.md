# Windows CASC Blizzard Taint

Windows startup against the CASC-synced Blizzard UI cache failed because loader taint classification still assumed repo-style `Interface/BlizzardUI` paths.

## Content

After Blizzard UI source moved to the CASC cache under `%LOCALAPPDATA%/wow-ui-sim/blizzard-ui`, path-based taint detection no longer recognized cached Blizzard addons as first-party code. Some Blizzard menu closures were stamped with addon taint, so `SecureTypes.CreateSecureFunction():SetFunction(...)` rejected them as secret values during startup.

The first fix was to classify cached Blizzard UI by TOC metadata instead of path shape. `AllowLoad` identifies most Blizzard UI TOCs, while `UseSecureEnvironment` identifies secure helper TOCs. A second failure during `TimeManager_LoadUI()` showed that some internal Blizzard addons, such as `Blizzard_PersonalResourceDisplay`, have no `AllowLoad` but still use the signed `Blizzard_` folder-name convention used by `C_AddOns.GetAddOnSecurity`.

Runtime `C_AddOns.LoadAddOn()` has an extra boundary: it can be called from a tainted Lua stack. Blizzard/secure TOCs must load with active stack taint cleared and restored immediately after loading, otherwise closures created by XML `OnLoad` handlers can inherit caller taint even though the source addon itself is first-party Blizzard UI.

A separate `CreateFrame(..., template)` boundary persisted after XML load: generated method-only/fallback script closures inherited the constructor's active addon taint. `/tmp/retail-regression/taint-provenance.stdout` explicitly pairs generated-wrapper taint `BetterBlizzFrames`/`DandersFrames` with underlying Blizzard methods whose taint is `nil`. `35adfc738` records declaration origin in the template registry and uses it only while generating those engine handlers, then restores constructor taint; addon method overrides remain independently tainted. `d63ddd32a` routes that declaration classification through the shared canonical TOC policy, including internal `Blizzard_` folders without `AllowLoad` metadata.

Hook factories had the same origin-boundary failure. The supplied `HookScript` RED (`/tmp/retail-regression/hookscript-origin-red.*`) reports `false false`: a trusted original was contaminated and the addon hook lost its taint. The `hooksecurefunc` RED (`/tmp/retail-regression/securehook-origin-red.*`) likewise reports `false false`, with its generated secret wrapper reported true. `fc7570022` creates the fixed engine adapters separately from addon callbacks: `HookScript` clears taint only around fixed adapter construction and restores it, while `hooksecurefunc` invokes a bootstrap-owned factory through `securecallfunction`. Source and focused tests define the intended boundary, but no current-tree runtime build, replay, or GREEN evidence exists.

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
- [template declaration taint](../../specs/template-script-declaration-taint.md) — declaration-origin contract and grouped regressions.
- [hook callback origin](../../specs/hook-callback-origin.md) — engine-adapter/callback taint contract.
- `/tmp/retail-regression/taint-provenance.stdout` — pre-fix generated-wrapper and underlying-method provenance.
- `/tmp/retail-regression/hookscript-origin-red.*` — pre-fix HookScript origin RED.
- `/tmp/retail-regression/securehook-origin-red.*` — pre-fix hooksecurefunc origin RED.

## See Also

- [[taint-system]] — secure/insecure execution model.
- [[casc-asset-cache]] — CASC-backed Blizzard UI source cache.
- [[casc-fdid-1579624-root-debug]] — related Windows CASC FDID resolution work.
