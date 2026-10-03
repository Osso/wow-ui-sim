# B98 identity verification — a4cce2db1

## Evidence and limits

Read-only artifact verification. HEAD is a4cce2db1639d7d88752925d06facc78f4ee5746. Read `git show a4cce2db1`, scoped diff, parent producers, source register, implementations, specs and behavioral tests. Repository remained clean. No cargo, test binaries, agents, model CLIs or repository writes.

[EXIST] PASS — all scoped implementations, specs and tests present and opened.
[SUBSTANTIVE] PASS — GUID/map/roster/ownership classification and stored model bindings are real host-state behavior, not placeholders.
[WIRED] PASS — getters register in group_queries.rs:63–64 and globals/register.rs:178; shared predicate is called from unit_misc.rs:192 and model/model_unit.rs:28; frame registration reaches widgets via env_init/frames.rs:61 and installs final SetUnit dispatcher at widgets/mod.rs:100.
[ANTI-PATTERN] PASS — zero TODO/FIXME/HACK/XXX/except-pass markers in added scoped producer/test lines. Existing 3D stubs are intentional scope exclusions, not the identity implementation.

404/404 GREEN and startup results are caller/commit-reported, not independently executed here. Source contains six instanced-identity tests and ten cast_events_identity tests (four cast, six model), not the supplied count of nine. Reconcile the actual run ledger before claiming all ten ran.

## 1. Instanced identity — ACCEPT WITH QUALIFICATIONS

**Recommended prose 197/198/199 status: partial**, with bounded credit for resolved player/party/target/focus identities. Prose says identities are restricted on instanced maps except player, pet/vehicle and group members; line 198 describes historical behavior, not a requirement to restore attackable-only policy.

### Implementation and default-state changes

instanced_identity.rs:17–39 resolves an existing GUID, applies explicit classification first, then local-player/group/player-owned exemptions, then the instance-map flag. Mind-control state deliberately does not decide secrecy. Default map flag is false and classification/ownership sets are empty (state.rs:220–227; instanced_identity.rs:7–15).

Fresh, unpolluted-registry behavior:

| Caller | Parent → commit behavior |
|---|---|
| UnitName / UnitNameUnmodified, existing party/raid | Non-Unknown roster names automatically secret → public without explicit classification. Inactive/missing names remain public Unknown. |
| GetUnitName / UnitPVPName, party tokens | Token spelling marked names, including Unknown and names obtained from an inactive seeded roster → public by default. |
| UnitFullName, party tokens | Name and shared SimRealm string marked secret → both returned public by default; restricted calls now return independently wrapped secret values. |
| GetUnitName / UnitPVPName / UnitFullName, raid tokens | These producers already returned Unknown rather than resolved raid names and did not directly classify raid spelling; payload limitation remains. No newly implemented raid identity. |
| UnitGUID, existing player/party/target/focus | Already public → still public by default. New behavior: secret when explicit classification or instance restriction applies. Raid/pet/vehicle GUIDs remain nil. |

UnitName/UnitNameUnmodified raid names become public even though raid GUID resolution remains absent. Player/target/focus ordinary default outputs retain their values and arity. Pet/vehicle name aliases retain the player-name placeholder. Missing party tokens no longer cause GetUnitName/UnitPVPName/UnitFullName to mark Unknown/SimRealm globally.

Beyond default state, nongroup friendly and attackable target/focus identities now become secret on instances; aliases follow GUID membership rather than token spelling. Explicit classification can restrict even player/group/owned identities, an explicitly inferred precedence rather than native-proven semantics. Restricted getters no longer incidentally mark equal interned literals; retained secret results remain secret after map exit. unit_misc.rs:63–80, group_queries.rs:461–488.

These changes match the retained prose for modeled categories. They do not establish whole-native parity. GetRaidRosterInfo still marks interned roster names at group_queries.rs:335, so subsequent equal-name results may remain secret despite public host classification; spec explicitly excludes that producer. Do not promise universal public outputs after arbitrary caller history.

Older/non-scoped identity producers retain their old branches: unit_misc.rs:55–61, :72–79, :158–162; group_queries.rs:482–490. No legacy-profile execution proof was supplied.

### Tests, fixture legitimacy and checkbox credit

All six instanced_identity tests assert real getter values/arity/secrecy, host-state transitions, actual attackability contrast, GUID aliases, group exit, ownership override and genuine addon taint. With new fixtures/types retained but old producers restored, each contains a discriminating assertion: visitors would remain public, party names would remain secret, or instance/override restrictions would not apply. No source-shape assertions.

security_api.rs:864–918 changes three tests: party-name secrecy, name/realm secrecy, and table accessibility. Seeding an explicit secret GUID is legitimate for those secret-value contracts; it intentionally drops automatic-party-secrecy coverage. unit_token_identity_secrecy.rs:107–129 captures public player GUID before classification. Legitimate isolation of reverse-lookup policy from secret-argument authentication; it does not prove UnitTokenFromGUID accepts the newly secret UnitGUID output. UnitName secret-token input tests remain unchanged.

| Spec checkbox | Earned evidence / limit |
|---|---|
| instanced-identity.md:7 | Friendly/enemy visitor instance restrictions; test :44–63. |
| :8 | Party/target/focus group aliases, local player and declared ownership; tests :86–131 and :148–165. No actual pet/vehicle/raid identities. |
| :9 | Controlled ally remains exempt; controlled visitor remains restricted; tests :86–145. |
| :10 | All six getter families and arity checked by helper :21–41. Exact public payload recovery asserted for target UnitName/UnitGUID, not every getter/token combination. |
| :11 | Retained target name/GUID, new public results and equal-name literal; tests :60–82. No direct equal-realm-literal non-poisoning assertion. |
| :12 | Owned visitor override and actual addon name/GUID classification/taint; tests :148–190. Not every exemption × caller × getter combination. |

Unresolved GUID categories remain real gaps (spec :39–40; unit_misc.rs:207–213). Quest NPC/arena/boss/nameplate behavior receives no implementation credit. Spec :37 is stale: it still calls implemented work staged/unrun. Native precedence/lifetime proof remains absent (:38).

## 2. Model unit identity guard — ACCEPT WITH QUALIFICATIONS

Exact consolidated rows are **widgets-PlayerModel-SetUnit-534** and **widgets-ModelSceneActorBase-SetModelByUnit-542**, both `+ RequiresDeclassifiedUnitIdentity`; these are widget rows, not global functions. Prose 194/195 adds secret-identity denial with nil instead of success, without error.

**Recommended:** row 542 bounded; row 534 partial until actual PlayerModel denial/binding routing is behaviorally tested. Prose 194/195 gets bounded non-3D credit for tested Model and scene actor, not whole-model/native loading parity.

### Guard, dispatch and profile change

model/model_unit.rs:23–43 uses the same unit_misc.rs:188–193 predicate as getters. Secret identities return exactly one public nil before binding mutation; missing identities return false; public existing identities store last_unit and return true. Assignment success is explicitly simulated binding, not successful 3D loading. The guard authenticates the token before classification; no broader optional-secret-argument/AllowedWhenUntainted claim is earned.

widgets/mod.rs:65–78 dispatches Model, ModelScene and PlayerModel to the model handler; every other resolved widget type reaches tooltip::set_tooltip_unit, which forwards unchanged at tooltip.rs:107–109. GameTooltip therefore still executes content.rs:510–522, populates content, stores displayed unit and fires OnTooltipSetUnit. Existing behavioral controls are tooltip_item_spell.rs:312–382, not tooltip_basic alone; their execution was not explicitly included in the caller's named proof list.

DressUpModel/CinematicModel/TabardModel map to WidgetType::PlayerModel (widget/mod.rs:140–145); no separate variants exist. A scene actor is internally WidgetType::Frame with object_type_name ModelSceneActor (model_scene_actors.rs:12–24). Its SetModelByUnit reaches the direct model registration; its SetUnit would take the tooltip branch, consistent with dispatcher scope rather than evidence for an actor SetUnit API.

**Cross-profile behavior change:** dispatcher installation is unconditional. Before this commit, legacy Model/PlayerModel/ModelScene:SetUnit accidentally executed the later tooltip handler; now non-scoped profiles execute SKIP_3D_RENDERING and return zero results (model.rs:341–357). Other-profile identity predicates stay unchanged, but actual SetUnit behavior does not. This makes the spec's other-profile-methods-unchanged statement (:24) inaccurate. It is a dispatch correction, not evidence of a newly modeled legacy guard; no legacy behavioral test covers it.

### Tests and checkbox credit

Six model tests at cast_events_identity.rs:217–357 assert exact pcall arity, nil/public denial, preserved prior bindings, GUID aliases, public assignment, host clearing, actual addon taint, environment separation, instance restriction/ownership/map exit and missing-identity failure. They would fail with parent producers: actor setter always stored/returned true; Model:SetUnit executed tooltip behavior instead of preserving a guarded model binding.

model-unit-identity-guard.md:7–13 are earned for the tested Model/scene-actor/public-token boundary. :10 and :11 remain simulator policies, not native success/missing-unit semantics. Tests create Model and scene actor, not PlayerModel/DressUpModel/ModelScene:SetUnit (fixture :146–148). Existing PlayerModel test at widget_methods_model.rs:112 makes an unchecked call with an unresolved token; it does not prove row 534 denial or assignment. Add behavior coverage before crediting that exact row as bounded.

### Remaining method-name collisions

No surviving model-versus-tooltip table intersection. SetUnit is now registered once through the dispatcher. Other overwrites remain pre-existing, not introduced by this commit:

| Name(s) | Registrations / winning handler |
|---|---|
| SetRotation | texture/mod.rs:59 vs model.rs:638; model wins. Both write frame.rotation for ordinary numeric inputs, but parsing differs. |
| SetViewTranslation | model.rs:702 vs :786; scene implementation replaces 3D stub inside the same table. |
| GetNumLines | text_attribute_event/mod.rs:248 vs tooltip.rs:27; tooltip wins on shared metatable. |
| Set/GetShadowOffset, Set/GetShadowColor | text_attribute_event/mod.rs:225–228 vs texture/mod.rs:22–25; texture wins. |
| ClearText; Set/GetMaxLines; Set/GetIndentedWordWrap | text_attribute_event/mod.rs:199, :271–272, :349–355 vs message_frame/mod.rs:32, :35–36, :69–70; message-frame handlers win. Per-instance SetMaxLines installation at :110 is separate. |

Registration order is env_init/frames.rs:59–61 and widgets/mod.rs:87–100. These are registration-level findings, not newly executed regression cases; some handlers share underlying fields. Do not assume every duplicate is independently a user-visible defect.

## Defects / merge risk

1. **Coverage gap:** exact PlayerModel row 534 lacks behavioral guard/routing coverage; tests/cast_events_identity.rs:146–148 only constructs Model and actor. Broader variant dispatch is source-proven, not test-proven.
2. **Cross-profile semantic change:** widgets/mod.rs:65–78, :100 changes legacy model SetUnit from tooltip result/side effects to zero-result stub; model-unit-identity-guard.md:24 claims unchanged other-profile methods. Document and behaviorally verify intentional scope.
3. **Stale spec:** model-unit-identity-guard.md:34 incorrectly says tooltip still overwrites SetUnit; :35 and instanced-identity.md:37 retain pending execution/staging statements. No unsupported independent runtime claim made here.
4. **Proof-count mismatch:** tests/cast_events_identity.rs contains ten tests; caller reports nine. Determine which revision/filter the run covered. This is evidence bookkeeping, not proof of a failed test.

No new defect found in the bounded secret-identity predicate, denial arity or stored-binding preservation. Merge risk is widened legacy dispatch plus overstated exact-row/profile/test coverage. Both items accepted only with the qualifications above; whole-native identity coverage is not earned.
