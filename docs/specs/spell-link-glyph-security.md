# Spell-link glyph argument security

Bounded source311 contract for `C_Spell.GetSpellLink` argument 2 (`glyphID`). The existing provider lives in `src/c_api/c_spell.rs` and uses the real generated spell catalog through `src/c_api/item_spell/c_item.rs`. See [Lua API architecture](../lua-api.md). This slice changes only the chosen arg2 security boundary, not identifier resolution or spell-link production.

Cached retail `Blizzard_APIDocumentationGenerated/SpellDocumentation.lua:408–424` declares `SecretArguments = "AllowedWhenTainted"`, arg1 `SpellIdentifier`, and arg2 `glyphID:number`, nilable, `NeverSecret = true`. That declaration motivates the chosen conservative simulator rule: reject authentic secret arg2 even in secure context, before identifier/provider lookup. It does not establish native permissions, exact error wording, or undocumented glyph behavior.

## What it must do

### Existing public controls

- [ ] Preserve the actual generated19750 output: `|cff71d5ff|Hspell:19750|h[Flash of Light]|h|r`, in secure and ordinary addon-tainted contexts.
- [ ] Preserve nil for unknown4294967295 in both contexts.
- [ ] Preserve current ignored public arg2 behavior for omitted/nil, numeric values, booleans, strings, a real Frame and a table; no new optional type validation or glyph semantics.

### Chosen arg2 boundary

- [ ] Reject authentic host-wrapped secret BOOL true/false, NUMBER, STRING, real Frame and table in arg2, even when caller is secure; ordinary addon taint does not permit them either.
- [ ] Reject secret arg2 for known19750, unknown4294967295 and invalid public arg1 (nil, false, unrecognized string, real Frame, table), rather than returning a provider link or nil. The intended boundary is before any identifier/provider lookup; tests observe rejection across those outcomes without replacing APIs or instrumenting private lookups.
- [ ] Emit a nonempty error without exposing private string or number fixture payloads; do not prescribe exact error text.
- [ ] Preserve caller taint through rejection and subsequent public recovery, restoring secure caller after addon return. Preserve wrapper secrecy, host wrapper/list identity and live allocation sequence, public Frame/table contents and identity, and spell alias state.
- [ ] After forced collection in both contexts, retain authentic secret roots, reject them again and recover exact known link/unknown nil without taint or state changes.

## How it works

- [Lua API architecture](../lua-api.md)
- [Frame data flow](../frame-data-flow.md)

## Implementation inventory

- `src/c_api/c_spell.rs` — existing `GetSpellLink` registration and numeric identifier/provider dispatch; unchanged in this tests/spec slice.
- `src/c_api/item_spell/c_item.rs` — existing catalog-backed spell-link producer; unchanged.
- `tests/spell_link_glyph_security.rs` — minimal real-provider fixture and authentic host VM wrappers.
- `build.rs` / `tests/integration.rs` — existing automatic top-level test discovery into the grouped `integration` target; no runner modification or new Cargo target required.

## Tests asserting this spec

`tests/spell_link_glyph_security.rs` adds eight tests under `retail-12-0-5`:

| Test suffix (`spell_link_glyph_`) | Observable boundary |
| --- | --- |
| `known_spell_retains_exact_real_link` | Real19750 exact output, both contexts |
| `unknown_spell_retains_nil` | Real unknown4294967295 miss, both contexts |
| `public_optionals_remain_ignored` | Nil/public optional compatibility on known/miss |
| `secret_bools_reject_before_known_miss_invalid_identifier` | Authentic true/false wrappers across seven arg1 cases, both contexts |
| `secret_number_rejects_before_known_miss_invalid_identifier` | Authentic numeric wrapper, same matrix |
| `secret_string_rejects_without_payload_leak` | Authentic string wrapper, same matrix and payload protection |
| `secret_real_frame_and_table_reject_without_mutation` | Real backed Frame/table wrappers, same matrix and unchanged state |
| `forced_gc_preserves_wrappers_and_public_recovery` | All six wrappers survive forced GC, rejection and recovery in both contexts |

Host metadata compares wrapper references/allocation sequences and rooted list entries without reading private payloads or adding GC roots. No Lua secret BOOL equality is used, especially no tainted `rawequal` on secret BOOLs. Public object identity assertions use only unwrapped public Frame/table values.

## Known gaps (current cycle)

- [ ] Parent must compile and observe actual RED before any producer change. Intended failures: current ignored arg2 returns link/nil instead of rejecting authentic secrets; public controls characterize unchanged behavior. No compilation or runtime results claimed here.
- [ ] Producer change and parent-owned GREEN/acceptance remain pending. All requirements remain unchecked until passing evidence exists.

Accounting remains214/134/14; batch56 verifier457 remains active. This input slice promotes no row and awards no arg1 credit.

## Out of scope

- New arg1 validation, aliases, identifier policies or arg1 security credit: retain existing policy.
- Native permission parity, acquisition, exact errors, result secrecy/arity or new output contracts: not established by the cached declaration or this slice.
- Actual glyph-dependent links or optional public validation: retain existing ignored behavior.
- Production edits, build/test/check/readability/gates, push/deploy/delegation: parent owns compiled RED and subsequent implementation/verification.
