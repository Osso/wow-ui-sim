# 11.1.5 non-inventory scout

99 rows assigned exactly once: 85 contractual candidates pending, 14 editorial/source rows metadata-only. No runtime credit. IDs refer to generated plaintext; wikitext remains authoritative. `[API LatestInterface]` preserves an unexpanded symbolic example, not current TOC version.

## summary (11)

Security/environment, owner IDs, tooltip producers, pet journal, TOC loading and encoding contracts require independent behavior proofs. Existing methods/registrations do not establish all claims. Inspect loader/TOC and secure-hook policies, then use concrete addon/load and DTO fixtures.

- `prose-undated-004` — * A new user-configurable color override system has been added. Currently, this system is limited to configuring item quality colors.
- `prose-undated-005` — * A new C_EncodingUtil namespace has been added with native APIs for common data compression, encoding, and serialization tasks.
- `prose-undated-006` — * A new LoadSavedVariablesFirst TOC directive has been added that will load saved variables before all scripts in an addon.
- `prose-undated-007` — * The AllowLoadGameType TOC directive is no longer restricted and can be used by insecure addons.
- `prose-undated-008` — * The TOC file format has been expanded with support for inline variables and directives.
- `prose-undated-009` — * SecureActionButtonTemplate now supports a per-button configurable useOnKeyDown attribute.
- `prose-undated-010` — * The UnitCreatureFamily and UnitCreatureType APIs now return an additional locale-independent value. Additionally, new functions have been added to the C_CreatureInfo namespace.
- `prose-undated-011` — * A new C_PetJournal.GetOwnedPetIDs API has been added to obtain a full list of a player's owned battle pets.
- `prose-undated-012` — * C_TooltipInfo APIs now include additional line data items for item and upgrade levels (e.g. Explorer 1/8).
- `prose-undated-013` — * All base Lua library functions (such as collectgarbage) can no longer be securely hooked.
- `prose-undated-014` — * SecureFrameTemplate now explicitly blocks mouse click event propagation.

## color-overrides (4)

Existing GetColorForQuality default is a placeholder. Need real per-quality override state, events, ITEM_QUALITY_COLORS integration and |cnIQ token rendering before credit.

- `prose-undated-024` — * Users can now reconfigure the colors used in the UI for displaying item qualities.
- `prose-undated-025` — * The existing ITEM_QUALITY_COLORS global table will be updated dynamically based upon user configuration, however it may be preferable to instead use the ColorManager.GetColorDataForItemQuality API instead in case this table is later removed.
- `prose-undated-026` — * Original item quality colors without user overrides can be obtained via the ColorManager.GetDefaultColorDataForItemQuality API.
- `prose-undated-027` — * For cases where item quality colors are displayed in strings, new markup support has been added in the form of |cnIQn:text|r where n corresponds to a numeric Enum.ItemQuality value.

## toc-and-inheritance (18)

Verify selected files under actual profile/game variables, insecure AllowLoadGameType and saved-variable ordering. Code sample comments describe behavior, not editorial build context. Warning generation needs duplicate inherited handler dispatch plus enabled/disabled CVar evidence; a published CVar is insufficient.

- `prose-undated-031` — * File lines now support inline variables that expand out to the current client flavor and game type.
- `prose-undated-032` — * File lines now support inline directives that can be used to omit loading individual files under specific client flavors and game types.
- `prose-undated-033` — ** This is limited to the AllowLoad and AllowLoadGameType directives.
- `prose-undated-035` — ## Interface: [API LatestInterface]
- `prose-undated-036` — ## Title: My Cool Addon
- `prose-undated-038` — # This will load "Mainline\File.lua" or "Classic\File.lua"
- `prose-undated-039` — # as appropriate for the current client.
- `prose-undated-040` — [Family]\File.lua
- `prose-undated-042` — # This will load "Standard\File.lua", "Mists\File.lua", "Cata\File.lua", ...
- `prose-undated-043` — # as appropriate for the current client.
- `prose-undated-044` — [Game]\File.lua
- `prose-undated-046` — # This will only be loaded on Mainline.
- `prose-undated-047` — MainlineOnly.lua [AllowLoadGameType mainline]
- `prose-undated-049` — # This will only be loaded in Vanilla or TBC.
- `prose-undated-050` — VanillaOrTBC.lua [AllowLoadGameType vanilla, tbc]
- `prose-undated-054` — * The client can now be configured to raise script warnings when a frame defined in XML would trigger duplicate calls to identical functions in script handlers.
- `prose-undated-055` — ** Typically, this occurs when using the inherit attribute on script handlers in conjunction with template inheritance.
- `prose-undated-056` — * This functionality is currently disabled by default, and can be configured via CVar frameScriptFunctionInheritanceWarningMode.

## enums (34)

Parent + member occurrences need exact current values, historical additions and actual consumer behavior. Generated Enum data presence does not prove the change or its runtime use. Never infer numeric values from names.

- `enumerations-Enum-AccountTransType-061` — Enum.AccountTransType
- `enumerations-Enum-AccountTransType-062` — + Profile
- `enumerations-Enum-CraftingOrderFlags-063` — Enum.CraftingOrderFlags
- `enumerations-Enum-CraftingOrderFlags-064` — + None
- `enumerations-Enum-CurrencySource-065` — Enum.CurrencySource
- `enumerations-Enum-CurrencySource-066` — + RenownRepGainInitialVisibility
- `enumerations-Enum-EditModeAccountSetting-067` — Enum.EditModeAccountSetting
- `enumerations-Enum-EditModeAccountSetting-068` — + ShowCooldownViewer
- `enumerations-Enum-EditModeSystem-069` — Enum.EditModeSystem
- `enumerations-Enum-EditModeSystem-070` — + CooldownViewer
- `enumerations-Enum-EventToastDisplayType-071` — Enum.EventToastDisplayType
- `enumerations-Enum-EventToastDisplayType-072` — + Scoreboard
- `enumerations-Enum-GossipNpcOption-073` — Enum.GossipNpcOption
- `enumerations-Enum-GossipNpcOption-074` — + GuildRename
- `enumerations-Enum-GossipNpcOption-075` — + ItemUpgrade
- `enumerations-Enum-ItemGemColor-076` — Enum.ItemGemColor
- `enumerations-Enum-ItemGemColor-077` — + FutureUse
- `enumerations-Enum-ItemSocketType-078` — Enum.ItemSocketType
- `enumerations-Enum-ItemSocketType-079` — + FutureUse
- `enumerations-Enum-PerksVendorCategoryType-080` — Enum.PerksVendorCategoryType
- `enumerations-Enum-PerksVendorCategoryType-081` — + Stipend
- `enumerations-Enum-PerksVendorCategoryType-082` — + Activity
- `enumerations-Enum-PerksVendorCategoryType-083` — + GmAdjustment
- `enumerations-Enum-PerksVendorCategoryType-084` — + Achievement
- `enumerations-Enum-PerksVendorCategoryType-085` — + Refund
- `enumerations-Enum-PlayerInteractionType-086` — Enum.PlayerInteractionType
- `enumerations-Enum-PlayerInteractionType-087` — + GuildRename
- `enumerations-Enum-PointsModifierSourceType-088` — Enum.PointsModifierSourceType
- `enumerations-Enum-PointsModifierSourceType-089` — + RaidEncounterLevel
- `enumerations-Enum-TooltipDataLineType-090` — Enum.TooltipDataLineType
- `enumerations-Enum-TooltipDataLineType-091` — + ItemLevel
- `enumerations-Enum-TooltipDataLineType-092` — + ItemUpgradeLevel
- `enumerations-Enum-UIWidgetUpdateAnimType-093` — Enum.UIWidgetUpdateAnimType
- `enumerations-Enum-UIWidgetUpdateAnimType-094` — + FlashAndAnimateNumber

## structures (18)

DTO additions/removals require concrete source records and returned field shapes. Atlas elementName has an existing producer but is not credited without a dedicated behavior proof. Pet/perks/warband and widget entries need their real producer boundaries; 3D policy remains excluded.

- `structures-AtlasInfo-097` — AtlasInfo
- `structures-AtlasInfo-098` — + elementName
- `structures-PerksVendorItemInfo-099` — PerksVendorItemInfo
- `structures-PerksVendorItemInfo-100` — + invType
- `structures-PerksVendorItemInfo-101` — + quality
- `structures-PerksVendorSubItemInfo-102` — PerksVendorSubItemInfo
- `structures-PerksVendorSubItemInfo-103` — # itemAppearanceID -> itemModifiedAppearanceID
- `structures-PlayerChoiceOptionInfo-104` — PlayerChoiceOptionInfo
- `structures-PlayerChoiceOptionInfo-105` — - rarityColor
- `structures-SpellCooldownInfo-106` — SpellCooldownInfo
- `structures-SpellCooldownInfo-107` — + activeCategory
- `structures-TextureAndTextVisualizationInfo-108` — TextureAndTextVisualizationInfo
- `structures-TextureAndTextVisualizationInfo-109` — + textFormatType
- `structures-TextureAndTextVisualizationInfo-110` — + updateAnimType
- `structures-TotemInfoScript-111` — TotemInfoScript
- `structures-TotemInfoScript-112` — + spellID
- `structures-WarbandSceneEntry-113` — WarbandSceneEntry
- `structures-WarbandSceneEntry-114` — - qualityColor

## Editorial/source context (14)

Headings, resource links, page title and explicit build transition receive no runtime credit.

- `source-context-001` — Patch 11.1.5 API changes
- `source-context-003` — == Summary ==
- `source-context-016` — == Resources ==
- `source-context-017` — * TOC: 110105
- `source-context-018` — * Official patch notes: 11.1.5 Update Notes
- `source-context-019` — * Diffs: wow-ui-source, BlizzardInterfaceResources
- `source-context-020` — * Deprecated API: Blizzard_Deprecated
- `source-context-022` — == Color Overrides ==
- `source-context-029` — == TOC format changes ==
- `source-context-052` — == Function Inheritance Warnings ==
- `source-context-058` — == Consolidated changes ==
- `source-context-059` — : 11.1.0 (59466) → 11.1.5 (61265) Jun  3 2025
- `source-context-060` — === Enumerations ===
- `source-context-096` — === Structures ===
