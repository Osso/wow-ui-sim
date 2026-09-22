# Lua 5.1 unknown short-string escapes

## Contract

- Unknown short-string escape bytes lose the backslash and retain the following byte: `\.` becomes `.`, `\s` becomes `s`, and `\z` becomes `z`.
- Existing recognized escapes retain their current behavior. Malformed hexadecimal/Unicode escapes, out-of-range decimal escapes, and unfinished strings remain errors.
- Parsing occurs in rilua, not through addon-source rewriting or simulator loader substitutions.

## Evidence and limits

Cached Buffalo `8934103` and dgks fail at this parser boundary. The local Elune Lua 5.1-derived `liblua/llex.c` establishes the unknown-escape rule; native Forever execution is unavailable. Lua patterns interpret the resulting bytes separately: the decoded `.` is a wildcard, not a Lua-pattern escaped dot.

Rilua commit `15b52249be4b830a359a14b79a20d29bc8d84516` was published to the explicitly authorized `Osso/rilua` branch `fix-lua51-unknown-escapes`. The simulator pins that revision. Targeted development proof: 52 lexer tests and one compile/execute regression; independent verification and real-addon replay remain pending.
