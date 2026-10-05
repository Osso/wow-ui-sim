# Patch 12.0.5 API Audit

### Round 101 — aura header ordering and talent-helper delegation

Commits `a9f6b0a26` and `d62a972eb`; [contract and acceptance](../../specs/secure-aura-header-helper-delegation.md). All four prose rows (03-25-112, 03-31-151, 03-25-120, 03-31-177) are **partial**: aura ordering is proven through the real cached header code, but retail does not load that file; the helper commands run the real cached callbacks without taint, but the delayed loadout commit is unmodeled. Behavior change: `C_ClassTalents.SwitchTo*` no longer mutates state when no helper callbacks are registered. Cast completion now lives in one module shared by GUI and headless ticking. **133 capabilities/362 IDs; 24 pending /270 bounded /33 partial /35 metadata**; audit **IN PROGRESS**.

### Round 100 — review follow-ups

Commits `942833a04` and `8828f7394`, integrated and proven in a worktree (RED 0/10 on the new behavior, GREEN 347 selected tests there, 140/140 on master, startup `[]`), reviewed and accepted with qualifications ([report](../../../data/patch-api/evidence/12.0.5-session-2026-10-03/round-100-review.md)). `GetRaidRosterInfo` names follow the shared identity predicate; model `SetUnit` is now tested on real PlayerModel-family frames, so exact534 becomes bounded; `SecureCmdOptionParse` returns the selected unit and the unchanged cached `/tm` handler honors it; viewed outfit is host state with a real `ChangeViewedOutfit`. Other rows keep their status with updated notes. Four tests that fail on master were shown, by building the session-start commit, to have failed before this session ([report](../../../data/patch-api/evidence/12.0.5-session-2026-10-03/baseline-failures.md)). **131 capabilities/362 IDs; 28 pending /270 bounded /29 partial /35 metadata**; audit **IN PROGRESS**.

### Batch99 and Classic — twelve capabilities, one rejected slice fixed

Commits `4d142e325` (eight slices), `ce418cbe4` (Classic secret policy, built and run under `client-mists` in a worktree) and `fcdcf43c6` (review fixes). B99 RED 6 PASS / 63 FAIL, GREEN 381/381 with control suites, startup `[]`.

- Bounded: [private aura sound Add](../../specs/private-aura-sound-add-context.md) (03-31-168), [navigation token](../../specs/navigation-nearest-party-token.md) (03-25-092), [housing bundles](../../specs/housing-bundle-structures.md) (641), [entrance PDEID](../../specs/tiered-entrance-pdeid.md) (exact260), [core fixture selection](../../specs/house-exterior-core-fixture.md) (exact270), [restricted outfit helper](../../specs/restricted-outfit-index.md) (03-12-032, tests only), and [table.freeze](../../specs/table-freeze.md) (03-25-094) after a strict 12.0.5 build showed RED with the old gate and GREEN with the new one.
- Partial: 03-25-117 (superseded for Remove), [follower display](../../specs/follower-nameplate-display.md) (03-31-174, query only), [aura entry rekey](../../specs/aura-entry-instance-ids.md) (03-31-150, host-staged only), [special-bar membership](../../specs/action-bar-special-membership.md) (exact245), 03-25-114 and 03-25-104, [Classic secret policy](../../specs/classic-secret-policy.md) (04-17-213/214; 215 stays pending).
- Review **rejected** the first aura-rekey version: with auras present and no staged IDs, encounter/M+/PvP entry events raised before listeners ran. Fixed: no staged batch, no rekey, event delivered. The same review found older profiles had lost `GetNearestPartyMemberToken` and `SelectCoreFixtureOption`; both inert defaults restored for them.
- Parked: chat expressions (253, needs an unpublished rilua helper), combat restrictions (03-25-074, no denial contract in the source), aura header and talent helper (03-25-112/120, 03-31-151/177).
- Behavior changes: entrance PDEID default 0 instead of 77011; pet/vehicle no longer always player-for-display.

**131 capabilities/362 IDs; 28 pending /269 bounded /30 partial /35 metadata**; audit **IN PROGRESS**.

### Batch98 — instanced identity, model guard, cast events, /tm, /outfit, delve instance, table.freeze, quest favor

Commit `a4cce2db1`; RED 49 PASS / 31 FAIL, GREEN 404/404 with control suites, startup `[]`; three independent source reviews, all accept with qualifications.

- Bounded: [/tm](../../specs/target-marker-macro-command.md) (03-25-098), [delve instance state](../../specs/delve-instance-state.md) (03-31-181), [outfit action and /outfit](../../specs/outfit-action-command.md) (03-25-111/122, 03-31-165/179), [quest favor](../../specs/quest-reward-favor.md) (exact293, a miss now returns zero), model guard row 542 and prose 04-10-194/195, and [cooldown countdown formatter](../../specs/cooldown-countdown-formatter.md) (03-25-115, 03-31-161) from existing tests with no code change.
- Partial: [instanced identity](../../specs/instanced-identity.md) (04-10-197/198/199; raid/pet/vehicle unmodeled), [model guard](../../specs/model-unit-identity-guard.md) row 534 (PlayerModel untested), [secret instant-cast events](../../specs/secret-instant-cast-events.md) (04-10-193; helper only, no runtime non-player cast), [table.freeze](../../specs/table-freeze.md) (03-25-094; strict 12.0.5 build not run).
- Behavior changes: on retail 12.0.5 party names are public by default (secrecy now needs explicit host state); selecting an outfit no longer sets the viewed outfit; model `SetUnit` no longer runs tooltip code on any profile.
- Blocked rows now carry specific notes instead of the generic one: [ledger](../../../data/patch-api/evidence/12.0.5-session-2026-10-03/blocked-rows-ledger.md); prose 03-12-055 metadata-only.

**119 capabilities/362 IDs; 42 pending /262 bounded /23 partial /35 metadata**; audit **IN PROGRESS**.

### Held rows reconciled against existing capabilities

Ten rows an earlier session held were reassessed against capabilities that already had independent bounded proof: exact342/347 (tooltip unit buff/debuff arg1 `NeverSecret` removal), exact278/279 (pending decor variant selector), exact281/282 (destroy-entry variant selector), exact291 (mount `SpellIdentifier`) become bounded; structures 650/651 (housing aggregates) partial, because an absent input omits a field the declaration requires; prose 03-25-088 metadata-only, superseded by the 03-31 comparison rules at source lines 141–144. One auditor reread sources, declarations and producers and reran 118 existing tests on a prebuilt binary; no separate verifier and no new tests. [Report](../../../data/patch-api/evidence/12.0.5-session-2026-10-03/handoff-held-live.md). Each row keeps its prior hold note. **110 capabilities/362 IDs; 61 pending /250 bounded /17 partial /34 metadata**; audit **IN PROGRESS**.

### Batch97 — equipset command, tooltip line secrecy, identity GUID suppression

Commit `92e4ea045`. [/equipset](../../specs/equipment-set-command.md) (prose 03-25-121, 03-31-178) and [tooltip secrecy](../../specs/tooltip-texture-secrecy.md) (prose 03-31-136, 03-25-075) are bounded; [identity-secret reverse GUID lookup](../../specs/unit-token-identity-secrecy.md) (prose 03-12-048, 03-25-084) is **partial**: party tokens only. RED 6 PASS / 20 FAIL, GREEN 62/62, startup `[]`; two independent source reviews accepted with qualifications. The same commit restored both catalog shop getters on profiles without `retail-12-0-5`. Quest favor (exact293) was **rejected**: a miss raises while cached reward tooltips call it unconditionally; fix pending. HousingBundleInfo (641) parked: the Lua market action mutates the seeded bundle the new getter no longer reads. Review reports and run logs now live in [evidence](../../../data/patch-api/evidence/12.0.5-session-2026-10-03/README.md). **110 capabilities/362 IDs; 71 pending /243 bounded /15 partial /33 metadata**; audit **IN PROGRESS**.

### Batch95/96 — eight explicit-state slices accepted

One commit, `a0e23199d`, replaced placeholders with explicit host state for twenty source IDs. Each slice owns its contract and acceptance:

- [Maximum cumulative aura applications](../../specs/spell-max-cumulative-aura-applications.md) — exact315/316, newly registered.
- [Maw powers](../../specs/spell-maw-powers.md) — exact299 bounded, exact297 **partial** (no cached declaration, `Deprecated_12_0_7.lua` reassigns the border query on a full load).
- [Delves inputs](../../specs/delves-api-inputs.md) — exact262/263/265/257/258. A curio-link miss now errors and the map is empty by default.
- [Voice-chat speech requests](../../specs/voice-chat-speak-text.md) — exact409, newly registered.
- [Click-binding spell identifier](../../specs/click-binding-spell-identifier.md) — exact255. Empty eligibility by default: retail click-binding UI refuses every spell until seeded.
- [Encounter-event sound](../../specs/encounter-event-sound-no-error.md) — prose 03-12-043, tests on the existing model, no producer change.
- [Raid roster unknown name](../../specs/raid-roster-unknown-name.md) — prose 03-25-123 and 03-31-180.
- [Catalog shop product structures](../../specs/catalog-shop-product-structures.md) — structures 631/632/634/635/636/637; seeded Lua getters removed.

RED with producers withheld 6 PASS / 63 FAIL; GREEN 396 PASS / 1 FAIL (`c_system_api::test_c_console_get_all_commands_empty`, untouched console command count); startup `[]`. Four independent reviews accepted every slice with qualifications and reran 59/59. Shared limit: secret spell identifiers and selectors are rejected for every caller, so declared `AllowedWhenTainted`/`AllowedWhenUntainted` argument permissions are unmodeled. Profiles without `retail-12-0-5` lost both catalog getters; fix follows. **107 capabilities/362 IDs; 77 pending /239 bounded /13 partial /33 metadata**; audit **IN PROGRESS**.

### Batch94 — GetSpellBookItemCastCount registered and accepted

**Exact318 `C_SpellBook.GetSpellBookItemCastCount`** (`SecretWhenSpellCooldownRestricted -> SecretWhenCooldownsRestricted`) was not registered. [Contract and acceptance SSOT](../../specs/spell-book-cast-count.md#development-proof-and-independent-bounded-acceptance--2026-10-03). It resolves the player book slot to its spell, returns the explicit cast count shared with `C_Spell.GetSpellCastCount`, zero on a miss, secret only under the cooldown restriction, with both selectors authenticated first. RED `96994beaf` 0/5 after a non-compiling first fixture; producer `a881a1729` GREEN 35/35, startup `[]`; independent review accepted with qualifications and reran 35/35. Non-spell entries and passive castability have no model. **99 capabilities/362 IDs; 97 pending /220 bounded /12 partial /33 metadata**; audit **IN PROGRESS**.

### Batch93 — GetAuraDuration registered and accepted

**Exact380 `C_UnitAuras.GetAuraDuration`** (`AllowedWhenTainted -> AllowedWhenUntainted`) was not registered at all: the name resolved to a nil-returning placeholder. [Contract and acceptance SSOT](../../specs/aura-duration-object.md#development-proof-and-independent-bounded-acceptance--2026-10-03). It now returns a duration object built from the stored aura times, errors for an invalid instance, and authenticates both arguments before validation through the Batch86 reader. Inputs `b9b9eeec8` RED 0/6; producer `012cf889a` GREEN 33/33, startup `[]`; independent review accepted with qualifications and reran 33/33. Mixed zero duration/expiration records are not rejected and aura access is unmodeled. **98 capabilities/362 IDs; 98 pending /219 bounded /12 partial /33 metadata**; audit **IN PROGRESS**.

### Batch92 — `UnitAttackSpeed` swing-time inputs

**Exact504** was the last constant stat producer. [Contract SSOT](../../specs/explicit-stat-inputs.md#b92-unitattackspeed-swing-time-inputs--row-504): player main and optional off-hand speeds are explicit inputs, a missing off-hand is a plain nil, party snapshots get a synthetic main-hand time, unknown units return `0, nil`. Inputs `cb1fc14ab` RED 0/4, producer `ca7e6778e` GREEN 73/73, startup `[]`; independent review accepted with qualifications. All 50 `SecretWhenUnitStatsRestricted` rows are now bounded.

### Batch91 — FontString SetFont shape and removed housing ID fields

Tests only (`cbfe0ad8e`), already green, no producer change; independent GPT-6.1-sol review (report SHA256 `2cabc6228fb5a7e2e38a927aafd79e666e86177648414afe1804f75c194cd3fb`, scratchpad-only) reran 45/45 and 15/15.

| Row | Status | What is asserted |
|---|---|---|
| 547 `SetFont` arg2 `uiUnit -> number` | bounded | Heights 13.2, 0.5, 40 round-trip as plain numbers. |
| 548 `SetFont` arg3 nilable | bounded | Omitted and nil flags accepted, outline kept; empty string clears. |
| 549 `SetFont` `+ ret1 = success` | **partial** | Exactly one boolean. It means "path was a string": unresolvable paths and non-numeric heights still return true. |
| 645/646 `HousingCatalogEntryID` field removals | bounded | No removed field on populated results or variant IDs; obsolete extras ignored as input; no `src` occurrence. |

Rows 544/546 (`FontAsset`) stay pending: the cached documentation references the type without defining it. Rows 534/542 are 3D model methods, an intentional gap. **97 capabilities/362 IDs; 99 pending /218 bounded /12 partial /33 metadata**; audit **IN PROGRESS**.

### Batch90 — four already-implemented rows accepted

The scouts found rows whose behavior and tests already existed but were never credited. An independent GPT-6.1-sol verification (report SHA256 `34555a5fc753f0f79859ea084d6619800c6b4ade4482df795a24a03844a8ed98`, scratchpad-only) quoted the asserting lines, checked they cannot pass vacuously and reran the filters; main ran the same filters at `62d0ce70f` source. No code changed.

| Row | Verdict | Proof |
|---|---|---|
| exact371 `GetAuraBaseDuration` arg3 `SpellIdentifier` | accept with qualifications | [aura refresh duration](../../specs/aura-refresh-duration.md); numeric and seeded-string identifiers select base 40 versus aura-derived 20; `aura_refresh_duration::` 18/18. Joins `aura-refresh-duration`. |
| `prose-2026-03-12-023` zero-span at maximum charges | accept | [spell charge state](../../specs/spell-charge-state.md); all three APIs return nonnil zero-span objects; `cooldown_probes::charge_duration` 10/10. |
| `prose-2026-03-12-026` zero-span fully elapsed | accept with qualifications | [duration core](../../specs/duration-core.md); expired, elapsed fraction 1, remaining 0; `duration_core::` 27/27. `HasStarted` stays false. |
| `prose-2026-03-12-029` five public aura flags | accept with qualifications | [aura classification flags](../../specs/aura-classification-flags.md); all five asserted non-secret from an addon-tainted caller; `aura_table_shape::` 7/7. |

Limits: configured simulator state only; no native security policy, restricted transition, every-identifier-form or other-profile credit. The historical batch5 failure note on row023 and the bounded-partial hold on row029 are superseded for these scopes only. **94 capabilities/362 IDs; 105 pending /213 bounded /11 partial /33 metadata**; audit **IN PROGRESS**.

### Batch88 — explicit inputs replace ten constant stat producers

Ten of the eleven constant or proxy stat rows left by Batch87 now have real inputs: **422, 434, 436, 440, 452, 464, 470, 472, 478, 488**. [Contract and acceptance SSOT](../../specs/explicit-stat-inputs.md#development-proof-and-independent-bounded-acceptance--2026-10-03) owns the input table, proof and limits. Inputs `be7c0cc2c` RED 33 PASS/11 FAIL; producers `62d0ce70f` GREEN 79/79, startup `[]`; independent GPT-6.1-sol review accepted with qualifications and reran 44/44.

Behavior change worth knowing: `GetShieldBlock` returned armor and read a unit argument; it now returns an explicit shield-block input, zero by default, and ignores arguments. **504 `UnitAttackSpeed`** is the one stat row still constant. **92 capabilities/362 IDs; 109 pending /209 bounded /11 partial /33 metadata**; audit **IN PROGRESS**.

### Batch89 — prose classification: 26 metadata-only, 6 restatements credited

Three independent GPT-6.1-sol scouts classified all 90 pending blue-post prose rows (reports scratchpad-only, SHA256 prefixes `7edf1210a531ed93`, `ccdbd95ed858eceb`, `c95f657beddc03cc`). Main checked every promoted row's register text before accounting.

- **26 rows → `metadata-only`**: post date headings, titles, greetings, author disclaimers, section headings, jokes, and March 12 sentences about the aura rework that the post itself defers beyond 12.0.5 and 12.0.7. Zero runtime or implementation credit, following the [six-occurrence precedent](#six-source-occurrences--metadata-only-accounting-accepted). IDs: 03-12 `054 057 059 060 061 062 063`; 03-25 `066 067 069 070 125`; 03-31 `128 129 131 132 138`; 04-10 `185 186 188 189`; 04-17 `205 206 208 209 210`.
- **5 rows → `bounded-coverage` under existing capabilities, no new behavior**: 03-25 `105/106/107` are byte-identical to bounded 03-31 `142/143/144` (`unit-comparison-permissions`); 03-25 `116` announces the removals already bounded as exact359/401/405; 03-31 `171` states the questID payload already bounded as the event row (`quest-accept-confirmation`).
- **1 row → `partial-development-green`**: 03-25 `104` is byte-identical to partial 03-31 `141`.

Not promoted: `prose-2026-03-12-055` (history of the private aura system; needs an explicit out-of-simulator accounting decision), partially covered `114` and `168`, and about 55 sentences describing behavior with no accepted coverage. Three of those already have implementations and development tests and need only bounded acceptance: 03-12 `023` (zero-span charge durations), `026` (zero-span objects fully elapsed), `029` (five public aura flags). [Coverage SSOT](../../../data/patch-api/sources/12.0.5-page-coverage.json): **91 capabilities/362 IDs; 119 pending /199 bounded /11 partial /33 metadata**; audit **IN PROGRESS**.

### Batch86 — aura expiration query argument policy accepted

**Exact365 `C_UnitAuras.DoesAuraHaveExpirationTime`** (`AllowedWhenTainted -> AllowedWhenUntainted`) is the first row in this continuation needing a producer change. [Acceptance SSOT](../../specs/aura-expiration-time.md#independent-bounded-acceptance--2026-10-03) owns the contract, proof table and limits. The function, previously an uncommitted change, was committed as `608d52558`; inputs `a342c6eec` gave RED 6 PASS/3 FAIL.

The first producer `0d2a98596` passed 27/27 and was **rejected by independent review**: it validated the unit before authenticating the instance ID, so a tainted caller's malformed unit pre-empted the secret denial. No payload was exposed. A failing ordering assertion (`08c9e8bc8`, 8 PASS/1 FAIL) and the corrected producer `d1bbdc8e8` (27/27) followed; independent re-verification accepted it with qualifications and reran 9/9 and 18/18 itself. The rejection is retained history, not erased.

[Coverage SSOT](../../../data/patch-api/sources/12.0.5-page-coverage.json): **91 capabilities/362 IDs; 150 pending /194 bounded /11 partial /7 metadata**. Aura access, restricted output, blocked/target records and startup after the reorder are unverified; audit **IN PROGRESS**.

### Batch87 — eight existing stat models accepted

An independent GPT-6.1-sol scout classified the 19 remaining `SecretWhenUnitStatsRestricted` rows. Main accepts its eight **state-backed and wrapped** rows as output annotations only: **418, 446, 454, 456, 462, 490, 492, 516**. [Acceptance SSOT](../../specs/retail-missing-stat-inputs.md#b87-existing-model-output-annotations-accepted--2026-10-03) owns the per-row state, assertions and limits. Fresh `character_stats::` run: **35 PASS/0 FAIL** on the binary built at `d1bbdc8e8`.

The other eleven are constants or proxies and earn nothing by audit: **422, 434, 436, 440, 452, 464, 470, 472, 478, 488, 504**. They need explicit inputs (seven player scalars, two expertise triples, an optional pet spell bonus, a per-GUID attack-speed pair) before any credit. Batch86 (row365) is implemented and awaiting re-verification. [Coverage SSOT](../../../data/patch-api/sources/12.0.5-page-coverage.json): **90 capabilities/362 IDs; 151 pending /193 bounded /11 partial /7 metadata**; audit **IN PROGRESS**.

### Batch85 — attack power and spell haste output annotations accepted

Main accepts an independent GPT-6.1-sol source audit (**ACCEPT WITH QUALIFICATIONS** per row) for **exact502 `UnitAttackPower`, 508 `UnitRangedAttackPower` and 512 `UnitSpellHaste` output annotations only**. [Acceptance SSOT](../../specs/unit-stat-output-restriction.md#b85-independent-bounded-acceptance-and-refreshed-restriction-proof--2026-10-03) owns evidence and limits. **478 `GetShieldBlock`** (returns armor) and **504 `UnitAttackSpeed`** (constant 2.0, 2.0) are deferred without credit: wrapping a proxy or constant is not modeled behavior.

**Fresh execution replaces record reuse:** at clean `0d2a98596` the four stat-restriction tests ran 4 PASS/0 FAIL, exit0, on the integration binary built by the B86 GREEN compile. That run also covers the B83 and B84 fixtures. `rustfmt` is now installed for toolchain 1.98.1 and `cargo fmt --check` exits0 on the whole tree, retiring the historical global-format failure in `aura_duration.rs` (committed as `608d52558`). [Coverage SSOT](../../../data/patch-api/sources/12.0.5-page-coverage.json): **89 capabilities/362 IDs; 159 pending /185 bounded /11 partial /7 metadata**. B82 agent719 metadata audit still uninspected; audit **IN PROGRESS**.

### Batch84 — ranged crit and haste output annotations accepted

After `64a19cb63`, main accepts an independent GPT-6.1-sol source audit (**ACCEPT WITH QUALIFICATIONS** per row) for **exact474 `GetRangedCritChance` and exact476 `GetRangedHaste` output annotations only**. [Acceptance SSOT](../../specs/unit-stat-output-restriction.md#b84-independent-bounded-acceptance--2026-10-03) owns evidence and limits. Existing providers reuse the melee getters: computed 360/180 +5 =7 and 510/170 =3, one wrapped result each; no ranged-specific model. `client-wowforever` `GetRangedHaste` returns two plain values and is excluded; that profile cannot combine with a retail epoch (source-derived, not compiled).

**No new execution**, same reused committed B82 record and unreadable raw artifacts as [Batch83](#batch83--spell-crit-output-annotation-accepted). [Coverage SSOT](../../../data/patch-api/sources/12.0.5-page-coverage.json): **88 capabilities/362 IDs; 162 pending /182 bounded /11 partial /7 metadata**. B82 agent719 metadata audit still uninspected; audit **IN PROGRESS**.

### Batch83 — spell crit output annotation accepted

After `e8a6b7527`, main accepts an independent GPT-6.1-sol source audit (**ACCEPT WITH QUALIFICATIONS**, eight items PASS) for **exact486 `GetSpellCritChance` output annotation only**. [Acceptance SSOT](../../specs/unit-stat-output-restriction.md#b83-independent-bounded-acceptance--2026-10-03) owns evidence and limits. Existing provider and fixtures unchanged: computed crit rating360/180 +5 =7, one wrapped result, seeded `(2)` fixture exercised by two of the four restriction tests. Cached declaration has no arguments or `SecretArguments`; B82's school guard is not copied.

**No new execution.** Proof is the committed B82 aggregate record of four restriction controls at `d21d4a208` with an empty `src`/`tests`/Cargo diff since. Raw `/tmp/patch-12.0.5-*` artifacts were unreadable on this host; B82's check-protocol qualification carries over. [Coverage SSOT](../../../data/patch-api/sources/12.0.5-page-coverage.json): **87 capabilities/362 IDs; 164 pending /180 bounded /11 partial /7 metadata**.

**Still open from B82:** agent719 metadata audit of the 86-capability accounting was not inspected — its `/tmp/patch-12.0.5-spell-bonus-stat-accounting-proof.{md,json}` reports are absent on this host; no acceptance is inferred. Spell-specific/native formulas, restricted no-argument and extra-argument behavior, activation, older profiles and full suites remain unverified; audit **IN PROGRESS**.

### Batch82 — independent bounded acceptance

After `7f53f4648`, main accepts716 **12 distinct refreshed PASS** (7 new school/output cases +4 stat controls +1 proxy control). [Acceptance SSOT](../../specs/spell-bonus-stat-security.md#independent-bounded-acceptance--2026-10-03) owns full proof, case inventory and provenance. Current local startup `[]` and loaded probe STDERR `public-accepted secure-accepted tainted-denied`, with no exec-Lua error, establish bounded simulator behavior. Scoped format0, source and readability accepted.

**Check PROTOCOL QUALIFIED:** success JSON and zero diagnostics were retained, but `stderr_to_file` misuse made execution **synchronous; direct exit not retained**. No check exit0, asynchronous execution or all-gates-pass claim; no rerun. Actual test compile0 independently supplies compilation proof.

Exact482/484 are now **bounded OUTPUT-only** annotations. Original-school authentication follows separate cached argument policy; no argument-removal credit. Existing spell-power intellect proxy/healing/output helper unchanged. [Coverage SSOT](../../../data/patch-api/sources/12.0.5-page-coverage.json): **86 capabilities/362 IDs;165 pending /179 bounded /11 partial /7 metadata**. New independent metadata audit **pending**, not inferred from behavior acceptance.

Automatic activation, per-school/native formulas, ordinary type/nil/range policy, older profiles, full suites and historical unowned globalfmt1 remain unverified. [Initial loaded-probe failure](../../specs/spell-bonus-stat-security.md#qualified-prior-full-runtime-failure) retains tainted acceptance/assertion failure despite CLI0 and corrected stdout-only false marker; `17211be121` RED4 PASS/3 FAIL does not establish downstream GC completion. Historical input/producer then-pending checkpoints below remain historical. B81 main707/durable71364/64 accepted and missing-artifact71148/48 unaccepted unchanged; audit **IN PROGRESS**.

### Batch82 — producer checkpoint, proof pending

After producer `d21d4a208`, [spell bonus stat security SSOT](../../specs/spell-bonus-stat-security.md) owns requirements, case inventory and full proof. Supplied grouped RED at `17211be121`: compile exit0/zero diagnostics **90.716985s**; run exit101/**1.115939s**, **4 PASS /3 FAIL**. Tainted NUM, malformed secret payload and GC schedules fail the required denial-before-output boundary; downstream root/GC completion is **not proved**. This supersedes only the compiling state of historical wiki checkpoint `71c384eec` below, not its then-pending truth.

Supplied producer changes **one callback/eight lines**: authenticate original school through existing VM `unwrap_secret` before spell-power read/output, gated `retail-12-0-5`, with API/argument error context. Public shape, ignored-school intellect proxy, healing and existing wrapping model remain unchanged. No new type/range/formula/nil policy or native claim. Main GREEN worker3892114 is **compiling/pending** seven new +four stat +one proxy cases, startup and loaded-runtime probe; independent verifier **pending**. No polling, inferred GREEN, startup, runtime-probe or acceptance credit.

[Prior real-runtime failure](../../specs/spell-bonus-stat-security.md#qualified-prior-full-runtime-failure) remains STDERR marker plus assertion failure; exit0 is **not PASS**. Exact482/484 `SecretWhenUnitStatsRestricted` deltas remain **OUTPUT only**, no argument-removal credit. B81 accepted707/durable corrective71364/64 and missing-artifact71148/48 unaccepted remain unchanged. **85 capabilities,362 ordered IDs;167 pending /177 bounded /11 partial /7 metadata** unchanged; audit **IN PROGRESS**.

### Batch82 — spell bonus inputs-only checkpoint

Committed `17211be121` contains **seven authored cases/test+spec inputs**. [Spell bonus stat security SSOT](../../specs/spell-bonus-stat-security.md) owns requirements, case inventory and qualified prior probe details; all new requirements remain unchecked, authored only. At the last supplied stage, main asynchronous compiled-RED worker3886715 at `/tmp/patch-12.0.5-spell-bonus-stat-red-ops/` was **compiling**: no completed compiled RED, GREEN or B82 producer yet. This historical checkpoint does not infer RED from expectations or later worker state; no logs polled.

The [qualified prior pinned full-runtime probe](../../specs/spell-bonus-stat-security.md#qualified-prior-full-runtime-failure) at `d7d7b41fc` recorded STDERR public/secure/tainted-secret acceptance followed by an `AllowedWhenUntainted` assertion error. CLI exit0 is **not API PASS**. Original stdout-only marker query falsely reported nonexecution; corrected both-stream query required no rerun. This is simulator defect evidence, not grouped RED or native proof.

Existing intellect proxy and explicit restriction-output model remain unchanged: no new wrapping or formulas. At that historical checkpoint, original-school authentication was planned; school type/nil/range/native policy, automatic restriction activation and older-profile execution remain unverified. Exact482/484 retained deltas concern **output secret flags only**: no argument removal, extra output credit, capability or status change. B81 main-accepted707 and durable corrective713 PASS64/64 remain accepted; original711 missing-artifact48/48 claim remains unaccepted, as recorded below. **85 capabilities,362 ordered IDs;167 pending /177 bounded /11 partial /7 metadata** remain unchanged. Audit **IN PROGRESS**.

### Batch81 — independent bounded acceptance

After `33e444bad`, [acceptance SSOT](../../specs/tooltip-unit-debuff-security.md#independent-bounded-acceptance--2026-10-03) owns main-accepted707: **71 distinct, all refreshed PASS** (15 Debuff +18 Buff +24 instance +14 filter), current local startup `[]` with `casc=false`, current scoped formatting/default check. Producer `d7d7b41fc` shares `c_tooltip_info_indexed_aura`; Buff preservation is accepted. Older-profile body/wiring preservation is source-proven, **not older-profile execution**. Full proof remains in SSOT, not duplicated here.

[Current page coverage](../../../data/patch-api/sources/12.0.5-page-coverage.json) records **85 capabilities,362 ordered IDs;167 pending /177 bounded /11 partial /7 metadata**, with order/statuses unchanged. Exact347 remains **AUDIT-PENDING**; access/restricted outputs **UNMODELED**. Main accepts corrective713 **PASS64/64** with durable host-read-back `/tmp/patch-12.0.5-debuff-indexed-accounting-corrective-proof.{md,json}`: prior84 capabilities/other361 rows and register/policy unchanged,12 intentional requirement closures/table15 match saved results. Original711 terminal48/48 remains **UNACCEPTED** because both claimed reports were absent; corrective fresh metadata-only proof adds no semantic execution credit. General-unit/native/GUI/CASC/all-profile/full-suite gaps and unowned globalfmt1 remain open. Main rejects helper-length findings classified as test code below200 lines; neutral callback naming is deferred, nonblocking. Historical RED, inputs `d860ea62b` and producer wiki `b07e860c8` retain their then-pending states below; B79/B80 terminal corrective proofs unchanged. Audit **IN PROGRESS**.

### Batch81 — producer checkpoint, proof pending

After producer `d7d7b41fc`, [GetUnitDebuff SSOT](../../specs/tooltip-unit-debuff-security.md) owns implementation inventory, inferred policies and proof requirements. Main supplies RED at `6aa7a391b`: compile exit0/zero diagnostics **89.594915s**; **15 FAIL / 0 PASS**, **1.898831s**, stdout hash `1dc876e2559e10a0df345d37f926900af05e3f5994ba9a2ee3c40729bf2cbccc`. Reached boundaries were empty selected payload, original userdata parsing and API error context; this does **not** prove all downstream GC branches. Supplied evidence only; no logs inspected or workers polled here.

Producer shares `c_tooltip_info_indexed_aura`: all three original positions authenticate in secure context before parsing; live harmful player/party state, visibility and PLAYER filtering precede indexing; existing builder retained. Modern old provider retired, older-profile provider retained. Buff behavior is intended unchanged, pending preservation controls. Fabricated target fixture is excluded from the chosen Debuff domain; domain/error/filter policies remain inferred, not native-verified.

At this supplied checkpoint, main GREEN worker3863957 is **pending/compiling** for15 Debuff +18 Buff +24 instance +14 filter cases and startup; verifier707 proof is **pending**. No GREEN, preservation, startup or acceptance credit follows from source or peer reports. Source347 remains **AUDIT-PENDING**, access/restricted outputs **UNMODELED**. B80 accounting unchanged: **84 capabilities,362 ordered IDs;167 pending /177 bounded /11 partial /7 metadata**. Historical authored/compiling inputs checkpoint `d860ea62b` below remains true at its original time; B79/B80 proofs and gaps retained. Audit **IN PROGRESS**.

### Batch81 — exact347 inputs-only checkpoint

Committed `6aa7a391b` contains test/spec inputs for exact347: **15 authored cases**. [GetUnitDebuff contract SSOT](../../specs/tooltip-unit-debuff-security.md) owns the case inventory, chosen policies and proof requirements. At the supplied checkpoint, main asynchronous RED worker3857761 at `/tmp/patch-12.0.5-debuff-indexed-red-ops/` was **compiling, NOT executed or accepted**. No completed behavioral RED, producer, PASS or acceptance is claimed; main supplies next evidence after the worker finishes.

Existing GetUnitDebuff discards unit/index and returns an empty DTO; real harmful player/party lookup is planned, not implemented by these inputs. Source347 remains **AUDIT-PENDING**; unit-aura access/restricted outputs remain **UNMODELED**. No accounting or capability credit. Prior wiki `31d7340c5` correctly records B80 terminal independent701 **PASS32/32 accepted after `7a9ce5d37`**; B79/B80 historical proofs and gaps below remain unchanged. Audit **IN PROGRESS**.

### Batch80 — bounded unit comparison acceptance

After `7a9ce5d37`, [B80 acceptance SSOT](../../specs/unit-identity-equality.md#b80-independent-bounded-acceptance--2026-10-03) owns main-accepted independent698: **12 refreshed PASS at pinned `ad1821805`**, six permission cases and six identity controls. Exact142/143/144 receive bounded credit; introductory141 remains partial. Current accounting: **84 capabilities,362 ordered IDs;167 pending / 177 bounded / 11 partial / 7 metadata**. Independent701 metadata audit **PASS32/32** accepted: actual parent `0c37c90e1`,83 other capabilities and358 other rows unchanged, declared register/hash/policy preserved. Report `/tmp/patch-12.0.5-unit-comparison-accounting-proof.{md,json}`; no execution-gate reruns. Earlier in-flight query failures were corrected before terminal PASS, not repository defects.

Historical B5/GUID GREEN and permission RED remain separate. Saved690 check/startup are inherited unchanged-production evidence, **not refreshed gates**. Native behavior/auth/arity, inferred lexical policy, full identity coverage, older profiles, full suite and GUI/CASC gaps remain open; globalfmt1 at unowned `aura_duration.rs:44` remains uncleared. The linked spec owns detailed proof and qualifications; audit **IN PROGRESS**.

### Batch79 accounting — terminal follow-up

Following the historical pending checkpoint below, main reports terminal independent696 **PASS31** and corrective699 **PASS33**. Earlier FAIL reports were in-flight query defects, not final repository failures; bounded18-table/register-hash accounting is corrected. [B79 SSOT](../../specs/tooltip-unit-buff-security.md#independent-bounded-acceptance--2026-10-03) retains exact342 pending and unchanged status limits. This follow-up does not erase the original checkpoint or grant B80 metadata acceptance.

### Batch79 — exact342 inputs-only checkpoint

Committed inputs `a23adf150`: [GetUnitBuff contract SSOT](../../specs/tooltip-unit-buff-security.md) owns the bounded domain, chosen policies and 18 authored tests. Main performed formatting precommit. At that historical inputs-only checkpoint, asynchronous compiled-RED worker3815774 was active at `/tmp/patch-12.0.5-buff-indexed-red-ops/`; no finished RED, GREEN or acceptance was then claimed. Later evidence does not retroactively change that checkpoint.

Main now supplies actual compiled RED at inputs `a23adf150`: compile exit0 in **88.098727s**, zero diagnostics; **0 PASS / 18 FAIL** in **2.161859s**. Identity/miss, secret-from-stack and authorization-context assertions failed; downstream GC branches were not all reached. Evidence is supplied, not independently inspected here.

Producer `a28293ee0` links [implementation inventory SSOT](../../specs/tooltip-unit-buff-security.md#implementation-inventory): C API unit-indexed lookup over existing visible helpful player/party state plus PLAYER filtering; all three original arguments authenticated before parsing via `unwrap_secret` (internal authorized read, not generic declassification). Modern old provider retired; older-profile body retained. Builder bridge unchanged, including hardcoded `1 hr`. No detailed inventory duplicated here.

At the historical producer checkpoint, GREEN and independent acceptance were pending. Current [bounded acceptance SSOT](../../specs/tooltip-unit-buff-security.md#independent-bounded-acceptance--2026-10-03), committed `1e042a289`, records main-accepted690+694: **18 refreshed at `ad1821805` +38 inherited =56 distinct cases, not56 new runs**. Direct root-table identity test fixed; changed-helper readability clean. Default check and pinned startup reuse remain producer-scoped. Original690 minor STATE/root-only source qualification, root-list identity limitation and RED history remain historical, not retroactively upgraded. Globalfmt1 reports only unowned `aura_duration.rs:44`; no whole-tree clearance.

Capability `tooltip-unit-buff-security` raises **83→84 capabilities**; **362 ordered IDs;167 pending / 174 bounded / 14 partial / 7 metadata** unchanged. Independent accounting is **PENDING a new agent**, not accepted. Prior `5ec536a36`/independent68737/37 remains the historical83-capability checkpoint. Exact342 stays **AUDIT-PENDING**: unit-aura access/restricted outputs **UNMODELED**, general-unit/target-identity/native/old-profile gaps retained. No GUI/CASC/full-suite credit; separate qualified mount/headless/CASC acceptance below remains distinct. Audit **IN PROGRESS**.

### Mount capability — qualified desktop saved acceptance

[Mount acceptance SSOT](../../specs/mount-spell-identifier.md#independent-qualified-saved-acceptance--2026-10-03) owns main-accepted independent674 and retained654/653 proof: bounded mount model, separate headless startup (`casc=false`) and three-texture native CASC decode. No headless CASC integration or GUI/frame/font/all-asset/native API/protected/globalfmt/profile/full-suite readiness credit. Original wrapper failure and missing immutable input/binary/source/content provenance remain disclosed there; no reruns.

Main adds bounded `mount-spell-identifier` from `22ea15a23`: **83 capabilities** (prior82), unchanged **167 pending / 174 bounded / 14 partial / 7 metadata,362 IDs**. Source291 remains **AUDIT-PENDING** for native grammar/AllowedWhenTainted/remaining overall contract; source245/B78 unchanged. Original broad-suite red history remains uncleared. Audit **IN PROGRESS**.

### Batch78 — public direct-spell membership bounded acceptance

After `04586ffe2`, [B78 acceptance SSOT](../../specs/action-bar-membership.md#independent-bounded-acceptance--2026-10-03) owns independent670 acceptance:11 new public membership PASS plus17 existing slot-query controls, local startup `[]`, defaultcheck0/scopedfmt0; globalfmt1 is unowned, not clearance. Credit covers the public direct-spell model only. Source245 remains **PENDING**: special-bar semantics and native AllowedWhenTainted access remain unmodeled. Current accounting: **167 pending / 174 bounded / 14 partial / 7 metadata,362 IDs/82 capabilities**; statuses unchanged, one bounded capability added. Source291 remains pending; desktop probe was unaccepted at this historical82-capability checkpoint. Current separate qualified acceptance is linked above. Audit **IN PROGRESS**.

### Batch76 — existing-model output annotations accepted

[B76 acceptance SSOT](../../specs/unit-stat-output-restriction.md#b76-existing-model-bounded-acceptance--2026-10-03) owns exact430/432/458/460 bounded output-annotation acceptance642 and accounting `ec7386f86`. Retained verifier647 terminal report records31/31 PASS; named raw artifacts were unavailable, as disclosed in the spec. No raw-artifact revalidation is claimed. Current coverage remains **167 pending / 174 bounded / 14 partial / 7 metadata,362 IDs/81 capabilities**; prior counts are historical. No current-process/native-formula/all-profile/input-secret/full-suite GREEN credit; audit IN PROGRESS.

### Batch77 — bounded mount proof; desktop startup pending

After `e49ec0626` and `8dc686e96`, [mount inventory/proof SSOT](../../specs/mount-spell-identifier.md#implementation-inventory) owns actual callable generic-nil RED, producer `22ea15a23`, local16/16 PASS accepted by independent654, and desktop16/16 PASS plus check exit0. These established bounded identifier/model proof; desktop startup/Blizzard UI/CASC readiness were **PENDING** at that checkpoint. Current separate qualified startup/texture acceptance is linked above, not GUI/access or headless CASC integration credit. Exact291 overall contract remains pending; no row promotion. Dirty-combined/transferred-tree provenance limits and native AllowedWhenTainted gap remain in the spec.

### UnitHasPowerType — independent saved-proof acceptance

[Acceptance SSOT](../../specs/unit-has-power-type.md#independent-bounded-acceptance--2026-10-03) owns independent662's accepted five saved PASS and bounded source-supported equivalence, not a new/current-process execution. Source157 was already bounded; no row promotion or count change. No UnitHasPowerType formatting, native secret-input or older-profile publication credit follows. Current coverage remains167 pending/174 bounded/14 partial/7 metadata,362 IDs/81 capabilities; audit IN PROGRESS.

### Batch75 — existing-model output annotations accepted

[Acceptance SSOT](../../specs/retail-missing-stat-inputs.md#b75-existing-model-bounded-acceptance--2026-10-03) records main-accepted independent632: **EXACT466/468/496 output annotations only**, six saved71973 PASS with five named-body equivalence through9441. B74 registration/module declarations changed; no current-process or whole-tree equivalence. Broad13 plus adjacent4 are not new PASS or additional API credit. Native activation/defaults/formulas/nominal type/all-profile limits and OPEN/red full suite retained; B74 gates are separately accepted in the linked bounded SSOT below, not additional B75 credit.

Main-supplied accounting `88362f9a6` promotes exactly three rows/adds one capability: **172 pending / 169 bounded / 14 partial / 7 metadata,362 ordered IDs/79 capabilities**. Independent accounting640 PASS39/39 validates this checkpoint. Earlier175/166/14/7 and78-capability counts below are historical checkpoints, not this accounting state. Audit **IN PROGRESS**; no neighboring input/formula/row credit.

### Batch74 — Ambiguate inputs-only checkpoint

[Ambiguate producer/proof SSOT](../../specs/ambiguate-context.md#b74-producer-checkpoint--2026-10-03) supersedes historical inputs-only status: supplied actual RED `d11` build exit0/zero diagnostics **828.602250s**, **22 cases = 4 PASS / 18 FAIL**, **21.161436s**. Inputs `2a4f34570` cover18 secret-context boundaries/four public controls. Earlier combined E0277 (`eval::<u32>`, repaired to `i32` plus checked conversion) ran no tests and remains not RED.

Supplied producer `9441f7c7c` promotes public transform to a real module/removes old fallback; private-epoch context-secret rejection before argument1, old Lua exact body and inverse boolean `false` retained. Historical pending gates and175/166 checkpoint are superseded by [main-accepted independent625 bounded SSOT](../../specs/ambiguate-context.md#b74-independent-bounded-acceptance--2026-10-03): fresh Retail22 new+2 inert PASS, separate Forever2 public controls, pinned startup `[]`, scopedfmt/check0 zero diagnostics; Forever8 warnings retained. Only satisfied bounded boxes checked; row416, secret argument1/output secrecy/native/all-profile credit withheld.

Source accounting `604a3eea9` promotes415 only; independent644 PASS validates **171 pending / 170 bounded / 14 partial / 7 metadata,362 ordered IDs/80 capabilities**, preserving other361 rows and prior79 capabilities/order/source hashes. B75 accounting64039/39 and scoped source equivalence remain valid at the older172/169 checkpoint. Check/source-to-current whole-tree equivalence is false; other main/build-host commits remain unowned. Dirty-combined hashes and pinned artifacts establish bounded proof only; historical RED, failed compiles, check-launch incidents and globalfmt/process limits retained in SSOT. Audit **IN PROGRESS**; no blanket warning-free/full-suite GREEN.

### Test-only fixture repairs — pending proof checkpoint

[Exterior repair SSOT](../../specs/house-exterior-attached-decor.md#test-only-exterior-fixture-repair--2026-10-03) records `365643a96`, four all-profile assertions with explicit modern host state and saved async64-case exit0; main now accepts independent609 scoped64 exterior +4 pinned-child CLI passes, not new B73 contract acceptance; check interruption and dirty-combined limits remain. [LoC repair SSOT](../../specs/spell-book-loss-of-control-outputs.md#test-only-loc-fixture-repair--2026-10-03) records `c896ffe77`, two explicit book/host fixtures after failed implicit defaults; main accepts independent625's two saved d11 LoC PASS results without new runs; linked SSOT owns exact fixture/equivalence proof and immutable original596 observations. Residual50 ordinary+11 custom failures are identity bookkeeping, not a new full-suite run. Neither repair changes production or clears historical full-suite failures, current whole HEAD or native parity.

### Batch72 — DestroyEntry bounded acceptance

[DestroyEntry contract/proof SSOT](../../specs/housing-destroy-entry.md#b72-implementation-and-accepted-compiled-red--2026-10-03) records producer `7f7d0fe8a` after accepted compiled RED `0f7f47297`, inputs `1b0e7ab99` + `0f7f47297`: **24 tests, 14 retained PASS / 10 new FAIL**, harness 7.33s; compile 261.349678s, exit0/zero diagnostics. Saved run wrapper elapsed 7.364796727s, exit101. Ten secret/authentication failures do not prove downstream GC/event assertions. Artifact provenance and exact test boundaries live in the linked SSOT; historical GREEN/verifier pending status is superseded by **main-accepted independent582 scoped PASS** at `ffb1845bc`, after `7f7d0fe8a` plus pure validation extraction. [Acceptance SSOT](../../specs/housing-destroy-entry.md#b72-independent-bounded-acceptance--2026-10-03) owns fresh **35 PASS = 24 DestroyEntry + 11 Admin/storage**, saved startup0 `[]`, scopedfmt/check0 zero diagnostics. No B71 89-case reuse: full body/test/wiring equivalence not established here.

Source inspection limited to `src/c_api/c_housing/catalog/{storage,destroy_input}.rs`: DestroyEntry-only AllowedWhenUntainted authentication of both original top-level arguments before selector/boolean types, and all three original fields before field types/ranges/domain/model. Underlying table access guard and stack roots remain; existing exact-key storage/count/event behavior and cancellation equivalence are unchanged, not newly accepted. Shared catalog/Admin/pending inputs unchanged. Cached full-variant/BOOL true-all/false-one declaration grounds shape only; eligible mixed-stack/default/error/synchronous-event policies remain inferred. Historical 15-case old-fixture proof remains separate; conservative secure-secret rejection is superseded only for DestroyEntry by accepted B72 GREEN. Model/publication/Admin/shared input bodies unchanged. GC credit is before-call/during-dispatch/nonaliasing only; no input-allocation forced GC, direct top/dispatch-error or queued-payload equality claim. Helper-length/matrix-nesting advisories are nonblocking, not new refactor scope.

Accounting `c6b23304c` updates only existing capability and notes281/282; independent584 accounting PASS43/43; acceptance SSOT links proof. Exact **281/282 remain audit-pending**; **177 pending / 164 bounded / 14 partial / 7 metadata**, 362 IDs/77 capabilities unchanged. No native all-stack or literal-row closure. No broad-row/native/all-profile/full-placement/whole-domain/whole-goal credit. Audit remains **IN PROGRESS**. Four authorized docs only; no delegation, runtime, build, checks, protected-source inspection or broad scans.

### Six source occurrences — metadata-only accounting accepted

After `c7f9635cf`, accepted independent556 [classification proof](../../../../../../../../tmp/patch-12.0.5-narrative-metadata-independent-proof.md) ([JSON](../../../../../../../../tmp/patch-12.0.5-narrative-metadata-independent-proof.json)) reconciles the [coverage SSOT](../../../data/patch-api/sources/12.0.5-page-coverage.json): **180 pending/161 bounded/14 partial/7 metadata,362 ordered IDs/76 capabilities**. Only `prose-2026-03-12-010/011/013/014/020/035` become metadata-only: dated header, title, introduction, author scope disclaimer, tentative formatter proposal and documentation Predicates-catalog announcement. Zero runtime or implementation credit; no claim the proposal shipped or disclaimer excludes simulator work. Register/plaintext and source-hash chain unchanged;356 other rows and all capabilities preserved. [Accounting validation](../../../../../../../../tmp/patch-12.0.5-narrative-metadata-accounting-validation.json) agrees. Concrete adjacent023/026/029/032 remain pending, separately audited, not dismissed. Audit remains **IN PROGRESS**.

Source post2026-03-12; classification commit2026-10-02 23:02:13−05:00 /2026-10-03 04:02:13Z. B69 `fae926860`/`c22c5d5a5` counts below are historical acceptance checkpoints, not current mutable accounting.

### Batch71 — pending-decor bounded acceptance

[Pending-decor acceptance SSOT](../../specs/housing-pending-decor.md#b71-implementation--independent-bounded-acceptance) records main acceptance of independent574 PASS: **89 distinct Retail PASS = 20 pending + 69 catalog**, startup0 `[]`; source/root/security/all-Lua/readability/wiring and import-only equivalence through `d83666b13`. Producer `c685f487`; saved `554efed4a` GREEN compile284.747327s retains one historical unused-export warning. Final scopedfmt/defaultcheck0, zero diagnostics,21.503126679s after export removal; original check218.079007s/one warning includes unmeasured lock wait. No final-new-ELF/zero-diagnostic-GREEN-rebuild claim. Actual `4bec1394a` RED13 PASS/7 FAIL, compile220.479727s/run5.58s; `df00d200` compile failure is not RED. Historical fourteen-test checkpoints retained; pending secure-secret rejection superseded only for pending, shared catalog security unchanged. Advisory helper length rejected as unrelated refactor; no blocking counterexample.

Accounting `c7366a4a0` updates only existing housing-pending-decor capability/notes278279; independent578 accounting PASS34/34; acceptance SSOT links proof. Exact278/279 **remain audit-pending**, no literal placement-delta credit: real placement/finish/instance/stock/events unmodeled. **177 pending/164 bounded/14 partial/7 metadata,362 IDs/77 capabilities unchanged**. Linked SSOT preserves native/type/acquisition/all-profile/cumulative-epoch/dirty/globalfmt/historical-process limits; audit **IN PROGRESS**. Four-doc reconciliation only, no source inspection or execution gates.

### Batch73 — fixture removal bounded acceptance

[Removal acceptance SSOT](../../specs/house-exterior-attached-decor.md#b73-independent-bounded-acceptance--2026-10-03) records saved final artifact acceptance at producer `71973af4b`: **60/60 modern PASS = 19 new + 41 retained**, separate Forever inverse1 PASS, startup exit0 `[]`, scopedfmt/check exit0 with zero diagnostics. Security, atomic Store/Detach, live eligibility, full-key rooted events and GC/reentry have bounded simulator proof; seven-path equivalence is not whole-tree/current-HEAD proof. Independent helper LENGTH advisory remains; inferred/native limits unchanged.

Later full-suite summary corrects provisional counts: **12,022 PASS / 60 FAIL / 18 ignored**, plus **11 separate custom failures**; doctests **0 PASS / 3 ignored**, finished exit0. No blanket GREEN/full-UI/whole-goal clearance. Four seeded exterior-query failures remain without causal/preexistence conclusions. Main accounting `cbf001ba5` promotes only267/268 and adds `house-exterior-fixture-removal`; independent603 PASS33/33. Current **175 pending/166 bounded/14 partial/7 metadata,362 ordered IDs/78 capabilities**; prior177/164/14/7 remains historical. Audit **IN PROGRESS**.

Historical checkpoint: producers `036036e7c`/`8502a2ced`/`71973af4b` implemented modern Remove, live flag and legacy-only no-op; compiled RED `710674128` had19 FAIL/0 PASS,2.63s, compile140.106230s/zero diagnostics. First loader compilation failure was not RED. Initial snapshot710 E0063 required approved fixture repair `9d9015e1e`; snapshot71973 then encountered EXDEV. Runtime/verifier/Forever pending status at that checkpoint is superseded only by the scoped acceptance above; B70 evidence remains separate.

### Batch70 — exterior attached-decor bounded acceptance

[Acceptance SSOT](../../specs/house-exterior-attached-decor.md#independent-bounded-acceptance--2026-10-03) records main acceptance of independent566 for EXACT272/274/276, new boundary tests and retained historical RED. Accounting `0cf868024`: **177 pending/164 bounded/14 partial/7 metadata,362 ordered IDs/77 capabilities**; independent568 accounting PASS30/30; acceptance SSOT links exact proof. Only these three rows promote; no enum/core/remove credit. Prior metadata/B69 counters remain historical. Linked SSOT owns runtime proof/native-inference limits; original globalfmt1, Forever mask-quad FAIL, historical502 process FAIL and protected-path limits remain explicit, no blanket clearance. Broader audit **IN PROGRESS**.

### Batch69 — EXACT313/326 LoC outputs accepted

Producer `cc69ac3c4`; [contract/inventory/proof SSOT](../../specs/spell-book-loss-of-control-outputs.md) and [[lua-api#Retail 12.0.5 spell and spellbook loss-of-control outputs]] describe existing typed LoC-map reads, actual player-book bank0 resolution, original book-selector authentication before model and shared rooted five-field writer. Explicit flag restricts three private NUM fields; two NeverSecret BOOLs stay public. Existing permissive spell parser remains; native AllowedWhenTainted secret-input parity is not credited.

Actual pre-producer `b324f2159289327cc4b8bd74f6793e7e0422824c` RED: **27 selected, 5 PASS/22 FAIL**, compile103.959996s exit0/zero diagnostics, run4.086695s exit101. Public spell record/parser/absence controls pass; meaningful book payload/bank/domain/offspec/missing-map/authentication and restricted outputs fail. Downstream GC mostly unreached, not 22 demonstrated GC failures. Authored/inferred snapshot, field/miss/validation/epoch policies remain distinct from proof; native/type/acquisition/pet/future/UI gaps explicit. Current [independent549+551 bounded acceptance SSOT](../../specs/spell-book-loss-of-control-outputs.md#independent-bounded-acceptance--2026-10-03), accepted after `fae926860`, supersedes pending output claims only. At the B69 acceptance checkpoint, only313/326 promote: **186 pending/161 bounded/14 partial/1 metadata,362 IDs/76 capabilities**. Linked proof retains current gates, separate Forever controls, historical warning/filter/collision/process evidence and all inferred/native limitations; no broader parity credit.

Earlier B68 `758cdf49c` acceptance and docs `e895`/`72df` remain separate: original537 Gate FAIL retained while narrow541 follow-up was accepted by main. No unrelated accounting credit. Full source histories and dirty-combined/globalfmt/process limits remain; no source-code-change or globalfmt/process clearance.

### Batch68 — exact spell and spellbook cooldown outputs accepted

[EXACT305/322 acceptance SSOT](../../specs/spell-book-cooldown-outputs.md#independent-bounded-acceptance--2026-10-02) records independent537+541 and `758cdf49c`: bounded meaningful intervals, book selector/offspec boundaries, trusted-host NUM payloads and Lua opacity/copy/root/GC accepted. Modern/legacy grouping equivalence proved; scopedfmt/check0, 27 refreshed+135 inherited=162 unique Retail PASS/one ignored, startup `[]` inherited. Post-grouping Forever compile0/legacy1 PASS counts one refreshed identity within original64 PASS/1 FAIL/1 ignored, not duplicate credit. Mask-quad test makes no cooldown call: applicability rejected, causal preexistence UNRESOLVED; whole filter not green. Only305/322 promote:188 pending/159 bounded/14 partial/1 metadata,362 ordered IDs/75 capabilities. Inferred policies explicit; nominal/native types, SpellIdentifier input, future fixture, pet/full banks/UI unproved. Dirty-combined/globalfmt1/historical502 failure retained. Dated producer checkpoints remain historical; broader goal open.

### Batch67 — exact spell count outputs accepted

[EXACT301/309 acceptance SSOT](../../specs/spell-count-outputs.md#independent-bounded-acceptance--2026-10-02) owns current accounting, refreshed/reused proof and retained limits. Trusted-host private payload and bounded Lua opacity/copy/root/GC behavior are proved, not pending; cast AllowedWhenTainted secret-input parity remains UNMODELED with conservative rejection, no NeverSecret/input credit. Nominal scalar type, native acquisition, global privacy and UI/full-profile parity remain unproved. Dated checkpoints below remain historical; broader goal open.

### Batch66 — exact action display/use-count outputs accepted

[Exact237/241 proof/model SSOT](../../specs/action-count-outputs.md#independent-bounded-acceptance--2026-10-01) records521+523+525 acceptance:145 distinct Retail PASS (28 refreshed+117 controls), source-valid startup/36 Forever controls, current typed-state check and corrected helper/literal equivalence/readability/format. Only237/241 promote: **192 pending/155 bounded/14 partial/1 metadata =362;73 capabilities**. Ordered IDs,360 unrelated rows,72 prior capabilities/source hashes preserved. Real slot-matched host counts and valid charge-display source before VM input/output privacy; no fabricated acquisition or private nominal-type claim. Native/inferred/default/global-privacy/UI/profile/dirty/globalfmt/historical process failures remain. Header follows main instructional date; observed host provenance separate. Broader goal open.

### Combined64/65 — exact cooldown/LoC output predicates accepted

[Combined233/239 proof SSOT](../../specs/action-cooldown-output-restriction.md#independent-bounded-acceptance--2026-10-02) and [LoC model contract](../../specs/action-loss-control-cooldown-info.md#independent-bounded-acceptance--2026-10-02) record515 acceptance:117 distinct Retail PASS/startup0 `[]`, freshfmt/check/security/wiring/readability plus36 separate existing Forever controls. Only233/239 promote: **194 pending/153 bounded/14 partial/1 metadata =362;72 capabilities**. Ordered IDs,360 unrelated rows,70 prior capabilities and source hashes preserved. Existing cooldown selection and meaningful typed LoC snapshots with three private numeric payloads/two public BOOLs; no generic declassification or nominal-type override. False511 publication diagnosis rejected; opaque VM type assumptions corrected without removing opacity/copy/GC proof. Native/inferred/profile/UI/globalfmt/dirty/historical502 process limits remain. Partial231 unchanged;237 separate; broader goal open.

### Batch63 — exact base-spell specialization boundary accepted

[Exact295 contract/proof SSOT](../../specs/base-spell-specialization-security.md#independent-bounded-acceptance--2026-10-02) records parent acceptance502+505+509:81 distinct Retail PASS plus three separate existing Forever PASS, scopedfmt/check/startup/security/wiring/readability and equivalent assertion split. Only295 promotes: **196 pending/151 bounded/14 partial/1 metadata =362;70 capabilities**. Ordered IDs,361 unrelated rows,69 prior capabilities/source hashes preserved. Existing empty-default specialization model supplies meaningful Retail publication; raw arg2 NeverSecret precedes arg1/model. Conservative arg1 policy/native acquisition/other profiles/UI unknown. Initial wrong no-GUI gate and protected-search process breach remain failures, not erased; globalfmt/dirty limits and broader goal remain open.

### Batch62 — exact item-context additions accepted

[Exact330/331 contract/proof SSOT](../../specs/tooltip-item-context.md#independent-bounded-acceptance--2026-10-02) records497+499 acceptance:24 refreshed focused PASS plus153 reusable controls/startup0 `[]`, producer-scoped fmt/check/security/wiring/readability and four equivalent assertion fixes. Only330/331 promote: **197 pending/150 bounded/14 partial/1 metadata =362;69 capabilities**. Ordered IDs,360 unrelated rows,68 prior capabilities/source hashes preserved. Explicit host level variants change same catalog item and derived budget estimates; no fabricated production data, ignored-argument credit or native acquisition/full variants/quality parity. Eleven pre-existing partial-fixture diagnostics match61; separate startup clean. Dirty/globalfmt/native/profile/UI limits and broader goal remain open.

### Batch61 — six aura-instance input-policy deltas accepted

[Six-row contract/proof SSOT](../../specs/tooltip-aura-instance-security.md#independent-bounded-acceptance--2026-10-02) records parent acceptance of independent486:209 uniquePASS/startup0 `[]`, scopedfmt/check/security/wiring/readability. Only339/340/344/345/349/350 promote: **199 pending/148 bounded/14 partial/1 metadata =362;68 capabilities**. Ordered IDs,356 unrelated rows,67 prior capabilities and source hashes retained. Meaningful player helpful/harmful controls, actual VM secure acceptance/tainted denial and preserved public caller taint; no arg2/3 NeverSecret-removal credit. Dirty/globalfmt/restricted-unit/output/filter/native/profile gaps remain; broader goal open.

### Batch60 — exact pending-cost modifier acceptance

[Exact357 contract/proof SSOT](../../specs/pending-transmog-cost.md#independent-bounded-acceptance--2026-10-02) records parent acceptance of independent477+481+483: ten refreshed focused PASS plus88 reusable controls/startup0 `[]`, scopedfmt/check/security/wiring and three equivalent assertion splits. Only357 promotes: **205 pending/142 bounded/14 partial/1 metadata =362;67 capabilities**. Ordered IDs,361 unrelated rows,66 prior capabilities and source hashes preserved. Explicit optional scalar snapshot only; absent-zero arity/local precision inferred, later BigUInteger evidence not projected backward. Dirty/globalfmt/native/profile/pricing/lifecycle/UI limits remain; broader goal open.

### Batch48 — exact bounded independent acceptance

[Classification acceptance SSOT](../../specs/aura-spell-classification-identifiers.md#independent-bounded-acceptance--2026-10-02) and [exact Add359 SSOT](../../specs/private-aura-anchors.md#exact-add359-bounded-acceptance--2026-10-02) own parent-accepted independent381 scopes, proof and limits. Only359/361/363 promote: **235 pending/113 bounded/14 partial =362;56 capabilities**. Prior54 capabilities/provenance and359 unrelated ordered rows retained;387 regression only. Native gaps remain unchecked; dirty combined proof is not clean revision. Parent owns postcommit validation; prior pending milestones historical.


### Combined aura color/enumeration — bounded independent acceptance

[Combined acceptance SSOT](../../specs/aura-dispel-color-arguments.md#independent-bounded-acceptance--2026-10-02) and [enumeration scope](../../specs/unit-aura-slot-enumeration-arguments.md#independent-bounded-acceptance--2026-10-02) record parent-accepted independent365 plus367 supplement. Only378/382 promote: **241/107/14 →239 pending/109 bounded/14 partial =362**; ordered IDs,360 unrelated rows, prior51 capabilities and provenance preserved. Reused196 uniquePASS/startup0[]/dirtycheck0/VM-security-wiring-readability; fresh eight-filefmt0 after admin formatting-only c436649ad. Original365 overallFAIL/globalfmt1 unowned duration historical, not fixed; dirty combined not clean revision. Runtime31.527s below60 target, bounded DEVELOPMENT not final whole-goal/page/native acceptance. Inferred policies/native permissions/secret POINTS/secrecy/pagination/consumer/profile/full setters gaps retained; advisories deferred. Parent owns postcommit validation; earlier pending checkpoints historical.


### Aura slot arguments — bounded independent acceptance

[Independent353 acceptance](../../specs/unit-aura-slot-secret-arguments.md#independent-bounded-acceptance--2026-10-02) owns150 unique PASS (12 slot +138 controls), startup0[], full hashes/commands and limits. Only376 promotes; unit retains NeverSecret. **242/106/14 →241 pending/107 bounded/14 partial =362**; ordered IDs,361 unrelated rows, prior50 capabilities and provenance preserved. Dirty-combined check0/scopedfmt0, globalfmt1 preserved unowned; later batch45 control bytes excluded. Two old plain CreateColor successes are invalid curve-contract inputs, not378 evidence. Partition55.283s misses60 target, retained without padding; bounded development acceptance, not final whole-goal acceptance. Four readability suggestions deferred; native access/output/profile gaps excluded. Historical pending checkpoints superseded; parent owns postcommit validation.



### Aura application display count — bounded independent acceptance

[Exact three-row acceptance](../../specs/aura-application-display-count.md#independent-bounded-acceptance--2026-10-02) owns independent345 parent acceptance,94 unique PASS/startup0[], full commands/hashes and native gaps. Only367/368 NeverSecret thresholds and369 AllowedWhenUntainted unit/ID promote: **245 pending /103 bounded /14 partial →242 /106 /14 =362**. Ordered362 IDs,359 unrelated rows, prior49 capability bytes and source provenance preserved. Dirty combined proof remains explicit, never clean revision; globalfmt1 unowned aura_duration.rs, scopedfmt0/dirtycheck0. Three nonblocking readability suggestions deferred; no reproduced issue or authorized adjacent refactor. Native permissions/restricted output/full-page parity excluded. Prior pending checkpoints are historical; parent owns postcommit validation and ignored PLAN.

### Indexed aura arguments — bounded independent acceptance

[Exact six-row contract/proof](../../specs/unit-aura-index-secret-arguments.md#independent-bounded-acceptance--2026-10-01) owns accepted80 uniquePASS, committed-producer startup0[], pinned-VM/security/readability and scoped formatting proof. Globalfmt1 on unowned unrelated `aura_duration.rs` retained; dirty combined check0 is not clean-revision proof. No native permission/output policy, full vocabulary, consumer closure or all-profile credit.

Only373/374/384/385/389/390 promote: **251 pending /97 bounded /14 partial →245 /103 /14 =362**. Source IDs/register/plaintext SHA, prior capabilities and356 unrelated rows preserved. Original indexed selection/DTO/stores unchanged; two nonblocking readability suggestions deferred. Older pending checkpoints are historical.

### Private-aura restriction removals — bounded independent acceptance

Exact source deltas401/405 remove `HasRestrictions`; they do not require new aura/warning content producers. Parent accepts the full `/tmp/patch-12.0.5-private-aura-annotation-independent-proof.md`: actual tainted public calls preserve caller taint and cause existing modeled side effects. [Anchor contract](../../specs/private-aura-anchors.md) and [warning contract](../../specs/private-warning-text-anchor.md) retain distinct lifecycle, secret and placement limits.

| Exact row | Bounded behavior | Proof limits |
| --- | --- | --- |
| `global api-C_UnitAuras-RemovePrivateAuraAnchor-401` | `ordinary_public_add_and_remove_preserve_addon_taint`: actual tainted closure deletes existing anchor, preserves insecurity inside and secure outer stack | Exact tainted fixture does not assert zero arity, callback delivery or combat combination. Ordinary controls do; no combined credit. |
| `global api-C_UnitAuras-SetPrivateWarningTextAnchor-405` | `public_binding_after_private_registration_accepts_tainted_caller_in_combat`: actual combat/tainted closure, zero arity, exact private parent/point geometry, preserved taint | Stored geometry, not rendered/native proof. Conservative secret rejection is not native AllowedWhenUntainted acceptance. |

Namespace/initialization changes `4b98920f0`, `0acd750af`, `795e2042e` intersected older batch33 integration proof; verifier330 refreshed only the two existing filters on hash-verified batch39 binary (no build): **18 anchors + 12 warnings = 30 PASS**, exit0 both, 105.990s. Artifact `/tmp/patch-12.0.5-batch40-annotation-20261001T162629-1739b432-runs.json` binds current source and compiled `795e2042e`, binary SHA256 `ebde3a19745cab8140e280c2ec4867766695cdb1462a3cd2ff7cf7551d03efd4`. Full source intersections and output hashes share that prefix; no VM/Cargo change invented. Applicable independent328 fmt/check0 reused on identical Rust scope. Original nonblocking readability findings retained.

Only401/405 gain bounded credit: **255 pending / 93 bounded / 14 partial → 253 / 95 / 14 = 362**. Before snapshot `/tmp/patch-12.0.5-batch40-accounting-before.json`; post-commit validation `/tmp/patch-12.0.5-batch40-accounting-validation.json` preserves ordered IDs, source/register hashes, previous capabilities and all unrelated rows. Prose168 unchanged. Cached warning `PingSystemTutorial`/`string.find` errors and historical broader specialization failure remain unresolved; no clean closure, native secret acceptance, all-profile or whole-page completion. Older pending row notes below are historical checkpoints.

### Outfit stored-index DTO — current bounded acceptance

[Independent315 exact acceptance](../../specs/outfit-catalog-lookups.md#independent-stored-indexdto-acceptance--2026-10-01) owns evidence/limits: current-VM5/5PASS, applicable fmt/check0 reused, no readability issues. Only `structures-TransmogOutfitEntryInfo-673` promotes, **258/90/14 → 257 pending / 91 bounded / 14 partial = 362**. Stored index/DTO fixtures accepted; incomplete second-record per-path assertions explicitly retained, not exhaustive schema runtime proof. No native/catalog lifecycle/all-profile parity. IDs/register/plaintext SHA/unrelated rows preserved in linked artifact; prior unaccepted315 checkpoints superseded. Aura rows392/396 unchanged; duration314 excluded.

### Aura spell identifiers — current bounded acceptance

[Exact independent acceptance](../../specs/aura-spell-identifier.md#independent-bounded-acceptance--2026-10-01) owns proof and source identity: only rows392/396 promote to bounded simulator capability, **260/88/14 → 258 pending / 90 bounded / 14 partial = 362**. Independent73PASS, saved startup0[], freshfmt/check0, no new readability violations. Seeded aliases/order/target fixture and conservative security inferred; no native name/link, generic visibility, all-profile or consumer closure parity. Row394/new duration314 separate; future315 unaccepted, row673 pending. IDs/register/plaintext SHA/unrelated rows preserved; explicit before/after artifact linked in contract. Earlier counts below are historical checkpoints.

### Row169 — current bounded predicate acceptance

[Chronological decision and four-family matrix](../../specs/party-countdown.md#exact-row169-decision--bounded-predicate-acceptance) supersedes the earlier pending rationale below. March25 source line74 names `SetRestrictPings`; March31 line169 changes the restriction condition. Ping action/delivery was an assistant-invented prerequisite, not a source requirement. Parent accepts only explicit chat-lockdown-versus-combat predicate coverage across countdown, ready checks, ping restriction-setting and loot setting, backed by all four independent proofs.

Only `prose-2026-03-31-169` changes **audit-pending → bounded-coverage**: **261/87/14 → 260 pending / 88 bounded / 14 partial = 362**. Row168, source IDs/register/plaintext SHA and unrelated rows unchanged. Native producer/security/error/permissions, exhaustive API inventory and all-profile parity remain unknown; no `C_Ping` action/delivery or whole-page completion claim. Earlier counts/statuses below are historical checkpoints, not current accounting.

### Cooldown abbreviation — bounded independent PASS

[Independent cooldown acceptance](../../specs/cooldown-abbreviation-threshold.md#independent-bounded-acceptance--2026-10-01): saved source/binary-bound **40 PASS**, fresh fmt/check **0**, no reruns. Exact numeric-unit/consumer slices536/538/540 only become bounded: **261 pending / 87 bounded / 14 partial =362**. IDs/register/source plaintext SHA/unrelated rows retained; aliases are annotations, not classes/behavior. Native/secret/error parity unproven; range/ceil/equality inferred. One naming suggestion deferred. PLAN remains ignored local accounting, never staged.

### Party loot method — bounded independent PASS

[Batch34 loot proof](../../specs/party-loot-method.md#reconciled-batch34-parent-proof--2026-10-01) owns saved artifacts, binary hashes, exact coverage and inferred policies. Inputs `42e15e675` compile101/110.49s (unsupported u8) is not RED; corrected `1c7af9c03` compile0/175.01s yields **12 FAIL**. Producer `f2e85fcb6` first GREEN attempt aborts in build.rs after3.40s against a concurrent incomplete countdown fixture, not runtime. Stable fixture `676e4c25a` compiles0/151.54s: loot12 + availability1 + broad legacy filter18 = **19 unique PASS / 31 executions** (12 duplicate loot hits). Saved startup exits0, `[]`, CLEAN0. Same binary run3 yields **10 countdown FAIL**, next-slice behavioral RED, not loot regression.

[Independent loot acceptance](../../specs/party-loot-method.md#independent-bounded-acceptance--2026-10-01): verifier288 accepts **19 unique PASS / 31 executions**, saved startup0 `[]`, fresh fmt/check0. Check snapshots/hash identity at `676e4c25a` remain valid; fmt revision snapshots were overwritten by dotted-basename artifact collision, with exit/time/empty streams recovered explicitly, not exact revision provenance. Later countdown registration/state edits are outside that check. Two nonblocking readability findings (effect-hiding `apply_selection` name; bare master/raid constants) deferred. Native security/permissions/general raid resolution remain unproved.

### Party countdown — bounded independent PASS

[Batch35 parent proof](../../specs/party-countdown.md#reconciled-batch35-parent-proof--2026-10-01) owns producer `27a840b34`, build0/**251.83s**, hash-bound **10 countdown + 12 loot + 6 ready + 7 ping = 35 PASS**, saved startup0 `[]`/CLEAN0. Supersedes countdown RED only; independent292 bounded PASS. No earlier loot check credit for countdown.

Historical countdown checkpoint: **`prose-2026-03-31-169` stayed pending** before the chronological correction above. **264 pending / 84 bounded / 14 partial = 362**, retained IDs/source hash unchanged. Future row decision must specify the bounded explicit-input predicate: chat lockdown blocks these four actions/setters; combat alone does not. No native lockdown production, network delivery, permission enforcement, full-page or all-profile credit.

[Independent countdown ledger](../../specs/party-countdown.md#independent-bounded-acceptance--2026-10-01): saved10+25controls35PASS, freshfmt/check0, startup0[]. Inferred lifecycle/security; one magic3600 readability finding deferred. Earlier pending-because-ping-action rationale withdrawn by the chronological correction above. Row168 separate; 362IDs/hash/counts unchanged. Cooldown parent GREEN recorded in linked batch36 proof; independent299 pending.

### Private warning placement — bounded independent PASS

[Batch33 acceptance](../../specs/private-warning-text-anchor.md#independent-bounded-acceptance--2026-10-01) owns exact evidence and limits. Independent report `/tmp/patch-12.0.5-warning-placement-independent-proof.md` accepts bounded saved producer `dae082322` proof: **12 warning + 18 anchor + 4 private-unit PASS / 1 historically established specialization FAIL** (34 PASS / 1 FAIL). Fresh default fmt/check exit **0** at producer; normal saved startup exit **0**, `[]`. Cached `PingSystemTutorial` string.find closure errors persist; broad controls and clean cached closure remain **NOT GREEN**. Two function-length suggestions (`validate_placement`, `apply_placement`) deferred as nonbehavioral blockers, not zero readability findings. Parenting/order/nil/snapshot/security policies remain inferred or unknown; no native, full-row/page or all-profile acceptance. **264 pending / 84 bounded / 14 partial = 362**, source IDs/hash unchanged. Exact warning rows `prose-2026-03-31-168` and `global api-C_UnitAuras-SetPrivateWarningTextAnchor-405` remain pending.

### Party ping restrictions — bounded independent PASS

[Batch32 parent proof](../../specs/party-ping-restrictions.md#reconciled-batch32-parent-proof--2026-10-01) owns corrected compiled RED `12e4a1a28` **0 PASS / 7 FAIL**; initial missing-trait compilation failure excluded. Producer `77ab2785f` compile exit0, saved **7 ping + 6 ready-check + 5 predicate + 11 group = 29 selected PASS**, parent startup exit0 `[]`; exact metadata/hashes in linked ledger. Full independent report accepts bounded saved runtime/startup proof and fresh default fmt/check exit0; relevant ping/guard/wiring/build and control-fixture hashes match producer despite warning-fixture/docs HEAD advancement. Linked spec owns exact gate provenance; no warning-runtime credit. Strict public enum validation, secret rejection, blocked error and None default are inferred simulator policies; no native `AllowedWhenUntainted`, actual ping delivery, role permissions or ping events proof.

Historical ping checkpoint: March31 **`prose-2026-03-31-169` stayed audit-pending** with independently accepted ready-check and ping subset links only. Countdown has parent GREEN above, independent292 bounded PASS; loot has bounded independent acceptance. Historical March25 superseded prose retained. **264 pending / 84 bounded / 14 partial = 362**, IDs/source SHA and unrelated statuses unchanged; **IN PROGRESS**, no whole-source/page or all-profile credit.

### Chat lockdown ready checks — bounded independent PASS

[Ready-check proof](../../specs/chat-lockdown-ready-checks.md#reconciled-batch31-parent-proof--2026-10-01) owns actual compiled RED `8649fe072` (**1 PASS / 5 FAIL**, compile 0/162.49s) and producer GREEN `f62420536` (**6 ready + 5 predicate + 11 group = 22 PASS**, compile 0/296.24s), plus saved startup exit0 `[]`. Intervening docs `6f264ce3e` is not compilation provenance. Full verifier266 report now accepts bounded saved behavior and wiring/readability; fresh default fmt/check exit0 at clean `6f264ce3e`, relevant producer/test hashes unchanged. Later concurrent ping inputs excluded; linked spec owns exact gate artifacts. Error reporting, legacy alias and explicit-input policy are simulator inferences, not native restriction parity.

Historical ready-check checkpoint: **`prose-2026-03-31-169` remained audit-pending** with bounded independently accepted ready-check subset; [Later ping proof](../../specs/party-ping-restrictions.md#reconciled-batch32-parent-proof--2026-10-01) now has separate bounded independent acceptance. Countdown has parent GREEN above, independent292 bounded PASS; loot has bounded independent acceptance. Superseded March25 prose is not the final contract. **264 pending / 84 bounded / 14 partial = 362**, IDs/text SHA256 and unrelated statuses retained; **IN PROGRESS**, no whole-row/page/native/all-profile acceptance.

### Chat lockdown predicate — bounded independent PASS

[Predicate proof](../../specs/chat-messaging-lockdown.md#reconciled-batch30-bounded-proof--2026-10-01) owns independent bounded acceptance: saved five predicate + two controls PASS at `18b09cbf9`, ancillary parent startup0 `[]`; fresh fmt/check once exit0 at `0d247625c`, original scope matches producer. Only row251 gains explicit-input one-boolean/no-second-reason coverage: **264 pending / 84 bounded / 14 partial = 362**, IDs/source hash/unrelated rows preserved. RED nonboolean nil, not two returns. Ready-check `8649fe072` and concurrent code excluded; no native producer/security/enforcement/macros/all-profile/full-page claim. **IN PROGRESS**.

### Private anchors — bounded independent PASS

[Batch29 bounded proof](../../specs/private-aura-anchors.md#reconciled-batch29-bounded-proof--2026-10-01) owns saved producer, event correction and actual BuffFrame fixture evidence. Startup's 21 earlier messages all disappear after exact retail-12.1 event correction; do not treat distinct messages as independent root causes. Container fixture now passes actual cached Add/Remove/re-add/unit transition after supplying inherited Symbol through the real root. Explicit simulator dispatch is not native event production or historical 12.0.5 availability.

Full independent report `/tmp/patch-12.0.5-private-anchor-independent-proof.md` accepts bounded saved behavior, not runtime reruns: fifteen original anchor PASS at `2b24386c5` plus one at `053f6c860`, one migrated integration and seven shape controls separately reusable. Callback-error **2 PASS** actually compiled/run at `91e845021` (test commit `f69497fc6`, identical file hash); inferred simulator no-rollback/error recovery characterization, not native policy. Fresh default fmt/check **0**, no compiler warnings, at clean `053f6c860`; callback file has separate targeted format proof. Chat inputs `f777027be` excluded.

Only literal structure rows **626** (`AddPrivateAuraAnchorArgs + isContainer`), **675** (`UnitPrivateAuraAnchorInfo + isContainer`), **676** (`+ parent`) gain bounded coverage for false/default and explicit true input/output flags and original usable callback/list parent identity including GC. Restriction rows359/401 remain pending. **265 pending / 83 bounded / 14 partial = 362**; source IDs/text SHA256 and unrelated rows unchanged. Native AllowedWhenUntainted, restrictions, native producer, destruction/rendering/full-page/all-profile parity remain unproven; audit **IN PROGRESS**, broad suite **NOT GREEN**. No builds/tests/delegation/push in this docs followup.


### Housing category search and removed raw fields — bounded independent PASS

[Batch26 search contract/proof](../../specs/housing-category-search.md#reconciled-batch26-bounded-proof--2026-10-01) owns producer `a608db343`, actual search RED **0/12**, saved **78 PASS/startup 0 []**, fresh independent default fmt/check **0**. Exact row **661** gains bounded new-filter-name/raw old-key exclusion and actual cached consumer bridge coverage. Featured-parent/order/explicit mode policies remain inferred; native AllowedWhenUntainted, full DTO and ordinary-searcher filtering unclaimed.

[Batch27 raw DTO proof](../../specs/housing-catalog-aggregates.md#reconciled-batch27-raw-output-proof--2026-10-01) owns tests-only `9cdb13a59`: separate saved **13 aggregate PASS**, fresh fmt **0**, prior production check valid. Literal assertions support exact rows **653–657** only: removed output fields absent in populated snapshots from all three raw getters, with local legacy injection and fresh current DTO preservation. Row652 remains a control; input645/646 unchanged. No invented RED for existing absence or absence claim on the cached wrapper, which intentionally adds aliases.

Current **268 pending / 80 bounded / 14 partial = 362**; all source IDs and text SHA256 retained. Saved behavior/startup independently inspected, not rerun; separate executions are not a combined 91-test run. Whole rows/native/all-profile/full catalog acceptance remains open; audit **IN PROGRESS**.

### Housing category DTO rename — bounded independent PASS

[Batch25 contract/proof](../../specs/housing-catalog-categories.md#reconciled-batch25-bounded-proof--2026-10-01) owns inputs `63b53dfe5`, producer `bbbacf8f0`, actual RED **2 PASS / 10 FAIL**, saved **66 GREEN PASS**, parent startup **0 []** and fresh independent default fmt/check **0** at clean producer. Explicit empty maps replace two seeded getter publishers; exact distinct keys, required/nullable fields, independent boolean and fresh nested snapshots have bounded proof. Saved runtime/startup inspected, not independently rerun; rooting/registration source-reviewed. Missing-ID nil and snapshot policies inferred.

Rows **643/659** receive **bounded-coverage** for `anyOwnedEntries`→`anyStoredEntries` only. **274 pending / 74 bounded / 14 partial = 362**, all IDs/text hash retained; **IN PROGRESS**. Batch25 does not cover row661; former seeded search/getter mismatch is addressed by [bounded batch26 search proof](../../specs/housing-category-search.md#reconciled-batch26-bounded-proof--2026-10-01); housing panel safety is unproven. Native AllowedWhenUntainted acceptance, ownership derivation, full catalog and all-profile claims excluded; later search/editor-context inputs not covered.

### Housing explicit aggregates — bounded independent PASS

[Batch24 contract/proof](../../specs/housing-catalog-aggregates.md#reconciled-batch24-bounded-proof--2026-10-01) owns inputs `fe874979b`, producer `72795fbfa`, compiled RED **2 PASS / 10 FAIL** (all ten stop at stored-field assertion, not isolated placed failures), saved **12 aggregate + 14 base + 24 variants/count + 11 storage + 15 destruction = 76 PASS** and parent startup **0 []**, zero errors, not independent execution. Fresh independent default fmt/check **0** at docs `2973774c5` covers identical relevant code/config. Explicit fields through three selectors, zero/nil distinction, unsigned range, independent snapshots and no mutation coupling are bounded simulator proof. `None`→nil is missing-data gap against cached required native numbers, not native default.

Exact structure rows **650/651 remain audit-pending**, linked only for bounded explicit fields. **276 pending / 72 bounded / 14 partial = 362** and all IDs/text hash retained. No synchronization, variant derivation, full DTO, nonempty wrapper, native or all-profile claim. Category spec remains separately owned; audit **IN PROGRESS**.

### Housing base selectors — bounded independent PASS

[Batch23 contract/proof](../../specs/housing-catalog-variants.md#reconciled-batch23-bounded-proof--2026-10-01) owns producer `346c7e1be`, inputs `1e452eac3`, compiled predecessor RED **0/12**, saved **14 base + 24 variants/count + 14 pending + 4 cart = 56 PASS**. Saved parent startup **0 []**, zero errors, is not independent execution. Independent default fmt/check **0** at clean `346c7e1be` excludes later batch24. Root cause: old base selectors read seeded data rather than explicit base records; producer replaces only these publishers/exclusive copier with public item-ID/link and exact record/type lookup, fresh bounded snapshots and no seed/variant fallback.

Only `global api-C_HousingCatalog-GetCatalogEntryInfoByItem-284` and `global api-C_HousingCatalog-GetCatalogEntryInfoByRecordID-286` receive **bounded-coverage** for removed trailing `tryGetOwnedInfo`, not broad query/full DTO/native row completion. Name support, native AllowedWhenUntainted and nonempty deprecated wrapper remain open; ambiguity error inferred. Two late-added controls have GREEN, no predecessor RED. Current accounting **276 pending / 72 bounded / 14 partial = 362**; IDs/text hash preserved. Batch23 proof excludes later batch24; aggregate proof is recorded separately above. Audit **IN PROGRESS**.

### Housing pending decor — bounded independent PASS

[Pending request contract](../../specs/housing-pending-decor.md#reconciled-bounded-proof--2026-10-01) owns independent acceptance at `f59c03402`: RED `659f79a3c` **1 PASS / 13 FAIL**, saved GREEN **45/45**, snapshot default fmt/check **0**. Explicit full-variant pending request only; eligibility/cancel/validation are simulator inferences. Parent startup **0 []** is saved, not independent compilation provenance. Exact StartPlacingNewDecor rows 278/279 gain proof links but **remain audit-pending**, like DestroyEntry: inferred partial behavior does not cover the literal delta across real placement. Historical batch22 accounting: **278 pending / 70 bounded / 14 partial**, all 362 IDs/source hash preserved; not current B71 counts. No finish/instance/3D/stock mutation/event production/native/all-profile or entire-domain claim. Audit **IN PROGRESS**.

### Housing DestroyEntry — bounded independent PASS

[Destruction contract](../../specs/housing-destroy-entry.md#reconciled-bounded-proof--2026-10-01) owns independent acceptance at `67b44f2c3`: saved **15 destruction + 11 storage + 24 catalog = 50 PASS**, snapshot default fmt/check **0**. Exact rows `global api-C_HousingCatalog-DestroyEntry-281`/`-282` alone link bounded selector rename/full variant-argument coverage; pending statuses retained. Mixed eligible subset, missing/zero no-op, consistency errors and synchronous mutation/event policies are simulator inferences, not native all-stack proof. Conservative secret rejection leaves native secure-access parity open. Parent batch21-green startup **0 []** is saved, not independent. All 362 IDs/source hash and unrelated rows/counts retained: **308 pending / 40 bounded / 14 partial**. No native/all-profile or whole-row/page completion.

### Housing storage event — bounded independent PASS

[Storage event contract](../../specs/housing-storage-entry-updated.md#reconciled-bounded-proof--2026-10-01) owns bounded independent acceptance at `5afd73d49`: saved **11 storage + 24 catalog + 5 quest + 28 party = 68 PASS**, snapshot default fmt/check **0**. Exact event rows 555/556 gain bounded first-argument rename/full variant-type coverage via explicit admin state producer. Synchronous timing/edge-only emission are simulator inferences; cached `UniqueEvent` is not native synchronous evidence. Parent batch20 startup **0 []** is saved, not independent. All 362 IDs/hash and unrelated accounting retained; **308 pending / 40 bounded / 14 partial**. No DestroyEntry-triggered/native/all-profile or whole-row/page claim; concurrent DestroyEntry work excluded.


### Housing destroyable count — bounded independent PASS

[Count contract](../../specs/housing-destroyable-count.md#reconciled-bounded-proof--2026-10-01) owns independent bounded acceptance at `3068e48d2`: saved **10 count + 14 variant controls PASS**, default fmt/check **0**, relevant source hashes unchanged. Exact rows `global api-C_HousingCatalog-GetDestroyableInstanceCount-288`/`-289` alone gain bounded coverage for parameter name/type and full variant-key lookup. Explicit count is independent of stored count; default/missing zero is simulator inference. Parent `batch19-green-startup-run.json` exits 0 `[]`, not independent startup proof. Native secure-secret parity/count policy and all-profile execution remain unverified. All 362 IDs/source hash and unrelated accounting retained: **310 audit-pending + 38 bounded-coverage + 14 partial-development-green = 362**. Storage events/mutations/placement excluded; no whole-row/page completion.

### Housing catalog variants — bounded independent PASS

[Variant contract](../../specs/housing-catalog-variants.md) owns actual parent RED **0/11** at input `05ca7dff0`, build/run artifacts, bounded capabilities, security limits and saved parent controls; [bounded independent report](/tmp/patch-12.0.5-housing-variants-independent-proof.md) accepts only `157d15cef`. New C API-owned queries serialize only explicit entry/variant inputs; source/results preserve full IDs in separate containers. Registration remains unconditional, matching the replaced temporary surface. Native publication is bound into the retained Lua lifecycle under `src/c_api/c_housing/catalog/`; callback and parameter methods remain, but no filter/sort/async/count policy is implemented.

Root cause: temporary queries/searches read hardcoded seeds instead of `HousingState.catalog`, collapsed selector type checks and published obsolete variant fields. Exact three query keys, searcher factory and replaced seeded publishers are removed from the temporary owner. Shared legacy ByItem/ByRecordID, count, storefront/cart/exterior/customize providers remain excluded and unchanged; none supplies fallback data to the new queries. Public addon selectors preserve taint; nested secrets reject without unwrapping, secured-table restrictions are checked. Conservative rejection is not native AllowedWhenUntainted parity.

Parent repaired compile at `157d15cef` passed; variants **14/14 PASS**, plus **4 cart + 4 free-place + 1 customize + 2 decor controls PASS**. Saved startup exits **0**, output **[]**; linked contract owns revision/hash metadata and the earlier cart-run boundary. Independent report confirms saved **14 variants + 11 controls PASS**, default fmt/check **exit 0**, scoped to nine hash-bound files at `157d15cef`, not the later checkout. Eight exact source rows receive bounded coverage: search output rename/type rows 524/525/527/528, base record/type and removed entryID rows 648/649/652, and dyeColorName row 663. Filtering, full DTO and native security remain partial; no other housing row gains credit. [Guarded-selector diagnosis](../../specs/housing-catalog-variants.md#guarded-selector-fixture-root-cause) traces the sole failure to retail's no-op `settablesecurity`, not catalog bypass. The test now installs real pinned VM policy locally and asserts secure/tainted source and catalog accesses; repaired parent execution passed without production changes. Initial 13/14 was a fixture-premise failure; earlier zero-match filters establish nothing, unlike later executed controls. Original seed assertions were reconciled through explicit fixtures, not deleted. No new builds/checks/delegation/push in this docs reconciliation; no native/all-profile or whole-catalog acceptance. Batch19 count is accepted separately below; its later model/fixtures remain excluded from batch18 proof. The following input section records the historical pre-producer checkpoint.

### Housing catalog variant inputs — historical pre-RED checkpoint

[Variant contract](../../specs/housing-catalog-variants.md) owns exact source/filter accounting, cached retail `12.1.0.69933` declaration hashes, eleven unrun grouped fixtures and replacement-test mapping. Empty-default C API-owned typed inputs are referenced by `HousingState.catalog`; all output providers remain unchanged. No build/check/delegation, actual fixture RED, query implementation or row credit. Actual compiled behavioral RED is required before any producer edit.

Ownership inspection corrects the temporary input plan: `env_init::init_lua_state` calls `globals::register_globals` → `missing_surface::register_all` → `c_housing::register_c_housing_surface` **before** `apply_temporary_bootstrap`. Lua `__wow_merge_namespace` fills only raw-missing keys, so eventual Rust ownership must register exact keys before that merge and remove only replaced Lua publications/helpers. Existing Rust category-name placeholder and BasicMode free-place ownership stay separate. Seed entry/variant helpers also serve featured products and customize selection; deleting the whole bootstrap or its shared seed maps would change excluded owners. This input commit deletes neither.

Cached GetAllSearchItems explicitly describes the source collection, not results; current Lua aliases both getters and uses row lengths for owned-instance count. Filter-free fixtures do not adopt either aliasing or count semantics. Base-info versus variant-stack schemas also reject the plan's proposed variant-zero entryID and stored=destroyable assumptions. Remaining legacy ByRecordID seed tests, missing secret-field proof and complete entry DTOs are recorded conflicts/gaps, not fallback requirements. All 362 accounting IDs/statuses remain unchanged.

### DamageMeter input only — pending RED

[Structure contract](../../specs/damage-meter-combat-source.md#reconciled-bounded-proof--2026-10-01) owns reconciled proof: producer `1a9fcd1ec`, registration `e38d98a89`, import `c1dce16c3`, host fixture `5b0644b33`. Default and historical Retail 12.0.0 each nine PASS; prior seventeen controls narrowly reused; captured default fmt/check exit 0. Shapes, empty input, selectors, reset and independent nested snapshots covered; authentic host-secret rejection and explicit combat errors are bounded policies, not native secrecy. Parent startup exit 0 `[]` remains saved, not independent. Historical heading retained for links. Exact row `structures-DamageMeterCombatSource-639` is **PARTIAL** due sensitive value/disclosure and native gaps; all 362 IDs and source hash retained, other accounting unchanged. No combat/native/all-profile/full-addon UI or whole-row/page closure.

### Spell confirmation prompts — bounded independent PASS

[Prompt contract](../../specs/spell-confirmation-prompts.md#tests-asserting-this-spec) owns producer `b07fc61f6`, compiled/docs `329eabfbb`, independent six prompt + twelve control PASS and snapshot-scoped default fmt/check exit 0. Validation/security/rooting remain source-only; no native/all-profile or whole-row/page acceptance. Exact event source rows 560/561/562 now have bounded coverage; all 362 IDs and source hash retained. Separate parent startup exits 0 with `[]`, zero Lua errors, 46.39s; not independently verified. Normal binary and integration emitted by batch16 combined build, which still exits **101** from unrelated wow-sim test `AddonMetadata.addon_dir` missing at `enable_state.rs:225`. Audit remains **IN PROGRESS**.

The expanded Patch 12.0.5 source audit is **IN PROGRESS** after source-retention commit `7ff275fd3`; full-page behavior coverage is not established. Earlier work was probe-driven rather than a full API-change-page audit. Retail `12.0.5.67823` live probes pinned core frame, event, attribute, identity, scale-event, and XML frame-level behavior; the simulator already models the safe findings with regression coverage. No dedicated `patch_12_0_5_inert_defaults` module exists.

## Content

### Quest accept confirmation — bounded independent PASS

[Quest confirmation contract](../../specs/quest-accept-confirmation.md#tests-asserting-this-spec) owns producer `e5ab15302`: actual RED 1 PASS/4 missing-method FAIL, GREEN build exit 0 in 510.99s, saved 5 quest + 6 prompt PASS and fresh independent 13 lifecycle controls PASS. Default fmt/check exit 0 is snapshot-scoped; security/rooting remains source-only. Separate parent hash-bound normal `wow-sim` startup exits 0 with `[]`, zero Lua errors, 12.70s; not verifier-run. Historical unrelated bin-test failure remains recorded, not a broad-suite GREEN claim. [Coverage register](../../../data/patch-api/sources/12.0.5-page-coverage.json) links exact `events-QUEST_ACCEPT_CONFIRM-558` to bounded simulator input while preserving all 362 IDs/source hash. Native, loaded-popup integration, all-profile and full-row/page acceptance remain unclaimed. Audit remains **IN PROGRESS**.

### UNIT_CONNECTION party transitions — bounded GREEN

[Party connection contract](../../specs/party-connection.md#tests-asserting-this-spec) owns exact retained `prose-2026-03-31-182`: tests `03ebe6972` actual RED 0/6 at missing setter; implementation `527cb2f57` GREEN 8 party + 28 admin-party + 19 admin-event = 55 PASS, saved startup exit 0 with `[]` and zero Lua errors. Shared explicit input/state/query transitions dispatch actual synchronous two-argument disconnect/reconnect listeners with post-mutation query observations. Cached `UnitDocumentation.lua:4057-4065` explicitly declares synchronous `(unitTarget, isConnected)`; defaults, change-only emission and lifecycle policies remain simulator inferences. [Coverage register](../../../data/patch-api/sources/12.0.5-page-coverage.json) retains all 362 IDs and unrelated rows. Independent report records bounded behavior/wiring/readability PASS and default fmt/check at captured `527cb2f57` source/config scope, excluding subsequent unrelated tests and concurrent `tests/forever_auto_roll.rs` edits. Parent startup is saved evidence (7.85s), not independently verified by that report. No current whole-worktree formatting, native, networking, all-profile execution, full-row or whole-page claim. Audit remains **IN PROGRESS**.

### UnitSpellTargetName snapshot — partial GREEN

[Cast-target contract](../../specs/unit-spell-target-name.md#tests-asserting-this-spec) owns exact retained `prose-2026-03-12-041` accounting, a dated PTR proposal rather than a consolidated shipped-API claim. Inputs `c14076510` yield actual missing-query RED 0/10; producer `5beaf7545`, compiled at `024afed64`, yields target GREEN 10/10, flyout 14/14 and vehicle/possession 22/22. Saved startup exits 0 with `[]`. [Coverage register](../../../data/patch-api/sources/12.0.5-page-coverage.json) remains partial: player caster only, explicit actual-cast snapshot input, no actual targeting producer, nonplayer caster model or native claim. Linked contract records independent bounded behavior/security PASS and default fmt/check snapshot `4c5aeb2d8`, excluding later party tests `03ebe6972`. Ten RED cases hit missing query surface, not ten independent behavioral failures; readability length finding is advisory. All 362 source IDs and unrelated statuses retained; no whole-page completion.

### Recent Allies snapshot producer — bounded GREEN

[Recent Allies contract](../../specs/recent-allies-state-data.md#tests-asserting-this-spec) owns exact `structures-RecentAllyStateData-669` accounting: inputs `d0495e796`, actual RED 0/4 at `ee3e27172`, producer `3666902bf` actual GREEN 4/4. Explicit-input nested snapshots publish opposite renamed flags, optional nils and independent results. Six run-1 source/TOC controls pass, not addon runtime integration; saved same-revision startup exits 0 with `[]` and zero Lua errors. [Coverage register](../../../data/patch-api/sources/12.0.5-page-coverage.json) records bounded development coverage only and retains all 362 source IDs. The linked contract now owns completed independent bounded PASS, scoped readability and default fmt/check gates over unchanged source `f442b0913` → `a956dfdd3`. Newer cast input `c14076510` is not covered; no current full-source, native, full-system or whole-page claim. Previous query was a generic lazy namespace nil closure, not an explicit targeted provider; nested stack-rooting is documented in the contract.

### Source scope

The original audit used live-client probe addons under `docs/addons/` and corresponding wiki investigations, not the full patch page. Commit `7ff275fd3` retains the entire plaintext **Patch 12.0.5/API changes** extract in [12.0.5-api-changes.txt](../../../data/patch-api/sources/12.0.5-api-changes.txt), with [retrieval provenance](../../../data/patch-api/sources/12.0.5-api-changes.provenance.json): retrieved 2026-09-30, SHA-256 `4da3872aa566695f46e2dacd4e79992f5b06be9541f0d19cf0e8dba45cea8329`. This is plaintext, not raw wikitext or its link graph.

The source inventory `/tmp/patch-12.0.5-inventory.json` counts **244 consolidated delta rows / 186 distinct subjects**, plus **118 chronological prose rows**; some prose is narrative rather than an API. Consolidated sections cover Global API, ScriptObjects, Widgets, Events, Enums, and Structures. The source has **0 CVar entries**. These are source-coverage counts, not implemented API counts or behavioral proof. Chronological PTR proposals and future plans must remain distinct from the consolidated snapshot; 362 inventory rows do not imply 362 shipped APIs.

**Guessed implementation policy:** the user explicitly requests best-supported guesses for missing APIs rather than stalling for native evidence. Label each guessed contract as a guess, record its supporting evidence, and track a concrete future probe identifying the call/input scenario and observable results needed to resolve uncertainty. Guesses and simulator tests do not establish native semantics. Existing probe-register classifications below are preserved, not silently reclassified by this expanded policy.

Primary retained 12.0.5 probe sources (13 SavedVariables captures): `AnimScriptProbe`, `AttributeDispatchProbe`, `CoreBehaviorProbe`, `DevToolsDumpProbe`, `FrameIdentityProbe`, `HookScriptBindingProbe`, `IsProtectedProbe`, `JustifyProbe`, `ProtectedRetailProbe`, `ScaleEventProbe`, `SetAtlasProbe`, `StoreForbiddenProbe`, and `TextureSetTextureProbe`. `XmlFrameLevelProbe` findings are documented, but its raw capture was not retained.

The machine register is `data/patch-api/12.0.5-probes.json`, sourced from `data/patch-api/sources/12.0.5-probes.json`; [[patch-12-0-5-probe-inventory]] is its human-readable inventory. It preserves 38 probe subfindings. Current machine classification is **33 best-effort, 0 implemented, 4 evidence-required, 1 exception-requested, and 0 untriaged**: the exception is approved provenance-only, while four behavior gaps remain evidence-required—one impossible same-size input-boundary gap and three unsafe Store/security gaps.

### Expanded capability and future-probe coverage

The six committed slices below are a **documentation/source-inference inventory, not code proof**. Linked specs own implementation proof updates; their recorded targeted results are not independently checked here. Full patch audit remains **IN PROGRESS**; independent final gate is pending, and none of these slices establishes native equivalence or adds credit to the 38-subfinding register.

| Commit / contract | Bounded capability described by spec | Evidence boundary / future native probe |
|---|---|---|
| `326e571b8` — [Action text](../../specs/action-text.md) | `UsesActionText` / `GetActionText` read occupied macros' exact current nonempty names through rename, move, replacement and deletion. | Label qualification is inferred; spec reports targeted 5/5 GREEN, not final proof. Probe names/whitespace/body variants, populated item/spell/empty slots and lifecycle changes; real item slots and cross-profile execution remain unproven. |
| `5bfca2e16` — [UnitHasPowerType](../../specs/unit-has-power-type.md) | One non-secret boolean from present-unit primary and explicitly modeled player secondary capability; cumulative 12.0.5 publication. | Absent-unit, pet/vehicle alias and strict argument policies are inferred; [independent662 acceptance](../../specs/unit-has-power-type.md#independent-bounded-acceptance--2026-10-03) covers five saved PASS by bounded source equivalence only. Source157 already bounded; no new row/count, formatting, native or older-profile credit. Native probes remain open. |
| `f372687b6` — [Scenario unit criteria](../../specs/scenario-unit-criteria.md) | Supplied per-token credit/percentage/display triples; empty-default map and explicit row identity restriction, with guarded secret input/output. | Missing-row zero-return and row restriction policies are inferred; proof checkboxes remain open. Probe no scenario/unknown/zero-credit units, reassignment, localization, restriction transitions and tainted/secret calls. Host-result adoption `6eace49d5` now passes four bounded scenario cases, including tainted plain-token restricted output; secret-input guards remain. Native identity policy and independent final proof remain open. |
| `e8ebb47c1` — [Enum publication and source/proof matrix](../../specs/patch-12-0-5-enum-additions.md#retained-source--proof-matrix) | Current-retail additions, removals and rename; source-backed related shifts and actual-member metadata. | Parent RED observed; batch7 post-change publication/metadata GREEN 20/20 observed. Spec owns exact source accounting and per-test proof. Historical/native numbering and downstream domain semantics are unclaimed; existing `Relinquished` remains unchanged. |
| `9d89c7021` — [Outfit catalog lookups](../../specs/outfit-catalog-lookups.md) | Four queries share an empty-default catalog; documented fields, case-insensitive names, explicit indices, fresh tables and guarded secret arguments. | Empty enumeration, Unicode/duplicate/order and invalid-input policies are inferred; focused GREEN/final verification pending in spec. Probe empty results, Unicode normalization/casing, sparse indices and exact invalid-input returns/errors. No mutation/persistence/lifecycle claim. |
| `50feedbfe` — [Cooldown decimal threshold](../../specs/cooldown-decimal-threshold.md) | Stored Lua threshold drives formatter: one decimal below threshold, whole-second ceiling otherwise; zero/reset and aura/abbreviation precedence. | “Below” is documented; zero-disable, ceiling and precedence are inferred. Spec lists formatter tests, not GPU/native proof; probe 19.9/20.0/20.1 at threshold 20, zero/changed thresholds, mode precedence and rollover. |

### Renown reward model and producer

`766272cdc` adds `RenownRewardInfo` under `src/c_api/c_major_factions/renown_rewards.rs`, publicly exported by `c_major_factions`: this is a C API backing input, not Lua glue. `SimState.major_faction_renown_rewards` is an unconditionally empty-default `(majorFactionID, renownLevel)` map of row sequences; `is_collected: Option<bool>` distinguishes absent, false and true. No production rewards are fabricated.

Exactly `structures-MajorFactionRenownRewardInfo-665` is the source link for this input-only slice. The [renown reward contract](../../specs/major-faction-renown-rewards.md) owns declared fields, model-choice inferences and future probes. The producer now publishes explicit pair-keyed snapshots and replaces only the temporary query default. The linked contract owns the actual batch8 RED and batch9 nine-case GREEN, declared fields, numeric/secret selector policy and earlier-profile/native limits. Input-only evidence was not publication proof; current bounded development proof is not independent final acceptance. No whole-source completion or native-probe classification changes.

### Current-retail enum publication

`e8ebb47c1` corrects current-retail enum values in `c_api/patch_12_0_5_enums.rs`, after base publication. The `client-retail` gate preserves historical/PTR surfaces. Existing 12.1 compatibility fills additional members; refresh derives metadata from actual numeric members after initialization and post-load. Disabled photo-status insertion requires shifted existing values, and current unit-frame names/values are grounded in cached documentation and the existing strict removal, not old sequential positions. No unrelated enum import or vendor edits.

The [enum spec](../../specs/patch-12-0-5-enum-additions.md#retained-source--proof-matrix) is the single source for exact row/subject counts, literal values, removal/rename cases, doc lines and grouped test proof. Earlier 36/24 accounting is superseded by that retained-source matrix. Independent bounded enum acceptance is recorded in the linked spec; numerical publication does not establish native/domain parity.

### Batch4 bounded development proof

Compiled revision `9a50d8a5cc20d0adf0b7c529d237fc043ef57532`: retained `/tmp/patch-12.0.5-batch4-integration-build.{json,log}` records successful integration compilation and executable `integration-0915b883f3757151`. The actual binary ran focused filters; these are **development GREEN**, not independent final acceptance, native proof, or API-coverage counts.

| Exact source row / capability | Executed scope | Remaining boundary |
|---|---|---|
| `prose-2026-03-25-091` — [FontString smooth scaling](../../specs/fontstring-smooth-scaling.md) | Six API cases PASS: isolated state, validation/secret caller guards, fractional and auto height, wrapping, ordinary/runtime XML. | This batch does not execute renderer tests; GPU/native metrics and earlier XML-profile support remain unproven. Source says XML predates the API; 12.0.5+ publication does not cover that earlier claim. |
| `prose-2026-03-12-019` — [Common duration formatting](../../specs/duration-core.md#common-numeric-formatting) | Four cases PASS: three real formatter instances, duration/modifier dispatch, invalid-input atomicity and secret provenance/caller guards. Abbreviated 4/4 and numeric-rule 7/7 controls PASS. | Native localized Seconds duration units remain RED in the separate batch4 regression (raw 93); source-wide formatter compatibility is not complete. Conservative secrecy is simulator policy, not native parity. |
| [Scenario unit criteria](../../specs/scenario-unit-criteria.md) — host-result adoption `6eace49d5` | Prior four-case host-secret development run PASS, including tainted plain-token restricted outputs without contaminating public values. | Explicit row classification remains supplied input; native identity policy and final acceptance remain open. |

Evidence: `/tmp/patch-12.0.5-batch4-{font-api-green,duration-common-green,abbreviated-control,numeric-rule-control}.log`, `/tmp/patch-12.0.5-scenario-host-secret-green.log`, and current `/tmp/patch-12.0.5-proof-ledger.md`. [Page coverage links](../../../data/patch-api/sources/12.0.5-page-coverage.json) attach only these exact supported source rows; other accounting remains audit-pending. Whole page remains **IN PROGRESS**. No new credit or reclassification enters the 38-subfinding native-probe register.

### Batch5 bounded development proof

Default integration build `e0a46d691` PASS: `/tmp/patch-12.0.5-batch5-integration-build.{json,log}`. Exact executable argv, filters, exits and log paths: `/tmp/patch-12.0.5-batch5-runs.json`. These runs supersede pending batch4 output claims only within the scopes below; independent verifier 76 is pending. No final Rust checks, native-client parity, or full-page completion.

| Exact source rows | Development result | Remaining boundary |
|---|---|---|
| `prose-2026-03-31-147` | Stat restriction 4 PASS covering 40 supported API outputs. | Explicit supplied restriction state; missing stats 10 FAIL; base models and native activation/parity pending. |
| `prose-2026-03-31-141`–`144` | UnitIsUnit final permission matrix 6 PASS, including nil denials. | Earlier chronology remains classified separately; no native parity. |
| `prose-2026-03-31-160`; consolidated `global api-C_ActionBar-GetActionCooldownDuration-235`, `global api-C_Spell-GetSpellCooldownDuration-307`, `global api-C_SpellBook-GetSpellBookItemCooldownDuration-324` | ignoreGCD 6 PASS. | Supplied cooldown/GCD selection only; no secrecy or native consumer parity. |
| `prose-2026-03-12-019` | Common format 5 PASS including curve/closure taint; Seconds format 7 PASS including cached garden consumer, configuration 7 PASS, native-method controls 3 PASS. | Native-enabled simulator output supersedes prior raw-93 RED/pending; not native-client parity or entire three-formatter statement completion. `/tmp/duration-numeric-formatters-proof.json` retains chronology. |
| `prose-2026-03-12-023` | Charge duration 1 PASS / 3 FAIL. Cooldown countdown formatter separately 8 FAIL. | Implementations pending agents 73/74/75; no new GREEN credit. `GetSpellChargeDuration` resolves through `runtime_surface_bootstrap.lua` lines 65–76 (`__wow_namespace_mt.__index` lazy nil closure), explaining no-data PASS; `C_Spell.GetSpellCharges` uses a temporary zero-table default. Models/producers are missing, not API globals. Other static C_* absence reports also require final-provider tracing. |

The compile-fixture FF failure gate fix `e0a46d691` is not Forever GREEN: other-main profile compilation remains pending. All 362 source row IDs and source-register chronology/classifications remain intact; linked rows can still contain unproven clauses. Counts are inventory, not capabilities. Source accounting and the entire 12.0.5 goal remain **IN PROGRESS**. The 38 native-probe subfinding statuses are unchanged.

### Bounded unit-stat output secrecy (verified 2026-10-01)

[Unit-stat restriction spec](../../specs/unit-stat-output-restriction.md) owns the exact 50-row coverage matrix and pending base-model names. `8da12a42c` adds the plain `SimState.unit_stats_restricted` boolean, default false, and matching `C_Secrets.ShouldUnitStatsBeSecret`; `f35d0293f` adds concrete grouped fixtures; `edab2563a` marks all numeric results of the 40 supported default-retail APIs while retaining existing values, arity and order. Ten unsupported base models remain separate pending work; secrecy support does not establish native stat-model parity.

`c_secrets::push_stat_number` accepts only a concrete Rust-computed `f64`, uses rilua's trusted host-number producer, and roots the result immediately without changing caller taint. The module compiles across profiles, but predicate and aura registrations retain their feature gates; profiles without `retail-12-0-5` push plain numbers. The existing zero-return producer was shared by source-covered PvP/resilience APIs and unrelated miss/enemy queries: only the source trio uses the restricted producer now.

Observed predicate RED at `a3ba2a23a` was missing publication. Output RED at `9a50d8a5c` passes the predicate and fails three output tests, including `GetAttackPowerForStat secrecy 1`; log `/tmp/patch-12.0.5-batch4-stats-output-red.log`. Batch5 at `e0a46d691` passes four stat-restriction cases covering the 40 supported API outputs; ten missing base-API cases fail. Independent verifier 76 and parent-owned final gates remain pending. Restriction activation is an explicit approved simulator input, not an aura/combat heuristic or native-verified policy. Future native probes and absent-unit limitations remain in the spec. Full patch audit remains **IN PROGRESS**; no register reclassification.

[Patch-page discovery index](../../../data/patch-api/patch-page-index.json) discovers **138 API pages / 98 retail-history candidate titles**. This is title classification only: neither retained page-content coverage, shipped API classification nor runtime/behavior proof.

### Itemized probe status

**Machine-classified with direct behavioral evidence:** the full Frame/AnimationGroup/nine-subtype script-handler matrix; repeated scalar/false attribute dispatch and the two-panel ShowUIPanel pulse; normal-frame forbidden behavior and absent retail forbidden constructor; valid/invalid unit-event filters; wildcard false/true/string attributes; Raise/Lower level boundaries and GUI mouse-focus ordering; frame identity slot, surrogate dispatch, duplicate-frame freshness, and DevTools frame-array dump metadata; normal HookScript chaining plus rejected explicit slots 0 and 2; absent legacy protection setters, the full plain-frame and XML-protected-frame sequences, and protected secure templates; frame-layer FontString default points, size variants, explicit anchors, implicit ButtonText anchors, EditBox backing regions/TextInsets, and MessageFrame/ScrollingMessageFrame owner-region behavior; complete observable display/UI-scale/CVAR ordering; the complete invalid-atlas argument matrix; texture path/FDID and clear behavior; and bare/fixed/parent/reparent XML frame-level semantics and flags.

**Evidence and exception state:** XML raw-capture provenance is the only approved exception because it concerns missing historical evidence while frame-level behavior is independently regression-tested. Same-size transitions remain an evidence-required impossible input-boundary gap. Secure Store behavior, Store dropdown population, and Store forbidden descendants remain evidence-required unsafe Store/security gaps; existing subsystem tests are not substituted for the missing probe behavior.

### Open probe gaps and evidence-required rows

A broad approval recorded on 2026-07-14 is superseded. The five rows below distinguish four item-specific evidence-required behavior gaps from one approved provenance-only exception-requested row. Evidence-required rows carry hashed repository evidence but need no approval, commit, or focused test; they await authoritative/live evidence or correct implementation.

1. **ProtectedRetailProbe.SecureStore — evidence-required unsafe:** Retained Store frames are forbidden, legacy setters are absent, and `IsProtected` errors; the current simulator returns normally, so exact forbidden/secret-return enforcement is unsafe to guess.
2. **ScaleEventProbe.SameSizeDuplicatePair — evidence-required impossible:** Retained live observations already establish that maximize/restore can produce another ordered display/scale pair without a dimension change. The missing boundary is the production window-transition signal, not screen-size evidence: the probe records dimensions and state, while the simulator receives only draw-time `iced::Size` and deliberately ignores equal sizes. Pinned iced 0.14.0 / winit 0.30.12 expose no maximize/restore/fullscreen transition notification. Their mode/maximize queries are not ordered transition events, so polling would be an approximation. Correct behavior remains unmodeled.
3. **XmlFrameLevelProbe.RawCaptureProvenance — approved impossible:** Behavior is regression-tested, but the raw SavedVariables capture does not exist and cannot be reconstructed locally.
4. **StoreForbiddenProbe.DropdownPopulation — evidence-required unsafe:** The retained capture has `StoreDropdown_SetDropdown == nil`, so population, reuse, text/check, callback, and protection behavior was never observed.
5. **StoreForbiddenProbe.ForbiddenDescendants — evidence-required unsafe:** The retained file lacks the `/sfp` manual descendant scan, so Store descendant forbidden/protected state is unknown; correct behavior remains unmodeled pending authoritative/live evidence.

The 38-row register is complete only for its explicit probe contract; generic fallbacks cannot be claimed as globally patch-complete without another concrete source.

### Completed modeled work

Retail `12.0.5.67823` probe results are modeled in these areas:

- `CreateForbiddenFrame` is absent on current retail, and `SetForbidden(true)` on addon-created normal frames succeeds without making the frame forbidden.
- `RegisterUnitEvent("UNIT_HEALTH", "not_a_unit")` registers the event but drops the invalid unit filter; `IsEventRegistered("UNIT_HEALTH")` returns registered with no unit filter.
- Wildcard `GetAttribute` preserves an explicit `false` stored with `SetAttribute("*type1", false)`.
- `Raise()` / `Lower()` only affect same-raw-level tie ordering and do not let a lower frame level overtake a higher one.
- Frame identity dispatch uses `frame[0]` userdata tokens; surrogate tables shaped with `[0] = frame[0]` dispatch shared frame methods, while `[1]`-only surrogates do not.
- Duplicate named `CreateFrame` calls produce fresh Lua objects and fresh identity tokens rather than copying stale custom fields from the prior global binding.
- XML bare `frameLevel` is an absolute initial value, not a parent-relative offset, but remains non-fixed so later parent level changes shift the child by the captured parent delta; `fixedFrameLevel="true"` pins the level.
- `DISPLAY_SIZE_CHANGED` and `UI_SCALE_CHANGED` fire as an ordered pair for observable size/scale recalculations, with startup pairs before `PLAYER_LOGIN`.

Key implementation locations:

- `src/lua_api/frame/methods/text_attribute_event/events.rs` — invalid `RegisterUnitEvent` filter fallback and animation handler validation.
- `src/lua_api/frame/methods/text_attribute_event/attributes.rs` — retail forbidden-frame and attribute behavior.
- `src/lua_api/methods.rs`, `src/lua_bridge/table_builder.rs`, `src/lua_api/globals/create_frame/helpers_shared.rs` — frame identity token dispatch and duplicate named-frame behavior.
- `src/lua_api/globals/template/direct/frame_level.rs` — XML frame-level resolution and fixed/non-fixed propagation.
- `src/lua_api/env_runtime.rs`, `src/startup.rs`, `src/iced_app/resize_event_tests.rs` — display/scale event pair behavior.

### Verification

Regression coverage exists in:

- `tests/admin_event_api.rs` — invalid unit-filter registration fallback.
- `tests/protected_frame_enforcement.rs` — retail `SetForbidden` no-op behavior.
- `tests/protected_attribute_enforcement.rs` — wildcard explicit-false lookup and repeated-false dispatch ordering.
- `tests/frame_level.rs` — Raise/Lower and raised-frame-level ordering.
- `src/iced_app/mouse_tests.rs` — GUI hover, `GetMouseFocus`/`GetMouseFoci`, and Raise/Lower focus ordering.
- `tests/security_api.rs`, `tests/frame_table_iteration.rs`, `tests/globals_legacy.rs` — frame identity slot, surrogate dispatch, opaque identity userdata, duplicate named-frame freshness.
- `tests/xml_frame_strata.rs` — XML `frameLevel` and `fixedFrameLevel` semantics.
- `src/iced_app/resize_event_tests.rs` — display/scale ordered-pair behavior.

### Same-size transition boundary

`ScaleEventProbe` captures screen and physical dimensions, UI-scale CVars, and `UIParent` scales with each ordered event pair; it does not need another dimensions capture. `App::sync_screen_size_to_state` receives only `iced::Size` during drawing and does nothing when both dimensions match. The pinned window stack has resize, move, focus, and scale-factor notifications but no mode/maximize/restore/fullscreen transition notification. There is therefore no production input that distinguishes an equal-size maximize/restore transition from no transition. Do not fire another pair merely because the same size is observed again, and do not add an admin event as a fidelity substitute.

### Remaining inert/default surface

There is no 12.0.5-specific inert-default module. Broad compatibility defaults still live in `src/lua_api/workarounds/temporary/` and permanent unsupported C API shims, but the 12.0.5 probe-backed findings listed above have modeled behavior and tests rather than patch-scoped inert stubs.

The remaining generic defaults are intentionally outside this 12.0.5 audit unless a probe or addon failure ties one to a 12.0.5 retail behavior contract. Examples include unsupported 3D/model domains, loose/placeholder namespace defaults, and compatibility fallbacks that are tracked by their own subsystem investigations.

### Audit state

The expanded source audit remains **IN PROGRESS**, with no completed full-page behavior claim. Separately, the historical 38-subfinding probe register remains open with 4 evidence-required rows and 1 approved provenance-only exception-requested row. The four behavior gaps are one impossible same-size input-boundary gap and three unsafe Store/security gaps; they are not exception or approval candidates. Authoritative/live evidence or correct behavior is still required before this audit can close. No 12.0.5-specific inert-default module remains, but absence of a patch shim is not proof that every retained probe result has exact regression coverage.

### Secret-string formatting bounded runtime proof

[Secret-string formatting spec](../../specs/secret-string-formatting.md) covers only retained line 40, `prose-2026-03-12-040`: secret `%s` ignores width/precision and preserves full payload; public formatting is unchanged. Allowed tainted opaque formatting is **inferred**, not native-verified permission; input/result unwrap guards and stack taint remain intact, with no arbitrary callback capability. `SetFormattedText`, display provenance and other formatting domains remain unproven.

MAIN published rilua `host-secret-bool` revision `6044544b960cd68b4b0c58bb3373412757c2caee` after explicit user approval and remote verification; simulator pin commit `c5ba89ae3` changes only Cargo/lock pin. Independent runtime report `/tmp/rilua-secret-format-independent-proof.md` records 6 formatter PASS, 2 host-guard PASS, fmt/check PASS with pre-existing `strlen` warning—not warning-free. Actual simulator old-pin RED at `eac08bda3` is 1 public PASS / 5 secret FAIL (`/tmp/patch-12.0.5-batch6-secret-format-red.log`). New-pin batch7 compiled successfully and simulator secret-formatting integration PASS 6/6; independent final acceptance and native proof remain unclaimed. Concrete future probe and remaining guard/display boundaries live in the spec. Full page remains IN PROGRESS; 38-row probe classifications are unchanged.

## Sources

- [GetUnitBuff bounded acceptance SSOT](../../specs/tooltip-unit-buff-security.md#independent-bounded-acceptance--2026-10-03) — B79 accepted690+694 proof and historical qualifications; exact342 and independent accounting remain pending.

- [Unit-stat output restriction](../../specs/unit-stat-output-restriction.md) — exact supported/pending matrix, proof and future native probes.
- [Retained full plaintext patch page](../../../data/patch-api/sources/12.0.5-api-changes.txt) and [provenance](../../../data/patch-api/sources/12.0.5-api-changes.provenance.json) — expanded source audit, not behavior proof.
- `/tmp/patch-12.0.5-inventory.json` — working source inventory; temporary artifact, not a committed manifest.
- [Patch-page discovery index](../../../data/patch-api/patch-page-index.json) — title-only discovery; capability contracts and commit references are linked in the table above.

- [[retail-core-behavior-probes]] — core 12.0.5 live-client behavior findings.
- [[frame-surrogate-identity-slot]] — frame `[0]` identity-token behavior.
- [[display-size-ui-scale-events]] — display/scale event pair behavior.
- [XmlFrameLevelProbe](../../../docs/addons/XmlFrameLevelProbe/README.md) — live XML frame-level probe notes.
- [CoreBehaviorProbe](../../../docs/addons/CoreBehaviorProbe/README.md) — live core behavior probe notes.
- [FrameIdentityProbe](../../../docs/addons/FrameIdentityProbe/README.md) — live frame identity probe notes.
- [ScaleEventProbe](../../../docs/addons/ScaleEventProbe/README.md) — live display/scale event probe notes.
- [ScaleEventProbe source](../../../docs/addons/ScaleEventProbe/ScaleEventProbe.lua) — captured dimensions, CVars, and event order.
- [screen-size synchronization](../../../src/iced_app/update_runtime.rs) — simulator's size-only input boundary.
- [Cargo lockfile](../../../Cargo.lock) — pinned iced 0.14.0 and winit 0.30.12.

## See Also

- [[patch-12-0-5-probe-inventory]] — only 38 native-probe subfindings, not full patch-page coverage.

- [[patch-12-0-7-api-audit]] — later additive API bridge audit pattern.
- [[patch-12-1-api-audit]] — PTR API bridge audit pattern.
- [[lua-api]] — Lua runtime surface and frame method dispatch.
- [[retail-core-behavior-probes]] — retained 12.0.5 core probe evidence.
- [[event-system]] — event registration/dispatch behavior.
- [[xml-template-system]] — XML template and frame-level handling.

## Batch7 observed proof — 2026-10-01

Observed batch7 default build snapshot `c5ba89ae3d35a951cd77ca8b773b4bfc56ad9ebd`, rilua `6044544b960cd68b4b0c58bb3373412757c2caee`, compiled successfully in 34m51s. Exact argv, artifact SHA256 and referenced outputs: `/tmp/patch-12.0.5-batch7-integration-runs.json` and `/tmp/patch-12.0.5-batch7-lib-runs.json`. Independent verifier 104 report `/tmp/patch-12.0.5-batch7-independent-proof.md` was not yet available when recording these logs; no independently validated final acceptance, native parity or whole-page completion is claimed.

Twelve integration filters PASS 111 cases: stats 13, restriction 4, charges 10, duration 27, countdown API 8, enums 20 + representative control 1, secret printf 6, common 5, Seconds 7/7/3. Library configured countdown 6 PASS / 2 FAIL; existing countdown/static/current EditMode controls PASS 8/2/1. Historical 12.0.0 filter zero is NOT proof. At that snapshot, post-snapshot `99102f631` GC and `97a0e1270` fixture GREEN, renown RED/producer proof and structure producer proof were pending. The superseding bounded evidence below does not alter historical batch7 counts.

## Batch8/9 bounded capability update — 2026-10-01

| Exact retained source IDs | Capability / authoritative proof | Status boundary |
|---|---|---|
| No retained `SetCountdownFormatter` row identified; no fabricated source link | [Cooldown formatter](../../specs/cooldown-countdown-formatter.md#batch8-independent-renderer-proof--2026-10-01): independent batch8 renderer 8/8 PASS at `693883c77` supersedes historical two failures. | Library text, not GPU/native or final whole-project acceptance. |
| `structures-AreaPOIInfo-628`, `structures-AreaPOIInfo-629` | [Area POI](../../specs/area-poi-patch-12-0-5.md): moved C API provider; explicit suppression/locking inputs; 2 new + 8 controls PASS. | Mainline development proof only; legacy execution pending. |
| `structures-MajorFactionRenownRewardInfo-665` | [Renown rewards](../../specs/major-faction-renown-rewards.md): empty-default pair model and snapshot producer; 9 PASS. | Optional `isCollected` publication, not native reward eligibility or earlier-profile proof. |
| `structures-PvpBrawlInfo-667`, `structures-TransmogAppearanceSourceInfoData-671`, `structures-ViewedTransmogOutfitSlotInfo-678` | [Structure inputs](../../specs/patch-12-0-5-structure-inputs.md): three explicit-state snapshot producers; 9 PASS. | Return/selector/security policies remain inferences; required-input/native probes pending. |
| `prose-2026-03-31-154`; `prose-2026-03-31-173` | [UnitName](../../specs/unit-name-secret-tokens.md): existing provider 3 PASS. [Propagators](../../specs/insecure-propagator-templates.md#evidence-and-pending-proof): superseding batch10 getter/inheritance proof. | UnitName wrapper behavior is not native-token proof; propagator independent bounded getter/inheritance acceptance is recorded in the linked spec; input/security/native behavior remains unclaimed. |

Batch9 saved successful build/run attribution at `4f9e1607c`: `/tmp/patch-12.0.5-batch9-{build-result,runs}.json`, logs `integration-0.log` through `integration-5.log`. New query/UnitName filters total **31 PASS** (10 + 9 + 9 + 3); propagators separately **1 PASS / 2 FAIL**. Independent producer audit `/tmp/patch-12.0.5-batch9-independent-proof.md` is complete: confirms those 31 PASS and saved no-addons/no-saved-vars startup `[]` at `4f9e1607c`, scoped to the inspected producers/fixtures, not all retained statements. Specs own behavior and inference details; the [exact source coverage](../../../data/patch-api/sources/12.0.5-page-coverage.json) retains all 362 IDs and bounded/partial statuses. Current runtime fmt/check and post-batch10 startup remain pending; saved batch9 startup is not current-revision acceptance. Page remains **IN PROGRESS**.

### Batch10 bounded follow-up

[Propagator contract](../../specs/insecure-propagator-templates.md#evidence-and-pending-proof) is the SSOT for XML repair, five-case GREEN and verifier 124's completed bounded independent report. Source prose 173 has bounded getter/inheritance coverage only, not whole-row completion.

Charge-policy filter `cooldown_restriction::` at `84f48be77` records **2 PASS / 6 FAIL** in `/tmp/patch-12.0.5-batch10-run-1.log` and `runs.json`, after corrected compilation/imports (not behavioral RED). Producer 123 owns [charge policy](../../specs/cooldown-restriction.md); no charge spec or C API change in this docs slice. Existing empty-data/type-input rule is unchanged.

### Batch7 snapshot startup

`/tmp/patch-12.0.5-batch7-startup-run.json` binds the new-pin `c5ba89ae3` wow-sim compiler artifact to the no-addons/no-saved-vars `lua-errors` command: exit 0, 26.89s. `/tmp/patch-12.0.5-batch7-startup-lua-errors.json` is `[]`; `/tmp/patch-12.0.5-batch7-startup-lua-errors.log` reports CLEAN, zero unique/occurrence errors. Independent audit pending; later GC/query changes are not covered. No current-revision final gate claim.

## Batch11 bounded charge policy — 2026-10-01

Batch11 at `d8a93bec37dc09980e41dd37b63d0dd26be88668` records **24 PASS / 0 FAIL**: policy 8, charge controls 10, ignoreGCD controls 6. `/tmp/patch-12.0.5-batch11-runs.json` binds all three logs (`run-0.log` through `run-2.log`) to integration artifact SHA-256 `a4d6874b74ceb453fe834a4e155d7ba58315f1e76c647b659631c3d478ded72f`; each exits 0. Historical startup metadata `/tmp/patch-12.0.5-batch11-startup-run.json` binds the same revision to `lua-errors`, exit 0, stdout `[]`; the saved executable hash is qualified in the [restriction contract](../../specs/cooldown-restriction.md#tests-asserting-this-spec). Shared builds later overwrote that target path; no current-binary startup or rerun claim. Independent saved-run confirmation does not establish native parity or whole-page completion.

[Restriction contract](../../specs/cooldown-restriction.md) links actual [source register](../../../data/patch-api/sources/12.0.5-register.json) IDs `global api-C_ActionBar-GetActionCharges-231`, `global api-C_Spell-GetSpellCharges-303`, and `global api-C_SpellBook-GetSpellBookItemCharges-320`. Only their bounded charge-table output policy gains coverage, not other normalized predicate rows. The predicate itself is retained in the 12.0.0 register, not invented as a 12.0.5 row. Explicit input is not automatic combat/encounter/M+/PvP hooks or per-spell exceptions. Secret spell identifiers `AllowedWhenTainted`, other cooldown returns and inherited policies remain unsolved. Ordinary/legacy older-profile execution remains unclaimed. All 362 source IDs and unrelated statuses remain unchanged.

The [restriction contract](../../specs/cooldown-restriction.md#tests-asserting-this-spec) now owns `/tmp/patch-12.0.5-charge-policy-independent-proof.md` and `/tmp/patch-12.0.5-batch11-rust-gates.json`: independent 24 PASS confirmation, default fmt/check exit 0 without warnings at `5e15752e2` through docs-only `74e6c8845`, and zero changed Rust/config hashes. These gates cover the recorded revision/source scope only, not all profiles or all page capabilities. XML verifier 124 independently confirms five bounded propagator cases, not physical input/security/native parity. All 362 IDs, 20 capabilities and counts (328 pending, 22 bounded, 12 partial) retain their classifications. Whole-page status remains **IN PROGRESS**.

## Current-Retail enums — bounded independent PASS

[Enum contract/proof](../../specs/patch-12-0-5-enum-additions.md#reconciled-bounded-proof--2026-10-01) owns exact 30-row reconciliation and unchanged-source, saved batch7 **20/20 PASS** binding. Publication/removal/rename and actual-member metadata are the external contract; downstream domains are not value-publication requirements. Only the specified 30 pending enum rows gain bounded coverage. Current accounting: **278 audit-pending + 70 bounded-coverage + 14 partial-development-green = 362**; all IDs/source hash and unrelated housing accounting preserved. Historical 12.0.5 numbering, PTR/native/all-profile/current-binary runtime and downstream behavior remain unproved; audit stays **IN PROGRESS**. No new execution.

Exact403/328 bounded acceptance: [combined evidence SSOT](../../specs/private-aura-sound-removal.md#independent-bounded-acceptance--2026-10-02), with [arg2-only space scope](../../specs/string-util-space-limit-security.md). Native and whole-page/goal acceptance excluded.

## Wikitext completeness check and publication sweep — 2026-10-04

The page's raw wikitext (revid 6747894, [retained](../../../data/patch-api/sources/12.0.5-api-changes.wikitext)) holds five collapsed consolidated tables with **363** entries ([register](../../../data/patch-api/sources/12.0.5-wikitext-register.json)); header counts equal parsed counts. **216** are missing from the crawler register: every added/removed row (78/19 globals, 60/6 script-object methods, 11 widget methods, 12/1 events, 10/19 CVars); 6 of those symbols appear only in blue-post prose. All 147 changed rows are present. Conversely, **0** crawler-register consolidated subjects are absent from the page's tables (Enums/Structures checked against raw text). The generator previously skipped 53 label-prefixed changed rows (`PlayerScript`/`Unit`/`Localization`); fixed without changing the 12.0.7/12.1.0 registers.

The [publication sweep](../../specs/patch-12-0-5-publication-sweep.md) probes all 363 rows on default Retail with 12.0.7→12.1.0 supersession (5 rows): 306 OK, 57 reviewed gaps ([result](../../../data/patch-api/evidence/12.0.5-session-2026-10-03/p1205-wikitext-sweep-result.json)). Fixes gated `retail-12-0-5`: retired `C_GossipInfo` delve members, `C_NamePlateManager.SetNamePlateHitTestFrame` and the `C_HousingPhotoSharing` namespace from the autostub; renamed `secret*`→`addon*RestrictionsForced` and the other CVar adds/removals; `HOUSE_EXTERIOR_DECOR_HIDDEN_CHANGED` registerable, `CATALOG_SHOP_PMT_IMAGE_DOWNLOADED` not; `HousingCatalogSearcher:ToggleStoredOnly/ToggleBaseVariantOnly`. Negative control: one flipped register row produced exactly one new gap.

## Wikitext supplement — 2026-10-04

The plaintext extract and crawler register missed every added/removed row of the page's collapsed tables: raw wikitext (revid 6747894, [register](../../../data/patch-api/sources/12.0.5-wikitext-register.json)) lists 363 symbols, 216 absent from the crawler register. The generator also skipped 53 labelled changed rows (fixed; 12.0.7/12.1.0 registers regenerate unchanged). [Sweep](../../specs/patch-12-0-5-publication-sweep.md): 306 OK, 57 reviewed gaps — 23 added globals and 21 methods the cached docs declare but the simulator doesn't publish (`C_HousingInspectMode`, `C_PhotoSharing`, `SecondsFormatter` ×16…), 5 nameplate hit-test methods with no Lua factory, alias-only deprecations, callback-only talent events. Fixed on the way: autostub-fabricated 12.0.5 removals, `secret*RestrictionsForced` → `addon*RestrictionsForced` CVar rename and other CVar/event deltas. Superseded-by-later-patch rows are metadata-only.
