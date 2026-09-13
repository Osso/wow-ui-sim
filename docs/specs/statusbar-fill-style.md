# StatusBar fill-style state

`StatusBar:SetFillStyle` and `GetFillStyle` expose numeric `Enum.StatusBarFillStyle` state. Pinned `SimpleStatusBarAPIDocumentation.lua` and `SimpleStatusBarConstantsDocumentation.lua` describe the method signatures and enum. Source lives in `src/lua_api/frame/methods/widgets/statusbar.rs`; see the [12.0.0 audit](../wiki/investigations/patch-12-0-0-api-audit.md).

## What it must do

- [x] Publish Standard=0, StandardNoRangeFill=1, Center=2, Reverse=3.
- [x] Round-trip each explicitly set style as exactly one number; setter returns zero values.
- [x] Preserve independent styles across two instances and later changes.
- [x] Simulator validation policy: reject non-numbers, non-integral/nonfinite values, and numbers outside 0..3 before changing prior valid state. Native coercion/error behavior remains unverified.

## How it works

- [12.0.0 API audit](../wiki/investigations/patch-12-0-0-api-audit.md)

## Implementation inventory

- `src/lua_api/frame/methods/widgets/statusbar.rs`: numeric state setter/getter.
- `src/widget/frame.rs`: numeric fill-style field.
- `src/widget/frame_defaults.rs`: initializes modeled Standard state.
- `src/widget/frame_size.rs`: accounts for dynamic allocations; numeric state needs no string allocation.

## Tests asserting this spec

- `tests/widget_methods_colorselect.rs`: five `statusbar_fill_style_*` tests. RED at `557036636`: 2/5 pass, three fail on string returns, lost instance state, and accepted invalid inputs. Runtime `3c8264198` reaches development GREEN 25/25 in the grouped module (five focused tests plus 20 controls). Independent verification pending.

## Known gaps (current cycle)

- [ ] Rendering does not consume fill-style state; Center, Reverse, and StandardNoRangeFill geometry/zero-range semantics remain unimplemented.
- [ ] Native default, validation/coercion/errors, interpolation, legacy reverse-fill interaction, full-LoD consumers, and lifecycle behavior remain unverified.

## Out of scope

- Secret/security enforcement and VM work: deferred by the broad audit.
- No native or whole-StatusBar conformance claim follows from state round-trip tests.
