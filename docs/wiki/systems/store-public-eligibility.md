# StorePublic eligibility observations

The [supplied eligibility contract](../../specs/store-public-eligibility.md) separates StorePublic eligibility from the existing secure Store catalog. Group membership is not evidence of purchase eligibility. The prior adapter reduced `GetProducts(groupID)` to nonempty-table truth and treated an unavailable secure getter as false; neither equation follows from the pinned public declarations.

## Environment-local observations

`SimState.store_public_eligibility` is a map of integer group IDs to explicitly supplied boolean observations. A host supplies or updates it through the environment's existing state handle. New environments start without observations. Missing keys remain distinct from supplied false and raise the named `missing-input` diagnostic. That failure is simulator input policy, not a native default claim.

The query lives under `src/c_api/store_public_eligibility.rs`. Its ACTIVE gate publishes on Retail, PTR, Forever, Mists, Era and Anniversary, not Wrath. It reads no secure product lists, entry flags, ownership fields or CatalogShop DTOs. Store availability is a separate input; see [[store-api-availability]]. Visibility state remains separate in the workaround layer.

## Original consumer and test boundary

Each declaring profile selects `Blizzard_SharedXML/InterfaceUtil.lua` through its active SharedXML TOC. `StoreInterfaceUtil.OpenToSubscriptionProduct` first tests group 156 and only tests 37 when 156 is false. It selects subscription/game time and toggles the store for true observations. Both false lead to the original sound and URL index 22 branch. Shop2 must be disabled to reach these public queries; this prerequisite does not establish native shop policy.

`tests/store_public_eligibility.rs` loads the active original addon closure without overrides, checks TOC selection and helper source identity, and observes query lines with a debug hook. Clearly labeled test-only selection/toggle/sound/URL recording sinks observe outbound decisions. They do not replace the helper or query and do not prove a working storefront. In particular, classic inbound Lua lacks `SelectSubscriptionProduct`; that remains an end-to-end blocker, not a reason to invent a production endpoint.

Fixed secure lists contradict the supplied eligibility observations through subscription/game-time/web transitions; a second list arrangement preserves those outcomes. Separate direct assertions test arity and boolean shape, updates, missing-input recovery and two-environment isolation. Original helper conditions alone do not prove strict shape and shape assertions alone do not reject constant producers.

## Final proof (2026-10-26)

Master `17baa3e9a` was merged once into each sibling. GREEN `23eb582af` and producer-subtracted RED `ec6ab6a76` have byte-identical integration tests and inline default tests. The integration source SHA256 is `06cada4728844e5dfd92ed0c267b7e0479b8ee3215f50293b9ecfe89b3c67231`.

Each declared profile has two exact one-case lists/runs on each side. Original-helper RED reaches the selected InterfaceUtil query lines with no setup errors and rejects `gametime,toggle` for supplied `(true,false)` against fixed contradictory lists. Direct-input RED rejects the supplied-value mismatch. GREEN covers ten original-helper observations, subscription priority, missing second input, membership-only mutation, strict return shape, updates, missing-input recovery and two environments. The legacy group-501 assertion remains, with explicit supplied input rather than catalog inference. Retail/Wrath default-unit cases also pass.

| Profile | Original query lines | Helper/direct proof | Stock keys/occurrences, master → GREEN | All-target warnings |
|---|---|---|---|---|
| Retail | 44/48 | RED → GREEN | 8/10 → 8/10 | 0 |
| PTR | 44/48 | RED → GREEN | 134/154 → 134/154 | 0 |
| Forever | 44/48 | RED → GREEN | 26/26 → 26/26 | 0 |
| Mists | 32/36 | RED → GREEN | 6/12 → 6/12 | 0 |
| Era | 32/36 | RED → GREEN | 0/0 → 0/0 | 0 |
| Anniversary | 32/36 | RED → GREEN | 0/0 → 0/0 | 0 |
| Wrath | Not selected | Query absence passes | 0/0 → 0/0 | 0 |

All seven stock pairs use exact master `17baa3e9a` executables, compiled-revision/receipt/SHA256/argv bindings and equal current-CVar input hashes. Every raw key and count is unchanged; no new emitter or masking-predecessor chain was exposed. All 68 final runtime invocations preserve original CVar bytes and stat fields under O_NOATIME guards. Profile-cache provenance and the six declaration/consumer hashes are unchanged.

Each profile has a separate accepted canonical all-target check receipt with zero Cargo JSON and plain warnings. Formatting and manual changed-line Rust readability pass. Historical pre-integration runs are not substituted for this matrix. No native eligibility policy, coercion, secrecy parity, storefront or purchase completion is implied.

## Sources

- [Feature contract](../../specs/store-public-eligibility.md).
- [Existing secure Store investigation](../investigations/store-secure-pool-constructors.md) — distinct catalog/secure Store boundary.
- Profile-pinned `Blizzard_APIDocumentationGenerated/StorePublicUIDocumentation.lua`: Retail/PTR/Forever 11–24, Mists/Era/Anniversary 11–23. All declare Environment All at line 6; only Retail/PTR/Forever include AllowedWhenUntainted secret metadata at line 13.
- Profile-pinned `Blizzard_SharedXML/InterfaceUtil.lua`: Retail/PTR/Forever queries 44/48; Mists/Era/Anniversary 32/36.

## See Also

- [[store-api-availability]] — independent hidden server-mirrored store flags.
- [[store-secure-pool-constructors]] — secure catalog projection and downstream limitations.
- [[consumer-selection-and-red-proof]] — original selected consumer evidence and RED boundaries.
