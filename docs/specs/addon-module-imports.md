# Addon module imports

World of Warcraft: Forever 1.60.1 Beta exposes `require(moduleName)` as an import of an already-finished addon Lua file, not a filesystem or package loader. The source contract is Warcraft Wiki revision 6879760; implementation notes live in [addon module imports](../wiki/systems/addon-module-imports.md).

## What it must do

### Completed module values

- [x] Publish values only after successful execution at the actual TOC/XML Lua-file loading boundary; imports do not execute files.
- [x] Return one original completed file value, including `nil`, `false`, tables and functions, preserving identity through garbage collection.
- [x] Reject imports of missing, self, or unfinished files with `Invalid import: No module with that name exists`.

### Module paths and addon boundaries

- [x] Resolve absolute logical paths as `Addon.Module` components without filesystem lookup; preserve requested addon and module spelling in the resolved identity.
- [x] Resolve a leading-dot path from the caller file directory; each additional leading dot ascends one directory.
- [x] Reject a relative path that has no caller-file origin or would escape its caller addon with `Invalid import: Relative imports may only be used within the same addon`.
- [x] Reject empty logical components, path separators, and control characters with `Invalid import: No module with that name exists`.

### Dependencies and dynamic callers

- [x] Permit a disk-file caller to import another addon only when its direct required or optional dependency matches the target addon case-insensitively; same-addon imports do not need a dependency.
- [x] Reject an undeclared cross-addon disk-file import with `Invalid import: Modules from other addons may only be imported if the calling addon has a direct dependency on the addon being imported`.
- [x] Exempt a dynamic caller from the cross-addon direct-dependency check for absolute paths.
- [x] Preserve registered file provenance for delayed nested closures; dynamically compiled chunks remain dynamic even with a spoofed addon filename.
- [x] Reject a dynamic relative request because it has no file origin. This is an explicit simulator policy; the source contract does not specify it.

### Profile isolation

- [ ] Expose this API only for the `wowforever` profile. Other client profiles must retain their current `require` surface.

Path/dependency requirements have sixteen pure resolver tests; registry/value/provenance requirements have fourteen actual-rilua unit tests. Ten grouped integration tests exercise the real TOC/XML loader, delayed/coroutine imports, dynamic chunks, secure environments, addon varargs and existing peer-file taint behavior. Cross-profile final verification remains pending.

## How it works

- [Addon module imports](../wiki/systems/addon-module-imports.md)
- [Addon loading pipeline](../addon-loading-pipeline.md)
- [Client profiles](../wiki/systems/client-profiles.md)

## Implementation inventory

- `src/loader/module_import.rs` — pure logical path and direct-dependency resolver.
- `src/loader/addon_modules.rs` — completed-value registry, prototype-identity caller provenance, and native `require` installation hook.
- `src/lua_api/env.rs` — per-VM module metadata; module values and source closures are rooted in the Lua registry.
- `src/loader/mod.rs` — declares the profile/test-gated resolver and runtime modules.
- `src/loader/lua_file.rs` — registers compiled-file provenance and publishes the existing execution helper's first return only after success.
- `src/lua_api/env_init/mod.rs` — installs Forever imports before secure-environment copying and preserves other profiles' sandbox behavior.
- `src/client_profile.rs` — selects the distinct Forever profile and interface version.

## Tests asserting this spec

- `src/loader/module_import.rs` — sixteen unit tests for absolute/relative resolution, addon boundary rejection, direct dependency authorization, dynamic absolute exemption, malformed requests, and exact errors.
- `src/loader/addon_modules.rs` — fourteen actual-rilua tests for completed values/GC identity and stack restoration, load boundaries, delayed/escaped closures, direct dependencies, spoofed dynamic sources, nearest dynamic origins, failed loads/replacement, non-reexecution and VM isolation.

- `tests/addon_require.rs` — ten Forever loader-boundary tests, including TOC dependencies, XML files, values, failed/self/forward imports and caller provenance.
- `tests/sandbox_dangerous_globals.rs` — profile-specific global/secure-environment exposure and absence of host package loading.
- `tests/wowforever_profile.rs` — distinct profile identity, client paths, TOC routing and build information.

## Known gaps (current cycle)

- [ ] Complete independent verification, cross-profile isolation and startup smoke checks.
- [ ] Native Forever execution is not part of this task; the Wiki contract and local behavioral fixtures do not establish undocumented native edge cases.

## Out of scope

No generic Lua package loader, filesystem lookup at import time, native-client conformance claim, or inferred behavior for multiple returns, module-name case normalization, duplicate module names, failed reload replacement, separate module environments, or TOC module-registration annotations. Those source semantics are unspecified and are not simulator guarantees.
