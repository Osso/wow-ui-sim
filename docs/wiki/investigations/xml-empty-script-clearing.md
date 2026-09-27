# XML Empty Script-Function Clearing

`function=""` was recognized by general XML script generation as a clear, but the optimized runtime-template installer treated it as an ordinary no-op handler and retained inherited scripts. Commit `3e67e7b6e` aligns that path with the existing general generator; tests-only `6325ae0d4` records the corrected pre-fix boundary. Independent GREEN verification remains pending.

## Content

### Root cause

The fast installer returned a no-op for an empty function before classifying its binding. It therefore installed the no-op through the normal path instead of clearing the inherited normal, precall, postcall, or intrinsic-default selected binding.

### Scoped contract

An empty `function=` clears only the selected inherited binding. It preserves other bindings, `method=`, nonempty `function=`, and existing script behavior. Ordinary XML and runtime templates use the same clearing classification. No vendor files changed.

### Evidence status

`6325ae0d4` is RED: ordinary normal, runtime precall, and cached FauxScrollFrame inherited-range cases fail; the nonempty-function control passes. The postcall and retained-binding assertions are after the first runtime failure, so they are covered by the test body but not independently established RED. `/tmp/cross-version-empty-script-proof.md` is the proof ledger; verifier GREEN is pending.

### Scroll argument diagnostic correction

The prior 46 alias cases passed behaviorally but were not error-clean: five FauxScrollFrame cases emitted diagnostics because `function=""` resolved `_G[""]` to a table. The three new alias tests are error-clean. Those diagnostics are repaired by this next slice; independent verification is pending. This attribution is scoped to `/tmp/scroll-error-isolated-attribution.md`, which identifies the five isolated FauxScrollFrame cases rather than a shared environment or ScrollBox cause.

## Sources

- [XML empty script-function clearing](../../specs/xml-empty-script-clearing.md) — contract and test boundary
- [helpers.rs](../../../src/loader/helpers.rs) — existing general XML clear classification
- [template_chain.rs](../../../src/lua_api/globals/create_frame/template_chain.rs) — optimized runtime-template installation
- [empty_script_overrides.rs](../../../tests/xml_templates/inline_advanced/empty_script_overrides.rs) — RED coverage

## See Also

- [[xml-template-system]] — XML script installation paths
- [[widget-system]] — ScrollFrame consumer boundary
- [[lua-api]] — public script bindings
