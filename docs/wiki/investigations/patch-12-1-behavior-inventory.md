# Patch 12.1 Broader Behavior Inventory
Non-FrameXML behavioral fidelity register. Family names group rows; status and evidence requirements remain item-specific.

## Content
- **Source:** `data/patch-api/sources/12.1-behaviors.json`
- **Source SHA-256:** `9e3ad69306c0e3d377e3aba6f5b928a21cd868ff98e5e9e43763385cdcecc83f`
- **Target:** PTR build `12.1.0`
- **Rows:** 54 changed behavioral boundaries — 0 implemented, 36 best-effort, 18 evidence-required, 0 exception-requested, 0 untriaged
- **Resolution split:** 36 behavioral, 18 unsafe; no exception rows

The three new best-effort credits use simulator tests, not native captures: [duration-binding representation](../../specs/duration-text-binding.md) and [strict-removal timing](../../specs/strict-removal-timing.md). The frozen source register retains the original fidelity objectives; bounded modeled policies do not establish those objectives' native equivalence. Native probe preparation alone changes no disposition.

| Symbol | Machine Status | Candidate | Family | Direction | Contract |
|---|---|---|---|---|---|
| `Patch12_1.UnitAura.AddonSecretError` | evidence-required  | unsafe  | UnitAura secrecy | changed | Addon-tainted UnitAura and aura access raise the retail secret-value error shape. |
| `Patch12_1.UnitAura.BlizzardSecretAccess` | evidence-required  | unsafe  | UnitAura secrecy | changed | Blizzard/internal callers receive the permitted secret-aura behavior distinct from addon-tainted callers. |
| `Patch12_1.UnitAura.SecretAuraData` | evidence-required  | unsafe  | UnitAura secrecy | changed | Fully secret AuraData fields remain inaccessible to addons while preserving the retail object shape. |
| `Patch12_1.UnitAura.SecretEventPayload` | evidence-required  | unsafe  | UnitAura secrecy | changed | Secret UNIT_AURA payload values preserve retail secrecy and tuple shape. |
| `Patch12_1.PrivateScriptObjects.PrivateIdentity` | evidence-required  | unsafe  | Private Script Objects | changed | Private or forbidden objects have identity distinct from their public frame view. |
| `Patch12_1.PrivateScriptObjects.InaccessiblePublicKeys` | evidence-required  | unsafe  | Private Script Objects | changed | Private keys remain inaccessible through the public object. |
| `Patch12_1.PrivateScriptObjects.ChildVisibility` | evidence-required  | unsafe  | Private Script Objects | changed | Public traversal cannot expose forbidden or private children. |
| `Patch12_1.PrivateScriptObjects.HookBoundary` | evidence-required  | unsafe  | Private Script Objects | changed | Hooks cannot cross private or forbidden partitions except through permitted delegates. |
| `Patch12_1.PrivateScriptObjects.ScriptStorage` | evidence-required  | unsafe  | Private Script Objects | changed | Script handlers stored in private partitions are not publicly readable or writable. |
| `Patch12_1.PrivateScriptObjects.SecureDelegateEnforcement` | evidence-required  | unsafe  | Private Script Objects | changed | Public delegates invoke permitted private behavior without exposing private receiver state. |
| `Patch12_1.ForbiddenAspects.UntrustedScriptExecution` | evidence-required  | unsafe  | Forbidden Aspects | changed | Operations requiring trusted script execution reject insecure callers. |
| `Patch12_1.ForbiddenAspects.UntrustedLayoutScriptExecution` | evidence-required  | unsafe  | Forbidden Aspects | changed | Layout-script operations reject insecure callers lacking the required aspect. |
| `Patch12_1.ForbiddenAspects.EventRegistrations` | evidence-required  | unsafe  | Forbidden Aspects | changed | Event registration operations enforce the EventRegistrations aspect restriction. |
| `Patch12_1.ForbiddenAspects.AlwaysPropagateInput` | evidence-required  | unsafe  | Forbidden Aspects | changed | Input propagation changes enforce the AlwaysPropagateInput aspect restriction. |
| `Patch12_1.ForbiddenAspects.ScriptedInput` | evidence-required  | unsafe  | Forbidden Aspects | changed | Scripted-input operations enforce the ScriptedInput aspect restriction. |
| `Patch12_1.ForbiddenAspects.QueryFocus` | evidence-required  | unsafe  | Forbidden Aspects | changed | Focus-query operations enforce the QueryFocus aspect restriction. |
| `Patch12_1.AuraContainer.CreationTypes` | best-effort  | behavioral  | AuraContainer | changed | AuraContainer, AuraButton, and ManagedAuraContainer can be created with compatible object types. |
| `Patch12_1.AuraContainer.Assignment` | best-effort  | behavioral  | AuraContainer | changed | Aura groups assign aura data to frames by auraInstanceID with compatible ownership; retained frames remain owned while removed entries are released. |
| `Patch12_1.AuraContainer.Filtering` | best-effort  | behavioral  | AuraContainer | changed | Aura groups apply HELPFUL/HARMFUL/PLAYER filtering to select compatible aura subsets. |
| `Patch12_1.AuraContainer.Sorting` | best-effort  | behavioral  | AuraContainer | changed | Aura groups honor configured comparator ordering; this does not claim the retail default comparator. |
| `Patch12_1.AuraContainer.PartitionPlacement` | best-effort  | behavioral  | AuraContainer | changed | Managed AuraContainer selects public-only, public-and-private, and edit-mode aura source partitions compatibly. |
| `Patch12_1.AuraContainer.ManagedButtonLifecycle` | best-effort  | behavioral  | AuraContainer | changed | Aura groups implement an acquire-release-reacquire lifecycle for managed frames as auraInstanceID entries change. |
| `Patch12_1.AuraContainer.TooltipBinding` | best-effort  | behavioral  | AuraContainer | changed | Aura buttons bind tooltip filter, aura-instance lookup, and leave-hide behavior. |
| `Patch12_1.AuraContainer.SecretVisibility` | evidence-required  | unsafe  | AuraContainer | changed | Secret aura values remain hidden while container and button structure stays usable. |
| `Patch12_1.TextureRadialProgress.Surface` | best-effort  | behavioral  | Texture radial progress | changed | A created Texture exposes the radial method family; no standalone constructor claim is made. Texture method availability and value storage are tested; exact retail clamping and visual rendering remain best-effort. |
| `Patch12_1.TextureRadialProgress.MethodDispatch` | best-effort  | behavioral  | Texture radial progress | changed | Radial progress methods dispatch on a Texture receiver. Texture method availability and value storage are tested; exact retail clamping and visual rendering remain best-effort. |
| `Patch12_1.TextureRadialProgress.StateBehavior` | best-effort  | behavioral  | Texture radial progress | changed | Texture-backed radial progress defaults, setters/getters, visual mode, and Clear reset are modeled. Texture method availability and value storage are tested; exact retail clamping and visual rendering remain best-effort. |
| `Patch12_1.DurationTextBinding.Lifetime` | best-effort  | behavioral  | DurationTextBinding | changed | A binding remains usable while retained by Lua references; exact Blizzard ownership and invalidation semantics remain unproven. |
| `Patch12_1.DurationTextBinding.StableIdentity` | best-effort  | behavioral  | DurationTextBinding | changed | Factory calls return distinct userdata handles with stable identity while referenced; this corrects the earlier table-backed model description. |
| `Patch12_1.DurationTextBinding.RepresentationFidelity` | best-effort  | behavioral  | DurationTextBinding | changed | Modeled userdata identity, raw table-access rejection, copied configuration, and reference retention through collection; exact native metatable, finalization, ownership/invalidation, and GC equivalence remain unproven. |
| `Patch12_1.DurationTextBinding.Formatter` | best-effort  | behavioral  | DurationTextBinding | changed | Duration formatting and interpolation use the documented compatible contract. |
| `Patch12_1.DurationTextBinding.ColorCurve` | best-effort  | behavioral  | DurationTextBinding | changed | Color-curve methods preserve compatible binding state. |
| `Patch12_1.DurationTextBinding.FontStringUpdate` | best-effort  | behavioral  | DurationTextBinding | changed | The binding updates a FontString through a documented compatible lifetime and update contract. |
| `Patch12_1.Service.Discord.OAuthState` | best-effort  | behavioral  | Service payloads | changed | Discord authorization and refresh state transitions expose compatible result payloads. |
| `Patch12_1.Service.Discord.GuildState` | best-effort  | behavioral  | Service payloads | changed | Discord guild link, unlink, and setting operations expose compatible state payloads. |
| `Patch12_1.Service.Discord.ServerChannelPayload` | best-effort  | behavioral  | Service payloads | changed | Discord server and channel lists, names, counts, and linkable-channel payloads are compatible. |
| `Patch12_1.Service.Housing.OwnedHouseState` | best-effort  | behavioral  | Service payloads | changed | Owned-house and plot state plus ResetHouse behavior follow the local compatibility model. |
| `Patch12_1.Service.Housing.BlueprintPayload` | best-effort  | behavioral  | Service payloads | changed | Housing blueprint export, import, and share-code payloads follow the local compatibility model. |
| `Patch12_1.Service.Housing.AvailabilityCodes` | best-effort  | behavioral  | Service payloads | changed | Housing availability and result codes plus import validation follow the local compatibility model. |
| `Patch12_1.Service.Housing.EditorDecorLayoutPayload` | best-effort  | behavioral  | Service payloads | changed | Housing editor, decor, room, budget, and floorplan payloads follow the local compatibility model. |
| `Patch12_1.Service.BattleNet.FriendInvitePayload` | best-effort  | behavioral  | Service payloads | changed | Verified Battle.net friend invite creation, deduplication, and info fields follow the local compatibility model. |
| `Patch12_1.Service.BattleNet.TitleFriendPayload` | best-effort  | behavioral  | Service payloads | changed | Battle.net title-friend custom names, tags, feature flags, and appear-offline state follow the local compatibility model. |
| `Patch12_1.Service.BattleNet.TitleFriendUnitInvite` | best-effort  | behavioral  | Service payloads | changed | Battle.net title-friend unit invite eligibility uses a documented deterministic compatibility result. |
| `Patch12_1.Service.EncounterJournal.DifficultyPayload` | best-effort  | behavioral  | Service payloads | changed | Encounter Journal base and valid difficulty IDs follow generated instance-data guesses. |
| `Patch12_1.Service.Cooldown.Payloads` | best-effort  | behavioral  | Service payloads | changed | Cooldown query structures, secret fields, and update payloads follow a documented compatibility contract. |
| `Patch12_1.Service.Pet.Payloads` | best-effort  | behavioral  | Service payloads | changed | Pet-related structures and state payloads follow a documented compatibility contract. |
| `Patch12_1.Service.LFG.Payloads` | best-effort  | behavioral  | Service payloads | changed | LFG service-result structures follow a documented compatibility contract. |
| `Patch12_1.Service.PlayerChoice.Payloads` | best-effort  | behavioral  | Service payloads | changed | Player-choice structures, options, and state payloads follow a documented compatibility contract. |
| `Patch12_1.Service.TieredEntrance.Payloads` | best-effort  | behavioral  | Service payloads | changed | C_DelvesUI TieredEntranceTierInfo rows expose tier, suggestedILvl, unlocked, tierDescription, modifierUIWidgetSetID, lockedReason, and rewards with id, quantity, rewardType, and context. Deterministic rows/rewards are modeled; live reward IDs, quantities, unlock timing, eligibility, and economics are not claimed. |
| `Patch12_1.Service.PrivateAura.Payloads` | evidence-required  | unsafe  | Service payloads | changed | Private-aura payloads preserve inaccessible and secret structural boundaries. |
| `Patch12_1.StrictRemoval.PreStartupVisibility` | best-effort  | behavioral  | Strict removal timing | changed | Selected compatibility publications remain through all first world-entry handlers, then retire before later startup events; this simulator boundary is assumed, not native timing. |
| `Patch12_1.StrictRemoval.BlizzardLoadCompatibility` | best-effort  | behavioral  | Strict removal timing | changed | Pinned Blizzard UI loads while required removed symbols remain temporarily available. |
| `Patch12_1.StrictRemoval.PostStartupHiding` | best-effort  | behavioral  | Strict removal timing | changed | Removed symbols are hidden from addon-facing checks after startup. |
| `Patch12_1.StrictRemoval.WrapperTiming` | best-effort  | behavioral  | Strict removal timing | changed | CVar wrappers install once at the modeled retirement boundary and remain identical through repeated cleanup/world entry; native per-wrapper timing is unverified. |

## Pending live evidence

Open behavior gaps:

- `Patch12_1.ForbiddenAspects.UntrustedScriptExecution`
- `Patch12_1.ForbiddenAspects.UntrustedLayoutScriptExecution`
- `Patch12_1.ForbiddenAspects.EventRegistrations`
- `Patch12_1.ForbiddenAspects.AlwaysPropagateInput`
- `Patch12_1.ForbiddenAspects.ScriptedInput`
- `Patch12_1.ForbiddenAspects.QueryFocus`

Native strict-removal and duration-binding captures can refine the bounded simulator policies; their absence does not turn best-effort tests into native conformance.

Probe installation, execution, or manual observations are not evidence captures and must not be used to resolve or close rows; retain the SavedVariables files first.

## Machine state totals

- implemented: 0
- best-effort: 36
- evidence-required: 18
- exception-requested: 0
- untriaged: 0

## Sources

- `data/patch-api/sources/12.1-behaviors.json` — normalized broader behavior boundaries and candidate disposition.
- [[patch-12-1-api-audit]] — broader audit context and family summaries.
- [ForbiddenAspectsProbe](../../addons/ForbiddenAspectsProbe/README.md) — six forbidden-aspect restriction evidence capture and limitations.
- [StrictRemovalTimingProbe](../../addons/StrictRemovalTimingProbe/README.md) — addon-visible strict-removal lifecycle timing capture and limitations.
- `12ed1355b` — added the strict-removal timing probe; probe code does not itself change audit status.

## See Also

- [[patch-12-1-framexml-symbol-inventory]] — separate 432-row FrameXML symbol occurrence register.
- [[patch-api-audit-manifest]] — manifest validation and exception-approval rules.
