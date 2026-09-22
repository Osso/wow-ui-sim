# Lua 5.1 unknown short-string escapes

Rilua rejected unknown short-string escapes that Lua 5.1 accepts. Cached Buffalo and dgks therefore failed before addon code could initialize. The parser fix is published and pinned, but independent verification and addon replay remain pending.

## Root cause

Buffalo `8934103` uses `\.` in `DigamAddonLib.lua`; dgks uses `\s`. Lua 5.1's `read_string()` default escape branch consumes the backslash and preserves the next byte. Thus `\.` becomes `.` and `\s` becomes `s`. Rilua instead rejected the byte as an invalid escape.

Rilua commit `15b52249be4b830a359a14b79a20d29bc8d84516`, published on `Osso/rilua` branch `fix-lua51-unknown-escapes`, reuses its single-byte escape path for unknown bytes. wow-ui-sim commit `b8f0982be` pins that revision.

## Scope and proof

- Development proof: 52 lexer tests and one compile/execute string-pattern regression pass.
- Recognized escapes and malformed `\x`/`\u` errors remain separately tested.
- This changes parser behavior only; it does not rewrite addons or relax malformed structured escapes.
- Native Forever execution is unavailable. Buffalo/dgks startup replay and independent verification remain required.

## Sources

- [unknown-escape spec](../../specs/lua51-unknown-escapes.md) — contract and limits.
- `/tmp/forever-addon-audit/buffalo-escape-lexer-cause.md` — cached failure and Elune Lua 5.1-derived lexer comparison.
- `/home/osso/Repos/elune/liblua/llex.c` — reference default escape handling.

## See Also

- [[forever-addon-comparison]] — inventory-level startup disposition.
- [[lua-api]] — Lua compatibility boundary.
