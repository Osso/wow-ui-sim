# String trimming

`string.trim` trims leading and trailing bytes from a character set. The pinned 12.0.0 register and generated `StringUtilDocumentation.lua` describe space, CR, LF, and tab as the default. Simulator implementation lives in `src/lua_api/env_init/shared_bootstrap.lua`; see the [12.0.0 audit](../wiki/investigations/patch-12-0-0-api-audit.md).

## What it must do

- [x] Omitted characters trim space/CR/LF/tab only; VT/FF stop trimming and remain intact.
- [x] Preserve interior bytes, including whitespace; handle empty and all-default-whitespace strings.
- [x] Preserve existing explicit `nil` default behavior and global `strtrim` alias behavior as simulator compatibility, not independently established native contracts.
- [x] Explicit `xy` removes only those edge bytes; an empty character set leaves the string unchanged.
- [x] Return exactly one string for the tested default, nil, custom, and empty cases.

## How it works

- [12.0.0 API audit](../wiki/investigations/patch-12-0-0-api-audit.md)

## Implementation inventory

- `src/lua_api/env_init/shared_bootstrap.lua`: default and custom trimming plus public alias.

## Tests asserting this spec

- `tests/utility_api.rs`: five focused `test_string_trim_*` cases plus the existing alias control. Test commit `a3ce6850b` reports RED 4/6 overall: both VT/FF boundaries fail; remaining controls pass. Runtime `0d648ebde` reaches development GREEN 6/6. Independent bounded PASS at `1c3cc302d`: unchanged retail 12.0.0 GREEN 6/6 reused by exact hash; retail 12.0.5 and 12.0.7 freshly pass 6/6 each. Fmt/check/default build/startup `[]`, validator, and readability pass; warnings are existing 6/1/profile and none on default commands. Ledger: `/tmp/verify-string-trim-ledger.json`.

## Known gaps (current cycle)

- [ ] Native coercion, wrong-type validation/errors, arbitrary custom byte sets and pattern metacharacters, profile availability, full-LoD consumers, and lifecycle behavior are not established by these tests.

## Out of scope

- Secret/security handling and native conformance claims: deferred by the broad audit scope.
- Unrelated string helpers and VM behavior.
