# Script-object context access (Forever)

## Contract

- `CanBeAccessedInContext()` takes no context argument and returns one boolean on the shared frame/script-object method surface. The caller's current Lua taint, not the frame's addon owner, determines the result.
- An untainted caller may query any modeled frame. A tainted caller cannot access a forbidden frame, or a frame with `DenyTaintedAccessWhenAurasAreSecret` while the explicit per-environment aura-secret context is active. Ordinary tainted frames remain accessible. Protection alone does not deny access.
- If the frame has `Enum.SecretAspect.ObjectSecurity` (explicitly or through forbidden, protected, or prevent-secret-values state), the boolean return is VM-secret. The trusted Rust producer uses rilua's host-computed boolean constructor; tainted callers cannot branch on, compare, or unwrap that result. Otherwise the return is an ordinary boolean.
- The conditional restriction does not itself confer `ObjectSecurity`. The query does not enforce access restrictions on other frame methods.

## Evidence and limits

Cached Forever `SimpleFrameScriptObjectAPIDocumentation.lua` declares the no-argument query and the ObjectSecurity-secret return. Its tainted-forbidden and enforced-restriction denial examples support this bounded policy, not a complete native decision algorithm. The `DenyTaintedAccessWhenAurasAreSecret` flag is `1`. No simulator producer can determine the live aura-secret context; `SimState.auras_secret_in_context` defaults to false as an explicit simulator scenario and can be configured by Rust integration tests. It is not inferred from combat, static spell attributes, or an aura's value. Default and unspecified native decisions remain unverified guesses; no public configuration API or global enforcement is added.

## Proof

`tests/script_object_context_access.rs` covers ordinary and forbidden frames, tainted and untainted callers, proxy forwarding, protected and explicit secret aspects, restriction activation and isolation, and tainted secret-result inspection. The cached BigWigs `/bw` workflow exercises an actual AuraContainer child; its deferred plugin load and options behavior require separate integration evidence.
