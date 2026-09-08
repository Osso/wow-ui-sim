# Action Button Icon Mask

Action button icons can disappear even when action state and icon textures are correct if the GPU mask samples RGB intensity for the action-bar icon mask. The action-bar mask stores coverage in alpha; visible regions may be black in RGB.

## Root Cause

The recurring symptom is a main action bar with visible button chrome and hotkey numbers but no spell icons. Lua state can still be healthy:

- `HasAction(1)` returns `true`.
- `C_ActionBar.GetActionTexture(1)` returns `ICONS/Spell_Holy_FlashHeal`.
- `ActionButton1.icon:GetTexture()` returns the same icon path.
- `ActionButton1Icon` is visible in `dump-tree`, has a 40x40 layout rect, and has `Interface\hud\uiactionbariconframemask` attached as a mask.

The failing layer is mask coverage. The older renderer used RGB intensity for selected paths, even though action-button visible regions can be black in RGB with opaque alpha. Multiplying by RGB turns the icon fully transparent.

## Fix Pattern

Do not remove `ActionButton1.icon` masks and do not patch `UpdateButtonArt`; those only hide the real problem.

Use alpha coverage for simulator mask paths. `939efe88d` removed the filename-selected coverage mode and `FLAG_MASK_ALPHA_COVERAGE`; WGSL now multiplies output alpha by `mask_color.a` for every resolved mask. This also fixes active SpellBook square icons, whose mask has an opaque black center.

## Reproduction

Before the fix:

```bash
target/debug/wow-sim --no-addons --no-saved-vars screenshot \
  -o /tmp/actionbar-current.webp --width 1200 --height 300 --filter MainActionBar
```

The screenshot showed only borders and hotkey labels.

Removing the masks in a probe made icons render, proving the fault was mask sampling:

```lua
for i = 1, 12 do
  local b = _G["ActionButton" .. i]
  if b and b.icon then
    for j = b.icon:GetNumMaskTextures(), 1, -1 do
      b.icon:RemoveMaskTexture(b.icon:GetMaskTexture(j))
    end
  end
end
```

After the fix, the same filtered screenshot renders spell icons with masks still attached.

## Sources

- [masking.rs](../../../src/iced_app/masking.rs) — UV application without filename-derived coverage mode.
- [quad.wgsl](../../../src/render/shader/quad.wgsl) — alpha-only mask coverage.
- `/tmp/pi-spell-mask-pixels-{red,green}.*` — pixel proof for action-style, SpellBook, and CircleMask assets.
- Blizzard UI cache: `~/.cache/wow-ui-sim/blizzard-ui/Blizzard_ActionBar/Mainline/ActionButtonTemplate.xml` — `UI-HUD-ActionBar-IconFrame-Mask` mask on action button icons.

## See Also

- [[minimap-map-ring-alignment]] — earlier minimap masking investigation; its former RGB-coverage claim is superseded.
- [[action-bar-spell-icons]] — earlier action-bar icon rendering failures in draw order and XML texture fields.
- [[texture-atlas]] — texture and atlas loading pipeline used by icon and mask textures.
