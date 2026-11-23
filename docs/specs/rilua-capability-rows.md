# Rilua capability rows

Bounded Retail 12.0.0/12.0.5 contracts unlocked by rilua `842e4d3`. Source declarations: cached retail `FrameScriptDocumentation.lua`, `PlayerScriptDocumentation.lua`, `ChatInfoDocumentation.lua`; shutdown exemptions: retained [12.0.5 changes](../../data/patch-api/sources/12.0.5-api-changes.txt), lines 50 and 95. [Audit](../wiki/investigations/patch-12-0-5-api-audit.md) records proof and residual gaps.

## What it must do

- [x] At `retail-12-0-0`, `dropsecretaccess` revokes the immediate caller's secret access without tainting it; descendants, protected/secure calls and coroutine entry remain denied. Normal/error return restores ancestor access. `canaccesssecrets` uses the same guard as VM secret unwrapping. Descendant propagation/lifetime are inferred VM policy.
- [x] `issecrettable` recognizes wrapped tables and `SecretWrapContents` tables, not ordinary tables merely containing wrappers. Metadata queries preserve caller taint, including addon/revoked callers; addon availability is bounded simulator policy despite the cached `AllowedWhenUntainted` annotation.
- [x] At `retail-12-0-5`, `Ambiguate` accepts genuine secret strings even in tainted/revoked callers; returns transformed secret strings without changing taint. Context remains `NeverSecret`, rejected before fullName processing. Existing public shortening (`none` unchanged, otherwise first hyphen with nonempty suffix) remains simulator policy, not native context enumeration.
- [x] `C_ChatInfo.ReplaceIconAndGroupExpressions` rejects original secret flags before text processing; accepts public/secret byte strings, independently applies icon/group vocabularies, and retains secret output/taint for secret input. Host-owned lowercase brace-token maps define vocabulary; English raid names/rt1–rt8 default to cached icon paths. Groups/localized aliases require explicit host input. One-pass expansion/unknown-token preservation and secret output propagation are inferred simulator policy.
- [ ] Host-owned tainted file and frame-handler invocations use a per-invocation elapsed-time watchdog through loader and dispatch paths. VERIFIED (R88bn, both builds): the VM polls after every 10,001 dispatch steps and raises `script ran too long` on overrun; the 10,000 instruction-count quota is not a native limit. INFERRED (R88bn): interpret thresholds as milliseconds; use 19,000 during loading and startup callbacks through the initial `PLAYER_ENTERING_WORLD`, then 200. Secure/unowned Blizzard execution stays outside the watchdog. Tests can set a deterministic timeout through `WowLuaEnv`.

## How it works

- [12.0.0 audit](../wiki/investigations/patch-12-0-0-api-audit.md)
- [12.0.5 audit](../wiki/investigations/patch-12-0-5-api-audit.md)

## Implementation inventory

- `src/lua_api/globals/security/retail_secret_helpers.rs` — VM-backed global predicates/revocation.
- `src/lua_api/globals/real/ambiguate.rs` — public and opaque secret name transform; older epochs retain prior Lua body.
- `src/c_api/chat_expressions.rs`, `c_chat_info.rs` — host vocabulary and real C API transformation; `SimState::chat_expression_inputs` holds per-environment maps.
- `src/lua_api/execution_budget.rs` — per-invocation watchdog timeout selection, owner scope, and diagnostic formatting for loader and frame dispatch.

## Tests asserting this spec

- `tests/rilua_rows.rs` — secure/addon/revoked contexts, exact transformed output, taint preservation, and one loader invocation below the test timeout followed by an owner-dispatch overrun.
- `tests/ambiguate_context.rs` — existing original flag rejection/public name controls.
- `tests/p1200_global_security.rs` — actual helper publication.

## Known gaps (current cycle)

- [ ] Native bucket-throttle accounting, complete timer/slash/key/callback ownership, and event-wide coverage outside the tainted loader/frame paths remain unproven. The 19,000/200 millisecond phase mapping is an inference, not a native fact.

## Verification — 2026-10-07

[Revision-scoped proof](https://github.com/Osso/wow-ui-sim/blob/183edc381/data/patch-api/evidence/rilua-rows-2026-10-07/proof.md): 8 new tests, 22 name controls, 4 security controls, 21/57 tests in the requested patch filters; Mists --tests check and formatting exit0, zero non-vendor warnings; startup exit0 `[]`. Patch filters require `WOW_SIM_ENABLE_BYTECODE_CACHE=0` because grouped tests otherwise initialize cache before prefork parent bypass. This preserves the same test scopes and source loading. Two helper fixture IDs removed; remaining exact gaps unchanged.

## Out of scope

Native-client parity and automatic roster-derived/localized chat vocabulary are not established by cached declarations. No vendor Lua changes or dependency edits. Historical source chronology is retained, not rewritten as current retail behavior.
