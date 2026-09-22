# XML child OnLoad local `self`

An XML child OnLoad assignment must resolve `self` and fields rooted at `self` against the executing child, not a same-named global. The fast-literal parser is in `src/lua_api/globals/create_frame/template_chain/parser.rs`.

## What it must do

- [x] Publishing bare `self` to a parent retains the child frame handle.
- [x] Publishing `self.Part` to a parent retains the child's field even if `_G.self.Part` differs.
- [x] A genuinely global dotted value remains available to an OnLoad parent assignment.

## How it works

- [XML template system](../xml-template-system.md)

## Implementation inventory

- `src/lua_api/globals/create_frame/template_chain/parser.rs` — accepts literal global paths only when they do not start at local `self`.

## Tests asserting this spec

- `src/loader/tests/runtime_template_parenting.rs` — child OnLoad bare-self, dotted-self, and dotted-global publication.

## Proof

`template-self-development-ledger.json` records `e11037217` passing all three `runtime_template_child_onload_` tests. `verify-context-readability-final-ledger.json` revalidates 3/3 after the `0bee9e939` readability refactor, plus formatting/default checking and the final BigWigs replay. The regression began in BigWigs AceGUI child OnLoad code: a fast-path assignment treated lexical `self` as `_G.self`, causing post-`DONE` OnUpdate errors. Rejecting bare and dotted `self` roots from fast global literals sends them through authoritative Lua evaluation; it is not a Blizzard/vendor patch.

## Known gaps (current cycle)

- [ ] Other Lua local names and expressions outside these cases are not covered by this bounded parser regression.

## Out of scope

No broader local-name analysis, XML handler redesign, or changes to Blizzard/vendor code.
