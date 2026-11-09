# PTR rewards shop predicates

The PTR profile publishes `C_StorePublic.IsRewardsShopEnabled()` and
`C_GameRules.IsRewardsShopEnabled()`. Both return one boolean.

The store predicate reads the supplied `rewardsShopEnabled` mirror, defaulting
false. The game-rules predicate additionally requires `bpayStoreEnable` or
`rewardsShopOverrideEnabled`, a nonzero supplied rule-set marker, and an integer
`RewardsShopFlags` rule with bit 1 set. Missing or non-integer rules fail the
check. Both other mirrors default false. These inputs model supplied state; they
do not implement server delivery.

Evidence: PTR 70668 reconstruction R11 in
`~/.local/state/wow-ui-sim/handoff/pr46-49-decomp-answers.md`. The override
helper's diagnostic is only reached when that helper is called; the game-rules
predicate short-circuits when `bpayStoreEnable` is enabled.
