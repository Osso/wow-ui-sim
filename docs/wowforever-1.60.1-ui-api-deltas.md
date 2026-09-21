# WoW Forever 1.60.1.69913: UI and API changes

Source-oriented notes on the **WoW client/UI contract** for addon-runtime and API-schema maintainers, including wowless.

## Summary

- **Forever is Mainline-family UI with a separate `camelot` game type.** Interface `16001` does not imply the Classic-era Lua/UI surface.
- **Mainline is not synonymous with Retail/`standard`.** Camelot selects its own constants, templates, data and implementations inside otherwise shared addons.
- **API namespace and execution environment matter.** The combat-event getter is documented under Internal/Secure namespaces, not public `C_CombatLog`; manufacturing legacy aliases can select the wrong addon path.
- **There are concrete differences from the pinned Retail and PTR schemas:** nine character/account bank indices, three additional bank functions, five additional SpellBook functions, and gamepad/input-style surfaces.
- **Modern security annotations, tooltip data handling and PlayerSpells coexist with Classic-like content.** Neither a Classic schema nor an unmodified Retail API superset is a safe description.

## Baselines and evidence

| Snapshot | Build | Pinned source |
| --- | --- | --- |
| Forever | **1.60.1.69913** | [Gethe `70ef1b2f`][forever] |
| Retail/live comparison | 12.1.0.69497 | [Gethe `027d26c3`][retail] |
| PTR comparison | 12.1.5.69594 | [Gethe `49b69918`][ptr] |

“Additional” below means present in the Forever snapshot and absent from the corresponding generated documentation in **both of these comparison snapshots**, unless stated otherwise. This is not a claim about later builds.

No pinned Anniversary **2.5.6 / interface 20506** source baseline was available for this comparison. The architectural migration discussion is useful when porting Anniversary addons, but this report does **not** claim an exhaustive Anniversary-to-Forever symbol diff.

Evidence labels:

- **Documented:** generated Blizzard API declarations, including exact names, values, parameters and environment tags.
- **Authored UI:** Blizzard Lua/XML/TOCs demonstrating a consumer or source-selection rule.
- **Wiki-documented:** separately identified community API documentation, not established by the source scan.
- **Addon observation:** a third-party adaptation or report; useful as a lead, not a native contract by itself.

No native client execution is claimed here. Source declarations do not establish every restriction transition, error boundary or default value.

## 1. Product identity and source selection

**Authored UI.** Forever uses both `Mainline/` and `Camelot/` files. `[Family]` and `[Game]` are separate dimensions; `mainline` and `camelot` occur in load filters, while `standard` distinguishes ordinary Retail-specific paths.

Examples:

| Area | What the Forever source selects |
| --- | --- |
| Core constants | Family constants excluded for Camelot; `[Game]/Constants.lua` selected instead |
| World map | Camelot constants, templates and Lua alongside shared Mainline infrastructure |
| Collections | Camelot collection constants and overrides, including different tab/content selection |
| Auction house | Family auction data excluded for Camelot; game-specific auction data selected |
| Shared panels | Camelot panel/progress-bar additions in the shared template addon |

See [FrameXMLBase TOC][framexmlbase], [WorldMap TOC][worldmap], [Collections TOC][collections], [AuctionHouse TOC][auctionhouse], and [SharedXML TOC][sharedxml].

**Interpreter consequence:** retain the distinction between family, game type and interface number. A numeric rule such as “less than 100000 means Classic APIs” selects the wrong surface here. Conversely, selecting Mainline files does not justify enabling every Retail feature.

The pinned [ProjectConstants Lua][projectconstants] defines `WOW_PROJECT_MAINLINE` and `WOW_PROJECT_CLASSIC`, not a new `WOW_PROJECT_FOREVER` constant. That file alone does not establish the value of every engine-defined global; it is not evidence that `WOW_PROJECT_ID` itself is absent.

### Loader details worth preserving

Forever TOCs use conditional `Dep` entries, `[Family]`/`[Game]` expansion, `AllowLoadGameType`/`ExcludeLoadGameType`, and inline `[Bootstrap]` entries. These are also modern Mainline mechanisms, **not all newly invented for Forever**. The [LoadOnDemand combat-log addon][combatlogtoc], for example, lists a bootstrap Lua file and a Mainline-only dependency on the combat-log processor.

Seeing `[Bootstrap]` in a TOC is not evidence for a separate global reorder pass. A source listing also does not by itself establish every LoadOnDemand/bootstrap scheduling rule.

Third-party packages additionally use game-specific filenames such as `Addon_Camelot.toc` and `Addon-Camelot.toc`. Carbonite's [Forever package][carbonite-package] contains `Carbonite-Camelot.toc` (`16001`) alongside a bare Retail `Carbonite.toc` (`120007, 120100`). This is **package evidence** that selecting the bare file indiscriminately loses the intended variant. Native tie-breaking when both separator variants coexist is not established here.

## 2. Combat-log getter ownership: do not reconstruct a public superset

**Documented.** The generated declarations separate three namespaces:

| Namespace | Declared environment | Relevant surface |
| --- | --- | --- |
| `C_CombatLog` | `All` | Public log settings/filtering operations; **does not declare `GetCurrentEventInfo`** |
| `C_CombatLogInternal` | `All` | Declares `GetCurrentEventInfo`; documents `COMBAT_LOG_EVENT_INTERNAL_UNFILTERED` as a synchronous callback event |
| `C_CombatLogSecure` | `SecureOnly` | Declares `GetCurrentEventInfo`, entry traversal/filtering and message construction |

Sources: [public][combat-public], [Internal][combat-internal], [Secure][combat-secure]. `HasRestrictions`, `CallbackEvent`, `SynchronousEvent` and `Environment` are independent pieces of the schema; a callable stub is not an implementation of those restrictions.

**Authored UI.** The [combat-log processor][combat-processor] reads `C_CombatLogSecure.GetCurrentEventInfo()`. The [deprecated wrapper][combat-deprecated] is conditional on `loadDeprecationFallbacks` and contains:

```lua
CombatLogGetCurrentEventInfo = C_CombatLog.GetCurrentEventInfo;
```

This assignment **does not prove the legacy global is a function**. If its source member is absent, the assignment leaves the alias absent. A wrapper file existing, loading, or mentioning a symbol is weaker evidence than that symbol's documented namespace/environment.

**Addon observation.** [EpicDamageMeter 2.39.0][epic-package] chooses its modern `C_DamageMeter` path when `CombatLogGetCurrentEventInfo` is absent and the meter API is available. Exposing an obsolete public getter therefore changes observable feature detection; this is not merely an extra harmless name. The addon report is corroborating evidence, not authority for removing unrelated APIs.

This section establishes the declared separation. It does not claim that every undocumented native alias, secure-environment access rule or combat-event delivery edge has been probed.

## 3. Bank indices differ from Retail

**Documented.** [Forever BagIndex][bags-forever] has 27 values, minimum `-3`, maximum `23`; the [Retail comparison][bags-retail] has 20, minimum `-3`, maximum `16`.

| Entries | Retail comparison | Forever |
| --- | --- | --- |
| `CharacterBankTab_1..6` | `6..11` | `6..11` |
| `CharacterBankTab_7..9` | Not declared | `12..14` |
| `AccountBankTab_1..5` | `12..16` | **`15..19`** |
| `AccountBankTab_6..9` | Not declared | `20..23` |
| `Accountbanktab`, `Characterbanktab`, `Keyring` | `-3`, `-2`, `-1` | `-3`, `-2`, `-1` |
| Backpack, bags 1–4, reagent bag | `0`, `1..4`, `5` | Same |

The account range moves; this is not just appending new enum members. The two bank ranges must not overlap.

### Additional bank functions

The [Forever bank declarations][bank-api] add these functions relative to both pinned Retail/PTR files:

| Function | Declared contract |
| --- | --- |
| `C_Bank.BankBagTypeAndIDToInvSlot(bankType, slotIndex)` | Nullable numeric inventory slot; `slotIndex` is a `luaIndex` |
| `C_Bank.FetchMaxNumBankTabs(bankType)` | Non-null numeric result |
| `C_Bank.ShouldUsePlayerBagsInBank()` | Non-null boolean |

The declaration names `FetchMaxNumBankTabs`' return field `numPurchasedBankTabs`; retain this spelling as schema data rather than silently interpreting it as a second purchased-count operation. `FetchNumPurchasedBankTabs` remains a separate function.

**Authored UI.** [Camelot BankFrame][bank-ui] works with `Enum.BankType.Character`/`Account`, purchased-tab data, maximum-tab queries and bank-to-inventory slot conversion. It is not the old single legacy-bank-bag interface.

**Not implied:** nine enum entries do not mean nine purchased tabs, nine populated containers, a particular slot capacity, or accessible account storage. An addon's private `hasWarbank` flag is not a Blizzard capability API. These state/availability questions need separate evidence.

## 4. SpellBook: Mainline objects with Forever extensions

**Documented.** The [Forever `C_SpellBook` declarations][spellbook-api] contain these five functions not present in either pinned Retail/PTR counterpart:

| Function | Relevant meaning |
| --- | --- |
| `GetClassSkillLineInfo` | May return nothing; otherwise returns `SpellBookSkillLineInfo` |
| `AbortSpellIntro` | Spell-introduction lifecycle operation |
| `SetBarSlotFromIntro` | Action-slot assignment from a spell introduction |
| `IsSpellBookItemLooseFlyoutMember` | Tests representation in a flyout |
| `IsSpellBookItemLowRank` | Tests a lower learned rank when a higher rank is known |

The low-rank operation is a concrete example of Classic-like content represented through a Mainline-style API. Do not infer Retail's presentation/content assumptions from the family alone.

**Authored UI.** Forever has [Camelot PlayerSpells overrides][player-spells] alongside shared SpellBook code. `C_ClassTalents`/`C_Traits`, rather than a wholesale restoration of old talent-tab APIs, are part of this modern UI family. Exact talent trees, specialization populations and profession data are content questions outside this bounded schema comparison.

`SpellBookSkillLineIndex` itself is shared with the pinned Retail snapshot: General `1`, Class `2`, MainSpec `3`, OffSpecStart `4`. It should not be advertised as a Forever-only addition.

## 5. Gamepad action bars and input style

**Documented.** Forever contains [GamepadUI declarations][gamepad-api] absent from the corresponding pinned Retail/PTR source files:

- `C_GamepadUI.GetFirstGamepadActionStorageSlotIndex()`
- `C_GamepadUI.GetFirstGamepadActionBarStorageSlotIndexForActiveStance()` — nullable result
- `C_GamepadUI.GetFirstGamepadPetActionStorageSlotIndex()`
- `C_GamepadUI.IsValidGamepadActionStorageSlotIndex(index)`
- `C_GamepadUI.IsValidGamepadPossessBarStorageSlotIndex(index)`

Two distinct override enums use values `1..12`. Their first values differ:

| Enum | Value `1` |
| --- | --- |
| `GamepadPossessBarOverride` | `SpecialPageTopBar` |
| `GamepadStanceBarOverride` | `None` |

Both then use Page1 left/right/bottom at `2..4`, Page2 top/left/right/bottom at `5..8`, and Page3 top/left/right/bottom at `9..12`. The documented change events carry `(oldOverride, newOverride)` of the corresponding enum.

`Constants.GamepadActionBarConstants` declares 4 slots/group and 2 groups/bar, hence 8 slots/bar; page-unit constants are expressed using these values. Do not confuse storage indices, visible action slots, group counts and override enum values.

[InputInterfaceStyle][input-style] additionally declares `C_InputInterfaceStyle.GetCurrentStyle() -> InputDeviceInterfaceType`. These declarations do not establish numeric storage-base defaults, host input-device delivery or live controller behavior.

## 6. Unit queries retained or added outside the Retail snapshots

**Documented.** The [Forever Unit declarations][unit-api] contain seven global functions absent from both comparison files: `RegionalUniqueNamesEnabled`, `UnitDefenseSkill`, `UnitResistance`, `UnitHasEffectivelyTankAura`, `UnitHasLootInteraction`, `UnitHasMouseoverHighlight`, and `UnitIsInInteractRange`.

This combines Classic-like character-stat queries with newer interaction/name-policy predicates. It is another reason not to treat the API as either the unchanged Classic or unchanged Retail set. Their return values and state transitions are not inferred from the names.

## 7. Security and tooltip migration are Mainline features, not Classic compatibility aliases

**Documented.** Forever's [chat API declarations][chat-api] include `SecretInChatMessagingLockdown`, per-payload `NeverSecret` markers and `SecretArguments` policies. For example, chat-line text/sender queries carry lockdown secrecy metadata, while selected event payload fields such as channel/language names are marked `NeverSecret`. `C_ChatInfo.InChatMessagingLockdown()` declares a boolean return in this snapshot.

A correct schema must preserve the annotations; exact transitions and enforcement still need runtime evidence. Neither “all values are ordinary Lua scalars” nor “every addon event registration is forbidden” follows from these declarations.

**Authored UI.** Forever uses the shared [tooltip data handler][tooltip-handler] and [TooltipUtil][tooltip-util], with `TooltipDataProcessor.AddTooltipPostCall` and displayed-item/spell/unit helpers. This is the modern data/post-call integration model, not proof that every old tooltip method or script has disappeared from all compatibility surfaces.

Likewise, copying an old Classic compatibility addon into a Camelot profile is not neutral: game-type conditions determine whether that publisher is meant to load at all.

## 8. Table security and texture metatable access

**Documented.** [Lua table extensions][table-extensions] include `table.freeze(t) -> t`. The declaration says freezing prevents content/metatable replacement, preserves forwarding through an existing `__newindex`, and restricts tainted callers to tables created by the same addon. This is present in Forever and the pinned PTR snapshot, **not** the pinned Retail 12.1.0 file; it is not uniquely Forever.

[FrameScript][frame-script] documents `securecopy(value, options?)`: table copies preserve cycles and shared references, and copied values receive the current execution taint. This is shared with both comparison snapshots. Treat it as a graph/security operation, not a synonym for an arbitrary recursive table copier.

**Authored UI.** [Mainline UnitFrameUtil][unitframe-util] executes `CopyTable(GetTextureMetatable().__index)` and uses the copied `IsObjectType` method. This establishes a concrete consumer requirement for texture-metatable access. It does not, by itself, specify every mutation, identity or protection property of the returned metatable.

## 9. Lua `require`: a separate loader contract

**Wiki-documented, not established by Blizzard call sites in this source scan.** The pinned [Warcraft Wiki `require` revision][require-wiki] describes a Forever API that is **not stock Lua filesystem/package loading**:

- It looks up an already successfully executed module and returns the original value returned by that file; no return means nil.
- Dotted module names identify addon-relative Lua files. Leading dots resolve relative to the calling file's directory, with additional dots ascending directories without escaping the addon.
- Cross-addon lookup requires a direct required or optional TOC dependency; transitive dependencies are insufficient.
- Dynamically loaded code is documented as exempt from that direct-dependency check.
- Missing modules, escaping relative imports, and missing direct dependencies have distinct errors.

For example, from `ExampleAddon/Core/Logging.lua`, `.Startup` refers to `Core/Startup.lua` and `..UI.Panel` to `UI/Panel.lua`.

No `package.path`, filesystem search, `package.loaded` compatibility, multiple-return policy, module-name case normalization or failed-reload replacement rule is inferred. No Blizzard `require(...)` usage was found in the bounded cached-source scan; native probes remain important for these edge cases.

## 10. What should not be promoted from addon diffs into client rules

| Observation | What it establishes—and what it does not |
| --- | --- |
| A package is tagged Forever | Declared compatibility, not successful loading or native API proof |
| An addon adds a nil guard | A compatibility decision; not proof the API is absent on every Forever build |
| An addon skips timeline or challenge-mode code | A feature choice; not evidence to remove namespaces still present in the generated docs |
| A bank addon disables Warbank UI | Its policy; not a native maximum-tab/purchase/access specification |
| A meter switches from CLEU to `C_DamageMeter` | A consumer migration; inspect the getter's real namespace/environment rather than publishing both paths indiscriminately |

The most actionable interpreter work is therefore product-aware source selection, exact per-build API/enumeration data, environment-aware publication, and the separately documented module-import contract. Native timing, restriction enforcement and content/state defaults should remain explicit open questions—not guessed from a successful addon startup.

[forever]: https://github.com/Gethe/wow-ui-source/tree/70ef1b2fd78061a73f886c4a1e79dc5b5cff6d5e
[retail]: https://github.com/Gethe/wow-ui-source/tree/027d26c3406d3de2cbd2b1f67d468fe033a1bcd4
[ptr]: https://github.com/Gethe/wow-ui-source/tree/49b69918fcdc77e109813281e4f537d45ec7dcbf
[framexmlbase]: https://github.com/Gethe/wow-ui-source/blob/70ef1b2fd78061a73f886c4a1e79dc5b5cff6d5e/Interface/AddOns/Blizzard_FrameXMLBase/Blizzard_FrameXMLBase.toc
[worldmap]: https://github.com/Gethe/wow-ui-source/blob/70ef1b2fd78061a73f886c4a1e79dc5b5cff6d5e/Interface/AddOns/Blizzard_WorldMap/Blizzard_WorldMap_Mainline.toc
[collections]: https://github.com/Gethe/wow-ui-source/blob/70ef1b2fd78061a73f886c4a1e79dc5b5cff6d5e/Interface/AddOns/Blizzard_Collections/Blizzard_Collections.toc
[auctionhouse]: https://github.com/Gethe/wow-ui-source/blob/70ef1b2fd78061a73f886c4a1e79dc5b5cff6d5e/Interface/AddOns/Blizzard_AuctionHouseUI/Blizzard_AuctionHouseUI.toc
[sharedxml]: https://github.com/Gethe/wow-ui-source/blob/70ef1b2fd78061a73f886c4a1e79dc5b5cff6d5e/Interface/AddOns/Blizzard_SharedXML/Blizzard_SharedXML.toc
[projectconstants]: https://github.com/Gethe/wow-ui-source/blob/70ef1b2fd78061a73f886c4a1e79dc5b5cff6d5e/Interface/AddOns/Blizzard_ProjectConstants/ProjectConstants.lua
[combatlogtoc]: https://github.com/Gethe/wow-ui-source/blob/70ef1b2fd78061a73f886c4a1e79dc5b5cff6d5e/Interface/AddOns/Blizzard_CombatLog/Blizzard_CombatLog.toc
[carbonite-package]: https://www.curseforge.com/wow/addons/carbonite/files/8926975
[combat-public]: https://github.com/Gethe/wow-ui-source/blob/70ef1b2fd78061a73f886c4a1e79dc5b5cff6d5e/Interface/AddOns/Blizzard_APIDocumentationGenerated/CombatLogDocumentation.lua
[combat-internal]: https://github.com/Gethe/wow-ui-source/blob/70ef1b2fd78061a73f886c4a1e79dc5b5cff6d5e/Interface/AddOns/Blizzard_APIDocumentationGenerated/CombatLogInternalDocumentation.lua
[combat-secure]: https://github.com/Gethe/wow-ui-source/blob/70ef1b2fd78061a73f886c4a1e79dc5b5cff6d5e/Interface/AddOns/Blizzard_APIDocumentationGenerated/CombatLogSecureDocumentation.lua
[combat-processor]: https://github.com/Gethe/wow-ui-source/blob/70ef1b2fd78061a73f886c4a1e79dc5b5cff6d5e/Interface/AddOns/Blizzard_CombatLogProcessor/Blizzard_CombatLogProcessor.lua
[combat-deprecated]: https://github.com/Gethe/wow-ui-source/blob/70ef1b2fd78061a73f886c4a1e79dc5b5cff6d5e/Interface/AddOns/Blizzard_DeprecatedCombatLog/Deprecated_CombatLog.lua
[epic-package]: https://www.curseforge.com/wow/addons/epic-damage-meter/files/8930362
[bags-forever]: https://github.com/Gethe/wow-ui-source/blob/70ef1b2fd78061a73f886c4a1e79dc5b5cff6d5e/Interface/AddOns/Blizzard_APIDocumentationGenerated/BagIndexConstantsDocumentation.lua
[bags-retail]: https://github.com/Gethe/wow-ui-source/blob/027d26c3406d3de2cbd2b1f67d468fe033a1bcd4/Interface/AddOns/Blizzard_APIDocumentationGenerated/BagIndexConstantsDocumentation.lua
[bank-api]: https://github.com/Gethe/wow-ui-source/blob/70ef1b2fd78061a73f886c4a1e79dc5b5cff6d5e/Interface/AddOns/Blizzard_APIDocumentationGenerated/BankDocumentation.lua
[bank-ui]: https://github.com/Gethe/wow-ui-source/blob/70ef1b2fd78061a73f886c4a1e79dc5b5cff6d5e/Interface/AddOns/Blizzard_UIPanels_Game/Camelot/BankFrame.lua
[spellbook-api]: https://github.com/Gethe/wow-ui-source/blob/70ef1b2fd78061a73f886c4a1e79dc5b5cff6d5e/Interface/AddOns/Blizzard_APIDocumentationGenerated/SpellBookDocumentation.lua
[player-spells]: https://github.com/Gethe/wow-ui-source/blob/70ef1b2fd78061a73f886c4a1e79dc5b5cff6d5e/Interface/AddOns/Blizzard_PlayerSpells/Camelot/SpellBook/Blizzard_SpellBookFrame.lua
[gamepad-api]: https://github.com/Gethe/wow-ui-source/blob/70ef1b2fd78061a73f886c4a1e79dc5b5cff6d5e/Interface/AddOns/Blizzard_APIDocumentationGenerated/GamepadUIDocumentation.lua
[input-style]: https://github.com/Gethe/wow-ui-source/blob/70ef1b2fd78061a73f886c4a1e79dc5b5cff6d5e/Interface/AddOns/Blizzard_APIDocumentationGenerated/InputInterfaceStyleDocumentation.lua
[chat-api]: https://github.com/Gethe/wow-ui-source/blob/70ef1b2fd78061a73f886c4a1e79dc5b5cff6d5e/Interface/AddOns/Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua
[tooltip-handler]: https://github.com/Gethe/wow-ui-source/blob/70ef1b2fd78061a73f886c4a1e79dc5b5cff6d5e/Interface/AddOns/Blizzard_SharedXMLGame/Tooltip/TooltipDataHandler.lua
[tooltip-util]: https://github.com/Gethe/wow-ui-source/blob/70ef1b2fd78061a73f886c4a1e79dc5b5cff6d5e/Interface/AddOns/Blizzard_SharedXMLGame/Tooltip/TooltipUtil.lua
[unit-api]: https://github.com/Gethe/wow-ui-source/blob/70ef1b2fd78061a73f886c4a1e79dc5b5cff6d5e/Interface/AddOns/Blizzard_APIDocumentationGenerated/UnitDocumentation.lua
[table-extensions]: https://github.com/Gethe/wow-ui-source/blob/70ef1b2fd78061a73f886c4a1e79dc5b5cff6d5e/Interface/AddOns/Blizzard_APIDocumentationGenerated/LuaTableExtensionsDocumentation.lua
[frame-script]: https://github.com/Gethe/wow-ui-source/blob/70ef1b2fd78061a73f886c4a1e79dc5b5cff6d5e/Interface/AddOns/Blizzard_APIDocumentationGenerated/FrameScriptDocumentation.lua
[unitframe-util]: https://github.com/Gethe/wow-ui-source/blob/70ef1b2fd78061a73f886c4a1e79dc5b5cff6d5e/Interface/AddOns/Blizzard_UnitFrame/Mainline/UnitFrameUtil.lua
[require-wiki]: https://warcraft.wiki.gg/wiki/API:require?oldid=6879760
