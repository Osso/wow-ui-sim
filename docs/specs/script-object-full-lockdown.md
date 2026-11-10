# ScriptObject full lockdown

VERIFIED R18 predicates distinguish ScriptObject full lockdown from ordinary forbidden state, aura access restrictions, and combat protection. Completed Lua call outcomes are INFERRED from that static analysis of Retail 12.1.5.70688 and WowForever 1.60.1.70707; see [the implementation notes](../wiki/systems/script-object-full-lockdown.md).

## What it must do

- [x] Model full lockdown as true when the explicit full-lockdown flag is set or the access-allowed flag is absent. New objects start with access allowed.
- [x] At child creation, inherit full lockdown from a fully locked parent and inherit its missing access-allowed state. Do not infer propagation to existing children or reparent transitions.
- [x] Apply `ScopedModifier fullLockdown="true"` to the creation default; `false` clears that default for the scope. Restore the prior default when the scope exits.
- [x] Make `IsForbidden()` true for ordinary forbidden state or full lockdown. `SetForbidden()` must not set full lockdown or clear access-allowed state.
- [x] Reject these measured locked-down-disallowed APIs for secure and tainted callers with `Attempt to access fully locked-down object.`: `DurationTextBinding:SetFontString`, `FontString:CanWordWrap`, `FontString:CanNonSpaceWrap`, `FontString:ClearAlphaGradient`, and `Frame:CreateFontString` on a fully locked receiver.
- [x] Keep full lockdown independent from aura access restrictions, `IsProtected()`, and `InCombatLockdown()`.

## How it works

- [ScriptObject full lockdown](../wiki/systems/script-object-full-lockdown.md)
- [Duration text binding](duration-text-binding.md)

## Implementation inventory

- `src/widget/frame.rs` — full-lock predicate and per-object state.
- `src/widget/registry/mod.rs` — creation-time parent inheritance.
- `src/loader/xml_file.rs` — scoped creation default.
- `src/loader/xml_fontstring.rs` and `src/lua_api/loader_env.rs` — private XML FontString construction path.
- `src/lua_api/methods.rs` — measured full-lock resolver gate.
- `src/lua_api/frame/methods/` — `IsForbidden`, FontString and alpha-gradient consumers.
- `src/c_api/duration_text_binding/state.rs` — FontString argument validation.

## Tests asserting this spec

- `tests/full_lockdown.rs` — scope restore/false behavior, secure and tainted callers, missing access-allowed state, child inheritance, and the `SetForbidden()` distinction.
- `tests/blizzard_auth_challenge_ui_lockdown.rs` — original Blizzard AuthChallengeUI consumer.
- `tests/duration_text_binding_copy.rs` — public/forbidden FontString views through `SetFontString`.

## Known gaps (current cycle)

All requested gates passed. See the [lane report](../../handoff/full-lockdown-model/REPORT.md) for commands, counts, warning totals, and source limits.

## Out of scope

- Live native reproduction, AddOn permission to declare `fullLockdown`, the engine's masking policy, complete resolver and writer inventories, the source of a missing access-allowed flag, teardown, existing-child propagation, and reparent behavior remain UNKNOWN. No Lua mutator for full lockdown was found; that search does not prove one cannot exist.
- Do not invent behavior for unmeasured consumers or transition paths.
