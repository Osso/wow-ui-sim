# Template existence

`DoesTemplateExist` exposes the registered virtual-frame model through `src/lua_api/globals/real/template_queries.rs`. See [XML templates](../xml-template-system.md) for registry/loading behavior.

## What it must do

- [x] Report whether a single virtual-frame template name is currently registered; missing and empty names return false.
- [x] Reflect actual XML loading and preserve the answer after creating an instance from the template.
- [x] Do not treat ordinary named Lua/XML frames as templates or interpret comma-separated inheritance lists as one template.
- [x] Use the existing case-insensitive registry lookup and typed string argument convention; reject missing/non-string arguments. These are simulator policies, not native-verified edge semantics.
- [x] Observe loaded Blizzard virtual templates without scanning disk, loading addons, or maintaining another registry.

## How it works

- [XML template system](../xml-template-system.md)
- [Addon loading pipeline](../addon-loading-pipeline.md)

## Implementation inventory

- `src/lua_api/globals/real/template_queries.rs` — modeled non-C predicate and registration.
- `src/lua_api/globals/real/mod.rs`, `src/lua_api/globals/register.rs` — shared global publication.
- `src/xml/template.rs` — existing single-template registry lookup, unchanged.

## Tests asserting this spec

- `tests/template_existence.rs` — real temporary addon XML before/after load, inherited template instantiation, ordinary named frames, missing/empty names, argument policy, and a loaded Blizzard CustomAuraContainer template. Included by the existing generated integration target.

## Known gaps (current cycle)

- Frozen targeted GREEN: 3/3 at `2c5bf78c7`, with unchanged source hashes across the build. Ledgers: `/tmp/forever-addon-audit/template-existence-green-emlklwik/{build-ledger.json,test-ledger.json}`. Frozen runtime RED: `/tmp/forever-addon-audit/template-existence-red-qovikc1y/ledger.json` reports `DoesTemplateExist missing`.
- [x] Independent final verification confirms `cargo fmt --check`, default offline `cargo check`, registry/source/readability review, and immutable replay provenance.
- [x] Parent-owned unchanged DRaidFrames `8922652` replay exits 0 with `AUDIT_ADDON DRaidFrames true true false`, `AUDIT_DONE`, and no collected Lua errors. Ledger: `/tmp/forever-addon-runtime/draidframes-template-gqfplb1k/ledger.json`; its wow-sim SHA-256 is `7d0ff153e1bf2149f670e09ba1eea253460bf58b6bb9d2f754c64f857f8ccfbf`, matching targeted proof.
- [ ] Local wowless API inventories list `DoesTemplateExist` for retail/PTR and Classic variants, but provide no input/output specification. Cached generated API documentation has no entry. Cross-profile publication follows that presence evidence; string validation and case folding follow simulator conventions, not native probes.

## Out of scope

Native conformance for argument coercion/case sensitivity, texture/font-string/animation-only registries, eager XML discovery, and addon/vendor modifications. DRaidFrames' observed query concerns a frame template; other template categories lack confirmed contract evidence.
