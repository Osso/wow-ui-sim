# Template-script declaration taint

Generated XML template scripts retain the declaration's security origin when a different addon constructs the frame. See [XML template system](../xml-template-system.md) for template application.

## What it must do

- [ ] Trusted XML method-only and fallback handlers remain trusted when a tainted addon creates the frame.
- [ ] Addon XML method-only and fallback handlers remain tainted when clean code creates the frame.
- [ ] A tainted addon method override remains tainted under a trusted XML handler.
- [ ] Applying XML template scripts restores the constructor's caller taint.

## How it works

- [XML template system](../xml-template-system.md)
- [[taint-system]] in the [wiki](../wiki/systems/taint-system.md)

## Implementation inventory

- `src/loader/xml_frame/preparation.rs` — records XML declaration origin when registering a template.
- `src/xml/template.rs` — stores the origin alongside a registered template.
- `src/lua_api/globals/create_frame/template_chain.rs` — scopes generated script installation to that origin.

## Tests asserting this spec

- `tests/xml_templates/registry.rs` — trusted, addon-defined, and overridden method/fallback behavior.

## Known gaps (current cycle)

- [ ] Grouped regression GREEN and full startup replay await the sole Cargo build owner.

## Out of scope

HookScript factories, other callback boundaries, vendor Lua, and filename-based source classification.
