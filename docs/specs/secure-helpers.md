# Secure helpers: `CreateSecureDelegate` and `securecopy`

## Contract

Source: pinned `Blizzard_APIDocumentationGenerated/FrameScriptDocumentation.lua`. Retail, PTR and Forever declare `CreateSecureDelegate` (:113), `securecopy` (:462), `SecureCopyOptions` (:589) and `SecureDelegateOptions` (:599). Mists, Era and Anniversary declare only `CreateSecureDelegate(luaFunction)` (:93), with no options or documentation text. Wrath declares neither.

Publication follows these declarations: `CreateSecureDelegate` is published on every profile except Wrath, and `securecopy` only on Retail, PTR and Forever. Implementation: `src/lua_api/globals/real/secure_helpers.rs`, on the rilua taint model. Post-cleanup restoration re-registers `CreateSecureDelegate` after `Blizzard_EnvironmentCleanup` sets it to nil.

Documented behaviour and bounded native findings:

- `FrameScriptDocumentation.lua` describes a protected delegate call that passes arguments and returns values. Actual target security remains UNKNOWN in `S4s7b`.
- **VERIFIED** (`S4s7b-createsecuredelegate-emulated.md` §§1–3): construction captures a source after copying the target. A nonempty execution source is rejected before dispatch; if its reporting helper returns, the entry returns zero values. The helper's effects are UNKNOWN. The first argument is required and accessible, and the examined parser has no function-only type check.
- **VERIFIED** (`S4s7b` §2): invocation applies the captured source to the arguments and inner call. Processing is shallow. With `wrapUntrustedFunctions` enabled, every function-valued argument is replaced by a wrapper that captures the original function and the execution source present before wrapper creation. A nil captured source clears taint; normal return restores the saved source. The helper requests all results, but complete result publication is not established.
- **VERIFIED** (`S4s7b` §3): changing argument provenance does not itself change secret markers.
- `SecureDelegateOptions.wrapUntrustedFunctions` defaults to `true`. The generated declaration describes wrapping function arguments; native behavior above is bounded by the readable helper regions.
- `securecopy(value, options?)` deep-copies tables and preserves shared and recursive references. Copied values receive the current execution taint: string-keyed and integer-keyed slots of each copy are stamped with the caller's stack taint, and a secure caller receives secure slots even from a tainted source.
- `SecureCopyOptions`: `maxTraversalDepth` (default 100), `wrapUntrustedFunctions` (default `true`; wraps function values), and `permitScriptObjects` (default `false`; a script object raises an error, and `true` keeps the reference).

## Best-guess basis (best-guess, native UNKNOWN)

Delegate errors do not propagate to the caller. Retail `FrameScriptDocumentation.lua:118` documents a protected call context. The simulator returns zero results and reports the error through its Lua error handler. This is a best guess: the separate `securecallmethod` declaration documents the secure-call family's empty-result convention at `FrameScriptDocumentation.lua:458`; native delegate error handling remains UNKNOWN.

## Best-guess, native UNKNOWN

The documentation does not state these details, and decompiled bodies were not readable:

- **Untrusted**, for `securecopy` only, means a function with a closure taint stamp. Functions without a stamp, including C functions and delegates, pass through unwrapped and keep their identity. Every call creates a new wrapper. The delegate path instead wraps every function argument (see above).
- **Depth boundary:** the root table is at depth 1. A table deeper than `maxTraversalDepth` raises an error. `maxTraversalDepth = 0` therefore rejects any table; it does not make a shallow copy. Tables already copied (shared or recursive references) are not checked again.
- **Error text:** the simulator uses `bad argument #1 to 'CreateSecureDelegate' (function expected, got nil)` for a nil required target. This does not reject non-nil non-function targets; their later invocation remains UNKNOWN. Other listed error strings are simulator policies, not native-verified text: `bad argument #2 to 'F' (table expected, got T)`, `securecopy: maximum traversal depth exceeded`, `securecopy: cannot copy a script object`, `securecopy: maxTraversalDepth must be a number, got T`.
- **Option types:** non-nil, non-boolean values of boolean options are treated by truthiness.
- **Script objects** are frame-backed tables (with native widget backing). Table keys are copied with the graph, and function keys are wrapped like values.
- **Classic profiles** use the Retail defaults (wrap untrusted arguments) and accept the undeclared `options` argument.

## Evidence cross-check

- **Implemented:** `securecopy` omits metatables. The independent VERIFIED static table-copy and allocation findings in `S4s7a-securecopy-emulated.md`, window E, corroborate the emulated metatable-bearing table result. The simulator allocates a new ordinary table without assigning a metatable.
- **Implemented:** the verified construction gate returns zero values for nonempty execution taint when the native reporting helper returns. Reporter effects remain UNKNOWN.
- **INFERRED simulator choice:** no additional reporter is called because its native effect is unknown.
- **Not fully implemented:** `secretwrap` variadic same-type values and marker gate. The Retail generated declaration `FrameScriptDocumentation.lua:426-440` independently documents variadic input and output and secret conversion; `S7a-secretwrap-emulated.md` verifies the conditional return-transfer rule, but the complete handler is EMULATED and the live gate is UNKNOWN. The current simulator invokes rilua `wrap_secret` for each argument; complete per-value marker/gate parity requires rilua work.
- **Not implemented:** the `secretunwrap` denied path returning zero values. `S7a-secretwrap-emulated.md` labels that result EMULATED and assumes the reporting helper returns; no independent route establishes this outcome.

## Known gaps

- **VERIFIED + EMULATED, single route:** `S4s7b-createsecuredelegate-emulated.md` reports the tainted-construction gate; static inspection and emulation read the same client binary. The generated declaration's `HasRestrictions` does not independently identify this restriction. The simulator applies the gate to its active stack source; equivalence between addon-chunk stamps and native data-flow taint is not established. Reporter effects remain UNKNOWN.
- **INFERRED from EMULATED:** the constructor accepts non-function values (`S4s7b-createsecuredelegate-emulated.md`); the simulator accepts them, but their invocation remains UNKNOWN. Result forwarding is also INFERRED because the complete return path is not corroborated.
- **EMULATED only:** Secret wrapping and unwrapping marker and result counts for the complete handler paths, including denied unwrap, remain conditional on the modeled state and stubbed reporting helper (`S7a-secretwrap-emulated.md`).

- Slot taint on copies covers only string and integer keys. Other key types (tables, booleans, non-integer numbers, functions) require a rilua change.
- Copied tables and wrapper closures carry no object-level taint stamp. rilua's `debug.setobjecttaint` stamps functions only; stamping tables requires a rilua change.
- `HasRestrictions = true` and `SecretArguments = "AllowedWhenUntainted"` restrictions are not enforced.
- The Retail consumer `Blizzard_SharedXMLBase/Mixin.lua` `ApplySecureDelegatesToTable` is selected at startup (`Blizzard_SharedXMLBase.toc`: `AllowLoad: Both`, lists `Mixin.lua`). No pinned Retail file calls it, so its test loads the original file directly. This is preservation evidence, not runtime-selected RED.

## Evidence

- `tests/secure_helpers_surface.rs`: the original Mixin consumer checks function-argument wrapping and per-invocation source; construction source rejection, required/access-checked arguments, secret-marker preservation, and INFERRED numeric-target construction/source capture; securecopy depth, graph identity, taint, function wrapping, and script-object handling.
- `tests/security_state_drivers.rs`: publication on each profile; delegate argument and return-value pass-through.
- Requires the rilua primitive `stdlib::taint::call_with_stack_taint` (rilua `secure-delegate` branch).
