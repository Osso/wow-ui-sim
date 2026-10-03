# B98 outfit / formatter independent verification

Revision: `a4cce2db1639d7d88752925d06facc78f4ee5746` (HEAD inspected). Read-only artifact verification; no cargo, test binary, repository mutation, agents or model CLIs. Caller reports GREEN 404/404, including seven outfit tests and control suites; I did not independently execute those tests. Counterfactual RED below is source inspection, not a newly executed RED.

## Artifact gate

[EXIST] PASS — all requested producers, tests, specs and source text present. Representative sizes: actions 95 lines; outfit tests 230; formatter configuration tests 195; tick producer 131; countdown-consumer tests 405.

[SUBSTANTIVE] PASS — catalog lookup changes host-owned selection; formatter attachment stores authenticated handles in private GC roots; engine ticks generate configured text consumed by countdown rendering. Neither slice is a placeholder.

[WIRED] PASS — outfit registration: `src/c_api/registration.rs:69` → `c_transmog_outfit_info.rs:43`; macro route `src/lua_api/globals/spell_macro_verbs.rs:178`. Formatter methods registered at `cooldown.rs:742,744`; tick called at `src/lua_api/on_update.rs:68`; renderer consumes configured text at `src/iced_app/quad_builders_cooldown.rs:174–176`. Integration harness includes generated modules (`tests/integration.rs:1`); renderer tests included at `quad_builders_cooldown.rs:4–5`.

[ANTI-PATTERN] PASS — requested implementation/test files contain zero TODO/FIXME/HACK/XXX markers. Retained legacy Lua is executable epoch-isolated compatibility behavior, not commented-out code.

## 1. Outfit action and /outfit — ACCEPT WITH QUALIFICATIONS

### Source coverage / recommended row status

| Rows | Verified bounded behavior | Not established |
|---|---|---|
| `prose-2026-03-25-111`, `prose-2026-03-31-165` (source lines 111,165) | Added outfit secure-action handler reaches catalog-index-backed change/toggle/clear state through unchanged cached vendor dispatch. | Actual appearance application, availability/disabled eligibility, events, protected authority, physical click/XML template loading, persistence/native parity. |
| `prose-2026-03-25-122`, `prose-2026-03-31-179` (122,179) | Conditional numeric `/outfit` examples do not error; matching conditions execute rather than merely ignoring the command. Sparse catalog indices, semicolon branches, combat state, bang/no-toggle and bare clear work. | Complete macro grammar, conditional-only empty clear branches, complete Lua `tonumber` grammar. |

Recommend **bounded** for all four rows with those exclusions, not complete-row acceptance. No extra prose clause defines viewed/pending lifecycle, native eligibility or events. The secure-action prose says “changing/clearing transmog outfits”; the implemented appearance effect is an active-ID transition, not transmog rendering.

### Behavioral tests / pre-commit discrimination

`tests/outfit_action_command.rs:44,59,82,107,140,164,191` assert host snapshots/environment isolation, sparse-index transitions, no fabricated invalid selections, atomic secret rejection/preserved addon taint, true/false conditional branches, toggle/bang/clear, and vendor handler effects. They assert observable state and Lua results, not source shape.

Against parent producers: old bootstrap used the argument directly as an outfit ID (`a4cce2db1^`, `transmog_outfit_slot_defaults.rs:145–153`), treated any true second argument as unconditional clear, and read writable Lua `__activeOutfitID`. Parent macro dispatcher had no `/outfit` arm. Thus host snapshot, sparse-index, invalid/secret, successful macro and vendor transition assertions discriminate the old behavior. The false-condition no-error assertion alone does NOT discriminate: ignoring every command would pass it. Taken with matching-condition changes, that test is meaningful. Full new tests need the new state field present to compile; do not describe a literal parent checkout execution as independently demonstrated here.

Vendor test loads the entire cached `SecureTemplates.lua`, creates a real Button and directly calls real `SecureActionButton_OnClick` (`tests/outfit_action_command.rs:191–230`). This IS real vendor action-handler dispatch, including default toggle; it is NOT simulator `Button:Click()`, physical input, secure-template XML inheritance or protected-authority proof. Spec explicitly acknowledges that distinction (`docs/specs/outfit-action-command.md:35`).

### Viewed/pending changes and fixture legitimacy

Cached `Blizzard_Transmog/Blizzard_Transmog.lua:514–527` explicitly separates active (worn in world) from viewed (transmog frame), then calls `ChangeViewedOutfit` to select the view. `Blizzard_TransmogTemplates.lua:65–78` applies appearances through `ChangeDisplayedOutfit`; `:140–159` selects view through `ChangeViewedOutfit`. Secure and slash handlers call only `ChangeToOutfit`/`ClearOutfit` (`SecureTemplates.lua:653–665`; `SlashCommands.lua:1708–1726`). No inspected consumer requires those calls to set viewed state or discard pending edits.

Separation is supported; exact pending preservation is an inferred simulator policy, NOT stated by source prose or proven natively. Removing the old fake combined reset is not an established consumer regression. Missing modeled viewed/displayed mutations and refresh events remain gaps; passing these fixtures does not prove the wardrobe UI works.

Rewritten `tests/transmog_outfit_info.rs:28–62` legitimately switches ID arguments to catalog indices and checks retained viewed/pending fixtures across toggle/clear. `tests/pending_transmog_cost.rs:104–134` explicitly seeds viewed compatibility metadata and preserves its original query-read-only purpose. Seeding a fixture is legitimate; it does not earn state-backed viewed-outfit capability credit. Documentation defect: their comments saying “Viewed metadata has no setter API” (`transmog_outfit_info.rs:33`, `pending_transmog_cost.rs:108`) are literally false: cached `TransmogOutfitInfoDocumentation.lua:62–70` declares `ChangeViewedOutfit`. The missing piece is its modeled setter, not the API.

### Epoch split

Correct feature split: actions/tests require `retail-12-0-5`; legacy chunk and historical combined-lifecycle unit test require its absence (`transmog_outfit_slot_defaults.rs:157,208,262`). `Cargo.toml:119–122,148–155`: `client-retail` includes 12.1.0 → 12.0.7 → 12.0.5; `client-ptr` includes 12.1.5 → same chain. Historical `profile-retail + retail-12-0-5` uses new behavior; `profile-retail + retail-12-0-0` uses legacy. Wrath/Mists/Era/Anniversary/Forever bundles do not enable the gate and retain legacy; explicitly enabling the epoch feature selects new code. No new-epoch fallback. Legacy runtime execution was not independently verified.

### Earned spec checkboxes / gaps

- `docs/specs/outfit-action-command.md:7,9,11`: bounded policies covered by concrete tests, including viewed/pending controls in rewritten fixtures; eligible for checked status when caller's GREEN is accepted.
- `:8`: transitions, sparse-index-vs-ID/vector-position and idempotent clear earned; BOTH mutation return arities are not fully asserted. ChangeToOutfit has a zero-return assertion (`tests/outfit_action_command.rs:89`); ClearOutfit zero-return behavior is source-inspected only (`actions.rs:65–67`). Do not fully check the compound requirement without that distinction.
- `:10`: authentication, strict types, missing-index policy earned; duplicate-index first-match policy has no behavioral fixture. Keep compound checkbox open or split it.
- `:33`: targeted RED not independently verified; caller GREEN is not RED proof. Native/full-click/grammar gaps `:34–36` remain open.

Merge risk: no demonstrated blocking runtime defect in bounded selection/command scope. Main risk is overstating complete secure action, native wardrobe lifecycle or historical-profile proof. Conditional-only empty `/outfit [exists]` cannot clear (`actions.rs:78–84`; selector `cmd_option.rs:90–95`), an acknowledged bounded limitation, not a newly proven regression.

## 2. Cooldown countdown formatter — ACCEPT WITH QUALIFICATIONS

Recommend **bounded** for `prose-2026-03-25-115` / `prose-2026-03-31-161` (source lines 115,161), credited once through `cooldown-countdown-formatter`. Both sentences announce “numeric formatter support for Cooldown frame fontstrings”; NumericFormatter is not synonymous with the later NumericRuleFormatter subtype.

### Exact proposed capability scope

> Retail 12.0.5+ Cooldown frames retain authenticated, live abbreviated-number and native Seconds NumericFormatter attachments through SetCountdownFormatter/GetCountdownFormatter; real engine ticks consume those objects into the library countdown text path, observing configuration changes, GC lifetime, nil clearing/default restoration, countdown suppression gates and tested secret-input/taint preservation. NumericRuleFormatter attachment/output is additionally supported only when numeric-rule-formatters is enabled. Coverage excludes Lua child FontString GetText consistency, GPU glyph/pixel output, native-client parity, arbitrary Lua formatter objects, complete failure/reentrancy coverage and all-profile parity.

### Behavioral proof and boundaries

Public tests (`tests/cooldown_countdown_formatter.rs:18–195`) assert actual identity, live configuration, per-frame isolation, nil-result arity, replacement/clear preservation, atomic invalid/impostor rejection and authenticated-secret/caller-taint behavior. These alone prove configuration, not rendering.

Consumer tests call real `env.fire_on_update` after setting the simulator clock (`countdown_formatter_tests.rs:17–29`), then the same `cooldown_countdown_text` used by glyph emission (`quad_builders_cooldown.rs:141,174–176`). The eight configured cases at `:186,231,255,281,298,320,345,365` cover collecting peer replacement, numeric-rule modRate/live rules, abbreviation changes, Seconds ticking/unit changes, clear/default restoration, hide/minimum/expiry, caller-release GC, and secret timing with unused public impostor callbacks. They assert real produced strings and side effects, not hardcoded producer substitutes. They stop before glyph emission: no glyph geometry/rasterization assertion.

Counts are feature-dependent: eight public + eight configured consumer cases on the current numeric-rule-enabled profile; **seven + seven** without `numeric-rule-formatters`. Numeric-rule public test is gated at `tests/cooldown_countdown_formatter.rs:50`; consumer at `countdown_formatter_tests.rs:229`. Strict 12.0.5 does not automatically enable that later feature (`Cargo.toml:119–122`). In these named eight consumer cases, explicit modRate=2 proof is the numeric-rule case; do not claim an executed strict-12.0.5 modRate fixture from it.

All formatter producers/tests/spec are unchanged between B98 parent and B98 (inspected `git diff a4cce2db1^ a4cce2db1 -- <formatter paths>`: empty). Therefore these tests are **not expected to fail against pre-B98 producers**. This is legitimate existing-capability accounting, not a newly implemented B98 fix or new B98 RED/GREEN. The spec and handoff describe earlier configuration/rendering RED and historical GREEN; I did not inspect their underlying execution logs here. Handoff reports source-scoped batch36 renderer proof and warns that artifact provenance/Cargo.toml do not establish a current complete build. Caller 404/404 does not independently establish current library-renderer execution.

### Earned spec checkboxes / gaps

- `docs/specs/cooldown-countdown-formatter.md:9–13`: configuration policies have behavioral fixtures. `:10` numeric-rule portion is conditional, not strict-12.0.5 credit.
- `:17–19`: library text, live ticking/configuration, clear/default and suppression behavior covered. Numeric-rule text and the named explicit modRate test depend on its feature. No GPU proof.
- `:21`: collecting peer-replacement behavior directly asserted at `countdown_formatter_tests.rs:186–227`, including replacement output and empty Lua error list.
- `:20`: only partly behavior-proven. GC survival/peer replacement, trusted dispatch bypassing impostors and secret/callback taint are exercised. Frame-ID snapshots/live-root lookup are source-inspected implementation, not behavioral requirements by themselves. Stack-height restoration on error, corrupt/unavailable roots, active self-detachment and the full error-policy clauses have no dedicated executed behavioral proof in these cases. Do not check this entire compound item.
- Current-revision gate `:44` remains unverified here; historical failure entries `:45–46` are explicitly superseded historical evidence, not current defects. Child GetText gap `:47` remains real and is directly asserted nil in consumer test `:401`.

No new formatter regression demonstrated. Bounded existing credit is honest provided historical source-scoped evidence is not relabeled current-build execution and later NumericRuleFormatter is not backdated to strict 12.0.5. Risk is audit overcredit, especially confusing configuration/text selection with Lua FontString text or GPU output.

## Final disposition

**Artifact verification PASS for bounded implementation/wiring; both items ACCEPT WITH QUALIFICATIONS.** No blocking runtime defect established within those scopes. Documentation defect: “no setter API” fixture comments (`tests/transmog_outfit_info.rs:33`, `tests/pending_transmog_cost.rs:108`). Unproven compound spec clauses and epoch/execution boundaries above prevent blanket checkbox or full-row acceptance. No repository files changed; only this authorized report written.
