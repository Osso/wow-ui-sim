#[cfg(feature = "casc")]
use std::path::PathBuf;
#[cfg(feature = "casc")]
use std::sync::OnceLock;

#[cfg(all(test, feature = "casc"))]
#[path = "../tests/common/bounded_process.rs"]
mod bounded_process;

#[cfg(feature = "casc")]
static RESOLVER: OnceLock<asset_resolver::CascListfileResolver> = OnceLock::new();

#[cfg(feature = "casc")]
pub fn resolver() -> &'static asset_resolver::CascListfileResolver {
    configure_casc_product_env();
    RESOLVER.get_or_init(|| asset_resolver::CascListfileResolver::new(config()))
}

#[cfg(feature = "casc")]
pub fn configure_casc_product_env() {
    if std::env::var_os("WOW_PRODUCT").is_none() {
        unsafe {
            std::env::set_var("WOW_PRODUCT", active_profile_casc_product());
        }
    }
}

/// Resolve the install root from an explicitly supplied settings file and export
/// it as `WOW_INSTALL_PATH` before the resolver caches a path. `None` uses only
/// the environment and automatic discovery and never reads or writes settings;
/// entrypoints pass [`crate::install_paths::SettingsFile::platform_default`].
#[cfg(feature = "casc")]
pub fn configure_casc_install_env(settings: Option<&crate::install_paths::SettingsFile>) {
    configure_casc_product_env();
    if std::env::var("WOW_SIM_CASC").ok().as_deref() == Some("0") {
        return;
    }
    let product = active_profile_casc_product();
    let inputs = crate::install_paths::InstallPathInputs::from_process_env();
    let resolved = crate::install_paths::resolve_install_path(settings, product, &inputs, || {
        asset_resolver::wow_install_path().map(std::path::Path::to_path_buf)
    });
    let Some(resolved) = resolved else {
        return;
    };
    unsafe {
        std::env::set_var("WOW_INSTALL_PATH", &resolved.root);
    }
}

#[cfg(feature = "casc")]
pub fn active_profile_casc_product() -> &'static str {
    match crate::client_profile::ACTIVE {
        crate::client_profile::ClientProfile::Ptr => "wowxptr",
        crate::client_profile::ClientProfile::Wrath
        | crate::client_profile::ClientProfile::Mists => "wow_classic",
        crate::client_profile::ClientProfile::Era
        | crate::client_profile::ClientProfile::Anniversary => "wow_classic_era",
        crate::client_profile::ClientProfile::Retail => "wow",
        crate::client_profile::ClientProfile::WowForever => "wowf",
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
    configure_casc_product_env();
    let Some(install) = asset_resolver::wow_install_path() else {
        return Ok(());
    };
    asset_resolver::casc_resolver::open_resolution_cache_for_install(install).map_err(|error| {
        format!(
            "prepare GUI CASC resolution cache for {}: {error}",
            install.display()
        )
    })?;
    // The dependency has no catalog-preparation API. An empty-path lookup loads
    // the community catalog without extracting an asset; lookup errors are logged
    // by the resolver. Replace this query when an explicit preparation API exists.
    let _ = resolver().lookup_path("");
    Ok(())
}

#[cfg(all(test, feature = "casc"))]
mod tests {
    use super::bounded_process;
    use std::path::Path;
    use std::process::Command;

    #[test]
    fn gui_resolution_cache_preparation_ignores_ambient_install() {
        let ambient = tempfile::tempdir().unwrap();
        let cache = tempfile::tempdir().unwrap();
        write_unindexed_install(ambient.path());
        let output = bounded_process::output(
            90,
            Command::new(std::env::current_exe().unwrap())
                .args([
                    "--exact",
                    "asset_resolver_config::tests::gui_resolution_cache_preparation",
                    "--nocapture",
                ])
                .env_remove("WOW_SIM_GUI_CACHE_TEST_MODE")
                .env("WOW_INSTALL_PATH", ambient.path())
                .env("WOW_PRODUCT", "wow")
                .env("ASSET_RESOLVER_CACHE_DIR", cache.path())
                .env("WOW_SIM_CASC", "1"),
        )
        .unwrap();
        assert!(
            output.status.success(),
            "cache regression must use its own install, not ambient unindexed content:\n{}\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr),
        );
        assert!(
            !cache.path().join("casc").exists(),
            "the library regression must not prepare the ambient install's cache"
        );
    }

    fn write_unindexed_install(root: &Path) {
        std::fs::create_dir_all(root.join("Data/data")).unwrap();
        std::fs::write(
            root.join(".product.db"),
            include_bytes!("../tests/fixtures/casc-installed-products.db"),
        )
        .unwrap();
        std::fs::write(
            root.join(".build.info"),
            "Branch!STRING:0|Active!DEC:1|Build Key!HEX:16|Version!STRING:0|Product!STRING:0\n",
        )
        .unwrap();
        let config = root.join("Data/config/dc/fc/dcfc90fffd79ba00406ae46f5f657592");
        std::fs::create_dir_all(config.parent().unwrap()).unwrap();
        std::fs::write(
            config,
            "root = 11111111111111111111111111111111\nencoding = 22222222222222222222222222222222 6f1851167dd66bfc2e85297b68ef2afd\n",
        )
        .unwrap();
    }

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
        let output = bounded_process::output(
            90,
            Command::new(std::env::current_exe().unwrap())
                .args([
                    "--exact",
                    "asset_resolver_config::tests::gui_resolution_cache_preparation",
                    "--nocapture",
                ])
                .env("WOW_SIM_GUI_CACHE_TEST_MODE", mode)
                .env("WOW_INSTALL_PATH", install)
                .env("ASSET_RESOLVER_CACHE_DIR", cache)
                .env("WOW_SIM_CASC", if disabled { "0" } else { "1" }),
        )
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
    fn wowforever_profile_selects_wowf_product() {
        assert_eq!(super::active_profile_casc_product(), "wowf");
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
