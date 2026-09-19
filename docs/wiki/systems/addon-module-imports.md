# Addon Module Imports

Forever's planned `require` support is a loader-bound import of already-executed addon files. The pure resolver exists, but no Lua global, module registry, caller provenance bridge, or wowforever profile gate is wired as of September 19, 2026.

## Content

`src/loader/module_import.rs` represents a logical target as addon spelling plus dot-separated file-stem components. It resolves absolute paths and caller-relative paths without loading files or touching the filesystem. For disk-file callers, cross-addon targets require a direct required or optional TOC dependency, compared ASCII-case-insensitively. Dynamic callers have no file origin: absolute requests skip dependency authorization and relative requests are rejected by explicit simulator policy.

The resolver deliberately does not establish module existence, finished-file state, self-import rejection, stored Lua return values, taint/environment retention, caller provenance, or profile exposure. Those require loader integration.

## Sources

- [Addon module imports spec](../../specs/addon-module-imports.md) — required behavior and proof boundary.
- [Forever require contract](/tmp/wowforever-require-contract.json) — Warcraft Wiki revision 6879760 capture.
- [Module resolver](../../../src/loader/module_import.rs) — current pure implementation.
- [Addon loading pipeline](../../addon-loading-pipeline.md) — loader context.

## See Also

- [[addon-loading]] — TOC and addon-file execution.
- [[client-profiles]] — profile-scoped runtime behavior.
