# SharedXMLBase utility publication

**Verified in Retail 12.1.5.70783 and Forever 1.60.1.70707:** the
original `Blizzard_SharedXMLBase` TOC allows both screens and publishes
the utilities below. The simulator does not install copies before that
addon loads.

| Utility | Lua publisher |
| --- | --- |
| `CreateAnchor` | `AnchorUtil.lua:58` |
| `GetFinalNameFromTextureKit`, `TextureKitConstants`, `GetMicroIconForRole` | `TextureUtil.lua:350,325,193` |
| `CopyValuesAsKeys`, `GetOrCreateTableEntry` | `TableUtil.lua:395,441` |
| `EnumUtil.MakeEnum` | `EnumUtil.lua:4` |
| `CreateCounter` | `MathUtil.lua:20` |
| `GenerateClosure` | `FunctionUtil.lua:115` |

`Blizzard_Menu` calls `CreateAnchor` at file scope, but its TOC declares
`Blizzard_SharedXMLBase` as a dependency. The simulator's PlayerSpells
readiness check reads `CreateAnchor` when a panel is requested, not at
bootstrap. No simulator Lua/Rust startup caller requires the retired
helpers before their publisher. The unrelated `PingSystemInitializer`
fallback remains for the later Settings provider
(`Blizzard_SettingsDefinitions_Frame/PingSystem.lua:143`).

The Forever closure test uses the original `AnchorUtil.CreateAnchor`
(`AnchorUtil.lua:112`) to anchor a frame and checks its point, parent,
relative point, and offsets. A separate provider check verifies the
original contracts formerly asserted on the temporary copies.
