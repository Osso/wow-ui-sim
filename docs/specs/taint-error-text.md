# Taint and secret error text

Exact error text for taint, secret-value, tainted-table and forbidden-aspect violations. The text matches the client's own templates, including case, quotes and punctuation.

Evidence levels:
- **NATIVE**: the clean-room S9 spec of the 12.1.5.70688 client. Templates are identical in 70707 (Forever) and 12.1.7 (PTR), with one exception noted below. Raise-not-return is verified for the table guard and the tainted secret templates in all three builds.
- **best-guess, native UNKNOWN**: simulator choice where the evidence does not pin the behaviour.

## What it must do

### Secret-value violations (rilua)
- [x] One formatter builds four variants (NATIVE S9 §12). The named form appears when the operand resolves to a local, upvalue, global, field or method name (stock Lua `getobjname`). The taint clause appears when the stack carries a taint origin; that is the innermost taint, as `debug.getstacktaint()` reports it.
  - `attempt to %s %s '%s' (a secret %s value, while execution tainted by '%s')`
  - `attempt to %s %s '%s' (a secret %s value)`
  - `attempt to %s a secret %s value (execution tainted by '%s')`
  - `attempt to %s a secret %s value`
- [x] The type slot is the payload's standard Lua type name; light and full userdata both render `userdata` (NATIVE S9 §9.1).
- [x] VM-raised violations carry the stock `source:line:` prefix; violations raised inside Rust stdlib functions do not.
- [x] Operation words (NATIVE vocabulary S9 §11; mapping to sites best-guess, native UNKNOWN):
  - `perform boolean test on` for `if`/`and`/`or`, `assert` and other stdlib truth checks;
  - `negate` for `not`;
  - `index` for reading a secret table;
  - `perform indexed assignment on` for writing to a secret value;
  - `compare` for equality and ordering. This word was not observed at any native call site.
- [x] Raises that rilua already made on secret operands now use the secret template with the stock Lua operation word: `perform arithmetic on`, `concatenate`, `call` and `get length of`. These are best-guess. No new arithmetic or concatenation raise was added.

### Table guards (rilua)
- [x] `attempted to %s a table that cannot be accessed while tainted (execution tainted by '%s')` and `attempted to %s a table that cannot be indexed with secret keys` raise (NATIVE S9 §10). Reads use `index`, writes `perform indexed assignment on`, `#` and `table.getn` `get length of`, and `next`/`pairs`/`ipairs`/`table.foreach`/`secureexecuterange` `iterate`. Metatable access, `unpack`, `os.time` and the generic `table.*` argument check use `index` (best-guess).
- [x] `#t` on a tainted-access table raises with `get length of`. The native length guard site is verified; before this change the VM allowed `#t`.

### Forbidden aspects and objects (simulator)
- [x] Methods denied by a forbidden aspect raise `Function call not permitted due to forbidden aspect '%s' (execution tainted by '%s')` (NATIVE S9 (d)).
- [x] A `SetParent` that would gain unowned aspects raises `Reparenting disallowed as child object would inherit forbidden aspects: %s`. An anchor raises `Anchoring disallowed as dependent object would inherit forbidden aspects: %s`. Both templates are NATIVE S9 (d). The slot lists the gained `Enum.ForbiddenAspect` names in bit order, joined by `, `. That content is best-guess, native UNKNOWN.
- [x] A tainted caller reaching an aura-restricted script object raises `Attempt to access forbidden object from code tainted by an AddOn` (NATIVE S9 (e) text). That this path uses this template is best-guess.
- [x] An unknown XML forbidden-aspect name reports `Unknown forbidden aspect '%s'` on PTR only, because the string exists only in 12.1.7 (NATIVE cross-build §2). Retail and Forever have no such string. Their `UI.xsd:129` `FORBIDDENASPECT` enumeration makes an unknown name a schema violation. The simulator keeps its load error `Unknown forbidden aspect '%s' in active Enum.ForbiddenAspect` there (best-guess, native UNKNOWN).

## How it works

- [Taint system](../wiki/systems/taint-system.md)
- [Forbidden-aspect inheritance](forbidden-aspect-inheritance.md), [secret auras](secret-auras.md)

## Implementation inventory

- rilua `src/table_security.rs`: `secret_violation`, `TableAccess`, the `checked_*` operand guards; `src/api.rs` `stack_taint_origin`; VM sites in `src/vm/execute.rs` and `src/vm/execute/runtime_ops.rs`.
- `src/lua_api/frame/methods/forbidden_aspects.rs`: aspect-denial, inheritance and unknown-aspect text.
- `src/c_api/unit_aura_access.rs`: forbidden-object text.

## Tests asserting this spec

- rilua `table_security::tests::secret_boolean_error_text_uses_name_and_taint_origin`: four variants. Untainted forms use `dropsecretaccess`-style revocation.
- rilua `table_security::tests::secret_operation_words_follow_native_vocabulary`, `table_security_vm::{tainted_table_error_text_matches_native_operations,secret_key_table_error_text_matches_native_template}`.
- `tests/forbidden_aspect_creation.rs`: aspect denial; reparent and anchor text, including the BetterBlizzFrames icon/border order; PTR-gated `ptr_unknown_forbidden_aspect_has_native_text`.
- `tests/patch_12_1_0_aura_access_restrictions.rs::forbidden_aura_button_error_text_matches_native`, through the Blizzard_AuraContainer AuraButton.
- Housing, private-aura-anchor and `break_up_large_numbers` tests assert the table and secret templates on their host paths.

## Known gaps

- [ ] Which secret variant fires at each native site, and the content of the origin slot, are only partly pinned. The selection mechanism is NATIVE for 70688 only.
- [ ] UNKNOWN: whether the client raises at all for arithmetic or concatenation on secrets. rilua keeps its existing raises with best-guess text.
- [ ] The unresolved tenth secret-formatter call site, the `compare` word and the per-site operation mapping are best-guess.
- [ ] Host C API argument rejections still use `secret access requires an untainted caller without revoked access`. The native `%s. Secret values are only allowed during untainted execution for this argument.` needs per-argument descriptions that are not modelled.
- [ ] Not emitted, because no simulator guard exists: `Cannot set point(s) to forbidden region '%s' from a tainted context`, `Cannot create a forbidden frame from a tainted context.`, `Cannot assign script handler for '%s' (...)`, `XML attribute or element '%s' not permitted due to forbidden aspect(s) '%s' (...)`, `attempted to %s a frozen table`, and the aura-specific `Auras cannot be accessed when secret while tainted by '%s'`.
- [ ] Table guards and Rust stdlib violations have no `source:line:` prefix; native positioning is UNKNOWN.

## Native sources

Clean-room S9 taint-template spec (12.1.5.70688), sections (a)–(e), §9–§12. Cross-build secrets parity comparison (70688, 70707, 12.1.7), sections 2 and 5.
