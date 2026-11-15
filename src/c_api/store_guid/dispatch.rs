//! Verified prefix dispatch only; field parsing and identity packing are separate.
pub const PREFIXES: [&str; 57] = [
    "Uniq",
    "Player",
    "Item",
    "WorldTransaction",
    "StaticDoor",
    "Transport",
    "Conversation",
    "Creature",
    "Vehicle",
    "Pet",
    "GameObject",
    "DynamicObject",
    "AreaTrigger",
    "Corpse",
    "LootObject",
    "SceneObject",
    "Scenario",
    "AIGroup",
    "DynamicDoor",
    "ClientActor",
    "Vignette",
    "CallForHelp",
    "AIResource",
    "AILock",
    "AILockTicket",
    "ChatChannel",
    "Party",
    "Guild",
    "WowAccount",
    "BNetAccount",
    "GMTask",
    "MobileSession",
    "RaidGroup",
    "Spell",
    "Mail",
    "WebObj",
    "LFGObject",
    "LFGList",
    "UserRouter",
    "PVPQueueGroup",
    "UserClient",
    "PetBattle",
    "UniqUserClient",
    "BattlePet",
    "CommerceObj",
    "ClientSession",
    "Cast",
    "ClientConnection",
    "ClubFinder",
    "ToolsClient",
    "WorldLayer",
    "ArenaTeam",
    "LMMParty",
    "LMMLobby",
    "Housing",
    "MeshObject",
    "Entity",
];

pub fn select_parser(bytes: &[u8]) -> (&'static str, &[u8]) {
    if let Some(separator) = bytes.iter().position(|byte| *byte == b'-') {
        if let Some(&name) = PREFIXES
            .iter()
            .find(|name| bytes[..separator].eq_ignore_ascii_case(name.as_bytes()))
        {
            return (name, &bytes[separator + 1..]);
        }
    }
    ("Uniq", bytes)
}

#[cfg(all(test, feature = "optional-tests"))]
mod tests {
    use super::*;

    #[test]
    fn all_fifty_seven_prefixes_use_their_complete_ascii_name() {
        for name in PREFIXES {
            let text = format!("{}-payload", name.to_ascii_lowercase());
            assert_eq!(
                select_parser(text.as_bytes()),
                (name, b"payload".as_slice())
            );
        }
        assert_eq!(
            select_parser(b"PLAYER-1-12"),
            ("Player", b"1-12".as_slice())
        );
        assert_eq!(
            select_parser(b"Creature-1-2-3-4-5-6"),
            ("Creature", b"1-2-3-4-5-6".as_slice())
        );
    }

    #[test]
    fn unknown_or_absent_prefix_keeps_original_for_uniq() {
        for text in [
            b"Unknown-123".as_slice(),
            b"Play-1-12",
            b"PlayerX-1-12",
            b"Player",
            b"",
        ] {
            assert_eq!(select_parser(text), ("Uniq", text));
        }
    }
}
