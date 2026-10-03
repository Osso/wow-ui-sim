```json
[
  {
    "source_id": "prose-2026-04-10-196",
    "status": "audit-pending",
    "note": "12.0.5-api-changes.txt:196: \"Various chat channel APIs are no longer usable while chat lockdown is in effect, or from within macros.\" Checked real channel mutations, explicit lockdown state, macro dispatch and cached ChatInfo declarations: none identifies the affected legacy mutator set. RunMacro records a persistent slot; RunMacroText has no dynamic /run or /script channel-call route. Missing API inventory, actual macro provenance and native denial convention; SendChatMessage annotations do not establish channel membership/moderation restrictions. Unblock with an authoritative per-API list or native probes, scoped executable macro entry/restoration, and denial-before-mutation/recovery proof. No guessed all-channel or combat-equivalence credit.",
    "evidence": [
      "data/patch-api/evidence/12.0.5-session-2026-10-03/handoff-channel-lockdown.md",
      "src/lua_api/globals/channel_verbs.rs",
      "src/lua_api/globals/spell_macro_verbs.rs",
      "/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua"
    ]
  },
  {
    "source_id": "prose-2026-03-25-093",
    "status": "audit-pending",
    "note": "12.0.5-api-changes.txt:93: \"Added improved error messaging for various security-related errors involving tables.\" Checked pinned rilua table-security guard messages, simulator freeze diagnostics and registration gates; real rejection exists but no named changed operation, native replacement text or error-shape baseline is supplied. Current table.freeze is registered for retail-12-0-5, contrary to the older handoff epoch caveat; native settablesecurity registration remains Forever-only. Unblock with operation-specific pre/post native diagnostics or authoritative wording/shape and applicable epoch, then prove actual failure boundaries. Rewording current errors or asserting inferred categories would not establish this delta.",
    "evidence": [
      "data/patch-api/evidence/12.0.5-session-2026-10-03/handoff-diag-throttle.md",
      "Cargo.lock",
      "src/lua_api/globals/register.rs",
      "src/lua_api/env_init/mod.rs",
      "/home/osso-test/.cargo/git/checkouts/rilua-fd5a0715e46b5888/6044544/src/table_security.rs"
    ]
  },
  {
    "source_id": "prose-2026-03-25-095",
    "status": "audit-pending",
    "note": "12.0.5-api-changes.txt:95: \"Addon execution throttles will no longer be applied while processing PLAYER_LOGOUT and ADDONS_UNLOADING events.\" Same exemption requirement as the other occurrence. Checked event registration/dispatch, ReloadUI and pinned rilua hooks: teardown names are recognized, but no actual teardown producer or trusted addon execution meter was found; debug count hooks and zero throttle-limit mocks are not enforcement. User reports script-execution-limits exists only on another machine; GitHub ls-remote returned no matching branch, and Cargo.lock still pins 6044544. Unblock by making that VM capability available, integrating trusted metering and real lifecycle-scoped exemption, specifying attribution/reset/failure policy, and proving normal-budget denial, both exemptions and restoration after errors.",
    "evidence": [
      "data/patch-api/evidence/12.0.5-session-2026-10-03/handoff-diag-throttle.md",
      "Cargo.lock",
      "src/lua_api/env_events.rs",
      "src/lua_api/globals/state_backed_queries.rs",
      "src/lua_api/workarounds/temporary/script_bucket_throttle_limits.rs"
    ]
  },
  {
    "source_id": "prose-2026-03-12-050",
    "status": "audit-pending",
    "note": "12.0.5-api-changes.txt:50: \"Addon execution throttles will no longer apply while the PLAYER_LOGOUT and ADDONS_UNLOADING events are being processed.\" Same exemption requirement as the other occurrence. Checked event registration/dispatch, ReloadUI and pinned rilua hooks: teardown names are recognized, but no actual teardown producer or trusted addon execution meter was found; debug count hooks and zero throttle-limit mocks are not enforcement. User reports script-execution-limits exists only on another machine; GitHub ls-remote returned no matching branch, and Cargo.lock still pins 6044544. Unblock by making that VM capability available, integrating trusted metering and real lifecycle-scoped exemption, specifying attribution/reset/failure policy, and proving normal-budget denial, both exemptions and restoration after errors.",
    "evidence": [
      "data/patch-api/evidence/12.0.5-session-2026-10-03/handoff-diag-throttle.md",
      "Cargo.lock",
      "src/lua_api/env_events.rs",
      "src/lua_api/globals/state_backed_queries.rs",
      "src/lua_api/workarounds/temporary/script_bucket_throttle_limits.rs"
    ]
  },
  {
    "source_id": "prose-2026-03-25-099",
    "status": "audit-pending",
    "note": "12.0.5-api-changes.txt:99: \"Fixed an issue with the IsUnderMouse] API where it would incorrectly return false for parent frames that were actually under the mouse.\" Recheck disproves the handoff blanket absence claim: cached RestrictedFrames.lua:360 defines HANDLE:IsUnderMouse(recursive), using rectangle checks and protected-visible child recursion at 324-370. Simulator IsMouseOver separately uses frame bounds and visibility/mouse-enabled flags; it is not an established alias. Missing an identified failing parent/restricted-handle fixture and native pre/post result, not generic rendering capability. Unblock by identifying the source API boundary and reproducing parent/child, recursive, scale and visibility cases through unchanged restricted Lua; do not invent an IsUnderMouse frame-method alias.",
    "evidence": [
      "/tmp/claude-1000/-home-osso-test-Projects-wow-wow-ui-sim/f86e1850-973d-4071-a472-f5757f1d7fdc/scratchpad/handoff-mouse-tm.md",
      "/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_RestrictedAddOnEnvironment/RestrictedFrames.lua",
      "src/lua_api/frame/methods/core_state/region.rs"
    ]
  },
  {
    "source_id": "prose-2026-03-12-049",
    "status": "audit-pending",
    "note": "12.0.5-api-changes.txt:49: \"UnitTokenNamePlate APIs will no longer accept party, raid, or target-of-target unit tokens.\" Checked NamePlate and NamePlateManager declarations: UnitTokenNamePlate is an argument type, not a callable API. GetNamePlateForUnit currently returns zero values unconditionally from a permanent shim, with no live token-to-plate association; this cannot prove category rejection or allowed-token success. Missing exhaustive affected-consumer inventory, accepted token grammar/alias policy and native rejection shape. Unblock with authoritative declarations/native category probes and a bounded real plate association/consumer model, then test allowed and rejected inputs, original secret authentication and unchanged state. No shim or input-secrecy credit for this category restriction.",
    "evidence": [
      "/tmp/claude-1000/-home-osso-test-Projects-wow-wow-ui-sim/f86e1850-973d-4071-a472-f5757f1d7fdc/scratchpad/handoff-identity-tokens.md",
      "src/c_api/permanent_shims/c_nameplate.rs",
      "/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/NamePlateDocumentation.lua",
      "/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/NamePlateManagerDocumentation.lua"
    ]
  },
  {
    "source_id": "prose-2026-03-25-085",
    "status": "audit-pending",
    "note": "12.0.5-api-changes.txt:85: \"UnitTokenNamePlate APIs no longer accept party, raid, or target-of-target unit tokens.\" Checked NamePlate and NamePlateManager declarations: UnitTokenNamePlate is an argument type, not a callable API. GetNamePlateForUnit currently returns zero values unconditionally from a permanent shim, with no live token-to-plate association; this cannot prove category rejection or allowed-token success. Missing exhaustive affected-consumer inventory, accepted token grammar/alias policy and native rejection shape. Unblock with authoritative declarations/native category probes and a bounded real plate association/consumer model, then test allowed and rejected inputs, original secret authentication and unchanged state. No shim or input-secrecy credit for this category restriction.",
    "evidence": [
      "/tmp/claude-1000/-home-osso-test-Projects-wow-wow-ui-sim/f86e1850-973d-4071-a472-f5757f1d7fdc/scratchpad/handoff-identity-tokens.md",
      "src/c_api/permanent_shims/c_nameplate.rs",
      "/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/NamePlateDocumentation.lua",
      "/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/NamePlateManagerDocumentation.lua"
    ]
  },
  {
    "source_id": "prose-2026-03-31-172",
    "status": "audit-pending",
    "note": "12.0.5-api-changes.txt:172: \"Added a secure delegate for ClearSelectedSystem in EditMode, so addons can call it without taint concerns while not in combat.\" Checked unchanged EditModeManager.lua:697-717 and CreateSecureDelegate declarations/options: vendor delegates secure or out-of-combat calls, but directly runs cleanup for insecure combat calls. Simulator delegate is still function identity. Missing protected secure execution, creation restriction policy, untrusted callback wrapping and caller-taint/error restoration; a securecall wrapper alone does not meet the documented boundary. Unblock with a real delegate primitive and loaded vendor highlighted-frame/dialog tests plus native creation-policy evidence. Do not invent combat denial/preserved selection or patch vendor cleanup.",
    "evidence": [
      "/tmp/claude-1000/-home-osso-test-Projects-wow-wow-ui-sim/f86e1850-973d-4071-a472-f5757f1d7fdc/scratchpad/handoff-editmode-nameplate.md",
      "src/lua_api/workarounds/temporary/debug_environment_defaults.rs",
      "/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_EditMode/Shared/EditModeManager.lua",
      "/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/FrameScriptDocumentation.lua"
    ]
  },
  {
    "source_id": "prose-2026-04-10-202",
    "status": "audit-pending",
    "note": "12.0.5-api-changes.txt:202: \"Fixed a bug that could cause some auras showing in the cooldown manager to have incorrect tooltips if they were active when an encounter begins.\" Checked cached CooldownViewer.lua:1779-1810 UNIT_AURA cache/layout refresh and ItemData.lua:1012-1015 tooltip lookup by cached aura instance ID. The handoff stages rekeying but explicitly supplies no loaded viewer refresh/tooltip proof; current src has no aura_entry producer. Missing pre-existing live aura encounter-entry rekey/notification ordering and resulting tooltip identity at the real consumer. Unblock with host-backed aura entry transitions and an unchanged loaded cooldown-viewer fixture proving old-ID invalidation, new binding and correct tooltip payload. Query authentication or isolated rekey tests do not close this row.",
    "evidence": [
      "/tmp/claude-1000/-home-osso-test-Projects-wow-wow-ui-sim/f86e1850-973d-4071-a472-f5757f1d7fdc/scratchpad/handoff-nav-rekey.md",
      "/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_CooldownViewer/CooldownViewer.lua",
      "/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_CooldownViewer/CooldownViewerItemData.lua",
      "src/lua_api/env_events.rs"
    ]
  },
  {
    "source_id": "global api-C_CatalogShop-GetProductInfo-247",
    "status": "audit-pending",
    "note": "12.0.5-api-changes.txt:247: \"+ HasRestrictions\" on C_CatalogShop.GetProductInfo. Checked cached CatalogShopDocumentation.lua:226-238/421-433 and current providers: GetProductInfo reads live product records. HasRestrictions and AllowedWhenUntainted are separate annotations; neither defines which caller/context triggers the new execution restriction or its denial result. Missing native restriction predicate and observable denial convention, not commerce/payment simulation. Unblock with authoritative policy or native per-context probes, then implement the evidenced guard before reads/mutations and prove allowed, denied and recovery cases. Presence, public DTOs or a host can-purchase boolean do not establish this exact delta; no metadata-only credit.",
    "evidence": [
      "data/patch-api/evidence/12.0.5-session-2026-10-03/scout-held-rows.md",
      "src/c_api/c_catalog_shop_products.rs",
      "src/lua_api/workarounds/temporary/housing_catalog_state.lua",
      "/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/CatalogShopDocumentation.lua"
    ]
  },
  {
    "source_id": "global api-C_CatalogShop-PurchaseProduct-249",
    "status": "audit-pending",
    "note": "12.0.5-api-changes.txt:249: \"+ HasRestrictions\" on C_CatalogShop.PurchaseProduct. Checked cached CatalogShopDocumentation.lua:226-238/421-433 and current providers: PurchaseProduct remains __wow_noop in housing_catalog_state.lua. HasRestrictions and AllowedWhenUntainted are separate annotations; neither defines which caller/context triggers the new execution restriction or its denial result. Missing native restriction predicate and observable denial convention, not commerce/payment simulation. Unblock with authoritative policy or native per-context probes, then implement the evidenced guard before reads/mutations and prove allowed, denied and recovery cases. Presence, public DTOs or a host can-purchase boolean do not establish this exact delta; no metadata-only credit.",
    "evidence": [
      "data/patch-api/evidence/12.0.5-session-2026-10-03/scout-held-rows.md",
      "src/c_api/c_catalog_shop_products.rs",
      "src/lua_api/workarounds/temporary/housing_catalog_state.lua",
      "/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/CatalogShopDocumentation.lua"
    ]
  },
  {
    "source_id": "widgets-FontString-GetFont-544",
    "status": "audit-pending",
    "note": "12.0.5-api-changes.txt:544: \"# ret1.Type cstring -> FontAsset\". Checked SimpleFontStringAPIDocumentation.lua:121-132/500-515 and current formatting.rs:330-419: setter converts/stores String paths; getter returns a path string/default. Generated declarations name FontAsset and validation flags but do not define its Lua runtime representations, accepted conversions or GetFont preservation/canonicalization. Existing path round trips prove old cstring behavior, not equivalence of the changed domain. Unblock with authoritative FontAsset definition or native representation/invalid-asset/return probes, then model each evidenced representation and atomic validation. Do not guess numeric IDs or mark metadata-only without demonstrated runtime equivalence.",
    "evidence": [
      "data/patch-api/evidence/12.0.5-session-2026-10-03/scout-held-rows.md",
      "src/lua_api/frame/methods/text_attribute_event/text/formatting.rs",
      "/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/SimpleFontStringAPIDocumentation.lua"
    ]
  },
  {
    "source_id": "widgets-FontString-SetFont-546",
    "status": "audit-pending",
    "note": "12.0.5-api-changes.txt:546: \"# arg1.Type cstring -> FontAsset\". Checked SimpleFontStringAPIDocumentation.lua:121-132/500-515 and current formatting.rs:330-419: setter converts/stores String paths; getter returns a path string/default. Generated declarations name FontAsset and validation flags but do not define its Lua runtime representations, accepted conversions or GetFont preservation/canonicalization. Existing path round trips prove old cstring behavior, not equivalence of the changed domain. Unblock with authoritative FontAsset definition or native representation/invalid-asset/return probes, then model each evidenced representation and atomic validation. Do not guess numeric IDs or mark metadata-only without demonstrated runtime equivalence.",
    "evidence": [
      "data/patch-api/evidence/12.0.5-session-2026-10-03/scout-held-rows.md",
      "src/lua_api/frame/methods/text_attribute_event/text/formatting.rs",
      "/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/SimpleFontStringAPIDocumentation.lua"
    ]
  },
  {
    "source_id": "prose-2026-03-12-056",
    "status": "audit-pending",
    "note": "12.0.5-api-changes.txt:56 says \"we made MOST encounter debuffs private auras rather than just a select few\" and \"Our own private aura displays don't currently have full parity with normal auras\"; it also states nameplates are unsupported. Checked cached AuraIsPrivate declaration, private-aura binding/callback plumbing and empty nameplate providers. These do not specify the encounter/debuff population, which display features differ, or an observable acceptance boundary. Mixed historical context and concrete content/display claims are not wholly metadata-only. Unblock with epoch-specific encounter fixtures and explicit display/nameplate behavior scope or native captures. A host private-spell set or anchor test alone cannot establish MOST or unspecified parity; retain pending without full private-aura coverage.",
    "evidence": [
      "data/patch-api/evidence/12.0.5-session-2026-10-03/scout-held-rows.md",
      "src/c_api/private_aura_anchors.rs",
      "src/c_api/permanent_shims/c_nameplate.rs",
      "/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/UnitAuraDocumentation.lua"
    ]
  },
  {
    "source_id": "prose-2026-03-25-076",
    "status": "audit-pending",
    "note": "12.0.5-api-changes.txt:76: \"Fixing a bug where some players randomly become unable to set raid markers until they restart their client.\" Checked SetRaidTarget live GUID-keyed assignment/removal and RaidMarkersDocumentation.lua restrictions; these contain no random sticky-denial trigger or native permission/context transition sequence. Missing reproduced process-lifetime failure boundary and recovery without restart. Unblock with a native/recorded failing sequence, then exercise the same ordering against the real marker producer and prove recovery. Repeated happy-path calls, fresh environments or an invented can_set_markers latch would not reproduce this bug; no regression or metadata-only credit.",
    "evidence": [
      "data/patch-api/evidence/12.0.5-session-2026-10-03/scout-held-rows.md",
      "src/lua_api/globals/targeting_verbs.rs",
      "/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/RaidMarkersDocumentation.lua"
    ]
  },
  {
    "source_id": "prose-2026-03-12-055",
    "status": "metadata-only",
    "note": "12.0.5-api-changes.txt:55: \"we had hoped (planned even) to be able to essentially retire the Private Aura system and replace it with just secrets.\" Checked the adjacent line56, which explicitly describes going in the opposite direction, and the scout chronology: this paragraph records historical origin, motivation and abandoned design intent, not a promised finalized 12.0.5 API delta. Explicit historical/narrative classification satisfies proof_policy; no missing runtime capability or probe is required for this sentence. Metadata-only retains source chronology and grants no runtime/implementation credit. Actual private-aura API restrictions and line56 content/display claims remain separately pending; reopen only if authoritative evidence turns this abandoned intention into a concrete applicable contract.",
    "evidence": [
      "data/patch-api/evidence/12.0.5-session-2026-10-03/scout-held-rows.md",
      "data/patch-api/sources/12.0.5-api-changes.txt"
    ]
  },
  {
    "source_id": "prose-2026-03-31-174",
    "status": "audit-pending",
    "note": "12.0.5-api-changes.txt:174: \"NPC followers are now treated as player units by nameplates\", enabling the two named friendly-player CVars. Checked cached NamePlateUnitFrame.lua classification and name-only updates: actual consumer calls ClearAllHitTestPoints/SetHitTestPoints, absent from src; C_NamePlate still has no live plates. Handoff stages a GUID-backed query only, explicitly not loaded nameplate/CVar consumer proof. Missing live nameplate fixture/lifecycle and hit-test primitives, not a boolean-classification assertion. Unblock by approving that bounded nameplate model and proving follower class-color/name-only output through unchanged vendor Lua with live CVar updates. Current intentional world-nameplate exclusion cannot erase this observable UI requirement.",
    "evidence": [
      "/tmp/claude-1000/-home-osso-test-Projects-wow-wow-ui-sim/f86e1850-973d-4071-a472-f5757f1d7fdc/scratchpad/handoff-editmode-nameplate.md",
      "src/c_api/permanent_shims/c_nameplate.rs",
      "/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_NamePlates/Blizzard_NamePlateUnitFrame.lua"
    ]
  },
  {
    "source_id": "prose-2026-04-10-200",
    "status": "audit-pending",
    "note": "12.0.5-api-changes.txt:200: \"Fixed a bug that caused taint logging to stop working after a reload UI.\" Checked current Cargo.lock, src logging hooks and ReloadUI: pin 6044544 lacks the tainted-table-read observer; no simulator producer consumes one. A newer observer exists in local rilua-tainted-read-hook, but is not the pinned dependency; v2 staging is not implementation. ReloadUI only dispatches PLAYER_ENTERING_WORLD in the same VM, not native VM recreation. Unblock with an available observer revision and real read-to-log sink integration, explicit log policy, and before/during/after-reload lifecycle proof. Supplied records, caller-taint polling or same-VM continuity alone do not establish native logging/reload parity.",
    "evidence": [
      "/tmp/claude-1000/-home-osso-test-Projects-wow-wow-ui-sim/f86e1850-973d-4071-a472-f5757f1d7fdc/scratchpad/handoff-taint-log-v2.md",
      "Cargo.lock",
      "src/lua_api/globals/state_backed_queries.rs",
      "/home/osso-test/.worktrees/rilua-tainted-read-hook/src/vm/state/tainted_table_read.rs"
    ]
  },
  {
    "source_id": "prose-2026-04-10-201",
    "status": "audit-pending",
    "note": "12.0.5-api-changes.txt:201: \"Fixed a bug that caused taint logging to not correctly write messages for tainted reads from tables.\" Checked current Cargo.lock, src logging hooks and ReloadUI: pin 6044544 lacks the tainted-table-read observer; no simulator producer consumes one. A newer observer exists in local rilua-tainted-read-hook, but is not the pinned dependency; v2 staging is not implementation. ReloadUI only dispatches PLAYER_ENTERING_WORLD in the same VM, not native VM recreation. Unblock with an available observer revision and real read-to-log sink integration, explicit log policy, and before/during/after-reload lifecycle proof. Supplied records, caller-taint polling or same-VM continuity alone do not establish native logging/reload parity.",
    "evidence": [
      "/tmp/claude-1000/-home-osso-test-Projects-wow-wow-ui-sim/f86e1850-973d-4071-a472-f5757f1d7fdc/scratchpad/handoff-taint-log-v2.md",
      "Cargo.lock",
      "src/lua_api/globals/state_backed_queries.rs",
      "/home/osso-test/.worktrees/rilua-tainted-read-hook/src/vm/state/tainted_table_read.rs"
    ]
  },
  {
    "source_id": "prose-2026-04-17-214",
    "status": "audit-pending",
    "note": "12.0.5-api-changes.txt:214 says \"The new chat restrictions will also be inactive in Classic\" and \"The existing pre-Midnight security systems (taint, restricted actions, etc.) will still be active of course.\" It also requires unchanged Classic combat logs, inactive Midnight guild/API restrictions and available duration/curve objects. Checked client-profile selection, security registration and scout accounting: individual constructors or chat tests do not establish this conjunction. Missing named guild/other API inventory, combat-log payload scope and per-profile epoch matrix. Unblock with that authoritative matrix and real allowed/denied/event/object fixtures across its profiles. Not metadata-only; neither Classic secret-disablement nor blanket removal of protected-action checks closes it.",
    "evidence": [
      "data/patch-api/evidence/12.0.5-session-2026-10-03/scout-remaining.md",
      "src/client_profile.rs",
      "src/lua_api/env_init/mod.rs",
      "data/patch-api/sources/12.0.5-api-changes.txt"
    ]
  },
  {
    "source_id": "global api-Localization CreateAbbreviateConfig-413",
    "status": "audit-pending",
    "note": "12.0.5-api-changes.txt:413: \"+ RequiresRestrictedAbbreviationBreakpoints\" on CreateAbbreviateConfig. Checked cached Localization/AbbreviateConfig declarations and LocalizationSharedDocumentation.lua:33-38: restriction failure mode is Error, but allowed breakpoint/divisor rules are unspecified. Current proxy factory copies arbitrary initial state and setter stores arbitrary data; round-trip tests do not validate the declared breakpoint-array contract. Missing native restricted predicate, not error-versus-return shape. Unblock with authoritative numeric rules or native accepted/rejected triples, then shared owned-data validation at constructor/setter boundaries with atomic rejection and isolation. Do not replace the unknown rule with a host allow-set or infer metadata-only from the annotation.",
    "evidence": [
      "data/patch-api/evidence/12.0.5-session-2026-10-03/scout-remaining.md",
      "src/lua_api/workarounds/temporary/proxy_object_factories.rs",
      "/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/LocalizationSharedDocumentation.lua",
      "/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/LocalizationDocumentation.lua",
      "/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/AbbreviateConfigAPIDocumentation.lua"
    ]
  },
  {
    "source_id": "scriptobjects-AbbreviateConfig-SetAbbreviateNumberData-522",
    "status": "audit-pending",
    "note": "12.0.5-api-changes.txt:522: \"+ RequiresRestrictedAbbreviationBreakpoints\" on AbbreviateConfig:SetAbbreviateNumberData. Checked cached Localization/AbbreviateConfig declarations and LocalizationSharedDocumentation.lua:33-38: restriction failure mode is Error, but allowed breakpoint/divisor rules are unspecified. Current proxy factory copies arbitrary initial state and setter stores arbitrary data; round-trip tests do not validate the declared breakpoint-array contract. Missing native restricted predicate, not error-versus-return shape. Unblock with authoritative numeric rules or native accepted/rejected triples, then shared owned-data validation at constructor/setter boundaries with atomic rejection and isolation. Do not replace the unknown rule with a host allow-set or infer metadata-only from the annotation.",
    "evidence": [
      "data/patch-api/evidence/12.0.5-session-2026-10-03/scout-remaining.md",
      "src/lua_api/workarounds/temporary/proxy_object_factories.rs",
      "/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/LocalizationSharedDocumentation.lua",
      "/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/LocalizationDocumentation.lua",
      "/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/AbbreviateConfigAPIDocumentation.lua"
    ]
  },
  {
    "source_id": "prose-2026-03-12-038",
    "status": "audit-pending",
    "note": "12.0.5-api-changes.txt:38: \"The Ambiguate function now accepts secrets from addons.\" Checked current ambiguate.rs and pinned rilua: name shortening is Lua string.match, while the pin lacks transform_host_secret_string. A trusted secret-string transform exists in the local rilua-secret-string-transform worktree; v2 dependency edits still use a revision placeholder. Missing an available integrated VM primitive that transforms authentic secret bytes without granting addon unwrap or changing caller taint. Unblock with a resolvable helper revision, real provider wiring and authentic tainted-secret input/result-secrecy tests. Public name/context proof does not establish secret fullName acceptance; do not reuse an untainted-only unwrap or add a Lua-visible escape hatch.",
    "evidence": [
      "/tmp/claude-1000/-home-osso-test-Projects-wow-wow-ui-sim/f86e1850-973d-4071-a472-f5757f1d7fdc/scratchpad/handoff-ambiguate-v2.md",
      "src/lua_api/globals/real/ambiguate.rs",
      "Cargo.lock",
      "/home/osso-test/.worktrees/rilua-secret-string-transform/src/table_security.rs"
    ]
  },
  {
    "source_id": "global api-PlayerScript Ambiguate-416",
    "status": "audit-pending",
    "note": "12.0.5-api-changes.txt:416: \"# SecretArguments AllowedWhenUntainted -> AllowedWhenTainted\" Checked current ambiguate.rs and pinned rilua: name shortening is Lua string.match, while the pin lacks transform_host_secret_string. A trusted secret-string transform exists in the local rilua-secret-string-transform worktree; v2 dependency edits still use a revision placeholder. Missing an available integrated VM primitive that transforms authentic secret bytes without granting addon unwrap or changing caller taint. Unblock with a resolvable helper revision, real provider wiring and authentic tainted-secret input/result-secrecy tests. Public name/context proof does not establish secret fullName acceptance; do not reuse an untainted-only unwrap or add a Lua-visible escape hatch.",
    "evidence": [
      "/tmp/claude-1000/-home-osso-test-Projects-wow-wow-ui-sim/f86e1850-973d-4071-a472-f5757f1d7fdc/scratchpad/handoff-ambiguate-v2.md",
      "src/lua_api/globals/real/ambiguate.rs",
      "Cargo.lock",
      "/home/osso-test/.worktrees/rilua-secret-string-transform/src/table_security.rs"
    ]
  }
]
```

| Unblocks with | Rows / remaining boundary |
|---|---|
| rilua capability | 050/095: trusted execution meter plus real teardown dispatch; other-machine script-execution-limits branch has no GitHub head. Additional 200/201: pin/integrate local read observer and logging lifecycle. Additional 038/416: pin/integrate local trusted secret-string transform. |
| Native probe | 093: changed table-error operations/messages; 099: actual restricted HANDLE parent regression (cached method exists); 076: sticky marker failure/recovery sequence; 247/249: restriction predicates/denial; 544/546: FontAsset representations/returns; additional 413/522: numeric restriction rules (Error failure mode already declared). |
| Authoritative API list | 196: affected channel calls plus real dynamic macro route; 049/085: all UnitTokenNamePlate consumers/category grammar; additional 214: Classic API/event/profile matrix. |
| Design decision / consumer capability | 172: complete secure-delegate trust boundary, not combat denial; 202: entry-ID/notification/loaded-tooltip ordering; 056: concrete encounter/display scope; additional 174: live nameplates and hit-test methods. 055 is abandoned intent, metadata-only, with no runtime credit. |
| 3D gap | World nameplates are intentionally absent (049/085/174); extending plate association/2D consumer fixtures needs explicit scope, not a 3D renderer workaround. Model mesh/render/load-success remains excluded for 534/542 and prose194/195; current model.rs now routes both setters to model_unit::assign_unit, so the scout's missing-wiring claim is stale and is not listed as a new blocker. Identity denial/nil behavior is not excluded. |
