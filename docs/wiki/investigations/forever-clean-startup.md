# Forever clean startup

Forever `1.60.1.69913` now loads its matching Blizzard UI with no collected Lua errors. The clean result follows source-backed compatibility work plus two explicit simulator policies where native numeric or dynamic behavior remains unavailable.

## Evidence

Immutable batch fifteen built commit `7e449f911` with `gui,client-wowforever`, copied and hashed the binary, then ran `--no-addons --no-saved-vars lua-errors`. It exited 0 with stdout `[]`: zero records and zero occurrences. Stderr contained startup timing and headless audio-device absence, but no UI error or warning candidate.

`/tmp/forever-final-interactions.lua` then completed without error. It checks both Gamepad page units' possess/stance maps and eight pet IDs, EditMode default 0, all weapon-enchant snapshots, BuffFrame update, Gamepad interact icons, MainActionBar OnShow, and chat-overflow pulse.

## Root causes resolved late

- Gamepad possess initialization needed a non-nil pet storage index. The simulator returns existing logical pet-slot base 1, so vendor pet buttons use the same one-based pet APIs. Native numeric equivalence is unverified.
- EditMode exposes named Modern default 0. This matches the supplied build observation; dynamic native selection by input style is unverified.
- Gamepad interaction icons required resolved unit game-object, loot, range, and interactability state plus a state-backed preferred interaction target.

## Limits

Clean simulator startup and these interactions do not prove native Gamepad hardware input, native pet-storage offset, dynamic EditMode policy, or full secret-value semantics.

## Sources

- [Forever report](../../wowforever-1.60.1.md) — build identity, batch artifacts, and proof scope
- [[forever-chat-overflow-slot-animations]] — one late UI initialization root
- [[client-profiles]] — Forever profile identity

## See Also

- [[forever-chat-overflow-slot-animations]] — XML animation binding fix
- [[client-profiles]] — profile routing and cache selection
