# Addon Module Imports

`require` is a WoW Forever-only loader import: it returns the first value of an addon Lua file that has already completed, without re-executing a file or consulting a filesystem/package loader. Loader integration landed in `78cf08372`; targeted loader evidence is 10/10 before formatting, extraction, and final verification.

## Content

`src/loader/lua_file.rs` registers each TOC or XML Lua file before execution, then publishes its first return only after successful completion. Missing, self, forward, and failed files therefore have no completed value. Registry roots retain returned values, including `nil`, `false`, tables, and functions, across GC; separate stored prototype metadata retains disk-file caller identity for delayed closures. Dynamically compiled chunks remain dynamic even when their source text resembles an addon path.

`src/loader/module_import.rs` resolves logical `Addon.Module` and leading-dot relative paths without file lookup. Disk-file cross-addon imports need a direct `## Dep:` or `## OptionalDeps:` target, compared ASCII-case-insensitively. Same-addon imports need no dependency. Dynamic callers may use absolute imports without that dependency check, but cannot make relative imports because they have no disk-file origin. The registry returns the documented errors for no completed module, cross-addon escape, and a missing direct dependency.

`src/lua_api/env_init/mod.rs` installs the native import before secure-environment copying only under `client-wowforever`; both Forever environments receive the same loader-bound global. Other profiles still remove `require` from the insecure sandbox. This does not add Lua `package`, `dofile`, `loadfile`, or host filesystem behavior.

Native Forever was not run. Multiple returns, module-name normalization, duplicate modules, failed-load replacement, and registration annotations remain source-unspecified and are not native-conformance claims.

## Sources

- [Addon module imports spec](../../specs/addon-module-imports.md) — contract and explicit proof limits.
- [Forever require contract](/tmp/wowforever-require-contract.json) — Warcraft Wiki revision 6879760 capture.
- `78cf08372` — loader completion and profile-exposure integration.
- [Module registry](../../../src/loader/addon_modules.rs) and [resolver](../../../src/loader/module_import.rs) — runtime mechanics.

## See Also

- [[addon-loading]] — TOC/XML file execution.
- [[client-profiles]] — Forever-only feature selection.
