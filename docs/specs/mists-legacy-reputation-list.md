# Mists legacy reputation list

The Mists Classic legacy reputation globals project the shared faction scenario rows into the original reputation-frame contract. VERIFIED native evidence from Mists 5.5.4.70631 covers visible counts, full-list lookup, tuple arities, invalid header operations and selected faction identity. The Mists-only publication lives in `src/c_api/mists_reputation_list.rs`; [the investigation](../wiki/investigations/mists-r21-classic-apis.md) records evidence limits and consumer selection.

## What it must do

- [ ] `GetNumFactions()` returns visible rows; collapsing a header removes its descendants from that count.
- [ ] `GetFactionInfo(index)` reads the full backing list, including rows above the visible count, and returns 16 values for a normal row, 14 for a header and 13 for a missing row.
- [ ] A missing row returns `nil, nil, 1, 0, 0, 0, nil, nil, nil, nil, nil, nil, nil`.
- [ ] `ExpandFactionHeader` and `CollapseFactionHeader` expand or collapse all headers when the index is invalid or does not select a header.
- [ ] Selection follows the selected faction ID when visible row indices change; no visible selection returns index 0.

## How it works

- [The Mists R21 native evidence and selected-consumer proof](../wiki/investigations/mists-r21-classic-apis.md)
- [Reputation fixture state](mists-reputation-fixture.md)

## Implementation inventory

- `src/c_api/mists_reputation_list.rs` — Mists legacy list globals over the shared scenario rows.
- `src/lua_api/workarounds/temporary/reputation_state.rs` — existing seeded `C_Reputation` scenario rows.
- `src/lua_api/env_init/mod.rs` — registers the Mists globals after profile bootstrap.

## Tests asserting this spec

- `tests/mists_r21_reputation.rs` — original Mists `ReputationFrame.lua`, visibility, backing lookup, tuple counts, fallback, selection and all-header behavior.

## Known gaps (current cycle)

- [ ] The focused Mists reputation tests must pass against the original selected consumer.

## Out of scope

- Native faction data and server-supplied selection defaults.
- The names and values of normal-row result positions 15–16. R21 proves their presence and count only; the simulator returns nil in these positions.
- Era and Anniversary behavior. Native evidence is UNKNOWN and this Mists implementation does not change those profiles.
