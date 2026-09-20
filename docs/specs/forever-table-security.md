# Forever table-security integration

## Contract

- Only `client-wowforever` opts into rilua's native `settablesecurity`, `secretwrap`, `secretunwrap`, and `issecretvalue` globals.
- Registration occurs after compatibility initialization and before secure-environment copying; guarded compatibility restoration must preserve these native functions.
- The authenticated `Blizzard_CooldownViewer/CooldownViewerSecure.lua` consumer must retain plain/wrapped key equivalence, reject wrapped keys on its secured backing table, and reject tainted callers on its secured proxy.
- Addon-loader closure taint must enforce the same restriction as explicit VM caller taint. A failing loader-boundary regression remains a blocker, not an allowed exception.
- Existing profiles retain their current registrations. No PTR bootstrap or no-op is added to Forever.

## Boundaries

Rilua implements cumulative `DisallowTaintedAccess` and `DisallowSecretKeys`. `SecretWrapContents` fails explicitly. Opaque wrapped keys support this consumer; this does not establish general secret arithmetic, native error-text compatibility, or native conformance.

## Evidence

Source: authenticated Forever `1.60.1.69913` runtime cache, `Blizzard_CooldownViewer/CooldownViewerSecure.lua` and generated `FrameScriptDocumentation.lua`.

Grouped regressions: `tests/wowforever_table_security.rs`. Initial RED: all three tests fail because `settablesecurity` is absent. At `4f92e7cb0`, with rilua closure-taint fix pinned by `5ebe21a46`, all four focused tests pass without compiler warnings, including the real addon-loader taint boundary and compatibility-restoration preservation. Ledger: `/tmp/forever-table-security-integration-ledger.json`. Final runtime and earlier-profile gates remain integration-owner responsibilities.
