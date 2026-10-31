# Secure helpers: `CreateSecureDelegate` and `securecopy`

## Contract

Source: pinned `Blizzard_APIDocumentationGenerated/FrameScriptDocumentation.lua`. Retail, PTR and Forever declare `CreateSecureDelegate` (:113), `securecopy` (:462), `SecureCopyOptions` (:589) and `SecureDelegateOptions` (:599). Mists, Era and Anniversary declare only `CreateSecureDelegate(luaFunction)` (:93), with no options or documentation text. Wrath declares neither.

Publication follows these declarations: `CreateSecureDelegate` is published on every profile except Wrath, and `securecopy` only on Retail, PTR and Forever. Implementation: `src/lua_api/globals/real/secure_helpers.rs`, on the rilua taint model. Post-cleanup restoration re-registers `CreateSecureDelegate` after `Blizzard_EnvironmentCleanup` sets it to nil.

Documented behaviour:

- `CreateSecureDelegate(luaFunction, options?)` returns a delegate that invokes the original function with secure execution taint, passes arguments and returns values directly. The callee runs with every caller frame's taint suspended. The caller's taint is restored afterwards.
- `SecureDelegateOptions.wrapUntrustedFunctions` (default `true`) wraps function arguments in secure closures that call through using the original function's taint.
- `securecopy(value, options?)` deep-copies tables and preserves shared and recursive references. Copied values receive the current execution taint: string-keyed and integer-keyed slots of each copy are stamped with the caller's stack taint, and a secure caller receives secure slots even from a tainted source.
- `SecureCopyOptions`: `maxTraversalDepth` (default 100), `wrapUntrustedFunctions` (default `true`; wraps function values), and `permitScriptObjects` (default `false`; a script object raises an error, and `true` keeps the reference).

## Best-guess, native UNKNOWN

The documentation does not state these details, and decompiled bodies were not readable:

- **Untrusted** means a function with a closure taint stamp. Functions without a stamp, including C functions and delegates, pass through unwrapped and keep their identity. Every call creates a new wrapper.
- **Delegate errors** propagate to the caller after taint restoration. The simulator does not interpret "protected call context" as catching errors or routing them to the error handler.
- **Depth boundary:** the root table is at depth 1. A table deeper than `maxTraversalDepth` raises an error. `maxTraversalDepth = 0` therefore rejects any table; it does not make a shallow copy. Tables already copied (shared or recursive references) are not checked again.
- **Error text** for all errors: `bad argument #1 to 'CreateSecureDelegate' (function expected, got T)`, `bad argument #2 to 'F' (table expected, got T)`, `securecopy: maximum traversal depth exceeded`, `securecopy: cannot copy a script object`, `securecopy: maxTraversalDepth must be a number, got T`.
- **Option types:** non-nil, non-boolean values of boolean options are treated by truthiness.
- **Script objects** are frame-backed tables (with native widget backing). Copies have no metatable. Table keys are copied with the graph, and function keys are wrapped like values.
- **Classic profiles** use the Retail defaults (wrap untrusted arguments) and accept the undeclared `options` argument.

## Known gaps

- Slot taint on copies covers only string and integer keys. Other key types (tables, booleans, non-integer numbers, functions) require a rilua change.
- Copied tables and wrapper closures carry no object-level taint stamp. rilua's `debug.setobjecttaint` stamps functions only; stamping tables requires a rilua change.
- `HasRestrictions = true` and `SecretArguments = "AllowedWhenUntainted"` restrictions are not enforced.
- The Retail consumer `Blizzard_SharedXMLBase/Mixin.lua` `ApplySecureDelegatesToTable` is selected at startup (`Blizzard_SharedXMLBase.toc`: `AllowLoad: Both`, lists `Mixin.lua`). No pinned Retail file calls it, so its test loads the original file directly. This is preservation evidence, not runtime-selected RED.

## Evidence

- `tests/secure_helpers_surface.rs`: the original Mixin consumer wraps untrusted arguments and restores caller taint; depth (best-guess boundary); shared and recursive references; taint stamped on copied slots; function wrapping; script objects; the delegate wrap option; taint restoration after errors (best-guess propagation).
- `tests/security_state_drivers.rs`: publication on each profile; the existing delegate tests.
- Requires the rilua primitive `stdlib::taint::call_with_stack_taint` (rilua `secure-delegate` branch).
