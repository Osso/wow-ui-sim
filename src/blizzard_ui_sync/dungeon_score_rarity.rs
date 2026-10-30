//! Generate the pinned Retail/PTR UIDungeonScoreRarity cache.
use super::pinned_download::{ContentEntry, PartialDb2};
use std::path::{Path, PathBuf};
pub fn cache_path() -> crate::Result<PathBuf> {
    let profile = crate::dungeon_score_rarity_pin::active()
        .map_err(crate::Error::Other)?
        .profile;
    dirs::cache_dir()
        .map(|p| {
            p.join("wow-ui-sim/dungeon-score-rarity")
                .join(profile)
                .join("rarity.json")
        })
        .ok_or_else(|| {
            crate::Error::Other("cannot determine dungeon score rarity cache directory".into())
        })
}
pub fn populate(output: &Path, source: super::forever_spells::Source) -> crate::Result<()> {
    let pin = crate::dungeon_score_rarity_pin::active().map_err(crate::Error::Other)?;
    let use_local = match source {
        super::forever_spells::Source::Local => true,
        super::forever_spells::Source::Cdn => false,
        super::forever_spells::Source::Auto => match asset_resolver::wow_install_path() {
            None => false,
            Some(path) => match cascette_client_storage::read_installed_product(path, &pin.product)
            {
                Ok(_) => true,
                Err(cascette_client_storage::ProductDbError::NotFound { .. }) => false,
                Err(error) => {
                    return Err(crate::Error::Other(format!(
                        "read installed product: {error}"
                    )));
                }
            },
        },
    };
    let PartialDb2 { data, zero_filled } = if use_local {
        fetch_local(&pin, output)?
    } else {
        fetch_cdn(&pin)?
    };
    let result =
        crate::dungeon_score_rarity_data::generate(&pin.profile, &pin.build, &data, zero_filled)
            .map_err(crate::Error::Other)?;
    let parent = output
        .parent()
        .ok_or_else(|| crate::Error::Other("cache output has no parent".into()))?;
    std::fs::create_dir_all(parent)
        .map_err(|e| crate::Error::Other(format!("create {}: {e}", parent.display())))?;
    std::fs::write(output, result)
        .map_err(|e| crate::Error::Other(format!("write {}: {e}", output.display())))?;
    Ok(())
}

fn fetch_cdn(pin: &crate::dungeon_score_rarity_pin::Pin) -> crate::Result<PartialDb2> {
    let client = reqwest::blocking::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(|e| crate::Error::Other(format!("create CDN client: {e}")))?;
    let roots = super::pinned_cdn::fetch_roots(&client, &pin.product)?;
    let entry = ContentEntry {
        path: "UIDungeonScoreRarity.db2".into(),
        content_key: pin.content_key.clone(),
        encoding_key: pin.encoding_key.clone(),
        content_size: pin.decoded_size,
        archive: pin.archive.clone(),
        offset: pin.offset,
        size: pin.size,
    };
    super::pinned_download::download_db2(&client, &entry, &roots)
}

fn fetch_local(
    pin: &crate::dungeon_score_rarity_pin::Pin,
    output: &Path,
) -> crate::Result<PartialDb2> {
    crate::asset_resolver_config::configure_casc_product_env();
    let install = asset_resolver::wow_install_path().ok_or(crate::Error::WowInstallNotFound)?;
    let cache = asset_resolver::casc_resolver::casc_cache_dir_for_install(install)
        .map_err(crate::Error::Other)?;
    if cache.file_name().and_then(|v| v.to_str()) != Some(pin.build.as_str()) {
        return Err(crate::Error::Other(format!(
            "installed {} build is not pinned build {}",
            pin.profile, pin.build
        )));
    }
    let parent = output
        .parent()
        .ok_or_else(|| crate::Error::Other("cache output has no parent".into()))?;
    std::fs::create_dir_all(parent)
        .map_err(|e| crate::Error::Other(format!("create {}: {e}", parent.display())))?;
    let path = parent.join(format!(".dungeon-score-rarity.{}.db2", std::process::id()));
    let extracted = asset_resolver::casc_resolver::extract_fdid_to_path(pin.fdid, &path)
        .map_err(crate::Error::Other)?;
    let data = std::fs::read(&path)
        .map_err(|e| crate::Error::Other(format!("read {}: {e}", path.display())))?;
    std::fs::remove_file(&path)
        .map_err(|e| crate::Error::Other(format!("remove {}: {e}", path.display())))?;
    if !extracted.missing_keys.is_empty()
        || data.len() != pin.decoded_size
        || cascette_crypto::ContentKey::from_data(&data).to_hex() != pin.content_key
    {
        return Err(crate::Error::Other(
            "local UIDungeonScoreRarity content identity mismatch or encrypted data".into(),
        ));
    }
    Ok(PartialDb2 {
        data,
        zero_filled: Vec::new(),
    })
}
