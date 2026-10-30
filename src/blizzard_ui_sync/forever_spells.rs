//! Acquire the pinned Forever spell DB2 inputs.
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use super::pinned_download::{ContentEntry, PartialDb2};

const BUILD_KEY: &str = "002a8047a7c3a53de5e981936db76cce";

struct Table {
    name: &'static str,
    fdid: u32,
    content_key: &'static str,
    encoding_key: &'static str,
    archive: &'static str,
    offset: u64,
    size: usize,
    decoded_size: usize,
}
const TABLES: &[Table] = &[
    Table {
        name: "Spell",
        fdid: 1140089,
        content_key: "ec9b23e283da464c708b23585cf019ec",
        encoding_key: "93e5b3e3f3542532be35fc9a0a459177",
        archive: "25753f242ed34770801aa57fbe1f457a",
        offset: 176142966,
        size: 567344,
        decoded_size: 1893986,
    },
    Table {
        name: "SpellEffect",
        fdid: 1140088,
        content_key: "32bdee167179afba639a5864e78c192a",
        encoding_key: "8b7e473e2cb1679ae13815c4d60748ae",
        archive: "25753f242ed34770801aa57fbe1f457a",
        offset: 180670308,
        size: 474224,
        decoded_size: 1765528,
    },
    Table {
        name: "SpellMisc",
        fdid: 1003144,
        content_key: "135e7af97084693e545f4dea75b10ecf",
        encoding_key: "5757b113b91cad045efd96a020d4facb",
        archive: "25753f242ed34770801aa57fbe1f457a",
        offset: 179829861,
        size: 380023,
        decoded_size: 3174166,
    },
    Table {
        name: "SpellName",
        fdid: 1990283,
        content_key: "4f5bb666b6dc3aa475364b02e2634a8e",
        encoding_key: "ff7c89578ab0458d1ddf1de9dbb175d4",
        archive: "25753f242ed34770801aa57fbe1f457a",
        offset: 176714081,
        size: 223521,
        decoded_size: 636996,
    },
    Table {
        name: "SpellPower",
        fdid: 982806,
        content_key: "54d79f2fde5458a1eb93985b9f81c743",
        encoding_key: "62f50c78878cd625cc674ebf309de6da",
        archive: "214de82764691ed754b8957dcd5da10a",
        offset: 52880160,
        size: 30086,
        decoded_size: 56155,
    },
];

#[derive(Clone, Copy)]
pub enum Source {
    Auto,
    Local,
    Cdn,
}

pub fn cache_path() -> crate::Result<PathBuf> {
    dirs::cache_dir()
        .map(|p| p.join("wow-ui-sim/spells/wowforever/spells.json"))
        .ok_or_else(|| crate::Error::Other("cannot determine Forever spell cache directory".into()))
}

pub fn populate(source: Source, output: &Path) -> crate::Result<()> {
    if crate::client_profile::ACTIVE != crate::client_profile::ClientProfile::WowForever {
        return Err(crate::Error::Other(
            "Forever spell generation requires client-wowforever".into(),
        ));
    }
    let use_local = match source {
        Source::Auto => has_local_product()?,
        Source::Local => true,
        Source::Cdn => false,
    };
    let inputs = if use_local {
        fetch_local(output)?
    } else {
        fetch_cdn()?
    };
    let selected = crate::forever_spell_data::selected_ids().map_err(crate::Error::Other)?;
    let tables = inputs
        .into_iter()
        .map(|(name, input)| {
            crate::forever_db2::decode(&name, &input.data, input.zero_filled, &selected)
                .map(|decoded| (name, decoded))
                .map_err(crate::Error::Other)
        })
        .collect::<crate::Result<BTreeMap<_, _>>>()?;
    let bytes =
        crate::forever_spell_data::generate(&selected, tables).map_err(crate::Error::Other)?;
    let parent = output
        .parent()
        .ok_or_else(|| crate::Error::Other("Forever spell output has no parent".into()))?;
    std::fs::create_dir_all(parent)
        .map_err(|e| crate::Error::Other(format!("create {}: {e}", parent.display())))?;
    std::fs::write(output, &bytes)
        .map_err(|e| crate::Error::Other(format!("write {}: {e}", output.display())))?;
    Ok(())
}

fn has_local_product() -> crate::Result<bool> {
    let Some(install) = asset_resolver::wow_install_path() else {
        return Ok(false);
    };
    match cascette_client_storage::read_installed_product(install, "wowf") {
        Ok(_) => Ok(true),
        Err(cascette_client_storage::ProductDbError::NotFound { .. }) => Ok(false),
        Err(error) => Err(crate::Error::Other(format!(
            "read installed Forever product: {error}"
        ))),
    }
}

fn fetch_cdn() -> crate::Result<BTreeMap<String, PartialDb2>> {
    let client = reqwest::blocking::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(|e| crate::Error::Other(format!("create CDN client: {e}")))?;
    let roots = super::pinned_cdn::fetch_roots(&client, "wowf")?;
    TABLES
        .iter()
        .map(|t| {
            let entry = ContentEntry {
                path: format!("{}.db2", t.name),
                content_key: t.content_key.into(),
                encoding_key: t.encoding_key.into(),
                content_size: t.decoded_size,
                archive: t.archive.into(),
                offset: t.offset,
                size: t.size,
            };
            super::pinned_download::download_db2(&client, &entry, &roots)
                .map(|data| (t.name.to_owned(), data))
        })
        .collect()
}

fn fetch_local(output: &Path) -> crate::Result<BTreeMap<String, PartialDb2>> {
    crate::asset_resolver_config::configure_casc_product_env();
    if std::env::var("WOW_PRODUCT").ok().as_deref() != Some("wowf") {
        return Err(crate::Error::Other(
            "Forever spell extraction requires WOW_PRODUCT=wowf".into(),
        ));
    }
    let install = asset_resolver::wow_install_path().ok_or(crate::Error::WowInstallNotFound)?;
    let cache = asset_resolver::casc_resolver::casc_cache_dir_for_install(install)
        .map_err(crate::Error::Other)?;
    if cache.file_name().and_then(|v| v.to_str()) != Some(BUILD_KEY) {
        return Err(crate::Error::Other(format!(
            "installed Forever build is not pinned build {BUILD_KEY}"
        )));
    }
    let parent = output
        .parent()
        .ok_or_else(|| crate::Error::Other("Forever spell output has no parent".into()))?;
    std::fs::create_dir_all(parent)
        .map_err(|e| crate::Error::Other(format!("create {}: {e}", parent.display())))?;
    TABLES
        .iter()
        .map(|t| {
            let path = parent.join(format!(".{}.{}.db2", t.name, std::process::id()));
            let extracted = asset_resolver::casc_resolver::extract_fdid_to_path(t.fdid, &path)
                .map_err(crate::Error::Other)?;
            let data = std::fs::read(&path)
                .map_err(|e| crate::Error::Other(format!("read {}: {e}", path.display())))?;
            std::fs::remove_file(&path)
                .map_err(|e| crate::Error::Other(format!("remove {}: {e}", path.display())))?;
            if data.len() != t.decoded_size {
                return Err(crate::Error::Other(format!(
                    "{} decoded size mismatch",
                    t.name
                )));
            }
            Ok((
                t.name.to_owned(),
                PartialDb2 {
                    data,
                    zero_filled: extracted
                        .missing_keys
                        .into_iter()
                        .map(|m| m.offset..m.offset + m.size)
                        .collect(),
                },
            ))
        })
        .collect()
}
