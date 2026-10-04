# Retail 12.1.0 bounded implementation batches

Preparation only; no implementation/testing is authorized by this audit. Breadth first: prove current producers before adding models. Blocked historical/DB/source rows have no design or batch. Every substantive nonblocked row occurs in one batch; every metadata row is immediately accountable without runtime credit. Batches are at most six related source rows, sized as logical scope rather than measured runtime. Shared state producers must be integrated once. No agents, deployment, vendor changes or 3D work proposed.

## Tests-only batches first

For implemented-needs-proof batches, add/read behavioral tests only, using the actual default Retail profile and real template/loader route where applicable. Do not reimplement existing Lua. A failure proving a missing producer changes that row to modelable before any code work. Existing tests cited in triage were NOT run.

### C01 — creation (3 rows; implemented-needs-proof)

Rows: `prose-undated-009`, `prose-2026-06-18-037`, `prose-2026-07-23-207`.

Boundary: AuraContainer/AuraButton type construction and template loading; this does not establish managed presentation or security.

Work: **tests only**. Use the exact source claims below as acceptance assertions; do not substitute namespace/member presence or inspect generated code shape.

- L9: Added AuraContainer and AuraButton intrinsic frames.
- L37: Aura Containers and Aura Buttons are new Lua object types that allow addons to display auras in custom ways. Here’s a small example showing how they can be used:
- L207: Aura containers can now be created by addons during combat.

Acceptance: concrete nonempty fixtures, non-default state transitions/readback, output arity/optionality and relevant rejection/taint/ownership boundaries; preserve explicitly bounded shape-only scope and no live-service claim. For layout/presentation rows, actual frame/region state is required; metadata round trips alone do not prove rendering.

Existing test entry points: `src/loader/tests/wow_api_globals/frames_and_attributes.rs:337`.

### C02 — framexml-migrations (2 rows; implemented-needs-proof)

Rows: `prose-undated-011`, `prose-undated-012`.

Boundary: Unmodified cached Lua supplies the relocated helpers. Prove actual addon calls and removed old publication under Retail, including mouse offsets and failed/disabled addon loads.

Work: **tests only**. Use the exact source claims below as acceptance assertions; do not substitute namespace/member presence or inspect generated code shape.

- L11: UIParentLoadAddOn has been moved to LoadAddOnWithErrorHandling.
- L12: MouseIsOver has been moved to InputUtil.IsMouseOver.

Acceptance: concrete nonempty fixtures, non-default state transitions/readback, output arity/optionality and relevant rejection/taint/ownership boundaries; preserve explicitly bounded shape-only scope and no live-service claim. For layout/presentation rows, actual frame/region state is required; metadata round trips alone do not prove rendering.

Existing test entry points: `patch-tests/patch_12_1/remaining_observations.rs:117`.

### C03 — bootstrap (3 rows; implemented-needs-proof)

Rows: `prose-2026-06-18-055`, `prose-2026-06-18-056`, `prose-2026-06-18-057`.

Boundary: TOC Bootstrap annotations and bootstrap-only LoD startup scheduling are implemented. Prove enabled/disabled addons, dependency ordering, one-time phase execution and subsequent full load under default Retail.

Work: **tests only**. Use the exact source claims below as acceptance assertions; do not substitute namespace/member presence or inspect generated code shape.

- L55: Load-on-Demand addons can now specify that specific files in the TOC should load on startup through a new per-file [Bootstrap] directive.
- L56: This still requires that the addon be enabled in order for these files to load.
- L57: UIParent.lua has been heavily refactored, with all of the code that previously handled loading LoD addons moved into the addons themselves, taking advantage of the new [Bootstrap] directive.

Acceptance: concrete nonempty fixtures, non-default state transitions/readback, output arity/optionality and relevant rejection/taint/ownership boundaries; preserve explicitly bounded shape-only scope and no live-service claim. For layout/presentation rows, actual frame/region state is required; metadata round trips alone do not prove rendering.

Existing test entry points: `src/loader/tests/lua_loading.rs:176`, `src/toc/tests.rs:388`.

### C04 — onupdate (2 rows; implemented-needs-proof)

Rows: `prose-2026-06-18-058`, `prose-2026-06-18-059`.

Boundary: Numeric OnUpdateMode state and dispatch modes are implemented, including hidden RunAlways and one-shot reset/rearm. Existing actual managed-aura dirty-phase test is meaningful but unexecuted in this audit.

Work: **tests only**. Use the exact source claims below as acceptance assertions; do not substitute namespace/member presence or inspect generated code shape.

- L58: Added a new API Frame:SetOnUpdateMode(mode), which lets you specify when the OnUpdate script on a frame should run.
- L59: The options are Disabled, RunWhenVisible (default), RunWhenVisibleOnce, RunOnce, and RunAlways

Acceptance: concrete nonempty fixtures, non-default state transitions/readback, output arity/optionality and relevant rejection/taint/ownership boundaries; preserve explicitly bounded shape-only scope and no live-service claim. For layout/presentation rows, actual frame/region state is required; metadata round trips alone do not prove rendering.

Existing test entry points: `tests/on_update_modes.rs:35`, `tests/on_update_modes.rs:152`.

### C05 — xml (4 rows; implemented-needs-proof)

Rows: `prose-2026-06-18-064`, `prose-2026-06-18-065`, `prose-2026-06-18-066`, `prose-2026-06-18-067`.

Boundary: XML local KeyValues, Mixins blocks and qualified mixin lookup have loader producers; prove per-addon local table identity, nested lookup, precedence and no cross-addon leakage.

Work: **tests only**. Use the exact source claims below as acceptance assertions; do not substitute namespace/member presence or inspect generated code shape.

- L64: KeyValues can now specify that their value should be pulled directly from the private addon table. Example usage: <KeyValue key="myKey" type="local"/>
- L65: Mixins can now be added on an object using a new <Mixins> element.
- L66: Using this element allows you to use the source="local" specifier to indicate the mixin lives in the private addon table.
- L67: Mixins added on an object (either through the Mixins element or the regular mixin="myMixin" attribute) can also now be nested within tables.

Acceptance: concrete nonempty fixtures, non-default state transitions/readback, output arity/optionality and relevant rejection/taint/ownership boundaries; preserve explicitly bounded shape-only scope and no live-service claim. For layout/presentation rows, actual frame/region state is required; metadata round trips alone do not prove rendering.

Existing test entry points: `src/loader/tests/xml_basics.rs:343`, `tests/xml_secure_delegates.rs:7`.

### C06 — events (2 rows; implemented-needs-proof)

Rows: `prose-2026-06-23-083`, `prose-2026-06-23-088`.

Boundary: RegisterEvent/related mutation paths reject the EventRegistrations aspect and preserve existing listeners; needs default Retail acceptance proof.

Work: **tests only**. Use the exact source claims below as acceptance assertions; do not substitute namespace/member presence or inspect generated code shape.

- L83: EventRegistrations: When active, addons cannot register a frame for events.
- L88: Aura Containers have had the EventRegistrations Forbidden Aspect applied to them.

Acceptance: concrete nonempty fixtures, non-default state transitions/readback, output arity/optionality and relevant rejection/taint/ownership boundaries; preserve explicitly bounded shape-only scope and no live-service claim. For layout/presentation rows, actual frame/region state is required; metadata round trips alone do not prove rendering.

Existing test entry points: `tests/forbidden_aspect_creation.rs:155`, `tests/forbidden_aspect_creation.rs:190`.

### C07 — inheritance (1 rows; implemented-needs-proof)

Rows: `prose-2026-06-23-090`.

Boundary: SetParent/SetPoint must reject implicit acquisition of additional forbidden aspects before mutation; current shared native guard implements that boundary.

Work: **tests only**. Use the exact source claims below as acceptance assertions; do not substitute namespace/member presence or inspect generated code shape.

- L90: API calls such as SetParent and SetPoint will error if an object would implicitly gain any Forbidden Aspects that it does not already have.

Acceptance: concrete nonempty fixtures, non-default state transitions/readback, output arity/optionality and relevant rejection/taint/ownership boundaries; preserve explicitly bounded shape-only scope and no live-service claim. For layout/presentation rows, actual frame/region state is required; metadata round trips alone do not prove rendering.

Existing test entry points: `tests/forbidden_aspect_creation.rs:14`.

### C08 — aura-group (6 rows; implemented-needs-proof)

Rows: `prose-2026-07-07-119`, `prose-2026-07-07-120`, `prose-2026-07-07-122`, `prose-2026-07-07-123`, `prose-2026-07-07-125`, `prose-2026-07-07-126`.

Boundary: Current unmodified cached Lua has managed group/slot/filter/layout logic. Native filter/secrecy/lifecycle integration is incomplete; existing mock-provider tests are narrower than real addon creation and managed frame updates.

Work: **tests only**. Use the exact source claims below as acceptance assertions; do not substitute namespace/member presence or inspect generated code shape.

- L119: Added a new construct to AuraContainers: AuraGroups. Broadly speaking, you can think of an AuraGroup as a dynamic, self-managing collection of auras within an AuraContainer.
- L120: AuraContainers can have multiple AuraGroups, each with their own filters and settings.
- L122: Addons add AuraGroups to AuraContainers using a new API AddAuraGroup(groupKey, filterString, options).
- L123: The groupKey param is an arbitrary addon-defined string used to access the group after creation.
- L125: The options param is a table that can contain a number of optional settings.
- L126: maxFrameCount: The maximum number of aura frames to show in this group

Acceptance: concrete nonempty fixtures, non-default state transitions/readback, output arity/optionality and relevant rejection/taint/ownership boundaries; preserve explicitly bounded shape-only scope and no live-service claim. For layout/presentation rows, actual frame/region state is required; metadata round trips alone do not prove rendering.

Existing test entry points: `patch-tests/patch_12_1/aura_container.rs:67`, `tests/on_update_modes.rs:152`.

### C09 — aura-group (3 rows; implemented-needs-proof)

Rows: `prose-2026-07-07-127`, `prose-2026-07-07-128`, `prose-2026-07-07-129`.

Boundary: Current unmodified cached Lua has managed group/slot/filter/layout logic. Native filter/secrecy/lifecycle integration is incomplete; existing mock-provider tests are narrower than real addon creation and managed frame updates.

Work: **tests only**. Use the exact source claims below as acceptance assertions; do not substitute namespace/member presence or inspect generated code shape.

- L127: sortMethod and sortDirection: Used to control how auras in this group are sorted (see new enum AuraContainerSortMethod for choices).
- L128: initializeFrame: A callback function that is called for each AuraButton created.
- L129: templateNames: A list of xml templates to apply to each AuraButton (in addition to CustomAuraButtonTemplate).

Acceptance: concrete nonempty fixtures, non-default state transitions/readback, output arity/optionality and relevant rejection/taint/ownership boundaries; preserve explicitly bounded shape-only scope and no live-service claim. For layout/presentation rows, actual frame/region state is required; metadata round trips alone do not prove rendering.

Existing test entry points: `patch-tests/patch_12_1/aura_container.rs:67`, `tests/on_update_modes.rs:152`.

### C10 — aura-sound-remove-proof (1 rows; implemented-needs-proof)

Rows: `prose-2026-07-21-181`.

Boundary: RemoveAuraSound is already registered under retail-12-1-0 and delegates to the native registration removal model. Tests only for renamed publication, zero results, deletion/readback and legacy-wrapper forwarding; no actual playback or historical intermediate alias timing claim.

Work: **tests only**. Use the exact source claims below as acceptance assertions; do not substitute namespace/member presence or inspect generated code shape.

- L181: The RemoveAuraAppliedSound API has been renamed RemoveAuraSound to match.

Acceptance: concrete nonempty fixtures, non-default state transitions/readback, output arity/optionality and relevant rejection/taint/ownership boundaries; preserve explicitly bounded shape-only scope and no live-service claim. For layout/presentation rows, actual frame/region state is required; metadata round trips alone do not prove rendering.

Existing test entry points: `tests/private_aura_sound_removal.rs:154`.

### C11 — enum-proof (6 rows; implemented-needs-proof)

Rows: `enumerations-Enum-CompanionConfigSlotTypes-270`, `enumerations-Enum-CompanionConfigSlotTypes-271`, `enumerations-Enum-CooldownViewerCategory-272`, `enumerations-Enum-CooldownViewerCategory-273`, `enumerations-Enum-CooldownViewerCategory-274`, `enumerations-Enum-CooldownViewerCategory-275`.

Boundary: Current default Retail already has the listed enum member producers, including the shared 12.1 compatibility bootstrap. Prove exact value, metadata, no cross-environment mutation and default-profile publication; names alone are not numerical proof.

Work: **tests only**. Use the exact source claims below as acceptance assertions; do not substitute namespace/member presence or inspect generated code shape.

- L270: Enum.CompanionConfigSlotTypes (C_DelvesUI.GetUnseenCuriosBySlotType, C_DelvesUI.SaveSeenCuriosBySlotType)
- L271: + Flavor
- L272: Enum.CooldownViewerCategory (C_CooldownViewer.GetCooldownViewerCategorySet, C_CooldownViewer.GetCooldownViewerCooldownInfo)
- L273: + GroupBuff
- L274: + SpecAgnosticEssential
- L275: + SpecAgnosticTracked

Acceptance: concrete nonempty fixtures, non-default state transitions/readback, output arity/optionality and relevant rejection/taint/ownership boundaries; preserve explicitly bounded shape-only scope and no live-service claim. For layout/presentation rows, actual frame/region state is required; metadata round trips alone do not prove rendering.

Existing test entry points: No exact test located; add targeted behavior tests for this slice.

### C12 — enum-proof (6 rows; implemented-needs-proof)

Rows: `enumerations-Enum-CooldownViewerCategory-276`, `enumerations-Enum-CooldownViewerCategory-277`, `enumerations-Enum-EditModeAccountSetting-278`, `enumerations-Enum-EditModeAccountSetting-279`, `enumerations-Enum-EditModeMinimapSetting-280`, `enumerations-Enum-EditModeMinimapSetting-281`.

Boundary: Current default Retail already has the listed enum member producers, including the shared 12.1 compatibility bootstrap. Prove exact value, metadata, no cross-environment mutation and default-profile publication; names alone are not numerical proof.

Work: **tests only**. Use the exact source claims below as acceptance assertions; do not substitute namespace/member presence or inspect generated code shape.

- L276: + EquipSlotEssential
- L277: + EquipSlotTracked
- L278: Enum.EditModeAccountSetting (C_EditMode.SetAccountSetting)
- L279: + ShowRaidWarning
- L280: Enum.EditModeMinimapSetting
- L281: + IconScale

Acceptance: concrete nonempty fixtures, non-default state transitions/readback, output arity/optionality and relevant rejection/taint/ownership boundaries; preserve explicitly bounded shape-only scope and no live-service claim. For layout/presentation rows, actual frame/region state is required; metadata round trips alone do not prove rendering.

Existing test entry points: No exact test located; add targeted behavior tests for this slice.

### C13 — enum-proof (6 rows; implemented-needs-proof)

Rows: `enumerations-Enum-EditModeSystem-282`, `enumerations-Enum-EditModeSystem-283`, `enumerations-Enum-HousingResult-296`, `enumerations-Enum-HousingResult-297`, `enumerations-Enum-HousingResult-298`, `enumerations-Enum-HousingResult-299`.

Boundary: Current default Retail already has the listed enum member producers, including the shared 12.1 compatibility bootstrap. Prove exact value, metadata, no cross-environment mutation and default-profile publication; names alone are not numerical proof.

Work: **tests only**. Use the exact source claims below as acceptance assertions; do not substitute namespace/member presence or inspect generated code shape.

- L282: Enum.EditModeSystem (C_EditMode.ConvertLayoutInfoToString, C_EditMode.ConvertStringToLayoutInfo, C_EditMode.GetLayouts, C_EditMode.SaveLayouts, EDIT_MODE_LAYOUTS_UPDATED)
- L283: + RaidWarning
- L296: + BlueprintGenericImportError
- L297: + BlueprintStorageLimit
- L298: + BlueprintTypeInvalid
- L299: + BlueprintNotFound

Acceptance: concrete nonempty fixtures, non-default state transitions/readback, output arity/optionality and relevant rejection/taint/ownership boundaries; preserve explicitly bounded shape-only scope and no live-service claim. For layout/presentation rows, actual frame/region state is required; metadata round trips alone do not prove rendering.

Existing test entry points: `src/loader/tests/wow_api_globals/frames_and_attributes.rs:6`, `src/loader/tests/wow_api_globals/housing_result.rs:219`.

### C14 — enum-proof (6 rows; implemented-needs-proof)

Rows: `enumerations-Enum-HousingResult-300`, `enumerations-Enum-HousingResult-301`, `enumerations-Enum-HousingResult-302`, `enumerations-Enum-HousingResult-303`, `enumerations-Enum-HousingResult-305`, `enumerations-Enum-HousingResult-306`.

Boundary: Current default Retail already has the listed enum member producers, including the shared 12.1 compatibility bootstrap. Prove exact value, metadata, no cross-environment mutation and default-profile publication; names alone are not numerical proof.

Work: **tests only**. Use the exact source claims below as acceptance assertions; do not substitute namespace/member presence or inspect generated code shape.

- L300: + InvalidExteriorDocument
- L301: + BlueprintGenericExportError
- L302: + InvalidInteriorDocument
- L303: + BlueprintRequirementsUnmet
- L305: + BlueprintCodeInvalid
- L306: + InsufficientRoomBudget

Acceptance: concrete nonempty fixtures, non-default state transitions/readback, output arity/optionality and relevant rejection/taint/ownership boundaries; preserve explicitly bounded shape-only scope and no live-service claim. For layout/presentation rows, actual frame/region state is required; metadata round trips alone do not prove rendering.

Existing test entry points: `src/loader/tests/wow_api_globals/housing_result.rs:219`.

### C15 — enum-proof (6 rows; implemented-needs-proof)

Rows: `enumerations-Enum-HousingResult-307`, `enumerations-Enum-HousingResult-308`, `enumerations-Enum-HousingResult-309`, `enumerations-Enum-NamePlateStyle-310`, `enumerations-Enum-NamePlateStyle-311`, `enumerations-Enum-PingResult-312`.

Boundary: Current default Retail already has the listed enum member producers, including the shared 12.1 compatibility bootstrap. Prove exact value, metadata, no cross-environment mutation and default-profile publication; names alone are not numerical proof.

Work: **tests only**. Use the exact source claims below as acceptance assertions; do not substitute namespace/member presence or inspect generated code shape.

- L307: + BlueprintLocationInvalid
- L308: + BlueprintNameInvalid
- L309: + BlueprintVersionInvalid
- L310: Enum.NamePlateStyle
- L311: + Classic
- L312: Enum.PingResult (C_PingSecure.SendHitTestPing, C_PingSecure.SendPlayerItemPing, C_PingSecure.SendPlayerSpellPing, C_PingSecure.SendUnitPing, C_PingSecure.SetHitTestTargetAndSendPing)

Acceptance: concrete nonempty fixtures, non-default state transitions/readback, output arity/optionality and relevant rejection/taint/ownership boundaries; preserve explicitly bounded shape-only scope and no live-service claim. For layout/presentation rows, actual frame/region state is required; metadata round trips alone do not prove rendering.

Existing test entry points: `src/loader/tests/wow_api_globals/housing_result.rs:219`, `src/loader/tests/wow_api_globals/patch_12_0_0_nameplate_style_enums.rs:8`.

### C16 — enum-proof (6 rows; implemented-needs-proof)

Rows: `enumerations-Enum-PingResult-313`, `enumerations-Enum-PingSubjectType-314`, `enumerations-Enum-PingSubjectType-315`, `enumerations-Enum-PingSubjectType-316`, `enumerations-Enum-PingSubjectType-317`, `enumerations-Enum-TieredEntranceType-320`.

Boundary: Current default Retail already has the listed enum member producers, including the shared 12.1 compatibility bootstrap. Prove exact value, metadata, no cross-environment mutation and default-profile publication; names alone are not numerical proof.

Work: **tests only**. Use the exact source claims below as acceptance assertions; do not substitute namespace/member presence or inspect generated code shape.

- L313: + FailedSilent
- L314: Enum.PingSubjectType (C_Ping.GetDefaultPingOptions, C_Ping.GetTextureKitForType, C_Ping.SendMacroPing, C_PingSecure.SendHitTestPing, C_PingSecure.SendPlayerItemPing, C_PingSecure.SendPlayerSpellPing, C_PingSecure.SendUnitPing, C_PingSecure.SetHitTestTargetAndSendPing)
- L315: + ActionReady
- L316: + ActionOnCooldown
- L317: + ActionUnavailable
- L320: Enum.TieredEntranceType (C_DelvesUI.GetTieredEntranceType)

Acceptance: concrete nonempty fixtures, non-default state transitions/readback, output arity/optionality and relevant rejection/taint/ownership boundaries; preserve explicitly bounded shape-only scope and no live-service claim. For layout/presentation rows, actual frame/region state is required; metadata round trips alone do not prove rendering.

Existing test entry points: No exact test located; add targeted behavior tests for this slice.

### C17 — enum-proof (5 rows; implemented-needs-proof)

Rows: `enumerations-Enum-TieredEntranceType-321`, `enumerations-Enum-TooltipDataLineType-322`, `enumerations-Enum-TooltipDataLineType-323`, `enumerations-Enum-TooltipDataLineType-324`, `enumerations-Enum-TooltipDataLineType-325`.

Boundary: Current default Retail already has the listed enum member producers, including the shared 12.1 compatibility bootstrap. Prove exact value, metadata, no cross-environment mutation and default-profile publication; names alone are not numerical proof.

Work: **tests only**. Use the exact source claims below as acceptance assertions; do not substitute namespace/member presence or inspect generated code shape.

- L321: + Lairs
- L322: Enum.TooltipDataLineType
- L323: + ItemSpellTriggerOnUse
- L324: + ItemSpellTriggerOnEquip
- L325: + ItemSpellTriggerOnProc

Acceptance: concrete nonempty fixtures, non-default state transitions/readback, output arity/optionality and relevant rejection/taint/ownership boundaries; preserve explicitly bounded shape-only scope and no live-service claim. For layout/presentation rows, actual frame/region state is required; metadata round trips alone do not prove rendering.

Existing test entry points: `src/loader/tests/wow_api_globals/patch_12_1_5_tooltip_line_enums.rs:77`.

### C18 — private-anchor-input-shape (5 rows; implemented-needs-proof)

Rows: `structures-AddPrivateAuraAnchorArgs-329`, `structures-AddPrivateAuraAnchorArgs-330`, `structures-AddPrivateAuraAnchorArgs-331`, `structures-AddPrivateAuraAnchorArgs-332`, `structures-AddPrivateAuraAnchorArgs-333`.

Boundary: Input reader models the new flags and omits showCountdownFrame. Prove each default/non-default flag, retired output absence, invalid inputs and callback-visible state.

Work: **tests only**. Use the exact source claims below as acceptance assertions; do not substitute namespace/member presence or inspect generated code shape.

- L329: AddPrivateAuraAnchorArgs (C_UnitAuras.AddPrivateAuraAnchor)
- L330: - showCountdownFrame
- L331: + showDispelIcon
- L332: + showCooldownEdge
- L333: + showCooldownFrame

Acceptance: concrete nonempty fixtures, non-default state transitions/readback, output arity/optionality and relevant rejection/taint/ownership boundaries; preserve explicitly bounded shape-only scope and no live-service claim. For layout/presentation rows, actual frame/region state is required; metadata round trips alone do not prove rendering.

Existing test entry points: `tests/private_aura_anchors.rs:301`.

### C19 — bnet-account-field-shape (3 rows; implemented-needs-proof)

Rows: `structures-BNetAccountInfo-334`, `structures-BNetAccountInfo-335`, `structures-BNetAccountInfo-336`.

Boundary: Retail writer supplies friendTags from explicit friend state and friendLevel as fixed 0. Prove the bounded published shape across all advertised getters; live friend-level progression remains unmodeled and uncredited.

Work: **tests only**. Use the exact source claims below as acceptance assertions; do not substitute namespace/member presence or inspect generated code shape.

- L334: BNetAccountInfo (C_BattleNet.GetAccountInfoByGUID, C_BattleNet.GetAccountInfoByID, C_BattleNet.GetFriendAccountInfo)
- L335: + friendLevel
- L336: + friendTags

Acceptance: concrete nonempty fixtures, non-default state transitions/readback, output arity/optionality and relevant rejection/taint/ownership boundaries; preserve explicitly bounded shape-only scope and no live-service claim. For layout/presentation rows, actual frame/region state is required; metadata round trips alone do not prove rendering.

Existing test entry points: `src/loader/tests/wow_api_globals/startup_globals.rs:913`.

### C20 — bnet-class-filename (2 rows; implemented-needs-proof)

Rows: `structures-BNetGameAccountInfo-337`, `structures-BNetGameAccountInfo-338`.

Boundary: Current producer emits classFilename by uppercasing game-account class_name. Tests only for bounded published shape, canonical class fixtures and optionality; uppercase conversion alone is not full native token fidelity.

Work: **tests only**. Use the exact source claims below as acceptance assertions; do not substitute namespace/member presence or inspect generated code shape.

- L337: BNetGameAccountInfo (C_BattleNet.GetAccountInfoByGUID, C_BattleNet.GetAccountInfoByID, C_BattleNet.GetFriendAccountInfo, C_BattleNet.GetFriendGameAccountInfo, C_BattleNet.GetGameAccountInfoByGUID, C_BattleNet.GetGameAccountInfoByID)
- L338: + classFilename

Acceptance: concrete nonempty fixtures, non-default state transitions/readback, output arity/optionality and relevant rejection/taint/ownership boundaries; preserve explicitly bounded shape-only scope and no live-service claim. For layout/presentation rows, actual frame/region state is required; metadata round trips alone do not prove rendering.

Existing test entry points: `src/loader/tests/wow_api_globals/startup_globals.rs:915`.

Contract qualifier: INFERRED: deterministic Battle.net fixtures; native service delivery excluded.

### C21 — lfg-search-field-shape (2 rows; implemented-needs-proof)

Rows: `structures-LfgSearchResultData-351`, `structures-LfgSearchResultData-352`.

Boundary: Retail search serializer publishes censored=false. Prove bounded field shape from a nonempty listing and independent results; actual censorship transitions are not implemented or claimed.

Work: **tests only**. Use the exact source claims below as acceptance assertions; do not substitute namespace/member presence or inspect generated code shape.

- L351: LfgSearchResultData (C_LFGList.GetSearchResultInfo)
- L352: + censored

Acceptance: concrete nonempty fixtures, non-default state transitions/readback, output arity/optionality and relevant rejection/taint/ownership boundaries; preserve explicitly bounded shape-only scope and no live-service claim. For layout/presentation rows, actual frame/region state is required; metadata round trips alone do not prove rendering.

Existing test entry points: `src/loader/tests/wow_api_globals/patch_12_1_service_payloads.rs:529`.

### C22 — delve-tier-field-shape (1 rows; implemented-needs-proof)

Rows: `structures-TieredEntranceTierInfo-369`.

Boundary: Retail serializer emits overrideTooltipSpellID=nil and isLFG=false. Current declaration requires a non-nil overrideTooltipSpellID and uses queueAsLFG rather than isLFG; model/epoch-reconcile that delta instead of crediting plausible placeholders.

Work: **tests only**. Use the exact source claims below as acceptance assertions; do not substitute namespace/member presence or inspect generated code shape.

- L369: + isLFG

Acceptance: concrete nonempty fixtures, non-default state transitions/readback, output arity/optionality and relevant rejection/taint/ownership boundaries; preserve explicitly bounded shape-only scope and no live-service claim. For layout/presentation rows, actual frame/region state is required; metadata round trips alone do not prove rendering.

Existing test entry points: `src/loader/tests/wow_api_globals/startup_globals.rs:834`.

Contract qualifier: INFERRED: historical isLFG name and tooltip default until pinned 69587 declaration is available; current-cache queueAsLFG must not silently replace captured source.

### C23 — private-anchor-output-shape (5 rows; implemented-needs-proof)

Rows: `structures-UnitPrivateAuraAnchorInfo-370`, `structures-UnitPrivateAuraAnchorInfo-371`, `structures-UnitPrivateAuraAnchorInfo-372`, `structures-UnitPrivateAuraAnchorInfo-373`, `structures-UnitPrivateAuraAnchorInfo-374`.

Boundary: Anchor output serializer publishes current flags without showCountdownFrame; prove public/callback result isolation and true/false values. This is structure shape, not private-aura rendering parity.

Work: **tests only**. Use the exact source claims below as acceptance assertions; do not substitute namespace/member presence or inspect generated code shape.

- L370: UnitPrivateAuraAnchorInfo
- L371: - showCountdownFrame
- L372: + showDispelIcon
- L373: + showCooldownEdge
- L374: + showCooldownFrame

Acceptance: concrete nonempty fixtures, non-default state transitions/readback, output arity/optionality and relevant rejection/taint/ownership boundaries; preserve explicitly bounded shape-only scope and no live-service claim. For layout/presentation rows, actual frame/region state is required; metadata round trips alone do not prove rendering.

Existing test entry points: `tests/private_aura_anchors.rs:267`.

### C24 — deprecation (2 rows; implemented-needs-proof)

Rows: `deprecated api-getglobal-380`, `deprecated api-setglobal-381`.

Boundary: Audit current cached deprecated wrappers and default Retail strict-removal timing. A historical PTR symbol observation is not Retail migration acceptance.

Work: **tests only**. Use the exact source claims below as acceptance assertions; do not substitute namespace/member presence or inspect generated code shape.

- L380: getglobal
- L381: setglobal

Acceptance: concrete nonempty fixtures, non-default state transitions/readback, output arity/optionality and relevant rejection/taint/ownership boundaries; preserve explicitly bounded shape-only scope and no live-service claim. For layout/presentation rows, actual frame/region state is required; metadata round trips alone do not prove rendering.

Existing test entry points: `patch-tests/patch_12_1/strict_removal_timing.rs:96`.

### C25 — bnet-wrapper-proof (2 rows; implemented-needs-proof)

Rows: `deprecated api-BNSendVerifiedBattleTagInvite-385`, `deprecated api-BNGetFriendInviteInfo-386`.

Boundary: Current native Battle.net replacement functions record/query explicit local invite state; cached deprecated aliases forward to them. Tests only for both wrapper/backend routes, duplicates, absent invites, exact arguments and result shape; no network delivery claim.

Work: **tests only**. Use the exact source claims below as acceptance assertions; do not substitute namespace/member presence or inspect generated code shape.

- L385: BNSendVerifiedBattleTagInvite -> C_BattleNet.SendVerifiedBattleNetFriendInvite
- L386: BNGetFriendInviteInfo -> C_BattleNet.GetFriendInviteInfo

Acceptance: concrete nonempty fixtures, non-default state transitions/readback, output arity/optionality and relevant rejection/taint/ownership boundaries; preserve explicitly bounded shape-only scope and no live-service claim. For layout/presentation rows, actual frame/region state is required; metadata round trips alone do not prove rendering.

Existing test entry points: `src/loader/tests/wow_api_globals/startup_globals.rs:894`.

### C26 — raid-warning-wrapper-proof (2 rows; implemented-needs-proof)

Rows: `deprecated api-RaidNotice_AddMessage-395`, `deprecated api-RaidNotice_Clear-396`.

Boundary: Unmodified RaidWarningUtil/RaidWarningFrameMixin implement message addition and pool-backed clearing. Tests only: actual deprecated wrapper, message pool contents/visibility, per-type clearing and default Retail loading; text pixels and native timing are not claimed.

Work: **tests only**. Use the exact source claims below as acceptance assertions; do not substitute namespace/member presence or inspect generated code shape.

- L395: RaidNotice_AddMessage -> RaidWarningUtil.AddMessage
- L396: RaidNotice_Clear -> RaidWarningFrameMixin:ClearMessages

Acceptance: concrete nonempty fixtures, non-default state transitions/readback, output arity/optionality and relevant rejection/taint/ownership boundaries; preserve explicitly bounded shape-only scope and no live-service claim. For layout/presentation rows, actual frame/region state is required; metadata round trips alone do not prove rendering.

Existing test entry points: No exact test located; add targeted behavior tests for this slice.

## Modelable batches

Use current declarations where available; otherwise keep triage INFERRED qualifiers. Explicit state beats placeholders, and mock-provider tests do not establish native integration. Do not wait on blocked rows.

### C27 — svg (4 rows; modelable)

Rows: `prose-undated-010`, `prose-2026-06-18-053`, `prose-2026-06-18-054`, `prose-2026-08-04-237`.

Boundary: VectorGraphics construction/SVG metadata storage exists, but native code explicitly says SVG path rendering is not modeled. Implement 2D SVG asset decoding/rendering and legal method surface; not a 3D feature.

Work: **model the missing delta, then behavioral proof**. Use the exact source claims below as acceptance assertions; do not substitute namespace/member presence or inspect generated code shape.

- L10: SVG textures are now supported with the VectorGraphics object type.
- L53: We now support showing SVG textures in our UI. They can be used on regular textures (e.g. file="Path/To/Texture.svg") or with a new VectorGraphics object type, which renders them at higher quality.
- L54: Note that the VectorGraphics objects don't currently support all of the APIs on regular Textures (rotation, masking, tex coords, etc.)
- L237: Fixed a bug that was causing addons to not be able to use the new SVG tech.

Acceptance: concrete nonempty fixtures, non-default state transitions/readback, output arity/optionality and relevant rejection/taint/ownership boundaries; preserve explicitly bounded shape-only scope and no live-service claim. For layout/presentation rows, actual frame/region state is required; metadata round trips alone do not prove rendering.

Existing test entry points: `src/loader/tests/wow_api_globals/frames_and_attributes.rs:103`.

Contract qualifier: INFERRED: fixture SVG subset and unsupported-method behavior pending exact native declarations.

### C28 — access (3 rows; modelable)

Rows: `prose-undated-013`, `prose-2026-07-21-183`, `prose-2026-07-23-213`.

Boundary: HasAccessConstraints reads native flags, but CanBeAccessedInContext is registered only under client-wowforever. Retail needs its context-access query and aura restriction policy, not a copied Forever availability claim.

Work: **model the missing delta, then behavioral proof**. Use the exact source claims below as acceptance assertions; do not substitute namespace/member presence or inspect generated code shape.

- L13: CanAccessObject has been replaced with FrameScriptObject:CanBeAccessedInContext.
- L183: Added new APIs FrameScriptObject:HasAccessConstraints and FrameScriptObject:CanBeAccessedInContext to script objects.
- L213: Aura buttons now permit native script object API calls - such as SetPoint, SetSize - during UI (re)load, until execution of PLAYER_LOGIN.

Acceptance: concrete nonempty fixtures, non-default state transitions/readback, output arity/optionality and relevant rejection/taint/ownership boundaries; preserve explicitly bounded shape-only scope and no live-service claim. For layout/presentation rows, actual frame/region state is required; metadata round trips alone do not prove rendering.

Existing test entry points: `tests/forbidden_frames.rs:92`, `tests/forbidden_frames.rs:172`.

### C29 — deprecation (6 rows; modelable)

Rows: `prose-undated-014`, `prose-2026-07-07-118`, `prose-2026-07-07-148`, `prose-2026-07-07-151`, `deprecated api-C_DyeColor-GetDyeColorForItem-390`, `deprecated api-C_DyeColor-GetDyeColorForItemLocation-391`.

Boundary: Audit current cached deprecated wrappers and default Retail strict-removal timing. A historical PTR symbol observation is not Retail migration acceptance.

Work: **model the missing delta, then behavioral proof**. Use the exact source claims below as acceptance assertions; do not substitute namespace/member presence or inspect generated code shape.

- L14: Deprecated getglobal and setglobal.
- L118: Addons no longer create AuraButtons directly. The AddAuraFrame API has been removed.
- L148: SecureAuraHeaderTemplate has been removed from Mainline (it will still exist for Classic). Addons still using SecureAuraHeaderTemplate should migrate over to using AuraContainers.
- L151: Added a new SecureGroupHeaderTemplate xml template that can be used to safely create a single AuraContainer on UnitFrame creation.
- L390: C_DyeColor.GetDyeColorForItem -> C_DyeColor.GetDyeColorsForItem
- L391: C_DyeColor.GetDyeColorForItemLocation -> C_DyeColor.GetDyeColorsForItemLocation

Acceptance: concrete nonempty fixtures, non-default state transitions/readback, output arity/optionality and relevant rejection/taint/ownership boundaries; preserve explicitly bounded shape-only scope and no live-service claim. For layout/presentation rows, actual frame/region state is required; metadata round trips alone do not prove rendering.

Existing test entry points: `patch-tests/patch_12_1/strict_removal_timing.rs:96`.

### C30 — cvar (2 rows; modelable)

Rows: `prose-undated-015`, `prose-2026-07-21-179`.

Boundary: Implement the declared CVar/default/scope and policy rather than generic name acceptance. Current CVar defaults are a nearby producer; account-wide autoLoot and session-only tooltip setting need persistence boundaries.

Work: **model the missing delta, then behavioral proof**. Use the exact source claims below as acceptance assertions; do not substitute namespace/member presence or inspect generated code shape.

- L15: The Auto Loot setting (CVar autoLootDefault) is now account wide.
- L179: Added a new CVar tooltipShowAuraSpellIDsCVar: tooltipShowAuraSpellIDs (Game)Default: 0Show spell IDs in tooltips for unit auras. which causes spell IDs to show in aura tooltips. This cvar will not persist between sessions.

Acceptance: concrete nonempty fixtures, non-default state transitions/readback, output arity/optionality and relevant rejection/taint/ownership boundaries; preserve explicitly bounded shape-only scope and no live-service claim. For layout/presentation rows, actual frame/region state is required; metadata round trips alone do not prove rendering.

Existing test entry points: `tests/set_cvar_global.rs:7`.

Contract qualifier: INFERRED only for local account/session fixture ownership; stated default and persistence behavior remain explicit requirements.

### C31 — partition (3 rows; modelable)

Rows: `prose-2026-06-18-032`, `prose-2026-06-18-041`, `prose-2026-07-23-217`.

Boundary: Public/forbidden proxy tables and XML mixin routing exist. Retail addon-inaccessible partitions, handler ownership and native access restrictions still need a complete model; table partitioning alone is not isolation proof.

Work: **model the missing delta, then behavioral proof**. Use the exact source claims below as acceptance assertions; do not substitute namespace/member presence or inspect generated code shape.

- L32: Private Script Objects are a new construct that lets us split the Lua representation of a script object across multiple Lua tables, or partitions. One of these partitions we call the Forbidden Partition, because it is inaccessible to addons. The Forbidden Partition can contain any kind of value, from mixins to key/value pairs, functions, script handlers, and child objects. This allows us to effectively hide portions of the object from addon code even when the object itself isn’t in the secure environment.
- L41: To answer that, let’s go back to Private Script Objects and Forbidden Aspects again. Aura Buttons and Aura Containers both have Forbidden Aspects applied to them on creation. When an Aura Button is added to an Aura Container using the AddAuraFrame API, it is added to the Forbidden Partition of that Aura Container. This means addon code cannot install script handlers on Aura Buttons to be notified when they show or hide. It also cannot hook functions called on the Aura Button’s mixins or register events on those buttons. While addons can still hold references to those individual Aura Buttons, calling certain APIs on them will be disallowed, and they cannot run logic based on whether those buttons are shown, because IsShown and similar APIs return secrets.
- L217: Resolved an issue involving the CastingBarTypeInfo table which was causing taint issues in nameplates.

Acceptance: concrete nonempty fixtures, non-default state transitions/readback, output arity/optionality and relevant rejection/taint/ownership boundaries; preserve explicitly bounded shape-only scope and no live-service claim. For layout/presentation rows, actual frame/region state is required; metadata round trips alone do not prove rendering.

Existing test entry points: `tests/userdata_proxy.rs:37`, `tests/xml_secure_delegates.rs:7`.

### C32 — forbidden (6 rows; modelable)

Rows: `prose-2026-06-18-034`, `prose-2026-06-18-035`, `prose-2026-06-23-081`, `prose-2026-06-23-082`, `prose-2026-06-23-087`, `prose-2026-07-14-163`.

Boundary: Native aspect masks/inheritance exist, but mask publication does not establish UntrustedScriptExecution or UntrustedLayoutScriptExecution suppression across all callback/anchor paths. Model caller-sensitive dispatch before claiming security.

Work: **model the missing delta, then behavioral proof**. Use the exact source claims below as acceptance assertions; do not substitute namespace/member presence or inspect generated code shape.

- L34: Forbidden Aspects are another new construct that works alongside Private Script Objects. Forbidden Aspects are similar in concept to the Secret Aspects we introduced in Midnight, but instead of causing certain object APIs to return secrets, they prevent addons from using certain functionality entirely. Where Secret Aspects obfuscate data, Forbidden Aspects restrict what addons are allowed to do with an object.
- L35: There are several Forbidden Aspects being added (details are in the docs), but let’s use the UntrustedScriptExecution Forbidden Aspect as an example. When a frame has the UntrustedScriptExecution Forbidden Aspect applied to it, any script binding handlers set on it (e.g. OnShow, OnLoad, OnSizeChanged) will not be run unless that handler lives in the object’s Forbidden Partition and execution is untainted. In other words, addons cannot install their own script bindings on the object, but our code can.
- L81: UntrustedScriptExecution: When active, addon-installed script handlers on a frame and its children will never be run.
- L82: UntrustedLayoutScriptExecution: When active, addon-installed OnSizeChanged handlers will never be run for a frame, its children, or any frames anchored to either.
- L87: Aura Buttons have had the following Forbidden Aspects applied to them: UntrustedScriptExecution, UntrustedLayoutScriptExecution, AlwaysPropagateInput, ScriptedInput, and QueryFocus
- L163: AuraButtons are now forbidden (meaning APIs called on them via tainted code will Lua error) whenever auras are secret.

Acceptance: concrete nonempty fixtures, non-default state transitions/readback, output arity/optionality and relevant rejection/taint/ownership boundaries; preserve explicitly bounded shape-only scope and no live-service claim. For layout/presentation rows, actual frame/region state is required; metadata round trips alone do not prove rendering.

Existing test entry points: `tests/forbidden_aspect_creation.rs:14`.

### C33 — forbidden (4 rows; modelable)

Rows: `prose-2026-07-14-164`, `prose-2026-07-14-167`, `prose-2026-07-23-220`, `prose-2026-08-04-240`.

Boundary: Native aspect masks/inheritance exist, but mask publication does not establish UntrustedScriptExecution or UntrustedLayoutScriptExecution suppression across all callback/anchor paths. Model caller-sensitive dispatch before claiming security.

Work: **model the missing delta, then behavioral proof**. Use the exact source claims below as acceptance assertions; do not substitute namespace/member presence or inspect generated code shape.

- L164: This forbidden state is not applied until after the initializeFrame callback has been called, and AuraButtons return to a non-forbidden state when auras become non-secret again (outside combat, encounters, etc.).
- L167: Made improvements to the error messaging displayed when attempting to call forbidden script APIs on script objects.
- L220: Aura containers that are configured to show aura groups will no longer receive OnSizeChanged updates. Note that this restriction also applies to frames anchored to aura containers (but only after the aura container has an aura group added).
- L240: Resolved an exploit involving the use of OnSizeChanged to track the number of auras in an AuraContainer.

Acceptance: concrete nonempty fixtures, non-default state transitions/readback, output arity/optionality and relevant rejection/taint/ownership boundaries; preserve explicitly bounded shape-only scope and no live-service claim. For layout/presentation rows, actual frame/region state is required; metadata round trips alone do not prove rendering.

Existing test entry points: `tests/forbidden_aspect_creation.rs:14`.

### C34 — aura-secret (6 rows; modelable)

Rows: `prose-2026-06-18-043`, `prose-2026-06-30-097`, `prose-2026-06-30-098`, `prose-2026-06-30-099`, `prose-2026-06-30-100`, `prose-2026-07-07-147`.

Boundary: Existing aura queries/filtering do not supply complete Retail caller-sensitive secret vectors, restricted index/slot/instance access, fully secret AuraData and UNIT_AURA payloads. Model these outputs and error boundaries over explicit aura-secret context.

Work: **model the missing delta, then behavioral proof**. Use the exact source claims below as acceptance assertions; do not substitute namespace/member presence or inspect generated code shape.

- L43: The main change to existing APIs is that, when auras are secret (during combat, encounters, M+, and PvP matches), all of the UnitAura APIs will now either return full secrets or nil when called by addons. That means that APIs like GetUnitAuras and GetUnitAuraInstanceIDs will return a secret vector, meaning addon code will not be able to determine how many auras it contains or iterate through it for display. Auras we explicitly flag as non-secret will still be returned as non-secret by UnitAura APIs, however.
- L97: This week brings the majority of the UnitAura API restrictions (with a few small pieces still remaining). Broadly speaking you can consider APIs that return aura data are no longer safe for addon use while aura data is secret. More specifically:
- L98: C_UnitAura and C_TooltipInfo APIs that provide access to aura data via index, slot, or instance ID will Lua error when called by addons while auras are secret.
- L99: C_UnitAura APIs that provide access to aura data via spell ID or spell name can still be called by addons as before (non-secret spells still return non-secrets).
- L100: The UNIT_AURA event now delivers a fully secret payload while auras are secret. AuraData structs are now always fully secret.
- L147: The UNIT_AURA event now delivers a fully secret payload while auras are secret. AuraData structs are now always fully secret.

Acceptance: concrete nonempty fixtures, non-default state transitions/readback, output arity/optionality and relevant rejection/taint/ownership boundaries; preserve explicitly bounded shape-only scope and no live-service claim. For layout/presentation rows, actual frame/region state is required; metadata round trips alone do not prove rendering.

Existing test entry points: `tests/unit_aura_slot_secret_arguments.rs:189`, `tests/tooltip_aura_instance_security.rs:1`.

Contract qualifier: INFERRED only for host context injection; public policy comes from this page and current declarations, and may differ between historical PTR weeks.

### C35 — roleset (5 rows; modelable)

Rows: `prose-2026-06-18-060`, `prose-2026-06-18-061`, `prose-2026-07-21-189`, `prose-2026-07-21-190`, `prose-2026-07-21-191`.

Boundary: Retail ApplyRolesetFilters is a temporary no-op; GetActiveBlockedRolesets/GetActiveAllowedRolesets/IsRolesetFiltered have no source producer. Model allowed/blocked sets, alwaysBlocked and effective visibility from current declarations.

Work: **model the missing delta, then behavioral proof**. Use the exact source claims below as acceptance assertions; do not substitute namespace/member presence or inspect generated code shape.

- L60: A new system has been added called the Roleset System, which allows you to tag a frame as being part of a "roleset". You can then use the new C_Roleset.ApplyRolesetFilters to specify which rolesets are currently active.
- L61: Frames in an inactive roleset will never be shown, regardless of their shown state. See Blizzard_UIModeManager.lua for more details and examples.
- L189: Added new GetActiveBlockedRolesets and GetActiveAllowedRolesets APIs to C_Roleset, which return the list of currently blocked and allowed rolesets.
- L190: Added a new IsRolesetFiltered API on frames, which returns whether the frame is currently filtered by roleset.
- L191: Added a new "alwaysBlocked" roleset, which can be applied to frames to cause them to never show.

Acceptance: concrete nonempty fixtures, non-default state transitions/readback, output arity/optionality and relevant rejection/taint/ownership boundaries; preserve explicitly bounded shape-only scope and no live-service claim. For layout/presentation rows, actual frame/region state is required; metadata round trips alone do not prove rendering.

Existing test entry points: `tests/wowforever_rolesets.rs:5`.

Contract qualifier: INFERRED where Retail declarations omit precedence; Forever tests are related evidence, not Retail implementation proof.

### C36 — radial (1 rows; modelable)

Rows: `prose-2026-06-18-062`.

Boundary: Radial APIs currently store per-frame metadata; this is not rendered texture/statusbar radial masking. Add presentation and secret-aspect behavior with concrete radial fixtures.

Work: **model the missing delta, then behavioral proof**. Use the exact source claims below as acceptance assertions; do not substitute namespace/member presence or inspect generated code shape.

- L62: Radial masking support has been added to textures and status bars, allowing them to have a radial mask applied to them without the need for hacky uses of cooldowns. Example usage on a texture:

Acceptance: concrete nonempty fixtures, non-default state transitions/readback, output arity/optionality and relevant rejection/taint/ownership boundaries; preserve explicitly bounded shape-only scope and no live-service claim. For layout/presentation rows, actual frame/region state is required; metadata round trips alone do not prove rendering.

Existing test entry points: `src/loader/tests/wow_api_globals/frames_and_attributes.rs:157`.

### C37 — aura-button (6 rows; modelable)

Rows: `prose-2026-06-23-077`, `prose-2026-06-23-078`, `prose-2026-07-07-142`, `prose-2026-07-07-152`, `prose-2026-07-07-153`, `prose-2026-07-21-175`.

Boundary: Cached custom button Lua has presentation setters and aura update handlers; real native aura data, secured region ownership and presentation updates remain incomplete. Exercise actual templates, not table-only substitutes.

Work: **model the missing delta, then behavioral proof**. Use the exact source claims below as acceptance assertions; do not substitute namespace/member presence or inspect generated code shape.

- L77: Dispel borders: Using the SetAuraBorder(texture, [options]) API (see DefaultAuraBorderOptions for available options).
- L78: Dispel type text: Using the SetAuraSymbol(fontString, [options]) API (see DefaultAuraSymbolOptions for available options).
- L142: Added a new API SetCancelAuraButtons to AuraButtons that can be used to specify which mouse clicks to use to cancel. This can be called on AuraButtons via the initializeFrame callback.
- L152: Resolved an issue where the SetApplicationCount function on CustomAuraButtons would error if not supplied an options table.
- L153: Resolved an issue where ApplyAuraSymbol on CustomAuraButtons was consulting the wrong region (AuraButton) for dispel type validation.
- L175: Added new ApplicationBar APIs to custom aura buttons that allow addons to show a status bar that tracks their number of applications.

Acceptance: concrete nonempty fixtures, non-default state transitions/readback, output arity/optionality and relevant rejection/taint/ownership boundaries; preserve explicitly bounded shape-only scope and no live-service claim. For layout/presentation rows, actual frame/region state is required; metadata round trips alone do not prove rendering.

Existing test entry points: `tests/aura_container_util.rs:6`, `patch-tests/patch_12_1/aura_tooltip.rs:34`.

### C38 — aura-button (6 rows; modelable)

Rows: `prose-2026-07-21-177`, `prose-2026-07-21-184`, `prose-2026-07-23-211`, `prose-2026-07-23-215`, `prose-2026-07-23-219`, `prose-2026-08-04-233`.

Boundary: Cached custom button Lua has presentation setters and aura update handlers; real native aura data, secured region ownership and presentation updates remain incomplete. Exercise actual templates, not table-only substitutes.

Work: **model the missing delta, then behavioral proof**. Use the exact source claims below as acceptance assertions; do not substitute namespace/member presence or inspect generated code shape.

- L177: Added support for color curves and color maps to the AuraButton:SetAuraBorder API (see CustomAuraButtonDispelTypeTextureOptions in the documentation files for details).
- L184: Fixed a bug that was causing the cooldown swipe to show incorrectly in some cases.
- L211: Added support for showing multiple dispel textures on an aura button.
- L215: Resolved an issue that was causing addons to not be able to call aura button APIs outside of the initializeFrame callback.
- L219: Child components of aura buttons can no longer be re-parented once configured.
- L233: Added stealable and showAlways options for AuraButton borders.

Acceptance: concrete nonempty fixtures, non-default state transitions/readback, output arity/optionality and relevant rejection/taint/ownership boundaries; preserve explicitly bounded shape-only scope and no live-service claim. For layout/presentation rows, actual frame/region state is required; metadata round trips alone do not prove rendering.

Existing test entry points: `tests/aura_container_util.rs:6`, `patch-tests/patch_12_1/aura_tooltip.rs:34`.

### C39 — aura-button (1 rows; modelable)

Rows: `prose-2026-08-04-238`.

Boundary: Cached custom button Lua has presentation setters and aura update handlers; real native aura data, secured region ownership and presentation updates remain incomplete. Exercise actual templates, not table-only substitutes.

Work: **model the missing delta, then behavioral proof**. Use the exact source claims below as acceptance assertions; do not substitute namespace/member presence or inspect generated code shape.

- L238: Fixed a bug that could sometimes cause the duration text on AuraButtons to incorrectly show as 0.

Acceptance: concrete nonempty fixtures, non-default state transitions/readback, output arity/optionality and relevant rejection/taint/ownership boundaries; preserve explicitly bounded shape-only scope and no live-service claim. For layout/presentation rows, actual frame/region state is required; metadata round trips alone do not prove rendering.

Existing test entry points: `tests/aura_container_util.rs:6`, `patch-tests/patch_12_1/aura_tooltip.rs:34`.

### C40 — tooltip (6 rows; modelable)

Rows: `prose-2026-06-23-079`, `prose-2026-07-23-208`, `prose-2026-07-23-209`, `prose-2026-07-23-212`, `prose-2026-07-23-216`, `prose-2026-08-04-234`.

Boundary: Cached aura-button tooltip lifecycle exists, including options and throttling. Need native managed button identity, visibility and restricted-aura integration; older AuraButtonMixin mock tooltip tests are not this new contract.

Work: **model the missing delta, then behavioral proof**. Use the exact source claims below as acceptance assertions; do not substitute namespace/member presence or inspect generated code shape.

- L79: Aura tooltips: Automatically enabled but can be disabled via the SetMouseMotionEnabled API.
- L208: Added support for adjusting aura button tooltip anchors.
- L209: Added support for hiding aura button tooltips while in combat.
- L212: Added new APIs to configure custom nineslice, backdrop, or background texture slice assets to use for all aura button tooltips. Note that these are global APIs that apply to all aura buttons (not to individual aura containers).
- L216: Resolved an issue where a Lua error would occur when toggling visibility of an aura container while the mouse was over a visible aura button.
- L234: AuraButton tooltips are now throttled to update once every 200ms instead of every frame.

Acceptance: concrete nonempty fixtures, non-default state transitions/readback, output arity/optionality and relevant rejection/taint/ownership boundaries; preserve explicitly bounded shape-only scope and no live-service claim. For layout/presentation rows, actual frame/region state is required; metadata round trips alone do not prove rendering.

Existing test entry points: `patch-tests/patch_12_1/aura_tooltip.rs:34`.

### C41 — input (3 rows; modelable)

Rows: `prose-2026-06-23-084`, `prose-2026-06-23-085`, `prose-2026-06-23-086`.

Boundary: Click/focus guards and keyboard propagation exist; reconcile the complete mouse+keyboard contract, child inheritance and secure callers rather than treating isolated probes as all-input proof.

Work: **model the missing delta, then behavioral proof**. Use the exact source claims below as acceptance assertions; do not substitute namespace/member presence or inspect generated code shape.

- L84: AlwaysPropagateInput: When active, a frame and its children will always propagate mouse and keyboard input.
- L85: ScriptedInput: When active, addons are not allowed to call input-related APIs (Click, SetFocus, etc.) on a frame or its children.
- L86: QueryFocus: When active, addons cannot query if a frame or its children are the current mouse or keyboard focus.

Acceptance: concrete nonempty fixtures, non-default state transitions/readback, output arity/optionality and relevant rejection/taint/ownership boundaries; preserve explicitly bounded shape-only scope and no live-service claim. For layout/presentation rows, actual frame/region state is required; metadata round trips alone do not prove rendering.

Existing test entry points: `tests/forbidden_aspect_creation.rs:356`, `tests/keyboard.rs:227`.

### C42 — editfocus (1 rows; modelable)

Rows: `prose-2026-06-23-089`.

Boundary: EditBox focus APIs exist, but visibility-driven auto-focus must respect the Shown secret aspect, including inherited state.

Work: **model the missing delta, then behavioral proof**. Use the exact source claims below as acceptance assertions; do not substitute namespace/member presence or inspect generated code shape.

- L89: Editboxes will no longer auto-focus if they become visible while they have the Shown secret aspect applied.

Acceptance: concrete nonempty fixtures, non-default state transitions/readback, output arity/optionality and relevant rejection/taint/ownership boundaries; preserve explicitly bounded shape-only scope and no live-service claim. For layout/presentation rows, actual frame/region state is required; metadata round trips alone do not prove rendering.

Existing test entry points: `tests/forbidden_aspect_creation.rs:379`.

### C43 — aura-group (6 rows; modelable)

Rows: `prose-2026-06-30-101`, `prose-2026-06-30-102`, `prose-2026-06-30-107`, `prose-2026-06-30-108`, `prose-2026-06-30-109`, `prose-2026-07-07-117`.

Boundary: Current unmodified cached Lua has managed group/slot/filter/layout logic. Native filter/secrecy/lifecycle integration is incomplete; existing mock-provider tests are narrower than real addon creation and managed frame updates.

Work: **model the missing delta, then behavioral proof**. Use the exact source claims below as acceptance assertions; do not substitute namespace/member presence or inspect generated code shape.

- L101: Added a new ManagedAuraContainer base type, which fully manages the display and layout of AuraButtons.
- L102: The Blizzard Target Frame now uses a ManagedAuraContainer for the display of its auras.
- L107: CustomAuraContainers will be converted to ManagedAuraContainers. As a result, AuraContainers will now handle the creation of AuraButtons entirely on their own.
- L108: AuraContainer support for filtering by Spell ID, dispel type, stealable, and max duration. Some filters will have restrictions - more details to come.
- L109: AuraContainer support for sorting (both sort rule and direction).
- L117: AuraContainers now handle the creation and anchoring of all AuraButtons inside of them entirely on their own.

Acceptance: concrete nonempty fixtures, non-default state transitions/readback, output arity/optionality and relevant rejection/taint/ownership boundaries; preserve explicitly bounded shape-only scope and no live-service claim. For layout/presentation rows, actual frame/region state is required; metadata round trips alone do not prove rendering.

Existing test entry points: `patch-tests/patch_12_1/aura_container.rs:67`, `tests/on_update_modes.rs:152`.

### C44 — aura-group (6 rows; modelable)

Rows: `prose-2026-07-07-130`, `prose-2026-07-07-131`, `prose-2026-07-07-132`, `prose-2026-07-07-133`, `prose-2026-07-07-134`, `prose-2026-07-07-137`.

Boundary: Current unmodified cached Lua has managed group/slot/filter/layout logic. Native filter/secrecy/lifecycle integration is incomplete; existing mock-provider tests are narrower than real addon creation and managed frame updates.

Work: **model the missing delta, then behavioral proof**. Use the exact source claims below as acceptance assertions; do not substitute namespace/member presence or inspect generated code shape.

- L130: candidateFilters: A table of additional filter information to apply when determining if an aura should be displayed. See ValidateCandidateFilters for the full list of options, but some examples are:
- L131: Include/exclude maps for spell IDs and dispel types.
- L132: maxDuration
- L133: Various boolean values from AuraData (isFromPlayerOrPlayerPet, isRoleAura, isPriorityAura, isStealable, etc.).
- L134: AuraGroups create and anchor AuraButtons in batches of 10 as needed.
- L137: AuraContainers now treat private auras just like regular auras, allowing them to be shown and sorted normally.

Acceptance: concrete nonempty fixtures, non-default state transitions/readback, output arity/optionality and relevant rejection/taint/ownership boundaries; preserve explicitly bounded shape-only scope and no live-service claim. For layout/presentation rows, actual frame/region state is required; metadata round trips alone do not prove rendering.

Existing test entry points: `patch-tests/patch_12_1/aura_container.rs:67`, `tests/on_update_modes.rs:152`.

### C45 — aura-group (6 rows; modelable)

Rows: `prose-2026-07-07-138`, `prose-2026-07-07-140`, `prose-2026-07-21-174`, `prose-2026-07-21-176`, `prose-2026-07-21-182`, `prose-2026-07-21-186`.

Boundary: Current unmodified cached Lua has managed group/slot/filter/layout logic. Native filter/secrecy/lifecycle integration is incomplete; existing mock-provider tests are narrower than real addon creation and managed frame updates.

Work: **model the missing delta, then behavioral proof**. Use the exact source claims below as acceptance assertions; do not substitute namespace/member presence or inspect generated code shape.

- L138: Added a new construct to AuraContainers: AuraSlots. You can think of AuraSlots as AuraGroups with maxFrameCount set to 1 (they will only ever show a single aura).
- L140: The AddAuraSlot(slotKey, filterString, options) API is used to add AuraSlots, and it supports most of the same options AddAuraGroup does.
- L174: Added a new GetAuraGroupFrame API to aura containers, which can be used to retrieve a child aura group frame by index.
- L176: Added a new SetAuraGroupFilterString API to aura containers, which allows addons to set the filter string after creation.
- L182: Auras flagged as non-secret can now be filtered using excludeSpellIDs and includeSpellIDs without restrictions on any unit.
- L186: Addons are no longer allowed to reparent aura buttons.

Acceptance: concrete nonempty fixtures, non-default state transitions/readback, output arity/optionality and relevant rejection/taint/ownership boundaries; preserve explicitly bounded shape-only scope and no live-service claim. For layout/presentation rows, actual frame/region state is required; metadata round trips alone do not prove rendering.

Existing test entry points: `patch-tests/patch_12_1/aura_container.rs:67`, `tests/on_update_modes.rs:152`.

### C46 — aura-group (3 rows; modelable)

Rows: `prose-2026-07-21-188`, `prose-2026-07-23-210`, `prose-2026-08-04-235`.

Boundary: Current unmodified cached Lua has managed group/slot/filter/layout logic. Native filter/secrecy/lifecycle integration is incomplete; existing mock-provider tests are narrower than real addon creation and managed frame updates.

Work: **model the missing delta, then behavioral proof**. Use the exact source claims below as acceptance assertions; do not substitute namespace/member presence or inspect generated code shape.

- L188: Boolean candidate filters now support negation by setting the boolean as false (e.g. isStealable = false). Leaving the option as nil will continue to mean "ignore".
- L210: Added a new aura instance ID-only sort method for aura containers, which sorts the auras by aura instance ID.
- L235: When an AuraContainer is disabled, all AuraButtons and ItemEnchantments belonging to it will now be cleared.

Acceptance: concrete nonempty fixtures, non-default state transitions/readback, output arity/optionality and relevant rejection/taint/ownership boundaries; preserve explicitly bounded shape-only scope and no live-service claim. For layout/presentation rows, actual frame/region state is required; metadata round trips alone do not prove rendering.

Existing test entry points: `patch-tests/patch_12_1/aura_container.rs:67`, `tests/on_update_modes.rs:152`.

### C47 — chat (1 rows; modelable)

Rows: `prose-2026-06-30-103`.

Boundary: Current Blizzard chat filter path needs a concrete 19-argument input/output probe; original 14-argument bug cannot be discharged from a callable symbol.

Work: **model the missing delta, then behavioral proof**. Use the exact source claims below as acceptance assertions; do not substitute namespace/member presence or inspect generated code shape.

- L103: Fixed a bug where only 14 of the 19 parameters were being passed to ChatFrame message event filter functions.

Acceptance: concrete nonempty fixtures, non-default state transitions/readback, output arity/optionality and relevant rejection/taint/ownership boundaries; preserve explicitly bounded shape-only scope and no live-service claim. For layout/presentation rows, actual frame/region state is required; metadata round trips alone do not prove rendering.

Existing test entry points: No exact test located; add targeted behavior tests for this slice.

Contract qualifier: INFERRED: local synthetic event fixture only; source fixes argument count to 19.

### C48 — aura-layout (6 rows; modelable)

Rows: `prose-2026-07-07-121`, `prose-2026-07-07-135`, `prose-2026-07-07-136`, `prose-2026-07-07-139`, `prose-2026-07-21-178`, `prose-2026-07-23-206`.

Boundary: Cached Lua implements group flow layout and resize. Native forbidden layout notifications and real managed-frame resize/anchor integration require modeling and behavioral proof.

Work: **model the missing delta, then behavioral proof**. Use the exact source claims below as acceptance assertions; do not substitute namespace/member presence or inspect generated code shape.

- L121: Auras from each group are anchored sequentially in the order the groups were added.
- L135: Anchoring of AuraButtons created by an AuraContainer can be adjusted via the SetAuraGroupLayout API (see ValidateAuraGroupLayoutOptions for available options).
- L136: AuraContainers now automatically resize to fit group-based AuraButtons inside of them.
- L139: Unlike AuraGroups, addons can manually anchor AuraSlots.
- L178: Addons can now specify custom ordering for the aura groups within an aura container, using the new layoutIndex option in the layout options table.
- L206: Added support for laying out aura groups in columns.

Acceptance: concrete nonempty fixtures, non-default state transitions/readback, output arity/optionality and relevant rejection/taint/ownership boundaries; preserve explicitly bounded shape-only scope and no live-service claim. For layout/presentation rows, actual frame/region state is required; metadata round trips alone do not prove rendering.

Existing test entry points: `tests/on_update_modes.rs:152`.

### C49 — aura-filter (5 rows; modelable)

Rows: `prose-2026-07-07-124`, `prose-2026-07-07-146`, `prose-2026-07-14-160`, `prose-2026-07-14-161`, `prose-2026-07-14-162`.

Boundary: Current native filter parser handles polarity and PLAYER only. Add negation, DISPELLABLE/IMPORTANT and raid-dispel eligibility using explicit aura/raid state; do not return plausible defaults.

Work: **model the missing delta, then behavioral proof**. Use the exact source claims below as acceptance assertions; do not substitute namespace/member presence or inspect generated code shape.

- L124: The filterString param is a standard aura filter string as used today (e.g. "HELPFUL|RAID").
- L146: Added support for negating most aura filters using the ! character. So for instance !PLAYER includes only auras NOT cast by the player.
- L160: Added a new aura filter, DISPELLABLE, which returns auras that have a dispel type of any kind, regardless of whether anyone in the player's raid can dispel it.
- L161: Added back the IMPORTANT aura filter now that it is no longer abusable.
- L162: The RAID_PLAYER_DISPELLABLE aura filter now also returns helpful auras on enemies that are dispellable/stealable by a raid member.

Acceptance: concrete nonempty fixtures, non-default state transitions/readback, output arity/optionality and relevant rejection/taint/ownership boundaries; preserve explicitly bounded shape-only scope and no live-service claim. For layout/presentation rows, actual frame/region state is required; metadata round trips alone do not prove rendering.

Existing test entry points: `tests/unit_aura_filter_query.rs:148`.

Contract qualifier: INFERRED: ordering of combined unknown/negated filters and synthetic raid-dispel fixtures; preserve declared token names.

### C50 — weapon (2 rows; modelable)

Rows: `prose-2026-07-07-141`, `prose-2026-07-21-187`.

Boundary: Cached AddItemEnchantment and cancel handling exist; connect explicit temporary weapon-enchant state to managed frames and click routing.

Work: **model the missing delta, then behavioral proof**. Use the exact source claims below as acceptance assertions; do not substitute namespace/member presence or inspect generated code shape.

- L141: Added a new API, AddItemEnchantment(itemEnchantmentSlot, options), to AuraContainers, which allows them to show temporary weapon enchants. See ValidateAddItemEnchantmentOptions for the list of options supported.
- L187: Temporary Weapon Enchants now support click-to-cancel.

Acceptance: concrete nonempty fixtures, non-default state transitions/readback, output arity/optionality and relevant rejection/taint/ownership boundaries; preserve explicitly bounded shape-only scope and no live-service claim. For layout/presentation rows, actual frame/region state is required; metadata round trips alone do not prove rendering.

Existing test entry points: `tests/on_update_modes.rs:152`.

Contract qualifier: INFERRED: deterministic enchant fixtures; no native cancellation/service timing claim.

### C51 — sounds (2 rows; modelable)

Rows: `prose-2026-07-07-145`, `prose-2026-07-21-180`.

Boundary: Existing private aura sound model and AddAuraSound entry point are relevant, but all-aura/application/removal triggers and successive renamed aliases need their own delta model; inherited add-context capability alone cannot close this row.

Work: **model the missing delta, then behavioral proof**. Use the exact source claims below as acceptance assertions; do not substitute namespace/member presence or inspect generated code shape.

- L145: The AddPrivateAuraAppliedSound and RemovePrivateAuraAppliedSound APIs have been renamed to AddAuraAppliedSound and RemoveAuraAppliedSound, and now work on any auras (not just private auras).
- L180: The AddAuraAppliedSound API has been renamed AddAuraSound and now supports specifying whether the sound should play when an aura is first added, gains an application or removed.

Acceptance: concrete nonempty fixtures, non-default state transitions/readback, output arity/optionality and relevant rejection/taint/ownership boundaries; preserve explicitly bounded shape-only scope and no live-service claim. For layout/presentation rows, actual frame/region state is required; metadata round trips alone do not prove rendering.

Existing test entry points: `tests/private_aura_sound_add_context.rs:51`, `tests/private_aura_sound_removal.rs:154`.

### C52 — worldframe (1 rows; modelable)

Rows: `prose-2026-07-14-166`.

Boundary: WorldFrame has an engine-created object; generic CreateFrame path still needs the addon creation rejection contract rather than assuming the existing root disallows duplicates.

Work: **model the missing delta, then behavioral proof**. Use the exact source claims below as acceptance assertions; do not substitute namespace/member presence or inspect generated code shape.

- L166: It is no longer possible for addons to create new WorldFrame instances via CreateFrame (preventing a crash).

Acceptance: concrete nonempty fixtures, non-default state transitions/readback, output arity/optionality and relevant rejection/taint/ownership boundaries; preserve explicitly bounded shape-only scope and no live-service claim. For layout/presentation rows, actual frame/region state is required; metadata round trips alone do not prove rendering.

Existing test entry points: `src/loader/tests/global_frame_access.rs:372`.

Contract qualifier: INFERRED: error wording/return shape; rejection requirement is explicit.

### C53 — secret-format (1 rows; modelable)

Rows: `prose-2026-07-21-185`.

Boundary: Current formatter/secret infrastructure is adjacent, but the secret argument causing unrelated object secrecy needs a behavioral input/output and object-state model; no broad FormatNumber safety credit.

Work: **model the missing delta, then behavioral proof**. Use the exact source claims below as acceptance assertions; do not substitute namespace/member presence or inspect generated code shape.

- L185: Fixed a bug where calling some APIs (like FormatNumber) with secrets would cause objects to be marked as secret incorrectly (and result in Lua Errors).

Acceptance: concrete nonempty fixtures, non-default state transitions/readback, output arity/optionality and relevant rejection/taint/ownership boundaries; preserve explicitly bounded shape-only scope and no live-service claim. For layout/presentation rows, actual frame/region state is required; metadata round trips alone do not prove rendering.

Existing test entry points: `tests/secret_value_security.rs:8`.

### C54 — unit-identity (4 rows; modelable)

Rows: `prose-2026-07-21-194`, `prose-2026-07-21-195`, `prose-2026-07-23-221`, `prose-2026-07-23-222`.

Boundary: Current UnitClass/related getters return ordinary values; implement secret results based on unit identity across every API named by the source. Existing UnitName policy does not cover these added contracts.

Work: **model the missing delta, then behavioral proof**. Use the exact source claims below as acceptance assertions; do not substitute namespace/member presence or inspect generated code shape.

- L194: A number of Unit APIs are being changed to return secret values when the unit's identity is secret. This is being done to prevent various methods of combining these API calls to compare secret units to each other in combat.
- L195: APIs affected: UnitClass, UnitClassBase, UnitIsOwnerOrControllerOfUnit, UnitSex, UnitSexBase, UnitPhaseReason, UnitGroupRolesAssigned, UnitGroupRolesAssignedEnum, UnitIsRaidOfficer, UnitInRaid, UnitIsPVP, UnitRace, UnitIsGroupLeader, UnitIsGroupAssistant, UnitLeadsAnyGroup, UnitGetAvailableRoles, GetInspectSpecialization.
- L221: A number of Unit APIs have been changed to return secret values when the unit's identity is secret. This is being done to prevent various methods of combining these API calls to compare secret units to each other in combat.
- L222: APIs affected: UnitClass, UnitClassBase, UnitIsOwnerOrControllerOfUnit, UnitSex, UnitSexBase, UnitPhaseReason, UnitGroupRolesAssigned, UnitGroupRolesAssignedEnum, UnitIsRaidOfficer, UnitInRaid, UnitIsPVP, UnitRace, UnitIsGroupLeader, UnitIsGroupAssistant, UnitLeadsAnyGroup, UnitGetAvailableRoles, GetInspectSpecialization

Acceptance: concrete nonempty fixtures, non-default state transitions/readback, output arity/optionality and relevant rejection/taint/ownership boundaries; preserve explicitly bounded shape-only scope and no live-service claim. For layout/presentation rows, actual frame/region state is required; metadata round trips alone do not prove rendering.

Existing test entry points: `tests/unit_name_secret_tokens.rs:22`.

Contract qualifier: INFERRED: explicit host unit-identity context fixture; do not infer complete security from one protected token.

### C55 — guild (2 rows; modelable)

Rows: `prose-2026-07-21-196`, `prose-2026-07-23-224`.

Boundary: GetGuildInfo has state-backed producers but the compound-token ban needs explicit authentication/validation before query results.

Work: **model the missing delta, then behavioral proof**. Use the exact source claims below as acceptance assertions; do not substitute namespace/member presence or inspect generated code shape.

- L196: The GetGuildInfo API is being changed to no longer accept compound unit tokens.
- L224: The GetGuildInfo API no longer accepts compound unit tokens.

Acceptance: concrete nonempty fixtures, non-default state transitions/readback, output arity/optionality and relevant rejection/taint/ownership boundaries; preserve explicitly bounded shape-only scope and no live-service claim. For layout/presentation rows, actual frame/region state is required; metadata round trips alone do not prove rendering.

Existing test entry points: No exact test located; add targeted behavior tests for this slice.

Contract qualifier: INFERRED: ban failure shape if declaration does not state it; do not silently accept compound tokens.

### C56 — possession (3 rows; modelable)

Rows: `prose-2026-07-21-197`, `prose-2026-07-23-223`, `prose-2026-08-04-236`.

Boundary: UnitIsPossessed currently returns false and UnitIsCharmed is a compatibility stub. Model charm/possession state plus aura secrecy and player/pet/vehicle exceptions.

Work: **model the missing delta, then behavioral proof**. Use the exact source claims below as acceptance assertions; do not substitute namespace/member presence or inspect generated code shape.

- L197: The following APIs are being changed to return secret values when auras are secret: UnitIsCharmed, UnitIsPossessed.
- L223: The following APIs now return secret values when auras are secret: UnitIsCharmed, UnitIsPossessed.
- L236: The UnitIsPossessed and UnitIsCharmed APIs no longer return secret values if the unit token passed is "player", "pet", or "vehicle".

Acceptance: concrete nonempty fixtures, non-default state transitions/readback, output arity/optionality and relevant rejection/taint/ownership boundaries; preserve explicitly bounded shape-only scope and no live-service claim. For layout/presentation rows, actual frame/region state is required; metadata round trips alone do not prove rendering.

Existing test entry points: `tests/unit_relationship_defaults.rs:4`.

Contract qualifier: INFERRED: deterministic possession/charm state fixtures; page explicitly specifies exemptions.

### C57 — resize (1 rows; modelable)

Rows: `prose-2026-07-23-214`.

Boundary: ResizeToBoundsRect is declared in cached SimpleFrame documentation but has no Rust/Lua simulator producer. Model resize from child bounds and preserve observable anchor/size semantics.

Work: **model the missing delta, then behavioral proof**. Use the exact source claims below as acceptance assertions; do not substitute namespace/member presence or inspect generated code shape.

- L214: Added a new addon-safe API, ResizeToBoundsRect, which can be used to resize a frame to match the bounds of its children.

Acceptance: concrete nonempty fixtures, non-default state transitions/readback, output arity/optionality and relevant rejection/taint/ownership boundaries; preserve explicitly bounded shape-only scope and no live-service claim. For layout/presentation rows, actual frame/region state is required; metadata round trips alone do not prove rendering.

Existing test entry points: No exact test located; add targeted behavior tests for this slice.

Contract qualifier: INFERRED: exact child visibility inclusion/empty bounds behavior unless current declaration or sibling GetBoundsRect clarifies it.

### C58 — ping (2 rows; modelable)

Rows: `prose-2026-07-23-218`, `prose-2026-08-04-239`.

Boundary: Unmodified cached PingableUnitFrameTemplate and C_PingSecure require a real addon-unit fixture proving taint and enemy ping behavior, not a namespace-presence test.

Work: **model the missing delta, then behavioral proof**. Use the exact source claims below as acceptance assertions; do not substitute namespace/member presence or inspect generated code shape.

- L218: Resolved an issue causing Lua errors when addons used PingableUnitFrameTemplate.
- L239: Fixed a bug that could cause Lua errors when pinging enemy units represented by addons using PingableUnitFrameTemplate.

Acceptance: concrete nonempty fixtures, non-default state transitions/readback, output arity/optionality and relevant rejection/taint/ownership boundaries; preserve explicitly bounded shape-only scope and no live-service claim. For layout/presentation rows, actual frame/region state is required; metadata round trips alone do not prove rendering.

Existing test entry points: No exact test located; add targeted behavior tests for this slice.

Contract qualifier: INFERRED: synthetic ping result/target fixtures; no live service claim.

### C59 — unitname (1 rows; modelable)

Rows: `prose-2026-07-23-225`.

Boundary: Existing 12.0.5 unit-name-secret-tokens capability protects PvP tokens; the new no-secrets-in-active-PvP delta cannot inherit that opposite policy. Add explicit active-match handling.

Work: **model the missing delta, then behavioral proof**. Use the exact source claims below as acceptance assertions; do not substitute namespace/member presence or inspect generated code shape.

- L225: The UnitName API will no longer return secrets while in an active PvP match.

Acceptance: concrete nonempty fixtures, non-default state transitions/readback, output arity/optionality and relevant rejection/taint/ownership boundaries; preserve explicitly bounded shape-only scope and no live-service claim. For layout/presentation rows, actual frame/region state is required; metadata round trips alone do not prove rendering.

Existing test entry points: `tests/unit_name_secret_tokens.rs:22`.

### C60 — pandemic (1 rows; modelable)

Rows: `prose-2026-08-04-232`.

Boundary: Cached custom AuraButton Lua has pandemic regions and window updates; wire duration/refresh state and real textures before claiming the feature.

Work: **model the missing delta, then behavioral proof**. Use the exact source claims below as acceptance assertions; do not substitute namespace/member presence or inspect generated code shape.

- L232: Added APIs to AuraButton that allow addons to show pandemic state via a texture.

Acceptance: concrete nonempty fixtures, non-default state transitions/readback, output arity/optionality and relevant rejection/taint/ownership boundaries; preserve explicitly bounded shape-only scope and no live-service claim. For layout/presentation rows, actual frame/region state is required; metadata round trips alone do not prove rendering.

Existing test entry points: `tests/aura_refresh_duration.rs:94`.

### C61 — enum-repair (6 rows; modelable)

Rows: `enumerations-Enum-ClubStreamType-268`, `enumerations-Enum-ClubStreamType-269`, `enumerations-Enum-EditModeUnitFrameSetting-284`, `enumerations-Enum-EditModeUnitFrameSetting-285`, `enumerations-Enum-EditModeUnitFrameSetting-286`, `enumerations-Enum-EditModeUnitFrameSetting-287`.

Boundary: Use exact declarations to model missing/reordered values and retire obsolete enum members; preserve epoch qualification and update metadata. Do not infer values by maximum-plus-one.

Work: **model the missing delta, then behavioral proof**. Use the exact source claims below as acceptance assertions; do not substitute namespace/member presence or inspect generated code shape.

- L268: Enum.ClubStreamType (C_Club.GetStreamInfo, C_Club.GetStreams)
- L269: + Discord
- L284: Enum.EditModeUnitFrameSetting
- L285: - IconSize
- L286: + BuffIconSize
- L287: + DebuffIconSize

Acceptance: concrete nonempty fixtures, non-default state transitions/readback, output arity/optionality and relevant rejection/taint/ownership boundaries; preserve explicitly bounded shape-only scope and no live-service claim. For layout/presentation rows, actual frame/region state is required; metadata round trips alone do not prove rendering.

Existing test entry points: No exact test located; add targeted behavior tests for this slice.

Contract qualifier: INFERRED: historical 69587 values if current docs postdate that epoch; exact current-cache numeric values are recorded per row.

### C62 — enum-repair (6 rows; modelable)

Rows: `enumerations-Enum-FragmentID-288`, `enumerations-Enum-FragmentID-289`, `enumerations-Enum-FragmentID-290`, `enumerations-Enum-FrameTutorialAccount-291`, `enumerations-Enum-FrameTutorialAccount-292`, `enumerations-Enum-HouseFinderSuggestionReason-293`.

Boundary: Use exact declarations to model missing/reordered values and retire obsolete enum members; preserve epoch qualification and update metadata. Do not infer values by maximum-plus-one.

Work: **model the missing delta, then behavioral proof**. Use the exact source claims below as acceptance assertions; do not substitute namespace/member presence or inspect generated code shape.

- L288: Enum.FragmentID
- L289: + FMapObject
- L290: + FWorldStateListenerData
- L291: Enum.FrameTutorialAccount
- L292: + HousingPetBeds
- L293: Enum.HouseFinderSuggestionReason (C_HousingNeighborhood.GetCornerstoneNeighborhoodInfo, B_NET_NEIGHBORHOOD_LIST_UPDATED, NEIGHBORHOOD_INFO_UPDATED, NEIGHBORHOOD_LIST_UPDATED, OPEN_NEIGHBORHOOD_CHARTER, OPEN_NEIGHBORHOOD_CHARTER_SIGNATURE_REQUEST, UPDATE_BULLETIN_BOARD_ROSTER)

Acceptance: concrete nonempty fixtures, non-default state transitions/readback, output arity/optionality and relevant rejection/taint/ownership boundaries; preserve explicitly bounded shape-only scope and no live-service claim. For layout/presentation rows, actual frame/region state is required; metadata round trips alone do not prove rendering.

Existing test entry points: `tests/patch_12_0_5_enum_additions.rs:94`.

Contract qualifier: INFERRED: historical 69587 values if current docs postdate that epoch; exact current-cache numeric values are recorded per row.

### C63 — enum-repair (5 rows; modelable)

Rows: `enumerations-Enum-HouseFinderSuggestionReason-294`, `enumerations-Enum-HousingResult-295`, `enumerations-Enum-HousingResult-304`, `enumerations-Enum-SecretAspect-318`, `enumerations-Enum-SecretAspect-319`.

Boundary: Use exact declarations to model missing/reordered values and retire obsolete enum members; preserve epoch qualification and update metadata. Do not infer values by maximum-plus-one.

Work: **model the missing delta, then behavioral proof**. Use the exact source claims below as acceptance assertions; do not substitute namespace/member presence or inspect generated code shape.

- L294: + Relinquished
- L295: Enum.HousingResult (C_HouseEditor.ActivateHouseEditorMode, C_HouseEditor.EnterHouseEditor, C_HouseEditor.GetHouseEditorAvailability, C_HouseEditor.GetHouseEditorModeAvailability, B_NET_NEIGHBORHOOD_LIST_UPDATED, CREATE_NEIGHBORHOOD_RESULT, HOUSE_EDITOR_MODE_CHANGE_FAILURE, HOUSE_EXTERIOR_POSITION_FAILURE, HOUSE_RESERVATION_RESPONSE_RECIEVED, HOUSE_RESET_FAILED, HOUSING_BLUEPRINT_COLLECTION_FAILURE, HOUSING_BLUEPRINT_CONTENTS_FAILURE, HOUSING_BLUEPRINT_DELETE_FAILURE, HOUSING_BLUEPRINT_EXPORT_FAILURE, HOUSING_BLUEPRINT_IMPORT_FAILURE, HOUSING_BLUEPRINT_RENAME_FAILURE, HOUSING_DECOR_DYE_FAILURE, HOUSING_DECOR_PLACE_FAILURE, HOUSING_DECOR_SELECT_RESPONSE, HOUSING_LAYOUT_ROOM_COMPONENT_THEME_SET_CHANGED, HOUSING_ROOM_COMPONENT_CUSTOMIZATION_CHANGE_FAILED, HOUSING_SET_EXTERIOR_HOUSE_SIZE_RESPONSE, HOUSING_SET_EXTERIOR_HOUSE_TYPE_RESPONSE, HOUSING_SET_FIXTURE_RESPONSE, NEIGHBORHOOD_LIST_UPDATED)
- L304: + RoomPlacementOutOfBounds
- L318: Enum.SecretAspect (FrameScriptObject:HasSecretAspect)
- L319: + RadialProgress

Acceptance: concrete nonempty fixtures, non-default state transitions/readback, output arity/optionality and relevant rejection/taint/ownership boundaries; preserve explicitly bounded shape-only scope and no live-service claim. For layout/presentation rows, actual frame/region state is required; metadata round trips alone do not prove rendering.

Existing test entry points: `tests/patch_12_0_5_enum_additions.rs:94`, `src/loader/tests/wow_api_globals/housing_result.rs:219`.

Contract qualifier: INFERRED: historical 69587 values if current docs postdate that epoch; exact current-cache numeric values are recorded per row.

### C64 — discord-chat-payload (2 rows; modelable)

Rows: `structures-ChatMessageEventParams-339`, `structures-ChatMessageEventParams-340`.

Boundary: Chat dispatch exists, but discordInfo has no simulator producer. Add explicit DiscordChatInfo payload to the local event model and preserve the expanded chat filter argument vector.

Work: **model the missing delta, then behavioral proof**. Use the exact source claims below as acceptance assertions; do not substitute namespace/member presence or inspect generated code shape.

- L339: ChatMessageEventParams
- L340: + discordInfo

Acceptance: concrete nonempty fixtures, non-default state transitions/readback, output arity/optionality and relevant rejection/taint/ownership boundaries; preserve explicitly bounded shape-only scope and no live-service claim. For layout/presentation rows, actual frame/region state is required; metadata round trips alone do not prove rendering.

Existing test entry points: No exact test located; add targeted behavior tests for this slice.

Contract qualifier: INFERRED: local Discord fixtures and delivery order; no OAuth/network delivery contract inferred.

### C65 — club-discord-field (2 rows; modelable)

Rows: `structures-ClubMemberInfo-341`, `structures-ClubMemberInfo-342`.

Boundary: Club member getters have a producer but no discordInfo field. Add optional DiscordChatInfo backed by explicit member state; nil defaults alone are not nonempty coverage.

Work: **model the missing delta, then behavioral proof**. Use the exact source claims below as acceptance assertions; do not substitute namespace/member presence or inspect generated code shape.

- L341: ClubMemberInfo (C_Club.GetInfoFromLastCommunityChatLine, C_Club.GetInvitationInfo, C_Club.GetInvitationsForClub, C_Club.GetInvitationsForSelf, C_Club.GetMemberInfo, C_Club.GetMemberInfoForSelf, C_Club.GetMessageInfo, C_Club.GetMessagesBefore, C_Club.GetMessagesInRange, C_Club.GetTickets, CLUB_INVITATION_ADDED_FOR_SELF, CLUB_TICKET_CREATED)
- L342: + discordInfo

Acceptance: concrete nonempty fixtures, non-default state transitions/readback, output arity/optionality and relevant rejection/taint/ownership boundaries; preserve explicitly bounded shape-only scope and no live-service claim. For layout/presentation rows, actual frame/region state is required; metadata round trips alone do not prove rendering.

Existing test entry points: No exact test located; add targeted behavior tests for this slice.

Contract qualifier: INFERRED: local club fixture state; current declaration supplies field type/optional status.

### C66 — cooldown-viewer-payload (4 rows; modelable)

Rows: `structures-CooldownViewerCooldown-343`, `structures-CooldownViewerCooldown-344`, `structures-CooldownViewerCooldown-345`, `structures-CooldownViewerCooldown-346`.

Boundary: GetCooldownViewerCooldownInfo is a temporary nil default; no spellCategoryID/equipSlot/isInvisible producer exists. Model explicit viewer entries with optional category/equipSlot and required visibility flag.

Work: **model the missing delta, then behavioral proof**. Use the exact source claims below as acceptance assertions; do not substitute namespace/member presence or inspect generated code shape.

- L343: CooldownViewerCooldown (C_CooldownViewer.GetCooldownViewerCooldownInfo)
- L344: + spellCategoryID
- L345: + equipSlot
- L346: + isInvisible

Acceptance: concrete nonempty fixtures, non-default state transitions/readback, output arity/optionality and relevant rejection/taint/ownership boundaries; preserve explicitly bounded shape-only scope and no live-service claim. For layout/presentation rows, actual frame/region state is required; metadata round trips alone do not prove rendering.

Existing test entry points: No exact test located; add targeted behavior tests for this slice.

Contract qualifier: INFERRED: synthetic cooldown catalog/visibility fixtures; current declaration supplies types/optionality.

### C67 — decor-pet-attachment (2 rows; modelable)

Rows: `structures-HousingDecorInstanceInfo-347`, `structures-HousingDecorInstanceInfo-348`.

Boundary: Housing backing state exists, but no canAttachPet output producer exists. Add eligibility to decor instance state and serialize it through each relevant mode query.

Work: **model the missing delta, then behavioral proof**. Use the exact source claims below as acceptance assertions; do not substitute namespace/member presence or inspect generated code shape.

- L347: HousingDecorInstanceInfo (C_HousingBasicMode.GetHoveredDecorInfo, C_HousingBasicMode.GetSelectedDecorInfo, C_HousingCleanupMode.GetHoveredDecorInfo, C_HousingCustomizeMode.GetHoveredDecorInfo, C_HousingCustomizeMode.GetSelectedDecorInfo, C_HousingDecor.GetDecorInstanceInfoForGUID, C_HousingDecor.GetHoveredDecorInfo, C_HousingDecor.GetSelectedDecorInfo, C_HousingExpertMode.GetHoveredDecorInfo, C_HousingExpertMode.GetSelectedDecorInfo)
- L348: + canAttachPet

Acceptance: concrete nonempty fixtures, non-default state transitions/readback, output arity/optionality and relevant rejection/taint/ownership boundaries; preserve explicitly bounded shape-only scope and no live-service claim. For layout/presentation rows, actual frame/region state is required; metadata round trips alone do not prove rendering.

Existing test entry points: No exact test located; add targeted behavior tests for this slice.

Contract qualifier: INFERRED: local pet-attachment eligibility fixture, not service/native eligibility rules.

### C68 — lfg-active-entry (2 rows; modelable)

Rows: `structures-LfgEntryData-349`, `structures-LfgEntryData-350`.

Boundary: GetActiveEntryInfo currently returns nil; implement a nonempty active entry with censored field from explicit local listing state. Search-result field does not prove active-entry output.

Work: **model the missing delta, then behavioral proof**. Use the exact source claims below as acceptance assertions; do not substitute namespace/member presence or inspect generated code shape.

- L349: LfgEntryData (C_LFGList.GetActiveEntryInfo)
- L350: + censored

Acceptance: concrete nonempty fixtures, non-default state transitions/readback, output arity/optionality and relevant rejection/taint/ownership boundaries; preserve explicitly bounded shape-only scope and no live-service claim. For layout/presentation rows, actual frame/region state is required; metadata round trips alone do not prove rendering.

Existing test entry points: `src/loader/tests/wow_api_globals/patch_12_1_service_payloads.rs:529`.

Contract qualifier: INFERRED: fixture censorship policy; no server filtering proof.

### C69 — pet-journal-table-shape (6 rows; modelable)

Rows: `structures-PetJournalPetInfo-353`, `structures-PetJournalPetInfo-354`, `structures-PetJournalPetInfo-355`, `structures-PetJournalPetInfo-356`, `structures-PetJournalPetInfo-357`, `structures-PetJournalPetInfo-358`.

Boundary: Species-ID table producer provides only a partial schema with canAttachToDecor=false/creatureModelScale=1; no GetPetInfoTableByPetID producer was found. Model both documented getters, complete named fields, nullable fields and luaIndex type over explicit pet state.

Work: **model the missing delta, then behavioral proof**. Use the exact source claims below as acceptance assertions; do not substitute namespace/member presence or inspect generated code shape.

- L353: PetJournalPetInfo (C_PetJournal.GetPetInfoTableByPetID, C_PetJournal.GetPetInfoTableBySpeciesID)
- L354: # [3].Nilable false -> true
- L355: # [4].Nilable false -> true
- L356: # [5].Nilable false -> true
- L357: # [6].Nilable false -> true
- L358: # [7].Nilable false -> true

Acceptance: concrete nonempty fixtures, non-default state transitions/readback, output arity/optionality and relevant rejection/taint/ownership boundaries; preserve explicitly bounded shape-only scope and no live-service claim. For layout/presentation rows, actual frame/region state is required; metadata round trips alone do not prove rendering.

Existing test entry points: `src/loader/tests/wow_api_globals/patch_12_1_service_payloads.rs:529`.

Contract qualifier: INFERRED: optional field data and deterministic pet eligibility fixtures; current docs define field/type/optionality.

### C70 — pet-journal-table-shape (4 rows; modelable)

Rows: `structures-PetJournalPetInfo-359`, `structures-PetJournalPetInfo-360`, `structures-PetJournalPetInfo-361`, `structures-PetJournalPetInfo-362`.

Boundary: Species-ID table producer provides only a partial schema with canAttachToDecor=false/creatureModelScale=1; no GetPetInfoTableByPetID producer was found. Model both documented getters, complete named fields, nullable fields and luaIndex type over explicit pet state.

Work: **model the missing delta, then behavioral proof**. Use the exact source claims below as acceptance assertions; do not substitute namespace/member presence or inspect generated code shape.

- L359: # [9].Type number -> luaIndex
- L360: # [14].Nilable false -> true
- L361: + canAttachToDecor
- L362: + creatureModelScale

Acceptance: concrete nonempty fixtures, non-default state transitions/readback, output arity/optionality and relevant rejection/taint/ownership boundaries; preserve explicitly bounded shape-only scope and no live-service claim. For layout/presentation rows, actual frame/region state is required; metadata round trips alone do not prove rendering.

Existing test entry points: `src/loader/tests/wow_api_globals/patch_12_1_service_payloads.rs:529`.

Contract qualifier: INFERRED: optional field data and deterministic pet eligibility fixtures; current docs define field/type/optionality.

### C71 — sound-volume-options (2 rows; modelable)

Rows: `structures-PlaySoundParams-363`, `structures-PlaySoundParams-364`.

Boundary: Current PlaySoundWithOptions compatibility path is a no-op. Model and validate volumeOverride in the sound request/options state, with observable queued request or recording sink; no hardware/service restart needed.

Work: **model the missing delta, then behavioral proof**. Use the exact source claims below as acceptance assertions; do not substitute namespace/member presence or inspect generated code shape.

- L363: PlaySoundParams (C_Sound.PlaySoundWithOptions)
- L364: + volumeOverride

Acceptance: concrete nonempty fixtures, non-default state transitions/readback, output arity/optionality and relevant rejection/taint/ownership boundaries; preserve explicitly bounded shape-only scope and no live-service claim. For layout/presentation rows, actual frame/region state is required; metadata round trips alone do not prove rendering.

Existing test entry points: `src/loader/tests/wow_api_globals/startup_globals.rs:823`.

Contract qualifier: INFERRED: volume range/unknown-option policy if no exact native declaration supplies it.

### C72 — player-choice-art (2 rows; modelable)

Rows: `structures-PlayerChoiceInfo-365`, `structures-PlayerChoiceInfo-366`.

Boundary: C_PlayerChoice serializes seeded local choice state but omits hideAnswerArt. Add the boolean state/serialization and prove live replacement through GetCurrentPlayerChoiceInfo.

Work: **model the missing delta, then behavioral proof**. Use the exact source claims below as acceptance assertions; do not substitute namespace/member presence or inspect generated code shape.

- L365: PlayerChoiceInfo (C_PlayerChoice.GetCurrentPlayerChoiceInfo)
- L366: + hideAnswerArt

Acceptance: concrete nonempty fixtures, non-default state transitions/readback, output arity/optionality and relevant rejection/taint/ownership boundaries; preserve explicitly bounded shape-only scope and no live-service claim. For layout/presentation rows, actual frame/region state is required; metadata round trips alone do not prove rendering.

Existing test entry points: `src/loader/tests/wow_api_globals/patch_12_1_service_payloads.rs:561`.

Contract qualifier: INFERRED: default/local fixture state; non-nil boolean type is declared.

### C73 — delve-tier-field-shape (2 rows; modelable)

Rows: `structures-TieredEntranceTierInfo-367`, `structures-TieredEntranceTierInfo-368`.

Boundary: Retail serializer emits overrideTooltipSpellID=nil and isLFG=false. Current declaration requires a non-nil overrideTooltipSpellID and uses queueAsLFG rather than isLFG; model/epoch-reconcile that delta instead of crediting plausible placeholders.

Work: **model the missing delta, then behavioral proof**. Use the exact source claims below as acceptance assertions; do not substitute namespace/member presence or inspect generated code shape.

- L367: TieredEntranceTierInfo (C_DelvesUI.GetActiveDelveTier, C_DelvesUI.GetDelveEntranceTiers)
- L368: + overrideTooltipSpellID

Acceptance: concrete nonempty fixtures, non-default state transitions/readback, output arity/optionality and relevant rejection/taint/ownership boundaries; preserve explicitly bounded shape-only scope and no live-service claim. For layout/presentation rows, actual frame/region state is required; metadata round trips alone do not prove rendering.

Existing test entry points: `src/loader/tests/wow_api_globals/startup_globals.rs:834`.

Contract qualifier: INFERRED: historical isLFG name and tooltip default until pinned 69587 declaration is available; current-cache queueAsLFG must not silently replace captured source.

## Metadata-only rows

- `source-context-002`: Editorial source context only; capabilities `[]`; no runtime credit.
- `source-context-003`: Editorial source context only; capabilities `[]`; no runtime credit.
- `source-context-004`: Editorial source context only; capabilities `[]`; no runtime credit.
- `source-context-005`: Editorial source context only; capabilities `[]`; no runtime credit.
- `source-context-008`: Editorial source context only; capabilities `[]`; no runtime credit.
- `source-context-019`: Editorial source context only; capabilities `[]`; no runtime credit.
- `source-context-022`: Editorial source context only; capabilities `[]`; no runtime credit.
- `source-context-023`: Editorial source context only; capabilities `[]`; no runtime credit.
- `source-context-026`: Editorial source context only; capabilities `[]`; no runtime credit.
- `source-context-029`: Editorial source context only; capabilities `[]`; no runtime credit.
- `source-context-031`: Editorial source context only; capabilities `[]`; no runtime credit.
- `source-context-033`: Editorial source context only; capabilities `[]`; no runtime credit.
- `source-context-036`: Editorial source context only; capabilities `[]`; no runtime credit.
- `source-context-040`: Editorial source context only; capabilities `[]`; no runtime credit.
- `source-context-042`: Editorial source context only; capabilities `[]`; no runtime credit.
- `source-context-044`: Editorial source context only; capabilities `[]`; no runtime credit.
- `source-context-049`: Editorial source context only; capabilities `[]`; no runtime credit.
- `source-context-051`: Editorial source context only; capabilities `[]`; no runtime credit.
- `source-context-068`: Editorial source context only; capabilities `[]`; no runtime credit.
- `source-context-071`: Editorial source context only; capabilities `[]`; no runtime credit.
- `source-context-072`: Editorial source context only; capabilities `[]`; no runtime credit.
- `source-context-074`: Editorial source context only; capabilities `[]`; no runtime credit.
- `source-context-076`: Editorial source context only; capabilities `[]`; no runtime credit.
- `source-context-080`: Editorial source context only; capabilities `[]`; no runtime credit.
- `source-context-093`: Editorial source context only; capabilities `[]`; no runtime credit.
- `source-context-094`: Editorial source context only; capabilities `[]`; no runtime credit.
- `source-context-095`: Editorial source context only; capabilities `[]`; no runtime credit.
- `source-context-104`: Editorial source context only; capabilities `[]`; no runtime credit.
- `source-context-105`: Editorial source context only; capabilities `[]`; no runtime credit.
- `source-context-112`: Editorial source context only; capabilities `[]`; no runtime credit.
- `source-context-113`: Editorial source context only; capabilities `[]`; no runtime credit.
- `source-context-114`: Editorial source context only; capabilities `[]`; no runtime credit.
- `source-context-143`: Editorial source context only; capabilities `[]`; no runtime credit.
- `source-context-156`: Editorial source context only; capabilities `[]`; no runtime credit.
- `source-context-157`: Editorial source context only; capabilities `[]`; no runtime credit.
- `source-context-158`: Editorial source context only; capabilities `[]`; no runtime credit.
- `source-context-170`: Editorial source context only; capabilities `[]`; no runtime credit.
- `source-context-171`: Editorial source context only; capabilities `[]`; no runtime credit.
- `source-context-172`: Editorial source context only; capabilities `[]`; no runtime credit.
- `source-context-192`: Editorial source context only; capabilities `[]`; no runtime credit.
- `source-context-198`: Editorial source context only; capabilities `[]`; no runtime credit.
- `source-context-202`: Editorial source context only; capabilities `[]`; no runtime credit.
- `source-context-203`: Editorial source context only; capabilities `[]`; no runtime credit.
- `source-context-204`: Editorial source context only; capabilities `[]`; no runtime credit.
- `source-context-228`: Editorial source context only; capabilities `[]`; no runtime credit.
- `source-context-229`: Editorial source context only; capabilities `[]`; no runtime credit.
- `source-context-230`: Editorial source context only; capabilities `[]`; no runtime credit.
- `source-context-243`: Editorial source context only; capabilities `[]`; no runtime credit.
- `source-context-244`: Editorial source context only; capabilities `[]`; no runtime credit.
- `source-context-247`: Editorial source context only; capabilities `[]`; no runtime credit.
- `source-context-248`: Editorial source context only; capabilities `[]`; no runtime credit.
- `source-context-251`: Editorial source context only; capabilities `[]`; no runtime credit.
- `source-context-252`: Editorial source context only; capabilities `[]`; no runtime credit.
- `source-context-255`: Editorial source context only; capabilities `[]`; no runtime credit.
- `source-context-258`: Editorial source context only; capabilities `[]`; no runtime credit.
- `source-context-261`: Editorial source context only; capabilities `[]`; no runtime credit.
- `source-context-264`: Editorial source context only; capabilities `[]`; no runtime credit.
- `source-context-267`: Editorial source context only; capabilities `[]`; no runtime credit.
- `source-context-328`: Editorial source context only; capabilities `[]`; no runtime credit.
- `source-context-377`: Editorial source context only; capabilities `[]`; no runtime credit.
- `source-context-378`: Editorial source context only; capabilities `[]`; no runtime credit.
- `source-context-383`: Editorial source context only; capabilities `[]`; no runtime credit.
- `source-context-388`: Editorial source context only; capabilities `[]`; no runtime credit.
- `source-context-393`: Editorial source context only; capabilities `[]`; no runtime credit.

## Blocked rows — last, no implementation design

- `prose-undated-016`: Blocked: simulator does not publish ManifestInterfaceData/exportinterfacefiles art; need authenticated pre/post-patch DB/export snapshots to prove filename retention and absence of new names. No implementation batch.
- `prose-2026-06-18-025`: Blocked: historical rollout/security rationale or author-facing communication is not an executable Retail contract; exact PTR build/native historical evidence is missing. Preserve the statement without runtime credit.
- `prose-2026-06-18-027`: Blocked: historical rollout/security rationale or author-facing communication is not an executable Retail contract; exact PTR build/native historical evidence is missing. Preserve the statement without runtime credit.
- `prose-2026-06-18-028`: Blocked: historical rollout/security rationale or author-facing communication is not an executable Retail contract; exact PTR build/native historical evidence is missing. Preserve the statement without runtime credit.
- `prose-2026-06-18-030`: Blocked: historical rollout/security rationale or author-facing communication is not an executable Retail contract; exact PTR build/native historical evidence is missing. Preserve the statement without runtime credit.
- `prose-2026-06-18-039`: Blocked: PTR1 five-button AddAuraFrame example is removed by later PTR4 text; exact original PTR1 addon/native surface is absent from current Retail. Current managed groups do not prove the historical example.
- `prose-2026-06-18-045`: Blocked: historical rollout/security rationale or author-facing communication is not an executable Retail contract; exact PTR build/native historical evidence is missing. Preserve the statement without runtime credit.
- `prose-2026-06-18-046`: Blocked: historical rollout/security rationale or author-facing communication is not an executable Retail contract; exact PTR build/native historical evidence is missing. Preserve the statement without runtime credit.
- `prose-2026-06-18-047`: Blocked: historical rollout/security rationale or author-facing communication is not an executable Retail contract; exact PTR build/native historical evidence is missing. Preserve the statement without runtime credit.
- `prose-2026-06-18-050`: Blocked: simulator does not publish ManifestInterfaceData/exportinterfacefiles art; need authenticated pre/post-patch DB/export snapshots to prove filename retention and absence of new names. No implementation batch.
- `prose-2026-06-23-073`: Blocked: historical rollout/security rationale or author-facing communication is not an executable Retail contract; exact PTR build/native historical evidence is missing. Preserve the statement without runtime credit.
- `prose-2026-07-07-115`: Blocked: narrative progress/exploit rationale has no specified observable Retail result; exact historical/native evidence is missing. No implementation batch.
- `prose-2026-07-07-149`: Blocked: PTR4 AuraContainer-in-combat crash/error was superseded by PTR7 allowing creation; current Retail cannot prove the original PTR4 native failure. Exact original build/native trace is missing.
- `prose-2026-07-07-150`: Blocked: PTR4 AuraContainer-in-combat crash/error was superseded by PTR7 allowing creation; current Retail cannot prove the original PTR4 native failure. Exact original build/native trace is missing.
- `prose-2026-07-14-165`: Blocked: narrative progress/exploit rationale has no specified observable Retail result; exact historical/native evidence is missing. No implementation batch.
- `prose-2026-07-21-199`: Blocked: retained extract omits the healer buff/HoT spell list. Exact 12.1.0 classification DB and pinned native spell secrecy flags are missing; scratch wikitext can recover names/IDs but not native flags.

Total: **73 batches**, covering 253 substantive nonblocked rows. No tests or runtime executed.
