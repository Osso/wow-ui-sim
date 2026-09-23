# MovieFrame subtitle preference

`MovieFrame:EnableSubtitles(enable)` stores a per-frame subtitle preference. The pinned Forever `SimpleMovieAPI` documents one required boolean argument, no return, and no getter. The simulator does not play movies or render subtitles.

## What it must do

- [x] Accept required boolean `true` and `false` on XML-created MovieFrames, returning no Lua values.
- [x] Keep the value independently for each MovieFrame without changing visibility.
- [x] Reject missing, nil, or non-boolean arguments through strict type validation; reject calls on ordinary frames without changing their state.
- [x] Initialize each preference to `false` as explicit simulator policy, not native-verified behavior.

## How it works

- [Widget system](../widget-system.md)
- [Lua API](../lua-api.md)

## Implementation inventory

- `src/widget/frame.rs`, `src/widget/frame_defaults.rs`: per-instance subtitle preference and simulator default.
- `src/lua_api/frame/methods/widgets/movie.rs`, `src/lua_api/frame/methods/widgets/mod.rs`: documented setter and shared frame-metatable registration with MovieFrame receiver validation.

## Tests asserting this spec

- `tests/movie_subtitles.rs`: real temporary addon XML, both boolean settings, independent backing frame state, rejection of unsupported receivers and invalid arguments, and unchanged visibility.

## Known gaps (current cycle)

- [ ] Replay unchanged Datamine archive to determine whether MovieFrame initialization proceeds and whether new errors surface.

## Out of scope

Movie playback, subtitle rendering, audio, events, undocumented getter, native default/conformance, and other MovieFrame methods.
