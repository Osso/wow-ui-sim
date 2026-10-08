# Integrated PlaySound caller proof

Complete scan: `../p730-integrated-sound-callers.json`, `/usr/bin/grep -rnE --exclude-dir=Wowless* --exclude-dir=*Documentation*` over src, tests, bundled Interface/AddOns and the entire cached Blizzard UI. Exit 0, empty stderr, 10,333 untruncated name matches: src 24, tests 164, bundled addons 0, cache 10,145. Counts include declarations, comments, other namespaces with the same member name, flat legacy-cache duplicates and all cached profiles; they are not deduplicated runtime caller counts.

`sound-string-callers.json` retains the separate literal-name/pcall scan, including every result. `sound-nonliteral-candidates.txt` retains 717 variable/expression candidates rather than assuming every non-literal argument is a string. `sound-retail-variable-contexts.json` preserves 137 retail candidate contexts and source hashes.

## Own consumers

No simulator production call or positive test requires legacy sound names. Bare and cached 7.3.0 tests deliberately reject `igInventoryRotateCharacter`; the cached test uses `not pcall`, and the bare test dynamically invokes both global/namespace functions and asserts errors without replacing request state. Utility tests use numeric 839; SOUNDKIT tests use numeric constants.

The src `PlaySound(checked)` matches are parser-only string fixtures under `#[cfg(test)]`, not calls to the production sound API. XML-flow tests install their own local logging functions. The microbutton fixture defines a logging function and the production click path supplies SOUNDKIT IDs. Account-store tests supply numeric sentinels or track numeric constants; their trackers do not establish audio-output parity.

## Blizzard consumers and real input contract

Cached Mainline Sound.lua assigns `PlaySound = C_Sound.PlaySound`. Generated SoundDocumentation.lua specifies `soundKitID` as non-nil `number`; `sound-contract.json` retains the exact argument declaration, alias text, source hash and primary Blizzard-generated documentation reference. The pinned 7.3.0 page states the sound-kit-ID-only transition. Rejecting old sound names therefore matches the documented modern retail contract. No native WoW executable was run; channel/return/audio/secret-policy parity is not inferred.

No active retail literal-name call was found. Its only quoted-name candidate is commented ReforgingUI Classic XML. Retail variable names such as ClickBinding's `soundFile` resolve to SOUNDKIT constants, not filenames. Store checkbox sounds, reputation/bank click choices and HouseEditor size selections similarly select SOUNDKIT IDs. Other calls consume documented sound-kit data fields, numeric constants, and SOUNDKIT expressions; variable names alone were not treated as proof of string input.

All-cache exceptions remain visible: two executable quoted-name call sites exist in Mists/Cata source (`WatchFrame.lua` bonus objective and `PVPFrame.lua` ReadyCheck), repeated in flat and wowforever caches; ten further quoted-name matches are comments. These are not modern retail callers. No vendor file was changed and no legacy compatibility fallback was added. Their native legacy-client event behavior is not claimed as covered by retail tests or by the Mists compilation check.

## Behavior proof

Sound selections pass: integration 55, prefork 5, lib 8. Actual static popup / game menu / UI panel selections pass: integration 40/24/71; prefork 33/14/39. These include numeric SOUNDKIT consumers after cached Blizzard alias installation. Publication proof passes all 44 cases. Branch startup with `--no-addons --no-saved-vars` prints `[]`; the addons-enabled invocation and separately built master `85c2acb2d` both print `[]`. No ambient WOW_SIM_NO_ADDONS override was present. This proves the available configured startup, not arbitrary third-party addon callbacks or every inactive cached profile.
