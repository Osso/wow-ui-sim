# Abbreviated number formatter

Retail 12.0.5 adds `C_StringUtil.CreateAbbreviatedNumberFormatter`. The typed model lives in `src/c_api/abbreviated_number_formatter/`; duration objects consume it through their documented `Format*Duration` methods. Sources: `data/patch-api/sources/12.0.5-api-changes.txt:19`, cached `AbbreviatedNumberFormatterAPIDocumentation.lua`, `NumericFormatterAPIDocumentation.lua`, `StringUtilDocumentation.lua`, `LocalizationSharedDocumentation.lua`, `LocalizationDocumentation.lua`, and `LuaDurationObjectAPIDocumentation.lua`. All examples/policies below lack native-client probe evidence unless explicitly identified as documentation examples.

## What it must do

- [ ] Gate the factory at `retail-12-0-5`; return typed userdata implementing AddBreakpoint, ClearBreakpoints, Copy, GetBreakpoints, ResetBreakpoints, SetBreakpoints, and the common documented NumericFormatter FormatNumber method.
- [ ] Format `123456` as `123k` (factory documentation example); support custom breakpoint rows and abbreviation global lookup.
- [ ] Own input/readback/copy data independently; validate replacements and additions before changing existing state.
- [ ] Clear to an empty configuration; reset to modeled locale defaults.
- [ ] Support actual duration FormatElapsedDuration, FormatRemainingDuration and FormatTotalDuration, including modifier selection.
- [ ] Reject secret arguments/configuration under tainted callers without clearing taint; preserve secret output and copied configuration provenance. Plain numbers produce plain strings with plain configuration.

## How it works

- [Duration core](../wiki/systems/duration-core.md)
- [Lua API](../lua-api.md)

## Implementation inventory

- `src/c_api/abbreviated_number_formatter.rs` — typed userdata, lifecycle, factory, and formatting.
- `src/c_api/abbreviated_number_formatter/config.rs` — atomic typed breakpoint validation and copied readback.
- `src/c_api/abbreviated_number_formatter/model.rs` — owned breakpoint data and inferred numeric/abbreviation rendering.
- `src/c_api/c_string_util.rs`, `src/c_api/mod.rs` — epoch-gated registration.
- `src/lua_api/globals/lua_duration_object/formatting.rs`, `lua_duration_object.rs` — documented duration consumer registration and typed dispatch.

## Tests asserting this spec

`tests/abbreviated_number_formatter.rs` is grouped into the existing integration target. Four focused cases assert source examples, custom/global breakpoints, copy/input/readback isolation, clear/reset, atomic invalid configurations, real duration formatting/modifiers, and ordinary/secret/tainted behavior. No new Cargo target or dependency.

## Known gaps (current cycle)

- [ ] Native default breakpoint lists, localized suffixes and decimal/grouping policy are unknown. Model supports enUS/enGB with paired thousand/million/billion rows and lowercase k/m/b. Other locales fail explicitly, with no English fallback. `123456 -> 123k` is documented; million/billion defaults are guesses.
- [ ] Guess: select greatest threshold at or below absolute input, truncate absolute significand units then divide by fractionDivisor, restore sign. Structure docs ground `1234 -> 1.2k` and `12345 -> 12k`; signed/noninteger/subthreshold/carry behavior remains unknown. Empty/subthreshold configuration preserves numeric text without abbreviation.
- [ ] Guess: finite positive breakpoint/divisors accept one or multiples of ten, reject duplicate thresholds and sparse/nonarray configurations; readback is threshold sorted. Divisor product must be finite. Native restricted/unrestricted validation distinctions and exact errors remain unknown. Global abbreviation templates support exactly one `%s` or `%d` plus escaped percent; other formats error explicitly.
- [ ] Guess: any secret configuration marks all copied row fields secret and all formatted output secret, even below that row. Secret duration/modifier/input also marks output secret. Reads of secret configuration require untainted callers; Copy returns ordinary userdata retaining configuration provenance. Public Lua wrapping/unwrapping uses rilua's guarded primitives, not host declassification or caller-taint changes.
- [ ] Duration Format* currently accepts the new typed abbreviated formatter only. Existing NumericRuleFormatter/SecondsFormatter methods and duration text bindings are unchanged; common multi-formatter duration dispatch is not claimed.

Future native probe: record all default rows plus FormatNumber at 999, 999.9, 1000, 1234, 9999, 10000, 12345, 123456, -1234 and 1e6 in each locale; try duplicate/non-power-of-ten/sparse rows and mutation after Copy/GetBreakpoints; format secret input, secret row fields, secret whole rows and secret duration/modifier in secure and addon-tainted closures. Record issecretvalue, untainted secretunwrap results, pcall errors and post-failure readback before deciding validation/propagation parity.

## Out of scope

Changing existing CreateAbbreviateConfig/abbreviation globals, vendor Lua, other profiles, localized defaults without evidence, native error wording, broad acceptance/deployment, and addon-specific workarounds.
