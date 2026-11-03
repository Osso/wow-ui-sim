# Widget recognition

## Scope

On Retail and Forever, `C_Widget.IsWidget(value)` returns one boolean. The shared
recognizer reads the slot-zero identity of Region-backed frames, textures and
font strings. Primitives, unrelated tables, animation groups, animations and
control points return false. Extra arguments do not affect the result.

Retail 70688 evidence distinguishes Regions from animations. Forever's
70532 capture establishes the input and return shape, and the 70707 table
inspector uses the query on arbitrary keys and values. Forever's class
recognition follows the Retail implementation as an inferred policy.

## Known gaps

Native class construction, full subclass compatibility, tainted-table access
and secret-value behavior remain unverified.

## Verification

The focused regressions distinguish Regions from lookalikes and check slot-zero
identity and cleanup. Set `MOVEANY_UI_PATH` to the installed MoveAny addon
directory to exercise the selected consumer.
[Forever cases](../../tests/forever_widget_recognition.rs),
[Retail case](../../tests/patch_7_3_0_cached_behavior.rs)

## Sources

- [Shared recognizer](../../src/c_api/c_widget.rs) — profile publication and slot-zero Region classification.
- [Captured Forever API surface](https://github.com/wowless/wowless/blob/69a3654e08305de6fd5ce3533faf7e5597135109/data/products/wow_classic_beta/apis.yaml#L34382-L34388) and [snapshot build](https://github.com/wowless/wowless/blob/69a3654e08305de6fd5ce3533faf7e5597135109/data/products/wow_classic_beta/build.yaml) — input/output shape and older Forever build identity.
- [Pinned Forever table inspector](https://github.com/Gethe/wow-ui-source/blob/aff0d8cd53bf4d9ef1c1dd2786b94461a276fb16/Interface/AddOns/Blizzard_DebugTools/Blizzard_TableInspectorAttributeDataProvider.lua#L46-L57) — broad-value use of the native query on UI build 70707.
