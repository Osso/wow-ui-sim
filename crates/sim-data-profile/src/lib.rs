//! Profile-selected generated catalogs. Selection is forwarded by wow-ui-sim.

#[cfg_attr(feature = "client-wowforever", path = "../data/atlas_wowforever.rs")]
#[cfg_attr(not(feature = "client-wowforever"), path = "../data/atlas.rs")]
pub mod atlas_data;
#[cfg_attr(
    feature = "client-wowforever",
    path = "../data/atlas_elements_wowforever.rs"
)]
#[cfg_attr(not(feature = "client-wowforever"), path = "../data/atlas_elements.rs")]
pub mod atlas_elements;
#[cfg_attr(
    feature = "client-wowforever",
    path = "../data/global_strings_wowforever.rs"
)]
#[cfg_attr(feature = "client-mists", path = "../data/global_strings_mists.rs")]
#[cfg_attr(feature = "client-ptr", path = "../data/global_strings_ptr.rs")]
#[cfg_attr(
    all(feature = "retail-12-1-5", not(feature = "client-ptr")),
    path = "../data/global_strings_retail.rs"
)]
#[cfg_attr(
    any(feature = "client-era", feature = "client-anniversary"),
    path = "../data/global_strings_classic.rs"
)]
#[cfg_attr(
    not(any(
        feature = "client-wowforever",
        feature = "client-mists",
        feature = "retail-12-1-5",
        feature = "client-ptr",
        feature = "client-era",
        feature = "client-anniversary"
    )),
    path = "../data/global_strings.rs"
)]
pub mod global_strings;

#[cfg(feature = "aura-containers")]
#[path = "../data/spell_aura_secrecy.rs"]
pub mod spell_aura_secrecy;
