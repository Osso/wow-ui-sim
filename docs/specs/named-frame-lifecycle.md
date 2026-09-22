# Named frame lifecycle

Named frame creation in `src/lua_api/globals/create_frame/helpers_shared.rs` distinguishes simulator bootstrap stand-ins from explicitly authored Lua/XML frames. Duplicate explicit-frame lifetime is inferred from unchanged ActionBarAuras retaining separate player/target container handles; native Forever conformance is not established.

## What it must do

- [ ] Two explicit creations with the same name produce distinct handles; the global name points to the latest ordinary frame without hiding, detaching, or transferring children from the first.
- [ ] Each explicit frame retains its own parent, children, fields, dimensions, and shown state, including duplicates under different parents.
- [ ] The first authored definition replacing a simulator bootstrap placeholder retires the stand-in and migrates its children; a subsequent explicit duplicate does not repeat that migration.
- [ ] Existing `UIParent`/`WorldFrame` global-binding preservation remains unchanged.
- [ ] Two native AuraContainers with the same name retain separate player/target units and their respective slot-frame parents.

## How it works

- [Widget system](../widget-system.md)
- [Frame data flow](../frame-data-flow.md)

## Implementation inventory

- `src/widget/frame.rs`, `frame_defaults.rs` — explicit simulator-placeholder provenance, false for ordinary frames.
- `src/lua_api/builtin_frames.rs` — marks Rust bootstrap stand-ins at construction.
- `src/lua_api/env_init/frames.rs`, `mod.rs` — marks Lua bootstrap-created stand-ins before addon loading begins.
- `src/lua_api/globals/create_frame/helpers_shared.rs` — permits retirement/migration only for a marked placeholder; existing engine-root exception is retained.

## Tests asserting this spec

- `tests/frame_creation/named_duplicates.rs` — explicit duplicate lifetimes, Rust parent/child edges, two-parent visibility, actual bootstrap replacement, root bindings, and native AuraContainer slots; included in existing `frame_creation` integration module.
- `src/loader/tests/global_frame_access.rs` — placeholder migration through the addon loading boundary; prior explicit-frame retirement assertion corrected to use the actual HelpFrame stand-in.
- `tests/globals_legacy.rs` — existing fresh identity, latest global binding, and independent fields.

## Known gaps (current cycle)

- [x] Targeted proof passes named duplicates 5/5, legacy identity 1/1, and placeholder migration 21/21 at exact producer revision `ebff90517`; `cargo fmt --check` and default `cargo check --offline` pass. The frozen-binary RED remains at `/tmp/forever-addon-runtime/duplicate-explicit-frames-e8uv6mc6/stdout`.
- [ ] Exact `ebff90517` replay confirms both ActionBarAuras containers survive, but its player slot remains unassigned after a modeled timed aura. Duration-display acceptance remains open.

## Out of scope

Addon/vendor changes, name-specific addon exceptions, secret-value policy, event registration, and unrelated frame reuse semantics. Bootstrap provenance is simulator-owned; no native placeholder API is exposed.
