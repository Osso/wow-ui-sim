# Held-row reassessment — 2026-10-03

Read-only source inspection; no tests, cargo, repository writes, git mutation, agents or model CLIs. Verdicts are feasibility, not acceptance or runtime proof.

## Exact register deltas

- `prose-2026-03-12-032` (Blue posts), source lines [32]: Midnight 12.0.5 PTR Changes Episode 1: The Secret Menace — `Added GetTransmogOutfitIndex to the restricted environment. Values returned from this can be used with the new /outfit slash command.`
- `prose-2026-03-12-055` (Blue posts), source lines [55]: Midnight 12.0.5 PTR Changes Episode 1: The Secret Menace — `We created the Private Aura system back in 10.1 as a means to prevent addons from automating certain very specific fight mechanics. In some ways Private Auras can be viewed as the beginning of our experimentation with adjusting addon capabilities, and while they certainly had their issues, for the most part they served their purpose (albeit in a rather heavy-handed way). When we started developing the Secret Value system, we had hoped (planned even) to be able to essentially retire the Private Aura system and replace it with just secrets.`
- `prose-2026-03-12-056` (Blue posts), source lines [56]: Midnight 12.0.5 PTR Changes Episode 1: The Secret Menace — `Unfortunately, as Midnight Alpha & Beta progressed it became clear that relying on Secret Values alone was not going to be enough to prevent addon automation involving auras. And so, we ended up having to go in almost the exact opposite direction: we made MOST encounter debuffs private auras rather than just a select few. While this approach gave us the security we needed for Midnight Season 1, it also came with some significant downsides. Our own private aura displays don't currently have full parity with normal auras, and they aren’t supported in nameplates. On top of that they also require our encounter designers to be very diligent about flagging almost every encounter debuff as private, which is cumbersome and very easy to get wrong.`
- `prose-2026-03-25-074` (Blue posts), source lines [74]: Midnight 12.0.5 PTR Changes Episode 2: Attack of the Secrets — `The following APIs will no longer be able to be called by addons during combat: PromoteToLeader, PromoteToAssistant, DemoteAssistant, SetEveryoneIsAssistant, DoReadyCheck, ConfirmReadyCheck, ConvertToParty, ConvertToRaid, ConfirmConvertToRaid, C_PartyInfo.DoCountdown, C_PartyInfo.SetRestrictPings, C_PartyInfo.SetLootMethod.`
- `prose-2026-03-25-076` (Blue posts), source lines [76]: Midnight 12.0.5 PTR Changes Episode 2: Attack of the Secrets — `Fixing a bug where some players randomly become unable to set raid markers until they restart their client.`
- `prose-2026-03-25-088` (Blue posts), source lines [88]: Midnight 12.0.5 PTR Changes Episode 2: Attack of the Secrets — `UnitIsUnit now returns secrets if either unit in the comparison is "targettarget" or "focustarget".`
- `prose-2026-03-25-117` (Blue posts), source lines [117]: Midnight 12.0.5 PTR Changes Episode 2: Attack of the Secrets — `Existing restrictions will remain on the AddPrivateAuraAppliedSound and RemovePrivateAuraAppliedSound APIs.`
- `global api-C_ActionBar-IsOnBarOrSpecialBar-245` (Global API), source lines [245]: C_ActionBar.IsOnBarOrSpecialBar — `# arg1.Type number -> SpellIdentifier`
- `global api-C_CatalogShop-GetProductInfo-247` (Global API), source lines [247]: C_CatalogShop.GetProductInfo — `+ HasRestrictions`
- `global api-C_CatalogShop-PurchaseProduct-249` (Global API), source lines [249]: C_CatalogShop.PurchaseProduct — `+ HasRestrictions`
- `global api-C_ChatInfo-ReplaceIconAndGroupExpressions-253` (Global API), source lines [253]: C_ChatInfo.ReplaceIconAndGroupExpressions — `+ arg2,3 NeverSecret`
- `global api-C_DelvesUI-GetTieredEntrancePDEID-260` (Global API), source lines [260]: C_DelvesUI.GetTieredEntrancePDEID — `- MayReturnNothing`
- `global api-C_HouseExterior-SelectCoreFixtureOption-270` (Global API), source lines [270]: C_HouseExterior.SelectCoreFixtureOption — `+ arg2 = attachedDecorAction`
- `global api-C_HousingBasicMode-StartPlacingNewDecor-278` (Global API), source lines [278]: C_HousingBasicMode.StartPlacingNewDecor — `# arg1.Name catalogEntryID -> catalogEntryVariantID`
- `global api-C_HousingBasicMode-StartPlacingNewDecor-279` (Global API), source lines [279]: C_HousingBasicMode.StartPlacingNewDecor — `# arg1.Type HousingCatalogEntryID -> HousingCatalogEntryVariantID`
- `global api-C_HousingCatalog-DestroyEntry-281` (Global API), source lines [281]: C_HousingCatalog.DestroyEntry — `# arg1.Name entryID -> entryVariantID`
- `global api-C_HousingCatalog-DestroyEntry-282` (Global API), source lines [282]: C_HousingCatalog.DestroyEntry — `# arg1.Type HousingCatalogEntryID -> HousingCatalogEntryVariantID`
- `global api-C_MountJournal-GetMountFromSpell-291` (Global API), source lines [291]: C_MountJournal.GetMountFromSpell — `# arg1.Type number -> SpellIdentifier`
- `global api-C_TooltipInfo-GetUnitBuff-342` (Global API), source lines [342]: C_TooltipInfo.GetUnitBuff — `- arg1 NeverSecret`
- `global api-C_TooltipInfo-GetUnitDebuff-347` (Global API), source lines [347]: C_TooltipInfo.GetUnitDebuff — `- arg1 NeverSecret`
- `widgets-PlayerModel-SetUnit-534` (Widgets), source lines [534]: PlayerModel:SetUnit — `+ RequiresDeclassifiedUnitIdentity`
- `widgets-ModelSceneActorBase-SetModelByUnit-542` (Widgets), source lines [542]: ModelSceneActorBase:SetModelByUnit — `+ RequiresDeclassifiedUnitIdentity`
- `widgets-FontString-GetFont-544` (Widgets), source lines [544]: FontString:GetFont — `# ret1.Type cstring -> FontAsset`
- `widgets-FontString-SetFont-546` (Widgets), source lines [546]: FontString:SetFont — `# arg1.Type cstring -> FontAsset`
- `structures-HousingCatalogEntryInfo-650` (Structures), source lines [650]: HousingCatalogEntryInfo — `+ totalNumStored`
- `structures-HousingCatalogEntryInfo-651` (Structures), source lines [651]: HousingCatalogEntryInfo — `+ totalNumPlaced`

## Current provider search anchors

```
src/c_api/c_spell.rs:18: //! - `GetMountFromSpell(spellID)` → scans `world.mounts` for matching spell
src/c_api/c_spell.rs:98:     ("GetMountFromSpell", get_mount_from_spell),
src/c_api/c_spell.rs:502: /// `C_Spell.GetMountFromSpell(spellID)` → mountID or nil.
src/c_api/c_action_bar_spell_slots.rs:25:         "IsOnBarOrSpecialBar",
src/c_api/c_action_bar_spell_slots.rs:75:     let has_slots = query_public_direct_spell_membership(state, "C_ActionBar.IsOnBarOrSpecialBar")?;
src/c_api/c_tooltip_info_aura_instance.rs:16:     table_set_rust_fn_static(state, namespace, "GetUnitBuffByAuraInstanceID", get_buff)?;
src/c_api/c_tooltip_info_aura_instance.rs:20:         "GetUnitDebuffByAuraInstanceID",
src/c_api/c_tooltip_info_aura_instance.rs:33:         "C_TooltipInfo.GetUnitBuffByAuraInstanceID",
src/c_api/c_tooltip_info_aura_instance.rs:41:         "C_TooltipInfo.GetUnitDebuffByAuraInstanceID",
src/c_api/c_mount_spell_lookup.rs:14:     table_set_rust_fn_static(state, namespace, "GetMountFromSpell", get_mount_from_spell)
src/c_api/c_mount_spell_lookup.rs:18:     let spell_id = read_public_spell_identifier_at(state, 1, "C_MountJournal.GetMountFromSpell")?;
src/c_api/c_tooltip_info_indexed_aura.rs:19:     table_set_rust_fn_static(state, namespace, "GetUnitBuff", get_unit_buff)?;
src/c_api/c_tooltip_info_indexed_aura.rs:20:     table_set_rust_fn_static(state, namespace, "GetUnitDebuff", get_unit_debuff)
src/c_api/c_tooltip_info_indexed_aura.rs:24:     get_indexed_aura(state, "C_TooltipInfo.GetUnitBuff", AuraFilter::Helpful)
src/c_api/c_tooltip_info_indexed_aura.rs:28:     get_indexed_aura(state, "C_TooltipInfo.GetUnitDebuff", AuraFilter::Harmful)
src/c_api/c_catalog_shop_products.rs:130:     table_set_rust_fn_static(state, namespace, "GetProductInfo", product_info)?;
src/lua_api/env_init/runtime_surface_bootstrap.lua:468: C_StoreSecure.GetProductInfo = function(productID)
src/lua_api/env_init/runtime_surface_bootstrap.lua:522:   local product = C_StoreSecure.GetProductInfo(productID)
src/lua_api/env_init/runtime_surface_bootstrap.lua:912: if rawget(C_StoreSecure, "GetProductInfo") == nil then
src/lua_api/env_init/runtime_surface_bootstrap.lua:913:   function C_StoreSecure.GetProductInfo(productID) return __wow_store_product(productID) end
src/lua_api/env_init/runtime_surface_bootstrap.lua:1022: if rawget(C_StoreSecure, "PurchaseProduct") == nil then
src/lua_api/env_init/runtime_surface_bootstrap.lua:1023:   function C_StoreSecure.PurchaseProduct(productID)
src/lua_api/env_init/runtime_surface_bootstrap.lua:1027: if rawget(C_StoreSecure, "PurchaseProductConfirm") == nil then
src/lua_api/env_init/runtime_surface_bootstrap.lua:1028:   function C_StoreSecure.PurchaseProductConfirm(confirm, _dollars, _cents)
src/lua_api/workarounds/temporary/secure_transfer_state.rs:41:         housingVCPurchaseProductID = 0,
src/lua_api/workarounds/temporary/secure_transfer_state.rs:81: if rawget(C_SecureTransfer, "GetHousingVCPurchaseProductID") == nil then
src/lua_api/workarounds/temporary/secure_transfer_state.rs:82:     function C_SecureTransfer.GetHousingVCPurchaseProductID()
src/lua_api/workarounds/temporary/secure_transfer_state.rs:83:         return tonumber(C_SecureTransfer._state.housingVCPurchaseProductID) or 0
src/lua_api/workarounds/temporary/secure_transfer_state.rs:144:                 C_SecureTransfer._state.housingVCPurchaseProductID = "77"
src/lua_api/workarounds/temporary/secure_transfer_state.rs:168:                 if C_SecureTransfer.GetHousingVCPurchaseProductID() ~= 77 then
src/lua_api/workarounds/temporary/c_chat_info_defaults.rs:27: installChatInfoDefault("ReplaceIconAndGroupExpressions", function(text)
src/lua_api/workarounds/temporary/c_chat_info_defaults.rs:87:                 if C_ChatInfo.ReplaceIconAndGroupExpressions("hello") ~= "hello" then return "replace" end
src/lua_api/workarounds/temporary/c_chat_info_defaults.rs:120:             function C_ChatInfo.ReplaceIconAndGroupExpressions(text)
src/lua_api/workarounds/temporary/c_chat_info_defaults.rs:134:                     C_ChatInfo.ReplaceIconAndGroupExpressions("hello")
src/lua_api/workarounds/temporary/housing_catalog_state.lua:54:     totalNumStored = 1,
src/lua_api/workarounds/temporary/housing_catalog_state.lua:56:     totalNumPlaced = 1,
src/lua_api/workarounds/temporary/housing_catalog_state.lua:81:     totalNumStored = 1,
src/lua_api/workarounds/temporary/housing_catalog_state.lua:83:     totalNumPlaced = 1,
src/lua_api/workarounds/temporary/housing_catalog_state.lua:643:   PurchaseProduct = __wow_noop,
src/lua_api/workarounds/temporary/housing_catalog_state.lua:899:   SelectCoreFixtureOption = __wow_noop,
src/lua_api/globals/missing_surface/delves_ui.rs:100:     ("GetTieredEntrancePDEID", get_tiered_entrance_pde_id),
src/lua_api/globals/stubs/namespace_stubs.rs:89:     // C_Spell GetMountFromSpell / GetSpellInfo are SimState/spell-data-backed in
src/lua_api/globals/action_bar_api/registration.rs:12:     ("IsOnBarOrSpecialBar", is_on_bar_or_special_bar),
src/lua_api/globals/missing_surface/tooltip_info/mod.rs:243:             ("GetUnitBuff", c_tooltip_get_unit_buff),
src/lua_api/globals/missing_surface/tooltip_info/mod.rs:246:                 "GetUnitBuffByAuraInstanceID",
src/lua_api/globals/missing_surface/tooltip_info/mod.rs:250:             ("GetUnitDebuff", c_tooltip_get_unit_debuff),
src/lua_api/globals/missing_surface/tooltip_info/mod.rs:253:                 "GetUnitDebuffByAuraInstanceID",
src/lua_api/frame/methods/widgets/tooltip/content.rs:532:     let _ = populate_tooltip_from_method(state, tooltip_id, "GetUnitBuff", &args, None)?;
src/lua_api/frame/methods/widgets/tooltip/content.rs:546:         "GetUnitBuffByAuraInstanceID",
src/lua_api/frame/methods/widgets/tooltip/content.rs:560:     let _ = populate_tooltip_from_method(state, tooltip_id, "GetUnitDebuff", &args, None)?;
src/lua_api/frame/methods/widgets/tooltip/content.rs:574:         "GetUnitDebuffByAuraInstanceID",
src/c_api/c_housing/basic_mode.rs:18:         ("StartPlacingNewDecor", start_placing_new_decor),
src/c_api/c_housing/catalog/queries.rs:36:         ("DestroyEntry", super::storage::destroy_entry),
src/c_api/c_housing/catalog/snapshot.rs:116:         ("totalNumStored", record.total_num_stored),
src/c_api/c_housing/catalog/snapshot.rs:117:         ("totalNumPlaced", record.total_num_placed),
src/c_api/c_housing/catalog/destroy_input.rs:1: //! DestroyEntry-only AllowedWhenUntainted inputs; shared catalog guards stay public.
```

## Accounting snapshot (not new proof)

- global api-C_ActionBar-IsOnBarOrSpecialBar-245: audit-pending; B78 bounded PUBLIC direct-spell membership accepted670:11new+17companion PASS, startup[], scopedfmt/check0. Explicit aliases/effective assignments now modeled; native AllowedWhenTainted permission/resultsecrecy and special-bar membership remain unmodeled. Exact source stays pending; no full identifier/annotation/native/all-profile credit.
- global api-C_CatalogShop-GetProductInfo-247: audit-pending; Source retained; behavioral applicability audit not completed.
- global api-C_CatalogShop-PurchaseProduct-249: audit-pending; Source retained; behavioral applicability audit not completed.
- global api-C_ChatInfo-ReplaceIconAndGroupExpressions-253: audit-pending; Source retained; behavioral applicability audit not completed.
- global api-C_DelvesUI-GetTieredEntrancePDEID-260: audit-pending; Source retained; behavioral applicability audit not completed.
- global api-C_HouseExterior-SelectCoreFixtureOption-270: audit-pending; Source retained; behavioral applicability audit not completed.
- global api-C_HousingBasicMode-StartPlacingNewDecor-278: audit-pending; Batch71 independent574 bounded PASS at d83666b13: original-selector/all-field AllowedWhenUntainted VM authentication, secure actual NUM/TABLE full-key pending requests, addon denial/taint, guarded-table/root/GC and model isolation. Compiled RED13PASS7FAIL; current-equivalent20pending+69catalog=89distinctRetailPASS/startup0[], final scopedfmt/defaultcheck0zeroDiagnostics. Inferred eligibility/cancel/parser policy and unmodeled finish/instance/stock/events do not establish real placement delta; exact278/279 retain audit-pending. Native/full-placement/all-profile and historical dirty/globalfmt/process limits remain.
- global api-C_HousingBasicMode-StartPlacingNewDecor-279: audit-pending; Batch71 independent574 bounded PASS at d83666b13: original-selector/all-field AllowedWhenUntainted VM authentication, secure actual NUM/TABLE full-key pending requests, addon denial/taint, guarded-table/root/GC and model isolation. Compiled RED13PASS7FAIL; current-equivalent20pending+69catalog=89distinctRetailPASS/startup0[], final scopedfmt/defaultcheck0zeroDiagnostics. Inferred eligibility/cancel/parser policy and unmodeled finish/instance/stock/events do not establish real placement delta; exact278/279 retain audit-pending. Native/full-placement/all-profile and historical dirty/globalfmt/process limits remain.
- global api-C_HousingCatalog-DestroyEntry-281: audit-pending; Batch72 independent582 bounded PASS at ffb1845bc: DestroyEntry-only original selector/destroyAll and all-field VM authentication, secure actual NUM/BOOL/TABLE full-key deletion, addon denial/taint, guard/root/GC and existing eligible-count/event isolation. Compiled RED14PASS10FAIL;35freshRetailPASS=24destruction+11Admin/storage,startup0[],scopedfmt/defaultcheck0zeroDiagnostics. Mixed eligible subset/noop/consistency/synchronous timing remain inferred, not native all-stack proof;281/282 retain audit-pending. No broader/catalog/native/all-profile/source-row closure.
- global api-C_HousingCatalog-DestroyEntry-282: audit-pending; Batch72 independent582 bounded PASS at ffb1845bc: DestroyEntry-only original selector/destroyAll and all-field VM authentication, secure actual NUM/BOOL/TABLE full-key deletion, addon denial/taint, guard/root/GC and existing eligible-count/event isolation. Compiled RED14PASS10FAIL;35freshRetailPASS=24destruction+11Admin/storage,startup0[],scopedfmt/defaultcheck0zeroDiagnostics. Mixed eligible subset/noop/consistency/synchronous timing remain inferred, not native all-stack proof;281/282 retain audit-pending. No broader/catalog/native/all-profile/source-row closure.
- global api-C_MountJournal-GetMountFromSpell-291: audit-pending; B77 bounded public journal lookup accepted654:16local/16desktop PASS and check; independent674 accepts separate headless startup and3CASCtexture decodes with provenance/GUI/content-version limits. Native AllowedWhenTainted and alias grammar remain unmodeled; exact291 stays pending. No whole-row/native/profile/full-suite credit.
- global api-C_TooltipInfo-GetUnitBuff-342: audit-pending; B79 bounded input/indexed-model acceptance690+694:18 refreshed+38 inherited controls PASS, startup[], scopedfmt/check and currentrootidentity assertion. Secure secret selector now queries actual live player/party state, not ignored-unit/player fallback; tainted auth precedes parse/lookup. Unit-aura access/restricted output and native/general-unit/profile parity remain unmodeled; exact342 stays pending. No whole-row/native/fullsuite credit.
- global api-C_TooltipInfo-GetUnitDebuff-347: audit-pending; B81 main-accepted707 bounded input/live harmful-model proof:15 new+56 refreshed controls=71 distinct PASS, startup[], scopedfmt/defaultcheck. All3 original secure selectors drive player/party harmful filtered state; tainted auth precedes parse/lookup, not secret-to-empty masking. Unit-aura access/restricted output unmodeled; target/general-unit/native/profile parity unverified and inferred parser/filter/miss policies explicit. Exact347 remains pending; arg2/arg3 are no additional retained removals, no whole-row/fullsuite credit.
- widgets-PlayerModel-SetUnit-534: audit-pending; Source retained; behavioral applicability audit not completed.
- widgets-ModelSceneActorBase-SetModelByUnit-542: audit-pending; Source retained; behavioral applicability audit not completed.
- widgets-FontString-GetFont-544: audit-pending; Source retained; behavioral applicability audit not completed.
- widgets-FontString-SetFont-546: audit-pending; Source retained; behavioral applicability audit not completed.
- structures-HousingCatalogEntryInfo-650: audit-pending; Batch24 bounded explicit-field publication only; linked aggregate spec owns saved76PASS/fresh gates and limits. Status retained audit-pending: None→nil missing-data simulator gap against cached required native numbers. No synchronization/variant derivation/full DTO/nonempty wrapper/native/all-profile or whole-row completion.
- structures-HousingCatalogEntryInfo-651: audit-pending; Batch24 bounded explicit-field publication only; linked aggregate spec owns saved76PASS/fresh gates and limits. Status retained audit-pending: None→nil missing-data simulator gap against cached required native numbers. No synchronization/variant derivation/full DTO/nonempty wrapper/native/all-profile or whole-row completion.

## Cached retail declarations (verbatim local cache)

```lua
HousingBasicModeUIDocumentation.lua:182: 			},
HousingBasicModeUIDocumentation.lua:183: 		},
HousingBasicModeUIDocumentation.lua:184: 		{
HousingBasicModeUIDocumentation.lua:185: 			Name = "StartPlacingNewDecor",
HousingBasicModeUIDocumentation.lua:186: 			Type = "Function",
HousingBasicModeUIDocumentation.lua:187: 			SecretArguments = "AllowedWhenUntainted",
HousingBasicModeUIDocumentation.lua:188: 
HousingBasicModeUIDocumentation.lua:189: 			Arguments =
HousingBasicModeUIDocumentation.lua:190: 			{
HousingBasicModeUIDocumentation.lua:191: 				{ Name = "catalogEntryVariantID", Type = "HousingCatalogEntryVariantID", Nilable = false },
HousingBasicModeUIDocumentation.lua:192: 			},
HousingBasicModeUIDocumentation.lua:193: 		},
HousingBasicModeUIDocumentation.lua:194: 		{
HousingBasicModeUIDocumentation.lua:195: 			Name = "StartPlacingPreviewDecor",
HousingBasicModeUIDocumentation.lua:196: 			Type = "Function",
HousingBasicModeUIDocumentation.lua:197: 			SecretArguments = "AllowedWhenUntainted",
HousingBasicModeUIDocumentation.lua:198: 
HousingBasicModeUIDocumentation.lua:199: 			Arguments =
HousingBasicModeUIDocumentation.lua:200: 			{
HousingBasicModeUIDocumentation.lua:201: 				{ Name = "decorRecordID", Type = "number", Nilable = false },
HousingBasicModeUIDocumentation.lua:202: 				{ Name = "bundleCatalogShopProductID", Type = "number", Nilable = true },
HousingBasicModeUIDocumentation.lua:203: 			},
HousingBasicModeUIDocumentation.lua:204: 		},
HousingBasicModeUIDocumentation.lua:205: 	},
HousingBasicModeUIDocumentation.lua:206: 
HousingBasicModeUIDocumentation.lua:207: 	Events =
HousingBasicModeUIDocumentation.lua:208: 	{
HousingBasicModeUIDocumentation.lua:209: 		{
HousingBasicModeUIDocumentation.lua:210: 			Name = "HousingBasicModeHoveredTargetChanged",
HousingBasicModeUIDocumentation.lua:211: 			Type = "Event",
HousingBasicModeUIDocumentation.lua:212: 			LiteralName = "HOUSING_BASIC_MODE_HOVERED_TARGET_CHANGED",
HousingBasicModeUIDocumentation.lua:213: 			SynchronousEvent = true,
HousingBasicModeUIDocumentation.lua:214: 			Payload =
HousingBasicModeUIDocumentation.lua:215: 			{
HousingBasicModeUIDocumentation.lua:216: 				{ Name = "hasHoveredTarget", Type = "bool", Nilable = false },
HousingBasicModeUIDocumentation.lua:217: 				{ Name = "targetType", Type = "HousingBasicModeTargetType", Nilable = false },
HousingBasicModeUIDocumentation.lua:218: 			},
HousingBasicModeUIDocumentation.lua:219: 		},
HousingBasicModeUIDocumentation.lua:220: 		{
HousingBasicModeUIDocumentation.lua:221: 			Name = "HousingBasicModePlacementFlagsUpdated",
HousingBasicModeUIDocumentation.lua:222: 			Type = "Event",
HousingBasicModeUIDocumentation.lua:223: 			LiteralName = "HOUSING_BASIC_MODE_PLACEMENT_FLAGS_UPDATED",
HousingBasicModeUIDocumentation.lua:224: 			SynchronousEvent = true,
HousingBasicModeUIDocumentation.lua:225: 			Payload =
HousingBasicModeUIDocumentation.lua:226: 			{
HousingBasicModeUIDocumentation.lua:227: 				{ Name = "targetType", Type = "HousingBasicModeTargetType", Nilable = false },
HousingBasicModeUIDocumentation.lua:228: 				{ Name = "activeFlags", Type = "HousingDecorPlacementRestriction", Nilable = false },
HousingBasicModeUIDocumentation.lua:229: 			},
HousingBasicModeUIDocumentation.lua:230: 		},
HousingBasicModeUIDocumentation.lua:231: 		{
HousingBasicModeUIDocumentation.lua:232: 			Name = "HousingBasicModeSelectedTargetChanged",

CatalogShopDocumentation.lua:223: 			},
CatalogShopDocumentation.lua:224: 		},
CatalogShopDocumentation.lua:225: 		{
CatalogShopDocumentation.lua:226: 			Name = "GetProductInfo",
CatalogShopDocumentation.lua:227: 			Type = "Function",
CatalogShopDocumentation.lua:228: 			HasRestrictions = true,
CatalogShopDocumentation.lua:229: 			SecretArguments = "AllowedWhenUntainted",
CatalogShopDocumentation.lua:230: 
CatalogShopDocumentation.lua:231: 			Arguments =
CatalogShopDocumentation.lua:232: 			{
CatalogShopDocumentation.lua:233: 				{ Name = "productID", Type = "number", Nilable = false },
CatalogShopDocumentation.lua:234: 			},
CatalogShopDocumentation.lua:235: 
CatalogShopDocumentation.lua:236: 			Returns =
CatalogShopDocumentation.lua:237: 			{
CatalogShopDocumentation.lua:238: 				{ Name = "productInfo", Type = "CatalogShopProductInfo", Nilable = true },
CatalogShopDocumentation.lua:239: 			},
CatalogShopDocumentation.lua:240: 		},
CatalogShopDocumentation.lua:241: 		{
CatalogShopDocumentation.lua:242: 			Name = "GetProductSortOrder",
CatalogShopDocumentation.lua:243: 			Type = "Function",
CatalogShopDocumentation.lua:244: 			SecretArguments = "AllowedWhenUntainted",
CatalogShopDocumentation.lua:245: 
CatalogShopDocumentation.lua:246: 			Arguments =
CatalogShopDocumentation.lua:247: 			{
CatalogShopDocumentation.lua:248: 				{ Name = "categoryID", Type = "number", Nilable = false },
CatalogShopDocumentation.lua:249: 				{ Name = "sectionID", Type = "number", Nilable = false },
CatalogShopDocumentation.lua:250: 				{ Name = "productID", Type = "number", Nilable = false },
CatalogShopDocumentation.lua:251: 			},
CatalogShopDocumentation.lua:252: 
CatalogShopDocumentation.lua:253: 			Returns =
CatalogShopDocumentation.lua:254: 			{
CatalogShopDocumentation.lua:255: 				{ Name = "sortOrder", Type = "number", Nilable = true },
CatalogShopDocumentation.lua:256: 			},
CatalogShopDocumentation.lua:257: 		},
CatalogShopDocumentation.lua:258: 		{
CatalogShopDocumentation.lua:259: 			Name = "GetRefundableDecors",
CatalogShopDocumentation.lua:260: 			Type = "Function",
CatalogShopDocumentation.lua:261: 			SecretArguments = "AllowedWhenUntainted",
CatalogShopDocumentation.lua:262: 
CatalogShopDocumentation.lua:263: 			Arguments =
CatalogShopDocumentation.lua:264: 			{
CatalogShopDocumentation.lua:265: 				{ Name = "productIdFilterOpt", Type = "number", Nilable = true },
CatalogShopDocumentation.lua:266: 			},
CatalogShopDocumentation.lua:267: 
CatalogShopDocumentation.lua:268: 			Returns =
CatalogShopDocumentation.lua:269: 			{
CatalogShopDocumentation.lua:270: 				{ Name = "refundableDecorInfos", Type = "table", InnerType = "RefundableDecorInfo", Nilable = false },
CatalogShopDocumentation.lua:271: 				{ Name = "minTimeRemainingSeconds", Type = "time_t", Nilable = false },
CatalogShopDocumentation.lua:272: 			},
CatalogShopDocumentation.lua:273: 		},

CatalogShopDocumentation.lua:418: 			},
CatalogShopDocumentation.lua:419: 		},
CatalogShopDocumentation.lua:420: 		{
CatalogShopDocumentation.lua:421: 			Name = "PurchaseProduct",
CatalogShopDocumentation.lua:422: 			Type = "Function",
CatalogShopDocumentation.lua:423: 			HasRestrictions = true,
CatalogShopDocumentation.lua:424: 			SecretArguments = "AllowedWhenUntainted",
CatalogShopDocumentation.lua:425: 
CatalogShopDocumentation.lua:426: 			Arguments =
CatalogShopDocumentation.lua:427: 			{
CatalogShopDocumentation.lua:428: 				{ Name = "productID", Type = "number", Nilable = false },
CatalogShopDocumentation.lua:429: 			},
CatalogShopDocumentation.lua:430: 
CatalogShopDocumentation.lua:431: 			Returns =
CatalogShopDocumentation.lua:432: 			{
CatalogShopDocumentation.lua:433: 				{ Name = "canPurchase", Type = "bool", Nilable = false },
CatalogShopDocumentation.lua:434: 			},
CatalogShopDocumentation.lua:435: 		},
CatalogShopDocumentation.lua:436: 		{
CatalogShopDocumentation.lua:437: 			Name = "RefreshRefundableDecors",
CatalogShopDocumentation.lua:438: 			Type = "Function",
CatalogShopDocumentation.lua:439: 		},
CatalogShopDocumentation.lua:440: 		{
CatalogShopDocumentation.lua:441: 			Name = "RefreshVirtualCurrencyBalance",
CatalogShopDocumentation.lua:442: 			Type = "Function",
CatalogShopDocumentation.lua:443: 			SecretArguments = "AllowedWhenUntainted",
CatalogShopDocumentation.lua:444: 
CatalogShopDocumentation.lua:445: 			Arguments =
CatalogShopDocumentation.lua:446: 			{
CatalogShopDocumentation.lua:447: 				{ Name = "currencyCode", Type = "string", Nilable = false },
CatalogShopDocumentation.lua:448: 			},
CatalogShopDocumentation.lua:449: 		},
CatalogShopDocumentation.lua:450: 		{
CatalogShopDocumentation.lua:451: 			Name = "ShouldShowHousingWarning",
CatalogShopDocumentation.lua:452: 			Type = "Function",
CatalogShopDocumentation.lua:453: 
CatalogShopDocumentation.lua:454: 			Returns =
CatalogShopDocumentation.lua:455: 			{
CatalogShopDocumentation.lua:456: 				{ Name = "shouldShowHousingWarning", Type = "bool", Nilable = false },
CatalogShopDocumentation.lua:457: 			},
CatalogShopDocumentation.lua:458: 		},
CatalogShopDocumentation.lua:459: 		{
CatalogShopDocumentation.lua:460: 			Name = "StartHousingVCPurchaseConfirmation",
CatalogShopDocumentation.lua:461: 			Type = "Function",
CatalogShopDocumentation.lua:462: 			SecretArguments = "AllowedWhenUntainted",
CatalogShopDocumentation.lua:463: 
CatalogShopDocumentation.lua:464: 			Arguments =
CatalogShopDocumentation.lua:465: 			{
CatalogShopDocumentation.lua:466: 				{ Name = "productID", Type = "number", Nilable = false },
CatalogShopDocumentation.lua:467: 			},
CatalogShopDocumentation.lua:468: 		},

MountJournalDocumentation.lua:246: 			},
MountJournalDocumentation.lua:247: 		},
MountJournalDocumentation.lua:248: 		{
MountJournalDocumentation.lua:249: 			Name = "GetMountFromSpell",
MountJournalDocumentation.lua:250: 			Type = "Function",
MountJournalDocumentation.lua:251: 			SecretArguments = "AllowedWhenTainted",
MountJournalDocumentation.lua:252: 
MountJournalDocumentation.lua:253: 			Arguments =
MountJournalDocumentation.lua:254: 			{
MountJournalDocumentation.lua:255: 				{ Name = "spellID", Type = "SpellIdentifier", Nilable = false },
MountJournalDocumentation.lua:256: 			},
MountJournalDocumentation.lua:257: 
MountJournalDocumentation.lua:258: 			Returns =
MountJournalDocumentation.lua:259: 			{
MountJournalDocumentation.lua:260: 				{ Name = "mountID", Type = "number", Nilable = true },
MountJournalDocumentation.lua:261: 			},
MountJournalDocumentation.lua:262: 		},
MountJournalDocumentation.lua:263: 		{
MountJournalDocumentation.lua:264: 			Name = "GetMountIDs",
MountJournalDocumentation.lua:265: 			Type = "Function",
MountJournalDocumentation.lua:266: 
MountJournalDocumentation.lua:267: 			Returns =
MountJournalDocumentation.lua:268: 			{
MountJournalDocumentation.lua:269: 				{ Name = "mountIDs", Type = "table", InnerType = "number", Nilable = false },
MountJournalDocumentation.lua:270: 			},
MountJournalDocumentation.lua:271: 		},
MountJournalDocumentation.lua:272: 		{
MountJournalDocumentation.lua:273: 			Name = "GetMountInfoByID",
MountJournalDocumentation.lua:274: 			Type = "Function",
MountJournalDocumentation.lua:275: 			MayReturnNothing = true,
MountJournalDocumentation.lua:276: 			SecretArguments = "AllowedWhenUntainted",
MountJournalDocumentation.lua:277: 
MountJournalDocumentation.lua:278: 			Arguments =
MountJournalDocumentation.lua:279: 			{
MountJournalDocumentation.lua:280: 				{ Name = "mountID", Type = "number", Nilable = false },
MountJournalDocumentation.lua:281: 			},
MountJournalDocumentation.lua:282: 
MountJournalDocumentation.lua:283: 			Returns =
MountJournalDocumentation.lua:284: 			{
MountJournalDocumentation.lua:285: 				{ Name = "name", Type = "cstring", Nilable = false },
MountJournalDocumentation.lua:286: 				{ Name = "spellID", Type = "number", Nilable = false },
MountJournalDocumentation.lua:287: 				{ Name = "icon", Type = "fileID", Nilable = false },
MountJournalDocumentation.lua:288: 				{ Name = "isActive", Type = "bool", Nilable = false },
MountJournalDocumentation.lua:289: 				{ Name = "isUsable", Type = "bool", Nilable = false },
MountJournalDocumentation.lua:290: 				{ Name = "sourceType", Type = "number", Nilable = false },
MountJournalDocumentation.lua:291: 				{ Name = "isFavorite", Type = "bool", Nilable = false },
MountJournalDocumentation.lua:292: 				{ Name = "isFactionSpecific", Type = "bool", Nilable = false },
MountJournalDocumentation.lua:293: 				{ Name = "faction", Type = "PvPFaction", Nilable = true },
MountJournalDocumentation.lua:294: 				{ Name = "shouldHideOnChar", Type = "bool", Nilable = false },
MountJournalDocumentation.lua:295: 				{ Name = "isCollected", Type = "bool", Nilable = false },
MountJournalDocumentation.lua:296: 				{ Name = "mountID", Type = "number", Nilable = false },

ChatInfoDocumentation.lua:482: 			},
ChatInfoDocumentation.lua:483: 		},
ChatInfoDocumentation.lua:484: 		{
ChatInfoDocumentation.lua:485: 			Name = "ReplaceIconAndGroupExpressions",
ChatInfoDocumentation.lua:486: 			Type = "Function",
ChatInfoDocumentation.lua:487: 			SecretArguments = "AllowedWhenTainted",
ChatInfoDocumentation.lua:488: 
ChatInfoDocumentation.lua:489: 			Arguments =
ChatInfoDocumentation.lua:490: 			{
ChatInfoDocumentation.lua:491: 				{ Name = "input", Type = "string", Nilable = false },
ChatInfoDocumentation.lua:492: 				{ Name = "noIconReplacement", Type = "bool", Nilable = true, NeverSecret = true },
ChatInfoDocumentation.lua:493: 				{ Name = "noGroupReplacement", Type = "bool", Nilable = true, NeverSecret = true },
ChatInfoDocumentation.lua:494: 			},
ChatInfoDocumentation.lua:495: 
ChatInfoDocumentation.lua:496: 			Returns =
ChatInfoDocumentation.lua:497: 			{
ChatInfoDocumentation.lua:498: 				{ Name = "output", Type = "string", Nilable = false },
ChatInfoDocumentation.lua:499: 			},
ChatInfoDocumentation.lua:500: 		},
ChatInfoDocumentation.lua:501: 		{
ChatInfoDocumentation.lua:502: 			Name = "RequestCanLocalWhisperTarget",
ChatInfoDocumentation.lua:503: 			Type = "Function",
ChatInfoDocumentation.lua:504: 			SecretArguments = "AllowedWhenUntainted",
ChatInfoDocumentation.lua:505: 
ChatInfoDocumentation.lua:506: 			Arguments =
ChatInfoDocumentation.lua:507: 			{
ChatInfoDocumentation.lua:508: 				{ Name = "whisperTarget", Type = "WOWGUID", Nilable = false },
ChatInfoDocumentation.lua:509: 			},
ChatInfoDocumentation.lua:510: 		},
ChatInfoDocumentation.lua:511: 		{
ChatInfoDocumentation.lua:512: 			Name = "ResetDefaultZoneChannels",
ChatInfoDocumentation.lua:513: 			Type = "Function",
ChatInfoDocumentation.lua:514: 		},
ChatInfoDocumentation.lua:515: 		{
ChatInfoDocumentation.lua:516: 			Name = "SendAddonMessage",
ChatInfoDocumentation.lua:517: 			Type = "Function",
ChatInfoDocumentation.lua:518: 			SecretArguments = "NotAllowed",
ChatInfoDocumentation.lua:519: 			Documentation = { "Sends a text payload to other clients specified by chatChannel and target which are registered to listen for prefix." },
ChatInfoDocumentation.lua:520: 
ChatInfoDocumentation.lua:521: 			Arguments =
ChatInfoDocumentation.lua:522: 			{
ChatInfoDocumentation.lua:523: 				{ Name = "prefix", Type = "cstring", Nilable = false },
ChatInfoDocumentation.lua:524: 				{ Name = "message", Type = "cstring", Nilable = false },
ChatInfoDocumentation.lua:525: 				{ Name = "chatType", Type = "cstring", Nilable = true, Documentation = { "ChatType, defaults to SLASH_CMD_PARTY." } },
ChatInfoDocumentation.lua:526: 				{ Name = "target", Type = "cstring", Nilable = true, Documentation = { "Only applies for targeted channels" } },
ChatInfoDocumentation.lua:527: 			},
ChatInfoDocumentation.lua:528: 
ChatInfoDocumentation.lua:529: 			Returns =
ChatInfoDocumentation.lua:530: 			{
ChatInfoDocumentation.lua:531: 				{ Name = "result", Type = "SendAddonMessageResult", Nilable = false },
ChatInfoDocumentation.lua:532: 			},

TooltipInfoDocumentation.lua:1236: 			},
TooltipInfoDocumentation.lua:1237: 		},
TooltipInfoDocumentation.lua:1238: 		{
TooltipInfoDocumentation.lua:1239: 			Name = "GetUnitBuff",
TooltipInfoDocumentation.lua:1240: 			Type = "Function",
TooltipInfoDocumentation.lua:1241: 			MayReturnNothing = true,
TooltipInfoDocumentation.lua:1242: 			RequiresUnitAuraAccess = true,
TooltipInfoDocumentation.lua:1243: 			SecretWhenUnitAuraRestricted = true,
TooltipInfoDocumentation.lua:1244: 			SecretArguments = "AllowedWhenUntainted",
TooltipInfoDocumentation.lua:1245: 
TooltipInfoDocumentation.lua:1246: 			Arguments =
TooltipInfoDocumentation.lua:1247: 			{
TooltipInfoDocumentation.lua:1248: 				{ Name = "unitToken", Type = "UnitTokenRestrictedForAddOns", Nilable = false },
TooltipInfoDocumentation.lua:1249: 				{ Name = "index", Type = "luaIndex", Nilable = false },
TooltipInfoDocumentation.lua:1250: 				{ Name = "filter", Type = "AuraFilters", Nilable = true },
TooltipInfoDocumentation.lua:1251: 			},
TooltipInfoDocumentation.lua:1252: 
TooltipInfoDocumentation.lua:1253: 			Returns =
TooltipInfoDocumentation.lua:1254: 			{
TooltipInfoDocumentation.lua:1255: 				{ Name = "data", Type = "TooltipData", Nilable = false },
TooltipInfoDocumentation.lua:1256: 			},
TooltipInfoDocumentation.lua:1257: 		},
TooltipInfoDocumentation.lua:1258: 		{
TooltipInfoDocumentation.lua:1259: 			Name = "GetUnitBuffByAuraInstanceID",
TooltipInfoDocumentation.lua:1260: 			Type = "Function",
TooltipInfoDocumentation.lua:1261: 			MayReturnNothing = true,
TooltipInfoDocumentation.lua:1262: 			RequiresUnitAuraAccess = true,
TooltipInfoDocumentation.lua:1263: 			SecretWhenUnitAuraRestricted = true,
TooltipInfoDocumentation.lua:1264: 			SecretArguments = "AllowedWhenUntainted",
TooltipInfoDocumentation.lua:1265: 
TooltipInfoDocumentation.lua:1266: 			Arguments =
TooltipInfoDocumentation.lua:1267: 			{
TooltipInfoDocumentation.lua:1268: 				{ Name = "unitToken", Type = "UnitTokenRestrictedForAddOns", Nilable = false },
TooltipInfoDocumentation.lua:1269: 				{ Name = "auraInstanceID", Type = "number", Nilable = false },
TooltipInfoDocumentation.lua:1270: 				{ Name = "filter", Type = "AuraFilters", Nilable = true },
TooltipInfoDocumentation.lua:1271: 			},
TooltipInfoDocumentation.lua:1272: 
TooltipInfoDocumentation.lua:1273: 			Returns =
TooltipInfoDocumentation.lua:1274: 			{
TooltipInfoDocumentation.lua:1275: 				{ Name = "data", Type = "TooltipData", Nilable = false },
TooltipInfoDocumentation.lua:1276: 			},
TooltipInfoDocumentation.lua:1277: 		},
TooltipInfoDocumentation.lua:1278: 		{
TooltipInfoDocumentation.lua:1279: 			Name = "GetUnitDebuff",
TooltipInfoDocumentation.lua:1280: 			Type = "Function",
TooltipInfoDocumentation.lua:1281: 			MayReturnNothing = true,
TooltipInfoDocumentation.lua:1282: 			RequiresUnitAuraAccess = true,
TooltipInfoDocumentation.lua:1283: 			SecretWhenUnitAuraRestricted = true,
TooltipInfoDocumentation.lua:1284: 			SecretArguments = "AllowedWhenUntainted",
TooltipInfoDocumentation.lua:1285: 
TooltipInfoDocumentation.lua:1286: 			Arguments =

TooltipInfoDocumentation.lua:1276: 			},
TooltipInfoDocumentation.lua:1277: 		},
TooltipInfoDocumentation.lua:1278: 		{
TooltipInfoDocumentation.lua:1279: 			Name = "GetUnitDebuff",
TooltipInfoDocumentation.lua:1280: 			Type = "Function",
TooltipInfoDocumentation.lua:1281: 			MayReturnNothing = true,
TooltipInfoDocumentation.lua:1282: 			RequiresUnitAuraAccess = true,
TooltipInfoDocumentation.lua:1283: 			SecretWhenUnitAuraRestricted = true,
TooltipInfoDocumentation.lua:1284: 			SecretArguments = "AllowedWhenUntainted",
TooltipInfoDocumentation.lua:1285: 
TooltipInfoDocumentation.lua:1286: 			Arguments =
TooltipInfoDocumentation.lua:1287: 			{
TooltipInfoDocumentation.lua:1288: 				{ Name = "unitToken", Type = "UnitTokenRestrictedForAddOns", Nilable = false },
TooltipInfoDocumentation.lua:1289: 				{ Name = "index", Type = "luaIndex", Nilable = false },
TooltipInfoDocumentation.lua:1290: 				{ Name = "filter", Type = "AuraFilters", Nilable = true },
TooltipInfoDocumentation.lua:1291: 			},
TooltipInfoDocumentation.lua:1292: 
TooltipInfoDocumentation.lua:1293: 			Returns =
TooltipInfoDocumentation.lua:1294: 			{
TooltipInfoDocumentation.lua:1295: 				{ Name = "data", Type = "TooltipData", Nilable = false },
TooltipInfoDocumentation.lua:1296: 			},
TooltipInfoDocumentation.lua:1297: 		},
TooltipInfoDocumentation.lua:1298: 		{
TooltipInfoDocumentation.lua:1299: 			Name = "GetUnitDebuffByAuraInstanceID",
TooltipInfoDocumentation.lua:1300: 			Type = "Function",
TooltipInfoDocumentation.lua:1301: 			MayReturnNothing = true,
TooltipInfoDocumentation.lua:1302: 			RequiresUnitAuraAccess = true,
TooltipInfoDocumentation.lua:1303: 			SecretWhenUnitAuraRestricted = true,
TooltipInfoDocumentation.lua:1304: 			SecretArguments = "AllowedWhenUntainted",
TooltipInfoDocumentation.lua:1305: 
TooltipInfoDocumentation.lua:1306: 			Arguments =
TooltipInfoDocumentation.lua:1307: 			{
TooltipInfoDocumentation.lua:1308: 				{ Name = "unitToken", Type = "UnitTokenRestrictedForAddOns", Nilable = false },
TooltipInfoDocumentation.lua:1309: 				{ Name = "auraInstanceID", Type = "number", Nilable = false },
TooltipInfoDocumentation.lua:1310: 				{ Name = "filter", Type = "AuraFilters", Nilable = true },
TooltipInfoDocumentation.lua:1311: 			},
TooltipInfoDocumentation.lua:1312: 
TooltipInfoDocumentation.lua:1313: 			Returns =
TooltipInfoDocumentation.lua:1314: 			{
TooltipInfoDocumentation.lua:1315: 				{ Name = "data", Type = "TooltipData", Nilable = false },
TooltipInfoDocumentation.lua:1316: 			},
TooltipInfoDocumentation.lua:1317: 		},
TooltipInfoDocumentation.lua:1318: 		{
TooltipInfoDocumentation.lua:1319: 			Name = "GetUpgradeItem",
TooltipInfoDocumentation.lua:1320: 			Type = "Function",
TooltipInfoDocumentation.lua:1321: 			MayReturnNothing = true,
TooltipInfoDocumentation.lua:1322: 
TooltipInfoDocumentation.lua:1323: 			Returns =
TooltipInfoDocumentation.lua:1324: 			{
TooltipInfoDocumentation.lua:1325: 				{ Name = "data", Type = "TooltipData", Nilable = false },
TooltipInfoDocumentation.lua:1326: 			},

DelvesUIDocumentation.lua:312: 			},
DelvesUIDocumentation.lua:313: 		},
DelvesUIDocumentation.lua:314: 		{
DelvesUIDocumentation.lua:315: 			Name = "GetTieredEntrancePDEID",
DelvesUIDocumentation.lua:316: 			Type = "Function",
DelvesUIDocumentation.lua:317: 
DelvesUIDocumentation.lua:318: 			Returns =
DelvesUIDocumentation.lua:319: 			{
DelvesUIDocumentation.lua:320: 				{ Name = "pdeID", Type = "number", Nilable = false },
DelvesUIDocumentation.lua:321: 			},
DelvesUIDocumentation.lua:322: 		},
DelvesUIDocumentation.lua:323: 		{
DelvesUIDocumentation.lua:324: 			Name = "GetTieredEntranceType",
DelvesUIDocumentation.lua:325: 			Type = "Function",
DelvesUIDocumentation.lua:326: 
DelvesUIDocumentation.lua:327: 			Returns =
DelvesUIDocumentation.lua:328: 			{
DelvesUIDocumentation.lua:329: 				{ Name = "entranceType", Type = "TieredEntranceType", Nilable = false },
DelvesUIDocumentation.lua:330: 			},
DelvesUIDocumentation.lua:331: 		},
DelvesUIDocumentation.lua:332: 		{
DelvesUIDocumentation.lua:333: 			Name = "GetTraitTreeForCompanion",
DelvesUIDocumentation.lua:334: 			Type = "Function",
DelvesUIDocumentation.lua:335: 			SecretArguments = "AllowedWhenUntainted",
DelvesUIDocumentation.lua:336: 
DelvesUIDocumentation.lua:337: 			Arguments =
DelvesUIDocumentation.lua:338: 			{
DelvesUIDocumentation.lua:339: 				{ Name = "companionID", Type = "number", Nilable = true },
DelvesUIDocumentation.lua:340: 			},
DelvesUIDocumentation.lua:341: 
DelvesUIDocumentation.lua:342: 			Returns =
DelvesUIDocumentation.lua:343: 			{
DelvesUIDocumentation.lua:344: 				{ Name = "treeID", Type = "number", Nilable = false },
DelvesUIDocumentation.lua:345: 			},
DelvesUIDocumentation.lua:346: 		},
DelvesUIDocumentation.lua:347: 		{
DelvesUIDocumentation.lua:348: 			Name = "GetUnseenCuriosBySlotType",
DelvesUIDocumentation.lua:349: 			Type = "Function",
DelvesUIDocumentation.lua:350: 			SecretArguments = "AllowedWhenUntainted",
DelvesUIDocumentation.lua:351: 
DelvesUIDocumentation.lua:352: 			Arguments =
DelvesUIDocumentation.lua:353: 			{
DelvesUIDocumentation.lua:354: 				{ Name = "slotType", Type = "CompanionConfigSlotTypes", Nilable = false },
DelvesUIDocumentation.lua:355: 				{ Name = "ownedCurioNodeIDs", Type = "table", InnerType = "number", Nilable = false },
DelvesUIDocumentation.lua:356: 			},
DelvesUIDocumentation.lua:357: 
DelvesUIDocumentation.lua:358: 			Returns =
DelvesUIDocumentation.lua:359: 			{
DelvesUIDocumentation.lua:360: 				{ Name = "unseenCurioNodeIDs", Type = "table", InnerType = "number", Nilable = false },
DelvesUIDocumentation.lua:361: 			},
DelvesUIDocumentation.lua:362: 		},

SimpleMessageFrameAPIDocumentation.lua:69: 			},
SimpleMessageFrameAPIDocumentation.lua:70: 		},
SimpleMessageFrameAPIDocumentation.lua:71: 		{
SimpleMessageFrameAPIDocumentation.lua:72: 			Name = "GetFont",
SimpleMessageFrameAPIDocumentation.lua:73: 			Type = "Function",
SimpleMessageFrameAPIDocumentation.lua:74: 
SimpleMessageFrameAPIDocumentation.lua:75: 			Arguments =
SimpleMessageFrameAPIDocumentation.lua:76: 			{
SimpleMessageFrameAPIDocumentation.lua:77: 			},
SimpleMessageFrameAPIDocumentation.lua:78: 
SimpleMessageFrameAPIDocumentation.lua:79: 			Returns =
SimpleMessageFrameAPIDocumentation.lua:80: 			{
SimpleMessageFrameAPIDocumentation.lua:81: 				{ Name = "fontFile", Type = "cstring", Nilable = false },
SimpleMessageFrameAPIDocumentation.lua:82: 				{ Name = "height", Type = "uiFontHeight", Nilable = false },
SimpleMessageFrameAPIDocumentation.lua:83: 				{ Name = "flags", Type = "TBFFlags", Nilable = false },
SimpleMessageFrameAPIDocumentation.lua:84: 			},
SimpleMessageFrameAPIDocumentation.lua:85: 		},
SimpleMessageFrameAPIDocumentation.lua:86: 		{
SimpleMessageFrameAPIDocumentation.lua:87: 			Name = "GetFontObject",
SimpleMessageFrameAPIDocumentation.lua:88: 			Type = "Function",
SimpleMessageFrameAPIDocumentation.lua:89: 
SimpleMessageFrameAPIDocumentation.lua:90: 			Arguments =
SimpleMessageFrameAPIDocumentation.lua:91: 			{
SimpleMessageFrameAPIDocumentation.lua:92: 			},
SimpleMessageFrameAPIDocumentation.lua:93: 
SimpleMessageFrameAPIDocumentation.lua:94: 			Returns =
SimpleMessageFrameAPIDocumentation.lua:95: 			{
SimpleMessageFrameAPIDocumentation.lua:96: 				{ Name = "font", Type = "SimpleFont", Nilable = false },
SimpleMessageFrameAPIDocumentation.lua:97: 			},
SimpleMessageFrameAPIDocumentation.lua:98: 		},
SimpleMessageFrameAPIDocumentation.lua:99: 		{
SimpleMessageFrameAPIDocumentation.lua:100: 			Name = "GetFontStringByID",
SimpleMessageFrameAPIDocumentation.lua:101: 			Type = "Function",
SimpleMessageFrameAPIDocumentation.lua:102: 			ConstSecretAccessor = true,
SimpleMessageFrameAPIDocumentation.lua:103: 			SecretArguments = "AllowedWhenUntainted",
SimpleMessageFrameAPIDocumentation.lua:104: 
SimpleMessageFrameAPIDocumentation.lua:105: 			Arguments =
SimpleMessageFrameAPIDocumentation.lua:106: 			{
SimpleMessageFrameAPIDocumentation.lua:107: 				{ Name = "messageID", Type = "number", Nilable = false },
SimpleMessageFrameAPIDocumentation.lua:108: 			},
SimpleMessageFrameAPIDocumentation.lua:109: 
SimpleMessageFrameAPIDocumentation.lua:110: 			Returns =
SimpleMessageFrameAPIDocumentation.lua:111: 			{
SimpleMessageFrameAPIDocumentation.lua:112: 				{ Name = "fontString", Type = "SimpleFontString", Nilable = false },
SimpleMessageFrameAPIDocumentation.lua:113: 			},
SimpleMessageFrameAPIDocumentation.lua:114: 		},
SimpleMessageFrameAPIDocumentation.lua:115: 		{
SimpleMessageFrameAPIDocumentation.lua:116: 			Name = "GetIndentedWordWrap",
SimpleMessageFrameAPIDocumentation.lua:117: 			Type = "Function",
SimpleMessageFrameAPIDocumentation.lua:118: 
SimpleMessageFrameAPIDocumentation.lua:119: 			Arguments =

SimpleMessageFrameAPIDocumentation.lua:292: 			},
SimpleMessageFrameAPIDocumentation.lua:293: 		},
SimpleMessageFrameAPIDocumentation.lua:294: 		{
SimpleMessageFrameAPIDocumentation.lua:295: 			Name = "SetFont",
SimpleMessageFrameAPIDocumentation.lua:296: 			Type = "Function",
SimpleMessageFrameAPIDocumentation.lua:297: 			RequiresValidFontAsset = true,
SimpleMessageFrameAPIDocumentation.lua:298: 			RequiresValidFontHeight = true,
SimpleMessageFrameAPIDocumentation.lua:299: 			SecretArguments = "AllowedWhenUntainted",
SimpleMessageFrameAPIDocumentation.lua:300: 
SimpleMessageFrameAPIDocumentation.lua:301: 			Arguments =
SimpleMessageFrameAPIDocumentation.lua:302: 			{
SimpleMessageFrameAPIDocumentation.lua:303: 				{ Name = "fontFile", Type = "cstring", Nilable = false },
SimpleMessageFrameAPIDocumentation.lua:304: 				{ Name = "height", Type = "uiFontHeight", Nilable = false },
SimpleMessageFrameAPIDocumentation.lua:305: 				{ Name = "flags", Type = "TBFFlags", Nilable = false },
SimpleMessageFrameAPIDocumentation.lua:306: 			},
SimpleMessageFrameAPIDocumentation.lua:307: 		},
SimpleMessageFrameAPIDocumentation.lua:308: 		{
SimpleMessageFrameAPIDocumentation.lua:309: 			Name = "SetFontObject",
SimpleMessageFrameAPIDocumentation.lua:310: 			Type = "Function",
SimpleMessageFrameAPIDocumentation.lua:311: 			SecretArguments = "AllowedWhenUntainted",
SimpleMessageFrameAPIDocumentation.lua:312: 
SimpleMessageFrameAPIDocumentation.lua:313: 			Arguments =
SimpleMessageFrameAPIDocumentation.lua:314: 			{
SimpleMessageFrameAPIDocumentation.lua:315: 				{ Name = "font", Type = "SimpleFont", Nilable = false },
SimpleMessageFrameAPIDocumentation.lua:316: 			},
SimpleMessageFrameAPIDocumentation.lua:317: 		},
SimpleMessageFrameAPIDocumentation.lua:318: 		{
SimpleMessageFrameAPIDocumentation.lua:319: 			Name = "SetIndentedWordWrap",
SimpleMessageFrameAPIDocumentation.lua:320: 			Type = "Function",
SimpleMessageFrameAPIDocumentation.lua:321: 			SecretArguments = "AllowedWhenUntainted",
SimpleMessageFrameAPIDocumentation.lua:322: 
SimpleMessageFrameAPIDocumentation.lua:323: 			Arguments =
SimpleMessageFrameAPIDocumentation.lua:324: 			{
SimpleMessageFrameAPIDocumentation.lua:325: 				{ Name = "wordWrap", Type = "bool", Nilable = false },
SimpleMessageFrameAPIDocumentation.lua:326: 			},
SimpleMessageFrameAPIDocumentation.lua:327: 		},
SimpleMessageFrameAPIDocumentation.lua:328: 		{
SimpleMessageFrameAPIDocumentation.lua:329: 			Name = "SetInsertMode",
SimpleMessageFrameAPIDocumentation.lua:330: 			Type = "Function",
SimpleMessageFrameAPIDocumentation.lua:331: 			SecretArguments = "AllowedWhenUntainted",
SimpleMessageFrameAPIDocumentation.lua:332: 
SimpleMessageFrameAPIDocumentation.lua:333: 			Arguments =
SimpleMessageFrameAPIDocumentation.lua:334: 			{
SimpleMessageFrameAPIDocumentation.lua:335: 				{ Name = "mode", Type = "InsertMode", Nilable = false },
SimpleMessageFrameAPIDocumentation.lua:336: 			},
SimpleMessageFrameAPIDocumentation.lua:337: 		},
SimpleMessageFrameAPIDocumentation.lua:338: 		{
SimpleMessageFrameAPIDocumentation.lua:339: 			Name = "SetJustifyH",
SimpleMessageFrameAPIDocumentation.lua:340: 			Type = "Function",
SimpleMessageFrameAPIDocumentation.lua:341: 			SecretArguments = "AllowedWhenUntainted",
SimpleMessageFrameAPIDocumentation.lua:342: 

SimpleFontAPIDocumentation.lua:30: 			},
SimpleFontAPIDocumentation.lua:31: 		},
SimpleFontAPIDocumentation.lua:32: 		{
SimpleFontAPIDocumentation.lua:33: 			Name = "GetFont",
SimpleFontAPIDocumentation.lua:34: 			Type = "Function",
SimpleFontAPIDocumentation.lua:35: 
SimpleFontAPIDocumentation.lua:36: 			Arguments =
SimpleFontAPIDocumentation.lua:37: 			{
SimpleFontAPIDocumentation.lua:38: 			},
SimpleFontAPIDocumentation.lua:39: 
SimpleFontAPIDocumentation.lua:40: 			Returns =
SimpleFontAPIDocumentation.lua:41: 			{
SimpleFontAPIDocumentation.lua:42: 				{ Name = "fontFile", Type = "cstring", Nilable = false },
SimpleFontAPIDocumentation.lua:43: 				{ Name = "height", Type = "uiFontHeight", Nilable = false },
SimpleFontAPIDocumentation.lua:44: 				{ Name = "flags", Type = "TBFFlags", Nilable = false },
SimpleFontAPIDocumentation.lua:45: 			},
SimpleFontAPIDocumentation.lua:46: 		},
SimpleFontAPIDocumentation.lua:47: 		{
SimpleFontAPIDocumentation.lua:48: 			Name = "GetFontHeight",
SimpleFontAPIDocumentation.lua:49: 			Type = "Function",
SimpleFontAPIDocumentation.lua:50: 			Documentation = { "Return is either in uiUnits or internal height due to fixedHeight." },
SimpleFontAPIDocumentation.lua:51: 
SimpleFontAPIDocumentation.lua:52: 			Arguments =
SimpleFontAPIDocumentation.lua:53: 			{
SimpleFontAPIDocumentation.lua:54: 			},
SimpleFontAPIDocumentation.lua:55: 
SimpleFontAPIDocumentation.lua:56: 			Returns =
SimpleFontAPIDocumentation.lua:57: 			{
SimpleFontAPIDocumentation.lua:58: 				{ Name = "height", Type = "number", Nilable = false },
SimpleFontAPIDocumentation.lua:59: 			},
SimpleFontAPIDocumentation.lua:60: 		},
SimpleFontAPIDocumentation.lua:61: 		{
SimpleFontAPIDocumentation.lua:62: 			Name = "GetFontObject",
SimpleFontAPIDocumentation.lua:63: 			Type = "Function",
SimpleFontAPIDocumentation.lua:64: 
SimpleFontAPIDocumentation.lua:65: 			Arguments =
SimpleFontAPIDocumentation.lua:66: 			{
SimpleFontAPIDocumentation.lua:67: 			},
SimpleFontAPIDocumentation.lua:68: 
SimpleFontAPIDocumentation.lua:69: 			Returns =
SimpleFontAPIDocumentation.lua:70: 			{
SimpleFontAPIDocumentation.lua:71: 				{ Name = "font", Type = "SimpleFont", Nilable = false },
SimpleFontAPIDocumentation.lua:72: 			},
SimpleFontAPIDocumentation.lua:73: 		},
SimpleFontAPIDocumentation.lua:74: 		{
SimpleFontAPIDocumentation.lua:75: 			Name = "GetFontObjectForAlphabet",
SimpleFontAPIDocumentation.lua:76: 			Type = "Function",
SimpleFontAPIDocumentation.lua:77: 			ConstSecretAccessor = true,
SimpleFontAPIDocumentation.lua:78: 			SecretArguments = "AllowedWhenUntainted",
SimpleFontAPIDocumentation.lua:79: 
SimpleFontAPIDocumentation.lua:80: 			Arguments =

SimpleFontAPIDocumentation.lua:196: 			},
SimpleFontAPIDocumentation.lua:197: 		},
SimpleFontAPIDocumentation.lua:198: 		{
SimpleFontAPIDocumentation.lua:199: 			Name = "SetFont",
SimpleFontAPIDocumentation.lua:200: 			Type = "Function",
SimpleFontAPIDocumentation.lua:201: 			RequiresValidFontAsset = true,
SimpleFontAPIDocumentation.lua:202: 			RequiresValidFontHeight = true,
SimpleFontAPIDocumentation.lua:203: 			SecretArguments = "AllowedWhenUntainted",
SimpleFontAPIDocumentation.lua:204: 
SimpleFontAPIDocumentation.lua:205: 			Arguments =
SimpleFontAPIDocumentation.lua:206: 			{
SimpleFontAPIDocumentation.lua:207: 				{ Name = "fontFile", Type = "cstring", Nilable = false },
SimpleFontAPIDocumentation.lua:208: 				{ Name = "height", Type = "uiFontHeight", Nilable = false },
SimpleFontAPIDocumentation.lua:209: 				{ Name = "flags", Type = "TBFFlags", Nilable = false },
SimpleFontAPIDocumentation.lua:210: 			},
SimpleFontAPIDocumentation.lua:211: 		},
SimpleFontAPIDocumentation.lua:212: 		{
SimpleFontAPIDocumentation.lua:213: 			Name = "SetFontHeight",
SimpleFontAPIDocumentation.lua:214: 			Type = "Function",
SimpleFontAPIDocumentation.lua:215: 			SecretArguments = "AllowedWhenUntainted",
SimpleFontAPIDocumentation.lua:216: 			Documentation = { "Preserves all flags, does correct height conversion due to fixedHeight." },
SimpleFontAPIDocumentation.lua:217: 
SimpleFontAPIDocumentation.lua:218: 			Arguments =
SimpleFontAPIDocumentation.lua:219: 			{
SimpleFontAPIDocumentation.lua:220: 				{ Name = "height", Type = "number", Nilable = false },
SimpleFontAPIDocumentation.lua:221: 			},
SimpleFontAPIDocumentation.lua:222: 		},
SimpleFontAPIDocumentation.lua:223: 		{
SimpleFontAPIDocumentation.lua:224: 			Name = "SetFontObject",
SimpleFontAPIDocumentation.lua:225: 			Type = "Function",
SimpleFontAPIDocumentation.lua:226: 			SecretArguments = "AllowedWhenUntainted",
SimpleFontAPIDocumentation.lua:227: 
SimpleFontAPIDocumentation.lua:228: 			Arguments =
SimpleFontAPIDocumentation.lua:229: 			{
SimpleFontAPIDocumentation.lua:230: 				{ Name = "font", Type = "SimpleFont", Nilable = false },
SimpleFontAPIDocumentation.lua:231: 			},
SimpleFontAPIDocumentation.lua:232: 		},
SimpleFontAPIDocumentation.lua:233: 		{
SimpleFontAPIDocumentation.lua:234: 			Name = "SetIndentedWordWrap",
SimpleFontAPIDocumentation.lua:235: 			Type = "Function",
SimpleFontAPIDocumentation.lua:236: 			SecretArguments = "AllowedWhenUntainted",
SimpleFontAPIDocumentation.lua:237: 
SimpleFontAPIDocumentation.lua:238: 			Arguments =
SimpleFontAPIDocumentation.lua:239: 			{
SimpleFontAPIDocumentation.lua:240: 				{ Name = "wordWrap", Type = "bool", Nilable = false },
SimpleFontAPIDocumentation.lua:241: 			},
SimpleFontAPIDocumentation.lua:242: 		},
SimpleFontAPIDocumentation.lua:243: 		{
SimpleFontAPIDocumentation.lua:244: 			Name = "SetJustifyH",
SimpleFontAPIDocumentation.lua:245: 			Type = "Function",
SimpleFontAPIDocumentation.lua:246: 			SecretArguments = "AllowedWhenUntainted",

HouseExteriorUIDocumentation.lua:157: 			},
HouseExteriorUIDocumentation.lua:158: 		},
HouseExteriorUIDocumentation.lua:159: 		{
HouseExteriorUIDocumentation.lua:160: 			Name = "SelectCoreFixtureOption",
HouseExteriorUIDocumentation.lua:161: 			Type = "Function",
HouseExteriorUIDocumentation.lua:162: 			SecretArguments = "AllowedWhenUntainted",
HouseExteriorUIDocumentation.lua:163: 
HouseExteriorUIDocumentation.lua:164: 			Arguments =
HouseExteriorUIDocumentation.lua:165: 			{
HouseExteriorUIDocumentation.lua:166: 				{ Name = "fixtureID", Type = "number", Nilable = false },
HouseExteriorUIDocumentation.lua:167: 				{ Name = "attachedDecorAction", Type = "HousingFixtureDecorAction", Nilable = false, Default = "Store", Documentation = { "What to do with any decor attached to the old core fixture; If the fixture being swapped is a variant (ie recolor) of the existing one, attached decor will always be reparented directly to the new one" } },
HouseExteriorUIDocumentation.lua:168: 			},
HouseExteriorUIDocumentation.lua:169: 		},
HouseExteriorUIDocumentation.lua:170: 		{
HouseExteriorUIDocumentation.lua:171: 			Name = "SelectFixtureOption",
HouseExteriorUIDocumentation.lua:172: 			Type = "Function",
HouseExteriorUIDocumentation.lua:173: 			SecretArguments = "AllowedWhenUntainted",
HouseExteriorUIDocumentation.lua:174: 
HouseExteriorUIDocumentation.lua:175: 			Arguments =
HouseExteriorUIDocumentation.lua:176: 			{
HouseExteriorUIDocumentation.lua:177: 				{ Name = "fixtureID", Type = "number", Nilable = false },
HouseExteriorUIDocumentation.lua:178: 				{ Name = "attachedDecorAction", Type = "HousingFixtureDecorAction", Nilable = false, Default = "Store", Documentation = { "If this fixture choice is replacing an existing one that has decor attached, what to do with any decor attached to the old one" } },
HouseExteriorUIDocumentation.lua:179: 			},
HouseExteriorUIDocumentation.lua:180: 		},
HouseExteriorUIDocumentation.lua:181: 		{
HouseExteriorUIDocumentation.lua:182: 			Name = "SetExteriorDecorHidden",
HouseExteriorUIDocumentation.lua:183: 			Type = "Function",
HouseExteriorUIDocumentation.lua:184: 			SecretArguments = "AllowedWhenUntainted",
HouseExteriorUIDocumentation.lua:185: 
HouseExteriorUIDocumentation.lua:186: 			Arguments =
HouseExteriorUIDocumentation.lua:187: 			{
HouseExteriorUIDocumentation.lua:188: 				{ Name = "decorHidden", Type = "bool", Nilable = false },
HouseExteriorUIDocumentation.lua:189: 			},
HouseExteriorUIDocumentation.lua:190: 		},
HouseExteriorUIDocumentation.lua:191: 		{
HouseExteriorUIDocumentation.lua:192: 			Name = "SetHouseExteriorSize",
HouseExteriorUIDocumentation.lua:193: 			Type = "Function",
HouseExteriorUIDocumentation.lua:194: 			SecretArguments = "AllowedWhenUntainted",
HouseExteriorUIDocumentation.lua:195: 
HouseExteriorUIDocumentation.lua:196: 			Arguments =
HouseExteriorUIDocumentation.lua:197: 			{
HouseExteriorUIDocumentation.lua:198: 				{ Name = "size", Type = "HousingFixtureSize", Nilable = false },
HouseExteriorUIDocumentation.lua:199: 				{ Name = "attachedDecorAction", Type = "HousingFixtureDecorAction", Nilable = false, Default = "Store", Documentation = { "What to do with decor attached to any of the house's existing exterior components" } },
HouseExteriorUIDocumentation.lua:200: 			},
HouseExteriorUIDocumentation.lua:201: 		},
HouseExteriorUIDocumentation.lua:202: 		{
HouseExteriorUIDocumentation.lua:203: 			Name = "SetHouseExteriorType",
HouseExteriorUIDocumentation.lua:204: 			Type = "Function",
HouseExteriorUIDocumentation.lua:205: 			SecretArguments = "AllowedWhenUntainted",
HouseExteriorUIDocumentation.lua:206: 
HouseExteriorUIDocumentation.lua:207: 			Arguments =

SimpleEditBoxAPIDocumentation.lua:113: 			},
SimpleEditBoxAPIDocumentation.lua:114: 		},
SimpleEditBoxAPIDocumentation.lua:115: 		{
SimpleEditBoxAPIDocumentation.lua:116: 			Name = "GetFont",
SimpleEditBoxAPIDocumentation.lua:117: 			Type = "Function",
SimpleEditBoxAPIDocumentation.lua:118: 
SimpleEditBoxAPIDocumentation.lua:119: 			Arguments =
SimpleEditBoxAPIDocumentation.lua:120: 			{
SimpleEditBoxAPIDocumentation.lua:121: 			},
SimpleEditBoxAPIDocumentation.lua:122: 
SimpleEditBoxAPIDocumentation.lua:123: 			Returns =
SimpleEditBoxAPIDocumentation.lua:124: 			{
SimpleEditBoxAPIDocumentation.lua:125: 				{ Name = "name", Type = "cstring", Nilable = false },
SimpleEditBoxAPIDocumentation.lua:126: 				{ Name = "fontHeight", Type = "uiUnit", Nilable = false },
SimpleEditBoxAPIDocumentation.lua:127: 				{ Name = "flags", Type = "TBFFlags", Nilable = false },
SimpleEditBoxAPIDocumentation.lua:128: 			},
SimpleEditBoxAPIDocumentation.lua:129: 		},
SimpleEditBoxAPIDocumentation.lua:130: 		{
SimpleEditBoxAPIDocumentation.lua:131: 			Name = "GetFontObject",
SimpleEditBoxAPIDocumentation.lua:132: 			Type = "Function",
SimpleEditBoxAPIDocumentation.lua:133: 
SimpleEditBoxAPIDocumentation.lua:134: 			Arguments =
SimpleEditBoxAPIDocumentation.lua:135: 			{
SimpleEditBoxAPIDocumentation.lua:136: 			},
SimpleEditBoxAPIDocumentation.lua:137: 
SimpleEditBoxAPIDocumentation.lua:138: 			Returns =
SimpleEditBoxAPIDocumentation.lua:139: 			{
SimpleEditBoxAPIDocumentation.lua:140: 				{ Name = "font", Type = "SimpleFont", Nilable = false },
SimpleEditBoxAPIDocumentation.lua:141: 			},
SimpleEditBoxAPIDocumentation.lua:142: 		},
SimpleEditBoxAPIDocumentation.lua:143: 		{
SimpleEditBoxAPIDocumentation.lua:144: 			Name = "GetHighlightColor",
SimpleEditBoxAPIDocumentation.lua:145: 			Type = "Function",
SimpleEditBoxAPIDocumentation.lua:146: 
SimpleEditBoxAPIDocumentation.lua:147: 			Arguments =
SimpleEditBoxAPIDocumentation.lua:148: 			{
SimpleEditBoxAPIDocumentation.lua:149: 			},
SimpleEditBoxAPIDocumentation.lua:150: 
SimpleEditBoxAPIDocumentation.lua:151: 			Returns =
SimpleEditBoxAPIDocumentation.lua:152: 			{
SimpleEditBoxAPIDocumentation.lua:153: 				{ Name = "colorR", Type = "number", Nilable = false },
SimpleEditBoxAPIDocumentation.lua:154: 				{ Name = "colorG", Type = "number", Nilable = false },
SimpleEditBoxAPIDocumentation.lua:155: 				{ Name = "colorB", Type = "number", Nilable = false },
SimpleEditBoxAPIDocumentation.lua:156: 				{ Name = "colorA", Type = "number", Nilable = false },
SimpleEditBoxAPIDocumentation.lua:157: 			},
SimpleEditBoxAPIDocumentation.lua:158: 		},
SimpleEditBoxAPIDocumentation.lua:159: 		{
SimpleEditBoxAPIDocumentation.lua:160: 			Name = "GetHistoryLines",
SimpleEditBoxAPIDocumentation.lua:161: 			Type = "Function",
SimpleEditBoxAPIDocumentation.lua:162: 
SimpleEditBoxAPIDocumentation.lua:163: 			Arguments =

SimpleEditBoxAPIDocumentation.lua:674: 			},
SimpleEditBoxAPIDocumentation.lua:675: 		},
SimpleEditBoxAPIDocumentation.lua:676: 		{
SimpleEditBoxAPIDocumentation.lua:677: 			Name = "SetFont",
SimpleEditBoxAPIDocumentation.lua:678: 			Type = "Function",
SimpleEditBoxAPIDocumentation.lua:679: 			RequiresValidFontAsset = true,
SimpleEditBoxAPIDocumentation.lua:680: 			RequiresValidFontHeight = true,
SimpleEditBoxAPIDocumentation.lua:681: 			SecretArguments = "AllowedWhenUntainted",
SimpleEditBoxAPIDocumentation.lua:682: 
SimpleEditBoxAPIDocumentation.lua:683: 			Arguments =
SimpleEditBoxAPIDocumentation.lua:684: 			{
SimpleEditBoxAPIDocumentation.lua:685: 				{ Name = "fontFile", Type = "cstring", Nilable = false },
SimpleEditBoxAPIDocumentation.lua:686: 				{ Name = "height", Type = "uiFontHeight", Nilable = false },
SimpleEditBoxAPIDocumentation.lua:687: 				{ Name = "flags", Type = "TBFFlags", Nilable = false },
SimpleEditBoxAPIDocumentation.lua:688: 			},
SimpleEditBoxAPIDocumentation.lua:689: 
SimpleEditBoxAPIDocumentation.lua:690: 			Returns =
SimpleEditBoxAPIDocumentation.lua:691: 			{
SimpleEditBoxAPIDocumentation.lua:692: 				{ Name = "success", Type = "bool", Nilable = false },
SimpleEditBoxAPIDocumentation.lua:693: 			},
SimpleEditBoxAPIDocumentation.lua:694: 		},
SimpleEditBoxAPIDocumentation.lua:695: 		{
SimpleEditBoxAPIDocumentation.lua:696: 			Name = "SetFontObject",
SimpleEditBoxAPIDocumentation.lua:697: 			Type = "Function",
SimpleEditBoxAPIDocumentation.lua:698: 			SecretArguments = "AllowedWhenUntainted",
SimpleEditBoxAPIDocumentation.lua:699: 
SimpleEditBoxAPIDocumentation.lua:700: 			Arguments =
SimpleEditBoxAPIDocumentation.lua:701: 			{
SimpleEditBoxAPIDocumentation.lua:702: 				{ Name = "font", Type = "SimpleFont", Nilable = false },
SimpleEditBoxAPIDocumentation.lua:703: 			},
SimpleEditBoxAPIDocumentation.lua:704: 		},
SimpleEditBoxAPIDocumentation.lua:705: 		{
SimpleEditBoxAPIDocumentation.lua:706: 			Name = "SetHighlightColor",
SimpleEditBoxAPIDocumentation.lua:707: 			Type = "Function",
SimpleEditBoxAPIDocumentation.lua:708: 			SecretArguments = "AllowedWhenUntainted",
SimpleEditBoxAPIDocumentation.lua:709: 
SimpleEditBoxAPIDocumentation.lua:710: 			Arguments =
SimpleEditBoxAPIDocumentation.lua:711: 			{
SimpleEditBoxAPIDocumentation.lua:712: 				{ Name = "colorR", Type = "number", Nilable = false },
SimpleEditBoxAPIDocumentation.lua:713: 				{ Name = "colorG", Type = "number", Nilable = false },
SimpleEditBoxAPIDocumentation.lua:714: 				{ Name = "colorB", Type = "number", Nilable = false },
SimpleEditBoxAPIDocumentation.lua:715: 				{ Name = "a", Type = "SingleColorValue", Nilable = true },
SimpleEditBoxAPIDocumentation.lua:716: 			},
SimpleEditBoxAPIDocumentation.lua:717: 		},
SimpleEditBoxAPIDocumentation.lua:718: 		{
SimpleEditBoxAPIDocumentation.lua:719: 			Name = "SetHistoryLines",
SimpleEditBoxAPIDocumentation.lua:720: 			Type = "Function",
SimpleEditBoxAPIDocumentation.lua:721: 			SecretArguments = "AllowedWhenUntainted",
SimpleEditBoxAPIDocumentation.lua:722: 
SimpleEditBoxAPIDocumentation.lua:723: 			Arguments =
SimpleEditBoxAPIDocumentation.lua:724: 			{

SimpleHTMLAPIDocumentation.lua:20: 			},
SimpleHTMLAPIDocumentation.lua:21: 		},
SimpleHTMLAPIDocumentation.lua:22: 		{
SimpleHTMLAPIDocumentation.lua:23: 			Name = "GetFont",
SimpleHTMLAPIDocumentation.lua:24: 			Type = "Function",
SimpleHTMLAPIDocumentation.lua:25: 			ConstSecretAccessor = true,
SimpleHTMLAPIDocumentation.lua:26: 			SecretArguments = "AllowedWhenUntainted",
SimpleHTMLAPIDocumentation.lua:27: 
SimpleHTMLAPIDocumentation.lua:28: 			Arguments =
SimpleHTMLAPIDocumentation.lua:29: 			{
SimpleHTMLAPIDocumentation.lua:30: 				{ Name = "textType", Type = "HTMLTextType", Nilable = false },
SimpleHTMLAPIDocumentation.lua:31: 			},
SimpleHTMLAPIDocumentation.lua:32: 
SimpleHTMLAPIDocumentation.lua:33: 			Returns =
SimpleHTMLAPIDocumentation.lua:34: 			{
SimpleHTMLAPIDocumentation.lua:35: 				{ Name = "fontFile", Type = "cstring", Nilable = false },
SimpleHTMLAPIDocumentation.lua:36: 				{ Name = "height", Type = "uiFontHeight", Nilable = false },
SimpleHTMLAPIDocumentation.lua:37: 				{ Name = "flags", Type = "TBFFlags", Nilable = false },
SimpleHTMLAPIDocumentation.lua:38: 			},
SimpleHTMLAPIDocumentation.lua:39: 		},
SimpleHTMLAPIDocumentation.lua:40: 		{
SimpleHTMLAPIDocumentation.lua:41: 			Name = "GetFontObject",
SimpleHTMLAPIDocumentation.lua:42: 			Type = "Function",
SimpleHTMLAPIDocumentation.lua:43: 			ConstSecretAccessor = true,
SimpleHTMLAPIDocumentation.lua:44: 			SecretArguments = "AllowedWhenUntainted",
SimpleHTMLAPIDocumentation.lua:45: 
SimpleHTMLAPIDocumentation.lua:46: 			Arguments =
SimpleHTMLAPIDocumentation.lua:47: 			{
SimpleHTMLAPIDocumentation.lua:48: 				{ Name = "textType", Type = "HTMLTextType", Nilable = false },
SimpleHTMLAPIDocumentation.lua:49: 			},
SimpleHTMLAPIDocumentation.lua:50: 
SimpleHTMLAPIDocumentation.lua:51: 			Returns =
SimpleHTMLAPIDocumentation.lua:52: 			{
SimpleHTMLAPIDocumentation.lua:53: 				{ Name = "font", Type = "SimpleFont", Nilable = false },
SimpleHTMLAPIDocumentation.lua:54: 			},
SimpleHTMLAPIDocumentation.lua:55: 		},
SimpleHTMLAPIDocumentation.lua:56: 		{
SimpleHTMLAPIDocumentation.lua:57: 			Name = "GetHyperlinkFormat",
SimpleHTMLAPIDocumentation.lua:58: 			Type = "Function",
SimpleHTMLAPIDocumentation.lua:59: 
SimpleHTMLAPIDocumentation.lua:60: 			Arguments =
SimpleHTMLAPIDocumentation.lua:61: 			{
SimpleHTMLAPIDocumentation.lua:62: 			},
SimpleHTMLAPIDocumentation.lua:63: 
SimpleHTMLAPIDocumentation.lua:64: 			Returns =
SimpleHTMLAPIDocumentation.lua:65: 			{
SimpleHTMLAPIDocumentation.lua:66: 				{ Name = "format", Type = "cstring", Nilable = false },
SimpleHTMLAPIDocumentation.lua:67: 			},
SimpleHTMLAPIDocumentation.lua:68: 		},
SimpleHTMLAPIDocumentation.lua:69: 		{
SimpleHTMLAPIDocumentation.lua:70: 			Name = "GetIndentedWordWrap",

SimpleHTMLAPIDocumentation.lua:200: 			},
SimpleHTMLAPIDocumentation.lua:201: 		},
SimpleHTMLAPIDocumentation.lua:202: 		{
SimpleHTMLAPIDocumentation.lua:203: 			Name = "SetFont",
SimpleHTMLAPIDocumentation.lua:204: 			Type = "Function",
SimpleHTMLAPIDocumentation.lua:205: 			RequiresValidFontAsset = true,
SimpleHTMLAPIDocumentation.lua:206: 			RequiresValidFontHeight = true,
SimpleHTMLAPIDocumentation.lua:207: 			SecretArguments = "AllowedWhenUntainted",
SimpleHTMLAPIDocumentation.lua:208: 
SimpleHTMLAPIDocumentation.lua:209: 			Arguments =
SimpleHTMLAPIDocumentation.lua:210: 			{
SimpleHTMLAPIDocumentation.lua:211: 				{ Name = "textType", Type = "HTMLTextType", Nilable = false },
SimpleHTMLAPIDocumentation.lua:212: 				{ Name = "fontFile", Type = "cstring", Nilable = false },
SimpleHTMLAPIDocumentation.lua:213: 				{ Name = "height", Type = "uiFontHeight", Nilable = false },
SimpleHTMLAPIDocumentation.lua:214: 				{ Name = "flags", Type = "TBFFlags", Nilable = false },
SimpleHTMLAPIDocumentation.lua:215: 			},
SimpleHTMLAPIDocumentation.lua:216: 		},
SimpleHTMLAPIDocumentation.lua:217: 		{
SimpleHTMLAPIDocumentation.lua:218: 			Name = "SetFontObject",
SimpleHTMLAPIDocumentation.lua:219: 			Type = "Function",
SimpleHTMLAPIDocumentation.lua:220: 			SecretArguments = "AllowedWhenUntainted",
SimpleHTMLAPIDocumentation.lua:221: 
SimpleHTMLAPIDocumentation.lua:222: 			Arguments =
SimpleHTMLAPIDocumentation.lua:223: 			{
SimpleHTMLAPIDocumentation.lua:224: 				{ Name = "textType", Type = "HTMLTextType", Nilable = false },
SimpleHTMLAPIDocumentation.lua:225: 				{ Name = "font", Type = "SimpleFont", Nilable = false },
SimpleHTMLAPIDocumentation.lua:226: 			},
SimpleHTMLAPIDocumentation.lua:227: 		},
SimpleHTMLAPIDocumentation.lua:228: 		{
SimpleHTMLAPIDocumentation.lua:229: 			Name = "SetHyperlinkFormat",
SimpleHTMLAPIDocumentation.lua:230: 			Type = "Function",
SimpleHTMLAPIDocumentation.lua:231: 			SecretArguments = "AllowedWhenUntainted",
SimpleHTMLAPIDocumentation.lua:232: 
SimpleHTMLAPIDocumentation.lua:233: 			Arguments =
SimpleHTMLAPIDocumentation.lua:234: 			{
SimpleHTMLAPIDocumentation.lua:235: 				{ Name = "format", Type = "cstring", Nilable = false },
SimpleHTMLAPIDocumentation.lua:236: 			},
SimpleHTMLAPIDocumentation.lua:237: 		},
SimpleHTMLAPIDocumentation.lua:238: 		{
SimpleHTMLAPIDocumentation.lua:239: 			Name = "SetIndentedWordWrap",
SimpleHTMLAPIDocumentation.lua:240: 			Type = "Function",
SimpleHTMLAPIDocumentation.lua:241: 			SecretArguments = "AllowedWhenUntainted",
SimpleHTMLAPIDocumentation.lua:242: 
SimpleHTMLAPIDocumentation.lua:243: 			Arguments =
SimpleHTMLAPIDocumentation.lua:244: 			{
SimpleHTMLAPIDocumentation.lua:245: 				{ Name = "textType", Type = "HTMLTextType", Nilable = false },
SimpleHTMLAPIDocumentation.lua:246: 				{ Name = "wordWrap", Type = "bool", Nilable = false },
SimpleHTMLAPIDocumentation.lua:247: 			},
SimpleHTMLAPIDocumentation.lua:248: 		},
SimpleHTMLAPIDocumentation.lua:249: 		{
SimpleHTMLAPIDocumentation.lua:250: 			Name = "SetJustifyH",

FrameAPICharacterModelBaseDocumentation.lua:237: 			},
FrameAPICharacterModelBaseDocumentation.lua:238: 		},
FrameAPICharacterModelBaseDocumentation.lua:239: 		{
FrameAPICharacterModelBaseDocumentation.lua:240: 			Name = "SetUnit",
FrameAPICharacterModelBaseDocumentation.lua:241: 			Type = "Function",
FrameAPICharacterModelBaseDocumentation.lua:242: 			RequiresDeclassifiedUnitIdentity = true,
FrameAPICharacterModelBaseDocumentation.lua:243: 			SecretArguments = "AllowedWhenUntainted",
FrameAPICharacterModelBaseDocumentation.lua:244: 
FrameAPICharacterModelBaseDocumentation.lua:245: 			Arguments =
FrameAPICharacterModelBaseDocumentation.lua:246: 			{
FrameAPICharacterModelBaseDocumentation.lua:247: 				{ Name = "unit", Type = "UnitToken", Nilable = false },
FrameAPICharacterModelBaseDocumentation.lua:248: 				{ Name = "blend", Type = "bool", Nilable = false, Default = true },
FrameAPICharacterModelBaseDocumentation.lua:249: 				{ Name = "useNativeForm", Type = "bool", Nilable = true },
FrameAPICharacterModelBaseDocumentation.lua:250: 			},
FrameAPICharacterModelBaseDocumentation.lua:251: 
FrameAPICharacterModelBaseDocumentation.lua:252: 			Returns =
FrameAPICharacterModelBaseDocumentation.lua:253: 			{
FrameAPICharacterModelBaseDocumentation.lua:254: 				{ Name = "success", Type = "bool", Nilable = false },
FrameAPICharacterModelBaseDocumentation.lua:255: 			},
FrameAPICharacterModelBaseDocumentation.lua:256: 		},
FrameAPICharacterModelBaseDocumentation.lua:257: 		{
FrameAPICharacterModelBaseDocumentation.lua:258: 			Name = "StopAnimKit",
FrameAPICharacterModelBaseDocumentation.lua:259: 			Type = "Function",
FrameAPICharacterModelBaseDocumentation.lua:260: 
FrameAPICharacterModelBaseDocumentation.lua:261: 			Arguments =
FrameAPICharacterModelBaseDocumentation.lua:262: 			{
FrameAPICharacterModelBaseDocumentation.lua:263: 			},
FrameAPICharacterModelBaseDocumentation.lua:264: 		},
FrameAPICharacterModelBaseDocumentation.lua:265: 		{
FrameAPICharacterModelBaseDocumentation.lua:266: 			Name = "ZeroCachedCenterXY",
FrameAPICharacterModelBaseDocumentation.lua:267: 			Type = "Function",
FrameAPICharacterModelBaseDocumentation.lua:268: 
FrameAPICharacterModelBaseDocumentation.lua:269: 			Arguments =
FrameAPICharacterModelBaseDocumentation.lua:270: 			{
FrameAPICharacterModelBaseDocumentation.lua:271: 			},
FrameAPICharacterModelBaseDocumentation.lua:272: 		},
FrameAPICharacterModelBaseDocumentation.lua:273: 	},
FrameAPICharacterModelBaseDocumentation.lua:274: 
FrameAPICharacterModelBaseDocumentation.lua:275: 	Events =
FrameAPICharacterModelBaseDocumentation.lua:276: 	{
FrameAPICharacterModelBaseDocumentation.lua:277: 	},
FrameAPICharacterModelBaseDocumentation.lua:278: 
FrameAPICharacterModelBaseDocumentation.lua:279: 	Tables =
FrameAPICharacterModelBaseDocumentation.lua:280: 	{
FrameAPICharacterModelBaseDocumentation.lua:281: 	},
FrameAPICharacterModelBaseDocumentation.lua:282: 	Predicates =
FrameAPICharacterModelBaseDocumentation.lua:283: 	{
FrameAPICharacterModelBaseDocumentation.lua:284: 	},
FrameAPICharacterModelBaseDocumentation.lua:285: };
FrameAPICharacterModelBaseDocumentation.lua:286: 
FrameAPICharacterModelBaseDocumentation.lua:287: APIDocumentation:AddDocumentationTable(FrameAPICharacterModelBase);

FrameAPIModelSceneFrameActorBaseDocumentation.lua:444: 			},
FrameAPIModelSceneFrameActorBaseDocumentation.lua:445: 		},
FrameAPIModelSceneFrameActorBaseDocumentation.lua:446: 		{
FrameAPIModelSceneFrameActorBaseDocumentation.lua:447: 			Name = "SetModelByUnit",
FrameAPIModelSceneFrameActorBaseDocumentation.lua:448: 			Type = "Function",
FrameAPIModelSceneFrameActorBaseDocumentation.lua:449: 			RequiresDeclassifiedUnitIdentity = true,
FrameAPIModelSceneFrameActorBaseDocumentation.lua:450: 			SecretArguments = "AllowedWhenUntainted",
FrameAPIModelSceneFrameActorBaseDocumentation.lua:451: 
FrameAPIModelSceneFrameActorBaseDocumentation.lua:452: 			Arguments =
FrameAPIModelSceneFrameActorBaseDocumentation.lua:453: 			{
FrameAPIModelSceneFrameActorBaseDocumentation.lua:454: 				{ Name = "unit", Type = "UnitToken", Nilable = false },
FrameAPIModelSceneFrameActorBaseDocumentation.lua:455: 				{ Name = "sheatheWeapons", Type = "bool", Nilable = false, Default = false },
FrameAPIModelSceneFrameActorBaseDocumentation.lua:456: 				{ Name = "autoDress", Type = "bool", Nilable = false, Default = true },
FrameAPIModelSceneFrameActorBaseDocumentation.lua:457: 				{ Name = "hideWeapons", Type = "bool", Nilable = false, Default = false },
FrameAPIModelSceneFrameActorBaseDocumentation.lua:458: 				{ Name = "usePlayerNativeForm", Type = "bool", Nilable = false, Default = true },
FrameAPIModelSceneFrameActorBaseDocumentation.lua:459: 				{ Name = "holdBowString", Type = "bool", Nilable = false, Default = false },
FrameAPIModelSceneFrameActorBaseDocumentation.lua:460: 				{ Name = "customRaceID", Type = "number", Nilable = true },
FrameAPIModelSceneFrameActorBaseDocumentation.lua:461: 			},
FrameAPIModelSceneFrameActorBaseDocumentation.lua:462: 
FrameAPIModelSceneFrameActorBaseDocumentation.lua:463: 			Returns =
FrameAPIModelSceneFrameActorBaseDocumentation.lua:464: 			{
FrameAPIModelSceneFrameActorBaseDocumentation.lua:465: 				{ Name = "success", Type = "bool", Nilable = false },
FrameAPIModelSceneFrameActorBaseDocumentation.lua:466: 			},
FrameAPIModelSceneFrameActorBaseDocumentation.lua:467: 		},
FrameAPIModelSceneFrameActorBaseDocumentation.lua:468: 		{
FrameAPIModelSceneFrameActorBaseDocumentation.lua:469: 			Name = "SetParticleOverrideScale",
FrameAPIModelSceneFrameActorBaseDocumentation.lua:470: 			Type = "Function",
FrameAPIModelSceneFrameActorBaseDocumentation.lua:471: 			SecretArguments = "AllowedWhenUntainted",
FrameAPIModelSceneFrameActorBaseDocumentation.lua:472: 
FrameAPIModelSceneFrameActorBaseDocumentation.lua:473: 			Arguments =
FrameAPIModelSceneFrameActorBaseDocumentation.lua:474: 			{
FrameAPIModelSceneFrameActorBaseDocumentation.lua:475: 				{ Name = "scale", Type = "number", Nilable = true },
FrameAPIModelSceneFrameActorBaseDocumentation.lua:476: 			},
FrameAPIModelSceneFrameActorBaseDocumentation.lua:477: 		},
FrameAPIModelSceneFrameActorBaseDocumentation.lua:478: 		{
FrameAPIModelSceneFrameActorBaseDocumentation.lua:479: 			Name = "SetPitch",
FrameAPIModelSceneFrameActorBaseDocumentation.lua:480: 			Type = "Function",
FrameAPIModelSceneFrameActorBaseDocumentation.lua:481: 			SecretArguments = "AllowedWhenUntainted",
FrameAPIModelSceneFrameActorBaseDocumentation.lua:482: 
FrameAPIModelSceneFrameActorBaseDocumentation.lua:483: 			Arguments =
FrameAPIModelSceneFrameActorBaseDocumentation.lua:484: 			{
FrameAPIModelSceneFrameActorBaseDocumentation.lua:485: 				{ Name = "pitch", Type = "number", Nilable = false },
FrameAPIModelSceneFrameActorBaseDocumentation.lua:486: 			},
FrameAPIModelSceneFrameActorBaseDocumentation.lua:487: 		},
FrameAPIModelSceneFrameActorBaseDocumentation.lua:488: 		{
FrameAPIModelSceneFrameActorBaseDocumentation.lua:489: 			Name = "SetPlayerModelFromGlues",
FrameAPIModelSceneFrameActorBaseDocumentation.lua:490: 			Type = "Function",
FrameAPIModelSceneFrameActorBaseDocumentation.lua:491: 			SecretArguments = "AllowedWhenUntainted",
FrameAPIModelSceneFrameActorBaseDocumentation.lua:492: 
FrameAPIModelSceneFrameActorBaseDocumentation.lua:493: 			Arguments =
FrameAPIModelSceneFrameActorBaseDocumentation.lua:494: 			{

ActionBarFrameDocumentation.lua:823: 			},
ActionBarFrameDocumentation.lua:824: 		},
ActionBarFrameDocumentation.lua:825: 		{
ActionBarFrameDocumentation.lua:826: 			Name = "IsOnBarOrSpecialBar",
ActionBarFrameDocumentation.lua:827: 			Type = "Function",
ActionBarFrameDocumentation.lua:828: 			SecretArguments = "AllowedWhenTainted",
ActionBarFrameDocumentation.lua:829: 
ActionBarFrameDocumentation.lua:830: 			Arguments =
ActionBarFrameDocumentation.lua:831: 			{
ActionBarFrameDocumentation.lua:832: 				{ Name = "spellID", Type = "SpellIdentifier", Nilable = false },
ActionBarFrameDocumentation.lua:833: 			},
ActionBarFrameDocumentation.lua:834: 
ActionBarFrameDocumentation.lua:835: 			Returns =
ActionBarFrameDocumentation.lua:836: 			{
ActionBarFrameDocumentation.lua:837: 				{ Name = "isOnBarOrSpecialBar", Type = "bool", Nilable = false },
ActionBarFrameDocumentation.lua:838: 			},
ActionBarFrameDocumentation.lua:839: 		},
ActionBarFrameDocumentation.lua:840: 		{
ActionBarFrameDocumentation.lua:841: 			Name = "IsPossessBarVisible",
ActionBarFrameDocumentation.lua:842: 			Type = "Function",
ActionBarFrameDocumentation.lua:843: 
ActionBarFrameDocumentation.lua:844: 			Returns =
ActionBarFrameDocumentation.lua:845: 			{
ActionBarFrameDocumentation.lua:846: 				{ Name = "isPossessBarVisible", Type = "bool", Nilable = false },
ActionBarFrameDocumentation.lua:847: 			},
ActionBarFrameDocumentation.lua:848: 		},
ActionBarFrameDocumentation.lua:849: 		{
ActionBarFrameDocumentation.lua:850: 			Name = "IsStackableAction",
ActionBarFrameDocumentation.lua:851: 			Type = "Function",
ActionBarFrameDocumentation.lua:852: 			RequiresValidActionSlot = true,
ActionBarFrameDocumentation.lua:853: 			SecretArguments = "AllowedWhenUntainted",
ActionBarFrameDocumentation.lua:854: 
ActionBarFrameDocumentation.lua:855: 			Arguments =
ActionBarFrameDocumentation.lua:856: 			{
ActionBarFrameDocumentation.lua:857: 				{ Name = "actionID", Type = "luaIndex", Nilable = false },
ActionBarFrameDocumentation.lua:858: 			},
ActionBarFrameDocumentation.lua:859: 
ActionBarFrameDocumentation.lua:860: 			Returns =
ActionBarFrameDocumentation.lua:861: 			{
ActionBarFrameDocumentation.lua:862: 				{ Name = "isStackableAction", Type = "bool", Nilable = false },
ActionBarFrameDocumentation.lua:863: 			},
ActionBarFrameDocumentation.lua:864: 		},
ActionBarFrameDocumentation.lua:865: 		{
ActionBarFrameDocumentation.lua:866: 			Name = "IsUsableAction",
ActionBarFrameDocumentation.lua:867: 			Type = "Function",
ActionBarFrameDocumentation.lua:868: 			RequiresValidActionSlot = true,
ActionBarFrameDocumentation.lua:869: 			SecretArguments = "AllowedWhenUntainted",
ActionBarFrameDocumentation.lua:870: 
ActionBarFrameDocumentation.lua:871: 			Arguments =
ActionBarFrameDocumentation.lua:872: 			{
ActionBarFrameDocumentation.lua:873: 				{ Name = "actionID", Type = "luaIndex", Nilable = false },

HousingCatalogUIDocumentation.lua:28: 			},
HousingCatalogUIDocumentation.lua:29: 		},
HousingCatalogUIDocumentation.lua:30: 		{
HousingCatalogUIDocumentation.lua:31: 			Name = "DestroyEntry",
HousingCatalogUIDocumentation.lua:32: 			Type = "Function",
HousingCatalogUIDocumentation.lua:33: 			SecretArguments = "AllowedWhenUntainted",
HousingCatalogUIDocumentation.lua:34: 			Documentation = { "Attempt to delete the entry from storage" },
HousingCatalogUIDocumentation.lua:35: 
HousingCatalogUIDocumentation.lua:36: 			Arguments =
HousingCatalogUIDocumentation.lua:37: 			{
HousingCatalogUIDocumentation.lua:38: 				{ Name = "entryVariantID", Type = "HousingCatalogEntryVariantID", Nilable = false },
HousingCatalogUIDocumentation.lua:39: 				{ Name = "destroyAll", Type = "bool", Nilable = false, Documentation = { "If true, deletes all entries within the stack; If false, will only delete one" } },
HousingCatalogUIDocumentation.lua:40: 			},
HousingCatalogUIDocumentation.lua:41: 		},
HousingCatalogUIDocumentation.lua:42: 		{
HousingCatalogUIDocumentation.lua:43: 			Name = "GetAllFilterTagGroups",
HousingCatalogUIDocumentation.lua:44: 			Type = "Function",
HousingCatalogUIDocumentation.lua:45: 
HousingCatalogUIDocumentation.lua:46: 			Returns =
HousingCatalogUIDocumentation.lua:47: 			{
HousingCatalogUIDocumentation.lua:48: 				{ Name = "filterTagGroups", Type = "table", InnerType = "HousingCatalogFilterTagGroupInfo", Nilable = false },
HousingCatalogUIDocumentation.lua:49: 			},
HousingCatalogUIDocumentation.lua:50: 		},
HousingCatalogUIDocumentation.lua:51: 		{
HousingCatalogUIDocumentation.lua:52: 			Name = "GetAllVariantInfosForEntry",
HousingCatalogUIDocumentation.lua:53: 			Type = "Function",
HousingCatalogUIDocumentation.lua:54: 			SecretArguments = "AllowedWhenUntainted",
HousingCatalogUIDocumentation.lua:55: 			Documentation = { "Returns variant info for all variants of a given catalog entry; Variants represent different visual modifications of the same base entry (ex: dyed versions)" },
HousingCatalogUIDocumentation.lua:56: 
HousingCatalogUIDocumentation.lua:57: 			Arguments =
HousingCatalogUIDocumentation.lua:58: 			{
HousingCatalogUIDocumentation.lua:59: 				{ Name = "entryID", Type = "HousingCatalogEntryID", Nilable = false },
HousingCatalogUIDocumentation.lua:60: 			},
HousingCatalogUIDocumentation.lua:61: 
HousingCatalogUIDocumentation.lua:62: 			Returns =
HousingCatalogUIDocumentation.lua:63: 			{
HousingCatalogUIDocumentation.lua:64: 				{ Name = "variantInfos", Type = "table", InnerType = "HousingCatalogEntryVariantInfo", Nilable = false },
HousingCatalogUIDocumentation.lua:65: 			},
HousingCatalogUIDocumentation.lua:66: 		},
HousingCatalogUIDocumentation.lua:67: 		{
HousingCatalogUIDocumentation.lua:68: 			Name = "GetBundleInfo",
HousingCatalogUIDocumentation.lua:69: 			Type = "Function",
HousingCatalogUIDocumentation.lua:70: 			SecretArguments = "AllowedWhenUntainted",
HousingCatalogUIDocumentation.lua:71: 
HousingCatalogUIDocumentation.lua:72: 			Arguments =
HousingCatalogUIDocumentation.lua:73: 			{
HousingCatalogUIDocumentation.lua:74: 				{ Name = "bundleCatalogShopProductID", Type = "number", Nilable = false },
HousingCatalogUIDocumentation.lua:75: 			},
HousingCatalogUIDocumentation.lua:76: 
HousingCatalogUIDocumentation.lua:77: 			Returns =
HousingCatalogUIDocumentation.lua:78: 			{

HousingCatalogUIDocumentation.lua:534: 			},
HousingCatalogUIDocumentation.lua:535: 		},
HousingCatalogUIDocumentation.lua:536: 		{
HousingCatalogUIDocumentation.lua:537: 			Name = "HousingCatalogEntryInfo",
HousingCatalogUIDocumentation.lua:538: 			Type = "Structure",
HousingCatalogUIDocumentation.lua:539: 			Documentation = { "Base information about an object in the Catalog; For info for a specific owned stack of this object, see HousingCatalogEntryVariantInfo" },
HousingCatalogUIDocumentation.lua:540: 			Fields =
HousingCatalogUIDocumentation.lua:541: 			{
HousingCatalogUIDocumentation.lua:542: 				{ Name = "recordID", Type = "number", Nilable = false },
HousingCatalogUIDocumentation.lua:543: 				{ Name = "entryType", Type = "HousingCatalogEntryType", Nilable = false },
HousingCatalogUIDocumentation.lua:544: 				{ Name = "itemID", Type = "number", Nilable = true },
HousingCatalogUIDocumentation.lua:545: 				{ Name = "name", Type = "cstring", Nilable = false },
HousingCatalogUIDocumentation.lua:546: 				{ Name = "asset", Type = "ModelAsset", Nilable = true, Documentation = { "3D model asset for displaying in the UI; May be nil if the entry doesn't have a model, or has one that isn't supported by UI model scenes" } },
HousingCatalogUIDocumentation.lua:547: 				{ Name = "iconTexture", Type = "FileAsset", Nilable = true, Documentation = { "Entry icon in the form of a texture file; Catalog entries should have either this OR an iconAtlas set" } },
HousingCatalogUIDocumentation.lua:548: 				{ Name = "iconAtlas", Type = "textureAtlas", Nilable = true, Documentation = { "Entry icon in the form a texture atlas element; Catalog entries should have either this OR an iconTexture set" } },
HousingCatalogUIDocumentation.lua:549: 				{ Name = "uiModelSceneID", Type = "number", Nilable = true, Documentation = { "Specific UI model scene ID to use when previewing this entry's 3D model; If not set, the default catalog model scene is used" } },
HousingCatalogUIDocumentation.lua:550: 				{ Name = "categoryIDs", Type = "table", InnerType = "number", Nilable = false },
HousingCatalogUIDocumentation.lua:551: 				{ Name = "subcategoryIDs", Type = "table", InnerType = "number", Nilable = false },
HousingCatalogUIDocumentation.lua:552: 				{ Name = "dataTagsByID", Type = "LuaValueVariant", Nilable = false, Documentation = { "Simple localized 'tag' strings that are primarily used for things like categorization and filtering" } },
HousingCatalogUIDocumentation.lua:553: 				{ Name = "size", Type = "HousingCatalogEntrySize", Nilable = false },
HousingCatalogUIDocumentation.lua:554: 				{ Name = "placementCost", Type = "number", Nilable = false, Documentation = { "How much of the applicable budget placing this entry would cost (if any)" } },
HousingCatalogUIDocumentation.lua:555: 				{ Name = "totalNumStored", Type = "number", Nilable = false, Documentation = { "The total number of instances of this entry that exist in storage across all variants; Does not include unredeemed instances (see remainingRedeemable)" } },
HousingCatalogUIDocumentation.lua:556: 				{ Name = "remainingRedeemable", Type = "number", Nilable = false, Documentation = { "The number of unredeemed instances of this entry that exist in storage; Some auto-awarded housing objects are granted in this 'lazily-instantiated' way, and will be 'redeemed' on first being placed" } },
HousingCatalogUIDocumentation.lua:557: 				{ Name = "totalNumPlaced", Type = "number", Nilable = false, Documentation = { "The total number of instances of this entry that have been placed across all of the player's houses and plots, across all variants" } },
HousingCatalogUIDocumentation.lua:558: 				{ Name = "destroyableInstanceCount", Type = "number", Nilable = false, Documentation = { "The number of instances that can be destroyed for this entry." } },
HousingCatalogUIDocumentation.lua:559: 				{ Name = "isUniqueTrophy", Type = "bool", Nilable = false, Documentation = { "This decor is flagged to display as a unique trophy item." } },
HousingCatalogUIDocumentation.lua:560: 				{ Name = "isAllowedOutdoors", Type = "bool", Nilable = false, Documentation = { "True if this entry is something that is allowed to be placed outside, within a plot" } },
HousingCatalogUIDocumentation.lua:561: 				{ Name = "isAllowedIndoors", Type = "bool", Nilable = false, Documentation = { "True if this entry is something that is allowed to be placed indoors, within a house interior" } },
HousingCatalogUIDocumentation.lua:562: 				{ Name = "canCustomize", Type = "bool", Nilable = false, Documentation = { "True if this entry is something that can be customized; Kinds of customization vary depending on the entry type" } },
HousingCatalogUIDocumentation.lua:563: 				{ Name = "isPrefab", Type = "bool", Nilable = false },
HousingCatalogUIDocumentation.lua:564: 				{ Name = "quality", Type = "ItemQuality", Nilable = true },
HousingCatalogUIDocumentation.lua:565: 				{ Name = "firstAcquisitionBonus", Type = "number", Nilable = false, Documentation = { "House XP that can be gained upon acquiring this entry for the first time" } },
HousingCatalogUIDocumentation.lua:566: 				{ Name = "sourceText", Type = "cstring", Nilable = false, Documentation = { "Describes specific sources this entry may be gained from; Faction-specific sources may or may not be included based on the current player's faction" } },
HousingCatalogUIDocumentation.lua:567: 			},
HousingCatalogUIDocumentation.lua:568: 		},
HousingCatalogUIDocumentation.lua:569: 		{
HousingCatalogUIDocumentation.lua:570: 			Name = "HousingCatalogEntryVariantInfo",
HousingCatalogUIDocumentation.lua:571: 			Type = "Structure",
HousingCatalogUIDocumentation.lua:572: 			Documentation = { "Represents a single stack of instances of an object in the Catalog, that are all of a specific variation; For example, a stack of undyed chairs, or blue-dyed tables" },
HousingCatalogUIDocumentation.lua:573: 			Fields =
HousingCatalogUIDocumentation.lua:574: 			{
HousingCatalogUIDocumentation.lua:575: 				{ Name = "entryVariantID", Type = "HousingCatalogEntryVariantID", Nilable = false },
HousingCatalogUIDocumentation.lua:576: 				{ Name = "numStored", Type = "number", Nilable = false, Documentation = { "The number of instances of this specific variant that exist in storage" } },
HousingCatalogUIDocumentation.lua:577: 				{ Name = "dyeSlots", Type = "table", InnerType = "HousingDecorDyeSlot", Nilable = false, Documentation = { "Dye slot information for this variant; Empty for entries that can't be dyed" } },
HousingCatalogUIDocumentation.lua:578: 			},
HousingCatalogUIDocumentation.lua:579: 		},
HousingCatalogUIDocumentation.lua:580: 		{
HousingCatalogUIDocumentation.lua:581: 			Name = "HousingCatalogSubcategoryInfo",
HousingCatalogUIDocumentation.lua:582: 			Type = "Structure",
HousingCatalogUIDocumentation.lua:583: 			Fields =
HousingCatalogUIDocumentation.lua:584: 			{

SimpleFontStringAPIDocumentation.lua:118: 			},
SimpleFontStringAPIDocumentation.lua:119: 		},
SimpleFontStringAPIDocumentation.lua:120: 		{
SimpleFontStringAPIDocumentation.lua:121: 			Name = "GetFont",
SimpleFontStringAPIDocumentation.lua:122: 			Type = "Function",
SimpleFontStringAPIDocumentation.lua:123: 
SimpleFontStringAPIDocumentation.lua:124: 			Arguments =
SimpleFontStringAPIDocumentation.lua:125: 			{
SimpleFontStringAPIDocumentation.lua:126: 			},
SimpleFontStringAPIDocumentation.lua:127: 
SimpleFontStringAPIDocumentation.lua:128: 			Returns =
SimpleFontStringAPIDocumentation.lua:129: 			{
SimpleFontStringAPIDocumentation.lua:130: 				{ Name = "fontFile", Type = "FontAsset", Nilable = true },
SimpleFontStringAPIDocumentation.lua:131: 				{ Name = "fontHeight", Type = "uiUnit", Nilable = false },
SimpleFontStringAPIDocumentation.lua:132: 				{ Name = "flags", Type = "TBFFlags", Nilable = false },
SimpleFontStringAPIDocumentation.lua:133: 			},
SimpleFontStringAPIDocumentation.lua:134: 		},
SimpleFontStringAPIDocumentation.lua:135: 		{
SimpleFontStringAPIDocumentation.lua:136: 			Name = "GetFontHeight",
SimpleFontStringAPIDocumentation.lua:137: 			Type = "Function",
SimpleFontStringAPIDocumentation.lua:138: 			SecretArguments = "AllowedWhenUntainted",
SimpleFontStringAPIDocumentation.lua:139: 
SimpleFontStringAPIDocumentation.lua:140: 			Arguments =
SimpleFontStringAPIDocumentation.lua:141: 			{
SimpleFontStringAPIDocumentation.lua:142: 				{ Name = "calculated", Type = "bool", Nilable = false, Default = true },
SimpleFontStringAPIDocumentation.lua:143: 			},
SimpleFontStringAPIDocumentation.lua:144: 
SimpleFontStringAPIDocumentation.lua:145: 			Returns =
SimpleFontStringAPIDocumentation.lua:146: 			{
SimpleFontStringAPIDocumentation.lua:147: 				{ Name = "height", Type = "uiUnit", Nilable = false },
SimpleFontStringAPIDocumentation.lua:148: 			},
SimpleFontStringAPIDocumentation.lua:149: 		},
SimpleFontStringAPIDocumentation.lua:150: 		{
SimpleFontStringAPIDocumentation.lua:151: 			Name = "GetFontObject",
SimpleFontStringAPIDocumentation.lua:152: 			Type = "Function",
SimpleFontStringAPIDocumentation.lua:153: 
SimpleFontStringAPIDocumentation.lua:154: 			Arguments =
SimpleFontStringAPIDocumentation.lua:155: 			{
SimpleFontStringAPIDocumentation.lua:156: 			},
SimpleFontStringAPIDocumentation.lua:157: 
SimpleFontStringAPIDocumentation.lua:158: 			Returns =
SimpleFontStringAPIDocumentation.lua:159: 			{
SimpleFontStringAPIDocumentation.lua:160: 				{ Name = "font", Type = "SimpleFont", Nilable = false },
SimpleFontStringAPIDocumentation.lua:161: 			},
SimpleFontStringAPIDocumentation.lua:162: 		},
SimpleFontStringAPIDocumentation.lua:163: 		{
SimpleFontStringAPIDocumentation.lua:164: 			Name = "GetIndentedWordWrap",
SimpleFontStringAPIDocumentation.lua:165: 			Type = "Function",
SimpleFontStringAPIDocumentation.lua:166: 
SimpleFontStringAPIDocumentation.lua:167: 			Arguments =
SimpleFontStringAPIDocumentation.lua:168: 			{

SimpleFontStringAPIDocumentation.lua:497: 			},
SimpleFontStringAPIDocumentation.lua:498: 		},
SimpleFontStringAPIDocumentation.lua:499: 		{
SimpleFontStringAPIDocumentation.lua:500: 			Name = "SetFont",
SimpleFontStringAPIDocumentation.lua:501: 			Type = "Function",
SimpleFontStringAPIDocumentation.lua:502: 			RequiresValidFontAsset = true,
SimpleFontStringAPIDocumentation.lua:503: 			RequiresValidFontHeight = true,
SimpleFontStringAPIDocumentation.lua:504: 			SecretArguments = "AllowedWhenUntainted",
SimpleFontStringAPIDocumentation.lua:505: 
SimpleFontStringAPIDocumentation.lua:506: 			Arguments =
SimpleFontStringAPIDocumentation.lua:507: 			{
SimpleFontStringAPIDocumentation.lua:508: 				{ Name = "fontFile", Type = "FontAsset", Nilable = false },
SimpleFontStringAPIDocumentation.lua:509: 				{ Name = "fontHeight", Type = "uiFontHeight", Nilable = false },
SimpleFontStringAPIDocumentation.lua:510: 				{ Name = "flags", Type = "TBFFlags", Nilable = true },
SimpleFontStringAPIDocumentation.lua:511: 			},
SimpleFontStringAPIDocumentation.lua:512: 
SimpleFontStringAPIDocumentation.lua:513: 			Returns =
SimpleFontStringAPIDocumentation.lua:514: 			{
SimpleFontStringAPIDocumentation.lua:515: 				{ Name = "success", Type = "bool", Nilable = false },
SimpleFontStringAPIDocumentation.lua:516: 			},
SimpleFontStringAPIDocumentation.lua:517: 		},
SimpleFontStringAPIDocumentation.lua:518: 		{
SimpleFontStringAPIDocumentation.lua:519: 			Name = "SetFontHeight",
SimpleFontStringAPIDocumentation.lua:520: 			Type = "Function",
SimpleFontStringAPIDocumentation.lua:521: 			SecretArguments = "AllowedWhenUntainted",
SimpleFontStringAPIDocumentation.lua:522: 
SimpleFontStringAPIDocumentation.lua:523: 			Arguments =
SimpleFontStringAPIDocumentation.lua:524: 			{
SimpleFontStringAPIDocumentation.lua:525: 				{ Name = "height", Type = "uiUnit", Nilable = false },
SimpleFontStringAPIDocumentation.lua:526: 			},
SimpleFontStringAPIDocumentation.lua:527: 		},
SimpleFontStringAPIDocumentation.lua:528: 		{
SimpleFontStringAPIDocumentation.lua:529: 			Name = "SetFontObject",
SimpleFontStringAPIDocumentation.lua:530: 			Type = "Function",
SimpleFontStringAPIDocumentation.lua:531: 			SecretArguments = "AllowedWhenUntainted",
SimpleFontStringAPIDocumentation.lua:532: 
SimpleFontStringAPIDocumentation.lua:533: 			Arguments =
SimpleFontStringAPIDocumentation.lua:534: 			{
SimpleFontStringAPIDocumentation.lua:535: 				{ Name = "font", Type = "SimpleFont", Nilable = false },
SimpleFontStringAPIDocumentation.lua:536: 			},
SimpleFontStringAPIDocumentation.lua:537: 		},
SimpleFontStringAPIDocumentation.lua:538: 		{
SimpleFontStringAPIDocumentation.lua:539: 			Name = "SetFormattedText",
SimpleFontStringAPIDocumentation.lua:540: 			Type = "Function",
SimpleFontStringAPIDocumentation.lua:541: 			SecretArgumentsAddAspect = { Enum.SecretAspect.Text },
SimpleFontStringAPIDocumentation.lua:542: 			SecretArguments = "AllowedWhenTainted",
SimpleFontStringAPIDocumentation.lua:543: 
SimpleFontStringAPIDocumentation.lua:544: 			Arguments =
SimpleFontStringAPIDocumentation.lua:545: 			{
SimpleFontStringAPIDocumentation.lua:546: 				{ Name = "text", Type = "cstring", Nilable = false },
SimpleFontStringAPIDocumentation.lua:547: 			},
```

## Restricted-environment cache references

```
Blizzard_RestrictedAddOnEnvironment/RestrictedEnvironment.lua:216: function ENV.GetTransmogOutfitIndex(outfitID)
```

## Prose-related cached declarations and version drift

```lua
Blizzard_RestrictedAddOnEnvironment/RestrictedEnvironment.lua:216: function ENV.GetTransmogOutfitIndex(outfitID)
Blizzard_RestrictedAddOnEnvironment/RestrictedEnvironment.lua:217: 	local outfitInfo = C_TransmogOutfitInfo.GetOutfitInfo(outfitID);
Blizzard_RestrictedAddOnEnvironment/RestrictedEnvironment.lua:218: 
Blizzard_RestrictedAddOnEnvironment/RestrictedEnvironment.lua:219: 	if outfitInfo then
Blizzard_RestrictedAddOnEnvironment/RestrictedEnvironment.lua:220: 		return outfitInfo.playerFacingOutfitIndex;
Blizzard_RestrictedAddOnEnvironment/RestrictedEnvironment.lua:221: 	else
Blizzard_RestrictedAddOnEnvironment/RestrictedEnvironment.lua:222: 		return nil;
Blizzard_RestrictedAddOnEnvironment/RestrictedEnvironment.lua:223: 	end

Blizzard_APIDocumentationGenerated/PartyInfoDocumentation.lua:63: 			Name = "ConfirmConvertToRaid",
Blizzard_APIDocumentationGenerated/PartyInfoDocumentation.lua:64: 			Type = "Function",
Blizzard_APIDocumentationGenerated/PartyInfoDocumentation.lua:65: 			HasRestrictions = true,
Blizzard_APIDocumentationGenerated/PartyInfoDocumentation.lua:66: 			Documentation = { "Immediately convert to raid with no regard for potentially destructive actions." },
Blizzard_APIDocumentationGenerated/PartyInfoDocumentation.lua:67: 		},
Blizzard_APIDocumentationGenerated/PartyInfoDocumentation.lua:68: 		{
Blizzard_APIDocumentationGenerated/PartyInfoDocumentation.lua:69: 			Name = "ConfirmInviteTravelPass",
Blizzard_APIDocumentationGenerated/PartyInfoDocumentation.lua:70: 			Type = "Function",
Blizzard_APIDocumentationGenerated/PartyInfoDocumentation.lua:71: 			SecretArguments = "AllowedWhenUntainted",
Blizzard_APIDocumentationGenerated/PartyInfoDocumentation.lua:72: 
Blizzard_APIDocumentationGenerated/PartyInfoDocumentation.lua:73: 			Arguments =
Blizzard_APIDocumentationGenerated/PartyInfoDocumentation.lua:74: 			{
Blizzard_APIDocumentationGenerated/PartyInfoDocumentation.lua:75: 				{ Name = "targetName", Type = "cstring", Nilable = false },
Blizzard_APIDocumentationGenerated/PartyInfoDocumentation.lua:76: 				{ Name = "targetGUID", Type = "WOWGUID", Nilable = false },
Blizzard_APIDocumentationGenerated/PartyInfoDocumentation.lua:77: 			},
Blizzard_APIDocumentationGenerated/PartyInfoDocumentation.lua:78: 		},
Blizzard_APIDocumentationGenerated/PartyInfoDocumentation.lua:79: 		{
Blizzard_APIDocumentationGenerated/PartyInfoDocumentation.lua:80: 			Name = "ConfirmInviteUnit",

Blizzard_APIDocumentationGenerated/PartyInfoDocumentation.lua:507: 			Name = "PromoteToLeader",
Blizzard_APIDocumentationGenerated/PartyInfoDocumentation.lua:508: 			Type = "Function",
Blizzard_APIDocumentationGenerated/PartyInfoDocumentation.lua:509: 			HasRestrictions = true,
Blizzard_APIDocumentationGenerated/PartyInfoDocumentation.lua:510: 			SecretArguments = "AllowedWhenUntainted",
Blizzard_APIDocumentationGenerated/PartyInfoDocumentation.lua:511: 
Blizzard_APIDocumentationGenerated/PartyInfoDocumentation.lua:512: 			Arguments =
Blizzard_APIDocumentationGenerated/PartyInfoDocumentation.lua:513: 			{
Blizzard_APIDocumentationGenerated/PartyInfoDocumentation.lua:514: 				{ Name = "name", Type = "cstring", Nilable = false },
Blizzard_APIDocumentationGenerated/PartyInfoDocumentation.lua:515: 				{ Name = "exactNameMatch", Type = "bool", Nilable = true },
Blizzard_APIDocumentationGenerated/PartyInfoDocumentation.lua:516: 			},
Blizzard_APIDocumentationGenerated/PartyInfoDocumentation.lua:517: 		},
Blizzard_APIDocumentationGenerated/PartyInfoDocumentation.lua:518: 		{
Blizzard_APIDocumentationGenerated/PartyInfoDocumentation.lua:519: 			Name = "RequestInviteFromUnit",
Blizzard_APIDocumentationGenerated/PartyInfoDocumentation.lua:520: 			Type = "Function",
Blizzard_APIDocumentationGenerated/PartyInfoDocumentation.lua:521: 			RequiresValidInviteTarget = true,
Blizzard_APIDocumentationGenerated/PartyInfoDocumentation.lua:522: 			SecretArguments = "AllowedWhenUntainted",
Blizzard_APIDocumentationGenerated/PartyInfoDocumentation.lua:523: 			Documentation = { "Attempt to request an invite into the target party, requires confirmation in some cases (e.g. there is a party sync in progress)." },
Blizzard_APIDocumentationGenerated/PartyInfoDocumentation.lua:524: 
Blizzard_APIDocumentationGenerated/PartyInfoDocumentation.lua:525: 			Arguments =

Blizzard_APIDocumentationGenerated/RaidMarkersDocumentation.lua:95: 			Name = "SetRaidTarget",
Blizzard_APIDocumentationGenerated/RaidMarkersDocumentation.lua:96: 			Type = "Function",
Blizzard_APIDocumentationGenerated/RaidMarkersDocumentation.lua:97: 			HasRestrictions = true,
Blizzard_APIDocumentationGenerated/RaidMarkersDocumentation.lua:98: 			SecretArguments = "AllowedWhenUntainted",
Blizzard_APIDocumentationGenerated/RaidMarkersDocumentation.lua:99: 
Blizzard_APIDocumentationGenerated/RaidMarkersDocumentation.lua:100: 			Arguments =
Blizzard_APIDocumentationGenerated/RaidMarkersDocumentation.lua:101: 			{
Blizzard_APIDocumentationGenerated/RaidMarkersDocumentation.lua:102: 				{ Name = "target", Type = "UnitToken", Nilable = false },
Blizzard_APIDocumentationGenerated/RaidMarkersDocumentation.lua:103: 				{ Name = "userIndex", Type = "luaIndex", Nilable = false },
Blizzard_APIDocumentationGenerated/RaidMarkersDocumentation.lua:104: 			},
Blizzard_APIDocumentationGenerated/RaidMarkersDocumentation.lua:105: 		},
Blizzard_APIDocumentationGenerated/RaidMarkersDocumentation.lua:106: 	},
Blizzard_APIDocumentationGenerated/RaidMarkersDocumentation.lua:107: 
Blizzard_APIDocumentationGenerated/RaidMarkersDocumentation.lua:108: 	Events =
Blizzard_APIDocumentationGenerated/RaidMarkersDocumentation.lua:109: 	{
Blizzard_APIDocumentationGenerated/RaidMarkersDocumentation.lua:110: 	},
Blizzard_APIDocumentationGenerated/RaidMarkersDocumentation.lua:111: 
Blizzard_APIDocumentationGenerated/RaidMarkersDocumentation.lua:112: 	Tables =
Blizzard_APIDocumentationGenerated/RaidMarkersDocumentation.lua:113: 	{
Blizzard_APIDocumentationGenerated/RaidMarkersDocumentation.lua:114: 	},
Blizzard_APIDocumentationGenerated/RaidMarkersDocumentation.lua:115: 	Predicates =

Blizzard_APIDocumentationGenerated/UnitAuraDocumentation.lua:69: 			Name = "AuraIsPrivate",
Blizzard_APIDocumentationGenerated/UnitAuraDocumentation.lua:70: 			Type = "Function",
Blizzard_APIDocumentationGenerated/UnitAuraDocumentation.lua:71: 			SecretArguments = "AllowedWhenTainted",
Blizzard_APIDocumentationGenerated/UnitAuraDocumentation.lua:72: 
Blizzard_APIDocumentationGenerated/UnitAuraDocumentation.lua:73: 			Arguments =
Blizzard_APIDocumentationGenerated/UnitAuraDocumentation.lua:74: 			{
Blizzard_APIDocumentationGenerated/UnitAuraDocumentation.lua:75: 				{ Name = "spellID", Type = "SpellIdentifier", Nilable = false },
Blizzard_APIDocumentationGenerated/UnitAuraDocumentation.lua:76: 			},
Blizzard_APIDocumentationGenerated/UnitAuraDocumentation.lua:77: 
Blizzard_APIDocumentationGenerated/UnitAuraDocumentation.lua:78: 			Returns =
Blizzard_APIDocumentationGenerated/UnitAuraDocumentation.lua:79: 			{
Blizzard_APIDocumentationGenerated/UnitAuraDocumentation.lua:80: 				{ Name = "isPrivate", Type = "bool", Nilable = false },
Blizzard_APIDocumentationGenerated/UnitAuraDocumentation.lua:81: 			},
Blizzard_APIDocumentationGenerated/UnitAuraDocumentation.lua:82: 		},
Blizzard_APIDocumentationGenerated/UnitAuraDocumentation.lua:83: 		{

Blizzard_Deprecated/Shared/Deprecated_12_1_0.lua:15: 	if forceinsecure then
Blizzard_Deprecated/Shared/Deprecated_12_1_0.lua:16: 		forceinsecure();
Blizzard_Deprecated/Shared/Deprecated_12_1_0.lua:17: 	end
Blizzard_Deprecated/Shared/Deprecated_12_1_0.lua:18: 
Blizzard_Deprecated/Shared/Deprecated_12_1_0.lua:19: 	_G[var] = val;
Blizzard_Deprecated/Shared/Deprecated_12_1_0.lua:20: end
Blizzard_Deprecated/Shared/Deprecated_12_1_0.lua:21: 
Blizzard_Deprecated/Shared/Deprecated_12_1_0.lua:22: C_UnitAuras.AddPrivateAuraAppliedSound = function(sound)
Blizzard_Deprecated/Shared/Deprecated_12_1_0.lua:23: 	return C_UnitAuras.AddAuraSound(Enum.UnitAuraSoundTrigger.Added, sound);
Blizzard_Deprecated/Shared/Deprecated_12_1_0.lua:24: end;
Blizzard_Deprecated/Shared/Deprecated_12_1_0.lua:25: 
Blizzard_Deprecated/Shared/Deprecated_12_1_0.lua:26: C_UnitAuras.RemovePrivateAuraAppliedSound = C_UnitAuras.RemoveAuraSound;
Blizzard_Deprecated/Shared/Deprecated_12_1_0.lua:27: 
```

## Interpretation and accounting boundary

**MODELABLE-NOW means a bounded live-state slice is feasible under the stated standard, not that the exact row can be marked complete today.** Several old holds are obsolete: membership, mount lookup, indexed aura input, pending placement, destruction and aggregate publication already have real producers and concrete tests. Their native/general-domain exclusions do not turn the implemented producer into a placeholder. Review the existing proof before adding duplicate models or tests.

Conversely, an arbitrary `restricted: bool` does not explain an undocumented native restriction. It can test an explicitly INFERRED host policy, but cannot establish the new `HasRestrictions` delta's actual governing condition. Source-only unknowns remain below.

The supplied examples do not uniformly authenticate secrets today: `c_catalog_shop_products.rs:139-153` requires a public i32; `c_spell_maw_powers.rs:37-66` and the curio producer use the public spell-identifier reader. Their bounded explicit-state credit does not establish full argument-policy parity. `unwrap_secret` belongs before validation for **AllowedWhenUntainted** inputs. **NeverSecret** inputs must be rejected even for secure callers; **AllowedWhenTainted** cannot be implemented by blindly using an untainted-only unwrap and calling it parity.

Cache is the requested local retail cache, not proven identical to 12.0.5 build 67602. It contains a `Deprecated_12_1_0.lua` sound rename and generated `AddAuraSound`, not an `AddPrivateAuraAppliedSound` declaration. No retail-root file provenance marker was found by direct directory inspection. Cache citations are observations, not a silent assertion of 12.0.5 provenance.

## Row-by-row reassessment

Cached paths below are relative to `~/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/`; simulator paths are relative to the repository. Exact deltas are reproduced above, including each member of grouped rows.

### 245 — C_ActionBar.IsOnBarOrSpecialBar — MODELABLE-NOW

- Cache: `ActionBarFrameDocumentation.lua:826-837`: SpellIdentifier, AllowedWhenTainted, one nonnil bool.
- Current: `src/c_api/c_action_bar_spell_slots.rs:29-40,73-90`, live `SimState.action_bars` with macro/outfit precedence, not a placeholder. Special bars explicitly unmodeled. Registration replacement at `:23-27` supersedes legacy action-bar registration.
- Smallest additional state: empty host set `special_bar_spells: HashSet<u32>`; resolve public number/string alias using the existing registry, return direct membership OR set membership. Do not synthesize special membership from an unrelated visibility flag.
- Tests: existing `tests/action_bar_membership.rs`; add special-only true, neither false, both true, live insertion/removal, aliased identifier, collision with macro/outfit direct slot, environment isolation. Native AllowedWhenTainted handling/result secrecy remains separately unproved; no need to guess identifier name/link grammar for the explicit alias slice.

### 247 / 249 — C_CatalogShop.GetProductInfo / PurchaseProduct — STILL-BLOCKED for the retained HasRestrictions delta

- Cache: `CatalogShopDocumentation.lua:226-238,421-433`: both HasRestrictions and AllowedWhenUntainted; nullable productInfo versus nonnil canPurchase bool.
- Current GetProductInfo: `src/c_api/c_catalog_shop_products.rs:128-171`, live `SimState.catalog_shop_products.products` (`src/lua_api/state/sim_state.rs:345`), a real producer. PurchaseProduct: `src/lua_api/workarounds/temporary/housing_catalog_state.lua:643`, `__wow_noop`; no purchase producer in the product module.
- Missing evidence: what triggers each new restriction (caller security, combat, lockdown, catalog access, store availability or another context), and whether denial errors/returns nil/false. `HasRestrictions = true` alone does not identify that predicate; it is not metadata-only because restricted calls are observable.
- Feasible but non-closing bounded preparation: authenticate GetProductInfo's original ID with unwrap_secret; seed an explicit can-purchase map default empty and have PurchaseProduct return one bool, with host-declared request recording only if specified. Test populated/missing/live updates, genuine taint, secure-secret success, denial before malformed-ID validation, output arity and isolation. None of this establishes either retained restriction row. Real commerce/payment/acquisition is neither authorized nor needed.

### 253 — C_ChatInfo.ReplaceIconAndGroupExpressions — MODELABLE-NOW (bounded public-text producer)

- Cache: `ChatInfoDocumentation.lua:485-498`: input AllowedWhenTainted; noIconReplacement and noGroupReplacement are optional bools, individually NeverSecret; nonnil string output.
- Current: `src/lua_api/workarounds/temporary/c_chat_info_defaults.rs:27`, returns `tostring(text or "")`; ignores both flags and performs no replacement. Placeholder.
- Smallest model: empty explicit host maps for known icon-expression and group-expression replacements; Rust producer processes supplied text against those exact keys, independently disabling each class when its flag is true. This is host-authored expression vocabulary, not invented native grammar. Empty maps return unchanged valid text, not fabricated substitutions.
- Tests: seed two distinct concrete expression keys; both replacements, each disabled independently, nil/false flags, public text unchanged when no matches, live map updates and isolation. Secret arg2/arg3 rejection for both secure and genuinely tainted callers before parsing or producing text, including malformed public text plus secret flag; side-effect-free rejection. Full secret arg1 AllowedWhenTainted/output propagation needs separate VM-policy proof, not an untainted-only unwrap shortcut.

### 260 — C_DelvesUI.GetTieredEntrancePDEID — MODELABLE-NOW

- Cache: `DelvesUIDocumentation.lua:315-320`: no arguments, one required numeric pdeID; no MayReturnNothing.
- Current: `src/lua_api/globals/missing_surface/delves_ui.rs:287-289`, always 77011. Placeholder, despite unrelated newly state-backed delve/curio APIs.
- Smallest model: one host integer `tiered_entrance_pde_id`, default 0, read live. Explicitly label zero as simulator empty-state policy, not native sentinel evidence. Removal of MayReturnNothing is observable independently of actual game ID acquisition.
- Tests: empty state returns exactly one numeric zero; seed 77012 and 88021 and observe live transitions; zero stays one result rather than nil/zero results; environment isolation. No argument authentication needed.

### 270 — C_HouseExterior.SelectCoreFixtureOption — MODELABLE-NOW

- Cache: `HouseExteriorUIDocumentation.lua:160-168`: fixtureID number; attachedDecorAction HousingFixtureDecorAction default Store; when replacement is a variant/recolor, attachments **always** reparent to new fixture.
- Current: `src/lua_api/workarounds/temporary/housing_catalog_state.lua:899`, noop. Existing real adjacent system: `src/c_api/c_housing/exterior.rs:25-78`, explicit selection/options/attached placements; `exterior/runtime.rs:17-42` registers SelectFixtureOption but NOT SelectCoreFixtureOption; `exterior/mutation.rs:54-69` and `exterior/storage.rs:16-25` already implement related Store/Detach transactions.
- Smallest model: explicit optional core-fixture selection, eligible core options with a host-supplied equivalence/recolor group, and placement attachment IDs. Use existing catalog variant storage and placed records; do not infer recolor equivalence from IDs. A same-core variant swap preserves attachment identities and reparents regardless of action. A genuinely different core applies Store or Detach to the affected attachments only.
- Tests: omitted action=Store, explicit Store counts/removal, Detach preserving placements but clearing parent, recolor reparent for BOTH actions with counts unchanged, unaffected attachments, host updates/isolation, secret original fixture/action authentication before enum/default/lookup, blocked mutation unchanged, valid secure-secret call. Reuse transaction validation rather than adding a Lua recording shim. Geometry/rendering excluded; this is observable ownership/storage state, not the 3D gap.

### 278 / 279 — C_HousingBasicMode.StartPlacingNewDecor — MODELABLE-NOW, already implemented bounded slice

- Cache: `HousingBasicModeUIDocumentation.lua:185-192`: AllowedWhenUntainted, full HousingCatalogEntryVariantID.
- Current: `src/c_api/c_housing/basic_mode.rs:28-41`, full key selects live catalog variant and stores `housing.pending_new_decor`; `basic_mode/pending_input.rs` authenticates original selector and fields. Actual state, not merely a table-shape guard.
- Smallest model is existing empty variant catalog + optional pending key; no new state required for the identifier delta. Full placement is not what the two retained name/type annotations themselves say.
- Tests already authored: `tests/housing_pending_decor.rs` covers different variants sharing base identity, eligibility, replacement, cancellation, live mutation, secret original selector/fields, tainted rejection before validation and isolation. Reuse the B71 ledger quoted above. Native eligibility/finish/stock/placement event semantics remain inferred or outside this bounded slice; do not invent them merely to close the row.

### 281 / 282 — C_HousingCatalog.DestroyEntry — MODELABLE-NOW, already implemented bounded slice

- Cache: `HousingCatalogUIDocumentation.lua:31-39`: full variant ID, destroyAll bool, AllowedWhenUntainted; “If true, deletes all entries within the stack; If false, will only delete one”.
- Current: `src/c_api/c_housing/catalog/storage.rs:25-54`, mutates selected full-key variant's stored/destroyable counts; `catalog/destroy_input.rs` authenticates originals and fields; `storage.rs:120-133` publishes committed storage changes. State-backed.
- Existing model is sufficient to exercise the variant-ID change; retain explicit eligible-subset inference for mixed stacks rather than claiming quoted “all entries” parity.
- Existing `tests/housing_destroy_entry.rs`: two variants sharing a base, one/all deletion, live counts, blocked/unknown keys, event/query visibility, secure actual secret selector+destroyAll, denial before validation, GC/reentry/isolation. Reuse B72 ledger. Exact all-stack/native behavior still requires evidence, not a replacement placeholder or whole-row claim.

### 291 — C_MountJournal.GetMountFromSpell — MODELABLE-NOW, already state-backed public identifiers

- Cache: `MountJournalDocumentation.lua:249-260`: SpellIdentifier, AllowedWhenTainted; nullable mountID.
- Current: `src/c_api/c_mount_spell_lookup.rs:17-32`: explicit alias resolution then live `world.mounts` lookup; no canned return or C_Spell substitution.
- Existing mount records and alias registry are the smallest model; no new map duplicating mounts needed.
- `tests/mount_spell_identifier.rs` already covers numeric/host string aliases, misses, record mutation and environment isolation. Native alias grammar and AllowedWhenTainted secret permission remain separate. Secret rejection by the public reader is conservative, not proof that the native input contract is complete.

### 342 / 347 — C_TooltipInfo.GetUnitBuff / GetUnitDebuff — MODELABLE-NOW, already state-backed input delta

- Cache: `TooltipInfoDocumentation.lua:1239-1255,1279-1295`: unit arg1 no NeverSecret; AllowedWhenUntainted; RequiresUnitAuraAccess and SecretWhenUnitAuraRestricted; MayReturnNothing.
- Current: `src/c_api/c_tooltip_info_indexed_aura.rs:23-50,92-113` authenticates all originals before parsing and reads indexed live helpful/harmful player/party aura stores; `src/lua_api/globals/missing_surface/tooltip_info/mod.rs:109` builds actual selected-aura tooltip. These are real producers. Helpful target handling and deliberately unsupported harmful target are not general-unit coverage.
- Existing per-unit stores are sufficient for the removed NeverSecret input annotation. `tests/tooltip_unit_buff_security.rs` and `tests/tooltip_unit_debuff_security.rs` cover secure secrets reaching distinct populated unit/index/filter outputs, true tainted rejection, malformed argument ordering, misses, host mutation/GC and isolation. Existing B79/B81 ledgers are recorded above.
- If expanding beyond this exact input delta, seed per-unit explicit aura-access/restricted-output classification and wire the real tooltip return to it; test denied access and secret output with real selected aura data. That is separate scope: do not make it an artificial prerequisite for crediting the already tested input removal, or call whole-row parity established.

### 650 / 651 — HousingCatalogEntryInfo.totalNumStored / totalNumPlaced — MODELABLE-NOW, already explicit-state publication

- Cache: `HousingCatalogUIDocumentation.lua:537-557`: both required numbers; stored across variants excluding unredeemed instances; placed across houses/plots/variants.
- Current: `src/c_api/c_housing/catalog.rs:35-44`, explicit Option<u32> inputs; `catalog/snapshot.rs:104-123`, reads them live into populated base DTOs, no sums inferred from partial variant data. Real producer, not DTO shape on fabricated record.
- Smallest model is existing host base records with supplied `Some(0)` or concrete counts; empty default catalog contains no records. No additional simulator sums or native defaults needed for bounded publication credit. For mandatory-number conformance of every existing host record, require supplied counts at host record creation (or make them required u32 fields); do NOT silently turn unknown None into native zero.
- Existing `tests/housing_catalog_aggregates.rs`: all three getters, concrete 37/11 independent of variant 3/5, supplied zero, unsigned large values, input and Lua snapshot mutation, environment isolation and GC. Existing None→nil remains an explicit gap against required cached fields. Full DTO/synchronization/native defaults are not proved; source should retain that qualification even if bounded accounting is reconciled.

### 544 / 546 — FontString.GetFont / SetFont FontAsset — STILL-BLOCKED for FontAsset's new runtime domain

- Cache: `SimpleFontStringAPIDocumentation.lua:121-132,500-515`: nullable FontAsset return; required FontAsset argument; RequiresValidFontAsset, AllowedWhenUntainted, success bool.
- Current: `src/lua_api/frame/methods/text_attribute_event/text/formatting.rs:330-390,392-421`: frame font stored as String; setter uses string conversion, getter publishes a string path (or hardcoded FRIZQT default). Already state-backed path behavior, not a font-asset representation model.
- Missing evidence: concrete accepted FontAsset Lua representations (string path versus numeric fileDataID versus another value), conversion/resolution and GetFont return preservation/canonicalization; generated cache names the type but does not define it. A round trip of the existing path string proves old cstring behavior, not the new type domain.
- Once supplied: smallest host registry of explicitly valid assets plus per-frame asset representation, live Rust setter/getter; test each evidenced representation, invalid/unregistered asset/no mutation, result arity, host update/isolation, untainted secret input and tainted rejection before validation. Do not classify as metadata-only without establishing runtime equivalence, and do not invent numeric support merely because other FileAsset types permit it.

### 534 / 542 — PlayerModel.SetUnit / ModelSceneActorBase.SetModelByUnit — MODELABLE-NOW for identity denial; 3D success remains OUT-OF-SIMULATOR

- Cache: `FrameAPICharacterModelBaseDocumentation.lua:240-254` and `FrameAPIModelSceneFrameActorBaseDocumentation.lua:447-465`: RequiresDeclassifiedUnitIdentity, AllowedWhenUntainted, success bool. Source lines194-195 further require denied call one nil/no error.
- Current inspected wiring: `src/lua_api/frame/methods/widgets/model.rs:341-350` actor setter stores last_unit and returns true without identity guard; `:689` SetUnit still SKIP_3D_RENDERING. `model/model_unit.rs:12-43` exists with unwrap-first, GUID secrecy denial and binding assignment, but its existence is NOT evidence both registrations invoke it. `src/lua_api/state/sim_state.rs:242` has host identity_secret_guids; current source is partly staged.
- Smallest model: reuse existing identity GUID set + resolver and `player_model_state.last_unit`; wire both existing setters to common assign_unit. No 3D engine or new SimState needed. Denied ordinary token must return exactly one public nil without mutation for secure AND tainted callers; secure secret-token authentication is a different input condition.
- Tests authored in `tests/cast_events_identity.rs`: each actual method separately, existing prior binding, exact pcall arity/no error, GUID aliases, host recovery, missing identity, genuine addon taint and environment isolation. Add secure actual secret token and tainted secret-before-validation cases if claiming full argument authentication. Spec still says unrun; this reassessment ran none.
- Explicit exclusion accounting must say: “3D mesh/model loading, rendering, auto-dress/visual flag effects and native load-success are intentionally unsupported; host binding true is simulator assignment only. RequiresDeclassifiedUnitIdentity denial and nil/no-error are IN scope and tracked separately.” A blanket “model methods out of scope” would wrongly erase the retained observable delta.

## Prose rows, chronology and exact reversals

All line references in this section are to `data/patch-api/sources/12.0.5-api-changes.txt` unless a cached/provider path is given. Exact original text for each row is already preserved in the register-delta section.

### 03-12-055 — METADATA-ONLY

Historical motivation and an abandoned hope to retire Private Auras, not a promised 12.0.5 API behavior. There is no standalone cached declaration or provider for historical design intent. Do not award runtime coverage or exclude the actual private-aura APIs through this classification.

The next paragraph explicitly reverses that hope, line56:
> “Unfortunately, as Midnight Alpha & Beta progressed it became clear that relying on Secret Values alone was not going to be enough to prevent addon automation involving auras. And so, we ended up having to go in almost the exact opposite direction: we made MOST encounter debuffs private auras rather than just a select few.”

### 03-12-056 — STILL-BLOCKED for its observable encounter/display claims

Not just metadata: it describes most encounter debuffs being private, private displays lacking parity and lacking nameplate support. Cache's nearest query is `UnitAuraDocumentation.lua:69-80` AuraIsPrivate(SpellIdentifier)→bool; this is not a definition of the encounter set or display differences. `src/lua_api/workarounds/temporary/private_aura_state.rs:79-80` implements callback plumbing; it cannot establish classification/display fidelity. No explicit AuraIsPrivate/AddPrivateAuraAppliedSound/AddAuraSound producer was found in Rust/Lua source search. No standalone provider represents this entire paragraph.

Missing: exact relevant encounter/debuff classification fixture, concrete absent display features and nameplate behavior, plus an observable acceptance boundary. A host-authored private-spell set can model a bounded classifier, but cannot establish “MOST” or unspecified parity differences. The paragraph itself says this describes Midnight Season1, not a precise new 12.0.5 signature. No later quoted source reverses the whole paragraph; do not fabricate one.

### 03-12-032 — MODELABLE-NOW

Cache: no generated standalone declaration; actual restricted ENV function is `../Blizzard_RestrictedAddOnEnvironment/RestrictedEnvironment.lua:216-223`, calls GetOutfitInfo(outfitID) and returns playerFacingOutfitIndex or nil. Current underlying provider `src/c_api/c_transmog_outfit_info/catalog.rs:44-55,82-96,117-137` reads live `SimState.transmog_outfit_catalog` (`sim_state.rs:179`), empty default, unwrap before numeric validation. Existing fallback secure-handler environment `src/lua_api/globals/security/secure_handler.rs:82-103` lacks the helper; loaded Blizzard ENV supplies it when its addon runs. No direct simulator helper occurrence was found.

Smallest model: reuse existing outfit catalog, no duplicate index map. Exercise the **actual loaded restricted ENV/snippet** function; ensure the proper environment can invoke the existing getter. Only implement simulator-side missing restricted-environment exposure if the behavioral test demonstrates it is absent; do not modify vendor Lua.

Tests: two seeded outfit IDs with different facing indices, unknown ID→nil, live index changes, environment isolation, helper value selecting the intended outfit in the /outfit consumer, actual secure snippet not global-API-only substitute, secret ID permitted/denied at the original getter boundary. Existing `tests/outfit_action_command.rs` is consumer coverage, not automatically proof the restricted helper exists.

### 03-25-074 — MODELABLE-NOW, mixed final restrictions rather than old blanket combat rule

Cache: `PartyInfoDocumentation.lua:63-66` ConfirmConvertToRaid HasRestrictions; `:507-515` PromoteToLeader HasRestrictions/AllowedWhenUntainted; ready-check/countdown/ping/loot signatures have corresponding current declarations. No single cache declaration expresses the entire prose list.

Current real producers: `src/c_api/c_party_info.rs:227-251` leadership/everyone-assistant mutation, without the new combat guard; `src/lua_api/globals/group_verbs.rs:162-185` ready-check mutation guarded by chat_messaging_lockdown; `src/c_api/c_party_info/countdown.rs:28`, `ping_restrictions.rs:24`, `loot_method.rs:61` already use the chat guard. No explicit ConvertToParty/ConvertToRaid/ConfirmConvertToRaid producer was found in simulator .rs/.lua sources; don't call an absent conversion model a proven live producer. Legacy promotion wrapper in cache `../Blizzard_DeprecatedPartyInfo/Deprecated_PartyInfo.lua:28-29` delegates to C_PartyInfo.

Partial reversal, line169, verbatim:
> “The recent restrictions to countdown, ready check, ping and loot method APIs have been loosened to only apply when in chat messaging lockdown rather than in all combat.”

This quote does NOT reverse PromoteToLeader/PromoteToAssistant/DemoteAssistant/SetEveryoneIsAssistant or party/raid conversion; do not silently exempt them. Minimum: existing player.in_combat for still-combat-restricted verbs; existing chat_messaging_lockdown for the explicitly loosened subset; real host group/assistant/conversion state and guarded mutation for missing conversions. Seed no active group/assistants or pending conversion by default. Authentication before validation; reject before state changes/events. Per-member assistant state is preferable to falsely treating every individual promotion as everyone_assistant.

Tests: each listed verb individually, all four combat/chat combinations for the loosened subset, combat blocked/out-of-combat allowed for remaining verbs, populated state changes on allowed calls, unchanged roster/settings/conversion/events on denial, public tainted versus secure-secret input handling, live flags and environment isolation. Historical rule should be accounted as superseded **for that subset**, linked to final behavior, not marked metadata-only or enforced as combat-wide today.

### 03-25-076 — STILL-BLOCKED

Cache: `RaidMarkersDocumentation.lua:95-103` SetRaidTarget(target,index), HasRestrictions/AllowedWhenUntainted; it does not explain the random sticky failure. Current `src/lua_api/globals/targeting_verbs.rs:408-429` resolves a live GUID, moves/removes icon state and publishes updates; `SimState.unit_raid_target_icons` at `sim_state.rs:259` is real state-backed marker storage, not placeholder.

Missing: reproduced sticky-client failure or a native/recorded sequence of permission/context transitions identifying why marking remains blocked until restart. Generic repeated SetRaidTarget or a fresh environment proves marker assignment, not recovery from this bug. Adding an invented `can_set_markers` latch would manufacture rather than fix the failure. No source text found reversing this bugfix statement. Required future test must reproduce the actual transition/lifetime ordering and show recovered assignment without restarting, not merely unit-test a chosen bool.

### 03-25-088 — MODELABLE-NOW via final policy, already mostly implemented

Cache: `UnitDocumentation.lua:2321-2335`: RequiresComparableUnitTokens, SecretWhenUnitComparisonRestricted, AllowedWhenUntainted. Current `src/lua_api/globals/unit_misc.rs:317-389` applies final permitted-token rules and compares live resolved GUIDs; disallowed calls currently return zero results at `:375-376` (nil in an assignment, not explicit one-nil arity). State-backed identity, not a secret-bool placeholder.

Reversal, lines141-144, verbatim:
> “The UnitIsUnit API has been adjusted to apply the following rules:”
> “If either unit is one of the following, the comparison is permitted: "player", "pet", "vehicle", "mouseover", "target", "softenemy", "softfriend", "softinteract", "focus", "none", "npc", "questnpc"”
> “If either unit is a party/raid token (or a pet of one) and the other is NOT a compound unit token ("boss1target"), nameplate token, or "targettarget"/"focustarget", the comparison is permitted.”
> “All other comparisons are disallowed and return nil.”

No new state required: seed existing GUID identities and exercise the final whitelist/counterpart policy in both argument orders, specifically player/targettarget, player/focustarget, party/compound and neither-whitelisted comparisons; mutate identity equality live, secret tokens with true caller taint and environment isolation. Retain zero-result versus explicit nil as an arity qualification unless native/source evidence resolves it. Do NOT implement the superseded blanket secret-return sentence. Existing `tests/unit_comparison_permissions.rs` and `tests/unit_token_identity_secrecy.rs` should be inspected before creating duplicates.

### 03-25-117 — MODELABLE-NOW via final Add restriction / Remove unrestriction, NOT the historical pair

Current Remove producer: `src/c_api/private_aura_sounds.rs:11-16,60-66`, host live-ID set, no combat/encounter gate; public ID reader is conservative and lacks secure-secret acceptance. No explicit AddPrivateAuraAppliedSound/AddAuraSound simulator producer was found. Cache has newer `UnitAuraDocumentation.lua:11-24` AddAuraSound HasRestrictions/AllowedWhenUntainted and UnitAuraSoundInfo in `UnitConstantsDocumentation.lua:43-51`; cached `../Blizzard_Deprecated/Shared/Deprecated_12_1_0.lua:22-26` maps old Add to Added trigger and old Remove to RemoveAuraSound. Version drift must not be treated as an original 12.0.5 signature.

Partial reversal, line168, verbatim:
> “Removed the restrictions that we recently placed on the AddPrivateAuraAnchor, RemovePrivateAuraAnchor, SetPrivateWarningTextAnchor and RemovePrivateAuraAppliedSound APIs. The AddPrivateAuraAppliedSound API is still restricted, but this restriction is now only in place during encounters and during M+/PvP matches.”

Smallest non-playback model: explicit host restriction context (encounter/M+/PvP booleans, false default), host-declared sound request-to-registration result records (empty default), and existing live registrations. Add authenticates original request/fields before validation, disallows addon addition in the stated context before recording/registering anything; outside it, read a host-declared nonempty request/result and publish its registration. Remove authenticates its original ID and mutates live registration regardless of those contexts. Confirm original 12.0.5 request DTO using the retained signature/probe before claiming native parameter shape; modern wrapper route can receive only separately scoped cache proof.

Tests: successful concrete Add with stored request and exact ID, Remove deleting same live ID, all context flags independently and combined, no automatic combat proxy, allowed secure versus blocked addon context where required by established call policy, secure-secret original/fields success and tainted-secret auth denial before validation, unchanged registration on blocked/malformed input, live context changes/isolation. Audio playback/acquisition is excluded explicitly, not used to erase restriction behavior. Native exception/denial result convention remains qualified; a contextual simulator error is inference, not source-derived exact text.

## Ranked MODELABLE-NOW work (smallest first)

### Now — reuse established live models before writing code

1. **342/347, 278/279, 281/282:** reconcile existing input-delta proof with current standard; inspect the committed behavior ledgers and avoid duplicate tests/models. No whole-provider/native closure.
2. **291, 650/651:** reuse existing public alias/mount and explicit aggregate publication proof; retain conservative secret/missing-required-data gaps.
3. **03-25-088:** reconcile final-policy chronological accounting using existing GUID model/tests; no resurrection of secret-return proposal.
4. **534/542:** wire already-authored common guard into BOTH real setters, then prove each actual call boundary; preserve intentional 3D exclusion.
5. **260:** replace the hardcoded PDEID with one live host scalar and exact-one-result tests.

### Later — bounded additions

6. **03-12-032:** loaded restricted-ENV test over existing outfit catalog and /outfit consumer; implement exposure only if proven missing.
7. **245:** add explicit special-bar membership set to existing direct-spell producer; keep AllowedWhenTainted/native grammar separate.
8. **253:** replace identity-only text shim with host-seeded expression transformation and independent NeverSecret flags.
9. **03-25-117:** real explicit sound-registration producer plus precisely scoped final Add guard; Remove already has live IDs.
10. **270:** model core options/equivalence and actual Store/Detach/reparent ownership changes using existing exterior/storage transactions.
11. **03-25-074:** finish per-verb group/conversion backing state and final mixed combat/chat restrictions; widest bounded group here.

## Verdict summary

| Rows | Verdict | Decisive reason |
|---|---|---|
|245|MODELABLE-NOW|Real direct membership; add explicit special-bar set|
|247,249|STILL-BLOCKED|HasRestrictions predicate/denial behavior not specified|
|253|MODELABLE-NOW|Real host replacement model + observable NeverSecret flags|
|260|MODELABLE-NOW|One host PDEID scalar; no zero-result return|
|270|MODELABLE-NOW|Core selection + Store/Detach/recolor reparent state|
|278,279|MODELABLE-NOW|Already live full-variant pending request, not shape-only|
|281,282|MODELABLE-NOW|Already live full-variant deletion, not shape-only|
|291|MODELABLE-NOW|Already live mount lookup/explicit public aliases|
|342,347|MODELABLE-NOW|Already live aura queries + unwrap-before-validation|
|650,651|MODELABLE-NOW|Already live explicit counts; None gap retained|
|544,546|STILL-BLOCKED|FontAsset runtime representations/canonicalization unknown|
|534,542|MODELABLE-NOW|Identity denial in scope; visual loading alone OUT-OF-SIMULATOR|
|03-12-055|METADATA-ONLY|Abandoned historical design intent, no runtime contract|
|03-12-056|STILL-BLOCKED|Encounter set/display parity assertions unspecified|
|03-12-032|MODELABLE-NOW|Actual cached ENV helper + existing outfit records|
|03-25-074|MODELABLE-NOW|Final mixed restrictions; reversal applies only to stated subset|
|03-25-076|STILL-BLOCKED|No sticky-permission failure sequence/reproducer|
|03-25-088|MODELABLE-NOW|Final permitted-token/nil policy replaces old proposal|
|03-25-117|MODELABLE-NOW|Final Add context restriction; Remove restriction removed|

No entire requested row is wholly OUT-OF-SIMULATOR. 3D is a sub-capability exclusion, not an excuse to omit 534/542 identity denial. No audit statuses were changed. No implementation or test-passing claim is made.

## Read-only file snapshot hashes

These are hashes at report finalization, not a claim that concurrent external edits are frozen.

- `src/c_api/c_catalog_shop_products.rs` SHA256 `2305ac86fef8c9ffbb2db18be064446b4be2349e94438bfb13f7bcd244519c49`
- `src/c_api/c_action_bar_spell_slots.rs` SHA256 `16073cf8b5788f9d81dbc4499c50e9d197f1bf637c9eb480e71036f82c638374`
- `src/c_api/c_tooltip_info_indexed_aura.rs` SHA256 `ea567153f740258c7120b039957f06b40ce568fb8575f9bef818cebfe4fad37f`
- `src/c_api/c_housing/basic_mode.rs` SHA256 `bc8140a9376c86001b9923e59c228c4f806c9cb072f060aa2116d5c6bdf37252`
- `src/c_api/c_housing/catalog/storage.rs` SHA256 `1bb97653654ea476622083c33a1ce2f198d911cec4591867b560d8787e8ce18e`
- `src/c_api/c_housing/catalog/snapshot.rs` SHA256 `61168ea643e73e85cdd30e3f3d5955c9b0591d4a7071f0be3b149394cea10897`
- `src/lua_api/frame/methods/widgets/model.rs` SHA256 `04c5d7191d67d671640ca567635dab8ba1926591cb9f990244451a4c2f47e925`
- `src/lua_api/frame/methods/widgets/model/model_unit.rs` SHA256 `416a0bc6be11d6197a8b7a0c1b63b044bb62faaaf89ab1a75491312857587dfb`
- `data/patch-api/sources/12.0.5-page-coverage.json` SHA256 `ecee38622055026312edc7b2d0ae98f713ec556095aae997948550c5f9193d2e`
