//! Club membership state. IDs are opaque strings, never roster positions.
//! INFERRED: string token representation; cached docs name an opaque type but do
//! not prescribe its Lua representation. Numeric selectors are not retained.
use std::collections::BTreeMap;

use crate::lua_api::state::GuildMember;

pub const OWNER: u8 = 1;
pub const LEADER: u8 = 2;
pub const MODERATOR: u8 = 3;
pub const MEMBER: u8 = 4;
pub const GUILD_ID: &str = "guild-0";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Member {
    pub id: String,
    pub name: String,
    pub role: u8,
    pub presence: u8,
    pub is_self: bool,
    pub note: String,
    pub guild_rank: Option<i32>,
}

impl Member {
    pub fn new(id: &str, name: &str, role: u8, is_self: bool) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            role,
            presence: 1,
            is_self,
            note: String::new(),
            guild_rank: None,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Invitation {
    pub id: String,
    pub invitee: Member,
    pub inviter_id: String,
}

#[derive(Clone, Debug)]
pub struct Club {
    pub id: String,
    pub name: String,
    pub description: String,
    pub club_type: u8,
    pub avatar_id: u32,
    pub members: Vec<Member>,
    /// Server-supplied candidates; sending an invitation never invents membership.
    pub candidates: BTreeMap<String, Member>,
    pub invitations: Vec<Invitation>,
}

impl Club {
    pub fn self_member(&self) -> Option<&Member> {
        self.members.iter().find(|member| member.is_self)
    }

    pub fn member(&self, id: &str) -> Option<&Member> {
        self.members.iter().find(|member| member.id == id)
    }

    pub fn member_mut(&mut self, id: &str) -> Option<&mut Member> {
        self.members.iter_mut().find(|member| member.id == id)
    }

    // INFERRED role policy: owner manages all lower roles; leader manages moderators
    // and members; moderator manages members. Guild club management stays denied:
    // guild ranks are not community roles and must not confer invented privileges.
    pub fn management_role(&self) -> Option<u8> {
        if self.club_type == 2 {
            return None;
        }
        self.self_member().map(|member| member.role)
    }

    pub fn privilege(&self, field: &str) -> bool {
        let Some(role) = self.management_role() else {
            return false;
        };
        match field {
            "canSetOwnMemberNote"
            | "canSetOwnMemberAttribute"
            | "canSetOwnPresenceLevel"
            | "canSetOwnVoiceState"
            | "canUseVoice"
            | "canCreateMessage"
            | "canDestroyOwnMessage"
            | "canEditOwnMessage"
            | "canGetInvitation"
            | "canRevokeOwnInvitation" => true,
            "canSendInvitation"
            | "canSetOtherMemberNote"
            | "canSetOtherMemberAttribute"
            | "canRevokeOtherInvitation" => role <= MODERATOR,
            "canDestroy" | "canSetAttribute" | "canSetName" | "canSetDescription"
            | "canSetAvatar" | "canSetBroadcast" | "canSetPrivacyLevel" => role == OWNER,
            _ => false,
        }
    }

    pub fn kickable_roles(&self) -> Vec<u8> {
        self.management_role()
            .map(|role| ((role + 1)..=MEMBER).collect())
            .unwrap_or_default()
    }

    pub fn assignable_roles(&self, id: &str) -> Vec<u8> {
        let Some(actor) = self.management_role() else {
            return Vec::new();
        };
        let Some(target) = self.member(id) else {
            return Vec::new();
        };
        if actor == OWNER && !target.is_self {
            return (OWNER..=MEMBER).collect();
        }
        if target.role <= actor || target.is_self {
            return Vec::new();
        }
        ((actor + 1)..=MEMBER).collect()
    }
}

#[derive(Clone, Debug)]
pub struct ClubState {
    pub clubs: BTreeMap<String, Club>,
    /// INFERRED host inputs for RequiresClubsInitialized / HasRestrictions.
    /// Restrictions deny mutations without events; queries remain available.
    pub initialized: bool,
    pub restriction_reason: u8,
    next_id: u64,
    pub(crate) guild_message_authors: BTreeMap<String, Member>,
    pub(crate) guild_seed_author_ids: Vec<String>,
}

impl Default for ClubState {
    fn default() -> Self {
        Self {
            clubs: BTreeMap::new(),
            initialized: true,
            restriction_reason: 0,
            next_id: 0,
            guild_message_authors: BTreeMap::new(),
            guild_seed_author_ids: Vec::new(),
        }
    }
}

impl ClubState {
    pub fn allocate_id(&mut self, kind: &str) -> String {
        self.next_id += 1;
        format!("{kind}-{:016x}", self.next_id)
    }

    pub fn can_mutate(&self) -> bool {
        self.initialized && self.restriction_reason == 0
    }

    /// Preserve the legacy guild roster producer without exposing its indices.
    /// INFERRED: full roster name identifies a guild character because GuildMember
    /// has no GUID. Reordering preserves IDs; removal/rejoin allocates a new ID.
    pub fn sync_guild(&mut self, name: Option<&str>, roster: &[GuildMember]) {
        let Some(name) = name else {
            self.clubs.remove(GUILD_ID);
            return;
        };
        let old = self.clubs.remove(GUILD_ID);
        let is_initial = old.is_none();
        let mut by_name: BTreeMap<_, _> = old
            .into_iter()
            .flat_map(|club| club.members)
            .map(|member| (member.name.clone(), member))
            .collect();
        let members: Vec<Member> = roster
            .iter()
            .enumerate()
            .map(|(index, entry)| {
                let previous = by_name.remove(&entry.name);
                self.project_guild_member(entry, previous, is_initial && index == 0)
            })
            .collect();
        self.archive_guild_authors(&members);
        self.clubs.insert(
            GUILD_ID.into(),
            Club {
                id: GUILD_ID.into(),
                name: name.into(),
                description: String::new(),
                club_type: 2,
                avatar_id: 0,
                members,
                candidates: BTreeMap::new(),
                invitations: Vec::new(),
            },
        );
    }

    fn project_guild_member(
        &mut self,
        entry: &GuildMember,
        previous: Option<Member>,
        is_self: bool,
    ) -> Member {
        let mut member = previous.unwrap_or_else(|| {
            let id = self.allocate_id("member");
            Member::new(&id, &entry.name, MEMBER, is_self)
        });
        // Preserve explicit host Away/Mobile/Busy presence while the legacy
        // roster's online bit remains true.
        member.presence = if !entry.online && member.presence != 0 {
            3
        } else if entry.online && (member.presence == 3 || member.presence == 0) {
            1
        } else {
            member.presence
        };
        member.guild_rank = Some(entry.rank_index);
        member
    }

    fn archive_guild_authors(&mut self, members: &[Member]) {
        if self.guild_seed_author_ids.is_empty() {
            self.guild_seed_author_ids = members.iter().map(|member| member.id.clone()).collect();
        }
        for member in members {
            self.guild_message_authors
                .insert(member.id.clone(), member.clone());
        }
    }
}
