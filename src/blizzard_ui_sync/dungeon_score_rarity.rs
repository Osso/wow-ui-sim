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
pub fn populate(output: &Path) -> crate::Result<()> {
    let pin = crate::dungeon_score_rarity_pin::active().map_err(crate::Error::Other)?;
    let PartialDb2 { data, zero_filled } = fetch_cdn(&pin)?;
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
