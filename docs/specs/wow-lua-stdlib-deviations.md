# WoW Lua standard-library deviations

The WoW client's Lua library differs from stock Lua 5.1 in the places below. The simulator matches the client, not stock Lua.

Evidence for existing rows: the 3.3.5a (build 12340) client decompile, read statically on 2026-10-29, and Retail 12.1.5.70575 in-game observations in Meorawr/elune issues #10, #11, #12, #15 and #16. The `string.format` details below use additional VERIFIED Retail 12.1.5.70688 evidence from R88af.3 and R88ag; they do not establish behavior for other profiles. The simulator installs this formatter only on Retail 12.1.5; every other profile, PTR included, keeps the previous simulator formatter (`src/lua_api/string_format_legacy.rs`, temporary), because native behavior there is UNKNOWN. The `string.format` tests below run only on Retail 12.1.5. The `%T`/`%F` punctuation gate remains UNKNOWN, and printf-backend rendering is INFERRED.

| Function | Required behaviour | Stock Lua / previous simulator |
|---|---|---|
| `strjoin(sep, ...)` | Concatenates with `lua_concat` semantics: strings and numbers (formatted `%.14g`) only. Any other value raises `attempt to concatenate a <type> value`. The separator is used only between two pieces. | Skipped `nil` and accepted booleans |
| `date(format, ...)` | `date("")` raises `'date' format too long`. Other formats behave like `os.date`. | Returned `""` |
| `bit.mod(a, b)` | When `b` truncates to zero (`-1 < b < 1`), returns `1 / b`, e.g. `bit.mod(5, 0.5) == 2` and `bit.mod(5, 0) == math.huge`. | Integer modulo by zero |
| `loadstring(s)` | Skips a leading UTF-8 BOM (`EF BB BF`). A BOM later in the chunk is still a syntax error. | Syntax error |
| `string.upper` / `string.lower` | Decode UTF-8. Map ASCII letters; U+00C0–U+00DE ↔ U+00E0–U+00FE as a flat 0x20 offset (so × ↔ ÷; ß and ÿ unchanged); U+0410–U+042F ↔ U+0430–U+044F; Ё ↔ ё. Bytes that are not valid UTF-8 are dropped. | Byte-wise C `toupper`/`tolower` |
| Retail `string.format` | VERIFIED for Retail 12.1.5.70688: secure and ordinary calls use the same format function; dispatch accepts exactly `c d i o u x X e E f F g G q s t T`; `%%` consumes no argument; positions 0–99 use one or two digits, allow leading zeros, and reset the sequential cursor; position 0 selects the format argument. `%F`, `%t`, and `%T` rewrite to `%f` with flags, width, and precision. Failed `%d`/`%i` numeric conversion returns zero. Converted `%d`/`%i` values above 2147483647 or below -2147483648 raise `integer overflow attempting to store %f`, with the value in place of `%f`. `%q` ignores flags, width, and precision. `%F`, `%t`, and `%T` accept secret-marked numbers; numeric conversion rejects secret-marked numeric strings. The result of a secret-number conversion is a secret string (INFERRED: R88ag does not state the result classification). Secret numbers with other numeric conversions are UNKNOWN and unchanged. Argument errors use `bad argument #%d to '%s' (%s)` with caller debug information and `bad argument #%d (%s)` without it; missing arguments say `no value`, nil says `nil`, and method calls adjust the argument number. A bad method self uses `calling '%s' on bad self (%s)`. Unknown letters, unsupported options, and an incomplete trailing `%` give `invalid option in \`format'`; overlong width or precision gives `invalid format (width or precision too long)`. The Retail GlobalString templates `TEXT_MODE_A_STRING_1` and `TEXT_MODE_A_STRING_TIMESTAMP` are covered. | The wrapper was installed after the secure standard-library shallow copy, so secureenv retained rilua's original parser. The `%T`/`%F` digit-run punctuation gate is UNKNOWN and is not modeled. Fixed-point printf-backend rendering is INFERRED; tests assert only the verified `%t`→`%f` mapping, not rendered output. |

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
- `secure_string_format_uses_retail_globalstring_templates`
- `integer_conversions_return_zero_when_conversion_fails`
- `signed_integer_formats_report_i32_overflow`
- `positional_formats_select_the_format_and_reset_the_cursor`
- `positional_prefixes_and_parser_errors_match_native_templates`
- `format_argument_errors_use_caller_and_method_names`
- `verified_format_extensions_map_and_ignore_quoted_width`
- `numeric_conversion_rejects_secret_marked_strings`
- `numeric_conversion_accepts_secret_numbers`
- `host_format_errors_omit_function_name_without_a_debug_caller`
