# Patch 12.1 strict-removal timing

The simulator keeps selected compatibility publications available for addon loading and the first world-entry handlers, then retires them once. This is an explicit simulator policy, not a claim about native per-symbol retirement timing. Sources: `src/lua_api/env_events.rs`, `src/lua_api/env.rs`, and `src/ptr/strict_removals.lua`. See [the patch 12.1 audit](../wiki/investigations/patch-12-1-api-audit.md).

## What it must do

- [ ] Preserve actually published compatibility globals, namespace methods, constants, and CVar callable identities from initialization through addon loading, post-load workarounds, and every first `PLAYER_ENTERING_WORLD` handler.
- [ ] Retire selected publications after those handlers return and before later startup events; later startup cleanup must not wrap CVar functions again.
- [ ] Hide retired CVars case-insensitively through global/default/namespace getters without changing unrelated CVar values.
- [ ] Preserve wrapper identities and retired publications across repeated cleanup and subsequent world-entry events.
- [ ] Reject `BATTLETAG_INVITE_SHOW` registration outside Blizzard addon loading, including before retirement. Its loader-only exception is separate from post-event publication cleanup.

## How it works

- [Event system](../event-system.md)
- [Patch 12.1 audit](../wiki/investigations/patch-12-1-api-audit.md)

## Implementation inventory

- `src/lua_api/env_events.rs` — dispatches all world-entry listeners before requesting cleanup.
- `src/lua_api/env.rs` — once-only post-event workaround application.
- `src/startup.rs` — real login/world/post-login event sequence and final cleanup request.
- `src/ptr/compat_bootstrap.rs` — compatibility publication and strict-removal entry points.
- `src/ptr/strict_removals.lua` — selected removals and CVar wrappers.
- `src/lua_api/frame/methods/text_attribute_event/events.rs` — removed-event rejection with the Blizzard-loader exception.

## Tests asserting this spec

- `patch-tests/patch_12_1/strict_removal_timing.rs` — real addon loader and startup dispatcher observations, two world-entry listeners, later startup events, repeated cleanup, and subsequent world entry; grouped `patch_12_1_audit` target.

## Known gaps (current cycle)

- [ ] Run the focused lifecycle test and record its exact revision and outcome.

## Out of scope

Native pre-startup visibility, per-wrapper native timing, universal publication of every retired symbol before cleanup, private/secret semantics, VM changes, vendor modifications, and moving the existing simulator retirement boundary. This focused fixture does not prove full Blizzard UI loading compatibility.
