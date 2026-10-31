use sim_toc::{Selection, TocFile};
use std::path::{Path, PathBuf};

#[derive(Clone, Debug)]
struct Retail;
impl Selection for Retail {
    type Screen = ();
    fn game_types() -> &'static [&'static str] {
        &["mainline", "standard"]
    }
    fn recognizes_camelot() -> bool {
        false
    }
    fn family_subdir() -> &'static str {
        "Mainline"
    }
    fn game_subdir() -> &'static str {
        "Standard"
    }
    fn is_variable_metadata(_: &str) -> bool {
        false
    }
    fn is_mists_game_menu_shared_file(_: &Path, _: &str) -> bool {
        false
    }
    fn expands_regional_text_locale() -> bool {
        false
    }
    fn permits_ptr_only() -> bool {
        false
    }
    fn is_mists() -> bool {
        false
    }
    fn is_game(_: ()) -> bool {
        true
    }
    fn is_glue(_: ()) -> bool {
        false
    }
    fn find_case_insensitive(_: &Path, _: &str) -> Option<PathBuf> {
        None
    }
    fn is_builtin_addon_folder(_: &str) -> bool {
        false
    }
}

macro_rules! modern_profile {
    ($name:ident, $types:expr) => {
        #[derive(Clone, Debug)]
        struct $name;
        impl Selection for $name {
            type Screen = ();
            fn game_types() -> &'static [&'static str] {
                $types
            }
            fn recognizes_camelot() -> bool {
                true
            }
            fn family_subdir() -> &'static str {
                Retail::family_subdir()
            }
            fn game_subdir() -> &'static str {
                Retail::game_subdir()
            }
            fn is_variable_metadata(line: &str) -> bool {
                Retail::is_variable_metadata(line)
            }
            fn is_mists_game_menu_shared_file(dir: &Path, line: &str) -> bool {
                Retail::is_mists_game_menu_shared_file(dir, line)
            }
            fn expands_regional_text_locale() -> bool {
                false
            }
            fn permits_ptr_only() -> bool {
                false
            }
            fn is_mists() -> bool {
                false
            }
            fn is_game(_: ()) -> bool {
                true
            }
            fn is_glue(_: ()) -> bool {
                false
            }
            fn find_case_insensitive(_: &Path, _: &str) -> Option<PathBuf> {
                None
            }
            fn is_builtin_addon_folder(_: &str) -> bool {
                false
            }
        }
    };
}
modern_profile!(Ptr, &["mainline", "standard"]);
modern_profile!(Forever, &["mainline", "camelot"]);

#[test]
fn modern_alias_and_client_mask_contrasts() {
    for token in ["camelot", "FOREVER"] {
        let header = format!("## AllowLoadGameType: {token}\nCore.lua");
        let ptr = TocFile::<Ptr>::parse(Path::new("/addons/Test"), &header);
        let forever = TocFile::<Forever>::parse(Path::new("/addons/Test"), &header);
        let retail = TocFile::<Retail>::parse(Path::new("/addons/Test"), &header);
        assert!(ptr.is_game_type_restricted(), "{token}");
        assert!(!forever.is_game_type_restricted(), "{token}");
        assert!(!retail.is_game_type_restricted(), "{token}");
    }
}

#[test]
fn retail_header_unknown_tokens_do_not_restrict() {
    let toc = TocFile::<Retail>::parse(
        Path::new("/addons/ThirdParty"),
        "## AllowLoadGameType: camelot [AllowLoadGameType standard]\nCore.lua\n",
    );
    assert!(
        !toc.is_game_type_restricted(),
        "third-party addon must be admitted"
    );
    assert_eq!(
        toc.metadata.get("AllowLoadGameType").unwrap(),
        "camelot [AllowLoadGameType standard]"
    );
    let tab = TocFile::<Retail>::parse(
        Path::new("/addons/ThirdParty"),
        "## AllowLoadGameType: classic\t\nCore.lua\n",
    );
    assert!(
        !tab.is_game_type_restricted(),
        "tab is part of an unknown token"
    );
    for (header, restricted) in [
        ("## AllowLoadGameType: imaginary, wowlabs", false),
        ("## AllowLoadGameType: CLASSIC, MaInLiNe", false),
        ("## AllowLoadGameType: classic, plunderstorm", true),
        ("## ExcludeLoadGameType: unknown", false),
        ("## ExcludeLoadGameType: unknown, MAINLINE", true),
    ] {
        let toc = TocFile::<Retail>::parse(Path::new("/addons/ThirdParty"), header);
        assert_eq!(toc.is_game_type_restricted(), restricted, "{header}");
    }
    let toc = TocFile::<Retail>::parse(
        Path::new("/addons/ThirdParty"),
        "Unknown.lua [AllowLoadGameType missing]\nMatch.lua [AllowLoadGameType classic, MAINLINE]\nExcluded.lua [ExcludeLoadGameType mainline]\nOther.lua [AllowLoadGameType classic]\n",
    );
    assert_eq!(
        toc.files,
        vec![PathBuf::from("Unknown.lua"), PathBuf::from("Match.lua")]
    );
}
