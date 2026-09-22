# Script-object context access (Forever)

## Contract

- `CanBeAccessedInContext()` takes no context argument and returns one boolean on the shared frame/script-object method surface. The caller's current Lua taint, not the frame's addon owner, determines the result.
- An untainted caller may query any modeled frame. A tainted caller cannot access a forbidden frame, or a frame with `DenyTaintedAccessWhenAurasAreSecret` while the explicit per-environment aura-secret context is active. Ordinary tainted frames remain accessible. Protection alone does not deny access.
- If the frame has `Enum.SecretAspect.ObjectSecurity` (explicitly or through forbidden, protected, or prevent-secret-values state), the boolean return is VM-secret. The trusted Rust producer uses rilua's host-computed boolean constructor; tainted callers cannot branch on, compare, or unwrap that result. Otherwise the return is an ordinary boolean.
- The conditional restriction does not itself confer `ObjectSecurity`. The query does not enforce access restrictions on other frame methods.

## Evidence and limits

Cached Forever `SimpleFrameScriptObjectAPIDocumentation.lua` declares the no-argument query and the ObjectSecurity-secret return. Its tainted-forbidden and enforced-restriction denial examples support this bounded policy, not a complete native decision algorithm. The `DenyTaintedAccessWhenAurasAreSecret` flag is `1`. No simulator producer can determine the live aura-secret context; `SimState.auras_secret_in_context` defaults to false as an explicit simulator scenario and can be configured by Rust integration tests. It is not inferred from combat, static spell attributes, or an aura's value. Default and unspecified native decisions remain unverified guesses; no public configuration API or global enforcement is added.

## Proof

Frozen `e3cafcc11` passes 17 unique integration tests across 18 executions (one overlapping filter), including four direct context-access cases, forbidden-frame behavior, secret-value security, and Forever Aura XML. After readability-only refactors, frozen `0bee9e939` revalidates context 4/4 and parser 3/3, formatting and default offline checking; unchanged-path proof is reused. The final frozen binary SHA-256 begins `9d3d0c54`; an unchanged 442-file BigWigs replay loads Core, Plugins, and Options through `/bw`, reaches `options-open` and `DONE`, and records `[]`. Five sound-chat warnings occur after the empty Lua-error array and remain warnings, not workflow errors.

Rilua revision `a5edc4c` is published and pinned. Its host-only secret-boolean support has 19 focused passes; its 462/463 full integration result retains one baseline-confirmed nil-diagnostic failure, so this is not a full-suite pass or native-conformance claim. The prior mixed-source `283029e6` binary is excluded. The query still does not model automatic aura-secret activation, global restriction enforcement, sound handling, test bars, raid modules, pixels, or native behavior.
