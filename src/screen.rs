use clap::ValueEnum;

/// High-level UI surface to boot.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, ValueEnum)]
pub enum ScreenKind {
    /// Normal in-game UI.
    #[default]
    Game,
    /// Glue login screen.
    Login,
    /// Glue character selection screen.
    #[value(
        alias = "character-select",
        alias = "charselect",
        alias = "character_select"
    )]
    CharacterSelect,
    /// Glue character creation screen.
    #[value(
        alias = "character-create",
        alias = "charcreate",
        alias = "character_create"
    )]
    CharacterCreate,
}

impl ScreenKind {
    pub const fn is_glue(self) -> bool {
        matches!(
            self,
            Self::Login | Self::CharacterSelect | Self::CharacterCreate
        )
    }

    pub const fn glue_screen_name(self) -> Option<&'static str> {
        match self {
            Self::Game => None,
            Self::Login => Some("login"),
            Self::CharacterSelect => Some("charselect"),
            Self::CharacterCreate => Some("charcreate"),
        }
    }

    /// Return the values expected by `C_Login.GetState()`.
    pub const fn login_state(self) -> (i32, bool, i32, bool) {
        const LE_AURORA_STATE_NONE: i32 = 1;
        let connection_none = if matches!(
            crate::client_profile::ACTIVE,
            crate::client_profile::ClientProfile::Retail
                | crate::client_profile::ClientProfile::WowForever
        ) {
            1
        } else {
            0
        };
        match self {
            Self::Game => (LE_AURORA_STATE_NONE, true, connection_none, false),
            Self::Login => (LE_AURORA_STATE_NONE, false, connection_none, false),
            Self::CharacterSelect => (LE_AURORA_STATE_NONE, true, connection_none, false),
            Self::CharacterCreate => (LE_AURORA_STATE_NONE, true, connection_none, false),
        }
    }
}
