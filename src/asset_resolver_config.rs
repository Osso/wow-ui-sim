#[cfg(feature = "casc")]
use std::path::PathBuf;
#[cfg(feature = "casc")]
use std::sync::OnceLock;

#[cfg(feature = "casc")]
static RESOLVER: OnceLock<asset_resolver::CascListfileResolver> = OnceLock::new();

#[cfg(feature = "casc")]
pub fn resolver() -> &'static asset_resolver::CascListfileResolver {
    configure_casc_product_env();
    RESOLVER.get_or_init(|| asset_resolver::CascListfileResolver::new(config()))
}

#[cfg(feature = "casc")]
pub(crate) fn configure_casc_product_env() {
    if std::env::var_os("WOW_PRODUCT").is_some() {
        return;
    }

    unsafe {
        std::env::set_var("WOW_PRODUCT", active_profile_casc_product());
    }
}

#[cfg(feature = "casc")]
pub(crate) fn active_profile_casc_product() -> &'static str {
    match crate::client_profile::ACTIVE {
        crate::client_profile::ClientProfile::Ptr => "wowxptr",
        crate::client_profile::ClientProfile::Wrath
        | crate::client_profile::ClientProfile::Mists => "wow_classic",
        crate::client_profile::ClientProfile::Era
        | crate::client_profile::ClientProfile::Anniversary => "wow_classic_era",
        crate::client_profile::ClientProfile::Retail => "wow",
        crate::client_profile::ClientProfile::WowForever => "wow_classic_beta",
    }
}

#[cfg(feature = "casc")]
fn config() -> asset_resolver::AssetResolverConfig {
    let cache_root = cache_root();
    asset_resolver::AssetResolverConfig::new()
        .with_cache_root(&cache_root)
        .with_shared_data_root(cache_root.join("data"))
}

#[cfg(feature = "casc")]
fn cache_root() -> PathBuf {
    std::env::var_os("ASSET_RESOLVER_CACHE_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(default_cache_root)
}

#[cfg(feature = "casc")]
fn default_cache_root() -> PathBuf {
    dirs::cache_dir()
        .unwrap_or_else(std::env::temp_dir)
        .join("asset-resolver")
}

#[cfg(feature = "casc")]
pub fn prepare_gui_casc_resolution_cache() -> Result<(), String> {
    if std::env::var("WOW_SIM_CASC").ok().as_deref() == Some("0") {
        return Ok(());
    }
    let Some(install) = asset_resolver::wow_install_path() else {
        return Ok(());
    };

    configure_casc_product_env();
    asset_resolver::casc_resolver::open_resolution_cache_for_install(install)
        .map(|_| ())
        .map_err(|error| {
            format!(
                "prepare GUI CASC resolution cache for {}: {error}",
                install.display()
            )
        })
}

#[cfg(all(test, feature = "casc"))]
mod tests {
    use std::path::Path;
    use std::process::Command;

    #[test]
    fn gui_resolution_cache_preparation() {
        if let Ok(mode) = std::env::var("WOW_SIM_GUI_CACHE_TEST_MODE") {
            match mode.as_str() {
                "cold" => {
                    super::configure_casc_product_env();
                    let install = asset_resolver::wow_install_path().expect("test install");
                    let cache_root = std::path::PathBuf::from(
                        std::env::var_os("ASSET_RESOLVER_CACHE_DIR").unwrap(),
                    );
                    assert!(!cache_root.join("casc").exists());
                    super::prepare_gui_casc_resolution_cache().expect("cold preparation");
                    let cache_dir =
                        asset_resolver::casc_resolver::casc_cache_dir_for_install(install)
                            .expect("active build");
                    assert!(
                        cache_dir.join("resolution.sqlite").exists(),
                        "preparation must build the resolution cache"
                    );
                    let cache =
                        asset_resolver::casc_resolver::open_resolution_cache_for_install(install)
                            .expect("prepared cache is usable");
                    drop(cache);
                    let first_modified = std::fs::metadata(cache_dir.join("resolution.sqlite"))
                        .unwrap()
                        .modified()
                        .unwrap();
                    super::prepare_gui_casc_resolution_cache().expect("warm preparation");
                    eprintln!("prepared CASC cache at {}", cache_dir.display());
                    assert_eq!(
                        std::fs::metadata(cache_dir.join("resolution.sqlite"))
                            .unwrap()
                            .modified()
                            .unwrap(),
                        first_modified,
                        "warm preparation must reuse the cache"
                    );
                }
                "disabled" => super::prepare_gui_casc_resolution_cache()
                    .expect("disabled CASC must not open an invalid install"),
                "failure" => {
                    let error = super::prepare_gui_casc_resolution_cache()
                        .expect_err("enabled CASC must report preparation failure");
                    assert!(error.contains("CASC"), "{error}");
                }
                _ => panic!("unknown cache test mode: {mode}"),
            }
            return;
        }

        let fixture = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(fixture.path().join("Data/data")).unwrap();
        run_gui_cache_test("disabled", fixture.path(), fixture.path(), true);
        run_gui_cache_test("failure", fixture.path(), fixture.path(), false);

        if let Some(install) = asset_resolver::wow_install_path() {
            let cache = tempfile::tempdir().unwrap();
            run_gui_cache_test("cold", install, cache.path(), false);
        } else {
            eprintln!("skipping cold/warm CASC preparation: no WoW install discovered");
        }
    }

    fn run_gui_cache_test(mode: &str, install: &Path, cache: &Path, disabled: bool) {
        let output = Command::new("timeout")
            .arg("90")
            .arg(std::env::current_exe().unwrap())
            .args([
                "--exact",
                "asset_resolver_config::tests::gui_resolution_cache_preparation",
                "--nocapture",
            ])
            .env("WOW_SIM_GUI_CACHE_TEST_MODE", mode)
            .env("WOW_INSTALL_PATH", install)
            .env("ASSET_RESOLVER_CACHE_DIR", cache)
            .env("WOW_SIM_CASC", if disabled { "0" } else { "1" })
            .output()
            .unwrap();
        eprintln!("{mode} child: {}", String::from_utf8_lossy(&output.stderr));
        assert!(
            output.status.success(),
            "{mode} preparation failed:\n{}\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }

    #[test]
    fn resolver_can_be_created_without_game_engine_root() {
        let resolver = super::new_test_resolver();

        assert!(resolver.lookup_path("not/a/real/path.blp").is_none());
    }

    #[test]
    #[cfg(feature = "client-wowforever")]
    fn wowforever_profile_selects_classic_beta_product() {
        assert_eq!(super::active_profile_casc_product(), "wow_classic_beta");
    }

    #[test]
    #[cfg(feature = "client-ptr")]
    fn ptr_profile_selects_wowxptr_product() {
        assert_eq!(super::active_profile_casc_product(), "wowxptr");
    }
}

#[cfg(all(test, feature = "casc"))]
fn new_test_resolver() -> asset_resolver::CascListfileResolver {
    asset_resolver::CascListfileResolver::new(config())
}
