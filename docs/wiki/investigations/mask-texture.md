# Mask Texture

WoW `MaskTexture` clips child textures through mask alpha. Visible regions may have black RGB, so RGB is not clipping coverage.

## How It Works

1. `MaskTexture` is created with `CreateMaskTexture()` or XML `<MaskTexture>`
2. `<MaskedTextures>` block calls `icon:AddMaskTexture(mask)` on each referenced child
3. During rendering, the masked texture's quads carry `mask_tex_index` and `mask_tex_coords` vertex attributes
4. Primitive preparation resolves the mask path from the RGBA atlas first, then the BC1/BC3 atlas, and remaps UVs into the resolved slot
5. The fragment shader multiplies output alpha by mask alpha — where mask alpha is zero, the pixel is fully transparent

## Atlas Resolution and UV Computation

Mask paths are deferred texture requests. A resolved RGBA mask uses one of the five RGBA atlas tiers; a compressed mask uses the BC1 or BC3 atlas binding. If neither atlas contains the path, the pending mask index is cleared and no mask is applied. This resolution step is required for CircleMask-style BC assets, which previously rendered unmasked because their pending requests were treated as unresolved.

The mask UV maps the icon's screen position into the mask's screen area. Critical: the mask should be **larger** than the icon it clips, so the icon samples only the opaque center of the mask texture.

For a 64×64 mask centered on a 45×45 icon:
- UV range: `(9.5/64, 54.5/64)` = `(0.148, 0.852)` on both axes
- Icon samples only the center 70% of the mask texture

If mask size is 0×0 (broken), the full mask (0–1 UV) maps to the icon, clipping visible area at transparent borders.

## Key Behavior: `useAtlasSize` Default

MaskTextures default to `useAtlasSize=true` when not specified in XML. Without this, the mask frame is 0×0 and the full mask texture (including transparent borders) maps to the icon, shrinking the visible area.

## `SmallActionButtonMixin` Override

For 30×30 small buttons, `SmallActionButtonMixin_OnLoad` explicitly sets IconMask to 45×45. This overrides the atlas-derived 64×64 size, giving similar proportional UV coverage (0.167–0.833).

## Action Bar Icon Chain

1. Icon (45×45, fills button) → masked by IconMask (64×64 rounded square atlas)
2. SlotBackground (dark fill visible at rounded corners)
3. SlotArt (decorative golden border)
4. NormalTexture (frame border, OVERLAY layer)

## Wrap Modes

`CLAMPTOBLACKADDITIVE` on a MaskTexture means areas outside the atlas are fully transparent — important for the sheen animation's mask.

## Key Files

- `src/iced_app/masking.rs` — mask UV computation
- `src/render/shader/primitive.rs` — deferred RGBA/BC mask resolution and UV remapping
- `src/render/shader/quad.wgsl` — alpha-only fragment mask sampling
- `src/loader/xml_texture.rs` — XML MaskTexture creation

## 2026-09-08: SpellBook square-mask correction

`spellbook-item-spellicon-mask` has an opaque black center (`RGBA 0,0,0,255`). The former filename-derived RGB/alpha split sampled RGB for this path, making valid active spell icons transparent. `939efe88d` removes that split: all simulator mask paths now use alpha coverage. The rendered-pixel regression covers this square spellbook mask, the passive `talents-node-circle-mask`, and legacy `CircleMask`: opaque centers preserve the source icon; transparent corners reveal the background.

Actual addon/SavedVariables GUI proof shows 35 active and six passive icons with visible artwork, not merely populated rows. Independent verification passed fmt/check, nine focused geometry/BC/minimap/spellbook regressions, and reviewed the three passing GPU pixel cases. The user's existing process was left untouched; the rebuilt binary requires relaunch.

## Sources

- [mask-texture-system.md](../../mask-texture-system.md) — full system description
- `/tmp/pi-spell-icon-mask-evidence.json` — live row/mask evidence and supplied screenshot hash
- `/tmp/pi-spell-mask-pixels-{red,green}.*` — rendered-pixel RED/GREEN proof
- `/tmp/pi-spell-icons-after.webp` and `/tmp/pi-spell-icons-gui.json` — actual GUI visual/state proof
- `/tmp/pi-spell-icons-verification.md` — independent bounded verification

## See Also

- [[action-button-icon-mask]] — earlier instance of black-RGB alpha coverage
- [[action-bar-spell-icons]] — concrete use of IconMask on action buttons
- [[talent-sheen]] — sheen animation uses MaskTexture for button shape clipping
