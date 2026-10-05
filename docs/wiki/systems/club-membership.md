# Club membership

Club state lives in `SimState.clubs`, implemented under `src/c_api/`. Lua receives opaque strings, not roster indices. Contract: [club membership](../../specs/club-membership.md).

## Identity and management

`ClubState` allocates member/club/invitation tokens. Guild roster projection matches full character names, preserving IDs under reordering; removal/rejoin allocates a fresh ID. This mapping is INFERRED because the existing GuildMember producer has no GUID. Guild ranks do not confer community privileges.

Community creation installs the player as owner. INFERRED policy: owner manages lower roles; leader manages moderators/members; moderator manages members. Owner is required/unique; transfer demotes old owner to leader atomically. Member notes distinguish own/other privileges. Invitations reference server-supplied candidates and never invent accepted membership. Restrictions and initialization are explicit host inputs; denied mutations are silent and have no side effects.

Successful changed notes emit MEMBER_UPDATED; role/kick operations emit documented role/removal payloads. Local invitation changes emit INVITATIONS_RECEIVED_FOR_CLUB. Local completion, change-only event policy and role privilege mapping are INFERRED, not native service claims.

## Sources

- [Contract](../../specs/club-membership.md).
- `~/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/ClubDocumentation.lua`: declarations, privileges, event payloads.
- Cached `Blizzard_Communities/CommunitiesMemberList.lua:632–743`: ID-keyed lookups, presence/role updates, invitation cancellation.

## See Also

- [[patch-12-0-7-api-audit]] — five opaque member event rows; ledger remains unchanged.
