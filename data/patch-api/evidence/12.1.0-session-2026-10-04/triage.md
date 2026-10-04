# Retail 12.1.0 page-audit preparation

Static snapshot: 2026-10-04. No Cargo, tests, builds, simulator, Git mutation, agents or model CLIs. All evidence below is source inspection; existing tests were located/read, **not run**. No conformance or passing-test claim.

## Identity, completeness and classification policy

One row per nonblank captured-text line (333 rows/396 physical lines), including field/type annotations, historical prose and headings. Named IDs use lowercase section + literal symbol with `.`/`:` changed to `-` + three-digit source line; spaces in section names are retained. Prose dates inherit explicit source date headings; Notes are genuinely undated. Context alone is `metadata-only`; all substantive ledger rows remain `audit-pending` with empty capabilities. This triage does not promote them.

**Retained extract is incomplete:** six inline collapsed consolidated tables, code examples, initial Bluepost body and healer spell list were omitted. Exact missing entries and capture requirements are in [README](README.md); no supplemental missing entry was invented as a ledger row. A complete audit of the actual Wiki page remains open.

`CACHE/` citations expand to `/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/`. Cache declaration signatures and unmodified Lua are source evidence, not native behavior or pinned build 69587 proof. Same-patch prose contains superseded PTR-week contracts; blocked historical checkpoints are preserved explicitly, never silently credited from final Retail.

Cargo.toml:118–121 defines 12.1.0 -> 12.0.7 -> 12.0.5 feature inheritance; Cargo.toml:149 selects retail-12-1-0 for client-retail. **src/lib.rs:80–81 also compiles src/ptr under retail-12-1-0**, and src/lua_api/env_init/mod.rs:77 executes its compatibility bootstrap. The directory name `ptr` does not establish PTR-only behavior. Its append-based enum fill publishes many symbols in default Retail, sometimes at wrong values. Table/name presence is not behavior proof. Wrong values and omitted producer paths are `modelable`, not tests-only.

Cross-reference inputs: source `12.1-behaviors.json` (54 behavior occurrences), source `12.1-framexml.json` (320 added/112 removed historical symbols), `12.1.0-ptr-cache-manifest.txt`, and current `data/patch-api/12.1-behaviors.json` / `12.1-framexml.json`. Their PTR manifest/hash/status fields were not reused as Retail acceptance. Relevant prior behavior anchors appear below; absence of a corresponding old row does not block new modeling. Current page FrameXML table says 337/124, so the old 320/112 register is not exhaustive for this capture.

Implemented-needs-proof means a current producer for the stated bounded contract is present, **not** that full native/service semantics are correct. Fixed-default structure fields get only shape scope. Modelable means declared or explicitly INFERRED state/policy can be implemented without waiting for problematic native traces. Blocked rows name the exact historical/source/DB evidence missing and receive no implementation design. No 3D work, live service implementation, vendor modification or new deployment is proposed.

No entire source row inherits a 12.0.5/12.0.7 capability here: new deltas exceed those recorded scopes. Relevant bounded reuse includes `unit-name-secret-tokens`, `enum-publication`, `private-aura-anchor-fields`, `private-aura-sound-add-context`, `duration-text-binding-12-0-7` and `vehicle-aura-sound-asset-12-0-7`; these do not supply blanket credit for changed policy/fields/renames.

## Counts

- Resources: 4
- Notes: 9
- Blue posts: 193
- Global API: 2
- FrameXML: 2
- ScriptObjects: 1
- Widgets: 1
- Events: 1
- CVars: 1
- Enumerations: 59
- Structures: 47
- Deprecated API: 13

- metadata-only: 64
- implemented-needs-proof: 92
- modelable: 161
- blocked: 16
- inherits-existing-capability: 0

## Per-row triage

### L2 `source-context-002` — metadata-only

Source: `== Resources ==`

Assessment: Editorial heading, chronology, navigation, file label or disclaimer only; no runtime credit.

### L3 `source-context-003` — metadata-only

Source: `TOC: 120100`

Assessment: Editorial heading, chronology, navigation, file label or disclaimer only; no runtime credit.

### L4 `source-context-004` — metadata-only

Source: `Official patch notes: Curse of Ula'tek Content Update Notes`

Assessment: Editorial heading, chronology, navigation, file label or disclaimer only; no runtime credit.

### L5 `source-context-005` — metadata-only

Source: `Diffs: wow-ui-source, BlizzardInterfaceResources`

Assessment: Editorial heading, chronology, navigation, file label or disclaimer only; no runtime credit.

### L8 `source-context-008` — metadata-only

Source: `=== Notes ===`

Assessment: Editorial heading, chronology, navigation, file label or disclaimer only; no runtime credit.

### L9 `prose-undated-009` — implemented-needs-proof

Source: `Added AuraContainer and AuraButton intrinsic frames.`

Assessment / ledger note: AuraContainer/AuraButton type construction and template loading; this does not establish managed presentation or security.

Current producer or adjacent incomplete producer: `src/xml/types_elements.rs:425`, `src/xml/types_elements.rs:433`.

Existing tests (unexecuted; related/bounded only): `src/loader/tests/wow_api_globals/frames_and_attributes.rs:337`.

Cached contract: Page statement plus current producer cited above; no exact generated declaration anchor located for the entire combined statement.

Prior 12.1 cross-reference (status NOT reused): `data/patch-api/12.1-behaviors.json:632`.

### L10 `prose-undated-010` — modelable

Source: `SVG textures are now supported with the VectorGraphics object type.`

Assessment / ledger note: VectorGraphics construction/SVG metadata storage exists, but native code explicitly says SVG path rendering is not modeled. Implement 2D SVG asset decoding/rendering and legal method surface; not a 3D feature.

Current producer or adjacent incomplete producer: `src/lua_api/frame/methods/widgets/texture/mod.rs:123`, `src/lua_api/frame/methods/widgets/texture/radial.rs:119`.

Existing tests (unexecuted; related/bounded only): `src/loader/tests/wow_api_globals/frames_and_attributes.rs:103`.

Cached contract: Page statement plus current producer cited above; no exact generated declaration anchor located for the entire combined statement.

Contract qualification: INFERRED: fixture SVG subset and unsupported-method behavior pending exact native declarations.

### L11 `prose-undated-011` — implemented-needs-proof

Source: `UIParentLoadAddOn has been moved to LoadAddOnWithErrorHandling.`

Assessment / ledger note: The migration is supplied by unmodified cached FrameXML; prove actual Retail helper behavior and ordinary/global lookup, not the stale PTR inventory alone.

Current producer or adjacent incomplete producer: `CACHE/Blizzard_SharedXML/AddOnUtil.lua:3`, `CACHE/Blizzard_SharedXML/InputUtil.lua:24`, `src/loader/addon.rs:125`.

Existing tests (unexecuted; related/bounded only): `patch-tests/patch_12_1/remaining_observations.rs:117`.

Cached contract: Page statement plus current producer cited above; no exact generated declaration anchor located for the entire combined statement.

### L12 `prose-undated-012` — implemented-needs-proof

Source: `MouseIsOver has been moved to InputUtil.IsMouseOver.`

Assessment / ledger note: The migration is supplied by unmodified cached FrameXML; prove actual Retail helper behavior and ordinary/global lookup, not the stale PTR inventory alone.

Current producer or adjacent incomplete producer: `CACHE/Blizzard_SharedXML/AddOnUtil.lua:3`, `CACHE/Blizzard_SharedXML/InputUtil.lua:24`, `src/loader/addon.rs:125`.

Existing tests (unexecuted; related/bounded only): `patch-tests/patch_12_1/remaining_observations.rs:117`.

Cached contract: Page statement plus current producer cited above; no exact generated declaration anchor located for the entire combined statement.

### L13 `prose-undated-013` — modelable

Source: `CanAccessObject has been replaced with FrameScriptObject:CanBeAccessedInContext.`

Assessment / ledger note: HasAccessConstraints reads native flags, but CanBeAccessedInContext is registered only under client-wowforever. Retail needs its context-access query and aura restriction policy, not a copied Forever availability claim.

Current producer or adjacent incomplete producer: `src/lua_api/frame/methods/text_attribute_event/attributes.rs:493`, `src/lua_api/frame/methods/misc/secret.rs:38`.

Existing tests (unexecuted; related/bounded only): `tests/forbidden_frames.rs:92`, `tests/forbidden_frames.rs:172`.

Cached contract: Page statement plus current producer cited above; no exact generated declaration anchor located for the entire combined statement.

### L14 `prose-undated-014` — modelable

Source: `Deprecated getglobal and setglobal.`

Assessment / ledger note: Audit current cached deprecated wrappers and default Retail strict-removal timing. A historical PTR symbol observation is not Retail migration acceptance.

Current producer or adjacent incomplete producer: `src/ptr/compat_bootstrap.rs:51`, `src/lua_api/globals/utility_system_spell/mod.rs:194`, `CACHE/Blizzard_Deprecated/Shared/Deprecated_12_1_0.lua:9`.

Existing tests (unexecuted; related/bounded only): `patch-tests/patch_12_1/strict_removal_timing.rs:96`.

Cached contract: Page statement plus current producer cited above; no exact generated declaration anchor located for the entire combined statement.

Prior 12.1 cross-reference (status NOT reused): `data/patch-api/12.1-behaviors.json:1553`.

### L15 `prose-undated-015` — modelable

Source: `The Auto Loot setting (CVar autoLootDefault) is now account wide.`

Assessment / ledger note: Implement the declared CVar/default/scope and policy rather than generic name acceptance. Current CVar defaults are a nearby producer; account-wide autoLoot and session-only tooltip setting need persistence boundaries.

Current producer or adjacent incomplete producer: `src/cvars.rs:1`.

Existing tests (unexecuted; related/bounded only): `tests/set_cvar_global.rs:7`.

Cached contract: Page statement plus current producer cited above; no exact generated declaration anchor located for the entire combined statement.

Contract qualification: INFERRED only for local account/session fixture ownership; stated default and persistence behavior remain explicit requirements.

### L16 `prose-undated-016` — blocked

Source: `New UI texture filenames will no longer be published to the ManifestInterfaceData DB. Existing filenames will remain available, and this change will not affect players. Addons will still be able to use these textures.`

Assessment / ledger note: Blocked: simulator does not publish ManifestInterfaceData/exportinterfacefiles art; need authenticated pre/post-patch DB/export snapshots to prove filename retention and absence of new names. No implementation batch.

Evidence: captured source L16; missing historical/native/source artifact stated above. No batch.

### L19 `source-context-019` — metadata-only

Source: `== Blue posts ==`

Assessment: Editorial heading, chronology, navigation, file label or disclaimer only; no runtime credit.

### L22 `source-context-022` — metadata-only

Source: `=== 2026-06-18 ===`

Assessment: Editorial heading, chronology, navigation, file label or disclaimer only; no runtime credit.

### L23 `source-context-023` — metadata-only

Source: `Midnight 12.1.0 PTR Changes 1 (Build 68209)`

Assessment: Editorial heading, chronology, navigation, file label or disclaimer only; no runtime credit.

### L25 `prose-2026-06-18-025` — blocked

Source: `Hello again from the World of Warcraft UI Engineering team! Today we’d like to talk about a significant set of Aura-related changes coming in 12.1. Most of these changes will be available when PTR launches, with the remaining pieces rolling out over the following few weeks.`

Assessment / ledger note: Blocked: historical rollout/security rationale or author-facing communication is not an executable Retail contract; exact PTR build/native historical evidence is missing. Preserve the statement without runtime credit.

Evidence: captured source L25; missing historical/native/source artifact stated above. No batch.

### L26 `source-context-026` — metadata-only

Source: `Why Auras?`

Assessment: Editorial heading, chronology, navigation, file label or disclaimer only; no runtime credit.

### L27 `prose-2026-06-18-027` — blocked

Source: `Since the Addon Disarmament project went live with Midnight, Auras (aka buffs and debuffs) have consistently been one of the weakest areas for addon security, with numerous exploits discovered both before launch and since then. The core issue is that, in many cases, simply knowing that any aura is present on a unit (whether it be the player, an enemy, or a raid/party member) is enough to determine that some important combat event has occurred. Aura filters are vital for many legitimate addon use cases, but they also make this problem harder to contain by allowing even more ways to tell if “special aura X” is on a unit, even if the unit has multiple auras on them.`

Assessment / ledger note: Blocked: historical rollout/security rationale or author-facing communication is not an executable Retail contract; exact PTR build/native historical evidence is missing. Preserve the statement without runtime credit.

Evidence: captured source L27; missing historical/native/source artifact stated above. No batch.

### L28 `prose-2026-06-18-028` — blocked

Source: `Up until now, our solution to this has been to lean on Private Auras. Unfortunately, Private Auras come with several downsides: they are invisible to addons, which prevents customization; they are not supported in every context, such as nameplates; and setting them up across every encounter adds significant setup work for our designers. Secret values were created specifically to protect against cases like this, providing passive protection by default.`

Assessment / ledger note: Blocked: historical rollout/security rationale or author-facing communication is not an executable Retail contract; exact PTR build/native historical evidence is missing. Preserve the statement without runtime credit.

Evidence: captured source L28; missing historical/native/source artifact stated above. No batch.

### L29 `source-context-029` — metadata-only

Source: `What is changing?`

Assessment: Editorial heading, chronology, navigation, file label or disclaimer only; no runtime credit.

### L30 `prose-2026-06-18-030` — blocked

Source: `We’ll get to the changes to existing APIs shortly, but first, we’d like to introduce a couple of new constructs we are adding to Lua, along with two new object types (Aura Containers and Aura Buttons).`

Assessment / ledger note: Blocked: historical rollout/security rationale or author-facing communication is not an executable Retail contract; exact PTR build/native historical evidence is missing. Preserve the statement without runtime credit.

Evidence: captured source L30; missing historical/native/source artifact stated above. No batch.

### L31 `source-context-031` — metadata-only

Source: `New Tech: Private Script Objects & The Forbidden Partition`

Assessment: Editorial heading, chronology, navigation, file label or disclaimer only; no runtime credit.

### L32 `prose-2026-06-18-032` — modelable

Source: `Private Script Objects are a new construct that lets us split the Lua representation of a script object across multiple Lua tables, or partitions. One of these partitions we call the Forbidden Partition, because it is inaccessible to addons. The Forbidden Partition can contain any kind of value, from mixins to key/value pairs, functions, script handlers, and child objects. This allows us to effectively hide portions of the object from addon code even when the object itself isn’t in the secure environment.`

Assessment / ledger note: Public/forbidden proxy tables and XML mixin routing exist. Retail addon-inaccessible partitions, handler ownership and native access restrictions still need a complete model; table partitioning alone is not isolation proof.

Current producer or adjacent incomplete producer: `src/lua_api/env_init/shared_bootstrap.lua:239`, `src/lua_api/script_object_transfer.rs:1`.

Existing tests (unexecuted; related/bounded only): `tests/userdata_proxy.rs:37`, `tests/xml_secure_delegates.rs:7`.

Cached contract: Page statement plus current producer cited above; no exact generated declaration anchor located for the entire combined statement.

### L33 `source-context-033` — metadata-only

Source: `New Tech: Forbidden Aspects`

Assessment: Editorial heading, chronology, navigation, file label or disclaimer only; no runtime credit.

### L34 `prose-2026-06-18-034` — modelable

Source: `Forbidden Aspects are another new construct that works alongside Private Script Objects. Forbidden Aspects are similar in concept to the Secret Aspects we introduced in Midnight, but instead of causing certain object APIs to return secrets, they prevent addons from using certain functionality entirely. Where Secret Aspects obfuscate data, Forbidden Aspects restrict what addons are allowed to do with an object.`

Assessment / ledger note: Native aspect masks/inheritance exist, but mask publication does not establish UntrustedScriptExecution or UntrustedLayoutScriptExecution suppression across all callback/anchor paths. Model caller-sensitive dispatch before claiming security.

Current producer or adjacent incomplete producer: `src/lua_api/frame/methods/forbidden_aspects.rs:64`.

Existing tests (unexecuted; related/bounded only): `tests/forbidden_aspect_creation.rs:14`.

Cached contract: Page statement plus current producer cited above; no exact generated declaration anchor located for the entire combined statement.

Prior 12.1 cross-reference (status NOT reused): `data/patch-api/12.1-behaviors.json:332`.

### L35 `prose-2026-06-18-035` — modelable

Source: `There are several Forbidden Aspects being added (details are in the docs), but let’s use the UntrustedScriptExecution Forbidden Aspect as an example. When a frame has the UntrustedScriptExecution Forbidden Aspect applied to it, any script binding handlers set on it (e.g. OnShow, OnLoad, OnSizeChanged) will not be run unless that handler lives in the object’s Forbidden Partition and execution is untainted. In other words, addons cannot install their own script bindings on the object, but our code can.`

Assessment / ledger note: Native aspect masks/inheritance exist, but mask publication does not establish UntrustedScriptExecution or UntrustedLayoutScriptExecution suppression across all callback/anchor paths. Model caller-sensitive dispatch before claiming security.

Current producer or adjacent incomplete producer: `src/lua_api/frame/methods/forbidden_aspects.rs:64`.

Existing tests (unexecuted; related/bounded only): `tests/forbidden_aspect_creation.rs:14`.

Cached contract: Page statement plus current producer cited above; no exact generated declaration anchor located for the entire combined statement.

Prior 12.1 cross-reference (status NOT reused): `data/patch-api/12.1-behaviors.json:332`.

### L36 `source-context-036` — metadata-only

Source: `New Object Types: Aura Containers & Aura Buttons`

Assessment: Editorial heading, chronology, navigation, file label or disclaimer only; no runtime credit.

### L37 `prose-2026-06-18-037` — implemented-needs-proof

Source: `Aura Containers and Aura Buttons are new Lua object types that allow addons to display auras in custom ways. Here’s a small example showing how they can be used:`

Assessment / ledger note: AuraContainer/AuraButton type construction and template loading; this does not establish managed presentation or security.

Current producer or adjacent incomplete producer: `src/xml/types_elements.rs:425`, `src/xml/types_elements.rs:433`.

Existing tests (unexecuted; related/bounded only): `src/loader/tests/wow_api_globals/frames_and_attributes.rs:337`.

Cached contract: Page statement plus current producer cited above; no exact generated declaration anchor located for the entire combined statement.

Prior 12.1 cross-reference (status NOT reused): `data/patch-api/12.1-behaviors.json:632`.

### L39 `prose-2026-06-18-039` — blocked

Source: `In the example above, we create an Aura Container, specify that it should track the first 5 helpful auras on the player’s target, and then add 5 Aura Buttons to it. For each Aura Button, we create a texture for the icon and a font string for the duration. The APIs shown here on the Aura Button are just a sample of the APIs provided (full details will be in the docs), but this should give you a sense of what is possible. Note that addon code still has a great deal of control over how the auras are presented, but it doesn’t interact with the underlying aura data at all. This separation is important for security, but it should also make custom aura displays easier to build and more performant. Aura Containers handle the tracking, filtering, and updating of aura assignments internally, so addons can focus more on presentation and less on repeatedly querying, diffing, and refreshing aura state themselves.`

Assessment / ledger note: Blocked: PTR1 five-button AddAuraFrame example is removed by later PTR4 text; exact original PTR1 addon/native surface is absent from current Retail. Current managed groups do not prove the historical example.

Evidence: captured source L39; missing historical/native/source artifact stated above. No batch.

### L40 `source-context-040` — metadata-only

Source: `Why Are Aura Containers Safer?`

Assessment: Editorial heading, chronology, navigation, file label or disclaimer only; no runtime credit.

### L41 `prose-2026-06-18-041` — modelable

Source: `To answer that, let’s go back to Private Script Objects and Forbidden Aspects again. Aura Buttons and Aura Containers both have Forbidden Aspects applied to them on creation. When an Aura Button is added to an Aura Container using the AddAuraFrame API, it is added to the Forbidden Partition of that Aura Container. This means addon code cannot install script handlers on Aura Buttons to be notified when they show or hide. It also cannot hook functions called on the Aura Button’s mixins or register events on those buttons. While addons can still hold references to those individual Aura Buttons, calling certain APIs on them will be disallowed, and they cannot run logic based on whether those buttons are shown, because IsShown and similar APIs return secrets.`

Assessment / ledger note: Public/forbidden proxy tables and XML mixin routing exist. Retail addon-inaccessible partitions, handler ownership and native access restrictions still need a complete model; table partitioning alone is not isolation proof.

Current producer or adjacent incomplete producer: `src/lua_api/env_init/shared_bootstrap.lua:239`, `src/lua_api/script_object_transfer.rs:1`.

Existing tests (unexecuted; related/bounded only): `tests/userdata_proxy.rs:37`, `tests/xml_secure_delegates.rs:7`.

Cached contract: Page statement plus current producer cited above; no exact generated declaration anchor located for the entire combined statement.

### L42 `source-context-042` — metadata-only

Source: `Which current APIs are changing?`

Assessment: Editorial heading, chronology, navigation, file label or disclaimer only; no runtime credit.

### L43 `prose-2026-06-18-043` — modelable

Source: `The main change to existing APIs is that, when auras are secret (during combat, encounters, M+, and PvP matches), all of the UnitAura APIs will now either return full secrets or nil when called by addons. That means that APIs like GetUnitAuras and GetUnitAuraInstanceIDs will return a secret vector, meaning addon code will not be able to determine how many auras it contains or iterate through it for display. Auras we explicitly flag as non-secret will still be returned as non-secret by UnitAura APIs, however.`

Assessment / ledger note: Existing aura queries/filtering do not supply complete Retail caller-sensitive secret vectors, restricted index/slot/instance access, fully secret AuraData and UNIT_AURA payloads. Model these outputs and error boundaries over explicit aura-secret context.

Current producer or adjacent incomplete producer: `src/lua_api/globals/auras.rs:420`, `src/c_api/c_unit_aura_filter_query.rs:22`.

Existing tests (unexecuted; related/bounded only): `tests/unit_aura_slot_secret_arguments.rs:189`, `tests/tooltip_aura_instance_security.rs:1`.

Cached contract: Page statement plus current producer cited above; no exact generated declaration anchor located for the entire combined statement.

Contract qualification: INFERRED only for host context injection; public policy comes from this page and current declarations, and may differ between historical PTR weeks.

Prior 12.1 cross-reference (status NOT reused): `data/patch-api/12.1-behaviors.json:24`.

### L44 `source-context-044` — metadata-only

Source: `Is all this in place in PTR Week 1?`

Assessment: Editorial heading, chronology, navigation, file label or disclaimer only; no runtime credit.

### L45 `prose-2026-06-18-045` — blocked

Source: `No, several pieces of this are not currently implemented in the first PTR build but will be coming over the next few weeks. The biggest pieces not in place yet are the changes to the UnitAura APIs. Some Aura Button protections are also not yet in place: their script handlers are protected, but script handlers on their child frames are not, and event registration is still currently allowed. Those protections, along with additional safeguards, will arrive over the next few weeks. In the meantime, though, feel free to start experimenting!`

Assessment / ledger note: Blocked: historical rollout/security rationale or author-facing communication is not an executable Retail contract; exact PTR build/native historical evidence is missing. Preserve the statement without runtime credit.

Evidence: captured source L45; missing historical/native/source artifact stated above. No batch.

### L46 `prose-2026-06-18-046` — blocked

Source: `As always, we are actively seeking your feedback and will be monitoring the ⁠author-wishlist channel, so please share feedback, bugs, and any potential exploits there. Thanks as always for helping us test and improve this system!`

Assessment / ledger note: Blocked: historical rollout/security rationale or author-facing communication is not an executable Retail contract; exact PTR build/native historical evidence is missing. Preserve the statement without runtime credit.

Evidence: captured source L46; missing historical/native/source artifact stated above. No batch.

### L47 `prose-2026-06-18-047` — blocked

Source: `DISCLAIMER: These notes are for addon authors and as such are focused specifically on addon-facing API and security changes only. Changes planned for other parts of the game (UI or otherwise) are not included here.`

Assessment / ledger note: Blocked: historical rollout/security rationale or author-facing communication is not an executable Retail contract; exact PTR build/native historical evidence is missing. Preserve the statement without runtime credit.

Evidence: captured source L47; missing historical/native/source artifact stated above. No batch.

### L49 `source-context-049` — metadata-only

Source: `Interface Texture Filenames`

Assessment: Editorial heading, chronology, navigation, file label or disclaimer only; no runtime credit.

### L50 `prose-2026-06-18-050` — blocked

Source: `Starting in 12.1, new interface texture filenames will no longer be published to the ManifestInterfaceData DB, and as a result will not be available via exportinterfacefiles art. Existing filenames will remain in the DB. You may notice that a few entries are still added in 12.1 and over the next few patches, but this is due to those assets already having been added prior to this change being made. We are making this change to prevent leaks caused by texture names containing hints about future content. We understand that this is going to be a somewhat disruptive change for some addon developers, so please let us know your largest pain points and we'll try to make accommodations where possible.`

Assessment / ledger note: Blocked: simulator does not publish ManifestInterfaceData/exportinterfacefiles art; need authenticated pre/post-patch DB/export snapshots to prove filename retention and absence of new names. No implementation batch.

Evidence: captured source L50; missing historical/native/source artifact stated above. No batch.

### L51 `source-context-051` — metadata-only

Source: `Other changes in 12.1 PTR 1`

Assessment: Editorial heading, chronology, navigation, file label or disclaimer only; no runtime credit.

### L53 `prose-2026-06-18-053` — modelable

Source: `We now support showing SVG textures in our UI. They can be used on regular textures (e.g. file="Path/To/Texture.svg") or with a new VectorGraphics object type, which renders them at higher quality.`

Assessment / ledger note: VectorGraphics construction/SVG metadata storage exists, but native code explicitly says SVG path rendering is not modeled. Implement 2D SVG asset decoding/rendering and legal method surface; not a 3D feature.

Current producer or adjacent incomplete producer: `src/lua_api/frame/methods/widgets/texture/mod.rs:123`, `src/lua_api/frame/methods/widgets/texture/radial.rs:119`.

Existing tests (unexecuted; related/bounded only): `src/loader/tests/wow_api_globals/frames_and_attributes.rs:103`.

Cached contract: Page statement plus current producer cited above; no exact generated declaration anchor located for the entire combined statement.

Contract qualification: INFERRED: fixture SVG subset and unsupported-method behavior pending exact native declarations.

### L54 `prose-2026-06-18-054` — modelable

Source: `Note that the VectorGraphics objects don't currently support all of the APIs on regular Textures (rotation, masking, tex coords, etc.)`

Assessment / ledger note: VectorGraphics construction/SVG metadata storage exists, but native code explicitly says SVG path rendering is not modeled. Implement 2D SVG asset decoding/rendering and legal method surface; not a 3D feature.

Current producer or adjacent incomplete producer: `src/lua_api/frame/methods/widgets/texture/mod.rs:123`, `src/lua_api/frame/methods/widgets/texture/radial.rs:119`.

Existing tests (unexecuted; related/bounded only): `src/loader/tests/wow_api_globals/frames_and_attributes.rs:103`.

Cached contract: Page statement plus current producer cited above; no exact generated declaration anchor located for the entire combined statement.

Contract qualification: INFERRED: fixture SVG subset and unsupported-method behavior pending exact native declarations.

### L55 `prose-2026-06-18-055` — implemented-needs-proof

Source: `Load-on-Demand addons can now specify that specific files in the TOC should load on startup through a new per-file [Bootstrap] directive.`

Assessment / ledger note: TOC Bootstrap annotations and bootstrap-only LoD startup scheduling are implemented. Prove enabled/disabled addons, dependency ordering, one-time phase execution and subsequent full load under default Retail.

Current producer or adjacent incomplete producer: `src/loader/startup_addons.rs:15`, `src/loader/addon.rs:125`.

Existing tests (unexecuted; related/bounded only): `src/loader/tests/lua_loading.rs:176`, `src/toc/tests.rs:388`.

Cached contract: Page statement plus current producer cited above; no exact generated declaration anchor located for the entire combined statement.

### L56 `prose-2026-06-18-056` — implemented-needs-proof

Source: `This still requires that the addon be enabled in order for these files to load.`

Assessment / ledger note: TOC Bootstrap annotations and bootstrap-only LoD startup scheduling are implemented. Prove enabled/disabled addons, dependency ordering, one-time phase execution and subsequent full load under default Retail.

Current producer or adjacent incomplete producer: `src/loader/startup_addons.rs:15`, `src/loader/addon.rs:125`.

Existing tests (unexecuted; related/bounded only): `src/loader/tests/lua_loading.rs:176`, `src/toc/tests.rs:388`.

Cached contract: Page statement plus current producer cited above; no exact generated declaration anchor located for the entire combined statement.

### L57 `prose-2026-06-18-057` — implemented-needs-proof

Source: `UIParent.lua has been heavily refactored, with all of the code that previously handled loading LoD addons moved into the addons themselves, taking advantage of the new [Bootstrap] directive.`

Assessment / ledger note: TOC Bootstrap annotations and bootstrap-only LoD startup scheduling are implemented. Prove enabled/disabled addons, dependency ordering, one-time phase execution and subsequent full load under default Retail.

Current producer or adjacent incomplete producer: `src/loader/startup_addons.rs:15`, `src/loader/addon.rs:125`.

Existing tests (unexecuted; related/bounded only): `src/loader/tests/lua_loading.rs:176`, `src/toc/tests.rs:388`.

Cached contract: Page statement plus current producer cited above; no exact generated declaration anchor located for the entire combined statement.

### L58 `prose-2026-06-18-058` — implemented-needs-proof

Source: `Added a new API Frame:SetOnUpdateMode(mode), which lets you specify when the OnUpdate script on a frame should run.`

Assessment / ledger note: Numeric OnUpdateMode state and dispatch modes are implemented, including hidden RunAlways and one-shot reset/rearm. Existing actual managed-aura dirty-phase test is meaningful but unexecuted in this audit.

Current producer or adjacent incomplete producer: `src/c_api/on_update_modes.rs:7`, `src/lua_api/frame/methods/text_attribute_event/mod.rs:155`.

Existing tests (unexecuted; related/bounded only): `tests/on_update_modes.rs:35`, `tests/on_update_modes.rs:152`.

Cached contract: Page statement plus current producer cited above; no exact generated declaration anchor located for the entire combined statement.

### L59 `prose-2026-06-18-059` — implemented-needs-proof

Source: `The options are Disabled, RunWhenVisible (default), RunWhenVisibleOnce, RunOnce, and RunAlways`

Assessment / ledger note: Numeric OnUpdateMode state and dispatch modes are implemented, including hidden RunAlways and one-shot reset/rearm. Existing actual managed-aura dirty-phase test is meaningful but unexecuted in this audit.

Current producer or adjacent incomplete producer: `src/c_api/on_update_modes.rs:7`, `src/lua_api/frame/methods/text_attribute_event/mod.rs:155`.

Existing tests (unexecuted; related/bounded only): `tests/on_update_modes.rs:35`, `tests/on_update_modes.rs:152`.

Cached contract: Page statement plus current producer cited above; no exact generated declaration anchor located for the entire combined statement.

### L60 `prose-2026-06-18-060` — modelable

Source: `A new system has been added called the Roleset System, which allows you to tag a frame as being part of a "roleset". You can then use the new C_Roleset.ApplyRolesetFilters to specify which rolesets are currently active.`

Assessment / ledger note: Retail ApplyRolesetFilters is a temporary no-op; GetActiveBlockedRolesets/GetActiveAllowedRolesets/IsRolesetFiltered have no source producer. Model allowed/blocked sets, alwaysBlocked and effective visibility from current declarations.

Current producer or adjacent incomplete producer: `src/lua_api/workarounds/temporary/roleset_defaults.rs:11`.

Existing tests (unexecuted; related/bounded only): `tests/wowforever_rolesets.rs:5`.

Cached contract: Page statement plus current producer cited above; no exact generated declaration anchor located for the entire combined statement.

Contract qualification: INFERRED where Retail declarations omit precedence; Forever tests are related evidence, not Retail implementation proof.

### L61 `prose-2026-06-18-061` — modelable

Source: `Frames in an inactive roleset will never be shown, regardless of their shown state. See Blizzard_UIModeManager.lua for more details and examples.`

Assessment / ledger note: Retail ApplyRolesetFilters is a temporary no-op; GetActiveBlockedRolesets/GetActiveAllowedRolesets/IsRolesetFiltered have no source producer. Model allowed/blocked sets, alwaysBlocked and effective visibility from current declarations.

Current producer or adjacent incomplete producer: `src/lua_api/workarounds/temporary/roleset_defaults.rs:11`.

Existing tests (unexecuted; related/bounded only): `tests/wowforever_rolesets.rs:5`.

Cached contract: Page statement plus current producer cited above; no exact generated declaration anchor located for the entire combined statement.

Contract qualification: INFERRED where Retail declarations omit precedence; Forever tests are related evidence, not Retail implementation proof.

### L62 `prose-2026-06-18-062` — modelable

Source: `Radial masking support has been added to textures and status bars, allowing them to have a radial mask applied to them without the need for hacky uses of cooldowns. Example usage on a texture:`

Assessment / ledger note: Radial APIs currently store per-frame metadata; this is not rendered texture/statusbar radial masking. Add presentation and secret-aspect behavior with concrete radial fixtures.

Current producer or adjacent incomplete producer: `src/lua_api/frame/methods/widgets/texture/radial.rs:51`.

Existing tests (unexecuted; related/bounded only): `src/loader/tests/wow_api_globals/frames_and_attributes.rs:157`.

Cached contract: Page statement plus current producer cited above; no exact generated declaration anchor located for the entire combined statement.

Prior 12.1 cross-reference (status NOT reused): `data/patch-api/12.1-behaviors.json:822`.

### L64 `prose-2026-06-18-064` — implemented-needs-proof

Source: `KeyValues can now specify that their value should be pulled directly from the private addon table. Example usage: <KeyValue key="myKey" type="local"/>`

Assessment / ledger note: XML local KeyValues, Mixins blocks and qualified mixin lookup have loader producers; prove per-addon local table identity, nested lookup, precedence and no cross-addon leakage.

Current producer or adjacent incomplete producer: `src/loader/xml_frame_codegen.rs:198`, `src/loader/xml_frame_codegen.rs:351`, `src/lua_api/env_init/shared_bootstrap.lua:239`.

Existing tests (unexecuted; related/bounded only): `src/loader/tests/xml_basics.rs:343`, `tests/xml_secure_delegates.rs:7`.

Cached contract: Page statement plus current producer cited above; no exact generated declaration anchor located for the entire combined statement.

### L65 `prose-2026-06-18-065` — implemented-needs-proof

Source: `Mixins can now be added on an object using a new <Mixins> element.`

Assessment / ledger note: XML local KeyValues, Mixins blocks and qualified mixin lookup have loader producers; prove per-addon local table identity, nested lookup, precedence and no cross-addon leakage.

Current producer or adjacent incomplete producer: `src/loader/xml_frame_codegen.rs:198`, `src/loader/xml_frame_codegen.rs:351`, `src/lua_api/env_init/shared_bootstrap.lua:239`.

Existing tests (unexecuted; related/bounded only): `src/loader/tests/xml_basics.rs:343`, `tests/xml_secure_delegates.rs:7`.

Cached contract: Page statement plus current producer cited above; no exact generated declaration anchor located for the entire combined statement.

### L66 `prose-2026-06-18-066` — implemented-needs-proof

Source: `Using this element allows you to use the source="local" specifier to indicate the mixin lives in the private addon table.`

Assessment / ledger note: XML local KeyValues, Mixins blocks and qualified mixin lookup have loader producers; prove per-addon local table identity, nested lookup, precedence and no cross-addon leakage.

Current producer or adjacent incomplete producer: `src/loader/xml_frame_codegen.rs:198`, `src/loader/xml_frame_codegen.rs:351`, `src/lua_api/env_init/shared_bootstrap.lua:239`.

Existing tests (unexecuted; related/bounded only): `src/loader/tests/xml_basics.rs:343`, `tests/xml_secure_delegates.rs:7`.

Cached contract: Page statement plus current producer cited above; no exact generated declaration anchor located for the entire combined statement.

### L67 `prose-2026-06-18-067` — implemented-needs-proof

Source: `Mixins added on an object (either through the Mixins element or the regular mixin="myMixin" attribute) can also now be nested within tables.`

Assessment / ledger note: XML local KeyValues, Mixins blocks and qualified mixin lookup have loader producers; prove per-addon local table identity, nested lookup, precedence and no cross-addon leakage.

Current producer or adjacent incomplete producer: `src/loader/xml_frame_codegen.rs:198`, `src/loader/xml_frame_codegen.rs:351`, `src/lua_api/env_init/shared_bootstrap.lua:239`.

Existing tests (unexecuted; related/bounded only): `src/loader/tests/xml_basics.rs:343`, `tests/xml_secure_delegates.rs:7`.

Cached contract: Page statement plus current producer cited above; no exact generated declaration anchor located for the entire combined statement.

### L68 `source-context-068` — metadata-only

Source: `Example usage:`

Assessment: Editorial heading, chronology, navigation, file label or disclaimer only; no runtime credit.

### L71 `source-context-071` — metadata-only

Source: `=== 2026-06-23 ===`

Assessment: Editorial heading, chronology, navigation, file label or disclaimer only; no runtime credit.

### L72 `source-context-072` — metadata-only

Source: `Midnight 12.1.0 PTR Changes 2 (Build 68301)`

Assessment: Editorial heading, chronology, navigation, file label or disclaimer only; no runtime credit.

### L73 `prose-2026-06-23-073` — blocked

Source: `Quick note: the previously mentioned restrictions to UnitAura APIs are currently planned for PTR 3, so any addons using those APIs should expect significant changes next week.`

Assessment / ledger note: Blocked: historical rollout/security rationale or author-facing communication is not an executable Retail contract; exact PTR build/native historical evidence is missing. Preserve the statement without runtime credit.

Evidence: captured source L73; missing historical/native/source artifact stated above. No batch.

### L74 `source-context-074` — metadata-only

Source: `Coming in 12.1.0 PTR 2`

Assessment: Editorial heading, chronology, navigation, file label or disclaimer only; no runtime credit.

### L76 `source-context-076` — metadata-only

Source: `Aura Buttons now support the following functionality:`

Assessment: Editorial heading, chronology, navigation, file label or disclaimer only; no runtime credit.

### L77 `prose-2026-06-23-077` — modelable

Source: `Dispel borders: Using the SetAuraBorder(texture, [options]) API (see DefaultAuraBorderOptions for available options).`

Assessment / ledger note: Cached custom button Lua has presentation setters and aura update handlers; real native aura data, secured region ownership and presentation updates remain incomplete. Exercise actual templates, not table-only substitutes.

Current producer or adjacent incomplete producer: `CACHE/Blizzard_AuraContainer/Blizzard_CustomAuraButton.lua:524`, `src/c_api/c_aura_container_util.rs:228`.

Existing tests (unexecuted; related/bounded only): `tests/aura_container_util.rs:6`, `patch-tests/patch_12_1/aura_tooltip.rs:34`.

Cached contract: Page statement plus current producer cited above; no exact generated declaration anchor located for the entire combined statement.

Prior 12.1 cross-reference (status NOT reused): `data/patch-api/12.1-behaviors.json:752`.

### L78 `prose-2026-06-23-078` — modelable

Source: `Dispel type text: Using the SetAuraSymbol(fontString, [options]) API (see DefaultAuraSymbolOptions for available options).`

Assessment / ledger note: Cached custom button Lua has presentation setters and aura update handlers; real native aura data, secured region ownership and presentation updates remain incomplete. Exercise actual templates, not table-only substitutes.

Current producer or adjacent incomplete producer: `CACHE/Blizzard_AuraContainer/Blizzard_CustomAuraButton.lua:524`, `src/c_api/c_aura_container_util.rs:228`.

Existing tests (unexecuted; related/bounded only): `tests/aura_container_util.rs:6`, `patch-tests/patch_12_1/aura_tooltip.rs:34`.

Cached contract: Page statement plus current producer cited above; no exact generated declaration anchor located for the entire combined statement.

Prior 12.1 cross-reference (status NOT reused): `data/patch-api/12.1-behaviors.json:752`.

### L79 `prose-2026-06-23-079` — modelable

Source: `Aura tooltips: Automatically enabled but can be disabled via the SetMouseMotionEnabled API.`

Assessment / ledger note: Cached aura-button tooltip lifecycle exists, including options and throttling. Need native managed button identity, visibility and restricted-aura integration; older AuraButtonMixin mock tooltip tests are not this new contract.

Current producer or adjacent incomplete producer: `CACHE/Blizzard_AuraContainer/Blizzard_AuraButton.lua:216`.

Existing tests (unexecuted; related/bounded only): `patch-tests/patch_12_1/aura_tooltip.rs:34`.

Cached contract: Page statement plus current producer cited above; no exact generated declaration anchor located for the entire combined statement.

Prior 12.1 cross-reference (status NOT reused): `data/patch-api/12.1-behaviors.json:776`.

### L80 `source-context-080` — metadata-only

Source: `Added the following new Forbidden Aspects:`

Assessment: Editorial heading, chronology, navigation, file label or disclaimer only; no runtime credit.

### L81 `prose-2026-06-23-081` — modelable

Source: `UntrustedScriptExecution: When active, addon-installed script handlers on a frame and its children will never be run.`

Assessment / ledger note: Native aspect masks/inheritance exist, but mask publication does not establish UntrustedScriptExecution or UntrustedLayoutScriptExecution suppression across all callback/anchor paths. Model caller-sensitive dispatch before claiming security.

Current producer or adjacent incomplete producer: `src/lua_api/frame/methods/forbidden_aspects.rs:64`.

Existing tests (unexecuted; related/bounded only): `tests/forbidden_aspect_creation.rs:14`.

Cached contract: Page statement plus current producer cited above; no exact generated declaration anchor located for the entire combined statement.

Prior 12.1 cross-reference (status NOT reused): `data/patch-api/12.1-behaviors.json:332`.

### L82 `prose-2026-06-23-082` — modelable

Source: `UntrustedLayoutScriptExecution: When active, addon-installed OnSizeChanged handlers will never be run for a frame, its children, or any frames anchored to either.`

Assessment / ledger note: Native aspect masks/inheritance exist, but mask publication does not establish UntrustedScriptExecution or UntrustedLayoutScriptExecution suppression across all callback/anchor paths. Model caller-sensitive dispatch before claiming security.

Current producer or adjacent incomplete producer: `src/lua_api/frame/methods/forbidden_aspects.rs:64`.

Existing tests (unexecuted; related/bounded only): `tests/forbidden_aspect_creation.rs:14`.

Cached contract: Page statement plus current producer cited above; no exact generated declaration anchor located for the entire combined statement.

Prior 12.1 cross-reference (status NOT reused): `data/patch-api/12.1-behaviors.json:332`.

### L83 `prose-2026-06-23-083` — implemented-needs-proof

Source: `EventRegistrations: When active, addons cannot register a frame for events.`

Assessment / ledger note: RegisterEvent/related mutation paths reject the EventRegistrations aspect and preserve existing listeners; needs default Retail acceptance proof.

Current producer or adjacent incomplete producer: `src/lua_api/frame/methods/text_attribute_event/events.rs:26`.

Existing tests (unexecuted; related/bounded only): `tests/forbidden_aspect_creation.rs:155`, `tests/forbidden_aspect_creation.rs:190`.

Cached contract: Page statement plus current producer cited above; no exact generated declaration anchor located for the entire combined statement.

Prior 12.1 cross-reference (status NOT reused): `data/patch-api/12.1-behaviors.json:376`.

### L84 `prose-2026-06-23-084` — modelable

Source: `AlwaysPropagateInput: When active, a frame and its children will always propagate mouse and keyboard input.`

Assessment / ledger note: Click/focus guards and keyboard propagation exist; reconcile the complete mouse+keyboard contract, child inheritance and secure callers rather than treating isolated probes as all-input proof.

Current producer or adjacent incomplete producer: `src/lua_api/frame/methods/forbidden_aspects.rs:46`, `src/lua_api/frame/methods/widgets/editbox.rs:97`.

Existing tests (unexecuted; related/bounded only): `tests/forbidden_aspect_creation.rs:356`, `tests/keyboard.rs:227`.

Cached contract: Page statement plus current producer cited above; no exact generated declaration anchor located for the entire combined statement.

Prior 12.1 cross-reference (status NOT reused): `data/patch-api/12.1-behaviors.json:332`.

### L85 `prose-2026-06-23-085` — modelable

Source: `ScriptedInput: When active, addons are not allowed to call input-related APIs (Click, SetFocus, etc.) on a frame or its children.`

Assessment / ledger note: Click/focus guards and keyboard propagation exist; reconcile the complete mouse+keyboard contract, child inheritance and secure callers rather than treating isolated probes as all-input proof.

Current producer or adjacent incomplete producer: `src/lua_api/frame/methods/forbidden_aspects.rs:46`, `src/lua_api/frame/methods/widgets/editbox.rs:97`.

Existing tests (unexecuted; related/bounded only): `tests/forbidden_aspect_creation.rs:356`, `tests/keyboard.rs:227`.

Cached contract: Page statement plus current producer cited above; no exact generated declaration anchor located for the entire combined statement.

Prior 12.1 cross-reference (status NOT reused): `data/patch-api/12.1-behaviors.json:332`.

### L86 `prose-2026-06-23-086` — modelable

Source: `QueryFocus: When active, addons cannot query if a frame or its children are the current mouse or keyboard focus.`

Assessment / ledger note: Click/focus guards and keyboard propagation exist; reconcile the complete mouse+keyboard contract, child inheritance and secure callers rather than treating isolated probes as all-input proof.

Current producer or adjacent incomplete producer: `src/lua_api/frame/methods/forbidden_aspects.rs:46`, `src/lua_api/frame/methods/widgets/editbox.rs:97`.

Existing tests (unexecuted; related/bounded only): `tests/forbidden_aspect_creation.rs:356`, `tests/keyboard.rs:227`.

Cached contract: Page statement plus current producer cited above; no exact generated declaration anchor located for the entire combined statement.

Prior 12.1 cross-reference (status NOT reused): `data/patch-api/12.1-behaviors.json:332`.

### L87 `prose-2026-06-23-087` — modelable

Source: `Aura Buttons have had the following Forbidden Aspects applied to them: UntrustedScriptExecution, UntrustedLayoutScriptExecution, AlwaysPropagateInput, ScriptedInput, and QueryFocus`

Assessment / ledger note: Native aspect masks/inheritance exist, but mask publication does not establish UntrustedScriptExecution or UntrustedLayoutScriptExecution suppression across all callback/anchor paths. Model caller-sensitive dispatch before claiming security.

Current producer or adjacent incomplete producer: `src/lua_api/frame/methods/forbidden_aspects.rs:64`.

Existing tests (unexecuted; related/bounded only): `tests/forbidden_aspect_creation.rs:14`.

Cached contract: Page statement plus current producer cited above; no exact generated declaration anchor located for the entire combined statement.

Prior 12.1 cross-reference (status NOT reused): `data/patch-api/12.1-behaviors.json:332`.

### L88 `prose-2026-06-23-088` — implemented-needs-proof

Source: `Aura Containers have had the EventRegistrations Forbidden Aspect applied to them.`

Assessment / ledger note: RegisterEvent/related mutation paths reject the EventRegistrations aspect and preserve existing listeners; needs default Retail acceptance proof.

Current producer or adjacent incomplete producer: `src/lua_api/frame/methods/text_attribute_event/events.rs:26`.

Existing tests (unexecuted; related/bounded only): `tests/forbidden_aspect_creation.rs:155`, `tests/forbidden_aspect_creation.rs:190`.

Cached contract: Page statement plus current producer cited above; no exact generated declaration anchor located for the entire combined statement.

Prior 12.1 cross-reference (status NOT reused): `data/patch-api/12.1-behaviors.json:376`.

### L89 `prose-2026-06-23-089` — modelable

Source: `Editboxes will no longer auto-focus if they become visible while they have the Shown secret aspect applied.`

Assessment / ledger note: EditBox focus APIs exist, but visibility-driven auto-focus must respect the Shown secret aspect, including inherited state.

Current producer or adjacent incomplete producer: `src/lua_api/frame/methods/widgets/editbox.rs:1`.

Existing tests (unexecuted; related/bounded only): `tests/forbidden_aspect_creation.rs:379`.

Cached contract: Page statement plus current producer cited above; no exact generated declaration anchor located for the entire combined statement.

### L90 `prose-2026-06-23-090` — implemented-needs-proof

Source: `API calls such as SetParent and SetPoint will error if an object would implicitly gain any Forbidden Aspects that it does not already have.`

Assessment / ledger note: SetParent/SetPoint must reject implicit acquisition of additional forbidden aspects before mutation; current shared native guard implements that boundary.

Current producer or adjacent incomplete producer: `src/lua_api/frame/methods/forbidden_aspects.rs:138`.

Existing tests (unexecuted; related/bounded only): `tests/forbidden_aspect_creation.rs:14`.

Cached contract: Page statement plus current producer cited above; no exact generated declaration anchor located for the entire combined statement.

### L93 `source-context-093` — metadata-only

Source: `=== 2026-06-30 ===`

Assessment: Editorial heading, chronology, navigation, file label or disclaimer only; no runtime credit.

### L94 `source-context-094` — metadata-only

Source: `Midnight 12.1.0 PTR Changes 3 (Build 68412)`

Assessment: Editorial heading, chronology, navigation, file label or disclaimer only; no runtime credit.

### L95 `source-context-095` — metadata-only

Source: `Coming in 12.1.0 PTR 3`

Assessment: Editorial heading, chronology, navigation, file label or disclaimer only; no runtime credit.

### L97 `prose-2026-06-30-097` — modelable

Source: `This week brings the majority of the UnitAura API restrictions (with a few small pieces still remaining). Broadly speaking you can consider APIs that return aura data are no longer safe for addon use while aura data is secret. More specifically:`

Assessment / ledger note: Existing aura queries/filtering do not supply complete Retail caller-sensitive secret vectors, restricted index/slot/instance access, fully secret AuraData and UNIT_AURA payloads. Model these outputs and error boundaries over explicit aura-secret context.

Current producer or adjacent incomplete producer: `src/lua_api/globals/auras.rs:420`, `src/c_api/c_unit_aura_filter_query.rs:22`.

Existing tests (unexecuted; related/bounded only): `tests/unit_aura_slot_secret_arguments.rs:189`, `tests/tooltip_aura_instance_security.rs:1`.

Cached contract: Page statement plus current producer cited above; no exact generated declaration anchor located for the entire combined statement.

Contract qualification: INFERRED only for host context injection; public policy comes from this page and current declarations, and may differ between historical PTR weeks.

Prior 12.1 cross-reference (status NOT reused): `data/patch-api/12.1-behaviors.json:24`.

### L98 `prose-2026-06-30-098` — modelable

Source: `C_UnitAura and C_TooltipInfo APIs that provide access to aura data via index, slot, or instance ID will Lua error when called by addons while auras are secret.`

Assessment / ledger note: Existing aura queries/filtering do not supply complete Retail caller-sensitive secret vectors, restricted index/slot/instance access, fully secret AuraData and UNIT_AURA payloads. Model these outputs and error boundaries over explicit aura-secret context.

Current producer or adjacent incomplete producer: `src/lua_api/globals/auras.rs:420`, `src/c_api/c_unit_aura_filter_query.rs:22`.

Existing tests (unexecuted; related/bounded only): `tests/unit_aura_slot_secret_arguments.rs:189`, `tests/tooltip_aura_instance_security.rs:1`.

Cached contract: Page statement plus current producer cited above; no exact generated declaration anchor located for the entire combined statement.

Contract qualification: INFERRED only for host context injection; public policy comes from this page and current declarations, and may differ between historical PTR weeks.

Prior 12.1 cross-reference (status NOT reused): `data/patch-api/12.1-behaviors.json:24`.

### L99 `prose-2026-06-30-099` — modelable

Source: `C_UnitAura APIs that provide access to aura data via spell ID or spell name can still be called by addons as before (non-secret spells still return non-secrets).`

Assessment / ledger note: Existing aura queries/filtering do not supply complete Retail caller-sensitive secret vectors, restricted index/slot/instance access, fully secret AuraData and UNIT_AURA payloads. Model these outputs and error boundaries over explicit aura-secret context.

Current producer or adjacent incomplete producer: `src/lua_api/globals/auras.rs:420`, `src/c_api/c_unit_aura_filter_query.rs:22`.

Existing tests (unexecuted; related/bounded only): `tests/unit_aura_slot_secret_arguments.rs:189`, `tests/tooltip_aura_instance_security.rs:1`.

Cached contract: Page statement plus current producer cited above; no exact generated declaration anchor located for the entire combined statement.

Contract qualification: INFERRED only for host context injection; public policy comes from this page and current declarations, and may differ between historical PTR weeks.

Prior 12.1 cross-reference (status NOT reused): `data/patch-api/12.1-behaviors.json:24`.

### L100 `prose-2026-06-30-100` — modelable

Source: `The UNIT_AURA event now delivers a fully secret payload while auras are secret. AuraData structs are now always fully secret.`

Assessment / ledger note: Existing aura queries/filtering do not supply complete Retail caller-sensitive secret vectors, restricted index/slot/instance access, fully secret AuraData and UNIT_AURA payloads. Model these outputs and error boundaries over explicit aura-secret context.

Current producer or adjacent incomplete producer: `src/lua_api/globals/auras.rs:420`, `src/c_api/c_unit_aura_filter_query.rs:22`.

Existing tests (unexecuted; related/bounded only): `tests/unit_aura_slot_secret_arguments.rs:189`, `tests/tooltip_aura_instance_security.rs:1`.

Cached contract: Page statement plus current producer cited above; no exact generated declaration anchor located for the entire combined statement.

Contract qualification: INFERRED only for host context injection; public policy comes from this page and current declarations, and may differ between historical PTR weeks.

Prior 12.1 cross-reference (status NOT reused): `data/patch-api/12.1-behaviors.json:24`.

### L101 `prose-2026-06-30-101` — modelable

Source: `Added a new ManagedAuraContainer base type, which fully manages the display and layout of AuraButtons.`

Assessment / ledger note: Current unmodified cached Lua has managed group/slot/filter/layout logic. Native filter/secrecy/lifecycle integration is incomplete; existing mock-provider tests are narrower than real addon creation and managed frame updates.

Current producer or adjacent incomplete producer: `CACHE/Blizzard_AuraContainer/Blizzard_CustomAuraContainer.lua:283`, `src/lua_api/globals/auras.rs:224`.

Existing tests (unexecuted; related/bounded only): `patch-tests/patch_12_1/aura_container.rs:67`, `tests/on_update_modes.rs:152`.

Cached contract: Page statement plus current producer cited above; no exact generated declaration anchor located for the entire combined statement.

Prior 12.1 cross-reference (status NOT reused): `data/patch-api/12.1-behaviors.json:656`.

### L102 `prose-2026-06-30-102` — modelable

Source: `The Blizzard Target Frame now uses a ManagedAuraContainer for the display of its auras.`

Assessment / ledger note: Current unmodified cached Lua has managed group/slot/filter/layout logic. Native filter/secrecy/lifecycle integration is incomplete; existing mock-provider tests are narrower than real addon creation and managed frame updates.

Current producer or adjacent incomplete producer: `CACHE/Blizzard_AuraContainer/Blizzard_CustomAuraContainer.lua:283`, `src/lua_api/globals/auras.rs:224`.

Existing tests (unexecuted; related/bounded only): `patch-tests/patch_12_1/aura_container.rs:67`, `tests/on_update_modes.rs:152`.

Cached contract: Page statement plus current producer cited above; no exact generated declaration anchor located for the entire combined statement.

Prior 12.1 cross-reference (status NOT reused): `data/patch-api/12.1-behaviors.json:656`.

### L103 `prose-2026-06-30-103` — modelable

Source: `Fixed a bug where only 14 of the 19 parameters were being passed to ChatFrame message event filter functions.`

Assessment / ledger note: Current Blizzard chat filter path needs a concrete 19-argument input/output probe; original 14-argument bug cannot be discharged from a callable symbol.

Current producer or adjacent incomplete producer: `src/lua_api/script_helpers.rs:1`.

Existing tests (unexecuted; related/bounded only): No exact behavioral test located for this row; add concrete contract assertions, not symbol-presence checks.

Cached contract: Page statement plus current producer cited above; no exact generated declaration anchor located for the entire combined statement.

Contract qualification: INFERRED: local synthetic event fixture only; source fixes argument count to 19.

### L104 `source-context-104` — metadata-only

Source: `Preview of PTR 4`

Assessment: Editorial heading, chronology, navigation, file label or disclaimer only; no runtime credit.

### L105 `source-context-105` — metadata-only

Source: `PTR 4 will add a whole swath of changes for AuraContainers and AuraButtons including:`

Assessment: Editorial heading, chronology, navigation, file label or disclaimer only; no runtime credit.

### L107 `prose-2026-06-30-107` — modelable

Source: `CustomAuraContainers will be converted to ManagedAuraContainers. As a result, AuraContainers will now handle the creation of AuraButtons entirely on their own.`

Assessment / ledger note: Current unmodified cached Lua has managed group/slot/filter/layout logic. Native filter/secrecy/lifecycle integration is incomplete; existing mock-provider tests are narrower than real addon creation and managed frame updates.

Current producer or adjacent incomplete producer: `CACHE/Blizzard_AuraContainer/Blizzard_CustomAuraContainer.lua:283`, `src/lua_api/globals/auras.rs:224`.

Existing tests (unexecuted; related/bounded only): `patch-tests/patch_12_1/aura_container.rs:67`, `tests/on_update_modes.rs:152`.

Cached contract: Page statement plus current producer cited above; no exact generated declaration anchor located for the entire combined statement.

Prior 12.1 cross-reference (status NOT reused): `data/patch-api/12.1-behaviors.json:656`.

### L108 `prose-2026-06-30-108` — modelable

Source: `AuraContainer support for filtering by Spell ID, dispel type, stealable, and max duration. Some filters will have restrictions - more details to come.`

Assessment / ledger note: Current unmodified cached Lua has managed group/slot/filter/layout logic. Native filter/secrecy/lifecycle integration is incomplete; existing mock-provider tests are narrower than real addon creation and managed frame updates.

Current producer or adjacent incomplete producer: `CACHE/Blizzard_AuraContainer/Blizzard_CustomAuraContainer.lua:283`, `src/lua_api/globals/auras.rs:224`.

Existing tests (unexecuted; related/bounded only): `patch-tests/patch_12_1/aura_container.rs:67`, `tests/on_update_modes.rs:152`.

Cached contract: Page statement plus current producer cited above; no exact generated declaration anchor located for the entire combined statement.

Prior 12.1 cross-reference (status NOT reused): `data/patch-api/12.1-behaviors.json:656`.

### L109 `prose-2026-06-30-109` — modelable

Source: `AuraContainer support for sorting (both sort rule and direction).`

Assessment / ledger note: Current unmodified cached Lua has managed group/slot/filter/layout logic. Native filter/secrecy/lifecycle integration is incomplete; existing mock-provider tests are narrower than real addon creation and managed frame updates.

Current producer or adjacent incomplete producer: `CACHE/Blizzard_AuraContainer/Blizzard_CustomAuraContainer.lua:283`, `src/lua_api/globals/auras.rs:224`.

Existing tests (unexecuted; related/bounded only): `patch-tests/patch_12_1/aura_container.rs:67`, `tests/on_update_modes.rs:152`.

Cached contract: Page statement plus current producer cited above; no exact generated declaration anchor located for the entire combined statement.

Prior 12.1 cross-reference (status NOT reused): `data/patch-api/12.1-behaviors.json:656`.

### L112 `source-context-112` — metadata-only

Source: `=== 2026-07-07 ===`

Assessment: Editorial heading, chronology, navigation, file label or disclaimer only; no runtime credit.

### L113 `source-context-113` — metadata-only

Source: `Midnight 12.1.0 PTR Changes 4 (Build 68569)`

Assessment: Editorial heading, chronology, navigation, file label or disclaimer only; no runtime credit.

### L114 `source-context-114` — metadata-only

Source: `Coming in 12.1.0 PTR 4`

Assessment: Editorial heading, chronology, navigation, file label or disclaimer only; no runtime credit.

### L115 `prose-2026-07-07-115` — blocked

Source: `This week brings some major changes to AuraContainers and AuraButtons.`

Assessment / ledger note: Blocked: narrative progress/exploit rationale has no specified observable Retail result; exact historical/native evidence is missing. No implementation batch.

Evidence: captured source L115; missing historical/native/source artifact stated above. No batch.

### L117 `prose-2026-07-07-117` — modelable

Source: `AuraContainers now handle the creation and anchoring of all AuraButtons inside of them entirely on their own.`

Assessment / ledger note: Current unmodified cached Lua has managed group/slot/filter/layout logic. Native filter/secrecy/lifecycle integration is incomplete; existing mock-provider tests are narrower than real addon creation and managed frame updates.

Current producer or adjacent incomplete producer: `CACHE/Blizzard_AuraContainer/Blizzard_CustomAuraContainer.lua:283`, `src/lua_api/globals/auras.rs:224`.

Existing tests (unexecuted; related/bounded only): `patch-tests/patch_12_1/aura_container.rs:67`, `tests/on_update_modes.rs:152`.

Cached contract: Page statement plus current producer cited above; no exact generated declaration anchor located for the entire combined statement.

Prior 12.1 cross-reference (status NOT reused): `data/patch-api/12.1-behaviors.json:656`.

### L118 `prose-2026-07-07-118` — modelable

Source: `Addons no longer create AuraButtons directly. The AddAuraFrame API has been removed.`

Assessment / ledger note: Current Retail managed creation supersedes direct AuraButton/AddAuraFrame publication; reconcile ordinary/raw lookup and actual cached templates. Original PTR producer and transition date remain unproved, but final Retail absence is modelable.

Current producer or adjacent incomplete producer: `src/ptr/compat_bootstrap.rs:51`, `src/lua_api/globals/utility_system_spell/mod.rs:194`, `CACHE/Blizzard_Deprecated/Shared/Deprecated_12_1_0.lua:9`.

Existing tests (unexecuted; related/bounded only): `patch-tests/patch_12_1/strict_removal_timing.rs:96`.

Cached contract: Page statement plus current producer cited above; no exact generated declaration anchor located for the entire combined statement.

Prior 12.1 cross-reference (status NOT reused): `data/patch-api/12.1-behaviors.json:1553`.

### L119 `prose-2026-07-07-119` — implemented-needs-proof

Source: `Added a new construct to AuraContainers: AuraGroups. Broadly speaking, you can think of an AuraGroup as a dynamic, self-managing collection of auras within an AuraContainer.`

Assessment / ledger note: Current CustomAuraContainer Lua stores/validates this group option and creates managed groups; existing provider tests are related but do not prove this exact addon-facing contract. Tests only: exercise the real template, its actual options and managed callbacks; no security or live-service credit.

Current producer or adjacent incomplete producer: `CACHE/Blizzard_AuraContainer/Blizzard_CustomAuraContainer.lua:283`, `src/lua_api/globals/auras.rs:224`.

Existing tests (unexecuted; related/bounded only): `patch-tests/patch_12_1/aura_container.rs:67`, `tests/on_update_modes.rs:152`.

Cached contract: Page statement plus current producer cited above; no exact generated declaration anchor located for the entire combined statement.

Prior 12.1 cross-reference (status NOT reused): `data/patch-api/12.1-behaviors.json:656`.

### L120 `prose-2026-07-07-120` — implemented-needs-proof

Source: `AuraContainers can have multiple AuraGroups, each with their own filters and settings.`

Assessment / ledger note: Current CustomAuraContainer Lua stores/validates this group option and creates managed groups; existing provider tests are related but do not prove this exact addon-facing contract. Tests only: exercise the real template, its actual options and managed callbacks; no security or live-service credit.

Current producer or adjacent incomplete producer: `CACHE/Blizzard_AuraContainer/Blizzard_CustomAuraContainer.lua:283`, `src/lua_api/globals/auras.rs:224`.

Existing tests (unexecuted; related/bounded only): `patch-tests/patch_12_1/aura_container.rs:67`, `tests/on_update_modes.rs:152`.

Cached contract: Page statement plus current producer cited above; no exact generated declaration anchor located for the entire combined statement.

Prior 12.1 cross-reference (status NOT reused): `data/patch-api/12.1-behaviors.json:656`.

### L121 `prose-2026-07-07-121` — modelable

Source: `Auras from each group are anchored sequentially in the order the groups were added.`

Assessment / ledger note: Cached Lua implements group flow layout and resize. Native forbidden layout notifications and real managed-frame resize/anchor integration require modeling and behavioral proof.

Current producer or adjacent incomplete producer: `CACHE/Blizzard_AuraContainer/Blizzard_CustomAuraContainer.lua:648`, `src/lua_api/frame/methods/forbidden_aspects.rs:73`.

Existing tests (unexecuted; related/bounded only): `tests/on_update_modes.rs:152`.

Cached contract: Page statement plus current producer cited above; no exact generated declaration anchor located for the entire combined statement.

Prior 12.1 cross-reference (status NOT reused): `data/patch-api/12.1-behaviors.json:704`.

### L122 `prose-2026-07-07-122` — implemented-needs-proof

Source: `Addons add AuraGroups to AuraContainers using a new API AddAuraGroup(groupKey, filterString, options).`

Assessment / ledger note: Current CustomAuraContainer Lua stores/validates this group option and creates managed groups; existing provider tests are related but do not prove this exact addon-facing contract. Tests only: exercise the real template, its actual options and managed callbacks; no security or live-service credit.

Current producer or adjacent incomplete producer: `CACHE/Blizzard_AuraContainer/Blizzard_CustomAuraContainer.lua:283`, `src/lua_api/globals/auras.rs:224`.

Existing tests (unexecuted; related/bounded only): `patch-tests/patch_12_1/aura_container.rs:67`, `tests/on_update_modes.rs:152`.

Cached contract: Page statement plus current producer cited above; no exact generated declaration anchor located for the entire combined statement.

Prior 12.1 cross-reference (status NOT reused): `data/patch-api/12.1-behaviors.json:656`.

### L123 `prose-2026-07-07-123` — implemented-needs-proof

Source: `The groupKey param is an arbitrary addon-defined string used to access the group after creation.`

Assessment / ledger note: Current CustomAuraContainer Lua stores/validates this group option and creates managed groups; existing provider tests are related but do not prove this exact addon-facing contract. Tests only: exercise the real template, its actual options and managed callbacks; no security or live-service credit.

Current producer or adjacent incomplete producer: `CACHE/Blizzard_AuraContainer/Blizzard_CustomAuraContainer.lua:283`, `src/lua_api/globals/auras.rs:224`.

Existing tests (unexecuted; related/bounded only): `patch-tests/patch_12_1/aura_container.rs:67`, `tests/on_update_modes.rs:152`.

Cached contract: Page statement plus current producer cited above; no exact generated declaration anchor located for the entire combined statement.

Prior 12.1 cross-reference (status NOT reused): `data/patch-api/12.1-behaviors.json:656`.

### L124 `prose-2026-07-07-124` — modelable

Source: `The filterString param is a standard aura filter string as used today (e.g. "HELPFUL|RAID").`

Assessment / ledger note: Current native filter parser handles polarity and PLAYER only. Add negation, DISPELLABLE/IMPORTANT and raid-dispel eligibility using explicit aura/raid state; do not return plausible defaults.

Current producer or adjacent incomplete producer: `src/lua_api/globals/auras.rs:224`.

Existing tests (unexecuted; related/bounded only): `tests/unit_aura_filter_query.rs:148`.

Cached contract: Page statement plus current producer cited above; no exact generated declaration anchor located for the entire combined statement.

Contract qualification: INFERRED: ordering of combined unknown/negated filters and synthetic raid-dispel fixtures; preserve declared token names.

### L125 `prose-2026-07-07-125` — implemented-needs-proof

Source: `The options param is a table that can contain a number of optional settings.`

Assessment / ledger note: Current CustomAuraContainer Lua stores/validates this group option and creates managed groups; existing provider tests are related but do not prove this exact addon-facing contract. Tests only: exercise the real template, its actual options and managed callbacks; no security or live-service credit.

Current producer or adjacent incomplete producer: `CACHE/Blizzard_AuraContainer/Blizzard_CustomAuraContainer.lua:283`, `src/lua_api/globals/auras.rs:224`.

Existing tests (unexecuted; related/bounded only): `patch-tests/patch_12_1/aura_container.rs:67`, `tests/on_update_modes.rs:152`.

Cached contract: Page statement plus current producer cited above; no exact generated declaration anchor located for the entire combined statement.

Prior 12.1 cross-reference (status NOT reused): `data/patch-api/12.1-behaviors.json:656`.

### L126 `prose-2026-07-07-126` — implemented-needs-proof

Source: `maxFrameCount: The maximum number of aura frames to show in this group`

Assessment / ledger note: Current CustomAuraContainer Lua stores/validates this group option and creates managed groups; existing provider tests are related but do not prove this exact addon-facing contract. Tests only: exercise the real template, its actual options and managed callbacks; no security or live-service credit.

Current producer or adjacent incomplete producer: `CACHE/Blizzard_AuraContainer/Blizzard_CustomAuraContainer.lua:283`, `src/lua_api/globals/auras.rs:224`.

Existing tests (unexecuted; related/bounded only): `patch-tests/patch_12_1/aura_container.rs:67`, `tests/on_update_modes.rs:152`.

Cached contract: Page statement plus current producer cited above; no exact generated declaration anchor located for the entire combined statement.

Prior 12.1 cross-reference (status NOT reused): `data/patch-api/12.1-behaviors.json:656`.

### L127 `prose-2026-07-07-127` — implemented-needs-proof

Source: `sortMethod and sortDirection: Used to control how auras in this group are sorted (see new enum AuraContainerSortMethod for choices).`

Assessment / ledger note: Current CustomAuraContainer Lua stores/validates this group option and creates managed groups; existing provider tests are related but do not prove this exact addon-facing contract. Tests only: exercise the real template, its actual options and managed callbacks; no security or live-service credit.

Current producer or adjacent incomplete producer: `CACHE/Blizzard_AuraContainer/Blizzard_CustomAuraContainer.lua:283`, `src/lua_api/globals/auras.rs:224`.

Existing tests (unexecuted; related/bounded only): `patch-tests/patch_12_1/aura_container.rs:67`, `tests/on_update_modes.rs:152`.

Cached contract: Page statement plus current producer cited above; no exact generated declaration anchor located for the entire combined statement.

Prior 12.1 cross-reference (status NOT reused): `data/patch-api/12.1-behaviors.json:656`.

### L128 `prose-2026-07-07-128` — implemented-needs-proof

Source: `initializeFrame: A callback function that is called for each AuraButton created.`

Assessment / ledger note: Current CustomAuraContainer Lua stores/validates this group option and creates managed groups; existing provider tests are related but do not prove this exact addon-facing contract. Tests only: exercise the real template, its actual options and managed callbacks; no security or live-service credit.

Current producer or adjacent incomplete producer: `CACHE/Blizzard_AuraContainer/Blizzard_CustomAuraContainer.lua:283`, `src/lua_api/globals/auras.rs:224`.

Existing tests (unexecuted; related/bounded only): `patch-tests/patch_12_1/aura_container.rs:67`, `tests/on_update_modes.rs:152`.

Cached contract: Page statement plus current producer cited above; no exact generated declaration anchor located for the entire combined statement.

Prior 12.1 cross-reference (status NOT reused): `data/patch-api/12.1-behaviors.json:656`.

### L129 `prose-2026-07-07-129` — implemented-needs-proof

Source: `templateNames: A list of xml templates to apply to each AuraButton (in addition to CustomAuraButtonTemplate).`

Assessment / ledger note: Current CustomAuraContainer Lua stores/validates this group option and creates managed groups; existing provider tests are related but do not prove this exact addon-facing contract. Tests only: exercise the real template, its actual options and managed callbacks; no security or live-service credit.

Current producer or adjacent incomplete producer: `CACHE/Blizzard_AuraContainer/Blizzard_CustomAuraContainer.lua:283`, `src/lua_api/globals/auras.rs:224`.

Existing tests (unexecuted; related/bounded only): `patch-tests/patch_12_1/aura_container.rs:67`, `tests/on_update_modes.rs:152`.

Cached contract: Page statement plus current producer cited above; no exact generated declaration anchor located for the entire combined statement.

Prior 12.1 cross-reference (status NOT reused): `data/patch-api/12.1-behaviors.json:656`.

### L130 `prose-2026-07-07-130` — modelable

Source: `candidateFilters: A table of additional filter information to apply when determining if an aura should be displayed. See ValidateCandidateFilters for the full list of options, but some examples are:`

Assessment / ledger note: Current unmodified cached Lua has managed group/slot/filter/layout logic. Native filter/secrecy/lifecycle integration is incomplete; existing mock-provider tests are narrower than real addon creation and managed frame updates.

Current producer or adjacent incomplete producer: `CACHE/Blizzard_AuraContainer/Blizzard_CustomAuraContainer.lua:283`, `src/lua_api/globals/auras.rs:224`.

Existing tests (unexecuted; related/bounded only): `patch-tests/patch_12_1/aura_container.rs:67`, `tests/on_update_modes.rs:152`.

Cached contract: Page statement plus current producer cited above; no exact generated declaration anchor located for the entire combined statement.

Prior 12.1 cross-reference (status NOT reused): `data/patch-api/12.1-behaviors.json:656`.

### L131 `prose-2026-07-07-131` — modelable

Source: `Include/exclude maps for spell IDs and dispel types.`

Assessment / ledger note: Current unmodified cached Lua has managed group/slot/filter/layout logic. Native filter/secrecy/lifecycle integration is incomplete; existing mock-provider tests are narrower than real addon creation and managed frame updates.

Current producer or adjacent incomplete producer: `CACHE/Blizzard_AuraContainer/Blizzard_CustomAuraContainer.lua:283`, `src/lua_api/globals/auras.rs:224`.

Existing tests (unexecuted; related/bounded only): `patch-tests/patch_12_1/aura_container.rs:67`, `tests/on_update_modes.rs:152`.

Cached contract: Page statement plus current producer cited above; no exact generated declaration anchor located for the entire combined statement.

Prior 12.1 cross-reference (status NOT reused): `data/patch-api/12.1-behaviors.json:656`.

### L132 `prose-2026-07-07-132` — modelable

Source: `maxDuration`

Assessment / ledger note: Current unmodified cached Lua has managed group/slot/filter/layout logic. Native filter/secrecy/lifecycle integration is incomplete; existing mock-provider tests are narrower than real addon creation and managed frame updates.

Current producer or adjacent incomplete producer: `CACHE/Blizzard_AuraContainer/Blizzard_CustomAuraContainer.lua:283`, `src/lua_api/globals/auras.rs:224`.

Existing tests (unexecuted; related/bounded only): `patch-tests/patch_12_1/aura_container.rs:67`, `tests/on_update_modes.rs:152`.

Cached contract: Page statement plus current producer cited above; no exact generated declaration anchor located for the entire combined statement.

Prior 12.1 cross-reference (status NOT reused): `data/patch-api/12.1-behaviors.json:656`.

### L133 `prose-2026-07-07-133` — modelable

Source: `Various boolean values from AuraData (isFromPlayerOrPlayerPet, isRoleAura, isPriorityAura, isStealable, etc.).`

Assessment / ledger note: Current unmodified cached Lua has managed group/slot/filter/layout logic. Native filter/secrecy/lifecycle integration is incomplete; existing mock-provider tests are narrower than real addon creation and managed frame updates.

Current producer or adjacent incomplete producer: `CACHE/Blizzard_AuraContainer/Blizzard_CustomAuraContainer.lua:283`, `src/lua_api/globals/auras.rs:224`.

Existing tests (unexecuted; related/bounded only): `patch-tests/patch_12_1/aura_container.rs:67`, `tests/on_update_modes.rs:152`.

Cached contract: Page statement plus current producer cited above; no exact generated declaration anchor located for the entire combined statement.

Prior 12.1 cross-reference (status NOT reused): `data/patch-api/12.1-behaviors.json:656`.

### L134 `prose-2026-07-07-134` — modelable

Source: `AuraGroups create and anchor AuraButtons in batches of 10 as needed.`

Assessment / ledger note: Current unmodified cached Lua has managed group/slot/filter/layout logic. Native filter/secrecy/lifecycle integration is incomplete; existing mock-provider tests are narrower than real addon creation and managed frame updates.

Current producer or adjacent incomplete producer: `CACHE/Blizzard_AuraContainer/Blizzard_CustomAuraContainer.lua:283`, `src/lua_api/globals/auras.rs:224`.

Existing tests (unexecuted; related/bounded only): `patch-tests/patch_12_1/aura_container.rs:67`, `tests/on_update_modes.rs:152`.

Cached contract: Page statement plus current producer cited above; no exact generated declaration anchor located for the entire combined statement.

Prior 12.1 cross-reference (status NOT reused): `data/patch-api/12.1-behaviors.json:656`.

### L135 `prose-2026-07-07-135` — modelable

Source: `Anchoring of AuraButtons created by an AuraContainer can be adjusted via the SetAuraGroupLayout API (see ValidateAuraGroupLayoutOptions for available options).`

Assessment / ledger note: Cached Lua implements group flow layout and resize. Native forbidden layout notifications and real managed-frame resize/anchor integration require modeling and behavioral proof.

Current producer or adjacent incomplete producer: `CACHE/Blizzard_AuraContainer/Blizzard_CustomAuraContainer.lua:648`, `src/lua_api/frame/methods/forbidden_aspects.rs:73`.

Existing tests (unexecuted; related/bounded only): `tests/on_update_modes.rs:152`.

Cached contract: Page statement plus current producer cited above; no exact generated declaration anchor located for the entire combined statement.

Prior 12.1 cross-reference (status NOT reused): `data/patch-api/12.1-behaviors.json:704`.

### L136 `prose-2026-07-07-136` — modelable

Source: `AuraContainers now automatically resize to fit group-based AuraButtons inside of them.`

Assessment / ledger note: Cached Lua implements group flow layout and resize. Native forbidden layout notifications and real managed-frame resize/anchor integration require modeling and behavioral proof.

Current producer or adjacent incomplete producer: `CACHE/Blizzard_AuraContainer/Blizzard_CustomAuraContainer.lua:648`, `src/lua_api/frame/methods/forbidden_aspects.rs:73`.

Existing tests (unexecuted; related/bounded only): `tests/on_update_modes.rs:152`.

Cached contract: Page statement plus current producer cited above; no exact generated declaration anchor located for the entire combined statement.

Prior 12.1 cross-reference (status NOT reused): `data/patch-api/12.1-behaviors.json:704`.

### L137 `prose-2026-07-07-137` — modelable

Source: `AuraContainers now treat private auras just like regular auras, allowing them to be shown and sorted normally.`

Assessment / ledger note: Current unmodified cached Lua has managed group/slot/filter/layout logic. Native filter/secrecy/lifecycle integration is incomplete; existing mock-provider tests are narrower than real addon creation and managed frame updates.

Current producer or adjacent incomplete producer: `CACHE/Blizzard_AuraContainer/Blizzard_CustomAuraContainer.lua:283`, `src/lua_api/globals/auras.rs:224`.

Existing tests (unexecuted; related/bounded only): `patch-tests/patch_12_1/aura_container.rs:67`, `tests/on_update_modes.rs:152`.

Cached contract: Page statement plus current producer cited above; no exact generated declaration anchor located for the entire combined statement.

Prior 12.1 cross-reference (status NOT reused): `data/patch-api/12.1-behaviors.json:656`.

### L138 `prose-2026-07-07-138` — modelable

Source: `Added a new construct to AuraContainers: AuraSlots. You can think of AuraSlots as AuraGroups with maxFrameCount set to 1 (they will only ever show a single aura).`

Assessment / ledger note: Current unmodified cached Lua has managed group/slot/filter/layout logic. Native filter/secrecy/lifecycle integration is incomplete; existing mock-provider tests are narrower than real addon creation and managed frame updates.

Current producer or adjacent incomplete producer: `CACHE/Blizzard_AuraContainer/Blizzard_CustomAuraContainer.lua:283`, `src/lua_api/globals/auras.rs:224`.

Existing tests (unexecuted; related/bounded only): `patch-tests/patch_12_1/aura_container.rs:67`, `tests/on_update_modes.rs:152`.

Cached contract: Page statement plus current producer cited above; no exact generated declaration anchor located for the entire combined statement.

Prior 12.1 cross-reference (status NOT reused): `data/patch-api/12.1-behaviors.json:656`.

### L139 `prose-2026-07-07-139` — modelable

Source: `Unlike AuraGroups, addons can manually anchor AuraSlots.`

Assessment / ledger note: Cached Lua implements group flow layout and resize. Native forbidden layout notifications and real managed-frame resize/anchor integration require modeling and behavioral proof.

Current producer or adjacent incomplete producer: `CACHE/Blizzard_AuraContainer/Blizzard_CustomAuraContainer.lua:648`, `src/lua_api/frame/methods/forbidden_aspects.rs:73`.

Existing tests (unexecuted; related/bounded only): `tests/on_update_modes.rs:152`.

Cached contract: Page statement plus current producer cited above; no exact generated declaration anchor located for the entire combined statement.

Prior 12.1 cross-reference (status NOT reused): `data/patch-api/12.1-behaviors.json:704`.

### L140 `prose-2026-07-07-140` — modelable

Source: `The AddAuraSlot(slotKey, filterString, options) API is used to add AuraSlots, and it supports most of the same options AddAuraGroup does.`

Assessment / ledger note: Current unmodified cached Lua has managed group/slot/filter/layout logic. Native filter/secrecy/lifecycle integration is incomplete; existing mock-provider tests are narrower than real addon creation and managed frame updates.

Current producer or adjacent incomplete producer: `CACHE/Blizzard_AuraContainer/Blizzard_CustomAuraContainer.lua:283`, `src/lua_api/globals/auras.rs:224`.

Existing tests (unexecuted; related/bounded only): `patch-tests/patch_12_1/aura_container.rs:67`, `tests/on_update_modes.rs:152`.

Cached contract: Page statement plus current producer cited above; no exact generated declaration anchor located for the entire combined statement.

Prior 12.1 cross-reference (status NOT reused): `data/patch-api/12.1-behaviors.json:656`.

### L141 `prose-2026-07-07-141` — modelable

Source: `Added a new API, AddItemEnchantment(itemEnchantmentSlot, options), to AuraContainers, which allows them to show temporary weapon enchants. See ValidateAddItemEnchantmentOptions for the list of options supported.`

Assessment / ledger note: Cached AddItemEnchantment and cancel handling exist; connect explicit temporary weapon-enchant state to managed frames and click routing.

Current producer or adjacent incomplete producer: `CACHE/Blizzard_AuraContainer/Blizzard_CustomAuraContainer.lua:450`, `src/c_api/weapon_enchants.rs:1`.

Existing tests (unexecuted; related/bounded only): `tests/on_update_modes.rs:152`.

Cached contract: Page statement plus current producer cited above; no exact generated declaration anchor located for the entire combined statement.

Contract qualification: INFERRED: deterministic enchant fixtures; no native cancellation/service timing claim.

### L142 `prose-2026-07-07-142` — modelable

Source: `Added a new API SetCancelAuraButtons to AuraButtons that can be used to specify which mouse clicks to use to cancel. This can be called on AuraButtons via the initializeFrame callback.`

Assessment / ledger note: Cached custom button Lua has presentation setters and aura update handlers; real native aura data, secured region ownership and presentation updates remain incomplete. Exercise actual templates, not table-only substitutes.

Current producer or adjacent incomplete producer: `CACHE/Blizzard_AuraContainer/Blizzard_CustomAuraButton.lua:524`, `src/c_api/c_aura_container_util.rs:228`.

Existing tests (unexecuted; related/bounded only): `tests/aura_container_util.rs:6`, `patch-tests/patch_12_1/aura_tooltip.rs:34`.

Cached contract: Page statement plus current producer cited above; no exact generated declaration anchor located for the entire combined statement.

Prior 12.1 cross-reference (status NOT reused): `data/patch-api/12.1-behaviors.json:752`.

### L143 `source-context-143` — metadata-only

Source: `Other changes`

Assessment: Editorial heading, chronology, navigation, file label or disclaimer only; no runtime credit.

### L145 `prose-2026-07-07-145` — modelable

Source: `The AddPrivateAuraAppliedSound and RemovePrivateAuraAppliedSound APIs have been renamed to AddAuraAppliedSound and RemoveAuraAppliedSound, and now work on any auras (not just private auras).`

Assessment / ledger note: Existing private aura sound model and AddAuraSound entry point are relevant, but all-aura/application/removal triggers and successive renamed aliases need their own delta model; inherited add-context capability alone cannot close this row.

Current producer or adjacent incomplete producer: `src/c_api/private_aura_sounds.rs:1`.

Existing tests (unexecuted; related/bounded only): `tests/private_aura_sound_add_context.rs:51`, `tests/private_aura_sound_removal.rs:154`.

Cached contract: Page statement plus current producer cited above; no exact generated declaration anchor located for the entire combined statement.

### L146 `prose-2026-07-07-146` — modelable

Source: `Added support for negating most aura filters using the ! character. So for instance !PLAYER includes only auras NOT cast by the player.`

Assessment / ledger note: Current native filter parser handles polarity and PLAYER only. Add negation, DISPELLABLE/IMPORTANT and raid-dispel eligibility using explicit aura/raid state; do not return plausible defaults.

Current producer or adjacent incomplete producer: `src/lua_api/globals/auras.rs:224`.

Existing tests (unexecuted; related/bounded only): `tests/unit_aura_filter_query.rs:148`.

Cached contract: Page statement plus current producer cited above; no exact generated declaration anchor located for the entire combined statement.

Contract qualification: INFERRED: ordering of combined unknown/negated filters and synthetic raid-dispel fixtures; preserve declared token names.

### L147 `prose-2026-07-07-147` — modelable

Source: `The UNIT_AURA event now delivers a fully secret payload while auras are secret. AuraData structs are now always fully secret.`

Assessment / ledger note: Existing aura queries/filtering do not supply complete Retail caller-sensitive secret vectors, restricted index/slot/instance access, fully secret AuraData and UNIT_AURA payloads. Model these outputs and error boundaries over explicit aura-secret context.

Current producer or adjacent incomplete producer: `src/lua_api/globals/auras.rs:420`, `src/c_api/c_unit_aura_filter_query.rs:22`.

Existing tests (unexecuted; related/bounded only): `tests/unit_aura_slot_secret_arguments.rs:189`, `tests/tooltip_aura_instance_security.rs:1`.

Cached contract: Page statement plus current producer cited above; no exact generated declaration anchor located for the entire combined statement.

Contract qualification: INFERRED only for host context injection; public policy comes from this page and current declarations, and may differ between historical PTR weeks.

Prior 12.1 cross-reference (status NOT reused): `data/patch-api/12.1-behaviors.json:24`.

### L148 `prose-2026-07-07-148` — modelable

Source: `SecureAuraHeaderTemplate has been removed from Mainline (it will still exist for Classic). Addons still using SecureAuraHeaderTemplate should migrate over to using AuraContainers.`

Assessment / ledger note: Audit current cached deprecated wrappers and default Retail strict-removal timing. A historical PTR symbol observation is not Retail migration acceptance.

Current producer or adjacent incomplete producer: `src/ptr/compat_bootstrap.rs:51`, `src/lua_api/globals/utility_system_spell/mod.rs:194`, `CACHE/Blizzard_Deprecated/Shared/Deprecated_12_1_0.lua:9`.

Existing tests (unexecuted; related/bounded only): `patch-tests/patch_12_1/strict_removal_timing.rs:96`.

Cached contract: Page statement plus current producer cited above; no exact generated declaration anchor located for the entire combined statement.

Prior 12.1 cross-reference (status NOT reused): `data/patch-api/12.1-behaviors.json:1553`.

### L149 `prose-2026-07-07-149` — blocked

Source: `Fixed a crash that happened when attempting to create an AuraContainer in combat.`

Assessment / ledger note: Blocked: PTR4 AuraContainer-in-combat crash/error was superseded by PTR7 allowing creation; current Retail cannot prove the original PTR4 native failure. Exact original build/native trace is missing.

Evidence: captured source L149; missing historical/native/source artifact stated above. No batch.

### L150 `prose-2026-07-07-150` — blocked

Source: `Attempting to do so will still generate a Lua error, however (as intended).`

Assessment / ledger note: Blocked: PTR4 AuraContainer-in-combat crash/error was superseded by PTR7 allowing creation; current Retail cannot prove the original PTR4 native failure. Exact original build/native trace is missing.

Evidence: captured source L150; missing historical/native/source artifact stated above. No batch.

### L151 `prose-2026-07-07-151` — modelable

Source: `Added a new SecureGroupHeaderTemplate xml template that can be used to safely create a single AuraContainer on UnitFrame creation.`

Assessment / ledger note: Audit current cached deprecated wrappers and default Retail strict-removal timing. A historical PTR symbol observation is not Retail migration acceptance.

Current producer or adjacent incomplete producer: `src/ptr/compat_bootstrap.rs:51`, `src/lua_api/globals/utility_system_spell/mod.rs:194`, `CACHE/Blizzard_Deprecated/Shared/Deprecated_12_1_0.lua:9`.

Existing tests (unexecuted; related/bounded only): `patch-tests/patch_12_1/strict_removal_timing.rs:96`.

Cached contract: Page statement plus current producer cited above; no exact generated declaration anchor located for the entire combined statement.

Prior 12.1 cross-reference (status NOT reused): `data/patch-api/12.1-behaviors.json:1553`.

### L152 `prose-2026-07-07-152` — modelable

Source: `Resolved an issue where the SetApplicationCount function on CustomAuraButtons would error if not supplied an options table.`

Assessment / ledger note: Cached custom button Lua has presentation setters and aura update handlers; real native aura data, secured region ownership and presentation updates remain incomplete. Exercise actual templates, not table-only substitutes.

Current producer or adjacent incomplete producer: `CACHE/Blizzard_AuraContainer/Blizzard_CustomAuraButton.lua:524`, `src/c_api/c_aura_container_util.rs:228`.

Existing tests (unexecuted; related/bounded only): `tests/aura_container_util.rs:6`, `patch-tests/patch_12_1/aura_tooltip.rs:34`.

Cached contract: Page statement plus current producer cited above; no exact generated declaration anchor located for the entire combined statement.

Prior 12.1 cross-reference (status NOT reused): `data/patch-api/12.1-behaviors.json:752`.

### L153 `prose-2026-07-07-153` — modelable

Source: `Resolved an issue where ApplyAuraSymbol on CustomAuraButtons was consulting the wrong region (AuraButton) for dispel type validation.`

Assessment / ledger note: Cached custom button Lua has presentation setters and aura update handlers; real native aura data, secured region ownership and presentation updates remain incomplete. Exercise actual templates, not table-only substitutes.

Current producer or adjacent incomplete producer: `CACHE/Blizzard_AuraContainer/Blizzard_CustomAuraButton.lua:524`, `src/c_api/c_aura_container_util.rs:228`.

Existing tests (unexecuted; related/bounded only): `tests/aura_container_util.rs:6`, `patch-tests/patch_12_1/aura_tooltip.rs:34`.

Cached contract: Page statement plus current producer cited above; no exact generated declaration anchor located for the entire combined statement.

Prior 12.1 cross-reference (status NOT reused): `data/patch-api/12.1-behaviors.json:752`.

### L156 `source-context-156` — metadata-only

Source: `=== 2026-07-14 ===`

Assessment: Editorial heading, chronology, navigation, file label or disclaimer only; no runtime credit.

### L157 `source-context-157` — metadata-only

Source: `Midnight 12.1.0 PTR Changes 5 (Build 68675)`

Assessment: Editorial heading, chronology, navigation, file label or disclaimer only; no runtime credit.

### L158 `source-context-158` — metadata-only

Source: `Coming in 12.1.0 PTR 5`

Assessment: Editorial heading, chronology, navigation, file label or disclaimer only; no runtime credit.

### L160 `prose-2026-07-14-160` — modelable

Source: `Added a new aura filter, DISPELLABLE, which returns auras that have a dispel type of any kind, regardless of whether anyone in the player's raid can dispel it.`

Assessment / ledger note: Current native filter parser handles polarity and PLAYER only. Add negation, DISPELLABLE/IMPORTANT and raid-dispel eligibility using explicit aura/raid state; do not return plausible defaults.

Current producer or adjacent incomplete producer: `src/lua_api/globals/auras.rs:224`.

Existing tests (unexecuted; related/bounded only): `tests/unit_aura_filter_query.rs:148`.

Cached contract: Page statement plus current producer cited above; no exact generated declaration anchor located for the entire combined statement.

Contract qualification: INFERRED: ordering of combined unknown/negated filters and synthetic raid-dispel fixtures; preserve declared token names.

### L161 `prose-2026-07-14-161` — modelable

Source: `Added back the IMPORTANT aura filter now that it is no longer abusable.`

Assessment / ledger note: Current native filter parser handles polarity and PLAYER only. Add negation, DISPELLABLE/IMPORTANT and raid-dispel eligibility using explicit aura/raid state; do not return plausible defaults.

Current producer or adjacent incomplete producer: `src/lua_api/globals/auras.rs:224`.

Existing tests (unexecuted; related/bounded only): `tests/unit_aura_filter_query.rs:148`.

Cached contract: Page statement plus current producer cited above; no exact generated declaration anchor located for the entire combined statement.

Contract qualification: INFERRED: ordering of combined unknown/negated filters and synthetic raid-dispel fixtures; preserve declared token names.

### L162 `prose-2026-07-14-162` — modelable

Source: `The RAID_PLAYER_DISPELLABLE aura filter now also returns helpful auras on enemies that are dispellable/stealable by a raid member.`

Assessment / ledger note: Current native filter parser handles polarity and PLAYER only. Add negation, DISPELLABLE/IMPORTANT and raid-dispel eligibility using explicit aura/raid state; do not return plausible defaults.

Current producer or adjacent incomplete producer: `src/lua_api/globals/auras.rs:224`.

Existing tests (unexecuted; related/bounded only): `tests/unit_aura_filter_query.rs:148`.

Cached contract: Page statement plus current producer cited above; no exact generated declaration anchor located for the entire combined statement.

Contract qualification: INFERRED: ordering of combined unknown/negated filters and synthetic raid-dispel fixtures; preserve declared token names.

### L163 `prose-2026-07-14-163` — modelable

Source: `AuraButtons are now forbidden (meaning APIs called on them via tainted code will Lua error) whenever auras are secret.`

Assessment / ledger note: Native aspect masks/inheritance exist, but mask publication does not establish UntrustedScriptExecution or UntrustedLayoutScriptExecution suppression across all callback/anchor paths. Model caller-sensitive dispatch before claiming security.

Current producer or adjacent incomplete producer: `src/lua_api/frame/methods/forbidden_aspects.rs:64`.

Existing tests (unexecuted; related/bounded only): `tests/forbidden_aspect_creation.rs:14`.

Cached contract: Page statement plus current producer cited above; no exact generated declaration anchor located for the entire combined statement.

Prior 12.1 cross-reference (status NOT reused): `data/patch-api/12.1-behaviors.json:332`.

### L164 `prose-2026-07-14-164` — modelable

Source: `This forbidden state is not applied until after the initializeFrame callback has been called, and AuraButtons return to a non-forbidden state when auras become non-secret again (outside combat, encounters, etc.).`

Assessment / ledger note: Native aspect masks/inheritance exist, but mask publication does not establish UntrustedScriptExecution or UntrustedLayoutScriptExecution suppression across all callback/anchor paths. Model caller-sensitive dispatch before claiming security.

Current producer or adjacent incomplete producer: `src/lua_api/frame/methods/forbidden_aspects.rs:64`.

Existing tests (unexecuted; related/bounded only): `tests/forbidden_aspect_creation.rs:14`.

Cached contract: Page statement plus current producer cited above; no exact generated declaration anchor located for the entire combined statement.

Prior 12.1 cross-reference (status NOT reused): `data/patch-api/12.1-behaviors.json:332`.

### L165 `prose-2026-07-14-165` — blocked

Source: `This change was necessary in order to prevent various exploits.`

Assessment / ledger note: Blocked: narrative progress/exploit rationale has no specified observable Retail result; exact historical/native evidence is missing. No implementation batch.

Evidence: captured source L165; missing historical/native/source artifact stated above. No batch.

### L166 `prose-2026-07-14-166` — modelable

Source: `It is no longer possible for addons to create new WorldFrame instances via CreateFrame (preventing a crash).`

Assessment / ledger note: WorldFrame has an engine-created object; generic CreateFrame path still needs the addon creation rejection contract rather than assuming the existing root disallows duplicates.

Current producer or adjacent incomplete producer: `src/lua_api/builtin_frames.rs:157`.

Existing tests (unexecuted; related/bounded only): `src/loader/tests/global_frame_access.rs:372`.

Cached contract: Page statement plus current producer cited above; no exact generated declaration anchor located for the entire combined statement.

Contract qualification: INFERRED: error wording/return shape; rejection requirement is explicit.

### L167 `prose-2026-07-14-167` — modelable

Source: `Made improvements to the error messaging displayed when attempting to call forbidden script APIs on script objects.`

Assessment / ledger note: Native aspect masks/inheritance exist, but mask publication does not establish UntrustedScriptExecution or UntrustedLayoutScriptExecution suppression across all callback/anchor paths. Model caller-sensitive dispatch before claiming security.

Current producer or adjacent incomplete producer: `src/lua_api/frame/methods/forbidden_aspects.rs:64`.

Existing tests (unexecuted; related/bounded only): `tests/forbidden_aspect_creation.rs:14`.

Cached contract: Page statement plus current producer cited above; no exact generated declaration anchor located for the entire combined statement.

Prior 12.1 cross-reference (status NOT reused): `data/patch-api/12.1-behaviors.json:332`.

### L170 `source-context-170` — metadata-only

Source: `=== 2026-07-21 ===`

Assessment: Editorial heading, chronology, navigation, file label or disclaimer only; no runtime credit.

### L171 `source-context-171` — metadata-only

Source: `Midnight 12.1.0 PTR Changes 6 (Build 68824)`

Assessment: Editorial heading, chronology, navigation, file label or disclaimer only; no runtime credit.

### L172 `source-context-172` — metadata-only

Source: `Coming in 12.1.0 PTR 6`

Assessment: Editorial heading, chronology, navigation, file label or disclaimer only; no runtime credit.

### L174 `prose-2026-07-21-174` — modelable

Source: `Added a new GetAuraGroupFrame API to aura containers, which can be used to retrieve a child aura group frame by index.`

Assessment / ledger note: Current unmodified cached Lua has managed group/slot/filter/layout logic. Native filter/secrecy/lifecycle integration is incomplete; existing mock-provider tests are narrower than real addon creation and managed frame updates.

Current producer or adjacent incomplete producer: `CACHE/Blizzard_AuraContainer/Blizzard_CustomAuraContainer.lua:283`, `src/lua_api/globals/auras.rs:224`.

Existing tests (unexecuted; related/bounded only): `patch-tests/patch_12_1/aura_container.rs:67`, `tests/on_update_modes.rs:152`.

Cached contract: Page statement plus current producer cited above; no exact generated declaration anchor located for the entire combined statement.

Prior 12.1 cross-reference (status NOT reused): `data/patch-api/12.1-behaviors.json:656`.

### L175 `prose-2026-07-21-175` — modelable

Source: `Added new ApplicationBar APIs to custom aura buttons that allow addons to show a status bar that tracks their number of applications.`

Assessment / ledger note: Cached custom button Lua has presentation setters and aura update handlers; real native aura data, secured region ownership and presentation updates remain incomplete. Exercise actual templates, not table-only substitutes.

Current producer or adjacent incomplete producer: `CACHE/Blizzard_AuraContainer/Blizzard_CustomAuraButton.lua:524`, `src/c_api/c_aura_container_util.rs:228`.

Existing tests (unexecuted; related/bounded only): `tests/aura_container_util.rs:6`, `patch-tests/patch_12_1/aura_tooltip.rs:34`.

Cached contract: Page statement plus current producer cited above; no exact generated declaration anchor located for the entire combined statement.

Prior 12.1 cross-reference (status NOT reused): `data/patch-api/12.1-behaviors.json:752`.

### L176 `prose-2026-07-21-176` — modelable

Source: `Added a new SetAuraGroupFilterString API to aura containers, which allows addons to set the filter string after creation.`

Assessment / ledger note: Current unmodified cached Lua has managed group/slot/filter/layout logic. Native filter/secrecy/lifecycle integration is incomplete; existing mock-provider tests are narrower than real addon creation and managed frame updates.

Current producer or adjacent incomplete producer: `CACHE/Blizzard_AuraContainer/Blizzard_CustomAuraContainer.lua:283`, `src/lua_api/globals/auras.rs:224`.

Existing tests (unexecuted; related/bounded only): `patch-tests/patch_12_1/aura_container.rs:67`, `tests/on_update_modes.rs:152`.

Cached contract: Page statement plus current producer cited above; no exact generated declaration anchor located for the entire combined statement.

Prior 12.1 cross-reference (status NOT reused): `data/patch-api/12.1-behaviors.json:656`.

### L177 `prose-2026-07-21-177` — modelable

Source: `Added support for color curves and color maps to the AuraButton:SetAuraBorder API (see CustomAuraButtonDispelTypeTextureOptions in the documentation files for details).`

Assessment / ledger note: Cached custom button Lua has presentation setters and aura update handlers; real native aura data, secured region ownership and presentation updates remain incomplete. Exercise actual templates, not table-only substitutes.

Current producer or adjacent incomplete producer: `CACHE/Blizzard_AuraContainer/Blizzard_CustomAuraButton.lua:524`, `src/c_api/c_aura_container_util.rs:228`.

Existing tests (unexecuted; related/bounded only): `tests/aura_container_util.rs:6`, `patch-tests/patch_12_1/aura_tooltip.rs:34`.

Cached contract: Page statement plus current producer cited above; no exact generated declaration anchor located for the entire combined statement.

Prior 12.1 cross-reference (status NOT reused): `data/patch-api/12.1-behaviors.json:752`.

### L178 `prose-2026-07-21-178` — modelable

Source: `Addons can now specify custom ordering for the aura groups within an aura container, using the new layoutIndex option in the layout options table.`

Assessment / ledger note: Cached Lua implements group flow layout and resize. Native forbidden layout notifications and real managed-frame resize/anchor integration require modeling and behavioral proof.

Current producer or adjacent incomplete producer: `CACHE/Blizzard_AuraContainer/Blizzard_CustomAuraContainer.lua:648`, `src/lua_api/frame/methods/forbidden_aspects.rs:73`.

Existing tests (unexecuted; related/bounded only): `tests/on_update_modes.rs:152`.

Cached contract: Page statement plus current producer cited above; no exact generated declaration anchor located for the entire combined statement.

Prior 12.1 cross-reference (status NOT reused): `data/patch-api/12.1-behaviors.json:704`.

### L179 `prose-2026-07-21-179` — modelable

Source: `Added a new CVar tooltipShowAuraSpellIDsCVar: tooltipShowAuraSpellIDs (Game)Default: 0Show spell IDs in tooltips for unit auras. which causes spell IDs to show in aura tooltips. This cvar will not persist between sessions.`

Assessment / ledger note: Implement the declared CVar/default/scope and policy rather than generic name acceptance. Current CVar defaults are a nearby producer; account-wide autoLoot and session-only tooltip setting need persistence boundaries.

Current producer or adjacent incomplete producer: `src/cvars.rs:1`.

Existing tests (unexecuted; related/bounded only): `tests/set_cvar_global.rs:7`.

Cached contract: Page statement plus current producer cited above; no exact generated declaration anchor located for the entire combined statement.

Contract qualification: INFERRED only for local account/session fixture ownership; stated default and persistence behavior remain explicit requirements.

### L180 `prose-2026-07-21-180` — modelable

Source: `The AddAuraAppliedSound API has been renamed AddAuraSound and now supports specifying whether the sound should play when an aura is first added, gains an application or removed.`

Assessment / ledger note: Existing private aura sound model and AddAuraSound entry point are relevant, but all-aura/application/removal triggers and successive renamed aliases need their own delta model; inherited add-context capability alone cannot close this row.

Current producer or adjacent incomplete producer: `src/c_api/private_aura_sounds.rs:1`.

Existing tests (unexecuted; related/bounded only): `tests/private_aura_sound_add_context.rs:51`, `tests/private_aura_sound_removal.rs:154`.

Cached contract: Page statement plus current producer cited above; no exact generated declaration anchor located for the entire combined statement.

### L181 `prose-2026-07-21-181` — implemented-needs-proof

Source: `The RemoveAuraAppliedSound API has been renamed RemoveAuraSound to match.`

Assessment / ledger note: RemoveAuraSound is already registered under retail-12-1-0 and delegates to the native registration removal model. Tests only for renamed publication, zero results, deletion/readback and legacy-wrapper forwarding; no actual playback or historical intermediate alias timing claim.

Current producer or adjacent incomplete producer: `src/c_api/private_aura_sounds.rs:36`, `src/c_api/private_aura_sounds.rs:65`.

Existing tests (unexecuted; related/bounded only): `tests/private_aura_sound_removal.rs:154`.

Cached contract: Page statement plus current producer cited above; no exact generated declaration anchor located for the entire combined statement.

### L182 `prose-2026-07-21-182` — modelable

Source: `Auras flagged as non-secret can now be filtered using excludeSpellIDs and includeSpellIDs without restrictions on any unit.`

Assessment / ledger note: Current unmodified cached Lua has managed group/slot/filter/layout logic. Native filter/secrecy/lifecycle integration is incomplete; existing mock-provider tests are narrower than real addon creation and managed frame updates.

Current producer or adjacent incomplete producer: `CACHE/Blizzard_AuraContainer/Blizzard_CustomAuraContainer.lua:283`, `src/lua_api/globals/auras.rs:224`.

Existing tests (unexecuted; related/bounded only): `patch-tests/patch_12_1/aura_container.rs:67`, `tests/on_update_modes.rs:152`.

Cached contract: Page statement plus current producer cited above; no exact generated declaration anchor located for the entire combined statement.

Prior 12.1 cross-reference (status NOT reused): `data/patch-api/12.1-behaviors.json:656`.

### L183 `prose-2026-07-21-183` — modelable

Source: `Added new APIs FrameScriptObject:HasAccessConstraints and FrameScriptObject:CanBeAccessedInContext to script objects.`

Assessment / ledger note: HasAccessConstraints reads native flags, but CanBeAccessedInContext is registered only under client-wowforever. Retail needs its context-access query and aura restriction policy, not a copied Forever availability claim.

Current producer or adjacent incomplete producer: `src/lua_api/frame/methods/text_attribute_event/attributes.rs:493`, `src/lua_api/frame/methods/misc/secret.rs:38`.

Existing tests (unexecuted; related/bounded only): `tests/forbidden_frames.rs:92`, `tests/forbidden_frames.rs:172`.

Cached contract: Page statement plus current producer cited above; no exact generated declaration anchor located for the entire combined statement.

### L184 `prose-2026-07-21-184` — modelable

Source: `Fixed a bug that was causing the cooldown swipe to show incorrectly in some cases.`

Assessment / ledger note: Cached custom button Lua has presentation setters and aura update handlers; real native aura data, secured region ownership and presentation updates remain incomplete. Exercise actual templates, not table-only substitutes.

Current producer or adjacent incomplete producer: `CACHE/Blizzard_AuraContainer/Blizzard_CustomAuraButton.lua:524`, `src/c_api/c_aura_container_util.rs:228`.

Existing tests (unexecuted; related/bounded only): `tests/aura_container_util.rs:6`, `patch-tests/patch_12_1/aura_tooltip.rs:34`.

Cached contract: Page statement plus current producer cited above; no exact generated declaration anchor located for the entire combined statement.

Prior 12.1 cross-reference (status NOT reused): `data/patch-api/12.1-behaviors.json:752`.

### L185 `prose-2026-07-21-185` — modelable

Source: `Fixed a bug where calling some APIs (like FormatNumber) with secrets would cause objects to be marked as secret incorrectly (and result in Lua Errors).`

Assessment / ledger note: Current formatter/secret infrastructure is adjacent, but the secret argument causing unrelated object secrecy needs a behavioral input/output and object-state model; no broad FormatNumber safety credit.

Current producer or adjacent incomplete producer: `src/c_api/numeric_rule_formatter.rs:1`.

Existing tests (unexecuted; related/bounded only): `tests/secret_value_security.rs:8`.

Cached contract: Page statement plus current producer cited above; no exact generated declaration anchor located for the entire combined statement.

### L186 `prose-2026-07-21-186` — modelable

Source: `Addons are no longer allowed to reparent aura buttons.`

Assessment / ledger note: Current unmodified cached Lua has managed group/slot/filter/layout logic. Native filter/secrecy/lifecycle integration is incomplete; existing mock-provider tests are narrower than real addon creation and managed frame updates.

Current producer or adjacent incomplete producer: `CACHE/Blizzard_AuraContainer/Blizzard_CustomAuraContainer.lua:283`, `src/lua_api/globals/auras.rs:224`.

Existing tests (unexecuted; related/bounded only): `patch-tests/patch_12_1/aura_container.rs:67`, `tests/on_update_modes.rs:152`.

Cached contract: Page statement plus current producer cited above; no exact generated declaration anchor located for the entire combined statement.

Prior 12.1 cross-reference (status NOT reused): `data/patch-api/12.1-behaviors.json:656`.

### L187 `prose-2026-07-21-187` — modelable

Source: `Temporary Weapon Enchants now support click-to-cancel.`

Assessment / ledger note: Cached AddItemEnchantment and cancel handling exist; connect explicit temporary weapon-enchant state to managed frames and click routing.

Current producer or adjacent incomplete producer: `CACHE/Blizzard_AuraContainer/Blizzard_CustomAuraContainer.lua:450`, `src/c_api/weapon_enchants.rs:1`.

Existing tests (unexecuted; related/bounded only): `tests/on_update_modes.rs:152`.

Cached contract: Page statement plus current producer cited above; no exact generated declaration anchor located for the entire combined statement.

Contract qualification: INFERRED: deterministic enchant fixtures; no native cancellation/service timing claim.

### L188 `prose-2026-07-21-188` — modelable

Source: `Boolean candidate filters now support negation by setting the boolean as false (e.g. isStealable = false). Leaving the option as nil will continue to mean "ignore".`

Assessment / ledger note: Current unmodified cached Lua has managed group/slot/filter/layout logic. Native filter/secrecy/lifecycle integration is incomplete; existing mock-provider tests are narrower than real addon creation and managed frame updates.

Current producer or adjacent incomplete producer: `CACHE/Blizzard_AuraContainer/Blizzard_CustomAuraContainer.lua:283`, `src/lua_api/globals/auras.rs:224`.

Existing tests (unexecuted; related/bounded only): `patch-tests/patch_12_1/aura_container.rs:67`, `tests/on_update_modes.rs:152`.

Cached contract: Page statement plus current producer cited above; no exact generated declaration anchor located for the entire combined statement.

Prior 12.1 cross-reference (status NOT reused): `data/patch-api/12.1-behaviors.json:656`.

### L189 `prose-2026-07-21-189` — modelable

Source: `Added new GetActiveBlockedRolesets and GetActiveAllowedRolesets APIs to C_Roleset, which return the list of currently blocked and allowed rolesets.`

Assessment / ledger note: Retail ApplyRolesetFilters is a temporary no-op; GetActiveBlockedRolesets/GetActiveAllowedRolesets/IsRolesetFiltered have no source producer. Model allowed/blocked sets, alwaysBlocked and effective visibility from current declarations.

Current producer or adjacent incomplete producer: `src/lua_api/workarounds/temporary/roleset_defaults.rs:11`.

Existing tests (unexecuted; related/bounded only): `tests/wowforever_rolesets.rs:5`.

Cached contract: Page statement plus current producer cited above; no exact generated declaration anchor located for the entire combined statement.

Contract qualification: INFERRED where Retail declarations omit precedence; Forever tests are related evidence, not Retail implementation proof.

### L190 `prose-2026-07-21-190` — modelable

Source: `Added a new IsRolesetFiltered API on frames, which returns whether the frame is currently filtered by roleset.`

Assessment / ledger note: Retail ApplyRolesetFilters is a temporary no-op; GetActiveBlockedRolesets/GetActiveAllowedRolesets/IsRolesetFiltered have no source producer. Model allowed/blocked sets, alwaysBlocked and effective visibility from current declarations.

Current producer or adjacent incomplete producer: `src/lua_api/workarounds/temporary/roleset_defaults.rs:11`.

Existing tests (unexecuted; related/bounded only): `tests/wowforever_rolesets.rs:5`.

Cached contract: Page statement plus current producer cited above; no exact generated declaration anchor located for the entire combined statement.

Contract qualification: INFERRED where Retail declarations omit precedence; Forever tests are related evidence, not Retail implementation proof.

### L191 `prose-2026-07-21-191` — modelable

Source: `Added a new "alwaysBlocked" roleset, which can be applied to frames to cause them to never show.`

Assessment / ledger note: Retail ApplyRolesetFilters is a temporary no-op; GetActiveBlockedRolesets/GetActiveAllowedRolesets/IsRolesetFiltered have no source producer. Model allowed/blocked sets, alwaysBlocked and effective visibility from current declarations.

Current producer or adjacent incomplete producer: `src/lua_api/workarounds/temporary/roleset_defaults.rs:11`.

Existing tests (unexecuted; related/bounded only): `tests/wowforever_rolesets.rs:5`.

Cached contract: Page statement plus current producer cited above; no exact generated declaration anchor located for the entire combined statement.

Contract qualification: INFERRED where Retail declarations omit precedence; Forever tests are related evidence, not Retail implementation proof.

### L192 `source-context-192` — metadata-only

Source: `Preview of PTR 7`

Assessment: Editorial heading, chronology, navigation, file label or disclaimer only; no runtime credit.

### L194 `prose-2026-07-21-194` — modelable

Source: `A number of Unit APIs are being changed to return secret values when the unit's identity is secret. This is being done to prevent various methods of combining these API calls to compare secret units to each other in combat.`

Assessment / ledger note: Current UnitClass/related getters return ordinary values; implement secret results based on unit identity across every API named by the source. Existing UnitName policy does not cover these added contracts.

Current producer or adjacent incomplete producer: `src/lua_api/globals/group_queries.rs:574`, `src/lua_api/globals/state_backed_queries.rs:322`.

Existing tests (unexecuted; related/bounded only): `tests/unit_name_secret_tokens.rs:22`.

Cached contract: Page statement plus current producer cited above; no exact generated declaration anchor located for the entire combined statement.

Contract qualification: INFERRED: explicit host unit-identity context fixture; do not infer complete security from one protected token.

### L195 `prose-2026-07-21-195` — modelable

Source: `APIs affected: UnitClass, UnitClassBase, UnitIsOwnerOrControllerOfUnit, UnitSex, UnitSexBase, UnitPhaseReason, UnitGroupRolesAssigned, UnitGroupRolesAssignedEnum, UnitIsRaidOfficer, UnitInRaid, UnitIsPVP, UnitRace, UnitIsGroupLeader, UnitIsGroupAssistant, UnitLeadsAnyGroup, UnitGetAvailableRoles, GetInspectSpecialization.`

Assessment / ledger note: Current UnitClass/related getters return ordinary values; implement secret results based on unit identity across every API named by the source. Existing UnitName policy does not cover these added contracts.

Current producer or adjacent incomplete producer: `src/lua_api/globals/group_queries.rs:574`, `src/lua_api/globals/state_backed_queries.rs:322`.

Existing tests (unexecuted; related/bounded only): `tests/unit_name_secret_tokens.rs:22`.

Cached contract: Page statement plus current producer cited above; no exact generated declaration anchor located for the entire combined statement.

Contract qualification: INFERRED: explicit host unit-identity context fixture; do not infer complete security from one protected token.

### L196 `prose-2026-07-21-196` — modelable

Source: `The GetGuildInfo API is being changed to no longer accept compound unit tokens.`

Assessment / ledger note: GetGuildInfo has state-backed producers but the compound-token ban needs explicit authentication/validation before query results.

Current producer or adjacent incomplete producer: `src/lua_api/globals/state_backed_queries.rs:86`.

Existing tests (unexecuted; related/bounded only): No exact behavioral test located for this row; add concrete contract assertions, not symbol-presence checks.

Cached contract: Page statement plus current producer cited above; no exact generated declaration anchor located for the entire combined statement.

Contract qualification: INFERRED: ban failure shape if declaration does not state it; do not silently accept compound tokens.

### L197 `prose-2026-07-21-197` — modelable

Source: `The following APIs are being changed to return secret values when auras are secret: UnitIsCharmed, UnitIsPossessed.`

Assessment / ledger note: UnitIsPossessed currently returns false and UnitIsCharmed is a compatibility stub. Model charm/possession state plus aura secrecy and player/pet/vehicle exceptions.

Current producer or adjacent incomplete producer: `src/lua_api/globals/group_queries_relationships.rs:56`, `src/lua_api/globals/stubs/global_stubs.rs:138`.

Existing tests (unexecuted; related/bounded only): `tests/unit_relationship_defaults.rs:4`.

Cached contract: Page statement plus current producer cited above; no exact generated declaration anchor located for the entire combined statement.

Contract qualification: INFERRED: deterministic possession/charm state fixtures; page explicitly specifies exemptions.

### L198 `source-context-198` — metadata-only

Source: `Aura Classifications`

Assessment: Editorial heading, chronology, navigation, file label or disclaimer only; no runtime credit.

### L199 `prose-2026-07-21-199` — blocked

Source: `Now that custom aura containers can be used to filter and position helpful auras on raid members, we have removed the following healer buffs and HoTs from the "never secret" list:`

Assessment / ledger note: Blocked: retained extract omits the healer buff/HoT spell list. Exact 12.1.0 classification DB and pinned native spell secrecy flags are missing; scratch wikitext can recover names/IDs but not native flags.

Evidence: captured source L199; missing historical/native/source artifact stated above. No batch.

### L202 `source-context-202` — metadata-only

Source: `=== 2026-07-23 ===`

Assessment: Editorial heading, chronology, navigation, file label or disclaimer only; no runtime credit.

### L203 `source-context-203` — metadata-only

Source: `Midnight 12.1.0 PTR Changes 7 (Build 68914)`

Assessment: Editorial heading, chronology, navigation, file label or disclaimer only; no runtime credit.

### L204 `source-context-204` — metadata-only

Source: `Coming in 12.1.0 PTR 7`

Assessment: Editorial heading, chronology, navigation, file label or disclaimer only; no runtime credit.

### L206 `prose-2026-07-23-206` — modelable

Source: `Added support for laying out aura groups in columns.`

Assessment / ledger note: Cached Lua implements group flow layout and resize. Native forbidden layout notifications and real managed-frame resize/anchor integration require modeling and behavioral proof.

Current producer or adjacent incomplete producer: `CACHE/Blizzard_AuraContainer/Blizzard_CustomAuraContainer.lua:648`, `src/lua_api/frame/methods/forbidden_aspects.rs:73`.

Existing tests (unexecuted; related/bounded only): `tests/on_update_modes.rs:152`.

Cached contract: Page statement plus current producer cited above; no exact generated declaration anchor located for the entire combined statement.

Prior 12.1 cross-reference (status NOT reused): `data/patch-api/12.1-behaviors.json:704`.

### L207 `prose-2026-07-23-207` — implemented-needs-proof

Source: `Aura containers can now be created by addons during combat.`

Assessment / ledger note: Current Retail allows AuraContainer creation in combat. Existing type-construction path and tests are relevant; add an actual combat-state creation probe, without claiming historical PTR4 behavior.

Current producer or adjacent incomplete producer: `src/xml/types_elements.rs:425`, `src/xml/types_elements.rs:433`.

Existing tests (unexecuted; related/bounded only): `src/loader/tests/wow_api_globals/frames_and_attributes.rs:337`.

Cached contract: Page statement plus current producer cited above; no exact generated declaration anchor located for the entire combined statement.

Prior 12.1 cross-reference (status NOT reused): `data/patch-api/12.1-behaviors.json:632`.

### L208 `prose-2026-07-23-208` — modelable

Source: `Added support for adjusting aura button tooltip anchors.`

Assessment / ledger note: Cached aura-button tooltip lifecycle exists, including options and throttling. Need native managed button identity, visibility and restricted-aura integration; older AuraButtonMixin mock tooltip tests are not this new contract.

Current producer or adjacent incomplete producer: `CACHE/Blizzard_AuraContainer/Blizzard_AuraButton.lua:216`.

Existing tests (unexecuted; related/bounded only): `patch-tests/patch_12_1/aura_tooltip.rs:34`.

Cached contract: Page statement plus current producer cited above; no exact generated declaration anchor located for the entire combined statement.

Prior 12.1 cross-reference (status NOT reused): `data/patch-api/12.1-behaviors.json:776`.

### L209 `prose-2026-07-23-209` — modelable

Source: `Added support for hiding aura button tooltips while in combat.`

Assessment / ledger note: Cached aura-button tooltip lifecycle exists, including options and throttling. Need native managed button identity, visibility and restricted-aura integration; older AuraButtonMixin mock tooltip tests are not this new contract.

Current producer or adjacent incomplete producer: `CACHE/Blizzard_AuraContainer/Blizzard_AuraButton.lua:216`.

Existing tests (unexecuted; related/bounded only): `patch-tests/patch_12_1/aura_tooltip.rs:34`.

Cached contract: Page statement plus current producer cited above; no exact generated declaration anchor located for the entire combined statement.

Prior 12.1 cross-reference (status NOT reused): `data/patch-api/12.1-behaviors.json:776`.

### L210 `prose-2026-07-23-210` — modelable

Source: `Added a new aura instance ID-only sort method for aura containers, which sorts the auras by aura instance ID.`

Assessment / ledger note: Current unmodified cached Lua has managed group/slot/filter/layout logic. Native filter/secrecy/lifecycle integration is incomplete; existing mock-provider tests are narrower than real addon creation and managed frame updates.

Current producer or adjacent incomplete producer: `CACHE/Blizzard_AuraContainer/Blizzard_CustomAuraContainer.lua:283`, `src/lua_api/globals/auras.rs:224`.

Existing tests (unexecuted; related/bounded only): `patch-tests/patch_12_1/aura_container.rs:67`, `tests/on_update_modes.rs:152`.

Cached contract: Page statement plus current producer cited above; no exact generated declaration anchor located for the entire combined statement.

Prior 12.1 cross-reference (status NOT reused): `data/patch-api/12.1-behaviors.json:656`.

### L211 `prose-2026-07-23-211` — modelable

Source: `Added support for showing multiple dispel textures on an aura button.`

Assessment / ledger note: Cached custom button Lua has presentation setters and aura update handlers; real native aura data, secured region ownership and presentation updates remain incomplete. Exercise actual templates, not table-only substitutes.

Current producer or adjacent incomplete producer: `CACHE/Blizzard_AuraContainer/Blizzard_CustomAuraButton.lua:524`, `src/c_api/c_aura_container_util.rs:228`.

Existing tests (unexecuted; related/bounded only): `tests/aura_container_util.rs:6`, `patch-tests/patch_12_1/aura_tooltip.rs:34`.

Cached contract: Page statement plus current producer cited above; no exact generated declaration anchor located for the entire combined statement.

Prior 12.1 cross-reference (status NOT reused): `data/patch-api/12.1-behaviors.json:752`.

### L212 `prose-2026-07-23-212` — modelable

Source: `Added new APIs to configure custom nineslice, backdrop, or background texture slice assets to use for all aura button tooltips. Note that these are global APIs that apply to all aura buttons (not to individual aura containers).`

Assessment / ledger note: Cached aura-button tooltip lifecycle exists, including options and throttling. Need native managed button identity, visibility and restricted-aura integration; older AuraButtonMixin mock tooltip tests are not this new contract.

Current producer or adjacent incomplete producer: `CACHE/Blizzard_AuraContainer/Blizzard_AuraButton.lua:216`.

Existing tests (unexecuted; related/bounded only): `patch-tests/patch_12_1/aura_tooltip.rs:34`.

Cached contract: Page statement plus current producer cited above; no exact generated declaration anchor located for the entire combined statement.

Prior 12.1 cross-reference (status NOT reused): `data/patch-api/12.1-behaviors.json:776`.

### L213 `prose-2026-07-23-213` — modelable

Source: `Aura buttons now permit native script object API calls - such as SetPoint, SetSize - during UI (re)load, until execution of PLAYER_LOGIN.`

Assessment / ledger note: HasAccessConstraints reads native flags, but CanBeAccessedInContext is registered only under client-wowforever. Retail needs its context-access query and aura restriction policy, not a copied Forever availability claim.

Current producer or adjacent incomplete producer: `src/lua_api/frame/methods/text_attribute_event/attributes.rs:493`, `src/lua_api/frame/methods/misc/secret.rs:38`.

Existing tests (unexecuted; related/bounded only): `tests/forbidden_frames.rs:92`, `tests/forbidden_frames.rs:172`.

Cached contract: Page statement plus current producer cited above; no exact generated declaration anchor located for the entire combined statement.

### L214 `prose-2026-07-23-214` — modelable

Source: `Added a new addon-safe API, ResizeToBoundsRect, which can be used to resize a frame to match the bounds of its children.`

Assessment / ledger note: ResizeToBoundsRect is declared in cached SimpleFrame documentation but has no Rust/Lua simulator producer. Model resize from child bounds and preserve observable anchor/size semantics.

Current producer or adjacent incomplete producer: `src/layout.rs:1`.

Existing tests (unexecuted; related/bounded only): No exact behavioral test located for this row; add concrete contract assertions, not symbol-presence checks.

Cached contract: Page statement plus current producer cited above; no exact generated declaration anchor located for the entire combined statement.

Contract qualification: INFERRED: exact child visibility inclusion/empty bounds behavior unless current declaration or sibling GetBoundsRect clarifies it.

### L215 `prose-2026-07-23-215` — modelable

Source: `Resolved an issue that was causing addons to not be able to call aura button APIs outside of the initializeFrame callback.`

Assessment / ledger note: Cached custom button Lua has presentation setters and aura update handlers; real native aura data, secured region ownership and presentation updates remain incomplete. Exercise actual templates, not table-only substitutes.

Current producer or adjacent incomplete producer: `CACHE/Blizzard_AuraContainer/Blizzard_CustomAuraButton.lua:524`, `src/c_api/c_aura_container_util.rs:228`.

Existing tests (unexecuted; related/bounded only): `tests/aura_container_util.rs:6`, `patch-tests/patch_12_1/aura_tooltip.rs:34`.

Cached contract: Page statement plus current producer cited above; no exact generated declaration anchor located for the entire combined statement.

Prior 12.1 cross-reference (status NOT reused): `data/patch-api/12.1-behaviors.json:752`.

### L216 `prose-2026-07-23-216` — modelable

Source: `Resolved an issue where a Lua error would occur when toggling visibility of an aura container while the mouse was over a visible aura button.`

Assessment / ledger note: Cached aura-button tooltip lifecycle exists, including options and throttling. Need native managed button identity, visibility and restricted-aura integration; older AuraButtonMixin mock tooltip tests are not this new contract.

Current producer or adjacent incomplete producer: `CACHE/Blizzard_AuraContainer/Blizzard_AuraButton.lua:216`.

Existing tests (unexecuted; related/bounded only): `patch-tests/patch_12_1/aura_tooltip.rs:34`.

Cached contract: Page statement plus current producer cited above; no exact generated declaration anchor located for the entire combined statement.

Prior 12.1 cross-reference (status NOT reused): `data/patch-api/12.1-behaviors.json:776`.

### L217 `prose-2026-07-23-217` — modelable

Source: `Resolved an issue involving the CastingBarTypeInfo table which was causing taint issues in nameplates.`

Assessment / ledger note: CastingBarTypeInfo/nameplate taint fix is in vendor Lua context; identify real addon caller/secure owner state and reproduce its route. Existing proxy/delegate infrastructure does not prove the specific taint bug.

Current producer or adjacent incomplete producer: `src/lua_api/env_init/shared_bootstrap.lua:239`, `src/lua_api/script_object_transfer.rs:1`.

Existing tests (unexecuted; related/bounded only): `tests/userdata_proxy.rs:37`, `tests/xml_secure_delegates.rs:7`.

Cached contract: Page statement plus current producer cited above; no exact generated declaration anchor located for the entire combined statement.

### L218 `prose-2026-07-23-218` — modelable

Source: `Resolved an issue causing Lua errors when addons used PingableUnitFrameTemplate.`

Assessment / ledger note: Unmodified cached PingableUnitFrameTemplate and C_PingSecure require a real addon-unit fixture proving taint and enemy ping behavior, not a namespace-presence test.

Current producer or adjacent incomplete producer: `src/c_api/c_ping_secure.rs:1`.

Existing tests (unexecuted; related/bounded only): No exact behavioral test located for this row; add concrete contract assertions, not symbol-presence checks.

Cached contract: Page statement plus current producer cited above; no exact generated declaration anchor located for the entire combined statement.

Contract qualification: INFERRED: synthetic ping result/target fixtures; no live service claim.

### L219 `prose-2026-07-23-219` — modelable

Source: `Child components of aura buttons can no longer be re-parented once configured.`

Assessment / ledger note: Cached custom button Lua has presentation setters and aura update handlers; real native aura data, secured region ownership and presentation updates remain incomplete. Exercise actual templates, not table-only substitutes.

Current producer or adjacent incomplete producer: `CACHE/Blizzard_AuraContainer/Blizzard_CustomAuraButton.lua:524`, `src/c_api/c_aura_container_util.rs:228`.

Existing tests (unexecuted; related/bounded only): `tests/aura_container_util.rs:6`, `patch-tests/patch_12_1/aura_tooltip.rs:34`.

Cached contract: Page statement plus current producer cited above; no exact generated declaration anchor located for the entire combined statement.

Prior 12.1 cross-reference (status NOT reused): `data/patch-api/12.1-behaviors.json:752`.

### L220 `prose-2026-07-23-220` — modelable

Source: `Aura containers that are configured to show aura groups will no longer receive OnSizeChanged updates. Note that this restriction also applies to frames anchored to aura containers (but only after the aura container has an aura group added).`

Assessment / ledger note: Native aspect masks/inheritance exist, but mask publication does not establish UntrustedScriptExecution or UntrustedLayoutScriptExecution suppression across all callback/anchor paths. Model caller-sensitive dispatch before claiming security.

Current producer or adjacent incomplete producer: `src/lua_api/frame/methods/forbidden_aspects.rs:64`.

Existing tests (unexecuted; related/bounded only): `tests/forbidden_aspect_creation.rs:14`.

Cached contract: Page statement plus current producer cited above; no exact generated declaration anchor located for the entire combined statement.

Prior 12.1 cross-reference (status NOT reused): `data/patch-api/12.1-behaviors.json:332`.

### L221 `prose-2026-07-23-221` — modelable

Source: `A number of Unit APIs have been changed to return secret values when the unit's identity is secret. This is being done to prevent various methods of combining these API calls to compare secret units to each other in combat.`

Assessment / ledger note: Current UnitClass/related getters return ordinary values; implement secret results based on unit identity across every API named by the source. Existing UnitName policy does not cover these added contracts.

Current producer or adjacent incomplete producer: `src/lua_api/globals/group_queries.rs:574`, `src/lua_api/globals/state_backed_queries.rs:322`.

Existing tests (unexecuted; related/bounded only): `tests/unit_name_secret_tokens.rs:22`.

Cached contract: Page statement plus current producer cited above; no exact generated declaration anchor located for the entire combined statement.

Contract qualification: INFERRED: explicit host unit-identity context fixture; do not infer complete security from one protected token.

### L222 `prose-2026-07-23-222` — modelable

Source: `APIs affected: UnitClass, UnitClassBase, UnitIsOwnerOrControllerOfUnit, UnitSex, UnitSexBase, UnitPhaseReason, UnitGroupRolesAssigned, UnitGroupRolesAssignedEnum, UnitIsRaidOfficer, UnitInRaid, UnitIsPVP, UnitRace, UnitIsGroupLeader, UnitIsGroupAssistant, UnitLeadsAnyGroup, UnitGetAvailableRoles, GetInspectSpecialization`

Assessment / ledger note: Current UnitClass/related getters return ordinary values; implement secret results based on unit identity across every API named by the source. Existing UnitName policy does not cover these added contracts.

Current producer or adjacent incomplete producer: `src/lua_api/globals/group_queries.rs:574`, `src/lua_api/globals/state_backed_queries.rs:322`.

Existing tests (unexecuted; related/bounded only): `tests/unit_name_secret_tokens.rs:22`.

Cached contract: Page statement plus current producer cited above; no exact generated declaration anchor located for the entire combined statement.

Contract qualification: INFERRED: explicit host unit-identity context fixture; do not infer complete security from one protected token.

### L223 `prose-2026-07-23-223` — modelable

Source: `The following APIs now return secret values when auras are secret: UnitIsCharmed, UnitIsPossessed.`

Assessment / ledger note: UnitIsPossessed currently returns false and UnitIsCharmed is a compatibility stub. Model charm/possession state plus aura secrecy and player/pet/vehicle exceptions.

Current producer or adjacent incomplete producer: `src/lua_api/globals/group_queries_relationships.rs:56`, `src/lua_api/globals/stubs/global_stubs.rs:138`.

Existing tests (unexecuted; related/bounded only): `tests/unit_relationship_defaults.rs:4`.

Cached contract: Page statement plus current producer cited above; no exact generated declaration anchor located for the entire combined statement.

Contract qualification: INFERRED: deterministic possession/charm state fixtures; page explicitly specifies exemptions.

### L224 `prose-2026-07-23-224` — modelable

Source: `The GetGuildInfo API no longer accepts compound unit tokens.`

Assessment / ledger note: GetGuildInfo has state-backed producers but the compound-token ban needs explicit authentication/validation before query results.

Current producer or adjacent incomplete producer: `src/lua_api/globals/state_backed_queries.rs:86`.

Existing tests (unexecuted; related/bounded only): No exact behavioral test located for this row; add concrete contract assertions, not symbol-presence checks.

Cached contract: Page statement plus current producer cited above; no exact generated declaration anchor located for the entire combined statement.

Contract qualification: INFERRED: ban failure shape if declaration does not state it; do not silently accept compound tokens.

### L225 `prose-2026-07-23-225` — modelable

Source: `The UnitName API will no longer return secrets while in an active PvP match.`

Assessment / ledger note: Existing 12.0.5 unit-name-secret-tokens capability protects PvP tokens; the new no-secrets-in-active-PvP delta cannot inherit that opposite policy. Add explicit active-match handling.

Current producer or adjacent incomplete producer: `src/lua_api/globals/group_queries.rs:488`.

Existing tests (unexecuted; related/bounded only): `tests/unit_name_secret_tokens.rs:22`.

Cached contract: Page statement plus current producer cited above; no exact generated declaration anchor located for the entire combined statement.

### L228 `source-context-228` — metadata-only

Source: `=== 2026-08-04 ===`

Assessment: Editorial heading, chronology, navigation, file label or disclaimer only; no runtime credit.

### L229 `source-context-229` — metadata-only

Source: `Midnight 12.1.0 PTR 8: Rise of the mouse (Build 69111)`

Assessment: Editorial heading, chronology, navigation, file label or disclaimer only; no runtime credit.

### L230 `source-context-230` — metadata-only

Source: `Coming in 12.1.0 PTR 8`

Assessment: Editorial heading, chronology, navigation, file label or disclaimer only; no runtime credit.

### L232 `prose-2026-08-04-232` — modelable

Source: `Added APIs to AuraButton that allow addons to show pandemic state via a texture.`

Assessment / ledger note: Cached custom AuraButton Lua has pandemic regions and window updates; wire duration/refresh state and real textures before claiming the feature.

Current producer or adjacent incomplete producer: `CACHE/Blizzard_AuraContainer/Blizzard_CustomAuraButton.lua:612`.

Existing tests (unexecuted; related/bounded only): `tests/aura_refresh_duration.rs:94`.

Cached contract: Page statement plus current producer cited above; no exact generated declaration anchor located for the entire combined statement.

### L233 `prose-2026-08-04-233` — modelable

Source: `Added stealable and showAlways options for AuraButton borders.`

Assessment / ledger note: Cached custom button Lua has presentation setters and aura update handlers; real native aura data, secured region ownership and presentation updates remain incomplete. Exercise actual templates, not table-only substitutes.

Current producer or adjacent incomplete producer: `CACHE/Blizzard_AuraContainer/Blizzard_CustomAuraButton.lua:524`, `src/c_api/c_aura_container_util.rs:228`.

Existing tests (unexecuted; related/bounded only): `tests/aura_container_util.rs:6`, `patch-tests/patch_12_1/aura_tooltip.rs:34`.

Cached contract: Page statement plus current producer cited above; no exact generated declaration anchor located for the entire combined statement.

Prior 12.1 cross-reference (status NOT reused): `data/patch-api/12.1-behaviors.json:752`.

### L234 `prose-2026-08-04-234` — modelable

Source: `AuraButton tooltips are now throttled to update once every 200ms instead of every frame.`

Assessment / ledger note: Cached aura-button tooltip lifecycle exists, including options and throttling. Need native managed button identity, visibility and restricted-aura integration; older AuraButtonMixin mock tooltip tests are not this new contract.

Current producer or adjacent incomplete producer: `CACHE/Blizzard_AuraContainer/Blizzard_AuraButton.lua:216`.

Existing tests (unexecuted; related/bounded only): `patch-tests/patch_12_1/aura_tooltip.rs:34`.

Cached contract: Page statement plus current producer cited above; no exact generated declaration anchor located for the entire combined statement.

Prior 12.1 cross-reference (status NOT reused): `data/patch-api/12.1-behaviors.json:776`.

### L235 `prose-2026-08-04-235` — modelable

Source: `When an AuraContainer is disabled, all AuraButtons and ItemEnchantments belonging to it will now be cleared.`

Assessment / ledger note: Current unmodified cached Lua has managed group/slot/filter/layout logic. Native filter/secrecy/lifecycle integration is incomplete; existing mock-provider tests are narrower than real addon creation and managed frame updates.

Current producer or adjacent incomplete producer: `CACHE/Blizzard_AuraContainer/Blizzard_CustomAuraContainer.lua:283`, `src/lua_api/globals/auras.rs:224`.

Existing tests (unexecuted; related/bounded only): `patch-tests/patch_12_1/aura_container.rs:67`, `tests/on_update_modes.rs:152`.

Cached contract: Page statement plus current producer cited above; no exact generated declaration anchor located for the entire combined statement.

Prior 12.1 cross-reference (status NOT reused): `data/patch-api/12.1-behaviors.json:656`.

### L236 `prose-2026-08-04-236` — modelable

Source: `The UnitIsPossessed and UnitIsCharmed APIs no longer return secret values if the unit token passed is "player", "pet", or "vehicle".`

Assessment / ledger note: UnitIsPossessed currently returns false and UnitIsCharmed is a compatibility stub. Model charm/possession state plus aura secrecy and player/pet/vehicle exceptions.

Current producer or adjacent incomplete producer: `src/lua_api/globals/group_queries_relationships.rs:56`, `src/lua_api/globals/stubs/global_stubs.rs:138`.

Existing tests (unexecuted; related/bounded only): `tests/unit_relationship_defaults.rs:4`.

Cached contract: Page statement plus current producer cited above; no exact generated declaration anchor located for the entire combined statement.

Contract qualification: INFERRED: deterministic possession/charm state fixtures; page explicitly specifies exemptions.

### L237 `prose-2026-08-04-237` — modelable

Source: `Fixed a bug that was causing addons to not be able to use the new SVG tech.`

Assessment / ledger note: VectorGraphics construction/SVG metadata storage exists, but native code explicitly says SVG path rendering is not modeled. Implement 2D SVG asset decoding/rendering and legal method surface; not a 3D feature.

Current producer or adjacent incomplete producer: `src/lua_api/frame/methods/widgets/texture/mod.rs:123`, `src/lua_api/frame/methods/widgets/texture/radial.rs:119`.

Existing tests (unexecuted; related/bounded only): `src/loader/tests/wow_api_globals/frames_and_attributes.rs:103`.

Cached contract: Page statement plus current producer cited above; no exact generated declaration anchor located for the entire combined statement.

Contract qualification: INFERRED: fixture SVG subset and unsupported-method behavior pending exact native declarations.

### L238 `prose-2026-08-04-238` — modelable

Source: `Fixed a bug that could sometimes cause the duration text on AuraButtons to incorrectly show as 0.`

Assessment / ledger note: Cached custom button Lua has presentation setters and aura update handlers; real native aura data, secured region ownership and presentation updates remain incomplete. Exercise actual templates, not table-only substitutes.

Current producer or adjacent incomplete producer: `CACHE/Blizzard_AuraContainer/Blizzard_CustomAuraButton.lua:524`, `src/c_api/c_aura_container_util.rs:228`.

Existing tests (unexecuted; related/bounded only): `tests/aura_container_util.rs:6`, `patch-tests/patch_12_1/aura_tooltip.rs:34`.

Cached contract: Page statement plus current producer cited above; no exact generated declaration anchor located for the entire combined statement.

Prior 12.1 cross-reference (status NOT reused): `data/patch-api/12.1-behaviors.json:752`.

### L239 `prose-2026-08-04-239` — modelable

Source: `Fixed a bug that could cause Lua errors when pinging enemy units represented by addons using PingableUnitFrameTemplate.`

Assessment / ledger note: Unmodified cached PingableUnitFrameTemplate and C_PingSecure require a real addon-unit fixture proving taint and enemy ping behavior, not a namespace-presence test.

Current producer or adjacent incomplete producer: `src/c_api/c_ping_secure.rs:1`.

Existing tests (unexecuted; related/bounded only): No exact behavioral test located for this row; add concrete contract assertions, not symbol-presence checks.

Cached contract: Page statement plus current producer cited above; no exact generated declaration anchor located for the entire combined statement.

Contract qualification: INFERRED: synthetic ping result/target fixtures; no live service claim.

### L240 `prose-2026-08-04-240` — modelable

Source: `Resolved an exploit involving the use of OnSizeChanged to track the number of auras in an AuraContainer.`

Assessment / ledger note: Native aspect masks/inheritance exist, but mask publication does not establish UntrustedScriptExecution or UntrustedLayoutScriptExecution suppression across all callback/anchor paths. Model caller-sensitive dispatch before claiming security.

Current producer or adjacent incomplete producer: `src/lua_api/frame/methods/forbidden_aspects.rs:64`.

Existing tests (unexecuted; related/bounded only): `tests/forbidden_aspect_creation.rs:14`.

Cached contract: Page statement plus current producer cited above; no exact generated declaration anchor located for the entire combined statement.

Prior 12.1 cross-reference (status NOT reused): `data/patch-api/12.1-behaviors.json:332`.

### L243 `source-context-243` — metadata-only

Source: `== Consolidated changes ==`

Assessment: Editorial heading, chronology, navigation, file label or disclaimer only; no runtime credit.

### L244 `source-context-244` — metadata-only

Source: `12.0.7 (68256) → 12.1.0 (69587) Aug 27 2026`

Assessment: Editorial heading, chronology, navigation, file label or disclaimer only; no runtime credit.

### L247 `source-context-247` — metadata-only

Source: `=== Global API ===`

Assessment: Editorial heading, chronology, navigation, file label or disclaimer only; no runtime credit.

### L248 `source-context-248` — metadata-only

Source: `Disclaimer: Any deprecated functions will still be listed as removed.`

Assessment: Editorial heading, chronology, navigation, file label or disclaimer only; no runtime credit.

### L251 `source-context-251` — metadata-only

Source: `=== FrameXML ===`

Assessment: Editorial heading, chronology, navigation, file label or disclaimer only; no runtime credit.

### L252 `source-context-252` — metadata-only

Source: `Disclaimer: Any deprecated functions will still be listed as removed.`

Assessment: Editorial heading, chronology, navigation, file label or disclaimer only; no runtime credit.

### L255 `source-context-255` — metadata-only

Source: `=== ScriptObjects ===`

Assessment: Editorial heading, chronology, navigation, file label or disclaimer only; no runtime credit.

### L258 `source-context-258` — metadata-only

Source: `=== Widgets ===`

Assessment: Editorial heading, chronology, navigation, file label or disclaimer only; no runtime credit.

### L261 `source-context-261` — metadata-only

Source: `=== Events ===`

Assessment: Editorial heading, chronology, navigation, file label or disclaimer only; no runtime credit.

### L264 `source-context-264` — metadata-only

Source: `=== CVars ===`

Assessment: Editorial heading, chronology, navigation, file label or disclaimer only; no runtime credit.

### L267 `source-context-267` — metadata-only

Source: `=== Enumerations ===`

Assessment: Editorial heading, chronology, navigation, file label or disclaimer only; no runtime credit.

### L268 `enumerations-Enum-ClubStreamType-268` — modelable

Source: `Enum.ClubStreamType (C_Club.GetStreamInfo, C_Club.GetStreams)`

Assessment / ledger note: Parent enum and linked API context; review every following delta, not just table existence. Default non-Forever base places Other at 3; generic append publishes Discord at 4, while cached declaration specifies Discord=3. Model exact numeric publication and reordering.

Current producer or adjacent incomplete producer: `src/lua_api/globals/enum_data/game_system.rs:367`, `src/ptr/compat_bootstrap.lua:24`.

Existing tests (unexecuted; related/bounded only): No exact behavioral test located for this row; add concrete contract assertions, not symbol-presence checks.

Cached contract: `CACHE/Blizzard_APIDocumentationGenerated/ClubDocumentation.lua:1738`.

Contract qualification: INFERRED: historical 69587 values if current docs postdate that epoch; exact current-cache numeric values are recorded per row.

### L269 `enumerations-Enum-ClubStreamType-269` — modelable

Source: `+ Discord`

Assessment / ledger note: Cached declaration: `ClubStreamType.Discord = 3`. Default non-Forever base places Other at 3; generic append publishes Discord at 4, while cached declaration specifies Discord=3. Model exact numeric publication and reordering.

Current producer or adjacent incomplete producer: `src/lua_api/globals/enum_data/game_system.rs:367`, `src/ptr/compat_bootstrap.lua:24`.

Existing tests (unexecuted; related/bounded only): No exact behavioral test located for this row; add concrete contract assertions, not symbol-presence checks.

Cached contract: `CACHE/Blizzard_APIDocumentationGenerated/ClubDocumentation.lua:1741`.

Contract qualification: INFERRED: historical 69587 values if current docs postdate that epoch; exact current-cache numeric values are recorded per row.

### L270 `enumerations-Enum-CompanionConfigSlotTypes-270` — implemented-needs-proof

Source: `Enum.CompanionConfigSlotTypes (C_DelvesUI.GetUnseenCuriosBySlotType, C_DelvesUI.SaveSeenCuriosBySlotType)`

Assessment / ledger note: Parent enum and linked API context; review every following delta, not just table existence. Current default Retail already has the listed enum member producers, including the shared 12.1 compatibility bootstrap. Prove exact value, metadata, no cross-environment mutation and default-profile publication; names alone are not numerical proof.

Current producer or adjacent incomplete producer: `src/lua_api/globals/enum_data/game_system.rs:689`, `src/ptr/compat_bootstrap.lua:25`.

Existing tests (unexecuted; related/bounded only): No exact behavioral test located for this row; add concrete contract assertions, not symbol-presence checks.

Cached contract: `CACHE/Blizzard_APIDocumentationGenerated/DelvesConstantsDocumentation.lua:13`.

### L271 `enumerations-Enum-CompanionConfigSlotTypes-271` — implemented-needs-proof

Source: `+ Flavor`

Assessment / ledger note: Cached declaration: `CompanionConfigSlotTypes.Flavor = 3`. Current default Retail already has the listed enum member producers, including the shared 12.1 compatibility bootstrap. Prove exact value, metadata, no cross-environment mutation and default-profile publication; names alone are not numerical proof.

Current producer or adjacent incomplete producer: `src/lua_api/globals/enum_data/game_system.rs:689`, `src/ptr/compat_bootstrap.lua:25`.

Existing tests (unexecuted; related/bounded only): No exact behavioral test located for this row; add concrete contract assertions, not symbol-presence checks.

Cached contract: `CACHE/Blizzard_APIDocumentationGenerated/DelvesConstantsDocumentation.lua:16`.

### L272 `enumerations-Enum-CooldownViewerCategory-272` — implemented-needs-proof

Source: `Enum.CooldownViewerCategory (C_CooldownViewer.GetCooldownViewerCategorySet, C_CooldownViewer.GetCooldownViewerCooldownInfo)`

Assessment / ledger note: Parent enum and linked API context; review every following delta, not just table existence. Current default Retail already has the listed enum member producers, including the shared 12.1 compatibility bootstrap. Prove exact value, metadata, no cross-environment mutation and default-profile publication; names alone are not numerical proof.

Current producer or adjacent incomplete producer: `src/lua_api/globals/enum_data/addon_system.rs:181`, `src/ptr/compat_bootstrap.lua:26`.

Existing tests (unexecuted; related/bounded only): No exact behavioral test located for this row; add concrete contract assertions, not symbol-presence checks.

Cached contract: `CACHE/Blizzard_APIDocumentationGenerated/CooldownViewerConstantsDocumentation.lua:78`.

### L273 `enumerations-Enum-CooldownViewerCategory-273` — implemented-needs-proof

Source: `+ GroupBuff`

Assessment / ledger note: Cached declaration: `CooldownViewerCategory.GroupBuff = 4`. Current default Retail already has the listed enum member producers, including the shared 12.1 compatibility bootstrap. Prove exact value, metadata, no cross-environment mutation and default-profile publication; names alone are not numerical proof.

Current producer or adjacent incomplete producer: `src/lua_api/globals/enum_data/addon_system.rs:181`, `src/ptr/compat_bootstrap.lua:26`.

Existing tests (unexecuted; related/bounded only): No exact behavioral test located for this row; add concrete contract assertions, not symbol-presence checks.

Cached contract: `CACHE/Blizzard_APIDocumentationGenerated/CooldownViewerConstantsDocumentation.lua:82`.

### L274 `enumerations-Enum-CooldownViewerCategory-274` — implemented-needs-proof

Source: `+ SpecAgnosticEssential`

Assessment / ledger note: Cached declaration: `CooldownViewerCategory.SpecAgnosticEssential = 5`. Current default Retail already has the listed enum member producers, including the shared 12.1 compatibility bootstrap. Prove exact value, metadata, no cross-environment mutation and default-profile publication; names alone are not numerical proof.

Current producer or adjacent incomplete producer: `src/lua_api/globals/enum_data/addon_system.rs:181`, `src/ptr/compat_bootstrap.lua:26`.

Existing tests (unexecuted; related/bounded only): No exact behavioral test located for this row; add concrete contract assertions, not symbol-presence checks.

Cached contract: `CACHE/Blizzard_APIDocumentationGenerated/CooldownViewerConstantsDocumentation.lua:83`.

### L275 `enumerations-Enum-CooldownViewerCategory-275` — implemented-needs-proof

Source: `+ SpecAgnosticTracked`

Assessment / ledger note: Cached declaration: `CooldownViewerCategory.SpecAgnosticTracked = 6`. Current default Retail already has the listed enum member producers, including the shared 12.1 compatibility bootstrap. Prove exact value, metadata, no cross-environment mutation and default-profile publication; names alone are not numerical proof.

Current producer or adjacent incomplete producer: `src/lua_api/globals/enum_data/addon_system.rs:181`, `src/ptr/compat_bootstrap.lua:26`.

Existing tests (unexecuted; related/bounded only): No exact behavioral test located for this row; add concrete contract assertions, not symbol-presence checks.

Cached contract: `CACHE/Blizzard_APIDocumentationGenerated/CooldownViewerConstantsDocumentation.lua:84`.

### L276 `enumerations-Enum-CooldownViewerCategory-276` — implemented-needs-proof

Source: `+ EquipSlotEssential`

Assessment / ledger note: Cached declaration: `CooldownViewerCategory.EquipSlotEssential = 7`. Current default Retail already has the listed enum member producers, including the shared 12.1 compatibility bootstrap. Prove exact value, metadata, no cross-environment mutation and default-profile publication; names alone are not numerical proof.

Current producer or adjacent incomplete producer: `src/lua_api/globals/enum_data/addon_system.rs:181`, `src/ptr/compat_bootstrap.lua:26`.

Existing tests (unexecuted; related/bounded only): No exact behavioral test located for this row; add concrete contract assertions, not symbol-presence checks.

Cached contract: `CACHE/Blizzard_APIDocumentationGenerated/CooldownViewerConstantsDocumentation.lua:85`.

### L277 `enumerations-Enum-CooldownViewerCategory-277` — implemented-needs-proof

Source: `+ EquipSlotTracked`

Assessment / ledger note: Cached declaration: `CooldownViewerCategory.EquipSlotTracked = 8`. Current default Retail already has the listed enum member producers, including the shared 12.1 compatibility bootstrap. Prove exact value, metadata, no cross-environment mutation and default-profile publication; names alone are not numerical proof.

Current producer or adjacent incomplete producer: `src/lua_api/globals/enum_data/addon_system.rs:181`, `src/ptr/compat_bootstrap.lua:26`.

Existing tests (unexecuted; related/bounded only): No exact behavioral test located for this row; add concrete contract assertions, not symbol-presence checks.

Cached contract: `CACHE/Blizzard_APIDocumentationGenerated/CooldownViewerConstantsDocumentation.lua:86`.

### L278 `enumerations-Enum-EditModeAccountSetting-278` — implemented-needs-proof

Source: `Enum.EditModeAccountSetting (C_EditMode.SetAccountSetting)`

Assessment / ledger note: Parent enum and linked API context; review every following delta, not just table existence. Current default Retail already has the listed enum member producers, including the shared 12.1 compatibility bootstrap. Prove exact value, metadata, no cross-environment mutation and default-profile publication; names alone are not numerical proof.

Current producer or adjacent incomplete producer: `src/lua_api/globals/enum_data/edit_mode.rs:71`, `src/ptr/compat_bootstrap.lua:27`.

Existing tests (unexecuted; related/bounded only): No exact behavioral test located for this row; add concrete contract assertions, not symbol-presence checks.

Cached contract: `CACHE/Blizzard_APIDocumentationGenerated/EditModeManagerConstantsDocumentation.lua:209`.

### L279 `enumerations-Enum-EditModeAccountSetting-279` — implemented-needs-proof

Source: `+ ShowRaidWarning`

Assessment / ledger note: Cached declaration: `EditModeAccountSetting.ShowRaidWarning = 33`. Current default Retail already has the listed enum member producers, including the shared 12.1 compatibility bootstrap. Prove exact value, metadata, no cross-environment mutation and default-profile publication; names alone are not numerical proof.

Current producer or adjacent incomplete producer: `src/lua_api/globals/enum_data/edit_mode.rs:71`, `src/ptr/compat_bootstrap.lua:27`.

Existing tests (unexecuted; related/bounded only): No exact behavioral test located for this row; add concrete contract assertions, not symbol-presence checks.

Cached contract: `CACHE/Blizzard_APIDocumentationGenerated/EditModeManagerConstantsDocumentation.lua:242`.

### L280 `enumerations-Enum-EditModeMinimapSetting-280` — implemented-needs-proof

Source: `Enum.EditModeMinimapSetting`

Assessment / ledger note: Parent enum and linked API context; review every following delta, not just table existence. Current default Retail already has the listed enum member producers, including the shared 12.1 compatibility bootstrap. Prove exact value, metadata, no cross-environment mutation and default-profile publication; names alone are not numerical proof.

Current producer or adjacent incomplete producer: `src/lua_api/globals/enum_data/edit_mode.rs:183`, `src/ptr/compat_bootstrap.lua:28`.

Existing tests (unexecuted; related/bounded only): No exact behavioral test located for this row; add concrete contract assertions, not symbol-presence checks.

Cached contract: `CACHE/Blizzard_APIDocumentationGenerated/EditModeManagerConstantsDocumentation.lua:529`.

### L281 `enumerations-Enum-EditModeMinimapSetting-281` — implemented-needs-proof

Source: `+ IconScale`

Assessment / ledger note: Cached declaration: `EditModeMinimapSetting.IconScale = 3`. Current default Retail already has the listed enum member producers, including the shared 12.1 compatibility bootstrap. Prove exact value, metadata, no cross-environment mutation and default-profile publication; names alone are not numerical proof.

Current producer or adjacent incomplete producer: `src/lua_api/globals/enum_data/edit_mode.rs:183`, `src/ptr/compat_bootstrap.lua:28`.

Existing tests (unexecuted; related/bounded only): No exact behavioral test located for this row; add concrete contract assertions, not symbol-presence checks.

Cached contract: `CACHE/Blizzard_APIDocumentationGenerated/EditModeManagerConstantsDocumentation.lua:532`.

### L282 `enumerations-Enum-EditModeSystem-282` — implemented-needs-proof

Source: `Enum.EditModeSystem (C_EditMode.ConvertLayoutInfoToString, C_EditMode.ConvertStringToLayoutInfo, C_EditMode.GetLayouts, C_EditMode.SaveLayouts, EDIT_MODE_LAYOUTS_UPDATED)`

Assessment / ledger note: Parent enum and linked API context; review every following delta, not just table existence. Current default Retail already has the listed enum member producers, including the shared 12.1 compatibility bootstrap. Prove exact value, metadata, no cross-environment mutation and default-profile publication; names alone are not numerical proof.

Current producer or adjacent incomplete producer: `src/lua_api/globals/enum_data/edit_mode.rs:15`, `src/ptr/compat_bootstrap.lua:29`.

Existing tests (unexecuted; related/bounded only): `src/loader/tests/wow_api_globals/frames_and_attributes.rs:6`.

Cached contract: `CACHE/Blizzard_APIDocumentationGenerated/EditModeManagerConstantsDocumentation.lua:643`.

### L283 `enumerations-Enum-EditModeSystem-283` — implemented-needs-proof

Source: `+ RaidWarning`

Assessment / ledger note: Cached declaration: `EditModeSystem.RaidWarning = 24`. Current default Retail already has the listed enum member producers, including the shared 12.1 compatibility bootstrap. Prove exact value, metadata, no cross-environment mutation and default-profile publication; names alone are not numerical proof.

Current producer or adjacent incomplete producer: `src/lua_api/globals/enum_data/edit_mode.rs:15`, `src/ptr/compat_bootstrap.lua:29`.

Existing tests (unexecuted; related/bounded only): `src/loader/tests/wow_api_globals/frames_and_attributes.rs:6`.

Cached contract: `CACHE/Blizzard_APIDocumentationGenerated/EditModeManagerConstantsDocumentation.lua:667`.

### L284 `enumerations-Enum-EditModeUnitFrameSetting-284` — modelable

Source: `Enum.EditModeUnitFrameSetting`

Assessment / ledger note: Parent enum and linked API context; review every following delta, not just table existence. Native base still publishes retired IconSize; generic append does not rename/reindex existing fields. Cached BuffIconSize=22 and DebuffIconSize=19 require exact table replacement/removal, not append.

Current producer or adjacent incomplete producer: `src/lua_api/globals/enum_data/edit_mode.rs:136`, `src/ptr/compat_bootstrap.lua:30`.

Existing tests (unexecuted; related/bounded only): No exact behavioral test located for this row; add concrete contract assertions, not symbol-presence checks.

Cached contract: `CACHE/Blizzard_APIDocumentationGenerated/EditModeManagerConstantsDocumentation.lua:691`.

Contract qualification: INFERRED: historical 69587 values if current docs postdate that epoch; exact current-cache numeric values are recorded per row.

### L285 `enumerations-Enum-EditModeUnitFrameSetting-285` — modelable

Source: `- IconSize`

Assessment / ledger note: Current declaration has no retired member; absence supports the stated removal, not native historical behavior. Native base still publishes retired IconSize; generic append does not rename/reindex existing fields. Cached BuffIconSize=22 and DebuffIconSize=19 require exact table replacement/removal, not append.

Current producer or adjacent incomplete producer: `src/lua_api/globals/enum_data/edit_mode.rs:136`, `src/ptr/compat_bootstrap.lua:30`.

Existing tests (unexecuted; related/bounded only): No exact behavioral test located for this row; add concrete contract assertions, not symbol-presence checks.

Cached contract: Page statement plus current producer cited above; no exact generated declaration anchor located for the entire combined statement.

Contract qualification: INFERRED: historical 69587 values if current docs postdate that epoch; exact current-cache numeric values are recorded per row.

### L286 `enumerations-Enum-EditModeUnitFrameSetting-286` — modelable

Source: `+ BuffIconSize`

Assessment / ledger note: Cached declaration: `EditModeUnitFrameSetting.BuffIconSize = 22`. Native base still publishes retired IconSize; generic append does not rename/reindex existing fields. Cached BuffIconSize=22 and DebuffIconSize=19 require exact table replacement/removal, not append.

Current producer or adjacent incomplete producer: `src/lua_api/globals/enum_data/edit_mode.rs:136`, `src/ptr/compat_bootstrap.lua:30`.

Existing tests (unexecuted; related/bounded only): No exact behavioral test located for this row; add concrete contract assertions, not symbol-presence checks.

Cached contract: `CACHE/Blizzard_APIDocumentationGenerated/EditModeManagerConstantsDocumentation.lua:713`.

Contract qualification: INFERRED: historical 69587 values if current docs postdate that epoch; exact current-cache numeric values are recorded per row.

### L287 `enumerations-Enum-EditModeUnitFrameSetting-287` — modelable

Source: `+ DebuffIconSize`

Assessment / ledger note: Cached declaration: `EditModeUnitFrameSetting.DebuffIconSize = 19`. Native base still publishes retired IconSize; generic append does not rename/reindex existing fields. Cached BuffIconSize=22 and DebuffIconSize=19 require exact table replacement/removal, not append.

Current producer or adjacent incomplete producer: `src/lua_api/globals/enum_data/edit_mode.rs:136`, `src/ptr/compat_bootstrap.lua:30`.

Existing tests (unexecuted; related/bounded only): No exact behavioral test located for this row; add concrete contract assertions, not symbol-presence checks.

Cached contract: `CACHE/Blizzard_APIDocumentationGenerated/EditModeManagerConstantsDocumentation.lua:710`.

Contract qualification: INFERRED: historical 69587 values if current docs postdate that epoch; exact current-cache numeric values are recorded per row.

### L288 `enumerations-Enum-FragmentID-288` — modelable

Source: `Enum.FragmentID`

Assessment / ledger note: Parent enum and linked API context; review every following delta, not just table existence. Base contains ReservedArchetypeSeparator=255; append assigns new values above 255, not cached FMapObject=43/FWorldStateListenerData=42. Model explicit values; do not copy the PTR-only override into a Retail proof claim.

Current producer or adjacent incomplete producer: `src/lua_api/globals/enum_data/missing_enums.lua:5656`, `src/ptr/compat_bootstrap.lua:31`.

Existing tests (unexecuted; related/bounded only): No exact behavioral test located for this row; add concrete contract assertions, not symbol-presence checks.

Cached contract: `CACHE/Blizzard_APIDocumentationGenerated/WowCSConstantsDocumentation.lua:13`.

Contract qualification: INFERRED: historical 69587 values if current docs postdate that epoch; exact current-cache numeric values are recorded per row.

### L289 `enumerations-Enum-FragmentID-289` — modelable

Source: `+ FMapObject`

Assessment / ledger note: Cached declaration: `FragmentID.FMapObject = 43`. Base contains ReservedArchetypeSeparator=255; append assigns new values above 255, not cached FMapObject=43/FWorldStateListenerData=42. Model explicit values; do not copy the PTR-only override into a Retail proof claim.

Current producer or adjacent incomplete producer: `src/lua_api/globals/enum_data/missing_enums.lua:5656`, `src/ptr/compat_bootstrap.lua:31`.

Existing tests (unexecuted; related/bounded only): No exact behavioral test located for this row; add concrete contract assertions, not symbol-presence checks.

Cached contract: `CACHE/Blizzard_APIDocumentationGenerated/WowCSConstantsDocumentation.lua:56`.

Contract qualification: INFERRED: historical 69587 values if current docs postdate that epoch; exact current-cache numeric values are recorded per row.

### L290 `enumerations-Enum-FragmentID-290` — modelable

Source: `+ FWorldStateListenerData`

Assessment / ledger note: Cached declaration: `FragmentID.FWorldStateListenerData = 42`. Base contains ReservedArchetypeSeparator=255; append assigns new values above 255, not cached FMapObject=43/FWorldStateListenerData=42. Model explicit values; do not copy the PTR-only override into a Retail proof claim.

Current producer or adjacent incomplete producer: `src/lua_api/globals/enum_data/missing_enums.lua:5656`, `src/ptr/compat_bootstrap.lua:31`.

Existing tests (unexecuted; related/bounded only): No exact behavioral test located for this row; add concrete contract assertions, not symbol-presence checks.

Cached contract: `CACHE/Blizzard_APIDocumentationGenerated/WowCSConstantsDocumentation.lua:55`.

Contract qualification: INFERRED: historical 69587 values if current docs postdate that epoch; exact current-cache numeric values are recorded per row.

### L291 `enumerations-Enum-FrameTutorialAccount-291` — modelable

Source: `Enum.FrameTutorialAccount`

Assessment / ledger note: Parent enum and linked API context; review every following delta, not just table existence. Native base ends at HousingCleanupMode=40; append gives HousingPetBeds=41, while current cache declares 50. Model exact publication and reconcile later-cache epoch.

Current producer or adjacent incomplete producer: `src/lua_api/globals/enum_data/game_system.rs:601`, `src/ptr/compat_bootstrap.lua:32`.

Existing tests (unexecuted; related/bounded only): No exact behavioral test located for this row; add concrete contract assertions, not symbol-presence checks.

Cached contract: `CACHE/Blizzard_APIDocumentationGenerated/TutorialDocumentation.lua:90`.

Contract qualification: INFERRED: historical 69587 values if current docs postdate that epoch; exact current-cache numeric values are recorded per row.

### L292 `enumerations-Enum-FrameTutorialAccount-292` — modelable

Source: `+ HousingPetBeds`

Assessment / ledger note: Cached declaration: `FrameTutorialAccount.HousingPetBeds = 50`. Native base ends at HousingCleanupMode=40; append gives HousingPetBeds=41, while current cache declares 50. Model exact publication and reconcile later-cache epoch.

Current producer or adjacent incomplete producer: `src/lua_api/globals/enum_data/game_system.rs:601`, `src/ptr/compat_bootstrap.lua:32`.

Existing tests (unexecuted; related/bounded only): No exact behavioral test located for this row; add concrete contract assertions, not symbol-presence checks.

Cached contract: `CACHE/Blizzard_APIDocumentationGenerated/TutorialDocumentation.lua:139`.

Contract qualification: INFERRED: historical 69587 values if current docs postdate that epoch; exact current-cache numeric values are recorded per row.

### L293 `enumerations-Enum-HouseFinderSuggestionReason-293` — modelable

Source: `Enum.HouseFinderSuggestionReason (C_HousingNeighborhood.GetCornerstoneNeighborhoodInfo, B_NET_NEIGHBORHOOD_LIST_UPDATED, NEIGHBORHOOD_INFO_UPDATED, NEIGHBORHOOD_LIST_UPDATED, OPEN_NEIGHBORHOOD_CHARTER, OPEN_NEIGHBORHOOD_CHARTER_SIGNATURE_REQUEST, UPDATE_BULLETIN_BOARD_ROSTER)`

Assessment / ledger note: Parent enum and linked API context; review every following delta, not just table existence. Existing test records Relinquished=65; cache declares 128. Existing enum-publication capability covers HomeOwner, not this member. Exact native/epoch bitmask mapping must be reconciled; declaration supplies a modelable target.

Current producer or adjacent incomplete producer: `src/lua_api/globals/enum_data/missing_enums.lua:6666`, `src/ptr/compat_bootstrap.lua:33`.

Existing tests (unexecuted; related/bounded only): `tests/patch_12_0_5_enum_additions.rs:94`.

Cached contract: `CACHE/Blizzard_APIDocumentationGenerated/PlayerHousingConstantsDocumentation.lua:59`.

Contract qualification: INFERRED: historical 69587 values if current docs postdate that epoch; exact current-cache numeric values are recorded per row.

### L294 `enumerations-Enum-HouseFinderSuggestionReason-294` — modelable

Source: `+ Relinquished`

Assessment / ledger note: Cached declaration: `HouseFinderSuggestionReason.Relinquished = 128`. Existing test records Relinquished=65; cache declares 128. Existing enum-publication capability covers HomeOwner, not this member. Exact native/epoch bitmask mapping must be reconciled; declaration supplies a modelable target.

Current producer or adjacent incomplete producer: `src/lua_api/globals/enum_data/missing_enums.lua:6666`, `src/ptr/compat_bootstrap.lua:33`.

Existing tests (unexecuted; related/bounded only): `tests/patch_12_0_5_enum_additions.rs:94`.

Cached contract: `CACHE/Blizzard_APIDocumentationGenerated/PlayerHousingConstantsDocumentation.lua:67`.

Contract qualification: INFERRED: historical 69587 values if current docs postdate that epoch; exact current-cache numeric values are recorded per row.

### L295 `enumerations-Enum-HousingResult-295` — modelable

Source: `Enum.HousingResult (C_HouseEditor.ActivateHouseEditorMode, C_HouseEditor.EnterHouseEditor, C_HouseEditor.GetHouseEditorAvailability, C_HouseEditor.GetHouseEditorModeAvailability, B_NET_NEIGHBORHOOD_LIST_UPDATED, CREATE_NEIGHBORHOOD_RESULT, HOUSE_EDITOR_MODE_CHANGE_FAILURE, HOUSE_EXTERIOR_POSITION_FAILURE, HOUSE_RESERVATION_RESPONSE_RECIEVED, HOUSE_RESET_FAILED, HOUSING_BLUEPRINT_COLLECTION_FAILURE, HOUSING_BLUEPRINT_CONTENTS_FAILURE, HOUSING_BLUEPRINT_DELETE_FAILURE, HOUSING_BLUEPRINT_EXPORT_FAILURE, HOUSING_BLUEPRINT_IMPORT_FAILURE, HOUSING_BLUEPRINT_RENAME_FAILURE, HOUSING_DECOR_DYE_FAILURE, HOUSING_DECOR_PLACE_FAILURE, HOUSING_DECOR_SELECT_RESPONSE, HOUSING_LAYOUT_ROOM_COMPONENT_THEME_SET_CHANGED, HOUSING_ROOM_COMPONENT_CUSTOMIZATION_CHANGE_FAILED, HOUSING_SET_EXTERIOR_HOUSE_SIZE_RESPONSE, HOUSING_SET_EXTERIOR_HOUSE_TYPE_RESPONSE, HOUSING_SET_FIXTURE_RESPONSE, NEIGHBORHOOD_LIST_UPDATED)`

Assessment / ledger note: Parent enum and linked API context; review every following delta, not just table existence. Current SeqEnumDef puts RoomPlacementOutOfBounds at 97; cache declares 96. Other listed additions are present at their cached values. Reconcile the changed ordering/member set without treating parent enum existence as full proof.

Current producer or adjacent incomplete producer: `src/lua_api/globals/enum_data/addon_system.rs:451`, `src/ptr/compat_bootstrap.lua:34`.

Existing tests (unexecuted; related/bounded only): `src/loader/tests/wow_api_globals/housing_result.rs:219`.

Cached contract: `CACHE/Blizzard_APIDocumentationGenerated/PlayerHousingConstantsDocumentation.lua:267`.

Contract qualification: INFERRED: historical 69587 values if current docs postdate that epoch; exact current-cache numeric values are recorded per row.

### L296 `enumerations-Enum-HousingResult-296` — implemented-needs-proof

Source: `+ BlueprintGenericImportError`

Assessment / ledger note: Cached declaration: `HousingResult.BlueprintGenericImportError = 6`. Current default Retail already has the listed enum member producers, including the shared 12.1 compatibility bootstrap. Prove exact value, metadata, no cross-environment mutation and default-profile publication; names alone are not numerical proof.

Current producer or adjacent incomplete producer: `src/lua_api/globals/enum_data/addon_system.rs:451`, `src/ptr/compat_bootstrap.lua:34`.

Existing tests (unexecuted; related/bounded only): `src/loader/tests/wow_api_globals/housing_result.rs:219`.

Cached contract: `CACHE/Blizzard_APIDocumentationGenerated/PlayerHousingConstantsDocumentation.lua:273`.

### L297 `enumerations-Enum-HousingResult-297` — implemented-needs-proof

Source: `+ BlueprintStorageLimit`

Assessment / ledger note: Cached declaration: `HousingResult.BlueprintStorageLimit = 14`. Current default Retail already has the listed enum member producers, including the shared 12.1 compatibility bootstrap. Prove exact value, metadata, no cross-environment mutation and default-profile publication; names alone are not numerical proof.

Current producer or adjacent incomplete producer: `src/lua_api/globals/enum_data/addon_system.rs:451`, `src/ptr/compat_bootstrap.lua:34`.

Existing tests (unexecuted; related/bounded only): `src/loader/tests/wow_api_globals/housing_result.rs:219`.

Cached contract: `CACHE/Blizzard_APIDocumentationGenerated/PlayerHousingConstantsDocumentation.lua:281`.

### L298 `enumerations-Enum-HousingResult-298` — implemented-needs-proof

Source: `+ BlueprintTypeInvalid`

Assessment / ledger note: Cached declaration: `HousingResult.BlueprintTypeInvalid = 12`. Current default Retail already has the listed enum member producers, including the shared 12.1 compatibility bootstrap. Prove exact value, metadata, no cross-environment mutation and default-profile publication; names alone are not numerical proof.

Current producer or adjacent incomplete producer: `src/lua_api/globals/enum_data/addon_system.rs:451`, `src/ptr/compat_bootstrap.lua:34`.

Existing tests (unexecuted; related/bounded only): `src/loader/tests/wow_api_globals/housing_result.rs:219`.

Cached contract: `CACHE/Blizzard_APIDocumentationGenerated/PlayerHousingConstantsDocumentation.lua:279`.

### L299 `enumerations-Enum-HousingResult-299` — implemented-needs-proof

Source: `+ BlueprintNotFound`

Assessment / ledger note: Cached declaration: `HousingResult.BlueprintNotFound = 9`. Current default Retail already has the listed enum member producers, including the shared 12.1 compatibility bootstrap. Prove exact value, metadata, no cross-environment mutation and default-profile publication; names alone are not numerical proof.

Current producer or adjacent incomplete producer: `src/lua_api/globals/enum_data/addon_system.rs:451`, `src/ptr/compat_bootstrap.lua:34`.

Existing tests (unexecuted; related/bounded only): `src/loader/tests/wow_api_globals/housing_result.rs:219`.

Cached contract: `CACHE/Blizzard_APIDocumentationGenerated/PlayerHousingConstantsDocumentation.lua:276`.

### L300 `enumerations-Enum-HousingResult-300` — implemented-needs-proof

Source: `+ InvalidExteriorDocument`

Assessment / ledger note: Cached declaration: `HousingResult.InvalidExteriorDocument = 54`. Current default Retail already has the listed enum member producers, including the shared 12.1 compatibility bootstrap. Prove exact value, metadata, no cross-environment mutation and default-profile publication; names alone are not numerical proof.

Current producer or adjacent incomplete producer: `src/lua_api/globals/enum_data/addon_system.rs:451`, `src/ptr/compat_bootstrap.lua:34`.

Existing tests (unexecuted; related/bounded only): `src/loader/tests/wow_api_globals/housing_result.rs:219`.

Cached contract: `CACHE/Blizzard_APIDocumentationGenerated/PlayerHousingConstantsDocumentation.lua:321`.

### L301 `enumerations-Enum-HousingResult-301` — implemented-needs-proof

Source: `+ BlueprintGenericExportError`

Assessment / ledger note: Cached declaration: `HousingResult.BlueprintGenericExportError = 5`. Current default Retail already has the listed enum member producers, including the shared 12.1 compatibility bootstrap. Prove exact value, metadata, no cross-environment mutation and default-profile publication; names alone are not numerical proof.

Current producer or adjacent incomplete producer: `src/lua_api/globals/enum_data/addon_system.rs:451`, `src/ptr/compat_bootstrap.lua:34`.

Existing tests (unexecuted; related/bounded only): `src/loader/tests/wow_api_globals/housing_result.rs:219`.

Cached contract: `CACHE/Blizzard_APIDocumentationGenerated/PlayerHousingConstantsDocumentation.lua:272`.

### L302 `enumerations-Enum-HousingResult-302` — implemented-needs-proof

Source: `+ InvalidInteriorDocument`

Assessment / ledger note: Cached declaration: `HousingResult.InvalidInteriorDocument = 59`. Current default Retail already has the listed enum member producers, including the shared 12.1 compatibility bootstrap. Prove exact value, metadata, no cross-environment mutation and default-profile publication; names alone are not numerical proof.

Current producer or adjacent incomplete producer: `src/lua_api/globals/enum_data/addon_system.rs:451`, `src/ptr/compat_bootstrap.lua:34`.

Existing tests (unexecuted; related/bounded only): `src/loader/tests/wow_api_globals/housing_result.rs:219`.

Cached contract: `CACHE/Blizzard_APIDocumentationGenerated/PlayerHousingConstantsDocumentation.lua:326`.

### L303 `enumerations-Enum-HousingResult-303` — implemented-needs-proof

Source: `+ BlueprintRequirementsUnmet`

Assessment / ledger note: Cached declaration: `HousingResult.BlueprintRequirementsUnmet = 10`. Current default Retail already has the listed enum member producers, including the shared 12.1 compatibility bootstrap. Prove exact value, metadata, no cross-environment mutation and default-profile publication; names alone are not numerical proof.

Current producer or adjacent incomplete producer: `src/lua_api/globals/enum_data/addon_system.rs:451`, `src/ptr/compat_bootstrap.lua:34`.

Existing tests (unexecuted; related/bounded only): `src/loader/tests/wow_api_globals/housing_result.rs:219`.

Cached contract: `CACHE/Blizzard_APIDocumentationGenerated/PlayerHousingConstantsDocumentation.lua:277`.

### L304 `enumerations-Enum-HousingResult-304` — modelable

Source: `+ RoomPlacementOutOfBounds`

Assessment / ledger note: Cached declaration: `HousingResult.RoomPlacementOutOfBounds = 96`. Current SeqEnumDef puts RoomPlacementOutOfBounds at 97; cache declares 96. Other listed additions are present at their cached values. Reconcile the changed ordering/member set without treating parent enum existence as full proof.

Current producer or adjacent incomplete producer: `src/lua_api/globals/enum_data/addon_system.rs:451`, `src/ptr/compat_bootstrap.lua:34`.

Existing tests (unexecuted; related/bounded only): `src/loader/tests/wow_api_globals/housing_result.rs:219`.

Cached contract: `CACHE/Blizzard_APIDocumentationGenerated/PlayerHousingConstantsDocumentation.lua:363`.

Contract qualification: INFERRED: historical 69587 values if current docs postdate that epoch; exact current-cache numeric values are recorded per row.

### L305 `enumerations-Enum-HousingResult-305` — implemented-needs-proof

Source: `+ BlueprintCodeInvalid`

Assessment / ledger note: Cached declaration: `HousingResult.BlueprintCodeInvalid = 3`. Current default Retail already has the listed enum member producers, including the shared 12.1 compatibility bootstrap. Prove exact value, metadata, no cross-environment mutation and default-profile publication; names alone are not numerical proof.

Current producer or adjacent incomplete producer: `src/lua_api/globals/enum_data/addon_system.rs:451`, `src/ptr/compat_bootstrap.lua:34`.

Existing tests (unexecuted; related/bounded only): `src/loader/tests/wow_api_globals/housing_result.rs:219`.

Cached contract: `CACHE/Blizzard_APIDocumentationGenerated/PlayerHousingConstantsDocumentation.lua:270`.

### L306 `enumerations-Enum-HousingResult-306` — implemented-needs-proof

Source: `+ InsufficientRoomBudget`

Assessment / ledger note: Cached declaration: `HousingResult.InsufficientRoomBudget = 64`. Current default Retail already has the listed enum member producers, including the shared 12.1 compatibility bootstrap. Prove exact value, metadata, no cross-environment mutation and default-profile publication; names alone are not numerical proof.

Current producer or adjacent incomplete producer: `src/lua_api/globals/enum_data/addon_system.rs:451`, `src/ptr/compat_bootstrap.lua:34`.

Existing tests (unexecuted; related/bounded only): `src/loader/tests/wow_api_globals/housing_result.rs:219`.

Cached contract: `CACHE/Blizzard_APIDocumentationGenerated/PlayerHousingConstantsDocumentation.lua:331`.

### L307 `enumerations-Enum-HousingResult-307` — implemented-needs-proof

Source: `+ BlueprintLocationInvalid`

Assessment / ledger note: Cached declaration: `HousingResult.BlueprintLocationInvalid = 7`. Current default Retail already has the listed enum member producers, including the shared 12.1 compatibility bootstrap. Prove exact value, metadata, no cross-environment mutation and default-profile publication; names alone are not numerical proof.

Current producer or adjacent incomplete producer: `src/lua_api/globals/enum_data/addon_system.rs:451`, `src/ptr/compat_bootstrap.lua:34`.

Existing tests (unexecuted; related/bounded only): `src/loader/tests/wow_api_globals/housing_result.rs:219`.

Cached contract: `CACHE/Blizzard_APIDocumentationGenerated/PlayerHousingConstantsDocumentation.lua:274`.

### L308 `enumerations-Enum-HousingResult-308` — implemented-needs-proof

Source: `+ BlueprintNameInvalid`

Assessment / ledger note: Cached declaration: `HousingResult.BlueprintNameInvalid = 8`. Current default Retail already has the listed enum member producers, including the shared 12.1 compatibility bootstrap. Prove exact value, metadata, no cross-environment mutation and default-profile publication; names alone are not numerical proof.

Current producer or adjacent incomplete producer: `src/lua_api/globals/enum_data/addon_system.rs:451`, `src/ptr/compat_bootstrap.lua:34`.

Existing tests (unexecuted; related/bounded only): `src/loader/tests/wow_api_globals/housing_result.rs:219`.

Cached contract: `CACHE/Blizzard_APIDocumentationGenerated/PlayerHousingConstantsDocumentation.lua:275`.

### L309 `enumerations-Enum-HousingResult-309` — implemented-needs-proof

Source: `+ BlueprintVersionInvalid`

Assessment / ledger note: Cached declaration: `HousingResult.BlueprintVersionInvalid = 15`. Current default Retail already has the listed enum member producers, including the shared 12.1 compatibility bootstrap. Prove exact value, metadata, no cross-environment mutation and default-profile publication; names alone are not numerical proof.

Current producer or adjacent incomplete producer: `src/lua_api/globals/enum_data/addon_system.rs:451`, `src/ptr/compat_bootstrap.lua:34`.

Existing tests (unexecuted; related/bounded only): `src/loader/tests/wow_api_globals/housing_result.rs:219`.

Cached contract: `CACHE/Blizzard_APIDocumentationGenerated/PlayerHousingConstantsDocumentation.lua:282`.

### L310 `enumerations-Enum-NamePlateStyle-310` — implemented-needs-proof

Source: `Enum.NamePlateStyle`

Assessment / ledger note: Parent enum and linked API context; review every following delta, not just table existence. Current default Retail already has the listed enum member producers, including the shared 12.1 compatibility bootstrap. Prove exact value, metadata, no cross-environment mutation and default-profile publication; names alone are not numerical proof.

Current producer or adjacent incomplete producer: `src/lua_api/globals/enum_data/combat_system.rs:368`, `src/ptr/compat_bootstrap.lua:50`.

Existing tests (unexecuted; related/bounded only): `src/loader/tests/wow_api_globals/patch_12_0_0_nameplate_style_enums.rs:8`.

Cached contract: `CACHE/Blizzard_APIDocumentationGenerated/NamePlateConstantsDocumentation.lua:128`.

### L311 `enumerations-Enum-NamePlateStyle-311` — implemented-needs-proof

Source: `+ Classic`

Assessment / ledger note: Cached declaration: `NamePlateStyle.Classic = 6`. Current default Retail already has the listed enum member producers, including the shared 12.1 compatibility bootstrap. Prove exact value, metadata, no cross-environment mutation and default-profile publication; names alone are not numerical proof.

Current producer or adjacent incomplete producer: `src/lua_api/globals/enum_data/combat_system.rs:368`, `src/ptr/compat_bootstrap.lua:50`.

Existing tests (unexecuted; related/bounded only): `src/loader/tests/wow_api_globals/patch_12_0_0_nameplate_style_enums.rs:8`.

Cached contract: `CACHE/Blizzard_APIDocumentationGenerated/NamePlateConstantsDocumentation.lua:134`.

### L312 `enumerations-Enum-PingResult-312` — implemented-needs-proof

Source: `Enum.PingResult (C_PingSecure.SendHitTestPing, C_PingSecure.SendPlayerItemPing, C_PingSecure.SendPlayerSpellPing, C_PingSecure.SendUnitPing, C_PingSecure.SetHitTestTargetAndSendPing)`

Assessment / ledger note: Parent enum and linked API context; review every following delta, not just table existence. Current default Retail already has the listed enum member producers, including the shared 12.1 compatibility bootstrap. Prove exact value, metadata, no cross-environment mutation and default-profile publication; names alone are not numerical proof.

Current producer or adjacent incomplete producer: `src/lua_api/globals/enum_data/addon_system.rs:773`, `src/ptr/compat_bootstrap.lua:51`.

Existing tests (unexecuted; related/bounded only): No exact behavioral test located for this row; add concrete contract assertions, not symbol-presence checks.

Cached contract: `CACHE/Blizzard_APIDocumentationGenerated/PingConstantsDocumentation.lua:25`.

### L313 `enumerations-Enum-PingResult-313` — implemented-needs-proof

Source: `+ FailedSilent`

Assessment / ledger note: Cached declaration: `PingResult.FailedSilent = 8`. Current default Retail already has the listed enum member producers, including the shared 12.1 compatibility bootstrap. Prove exact value, metadata, no cross-environment mutation and default-profile publication; names alone are not numerical proof.

Current producer or adjacent incomplete producer: `src/lua_api/globals/enum_data/addon_system.rs:773`, `src/ptr/compat_bootstrap.lua:51`.

Existing tests (unexecuted; related/bounded only): No exact behavioral test located for this row; add concrete contract assertions, not symbol-presence checks.

Cached contract: `CACHE/Blizzard_APIDocumentationGenerated/PingConstantsDocumentation.lua:33`.

### L314 `enumerations-Enum-PingSubjectType-314` — implemented-needs-proof

Source: `Enum.PingSubjectType (C_Ping.GetDefaultPingOptions, C_Ping.GetTextureKitForType, C_Ping.SendMacroPing, C_PingSecure.SendHitTestPing, C_PingSecure.SendPlayerItemPing, C_PingSecure.SendPlayerSpellPing, C_PingSecure.SendUnitPing, C_PingSecure.SetHitTestTargetAndSendPing)`

Assessment / ledger note: Parent enum and linked API context; review every following delta, not just table existence. Current default Retail already has the listed enum member producers, including the shared 12.1 compatibility bootstrap. Prove exact value, metadata, no cross-environment mutation and default-profile publication; names alone are not numerical proof.

Current producer or adjacent incomplete producer: `src/lua_api/globals/enum_data/missing_enums.lua:10719`, `src/ptr/compat_bootstrap.lua:52`.

Existing tests (unexecuted; related/bounded only): No exact behavioral test located for this row; add concrete contract assertions, not symbol-presence checks.

Cached contract: `CACHE/Blizzard_APIDocumentationGenerated/PingConstantsDocumentation.lua:57`.

### L315 `enumerations-Enum-PingSubjectType-315` — implemented-needs-proof

Source: `+ ActionReady`

Assessment / ledger note: Cached declaration: `PingSubjectType.ActionReady = 6`. Current default Retail already has the listed enum member producers, including the shared 12.1 compatibility bootstrap. Prove exact value, metadata, no cross-environment mutation and default-profile publication; names alone are not numerical proof.

Current producer or adjacent incomplete producer: `src/lua_api/globals/enum_data/missing_enums.lua:10719`, `src/ptr/compat_bootstrap.lua:52`.

Existing tests (unexecuted; related/bounded only): No exact behavioral test located for this row; add concrete contract assertions, not symbol-presence checks.

Cached contract: `CACHE/Blizzard_APIDocumentationGenerated/PingConstantsDocumentation.lua:63`.

### L316 `enumerations-Enum-PingSubjectType-316` — implemented-needs-proof

Source: `+ ActionOnCooldown`

Assessment / ledger note: Cached declaration: `PingSubjectType.ActionOnCooldown = 7`. Current default Retail already has the listed enum member producers, including the shared 12.1 compatibility bootstrap. Prove exact value, metadata, no cross-environment mutation and default-profile publication; names alone are not numerical proof.

Current producer or adjacent incomplete producer: `src/lua_api/globals/enum_data/missing_enums.lua:10719`, `src/ptr/compat_bootstrap.lua:52`.

Existing tests (unexecuted; related/bounded only): No exact behavioral test located for this row; add concrete contract assertions, not symbol-presence checks.

Cached contract: `CACHE/Blizzard_APIDocumentationGenerated/PingConstantsDocumentation.lua:64`.

### L317 `enumerations-Enum-PingSubjectType-317` — implemented-needs-proof

Source: `+ ActionUnavailable`

Assessment / ledger note: Cached declaration: `PingSubjectType.ActionUnavailable = 8`. Current default Retail already has the listed enum member producers, including the shared 12.1 compatibility bootstrap. Prove exact value, metadata, no cross-environment mutation and default-profile publication; names alone are not numerical proof.

Current producer or adjacent incomplete producer: `src/lua_api/globals/enum_data/missing_enums.lua:10719`, `src/ptr/compat_bootstrap.lua:52`.

Existing tests (unexecuted; related/bounded only): No exact behavioral test located for this row; add concrete contract assertions, not symbol-presence checks.

Cached contract: `CACHE/Blizzard_APIDocumentationGenerated/PingConstantsDocumentation.lua:65`.

### L318 `enumerations-Enum-SecretAspect-318` — modelable

Source: `Enum.SecretAspect (FrameScriptObject:HasSecretAspect)`

Assessment / ledger note: Parent enum and linked API context; review every following delta, not just table existence. Base maximum is 524288; append gives RadialProgress=524289, while cached mask is 8388608. Replace append arithmetic with explicit mask publication and implement radial security at its producer.

Current producer or adjacent incomplete producer: `src/lua_api/globals/enum_data/missing_enums.lua:12414`, `src/ptr/compat_bootstrap.lua:53`.

Existing tests (unexecuted; related/bounded only): No exact behavioral test located for this row; add concrete contract assertions, not symbol-presence checks.

Cached contract: `CACHE/Blizzard_APIDocumentationGenerated/SecretAspectConstantsDocumentation.lua:13`.

Contract qualification: INFERRED: historical 69587 values if current docs postdate that epoch; exact current-cache numeric values are recorded per row.

### L319 `enumerations-Enum-SecretAspect-319` — modelable

Source: `+ RadialProgress`

Assessment / ledger note: Cached declaration: `SecretAspect.RadialProgress = 8388608`. Base maximum is 524288; append gives RadialProgress=524289, while cached mask is 8388608. Replace append arithmetic with explicit mask publication and implement radial security at its producer.

Current producer or adjacent incomplete producer: `src/lua_api/globals/enum_data/missing_enums.lua:12414`, `src/ptr/compat_bootstrap.lua:53`.

Existing tests (unexecuted; related/bounded only): No exact behavioral test located for this row; add concrete contract assertions, not symbol-presence checks.

Cached contract: `CACHE/Blizzard_APIDocumentationGenerated/SecretAspectConstantsDocumentation.lua:42`.

Contract qualification: INFERRED: historical 69587 values if current docs postdate that epoch; exact current-cache numeric values are recorded per row.

### L320 `enumerations-Enum-TieredEntranceType-320` — implemented-needs-proof

Source: `Enum.TieredEntranceType (C_DelvesUI.GetTieredEntranceType)`

Assessment / ledger note: Parent enum and linked API context; review every following delta, not just table existence. Current default Retail already has the listed enum member producers, including the shared 12.1 compatibility bootstrap. Prove exact value, metadata, no cross-environment mutation and default-profile publication; names alone are not numerical proof.

Current producer or adjacent incomplete producer: `src/lua_api/globals/enum_data/addon_system.rs:750`.

Existing tests (unexecuted; related/bounded only): No exact behavioral test located for this row; add concrete contract assertions, not symbol-presence checks.

Cached contract: `CACHE/Blizzard_APIDocumentationGenerated/DelvesConstantsDocumentation.lua:63`.

### L321 `enumerations-Enum-TieredEntranceType-321` — implemented-needs-proof

Source: `+ Lairs`

Assessment / ledger note: Cached declaration: `TieredEntranceType.Lairs = 4`. Current default Retail already has the listed enum member producers, including the shared 12.1 compatibility bootstrap. Prove exact value, metadata, no cross-environment mutation and default-profile publication; names alone are not numerical proof.

Current producer or adjacent incomplete producer: `src/lua_api/globals/enum_data/addon_system.rs:750`.

Existing tests (unexecuted; related/bounded only): No exact behavioral test located for this row; add concrete contract assertions, not symbol-presence checks.

Cached contract: `CACHE/Blizzard_APIDocumentationGenerated/DelvesConstantsDocumentation.lua:67`.

### L322 `enumerations-Enum-TooltipDataLineType-322` — implemented-needs-proof

Source: `Enum.TooltipDataLineType`

Assessment / ledger note: Parent enum and linked API context; review every following delta, not just table existence. Current default Retail already has the listed enum member producers, including the shared 12.1 compatibility bootstrap. Prove exact value, metadata, no cross-environment mutation and default-profile publication; names alone are not numerical proof.

Current producer or adjacent incomplete producer: `src/lua_api/globals/enum_data/missing_enums.lua:13328`, `src/ptr/compat_bootstrap.lua:54`.

Existing tests (unexecuted; related/bounded only): `src/loader/tests/wow_api_globals/patch_12_1_5_tooltip_line_enums.rs:77`.

Cached contract: `CACHE/Blizzard_APIDocumentationGenerated/TooltipInfoSharedDocumentation.lua:34`.

### L323 `enumerations-Enum-TooltipDataLineType-323` — implemented-needs-proof

Source: `+ ItemSpellTriggerOnUse`

Assessment / ledger note: Cached declaration: `TooltipDataLineType.ItemSpellTriggerOnUse = 44`. Current default Retail already has the listed enum member producers, including the shared 12.1 compatibility bootstrap. Prove exact value, metadata, no cross-environment mutation and default-profile publication; names alone are not numerical proof.

Current producer or adjacent incomplete producer: `src/lua_api/globals/enum_data/missing_enums.lua:13328`, `src/ptr/compat_bootstrap.lua:54`.

Existing tests (unexecuted; related/bounded only): `src/loader/tests/wow_api_globals/patch_12_1_5_tooltip_line_enums.rs:77`.

Cached contract: `CACHE/Blizzard_APIDocumentationGenerated/TooltipInfoSharedDocumentation.lua:78`.

### L324 `enumerations-Enum-TooltipDataLineType-324` — implemented-needs-proof

Source: `+ ItemSpellTriggerOnEquip`

Assessment / ledger note: Cached declaration: `TooltipDataLineType.ItemSpellTriggerOnEquip = 45`. Current default Retail already has the listed enum member producers, including the shared 12.1 compatibility bootstrap. Prove exact value, metadata, no cross-environment mutation and default-profile publication; names alone are not numerical proof.

Current producer or adjacent incomplete producer: `src/lua_api/globals/enum_data/missing_enums.lua:13328`, `src/ptr/compat_bootstrap.lua:54`.

Existing tests (unexecuted; related/bounded only): `src/loader/tests/wow_api_globals/patch_12_1_5_tooltip_line_enums.rs:77`.

Cached contract: `CACHE/Blizzard_APIDocumentationGenerated/TooltipInfoSharedDocumentation.lua:79`.

### L325 `enumerations-Enum-TooltipDataLineType-325` — implemented-needs-proof

Source: `+ ItemSpellTriggerOnProc`

Assessment / ledger note: Cached declaration: `TooltipDataLineType.ItemSpellTriggerOnProc = 46`. Current default Retail already has the listed enum member producers, including the shared 12.1 compatibility bootstrap. Prove exact value, metadata, no cross-environment mutation and default-profile publication; names alone are not numerical proof.

Current producer or adjacent incomplete producer: `src/lua_api/globals/enum_data/missing_enums.lua:13328`, `src/ptr/compat_bootstrap.lua:54`.

Existing tests (unexecuted; related/bounded only): `src/loader/tests/wow_api_globals/patch_12_1_5_tooltip_line_enums.rs:77`.

Cached contract: `CACHE/Blizzard_APIDocumentationGenerated/TooltipInfoSharedDocumentation.lua:80`.

### L328 `source-context-328` — metadata-only

Source: `=== Structures ===`

Assessment: Editorial heading, chronology, navigation, file label or disclaimer only; no runtime credit.

### L329 `structures-AddPrivateAuraAnchorArgs-329` — implemented-needs-proof

Source: `AddPrivateAuraAnchorArgs (C_UnitAuras.AddPrivateAuraAnchor)`

Assessment / ledger note: Input reader models the new flags and omits showCountdownFrame. Prove each default/non-default flag, retired output absence, invalid inputs and callback-visible state.

Current producer or adjacent incomplete producer: `src/c_api/private_aura_anchors/input.rs:141`.

Existing tests (unexecuted; related/bounded only): `tests/private_aura_anchors.rs:301`.

Cached contract: `CACHE/Blizzard_APIDocumentationGenerated/UnitConstantsDocumentation.lua:6`.

### L330 `structures-AddPrivateAuraAnchorArgs-330` — implemented-needs-proof

Source: `- showCountdownFrame`

Assessment / ledger note: Input reader models the new flags and omits showCountdownFrame. Prove each default/non-default flag, retired output absence, invalid inputs and callback-visible state.

Current producer or adjacent incomplete producer: `src/c_api/private_aura_anchors/input.rs:141`.

Existing tests (unexecuted; related/bounded only): `tests/private_aura_anchors.rs:301`.

Cached contract: `CACHE/Blizzard_APIDocumentationGenerated/UnitConstantsDocumentation.lua:6`.

### L331 `structures-AddPrivateAuraAnchorArgs-331` — implemented-needs-proof

Source: `+ showDispelIcon`

Assessment / ledger note: Input reader models the new flags and omits showCountdownFrame. Prove each default/non-default flag, retired output absence, invalid inputs and callback-visible state.

Current producer or adjacent incomplete producer: `src/c_api/private_aura_anchors/input.rs:141`.

Existing tests (unexecuted; related/bounded only): `tests/private_aura_anchors.rs:301`.

Cached contract: `CACHE/Blizzard_APIDocumentationGenerated/UnitConstantsDocumentation.lua:16`.

### L332 `structures-AddPrivateAuraAnchorArgs-332` — implemented-needs-proof

Source: `+ showCooldownEdge`

Assessment / ledger note: Input reader models the new flags and omits showCountdownFrame. Prove each default/non-default flag, retired output absence, invalid inputs and callback-visible state.

Current producer or adjacent incomplete producer: `src/c_api/private_aura_anchors/input.rs:141`.

Existing tests (unexecuted; related/bounded only): `tests/private_aura_anchors.rs:301`.

Cached contract: `CACHE/Blizzard_APIDocumentationGenerated/UnitConstantsDocumentation.lua:14`.

### L333 `structures-AddPrivateAuraAnchorArgs-333` — implemented-needs-proof

Source: `+ showCooldownFrame`

Assessment / ledger note: Input reader models the new flags and omits showCountdownFrame. Prove each default/non-default flag, retired output absence, invalid inputs and callback-visible state.

Current producer or adjacent incomplete producer: `src/c_api/private_aura_anchors/input.rs:141`.

Existing tests (unexecuted; related/bounded only): `tests/private_aura_anchors.rs:301`.

Cached contract: `CACHE/Blizzard_APIDocumentationGenerated/UnitConstantsDocumentation.lua:13`.

### L334 `structures-BNetAccountInfo-334` — implemented-needs-proof

Source: `BNetAccountInfo (C_BattleNet.GetAccountInfoByGUID, C_BattleNet.GetAccountInfoByID, C_BattleNet.GetFriendAccountInfo)`

Assessment / ledger note: Retail writer supplies friendTags from explicit friend state and friendLevel as fixed 0. Prove the bounded published shape across all advertised getters; live friend-level progression remains unmodeled and uncredited.

Current producer or adjacent incomplete producer: `src/c_api/c_battle_net.rs:681`.

Existing tests (unexecuted; related/bounded only): `src/loader/tests/wow_api_globals/startup_globals.rs:913`.

Cached contract: `CACHE/Blizzard_APIDocumentationGenerated/BattleNetDocumentation.lua:399`.

Prior 12.1 cross-reference (status NOT reused): `data/patch-api/12.1-behaviors.json:1242`.

### L335 `structures-BNetAccountInfo-335` — implemented-needs-proof

Source: `+ friendLevel`

Assessment / ledger note: Retail writer supplies friendTags from explicit friend state and friendLevel as fixed 0. Prove the bounded published shape across all advertised getters; live friend-level progression remains unmodeled and uncredited.

Current producer or adjacent incomplete producer: `src/c_api/c_battle_net.rs:681`.

Existing tests (unexecuted; related/bounded only): `src/loader/tests/wow_api_globals/startup_globals.rs:913`.

Cached contract: `CACHE/Blizzard_APIDocumentationGenerated/BattleNetDocumentation.lua:408`.

Prior 12.1 cross-reference (status NOT reused): `data/patch-api/12.1-behaviors.json:1242`.

### L336 `structures-BNetAccountInfo-336` — implemented-needs-proof

Source: `+ friendTags`

Assessment / ledger note: Retail writer supplies friendTags from explicit friend state and friendLevel as fixed 0. Prove the bounded published shape across all advertised getters; live friend-level progression remains unmodeled and uncredited.

Current producer or adjacent incomplete producer: `src/c_api/c_battle_net.rs:681`.

Existing tests (unexecuted; related/bounded only): `src/loader/tests/wow_api_globals/startup_globals.rs:913`.

Cached contract: `CACHE/Blizzard_APIDocumentationGenerated/BattleNetDocumentation.lua:413`.

Prior 12.1 cross-reference (status NOT reused): `data/patch-api/12.1-behaviors.json:1242`.

### L337 `structures-BNetGameAccountInfo-337` — implemented-needs-proof

Source: `BNetGameAccountInfo (C_BattleNet.GetAccountInfoByGUID, C_BattleNet.GetAccountInfoByID, C_BattleNet.GetFriendAccountInfo, C_BattleNet.GetFriendGameAccountInfo, C_BattleNet.GetGameAccountInfoByGUID, C_BattleNet.GetGameAccountInfoByID)`

Assessment / ledger note: Current Retail producer publishes classFilename from uppercase class_name. Tests only for the bounded field shape across advertised getters and canonical single/multiword class fixtures; do not infer full native class-token or nilability behavior from the existing simple string assertion.

Current producer or adjacent incomplete producer: `src/c_api/c_battle_net.rs:691`.

Existing tests (unexecuted; related/bounded only): `src/loader/tests/wow_api_globals/startup_globals.rs:915`.

Cached contract: `CACHE/Blizzard_APIDocumentationGenerated/BattleNetDocumentation.lua:434`.

Contract qualification: INFERRED: deterministic Battle.net fixtures; native service delivery excluded.

Prior 12.1 cross-reference (status NOT reused): `data/patch-api/12.1-behaviors.json:1242`.

### L338 `structures-BNetGameAccountInfo-338` — implemented-needs-proof

Source: `+ classFilename`

Assessment / ledger note: Current Retail producer publishes classFilename from uppercase class_name. Tests only for the bounded field shape across advertised getters and canonical single/multiword class fixtures; do not infer full native class-token or nilability behavior from the existing simple string assertion.

Current producer or adjacent incomplete producer: `src/c_api/c_battle_net.rs:691`.

Existing tests (unexecuted; related/bounded only): `src/loader/tests/wow_api_globals/startup_globals.rs:915`.

Cached contract: `CACHE/Blizzard_APIDocumentationGenerated/BattleNetDocumentation.lua:453`.

Contract qualification: INFERRED: deterministic Battle.net fixtures; native service delivery excluded.

Prior 12.1 cross-reference (status NOT reused): `data/patch-api/12.1-behaviors.json:1242`.

### L339 `structures-ChatMessageEventParams-339` — modelable

Source: `ChatMessageEventParams`

Assessment / ledger note: Chat dispatch exists, but discordInfo has no simulator producer. Add explicit DiscordChatInfo payload to the local event model and preserve the expanded chat filter argument vector.

Current producer or adjacent incomplete producer: `src/lua_api/globals/state_backed_queries.rs:98`, `src/c_api/c_discord.rs:1`.

Existing tests (unexecuted; related/bounded only): No exact behavioral test located for this row; add concrete contract assertions, not symbol-presence checks.

Cached contract: `CACHE/Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua:2813`.

Contract qualification: INFERRED: local Discord fixtures and delivery order; no OAuth/network delivery contract inferred.

### L340 `structures-ChatMessageEventParams-340` — modelable

Source: `+ discordInfo`

Assessment / ledger note: Chat dispatch exists, but discordInfo has no simulator producer. Add explicit DiscordChatInfo payload to the local event model and preserve the expanded chat filter argument vector.

Current producer or adjacent incomplete producer: `src/lua_api/globals/state_backed_queries.rs:98`, `src/c_api/c_discord.rs:1`.

Existing tests (unexecuted; related/bounded only): No exact behavioral test located for this row; add concrete contract assertions, not symbol-presence checks.

Cached contract: `CACHE/Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua:2834`.

Contract qualification: INFERRED: local Discord fixtures and delivery order; no OAuth/network delivery contract inferred.

### L341 `structures-ClubMemberInfo-341` — modelable

Source: `ClubMemberInfo (C_Club.GetInfoFromLastCommunityChatLine, C_Club.GetInvitationInfo, C_Club.GetInvitationsForClub, C_Club.GetInvitationsForSelf, C_Club.GetMemberInfo, C_Club.GetMemberInfoForSelf, C_Club.GetMessageInfo, C_Club.GetMessagesBefore, C_Club.GetMessagesInRange, C_Club.GetTickets, CLUB_INVITATION_ADDED_FOR_SELF, CLUB_TICKET_CREATED)`

Assessment / ledger note: Club member getters have a producer but no discordInfo field. Add optional DiscordChatInfo backed by explicit member state; nil defaults alone are not nonempty coverage.

Current producer or adjacent incomplete producer: `src/lua_api/globals/missing_surface/club_info.rs:197`.

Existing tests (unexecuted; related/bounded only): No exact behavioral test located for this row; add concrete contract assertions, not symbol-presence checks.

Cached contract: `CACHE/Blizzard_APIDocumentationGenerated/ClubDocumentation.lua:1808`.

Contract qualification: INFERRED: local club fixture state; current declaration supplies field type/optional status.

### L342 `structures-ClubMemberInfo-342` — modelable

Source: `+ discordInfo`

Assessment / ledger note: Club member getters have a producer but no discordInfo field. Add optional DiscordChatInfo backed by explicit member state; nil defaults alone are not nonempty coverage.

Current producer or adjacent incomplete producer: `src/lua_api/globals/missing_surface/club_info.rs:197`.

Existing tests (unexecuted; related/bounded only): No exact behavioral test located for this row; add concrete contract assertions, not symbol-presence checks.

Cached contract: `CACHE/Blizzard_APIDocumentationGenerated/ClubDocumentation.lua:1843`.

Contract qualification: INFERRED: local club fixture state; current declaration supplies field type/optional status.

### L343 `structures-CooldownViewerCooldown-343` — modelable

Source: `CooldownViewerCooldown (C_CooldownViewer.GetCooldownViewerCooldownInfo)`

Assessment / ledger note: GetCooldownViewerCooldownInfo is a temporary nil default; no spellCategoryID/equipSlot/isInvisible producer exists. Model explicit viewer entries with optional category/equipSlot and required visibility flag.

Current producer or adjacent incomplete producer: `src/lua_api/workarounds/temporary/cooldown_viewer_defaults.rs:20`.

Existing tests (unexecuted; related/bounded only): No exact behavioral test located for this row; add concrete contract assertions, not symbol-presence checks.

Cached contract: `CACHE/Blizzard_APIDocumentationGenerated/CooldownViewerDocumentation.lua:127`.

Contract qualification: INFERRED: synthetic cooldown catalog/visibility fixtures; current declaration supplies types/optionality.

Prior 12.1 cross-reference (status NOT reused): `data/patch-api/12.1-behaviors.json:1321`.

### L344 `structures-CooldownViewerCooldown-344` — modelable

Source: `+ spellCategoryID`

Assessment / ledger note: GetCooldownViewerCooldownInfo is a temporary nil default; no spellCategoryID/equipSlot/isInvisible producer exists. Model explicit viewer entries with optional category/equipSlot and required visibility flag.

Current producer or adjacent incomplete producer: `src/lua_api/workarounds/temporary/cooldown_viewer_defaults.rs:20`.

Existing tests (unexecuted; related/bounded only): No exact behavioral test located for this row; add concrete contract assertions, not symbol-presence checks.

Cached contract: `CACHE/Blizzard_APIDocumentationGenerated/CooldownViewerDocumentation.lua:133`.

Contract qualification: INFERRED: synthetic cooldown catalog/visibility fixtures; current declaration supplies types/optionality.

Prior 12.1 cross-reference (status NOT reused): `data/patch-api/12.1-behaviors.json:1321`.

### L345 `structures-CooldownViewerCooldown-345` — modelable

Source: `+ equipSlot`

Assessment / ledger note: GetCooldownViewerCooldownInfo is a temporary nil default; no spellCategoryID/equipSlot/isInvisible producer exists. Model explicit viewer entries with optional category/equipSlot and required visibility flag.

Current producer or adjacent incomplete producer: `src/lua_api/workarounds/temporary/cooldown_viewer_defaults.rs:20`.

Existing tests (unexecuted; related/bounded only): No exact behavioral test located for this row; add concrete contract assertions, not symbol-presence checks.

Cached contract: `CACHE/Blizzard_APIDocumentationGenerated/CooldownViewerDocumentation.lua:136`.

Contract qualification: INFERRED: synthetic cooldown catalog/visibility fixtures; current declaration supplies types/optionality.

Prior 12.1 cross-reference (status NOT reused): `data/patch-api/12.1-behaviors.json:1321`.

### L346 `structures-CooldownViewerCooldown-346` — modelable

Source: `+ isInvisible`

Assessment / ledger note: GetCooldownViewerCooldownInfo is a temporary nil default; no spellCategoryID/equipSlot/isInvisible producer exists. Model explicit viewer entries with optional category/equipSlot and required visibility flag.

Current producer or adjacent incomplete producer: `src/lua_api/workarounds/temporary/cooldown_viewer_defaults.rs:20`.

Existing tests (unexecuted; related/bounded only): No exact behavioral test located for this row; add concrete contract assertions, not symbol-presence checks.

Cached contract: `CACHE/Blizzard_APIDocumentationGenerated/CooldownViewerDocumentation.lua:143`.

Contract qualification: INFERRED: synthetic cooldown catalog/visibility fixtures; current declaration supplies types/optionality.

Prior 12.1 cross-reference (status NOT reused): `data/patch-api/12.1-behaviors.json:1321`.

### L347 `structures-HousingDecorInstanceInfo-347` — modelable

Source: `HousingDecorInstanceInfo (C_HousingBasicMode.GetHoveredDecorInfo, C_HousingBasicMode.GetSelectedDecorInfo, C_HousingCleanupMode.GetHoveredDecorInfo, C_HousingCustomizeMode.GetHoveredDecorInfo, C_HousingCustomizeMode.GetSelectedDecorInfo, C_HousingDecor.GetDecorInstanceInfoForGUID, C_HousingDecor.GetHoveredDecorInfo, C_HousingDecor.GetSelectedDecorInfo, C_HousingExpertMode.GetHoveredDecorInfo, C_HousingExpertMode.GetSelectedDecorInfo)`

Assessment / ledger note: Housing backing state exists, but no canAttachPet output producer exists. Add eligibility to decor instance state and serialize it through each relevant mode query.

Current producer or adjacent incomplete producer: `src/c_api/c_housing.rs:1`.

Existing tests (unexecuted; related/bounded only): No exact behavioral test located for this row; add concrete contract assertions, not symbol-presence checks.

Cached contract: `CACHE/Blizzard_APIDocumentationGenerated/HousingDecorSharedDocumentation.lua:36`.

Contract qualification: INFERRED: local pet-attachment eligibility fixture, not service/native eligibility rules.

### L348 `structures-HousingDecorInstanceInfo-348` — modelable

Source: `+ canAttachPet`

Assessment / ledger note: Housing backing state exists, but no canAttachPet output producer exists. Add eligibility to decor instance state and serialize it through each relevant mode query.

Current producer or adjacent incomplete producer: `src/c_api/c_housing.rs:1`.

Existing tests (unexecuted; related/bounded only): No exact behavioral test located for this row; add concrete contract assertions, not symbol-presence checks.

Cached contract: `CACHE/Blizzard_APIDocumentationGenerated/HousingDecorSharedDocumentation.lua:50`.

Contract qualification: INFERRED: local pet-attachment eligibility fixture, not service/native eligibility rules.

### L349 `structures-LfgEntryData-349` — modelable

Source: `LfgEntryData (C_LFGList.GetActiveEntryInfo)`

Assessment / ledger note: GetActiveEntryInfo currently returns nil; implement a nonempty active entry with censored field from explicit local listing state. Search-result field does not prove active-entry output.

Current producer or adjacent incomplete producer: `src/lua_api/globals/lfg_list/catalog.rs:343`.

Existing tests (unexecuted; related/bounded only): `src/loader/tests/wow_api_globals/patch_12_1_service_payloads.rs:529`.

Cached contract: `CACHE/Blizzard_APIDocumentationGenerated/LFGListInfoDocumentation.lua:923`.

Contract qualification: INFERRED: fixture censorship policy; no server filtering proof.

Prior 12.1 cross-reference (status NOT reused): `data/patch-api/12.1-behaviors.json:1369`.

### L350 `structures-LfgEntryData-350` — modelable

Source: `+ censored`

Assessment / ledger note: GetActiveEntryInfo currently returns nil; implement a nonempty active entry with censored field from explicit local listing state. Search-result field does not prove active-entry output.

Current producer or adjacent incomplete producer: `src/lua_api/globals/lfg_list/catalog.rs:343`.

Existing tests (unexecuted; related/bounded only): `src/loader/tests/wow_api_globals/patch_12_1_service_payloads.rs:529`.

Cached contract: `CACHE/Blizzard_APIDocumentationGenerated/LFGListInfoDocumentation.lua:933`.

Contract qualification: INFERRED: fixture censorship policy; no server filtering proof.

Prior 12.1 cross-reference (status NOT reused): `data/patch-api/12.1-behaviors.json:1369`.

### L351 `structures-LfgSearchResultData-351` — implemented-needs-proof

Source: `LfgSearchResultData (C_LFGList.GetSearchResultInfo)`

Assessment / ledger note: Retail search serializer publishes censored=false. Prove bounded field shape from a nonempty listing and independent results; actual censorship transitions are not implemented or claimed.

Current producer or adjacent incomplete producer: `src/lua_api/globals/lfg_list.rs:223`.

Existing tests (unexecuted; related/bounded only): `src/loader/tests/wow_api_globals/patch_12_1_service_payloads.rs:529`.

Cached contract: `CACHE/Blizzard_APIDocumentationGenerated/LFGListInfoDocumentation.lua:965`.

Prior 12.1 cross-reference (status NOT reused): `data/patch-api/12.1-behaviors.json:1369`.

### L352 `structures-LfgSearchResultData-352` — implemented-needs-proof

Source: `+ censored`

Assessment / ledger note: Retail search serializer publishes censored=false. Prove bounded field shape from a nonempty listing and independent results; actual censorship transitions are not implemented or claimed.

Current producer or adjacent incomplete producer: `src/lua_api/globals/lfg_list.rs:223`.

Existing tests (unexecuted; related/bounded only): `src/loader/tests/wow_api_globals/patch_12_1_service_payloads.rs:529`.

Cached contract: `CACHE/Blizzard_APIDocumentationGenerated/LFGListInfoDocumentation.lua:975`.

Prior 12.1 cross-reference (status NOT reused): `data/patch-api/12.1-behaviors.json:1369`.

### L353 `structures-PetJournalPetInfo-353` — modelable

Source: `PetJournalPetInfo (C_PetJournal.GetPetInfoTableByPetID, C_PetJournal.GetPetInfoTableBySpeciesID)`

Assessment / ledger note: Species-ID table producer provides only a partial schema with canAttachToDecor=false/creatureModelScale=1; no GetPetInfoTableByPetID producer was found. Model both documented getters, complete named fields, nullable fields and luaIndex type over explicit pet state.

Current producer or adjacent incomplete producer: `src/lua_api/globals/font_strings_collection/pet_journal.rs:209`.

Existing tests (unexecuted; related/bounded only): `src/loader/tests/wow_api_globals/patch_12_1_service_payloads.rs:529`.

Cached contract: `CACHE/Blizzard_APIDocumentationGenerated/PetJournalInfoDocumentation.lua:483`.

Contract qualification: INFERRED: optional field data and deterministic pet eligibility fixtures; current docs define field/type/optionality.

Prior 12.1 cross-reference (status NOT reused): `data/patch-api/12.1-behaviors.json:1345`.

### L354 `structures-PetJournalPetInfo-354` — modelable

Source: `# [3].Nilable false -> true`

Assessment / ledger note: Species-ID table producer provides only a partial schema with canAttachToDecor=false/creatureModelScale=1; no GetPetInfoTableByPetID producer was found. Model both documented getters, complete named fields, nullable fields and luaIndex type over explicit pet state.

Current producer or adjacent incomplete producer: `src/lua_api/globals/font_strings_collection/pet_journal.rs:209`.

Existing tests (unexecuted; related/bounded only): `src/loader/tests/wow_api_globals/patch_12_1_service_payloads.rs:529`.

Cached contract: `CACHE/Blizzard_APIDocumentationGenerated/PetJournalInfoDocumentation.lua:489`.

Contract qualification: INFERRED: optional field data and deterministic pet eligibility fixtures; current docs define field/type/optionality.

Prior 12.1 cross-reference (status NOT reused): `data/patch-api/12.1-behaviors.json:1345`.

### L355 `structures-PetJournalPetInfo-355` — modelable

Source: `# [4].Nilable false -> true`

Assessment / ledger note: Species-ID table producer provides only a partial schema with canAttachToDecor=false/creatureModelScale=1; no GetPetInfoTableByPetID producer was found. Model both documented getters, complete named fields, nullable fields and luaIndex type over explicit pet state.

Current producer or adjacent incomplete producer: `src/lua_api/globals/font_strings_collection/pet_journal.rs:209`.

Existing tests (unexecuted; related/bounded only): `src/loader/tests/wow_api_globals/patch_12_1_service_payloads.rs:529`.

Cached contract: `CACHE/Blizzard_APIDocumentationGenerated/PetJournalInfoDocumentation.lua:490`.

Contract qualification: INFERRED: optional field data and deterministic pet eligibility fixtures; current docs define field/type/optionality.

Prior 12.1 cross-reference (status NOT reused): `data/patch-api/12.1-behaviors.json:1345`.

### L356 `structures-PetJournalPetInfo-356` — modelable

Source: `# [5].Nilable false -> true`

Assessment / ledger note: Species-ID table producer provides only a partial schema with canAttachToDecor=false/creatureModelScale=1; no GetPetInfoTableByPetID producer was found. Model both documented getters, complete named fields, nullable fields and luaIndex type over explicit pet state.

Current producer or adjacent incomplete producer: `src/lua_api/globals/font_strings_collection/pet_journal.rs:209`.

Existing tests (unexecuted; related/bounded only): `src/loader/tests/wow_api_globals/patch_12_1_service_payloads.rs:529`.

Cached contract: `CACHE/Blizzard_APIDocumentationGenerated/PetJournalInfoDocumentation.lua:491`.

Contract qualification: INFERRED: optional field data and deterministic pet eligibility fixtures; current docs define field/type/optionality.

Prior 12.1 cross-reference (status NOT reused): `data/patch-api/12.1-behaviors.json:1345`.

### L357 `structures-PetJournalPetInfo-357` — modelable

Source: `# [6].Nilable false -> true`

Assessment / ledger note: Species-ID table producer provides only a partial schema with canAttachToDecor=false/creatureModelScale=1; no GetPetInfoTableByPetID producer was found. Model both documented getters, complete named fields, nullable fields and luaIndex type over explicit pet state.

Current producer or adjacent incomplete producer: `src/lua_api/globals/font_strings_collection/pet_journal.rs:209`.

Existing tests (unexecuted; related/bounded only): `src/loader/tests/wow_api_globals/patch_12_1_service_payloads.rs:529`.

Cached contract: `CACHE/Blizzard_APIDocumentationGenerated/PetJournalInfoDocumentation.lua:492`.

Contract qualification: INFERRED: optional field data and deterministic pet eligibility fixtures; current docs define field/type/optionality.

Prior 12.1 cross-reference (status NOT reused): `data/patch-api/12.1-behaviors.json:1345`.

### L358 `structures-PetJournalPetInfo-358` — modelable

Source: `# [7].Nilable false -> true`

Assessment / ledger note: Species-ID table producer provides only a partial schema with canAttachToDecor=false/creatureModelScale=1; no GetPetInfoTableByPetID producer was found. Model both documented getters, complete named fields, nullable fields and luaIndex type over explicit pet state.

Current producer or adjacent incomplete producer: `src/lua_api/globals/font_strings_collection/pet_journal.rs:209`.

Existing tests (unexecuted; related/bounded only): `src/loader/tests/wow_api_globals/patch_12_1_service_payloads.rs:529`.

Cached contract: `CACHE/Blizzard_APIDocumentationGenerated/PetJournalInfoDocumentation.lua:493`.

Contract qualification: INFERRED: optional field data and deterministic pet eligibility fixtures; current docs define field/type/optionality.

Prior 12.1 cross-reference (status NOT reused): `data/patch-api/12.1-behaviors.json:1345`.

### L359 `structures-PetJournalPetInfo-359` — modelable

Source: `# [9].Type number -> luaIndex`

Assessment / ledger note: Species-ID table producer provides only a partial schema with canAttachToDecor=false/creatureModelScale=1; no GetPetInfoTableByPetID producer was found. Model both documented getters, complete named fields, nullable fields and luaIndex type over explicit pet state.

Current producer or adjacent incomplete producer: `src/lua_api/globals/font_strings_collection/pet_journal.rs:209`.

Existing tests (unexecuted; related/bounded only): `src/loader/tests/wow_api_globals/patch_12_1_service_payloads.rs:529`.

Cached contract: `CACHE/Blizzard_APIDocumentationGenerated/PetJournalInfoDocumentation.lua:495`.

Contract qualification: INFERRED: optional field data and deterministic pet eligibility fixtures; current docs define field/type/optionality.

Prior 12.1 cross-reference (status NOT reused): `data/patch-api/12.1-behaviors.json:1345`.

### L360 `structures-PetJournalPetInfo-360` — modelable

Source: `# [14].Nilable false -> true`

Assessment / ledger note: Species-ID table producer provides only a partial schema with canAttachToDecor=false/creatureModelScale=1; no GetPetInfoTableByPetID producer was found. Model both documented getters, complete named fields, nullable fields and luaIndex type over explicit pet state.

Current producer or adjacent incomplete producer: `src/lua_api/globals/font_strings_collection/pet_journal.rs:209`.

Existing tests (unexecuted; related/bounded only): `src/loader/tests/wow_api_globals/patch_12_1_service_payloads.rs:529`.

Cached contract: `CACHE/Blizzard_APIDocumentationGenerated/PetJournalInfoDocumentation.lua:500`.

Contract qualification: INFERRED: optional field data and deterministic pet eligibility fixtures; current docs define field/type/optionality.

Prior 12.1 cross-reference (status NOT reused): `data/patch-api/12.1-behaviors.json:1345`.

### L361 `structures-PetJournalPetInfo-361` — modelable

Source: `+ canAttachToDecor`

Assessment / ledger note: Species-ID table producer provides only a partial schema with canAttachToDecor=false/creatureModelScale=1; no GetPetInfoTableByPetID producer was found. Model both documented getters, complete named fields, nullable fields and luaIndex type over explicit pet state.

Current producer or adjacent incomplete producer: `src/lua_api/globals/font_strings_collection/pet_journal.rs:209`.

Existing tests (unexecuted; related/bounded only): `src/loader/tests/wow_api_globals/patch_12_1_service_payloads.rs:529`.

Cached contract: `CACHE/Blizzard_APIDocumentationGenerated/PetJournalInfoDocumentation.lua:505`.

Contract qualification: INFERRED: optional field data and deterministic pet eligibility fixtures; current docs define field/type/optionality.

Prior 12.1 cross-reference (status NOT reused): `data/patch-api/12.1-behaviors.json:1345`.

### L362 `structures-PetJournalPetInfo-362` — modelable

Source: `+ creatureModelScale`

Assessment / ledger note: Species-ID table producer provides only a partial schema with canAttachToDecor=false/creatureModelScale=1; no GetPetInfoTableByPetID producer was found. Model both documented getters, complete named fields, nullable fields and luaIndex type over explicit pet state.

Current producer or adjacent incomplete producer: `src/lua_api/globals/font_strings_collection/pet_journal.rs:209`.

Existing tests (unexecuted; related/bounded only): `src/loader/tests/wow_api_globals/patch_12_1_service_payloads.rs:529`.

Cached contract: `CACHE/Blizzard_APIDocumentationGenerated/PetJournalInfoDocumentation.lua:506`.

Contract qualification: INFERRED: optional field data and deterministic pet eligibility fixtures; current docs define field/type/optionality.

Prior 12.1 cross-reference (status NOT reused): `data/patch-api/12.1-behaviors.json:1345`.

### L363 `structures-PlaySoundParams-363` — modelable

Source: `PlaySoundParams (C_Sound.PlaySoundWithOptions)`

Assessment / ledger note: Current PlaySoundWithOptions compatibility path is a no-op. Model and validate volumeOverride in the sound request/options state, with observable queued request or recording sink; no hardware/service restart needed.

Current producer or adjacent incomplete producer: `src/lua_api/workarounds/temporary/sound_driver_defaults.rs:47`.

Existing tests (unexecuted; related/bounded only): `src/loader/tests/wow_api_globals/startup_globals.rs:823`.

Cached contract: `CACHE/Blizzard_APIDocumentationGenerated/SoundDocumentation.lua:125`.

Contract qualification: INFERRED: volume range/unknown-option policy if no exact native declaration supplies it.

### L364 `structures-PlaySoundParams-364` — modelable

Source: `+ volumeOverride`

Assessment / ledger note: Current PlaySoundWithOptions compatibility path is a no-op. Model and validate volumeOverride in the sound request/options state, with observable queued request or recording sink; no hardware/service restart needed.

Current producer or adjacent incomplete producer: `src/lua_api/workarounds/temporary/sound_driver_defaults.rs:47`.

Existing tests (unexecuted; related/bounded only): `src/loader/tests/wow_api_globals/startup_globals.rs:823`.

Cached contract: `CACHE/Blizzard_APIDocumentationGenerated/SoundDocumentation.lua:134`.

Contract qualification: INFERRED: volume range/unknown-option policy if no exact native declaration supplies it.

### L365 `structures-PlayerChoiceInfo-365` — modelable

Source: `PlayerChoiceInfo (C_PlayerChoice.GetCurrentPlayerChoiceInfo)`

Assessment / ledger note: C_PlayerChoice serializes seeded local choice state but omits hideAnswerArt. Add the boolean state/serialization and prove live replacement through GetCurrentPlayerChoiceInfo.

Current producer or adjacent incomplete producer: `src/c_api/c_player_choice.rs:116`.

Existing tests (unexecuted; related/bounded only): `src/loader/tests/wow_api_globals/patch_12_1_service_payloads.rs:561`.

Cached contract: `CACHE/Blizzard_APIDocumentationGenerated/PlayerChoiceDocumentation.lua:100`.

Contract qualification: INFERRED: default/local fixture state; non-nil boolean type is declared.

Prior 12.1 cross-reference (status NOT reused): `data/patch-api/12.1-behaviors.json:1393`.

### L366 `structures-PlayerChoiceInfo-366` — modelable

Source: `+ hideAnswerArt`

Assessment / ledger note: C_PlayerChoice serializes seeded local choice state but omits hideAnswerArt. Add the boolean state/serialization and prove live replacement through GetCurrentPlayerChoiceInfo.

Current producer or adjacent incomplete producer: `src/c_api/c_player_choice.rs:116`.

Existing tests (unexecuted; related/bounded only): `src/loader/tests/wow_api_globals/patch_12_1_service_payloads.rs:561`.

Cached contract: `CACHE/Blizzard_APIDocumentationGenerated/PlayerChoiceDocumentation.lua:112`.

Contract qualification: INFERRED: default/local fixture state; non-nil boolean type is declared.

Prior 12.1 cross-reference (status NOT reused): `data/patch-api/12.1-behaviors.json:1393`.

### L367 `structures-TieredEntranceTierInfo-367` — modelable

Source: `TieredEntranceTierInfo (C_DelvesUI.GetActiveDelveTier, C_DelvesUI.GetDelveEntranceTiers)`

Assessment / ledger note: Retail serializer emits overrideTooltipSpellID=nil and isLFG=false. Current declaration requires a non-nil overrideTooltipSpellID and uses queueAsLFG rather than isLFG; model/epoch-reconcile that delta instead of crediting plausible placeholders.

Current producer or adjacent incomplete producer: `src/lua_api/globals/missing_surface/delves_ui.rs:503`.

Existing tests (unexecuted; related/bounded only): `src/loader/tests/wow_api_globals/startup_globals.rs:834`.

Cached contract: `CACHE/Blizzard_APIDocumentationGenerated/DelvesUIDocumentation.lua:594`.

Contract qualification: INFERRED: historical isLFG name and tooltip default until pinned 69587 declaration is available; current-cache queueAsLFG must not silently replace captured source.

Prior 12.1 cross-reference (status NOT reused): `data/patch-api/12.1-behaviors.json:1417`.

### L368 `structures-TieredEntranceTierInfo-368` — modelable

Source: `+ overrideTooltipSpellID`

Assessment / ledger note: Retail serializer emits overrideTooltipSpellID=nil and isLFG=false. Current declaration requires a non-nil overrideTooltipSpellID and uses queueAsLFG rather than isLFG; model/epoch-reconcile that delta instead of crediting plausible placeholders.

Current producer or adjacent incomplete producer: `src/lua_api/globals/missing_surface/delves_ui.rs:503`.

Existing tests (unexecuted; related/bounded only): `src/loader/tests/wow_api_globals/startup_globals.rs:834`.

Cached contract: `CACHE/Blizzard_APIDocumentationGenerated/DelvesUIDocumentation.lua:600`.

Contract qualification: INFERRED: historical isLFG name and tooltip default until pinned 69587 declaration is available; current-cache queueAsLFG must not silently replace captured source.

Prior 12.1 cross-reference (status NOT reused): `data/patch-api/12.1-behaviors.json:1417`.

### L369 `structures-TieredEntranceTierInfo-369` — implemented-needs-proof

Source: `+ isLFG`

Assessment / ledger note: Existing Retail producer explicitly emits isLFG=false. Tests-only bounded captured-source shape proof; current cache uses queueAsLFG instead, so no historical/native or renamed-field parity credit.

Current producer or adjacent incomplete producer: `src/lua_api/globals/missing_surface/delves_ui.rs:503`.

Existing tests (unexecuted; related/bounded only): `src/loader/tests/wow_api_globals/startup_globals.rs:834`.

Cached contract: `CACHE/Blizzard_APIDocumentationGenerated/DelvesUIDocumentation.lua:594`.

Contract qualification: INFERRED: historical isLFG name and tooltip default until pinned 69587 declaration is available; current-cache queueAsLFG must not silently replace captured source.

Prior 12.1 cross-reference (status NOT reused): `data/patch-api/12.1-behaviors.json:1417`.

### L370 `structures-UnitPrivateAuraAnchorInfo-370` — implemented-needs-proof

Source: `UnitPrivateAuraAnchorInfo`

Assessment / ledger note: Anchor output serializer publishes current flags without showCountdownFrame; prove public/callback result isolation and true/false values. This is structure shape, not private-aura rendering parity.

Current producer or adjacent incomplete producer: `src/c_api/private_aura_anchors.rs:201`.

Existing tests (unexecuted; related/bounded only): `tests/private_aura_anchors.rs:267`.

Cached contract: `CACHE/Blizzard_APIDocumentationGenerated/UnitConstantsDocumentation.lua:66`.

### L371 `structures-UnitPrivateAuraAnchorInfo-371` — implemented-needs-proof

Source: `- showCountdownFrame`

Assessment / ledger note: Anchor output serializer publishes current flags without showCountdownFrame; prove public/callback result isolation and true/false values. This is structure shape, not private-aura rendering parity.

Current producer or adjacent incomplete producer: `src/c_api/private_aura_anchors.rs:201`.

Existing tests (unexecuted; related/bounded only): `tests/private_aura_anchors.rs:267`.

Cached contract: `CACHE/Blizzard_APIDocumentationGenerated/UnitConstantsDocumentation.lua:66`.

### L372 `structures-UnitPrivateAuraAnchorInfo-372` — implemented-needs-proof

Source: `+ showDispelIcon`

Assessment / ledger note: Anchor output serializer publishes current flags without showCountdownFrame; prove public/callback result isolation and true/false values. This is structure shape, not private-aura rendering parity.

Current producer or adjacent incomplete producer: `src/c_api/private_aura_anchors.rs:201`.

Existing tests (unexecuted; related/bounded only): `tests/private_aura_anchors.rs:267`.

Cached contract: `CACHE/Blizzard_APIDocumentationGenerated/UnitConstantsDocumentation.lua:76`.

### L373 `structures-UnitPrivateAuraAnchorInfo-373` — implemented-needs-proof

Source: `+ showCooldownEdge`

Assessment / ledger note: Anchor output serializer publishes current flags without showCountdownFrame; prove public/callback result isolation and true/false values. This is structure shape, not private-aura rendering parity.

Current producer or adjacent incomplete producer: `src/c_api/private_aura_anchors.rs:201`.

Existing tests (unexecuted; related/bounded only): `tests/private_aura_anchors.rs:267`.

Cached contract: `CACHE/Blizzard_APIDocumentationGenerated/UnitConstantsDocumentation.lua:74`.

### L374 `structures-UnitPrivateAuraAnchorInfo-374` — implemented-needs-proof

Source: `+ showCooldownFrame`

Assessment / ledger note: Anchor output serializer publishes current flags without showCountdownFrame; prove public/callback result isolation and true/false values. This is structure shape, not private-aura rendering parity.

Current producer or adjacent incomplete producer: `src/c_api/private_aura_anchors.rs:201`.

Existing tests (unexecuted; related/bounded only): `tests/private_aura_anchors.rs:267`.

Cached contract: `CACHE/Blizzard_APIDocumentationGenerated/UnitConstantsDocumentation.lua:73`.

### L377 `source-context-377` — metadata-only

Source: `== Deprecated API ==`

Assessment: Editorial heading, chronology, navigation, file label or disclaimer only; no runtime credit.

### L378 `source-context-378` — metadata-only

Source: `Blizzard_Deprecated/Deprecated_12_1_0.lua`

Assessment: Editorial heading, chronology, navigation, file label or disclaimer only; no runtime credit.

### L380 `deprecated api-getglobal-380` — implemented-needs-proof

Source: `getglobal`

Assessment / ledger note: Native global helper and unmodified cached deprecated wrapper both exist. Tests only: raw/ordinary lookup, canonical dotted lookup, write/read/error behavior and actual deprecated-addon loading. Deprecation is not equivalent to complete native absence.

Current producer or adjacent incomplete producer: `src/lua_api/globals/utility_system_spell/mod.rs:194`, `CACHE/Blizzard_Deprecated/Shared/Deprecated_12_1_0.lua:9`.

Existing tests (unexecuted; related/bounded only): `patch-tests/patch_12_1/strict_removal_timing.rs:96`.

Cached contract: Page statement plus current producer cited above; no exact generated declaration anchor located for the entire combined statement.

Prior 12.1 cross-reference (status NOT reused): `data/patch-api/12.1-behaviors.json:1553`.

### L381 `deprecated api-setglobal-381` — implemented-needs-proof

Source: `setglobal`

Assessment / ledger note: Native global helper and unmodified cached deprecated wrapper both exist. Tests only: raw/ordinary lookup, canonical dotted lookup, write/read/error behavior and actual deprecated-addon loading. Deprecation is not equivalent to complete native absence.

Current producer or adjacent incomplete producer: `src/lua_api/globals/utility_system_spell/mod.rs:221`, `CACHE/Blizzard_Deprecated/Shared/Deprecated_12_1_0.lua:14`.

Existing tests (unexecuted; related/bounded only): `patch-tests/patch_12_1/strict_removal_timing.rs:96`.

Cached contract: Page statement plus current producer cited above; no exact generated declaration anchor located for the entire combined statement.

Prior 12.1 cross-reference (status NOT reused): `data/patch-api/12.1-behaviors.json:1553`.

### L383 `source-context-383` — metadata-only

Source: `Blizzard_DeprecatedBattleNet/Deprecated_BattleNet.lua`

Assessment: Editorial heading, chronology, navigation, file label or disclaimer only; no runtime credit.

### L385 `deprecated api-BNSendVerifiedBattleTagInvite-385` — implemented-needs-proof

Source: `BNSendVerifiedBattleTagInvite -> C_BattleNet.SendVerifiedBattleNetFriendInvite`

Assessment / ledger note: Current native Battle.net replacement functions record/query explicit local invite state; cached deprecated aliases forward to them. Tests only for both wrapper/backend routes, duplicates, absent invites, exact arguments and result shape; no network delivery claim.

Current producer or adjacent incomplete producer: `CACHE/Blizzard_DeprecatedBattleNet/Deprecated_BattleNet.lua:25`, `src/ptr/compat_bootstrap.rs:51`, `src/c_api/c_battle_net.rs:1`, `src/c_api/c_battle_net.rs:155`, `src/c_api/c_battle_net.rs:149`.

Existing tests (unexecuted; related/bounded only): `src/loader/tests/wow_api_globals/startup_globals.rs:894`.

Cached contract: Page statement plus current producer cited above; no exact generated declaration anchor located for the entire combined statement.

### L386 `deprecated api-BNGetFriendInviteInfo-386` — implemented-needs-proof

Source: `BNGetFriendInviteInfo -> C_BattleNet.GetFriendInviteInfo`

Assessment / ledger note: Current native Battle.net replacement functions record/query explicit local invite state; cached deprecated aliases forward to them. Tests only for both wrapper/backend routes, duplicates, absent invites, exact arguments and result shape; no network delivery claim.

Current producer or adjacent incomplete producer: `CACHE/Blizzard_DeprecatedBattleNet/Deprecated_BattleNet.lua:29`, `src/ptr/compat_bootstrap.rs:51`, `src/c_api/c_battle_net.rs:1`, `src/c_api/c_battle_net.rs:155`, `src/c_api/c_battle_net.rs:149`.

Existing tests (unexecuted; related/bounded only): `src/loader/tests/wow_api_globals/startup_globals.rs:894`.

Cached contract: Page statement plus current producer cited above; no exact generated declaration anchor located for the entire combined statement.

### L388 `source-context-388` — metadata-only

Source: `Blizzard_DeprecatedHousing/Deprecated_Housing.lua`

Assessment: Editorial heading, chronology, navigation, file label or disclaimer only; no runtime credit.

### L390 `deprecated api-C_DyeColor-GetDyeColorForItem-390` — modelable

Source: `C_DyeColor.GetDyeColorForItem -> C_DyeColor.GetDyeColorsForItem`

Assessment / ledger note: Cached singular-to-plural dye wrappers exist, but plural native queries return empty compatibility lists because item/dye data is unmodeled. Model an explicit item/location-to-color catalog, then prove first-result/nil legacy forwarding. INFERRED: deterministic local catalog and invalid-location policy.

Current producer or adjacent incomplete producer: `CACHE/Blizzard_Deprecated/Mainline/Deprecated_12_1_0.lua:39`, `CACHE/Blizzard_Deprecated/Mainline/Deprecated_12_1_0.lua:49`, `src/ptr/compat_bootstrap.rs:51`, `src/lua_api/workarounds/temporary/dye_color_defaults.rs:3`.

Existing tests (unexecuted; related/bounded only): `patch-tests/patch_12_1/strict_removal_timing.rs:96`.

Cached contract: Page statement plus current producer cited above; no exact generated declaration anchor located for the entire combined statement.

Prior 12.1 cross-reference (status NOT reused): `data/patch-api/12.1-behaviors.json:1553`.

### L391 `deprecated api-C_DyeColor-GetDyeColorForItemLocation-391` — modelable

Source: `C_DyeColor.GetDyeColorForItemLocation -> C_DyeColor.GetDyeColorsForItemLocation`

Assessment / ledger note: Cached singular-to-plural dye wrappers exist, but plural native queries return empty compatibility lists because item/dye data is unmodeled. Model an explicit item/location-to-color catalog, then prove first-result/nil legacy forwarding. INFERRED: deterministic local catalog and invalid-location policy.

Current producer or adjacent incomplete producer: `CACHE/Blizzard_Deprecated/Mainline/Deprecated_12_1_0.lua:49`, `src/ptr/compat_bootstrap.rs:51`, `src/lua_api/workarounds/temporary/dye_color_defaults.rs:3`.

Existing tests (unexecuted; related/bounded only): `patch-tests/patch_12_1/strict_removal_timing.rs:96`.

Cached contract: Page statement plus current producer cited above; no exact generated declaration anchor located for the entire combined statement.

Prior 12.1 cross-reference (status NOT reused): `data/patch-api/12.1-behaviors.json:1553`.

### L393 `source-context-393` — metadata-only

Source: `Blizzard_DeprecatedRaidWarning/Deprecated_RaidWarning.lua`

Assessment: Editorial heading, chronology, navigation, file label or disclaimer only; no runtime credit.

### L395 `deprecated api-RaidNotice_AddMessage-395` — implemented-needs-proof

Source: `RaidNotice_AddMessage -> RaidWarningUtil.AddMessage`

Assessment / ledger note: Unmodified RaidWarningUtil/RaidWarningFrameMixin implement message addition and pool-backed clearing. Tests only: actual deprecated wrapper, message pool contents/visibility, per-type clearing and default Retail loading; text pixels and native timing are not claimed.

Current producer or adjacent incomplete producer: `CACHE/Blizzard_DeprecatedRaidWarning/Deprecated_RaidWarning.lua:8`, `src/ptr/compat_bootstrap.rs:51`, `CACHE/Blizzard_RaidWarning/RaidWarningUtil.lua:28`, `CACHE/Blizzard_RaidWarning/RaidWarning.lua:262`.

Existing tests (unexecuted; related/bounded only): No exact behavioral test located for this row; add concrete contract assertions, not symbol-presence checks.

Cached contract: Page statement plus current producer cited above; no exact generated declaration anchor located for the entire combined statement.

### L396 `deprecated api-RaidNotice_Clear-396` — implemented-needs-proof

Source: `RaidNotice_Clear -> RaidWarningFrameMixin:ClearMessages`

Assessment / ledger note: Unmodified RaidWarningUtil/RaidWarningFrameMixin implement message addition and pool-backed clearing. Tests only: actual deprecated wrapper, message pool contents/visibility, per-type clearing and default Retail loading; text pixels and native timing are not claimed.

Current producer or adjacent incomplete producer: `CACHE/Blizzard_DeprecatedRaidWarning/Deprecated_RaidWarning.lua:12`, `src/ptr/compat_bootstrap.rs:51`, `CACHE/Blizzard_RaidWarning/RaidWarningUtil.lua:28`, `CACHE/Blizzard_RaidWarning/RaidWarning.lua:262`.

Existing tests (unexecuted; related/bounded only): No exact behavioral test located for this row; add concrete contract assertions, not symbol-presence checks.

Cached contract: Page statement plus current producer cited above; no exact generated declaration anchor located for the entire combined statement.

