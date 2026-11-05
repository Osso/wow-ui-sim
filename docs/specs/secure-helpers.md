# Secure helpers: `CreateSecureDelegate` and `securecopy`

## Contract

Source: pinned `Blizzard_APIDocumentationGenerated/FrameScriptDocumentation.lua`. Retail, PTR and Forever declare `CreateSecureDelegate` (:113), `securecopy` (:462), `SecureCopyOptions` (:589) and `SecureDelegateOptions` (:599). Mists, Era and Anniversary declare only `CreateSecureDelegate(luaFunction)` (:93), with no options or documentation text. Wrath declares neither.

Publication follows these declarations: `CreateSecureDelegate` is published on every profile except Wrath, and `securecopy` only on Retail, PTR and Forever. Implementation: `src/lua_api/globals/real/secure_helpers.rs`, on the rilua taint model. Post-cleanup restoration re-registers `CreateSecureDelegate` after `Blizzard_EnvironmentCleanup` sets it to nil.

Documented behaviour:

- `CreateSecureDelegate(luaFunction, options?)` returns a delegate that invokes the original function with secure execution taint, passes arguments and returns values directly. The callee runs with every caller frame's taint suspended. The caller's taint is restored afterwards.
- **VERIFIED** (`S4s7b-createsecuredelegate-emulated.md` §2): with `wrapUntrustedFunctions` enabled, the delegate replaces **every** function-valued argument with a wrapper. It does not test whether the argument is untrusted. The wrapper calls the original under the execution source captured when the delegate was invoked; a nil captured source clears taint.
- `SecureDelegateOptions.wrapUntrustedFunctions` (default `true`) wraps function arguments in secure closures that call through using the original function's taint.
- `securecopy(value, options?)` deep-copies tables and preserves shared and recursive references. Copied values receive the current execution taint: string-keyed and integer-keyed slots of each copy are stamped with the caller's stack taint, and a secure caller receives secure slots even from a tainted source.
- `SecureCopyOptions`: `maxTraversalDepth` (default 100), `wrapUntrustedFunctions` (default `true`; wraps function values), and `permitScriptObjects` (default `false`; a script object raises an error, and `true` keeps the reference).

## Best-guess basis (best-guess, native UNKNOWN)

Delegate errors do not propagate to the caller. Retail `FrameScriptDocumentation.lua:118` documents a protected call context. The simulator returns zero results and reports the error through its Lua error handler. This is a best guess: the separate `securecallmethod` declaration documents the secure-call family's empty-result convention at `FrameScriptDocumentation.lua:458`; native delegate error handling remains UNKNOWN.

## Best-guess, native UNKNOWN

The documentation does not state these details, and decompiled bodies were not readable:

- **Untrusted**, for `securecopy` only, means a function with a closure taint stamp. Functions without a stamp, including C functions and delegates, pass through unwrapped and keep their identity. Every call creates a new wrapper. The delegate path instead wraps every function argument (see above).
- **Depth boundary:** the root table is at depth 1. A table deeper than `maxTraversalDepth` raises an error. `maxTraversalDepth = 0` therefore rejects any table; it does not make a shallow copy. Tables already copied (shared or recursive references) are not checked again.
- **Error text** for all errors: `bad argument #1 to 'CreateSecureDelegate' (function expected, got T)`, `bad argument #2 to 'F' (table expected, got T)`, `securecopy: maximum traversal depth exceeded`, `securecopy: cannot copy a script object`, `securecopy: maxTraversalDepth must be a number, got T`.
- **Option types:** non-nil, non-boolean values of boolean options are treated by truthiness.
- **Script objects** are frame-backed tables (with native widget backing). Table keys are copied with the graph, and function keys are wrapped like values.
- **Classic profiles** use the Retail defaults (wrap untrusted arguments) and accept the undeclared `options` argument.

## Evidence cross-check

- **Implemented:** `securecopy` omits metatables. The independent VERIFIED static table-copy and allocation findings in `S4s7a-securecopy-emulated.md`, window E, corroborate the emulated metatable-bearing table result. The simulator allocates a new ordinary table without assigning a metatable.
- **Not implemented:** rejecting delegate construction from tainted code. See Known gaps.
- **Not fully implemented:** `secretwrap` variadic same-type values and marker gate. The Retail generated declaration `FrameScriptDocumentation.lua:426-440` independently documents variadic input and output and secret conversion; `S7a-secretwrap-emulated.md` verifies the conditional return-transfer rule, but the complete handler is EMULATED and the live gate is UNKNOWN. The current simulator invokes rilua `wrap_secret` for each argument; complete per-value marker/gate parity requires rilua work.
- **Not implemented:** the `secretunwrap` denied path returning zero values. `S7a-secretwrap-emulated.md` labels that result EMULATED and assumes the reporting helper returns; no independent route establishes this outcome.

## Known gaps

- **VERIFIED + EMULATED, single route:** `S4s7b-createsecuredelegate-emulated.md` reports that tainted construction is rejected; static inspection and emulation read the same client binary. The generated declaration's `HasRestrictions` does not independently identify this restriction. Its reporting effects are UNKNOWN. **Deliberately not implemented:** the simulator stamps every addon chunk with its addon name, so its taint means "insecure addon code"; native taint means "tainted data flowed here". A naive check would reject every addon-created delegate that native accepts.
- **EMULATED only:** Delegate constructor accepts non-function values, and normal invocation forwards every target result (`S4s7b-createsecuredelegate-emulated.md`). Neither fact has independent complete-path corroboration.
- **EMULATED only:** Secret wrapping and unwrapping marker and result counts for the complete handler paths, including denied unwrap, remain conditional on the modeled state and stubbed reporting helper (`S7a-secretwrap-emulated.md`).

- Slot taint on copies covers only string and integer keys. Other key types (tables, booleans, non-integer numbers, functions) require a rilua change.
- Copied tables and wrapper closures carry no object-level taint stamp. rilua's `debug.setobjecttaint` stamps functions only; stamping tables requires a rilua change.
- `HasRestrictions = true` and `SecretArguments = "AllowedWhenUntainted"` restrictions are not enforced.
- The Retail consumer `Blizzard_SharedXMLBase/Mixin.lua` `ApplySecureDelegatesToTable` is selected at startup (`Blizzard_SharedXMLBase.toc`: `AllowLoad: Both`, lists `Mixin.lua`). No pinned Retail file calls it, so its test loads the original file directly. This is preservation evidence, not runtime-selected RED.

## Evidence

- `tests/secure_helpers_surface.rs`: the original Mixin consumer wraps untrusted arguments and restores caller taint; the original `ApplySecureDelegatesToTable` consumer wraps an unstamped argument with the caller's captured source; depth (best-guess boundary); shared and recursive references; taint stamped on copied slots; function wrapping; script objects; the delegate wrap option; and delegate error reporting and taint restoration, including the original `Blizzard_UnitFrame/Shared/UnitFrameUtil.lua:436` consumer.
- `tests/security_state_drivers.rs`: publication on each profile; the existing delegate tests.
- Requires the rilua primitive `stdlib::taint::call_with_stack_taint` (rilua `secure-delegate` branch).
