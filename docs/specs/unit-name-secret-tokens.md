# UnitName secret unit tokens

Retained [12.0.5 prose](../../data/patch-api/sources/12.0.5-api-changes.txt), line 154: “The UnitName API no longer accepts secret unit tokens.” Cached retail `Blizzard_APIDocumentationGenerated/UnitDocumentation.lua:2400–2416` instead declares `SecretArguments = "AllowedWhenTainted"`, a `UnitToken` input, two named returns, and restricted-name output secrecy. These are distinct evidence surfaces; that metadata does not establish unconditional input rejection for both caller classes.

## What it must do

- [ ] Ordinary player and actual test-local target queries retain fixture names and the current final provider's one-return arity (second assignment is nil), not the cached declaration's two-return shape.
- [ ] Reject secret unit-token inputs through `pcall` from secure top-level and taint-stamped callers, preserving caller taint. Applying the retained prose to both caller classes using `secretwrap` fixtures is simulator inference, not native-verified semantics.
- [ ] Public tokens created by a tainted caller remain usable; taint alone is not secrecy. Assert failure only for secret inputs, without inventing exact error text.

## How it works

- [Lua API architecture](../lua-api.md).
- [Retail secret-value contract](retail-secret-values.md).

## Implementation inventory

- `src/lua_api/globals/group_queries.rs`: final registered `unit_name` provider decodes `Option<String>` and returns one name. Absence of an explicit guard does not establish missing behavior: conversion may already reject wrappers. No production change in this slice.

## Tests asserting this spec

- `tests/unit_name_secret_tokens.rs`: three grouped cases gated by `all(retail-12-0-5, any(profile-retail, client-ptr))`; existing Admin setters populate concrete test-only player and target names. Wrappers remain opaque; no taint clearing, argument unwrapping, or provider replacement.

## Known gaps (current cycle)

- [ ] Compilation and behavior pending parent batch9 registration/execution; batch8 compiles an older registry. Tests may already be GREEN. No registry or manifest changes here.
- [ ] Future native restricted-unit-token probe must distinguish secure versus tainted callers, public-token controls, actual restricted `UnitToken` values versus simulator wrappers, and return arity.

## Out of scope

Production fixes, native parity claims, error-message contracts, output-secrecy changes, other unit-name APIs, vendor edits, and build/test/check execution.
