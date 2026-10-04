# 12.0.7 tooltip

Proposed bounded contract for `prose-undated-007`, `global api-GameTooltip_AddMoneyLine-058`. Source: [retained API excerpt](../../data/patch-api/sources/12.0.7-api-changes.txt). Later cached declarations may postdate 12.0.7; no native historical proof.

## What it must do

- [ ] Loaded helper appends coin-atlas lines for 123 copper, 20000 copper and zero after an existing label.
- [ ] Highlight/red selection and line clearing preserve exact text and colors; live mailbox values are consumed, never a fixed money prefix.

## How it works

- [Lua API architecture](../lua-api.md)

## Implementation inventory

- Cached Blizzard_GameTooltip/Mainline/GameTooltip.lua:315–323 — actual helper and MoneyFormatter delegate.
- src/lua_api/frame/methods/widgets/tooltip/ — line storage/public getters.

## Tests asserting this spec

`tests/p1207_b14_tooltip.rs` — One loaded-helper test is predicted PASS on current code, supplementing existing 12345/zero and real MailFrame consumer tests, not duplicating their fixture amounts. Constants/no-op/prefix-only formatting fail amount/text/NumLines assertions. With the loaded producer withheld the helper call errors. No state or producer edits needed. Existing tests whose expectations change: none.

Source-reading predictions only; no test execution or accepted coverage.

## Known gaps (current cycle)

- [ ] Integrate and run staged tests under strict retail-12-0-7.
- [ ] Known harness dependency Lua errors are not a clean full-addon proof. Generated helper cache may postdate 12.0.7; source does not state argument/security/native malformed-money policies, so none are invented.

## Out of scope

Native parity, automatic server synchronization, production datasets, vendor/cache edits and adjacent API rows.
