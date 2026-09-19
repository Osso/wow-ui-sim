# Addon module imports

World of Warcraft: Forever 1.60.1 Beta exposes `require(moduleName)` as an import of an already-finished addon Lua file, not a filesystem or package loader. The source contract is Warcraft Wiki revision 6879760; implementation notes live in [addon module imports](../wiki/systems/addon-module-imports.md).

## What it must do

### Completed module values

- [ ] Look up only files that have completed initial execution; `require` must not load or re-execute files.
- [ ] Return the original completed file value, including `nil`, `false`, and non-table values.
- [ ] Reject imports of missing, self, or unfinished files with `Invalid import: No module with that name exists`.

### Module paths and addon boundaries

- [x] Resolve absolute logical paths as `Addon.Module` components without filesystem lookup; preserve requested addon and module spelling in the resolved identity.
- [x] Resolve a leading-dot path from the caller file directory; each additional leading dot ascends one directory.
- [x] Reject a relative path that has no caller-file origin or would escape its caller addon with `Invalid import: Relative imports may only be used within the same addon`.
- [x] Reject empty logical components, path separators, and control characters with `Invalid import: No module with that name exists`.

### Dependencies and dynamic callers

- [x] Permit a disk-file caller to import another addon only when its direct required or optional dependency matches the target addon case-insensitively; same-addon imports do not need a dependency.
- [x] Reject an undeclared cross-addon disk-file import with `Invalid import: Modules from other addons may only be imported if the calling addon has a direct dependency on the addon being imported`.
- [x] Exempt a dynamic caller from the cross-addon direct-dependency check for absolute paths.
- [x] Reject a dynamic relative request because it has no file origin. This is an explicit simulator policy; the source contract does not specify it.

### Profile isolation

- [ ] Expose this API only for the `wowforever` profile. Other client profiles must retain their current `require` surface.

The checked bullets above are limited to the sixteen unit tests in `src/loader/module_import.rs` at `be073174d`; they prove pure identity resolution and dependency authorization only.

## How it works

- [Addon module imports](../wiki/systems/addon-module-imports.md)
- [Addon loading pipeline](../addon-loading-pipeline.md)
- [Client profiles](../wiki/systems/client-profiles.md)

## Implementation inventory

- `src/loader/module_import.rs` — pure logical path and direct-dependency resolver; currently the only implementation file.
- `src/loader/mod.rs` — declares `module_import`; no loader-backed registry, Lua global, or profile gate is wired yet.

## Tests asserting this spec

- `src/loader/module_import.rs` — sixteen unit tests for absolute/relative resolution, addon boundary rejection, direct dependency authorization, dynamic absolute exemption, malformed requests, and exact errors.

## Known gaps (current cycle)

- [ ] Add a loader-owned completed-module registry and preserve original Lua return values.
- [ ] Capture disk-file caller provenance and distinguish dynamic callers without using a Lua-supplied path.
- [ ] Register the `require` global only for `client-wowforever`.
- [ ] Add the distinct wowforever profile, source mapping, manifest integration, and profile-isolation tests.

## Out of scope

No generic Lua package loader, filesystem lookup at import time, native-client conformance claim, or inferred behavior for multiple returns, module-name case normalization, duplicate module names, failed reload replacement, separate module environments, or TOC module-registration annotations. Those source semantics are unspecified and are not simulator guarantees.
