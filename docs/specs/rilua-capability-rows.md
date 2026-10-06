# Rilua capability rows

Bounded Retail 12.0.0/12.0.5 contracts unlocked by rilua `842e4d3`. Source declarations: cached retail `FrameScriptDocumentation.lua`, `PlayerScriptDocumentation.lua`, `ChatInfoDocumentation.lua`; shutdown exemptions: retained [12.0.5 changes](../../data/patch-api/sources/12.0.5-api-changes.txt), lines 50 and 95. [Audit](../wiki/investigations/patch-12-0-5-api-audit.md) records proof and residual gaps.

## What it must do

- [ ] At `retail-12-0-0`, `dropsecretaccess` revokes the immediate caller's secret access without tainting it; descendants, protected/secure calls and coroutine entry remain denied. Normal/error return restores ancestor access. `canaccesssecrets` uses the same guard as VM secret unwrapping. Descendant propagation/lifetime are inferred VM policy.
- [ ] `issecrettable` recognizes wrapped tables and `SecretWrapContents` tables, not ordinary tables merely containing wrappers. Metadata queries preserve caller taint, including addon/revoked callers; addon availability is bounded simulator policy despite the cached `AllowedWhenUntainted` annotation.
- [ ] At `retail-12-0-5`, `Ambiguate` accepts genuine secret strings even in tainted/revoked callers; returns transformed secret strings without changing taint. Context remains `NeverSecret`, rejected before fullName processing. Existing public shortening (`none` unchanged, otherwise first hyphen with nonempty suffix) remains simulator policy, not native context enumeration.
- [ ] `C_ChatInfo.ReplaceIconAndGroupExpressions` rejects original secret flags before text processing; accepts public/secret byte strings, independently applies icon/group vocabularies, and retains secret output/taint for secret input. Host-owned lowercase brace-token maps define vocabulary; English raid names/rt1–rt8 default to cached icon paths. Groups/localized aliases require explicit host input. One-pass expansion/unknown-token preservation and secret output propagation are inferred simulator policy.
- [ ] Host-owned addon load chunks and frame scripts consume configured cumulative instruction budgets. `PLAYER_LOGOUT`/`ADDONS_UNLOADING` handler scopes (including nested execution) are exempt, with no usage refund. Exhaustion errors recover VM state; host reset permits subsequent execution. Default 10 million instructions per addon per host frame (reset before OnUpdate) is inferred policy, not a native time threshold.

## How it works

- [12.0.0 audit](../wiki/investigations/patch-12-0-0-api-audit.md)
- [12.0.5 audit](../wiki/investigations/patch-12-0-5-api-audit.md)

## Implementation inventory

- `src/lua_api/globals/security/retail_secret_helpers.rs` — VM-backed global predicates/revocation.
- `src/lua_api/globals/real/ambiguate.rs` — public and opaque secret name transform; older epochs retain prior Lua body.
- `src/c_api/chat_expressions.rs`, `c_chat_info.rs` — host vocabulary and real C API transformation; `SimState::chat_expression_inputs` holds per-environment maps.
- `src/lua_api/execution_budget.rs` — trusted addon owner scopes and shutdown exemptions, reused by loader, host events, named events and frame/OnUpdate dispatch.

## Tests asserting this spec

- `tests/rilua_rows.rs` — secure/addon/revoked contexts, exact transformed output, taint preservation, budget exhaustion, both shutdown events and recovery through host and LoaderEnv dispatch.
- `tests/ambiguate_context.rs` — existing original flag rejection/public name controls.
- `tests/p1200_global_security.rs` — actual helper publication.

## Known gaps (current cycle)

- [ ] Native throttle thresholds, elapsed-time accounting, complete timer/slash/key/callback ownership and full event-wide exemption outside frame handlers remain unproven. The two prose rows retain pending status until those boundaries are accounted for.

## Out of scope

Native-client parity and automatic roster-derived/localized chat vocabulary are not established by cached declarations. No vendor Lua changes or dependency edits. Historical source chronology is retained, not rewritten as current retail behavior.
