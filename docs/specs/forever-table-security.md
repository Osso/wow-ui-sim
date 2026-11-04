# Forever table-security integration

## Contract

- Only `client-wowforever` opts into rilua's native `settablesecurity`, `secretwrap`, `secretunwrap`, and `issecretvalue` globals.
- Registration occurs after compatibility initialization and before secure-environment copying; guarded compatibility restoration must preserve these native functions.
- The authenticated `Blizzard_CooldownViewer/CooldownViewerSecure.lua` consumer must retain plain/wrapped key equivalence, reject wrapped keys on its secured backing table, and reject tainted callers on its secured proxy.
- Addon-loader closure taint must enforce the same restriction as explicit VM caller taint. A failing loader-boundary regression remains a blocker, not an allowed exception.
- Existing profiles retain their current registrations. No PTR bootstrap or no-op is added to Forever.

## Value secrecy and ordinary party identity (WM91)

Forever value-access checks use the same VM per-value secrecy predicate as
`issecretvalue`. Ordinary `loadstring` closure provenance is not a secret marker;
public literal results stay accessible during tainted execution. Returning an
existing VM secret does not declassify it. Deep table access still rejects
nested VM secrets.

Ordinary existing party members queried without secret input or inherited secret
influence have public identity outputs. Combat or group membership alone does not
mark those names secret. This is the inferred normal contract from WM91, not a
claim that all possible identity invocations are public. Retail's explicitly
restricted test fixtures and current expectations are unchanged.

The same-realm Forever `UnitFullName` second payload remains UNKNOWN. The named
`unresolved_forever_non_player_full_name_second` seam preserves its prior absence;
tests do not select nil, empty text or realm text as a native contract. Player
surname behavior remains in [character identity](character-identity.md).

The original `Blizzard_SharedXMLBase/SecureTypes.lua:33–36` checks secrecy, not
ordinary provenance, before storing keys and values. Tests load that unchanged
source and preserve both exact secret-rejection reasons. The TOC-selected
`Blizzard_UnitFrame/Shared/CompactUnitFrame.lua:826–892` name consumer exercises
distinct party1/2/4 fixtures outside and inside combat.

Evidence: **WM91 — Forever identity strings, value secrecy and SecureMap**,
Forever 1.60.1.70707 and Retail 12.1.5/70688 (PARTIAL). Local native predicate and
loadstring provenance decisions are verified; normal party and literal-result
contracts are inferred. Same-realm realm text and complete invocation-policy
propagation remain unproved. Regression tests: `tests/security_api.rs`.

## Boundaries

Rilua implements cumulative `DisallowTaintedAccess` and `DisallowSecretKeys`. `SecretWrapContents` fails explicitly. Opaque wrapped keys support this consumer; this does not establish general secret arithmetic, native error-text compatibility, or native conformance.

## Evidence

Source: authenticated Forever `1.60.1.69913` runtime cache, `Blizzard_CooldownViewer/CooldownViewerSecure.lua` and generated `FrameScriptDocumentation.lua`.

Grouped regressions: `tests/wowforever_table_security.rs`. Initial RED: all three tests fail because `settablesecurity` is absent. At `4f92e7cb0`, with rilua closure-taint fix pinned by `5ebe21a46`, all four focused tests pass without compiler warnings, including the real addon-loader taint boundary and compatibility-restoration preservation. Ledger: `/tmp/forever-table-security-integration-ledger.json`. Final runtime and earlier-profile gates remain integration-owner responsibilities.
