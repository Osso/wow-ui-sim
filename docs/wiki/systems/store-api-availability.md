# Store API availability

The covered PTR/Retail store APIs read separate hidden server-mirrored values. Their contract is in the [store API availability spec](../../specs/store-api-availability.md).

## State and registration

`CVarStorage` keeps server mirrors in a separate per-environment map. Ordinary CVar value/default/key enumeration does not read this map. Retail and PTR start with `bpayStoreEnable=0` and `shop2Enabled=1`, from R38. The host setter changes either value without saving it to the user's CVar file.

`src/c_api/store_public_eligibility.rs` registers `C_StorePublic.IsEnabled()` for Retail/PTR. `src/c_api/c_catalog_shop.rs` registers `C_CatalogShop.IsShop2Enabled()` for the same profiles. Each query reads only its own value. The Shop 2 workaround preserves the registered Rust method; profiles outside R38 retain the pre-existing temporary compatibility default, without a native-parity claim.

## Update events

`WowLuaEnv::set_server_mirrored_cvar` fires `STORE_STATUS_CHANGED` with no arguments only when `bpayStoreEnable` changes. It does not fire `CVAR_UPDATE`. Changes to `shop2Enabled` fire no event on the initialized path. First-use behavior remains UNKNOWN and has no assertion.

On PTR, `C_GameRules.IsRewardsShopEnabled()` also reads the hidden `bpayStoreEnable` input. Its other gates remain separate.

## Evidence boundary

R38 verifies PTR 12.1.7.70668 and provides a limited Retail 12.1.5.70688 comparison. It records independent reads, defaults `false`/`true`, the hidden CVar flag, and the event outcomes above. The `C_CVar.GetCVar` refusal is verified, but a nil result is inferred; tests only check that the host values are not returned. PTR's shipped GameMenu uses an AND gate, so the default-hidden result is inferred. Retail's shipped GameMenu uses an OR gate, so the same defaults infer a visible button. 70493, real-account values, and the shop2 first-use path remain UNKNOWN.

## Sources

- [Behavioral contract](../../specs/store-api-availability.md).
- [R38 implementation report](../../handoff/ptr-int-clean/REPORT.md#store-api-behavior--r38-evidence-and-implementation-verification) — source evidence and targeted test results.
- R38 source evidence: `~/.local/state/wow-ui-sim/handoff/pr46-49-decomp-answers.md#r38`.
- PTR `Blizzard_GameMenu/Shared/GameMenuFrame.lua:198-204` uses an AND gate.
- Retail `Blizzard_GameMenu/Shared/GameMenuFrame.lua:148-154` uses an OR gate.

## See Also

- [[store-public-eligibility]] — separate per-group purchase eligibility input.
- [[store-secure-pool-constructors]] — secure catalog boundary.
