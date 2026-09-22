# Registered template existence

`DoesTemplateExist` is a modeled query over the XML virtual-template registry, not a loader or filesystem predicate. Commit `2c5bf78c7` makes cached DRaidFrames `8922652` start cleanly without widening template state.

## Evidence

- The predicate reads the existing registry only. It neither scans disk, forces addon/XML loading, nor treats ordinary global frame names as templates.
- The frozen `2c5bf78c7` targeted integration binary passes 3/3 template lifecycle tests. Build and test ledgers record unchanged source hashes, test binary SHA-256 `f83828f5b1f7a52054621d8acd4b89f77dc245d40c3bbd586ecfa536a7c25bdd`, and wow-sim SHA-256 `7d0ff153e1bf2149f670e09ba1eea253460bf58b6bb9d2f754c64f857f8ccfbf`.
- Independent final verification confirmed formatting, default offline checking, registry/source/readability review, and the unchanged DRaidFrames replay provenance.
- The unchanged DRaidFrames root emits `AUDIT_ADDON\tDRaidFrames\ttrue\ttrue\tfalse`, `AUDIT_DONE`, then `[]`; its replay ledger records exit 0, unchanged host CVars, and the same wow-sim SHA-256.

This is startup and registry-lifecycle evidence only. DRaidFrames layout, aura rendering, and user workflows remain untested. Argument validation and case folding remain simulator policies rather than native-probed behavior.

## Sources

- [template predicate specification](../../specs/template-existence.md) — behavior and explicit policy boundary.
- [runtime coverage](../../forever-addon-runtime-coverage.md) — isolated cached-addon startup matrix and replay scope.
- `/tmp/forever-addon-audit/template-existence-green-emlklwik/{build-ledger.json,test-ledger.json}` — frozen build and targeted test provenance.
- `/tmp/forever-addon-runtime/draidframes-template-gqfplb1k/ledger.json` — unchanged cached-package replay provenance.

## See Also

- [[lua-api]] — global API surface.
- [[forever-addon-comparison]] — cached-addon compatibility boundary.
