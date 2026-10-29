# WoW Lua standard-library deviations

The WoW client's Lua library differs from stock Lua 5.1 in the places below. The simulator matches the client, not stock Lua.

Evidence for every row: the 3.3.5a (build 12340) client decompile, read statically by the laptop session on 2026-10-29, and Retail 12.1.5.70575 in-game observations reported in Meorawr/elune issues #10, #11, #12, #15 and #16. The two sources agree. Behaviour in the profiles between those two clients is inferred from that agreement; it has not been observed directly.

| Function | Required behaviour | Stock Lua / previous simulator |
|---|---|---|
| `strjoin(sep, ...)` | Concatenates with `lua_concat` semantics: strings and numbers (formatted `%.14g`) only. Any other value raises `attempt to concatenate a <type> value`. The separator is used only between two pieces. | Skipped `nil` and accepted booleans |
| `date(format, ...)` | `date("")` raises `'date' format too long`. Other formats behave like `os.date`. | Returned `""` |
| `bit.mod(a, b)` | When `b` truncates to zero (`-1 < b < 1`), returns `1 / b`, e.g. `bit.mod(5, 0.5) == 2` and `bit.mod(5, 0) == math.huge`. | Integer modulo by zero |
| `loadstring(s)` | Skips a leading UTF-8 BOM (`EF BB BF`). A BOM later in the chunk is still a syntax error. | Syntax error |
| `string.upper` / `string.lower` | Decode UTF-8. Map ASCII letters; U+00C0–U+00DE ↔ U+00E0–U+00FE as a flat 0x20 offset (so × ↔ ÷; ß and ÿ unchanged); U+0410–U+042F ↔ U+0430–U+044F; Ё ↔ ё. Bytes that are not valid UTF-8 are dropped. | Byte-wise C `toupper`/`tolower` |

Out of scope:
- Addon `.lua` file loading with a BOM. The client path is not established.
- The initial GC pause: 3.3.5a uses 200, while a Retail 110 is reported, so it is profile-dependent and unverified.
- `error()` with no argument. The client's display of a `nil` error is unknown.

Tests:
- `strjoin_concatenates_like_lua_concat`
- `date_rejects_an_empty_format`
- `installs_legacy_bit_library`
- `loadstring_skips_a_leading_utf8_bom`
- `case_maps_latin1_and_cyrillic_like_the_client`
- `rejects_non_string_arguments`
