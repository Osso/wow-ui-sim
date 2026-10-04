# B99 read-only verification — 4d142e325

Artifact/source review only. No builds/tests rerun. GREEN 381/381 and startup [] are caller-supplied, not independently reproduced.

## 1. Private aura sound Add — ACCEPT WITH QUALIFICATIONS

Recommended rows: March31-168 bounded Add-context slice, not native/full-row completion; March25-117 superseded in part, not independently completed as its older Remove restriction.

Governing retained source, data/patch-api/sources/12.0.5-api-changes.txt:
- :117: “Existing restrictions will remain on the AddPrivateAuraAppliedSound and RemovePrivateAuraAppliedSound APIs.”
- :168: “Removed the restrictions that we recently placed on the AddPrivateAuraAnchor, RemovePrivateAuraAnchor, SetPrivateWarningTextAnchor and RemovePrivateAuraAppliedSound APIs. The AddPrivateAuraAppliedSound API is still restricted, but this restriction is now only in place during encounters and during M+/PvP matches.”
Later March31 text governs: Remove is unrestricted; Add is context restricted. Secure-caller exemption is neither stated nor contradicted: consistent as a labeled inference, not established permission parity. Cached UnitAuraDocumentation.lua:11-25 declares modern AddAuraSound HasRestrictions and AllowedWhenUntainted; it does not prove the old 12.0.5 permission boundary.

Implementation is state-backed: private_aura_sounds/add.rs:18-98 authenticates original arguments, extra arguments, all raw table keys/values and all five declared fields before validation; :101-178 validates then checks encounter/M+/PvP; :181-208 allocates and stores owned payload. private_aura_sounds.rs:65-71 removes payload and live handle together. Inputs are owned per environment, not Lua placeholders.

Behavioral tests: private_aura_sound_add_context.rs:51-113 copies payload, mutates later Lua input, then removes one registration in restricted contexts; :116-167 denies each context, preserves allocator/taint, recovers, permits secure registration and isolates environments; :170-231 tests collision/exhaustion and invalid-input recovery; :252-306 tests authentic VM secret table/number, GC, addon denial and malformed-early-field authentication ordering. Modern-only :311-374 covers later secret argument before trigger validation and complete cached deprecated chunk. Existing removal suite is a control, not all newly RED evidence. Parent producer did not publish Add or store payload; these new registration tests have genuine observable pre-producer failure boundaries. Compiled RED is supplied by caller/commit, not reproduced here.

Earned bounded checkboxes, docs/specs/private-aura-sound-add-context.md:11-15,19-22,27. :26 has representative real-secret proof, but NOT a per-argument/per-field matrix. :28 legacy publication/removal are earned; modern part requires retail-12-1-0 execution (strict 12.0.5 excludes those two tests). No credit from 381/381 alone for a feature-gated-out test. Missing behavioral probes: secret unitToken/spellID/soundFileName/outputChannel, secret unknown table keys/values and extra arguments; modern secure secret trigger, trigger1, trigger range controls. These are coverage limits, not observed auth bypasses.

Runtime qualification: pvp_match_active is a host-only boolean (inputs.rs:19-20); git grep finds no runtime writer/event synchronizer. Tests set it explicitly (:43-47). No proof that firing PVP_MATCH_ACTIVE/COMPLETE changes Add permission. This is valid only as the specified host-state model, not automatic live PvP enforcement. No playback/trigger-dispatch credit.

## 2. Follower display — ACCEPT WITH QUALIFICATIONS

Recommended March31-174 status: PARTIAL. Source :174 promises actual nameplate CVar effects, not only a query.

nameplate_display.rs:11-36 authenticates the token and derives classification from resolved GUID membership plus modeled player identity. group_queries.rs:647-653 routes retail12.0.5; earlier epochs retain old implementation. Seven tests in follower_nameplate_display.rs:35-194 are behavioral: reversible follower membership, unchanged identity/class/friendliness, target/focus GUID replacement, environment isolation, hostile follower, taint/host-secret/GC, absent tokens and pet/vehicle controls. Parent producer ignores the follower set and accepts pet/vehicle unconditionally, so these assertions distinguish new behavior from pre-commit producer; no native consumer proof follows.

Earned docs/specs/follower-nameplate-display.md:7-11 as bounded query contracts. :34 stays unearned: unchanged loaded nameplate classification, FontString class-color output, name-only visibility, CVAR_UPDATE/reversible settings have no tests. :32-33 need caller's revision/feature-qualified execution evidence, not this source review.

Cached affected callers (paths relative to retail/AddOns):
- Blizzard_NamePlates/Blizzard_NamePlateUnitFrame.lua:248-269 — UpdateIsPlayer ORs UnitIsPlayer with the query; changed result drives UpdateShowOnlyName, UpdateNameClassColor, realm, auras and health bars.
- Blizzard_UnitFrame/Shared/CompactUnitFrame.lua:672-674 — class-color eligibility for compact unit health bars.
- Blizzard_Channels/ChannelRoster.lua:111-123 — passes a GUID and prefixes follower roster names.
Pet/vehicle now return false unless another modeled identity route supports them; the first two consumers therefore lose the former unconditional display-player eligibility when passed those tokens. No loaded-consumer regression control covers it. This is explicit model tightening, not evidence of native pet/vehicle parity.
Additional consumer gap: nameplate_display.rs:29-30 requires existing_guid_for_unit; unit_misc.rs:195-212 recognizes modeled tokens, not arbitrary GUID strings or live nameplateN. ChannelRoster's GUID call remains false even for a host-listed follower GUID. That gap pre-existed; this commit does not earn that caller's behavior. Query-only acceptance cannot close the nameplate prose row.

## 3. Navigation + aura rekey — REJECT (combined delivery)

### Navigation: ACCEPT WITH QUALIFICATIONS; March25-092 bounded/partial pending older-profile regression resolution

c_navigation.rs:18-46 rejects any active tainted call frame before reading host selection, then returns the selected string or an explicit missing-input error. Cached InGameNavigationDocumentation.lua:29-35 says HasRestrictions and nonnil cstring; source :92 says “C_Navigation.GetNearestPartyMemberToken can no longer be called by addons.” Tests patch_12_0_5_navigation_aura_entry.rs:99-163 assert changing host strings, missing selection, real stamped nested addon denial, caller restoration and unchanged state. Parent nil producer cannot satisfy selected strings or addon denial, so these are behavioral pre-producer discriminators.

Earned docs/specs/navigation-nearest-party-token.md:7-10 for the bounded host query. :30 real loaded-addon invocation remains unproved: stamping a fixture closure is not loading an addon. There is no runtime nearest_party_member_token writer, so cached Blizzard_QuestNavigation/SuperTrackedFrame.lua:231 can fail if party tracking becomes active without an external host setter; geometry is explicitly excluded and this is not native nearest-selection proof.

**Regression:** navigation_defaults.rs:15-33 no longer installs GetNearestPartyMemberToken on ANY profile; replacement publication is gated in c_api/mod.rs:241-242 by retail-12-0-5. Repository-wide revision grep finds no other definition. Historical retail12.0.0, Classic and Forever builds lose a formerly callable nil-returning API altogether. Cargo.toml:118-122,148-155 confirms those feature boundaries. tests/c_navigation_probes.rs:19-24 and navigation_defaults.rs:58-63 delete the call instead of preserving a non-retail control. docs/specs/navigation-nearest-party-token.md:21 claims other temporary entries unchanged, but does not disclose this profile-wide removal. This is an observed API-surface regression; whether particular Classic vendor code calls it is not established.

### Aura entry: REJECT; March31-150 BLOCKED/PARTIAL, not accepted runtime capability

**High-confidence merge defect:** aura_entry.rs:34-37 returns “aura entry requires host-supplied replacement IDs” for nonempty stores with pending=None. env_events.rs:232-240, globals/state_backed_queries.rs:63-75 and loader_env.rs:151-158 propagate it before callbacks/listeners. IDs remain OLD, event is NOT delivered, and caller gets an error (not a successful silent skip). Lua A_Admin.FireEvent and FireEvent error via admin_events.rs:16-26 and state_backed_queries.rs:98-105; Rust fire_event returns Err. Error logging/display depends on caller handling, but event loss does not.

No production code stages aura_entry_ids.pending: revision grep finds only initialization at state.rs:297, public field declaration at state/sim_state.rs:338 and rekey reads/consumption. Tests alone set it. Normal WowLuaEnv::new uses SimState::default (env.rs:77-79); default state seeds buffs (state.rs:830), and game_data.rs:605-622 creates 4–6 unless WOW_SIM_NO_BUFFS is set. Thus ordinary runtime/admin encounter/M+/PvP entry is broken without a separate unavailable batch setter. Empty stores with no batch do no-op (aura_entry.rs:59-60); this does not protect normal populated stores. No evidence that startup fires these events; startup [] does not exercise the defect.

Tests are substantive and disclose the defect: patch_12_0_5_navigation_aura_entry.rs:166-187 checks rekey-before-observer across all entry types/repeated entries, old instance misses and query consistency; :191-219 expressly expects error on missing input (:215-216) and rejection without delivery on bad plans; :223-236 exercises host/admin/global/loader paths only AFTER staging; :240-278 covers party helpful/harmful stores and payload, :282-296 post-entry insertion. These are behavioral and fail with producer withheld, but GREEN proves the artificial host protocol, not usable runtime transitions.

Mechanically earned bounded spec tests: docs/specs/aura-entry-instance-ids.md:7-11 (explicit staged-host protocol, atomic replacement, retired-ID rejection, query/observer ordering, non-entry stability). They must NOT be promoted to row150 completion: source :150 says IDs “re-randomize when the player enters” and does not authorize suppressing entry events. :8-9 cover only modeled player/party stores; target constant fixtures and stale blocked/provider bindings are excluded (:37). :32 native randomness/universal storage and :33 loaded cooldown-viewer/UNIT_AURA ordering remain unearned. Empty-store no-batch behavior is code-inspected, not directly tested in this new suite.

## 4. ignoreGCD + UnitIsUnit — ACCEPT WITH QUALIFICATIONS

Recommended March25-114 and March25-104: BOTH PARTIAL.

ignoreGCD is a real existing state-backed cooldown selection model, now with authenticated action/book flags. cooldown_duration.rs:14-20 unwraps the flag; c_action_bar.rs:80-83 and c_spell_book.rs:470-481 route through it, book authentication before missing-entry return. Cached ActionBarFrameDocumentation.lua:172-181 and SpellBookDocumentation.lua:234-244 declare AllowedWhenUntainted; SpellDocumentation.lua:286-295 instead says AllowedWhenTainted. Applying the new helper to spell would not establish that different contract.

All five new tests are behavioral. retail_12_0_5_partial_104_114.rs:49-94 drives real Cooldown widgets from three queried objects, then removes the individual cooldown and checks new output plus retained snapshots. :98-123 covers secure secret true/false flags; :126-151 tainted secret rejection/recovery/taint; :154-174 invalid book entries still authenticate the secret flag. Last three distinguish parent read_ignore_gcd's raw Bool(true)-only producer and book's previous early nil return, matching supplied RED 3 failures. Widget-consumption test already passes parent: it is stronger control coverage, not newly implemented interval selection.

Earned docs/specs/spellbook-cooldown-duration.md:40-42, narrowly as stated. Unproven/open in :44-46: native type/coercion and interval-selection semantics, older-epoch controls, action slot/book slot+bank secret authentication, spell's AllowedWhenTainted handling, restricted duration-output secrecy, pet-book behavior and loaded Blizzard consumers. Row114's introductory “APIs that construct and return duration objects” is broader than this tested action/spell/book cooldown subset. No new regression established in the two changed producers, but their existing selector/security limits remain.

UnitIsUnit test :22-45 verifies same-name/different-GUID false symmetry and public FocusUnit recovery to same-GUID true. No UnitIsUnit producer changed in this commit; it already passes parent and is a regression control, not RED feature evidence. Earned docs/specs/unit-identity-equality.md:71 only. Existing docs :50 and :63 still call same-name fixtures unproven: obsolete for this precise target/focus fixture once caller's GREEN is accepted, not for all identities.

Unproven umbrella clauses: all source-permitted token identities (pet/vehicle/mouseover/soft*/npc/questnpc and raid/pets lack full modeled GUIDs), native token grammar/canonical bounds, missing/nil policy and native denial arity. Source :105-107 specifies allowed token classes and “All other comparisons are disallowed and return nil”; zero-return implementation is an explicitly inferred contract (unit-identity-equality.md:14,23), not independently established native behavior. Older-profile proof and live SELF-menu consumer remain open. Existing bounded permission matrices do not close introductory row104.

## Verification ledger and merge risk

- [EXIST] PASS: all 27 assigned code/test/spec paths retrieved at exact commit via git show. New implementations: Add 209 lines + inputs34, display37, navigation47, aura79 + IDs12, cooldown helper37. New behavioral suites: Add374, display194, navigation/aura297, follow-up174; removal370 is largely prior control coverage.
- [SUBSTANTIVE] PASS: owned registration payloads, membership-backed classification, selected host string, validated ID transactions and authenticated duration flag; no placeholder-only acceptance.
- [WIRED] PARTIAL/FAIL: Add register globals/register.rs:102; display group_queries.rs:73,648-649; navigation c_api/mod.rs:241-242; aura three event entry hooks; cooldown action/book callers. Tests auto-discovered by build.rs:463-470 and tests/integration.rs:1. Aura runtime replacement-input wiring is absent; non-retail navigation publication disappears.
- [ANTI-PATTERN] PASS for assigned added lines: TODO=0, FIXME=0, HACK=0, XXX=0, except: pass=0; no new empty catch or commented-out implementation observed in inspected changes. This scan is not correctness proof.
- OVERALL: FAIL for merging this delivery unchanged. Populated-aura entry events can now fail before delivery; older profiles lose a callable navigation entrypoint. Retain bounded sound/display/cooldown/control-test credit, but do not promote query-only or host-staged fixtures into full prose compliance. No cargo/test binary/agents/model CLIs, git mutation or repository edits were performed. Only this authorized report was written.
