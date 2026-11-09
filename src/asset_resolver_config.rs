#[cfg(feature = "casc")]
use std::path::PathBuf;
#[cfg(feature = "casc")]
use std::sync::OnceLock;

#[cfg(all(test, feature = "casc"))]
#[path = "../tests/common/bounded_process.rs"]
mod bounded_process;

#[cfg(all(test, feature = "casc"))]
#[path = "../tests/common/gui_casc_cache.rs"]
mod gui_casc_cache;

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
    use super::{bounded_process, gui_casc_cache};
    use std::path::Path;
    use std::process::Command;

    const CACHE_TEST: &str = "asset_resolver_config::tests::gui_resolution_cache_preparation";

    #[test]
    fn gui_resolution_cache_preparation_ignores_ambient_install() {
        let ambient = tempfile::tempdir().unwrap();
        let cache = tempfile::tempdir().unwrap();
        let catalog = tempfile::tempdir().unwrap();
        write_empty_catalog(catalog.path());
        gui_casc_cache::write_install(
            ambient.path(),
            gui_casc_cache::Options {
                index_encoding_file: false,
                corrupt_encoding_file: false,
            },
        );
        let output = bounded_process::output(
            90,
            Command::new(std::env::current_exe().unwrap())
                .args(["--exact", CACHE_TEST, "--nocapture"])
                .env_remove("WOW_SIM_GUI_CACHE_TEST_MODE")
                .env("WOW_INSTALL_PATH", ambient.path())
                .env("WOW_PRODUCT", "wow")
                .env("ASSET_RESOLVER_CACHE_DIR", cache.path())
                .env("ASSET_RESOLVER_DATA_DIR", catalog.path())
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

    #[test]
    fn gui_resolution_cache_preparation() {
        if let Ok(mode) = std::env::var("WOW_SIM_GUI_CACHE_TEST_MODE") {
            run_cache_test_mode(&mode);
            return;
        }

        let catalog = tempfile::tempdir().unwrap();
        write_empty_catalog(catalog.path());
        let invalid_install = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(invalid_install.path().join("Data/data")).unwrap();
        run_gui_cache_test("disabled", invalid_install.path(), catalog.path(), true);
        run_gui_cache_test("failure", invalid_install.path(), catalog.path(), false);

        let valid_install = tempfile::tempdir().unwrap();
        gui_casc_cache::write_install(valid_install.path(), Default::default());
        run_gui_cache_test("cold", valid_install.path(), catalog.path(), false);

        let unindexed_install = tempfile::tempdir().unwrap();
        gui_casc_cache::write_install(
            unindexed_install.path(),
            gui_casc_cache::Options {
                index_encoding_file: false,
                corrupt_encoding_file: false,
            },
        );
        run_gui_cache_test("unindexed", unindexed_install.path(), catalog.path(), false);

        let corrupt_install = tempfile::tempdir().unwrap();
        gui_casc_cache::write_install(
            corrupt_install.path(),
            gui_casc_cache::Options {
                index_encoding_file: true,
                corrupt_encoding_file: true,
            },
        );
        run_gui_cache_test("corrupt", corrupt_install.path(), catalog.path(), false);
    }

    fn run_cache_test_mode(mode: &str) {
        match mode {
            "cold" => assert_cold_cache(),
            "disabled" => super::prepare_gui_casc_resolution_cache()
                .expect("disabled CASC must not open an invalid install"),
            "failure" => {
                let error = super::prepare_gui_casc_resolution_cache()
                    .expect_err("enabled CASC must report preparation failure");
                assert!(
                    error.contains("prepare GUI CASC resolution cache"),
                    "{error}"
                );
            }
            "unindexed" => {
                let error = super::prepare_gui_casc_resolution_cache()
                    .expect_err("missing encoding index entry must fail preparation");
                assert!(error.contains("read encoding file"), "{error}");
                assert!(error.contains("missing archive location"), "{error}");
            }
            "corrupt" => {
                let error = super::prepare_gui_casc_resolution_cache()
                    .expect_err("corrupt indexed encoding content must fail preparation");
                assert!(error.contains("parse encoding.bin"), "{error}");
                assert!(!error.contains("missing archive location"), "{error}");
            }
            _ => panic!("unknown cache test mode: {mode}"),
        }
    }

    fn assert_cold_cache() {
        super::configure_casc_product_env();
        let install = asset_resolver::wow_install_path().expect("synthetic test install");
        let cache_dir = asset_resolver::casc_resolver::casc_cache_dir_for_install(install)
            .expect("selected fixture build");
        assert!(
            !cache_dir.join("resolution.sqlite").exists(),
            "fixture starts without a resolution cache"
        );

        super::prepare_gui_casc_resolution_cache().expect("cold preparation");
        let cache_path = cache_dir.join("resolution.sqlite");
        let first_bytes = std::fs::read(&cache_path).expect("cold preparation writes cache");
        let first_modified = std::fs::metadata(&cache_path).unwrap().modified().unwrap();
        let cache = asset_resolver::casc_resolver::open_resolution_cache_for_install(install)
            .expect("prepared cache is usable");
        let expected_resolutions = gui_casc_cache::expected_resolutions();
        for (fdid, expected) in [
            (gui_casc_cache::FIRST_FDID, expected_resolutions[0]),
            (gui_casc_cache::SECOND_FDID, expected_resolutions[1]),
        ] {
            assert_eq!(
                cache.resolve_fdid(fdid),
                Some((expected.content_key, expected.encoding_key)),
                "prepared cache resolves FDID {fdid}"
            );
        }
        drop(cache);

        super::prepare_gui_casc_resolution_cache().expect("warm preparation");
        assert_eq!(
            std::fs::read(&cache_path).unwrap(),
            first_bytes,
            "warm preparation keeps cache bytes unchanged"
        );
        assert_eq!(
            std::fs::metadata(&cache_path).unwrap().modified().unwrap(),
            first_modified,
            "warm preparation reuses the cache"
        );
    }

    fn run_gui_cache_test(mode: &str, install: &Path, catalog: &Path, disabled: bool) {
        let cache = tempfile::tempdir().unwrap();
        let output = bounded_process::output(
            90,
            Command::new(std::env::current_exe().unwrap())
                .args(["--exact", CACHE_TEST, "--nocapture"])
                .env("WOW_SIM_GUI_CACHE_TEST_MODE", mode)
                .env("WOW_INSTALL_PATH", install)
                .env_remove("WOW_DATA_PATH")
                .env("WOW_PRODUCT", "wow")
                .env("ASSET_RESOLVER_CACHE_DIR", cache.path())
                .env("ASSET_RESOLVER_DATA_DIR", catalog)
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

    fn write_empty_catalog(data_dir: &Path) {
        std::fs::create_dir_all(data_dir).unwrap();
        std::fs::write(data_dir.join("community-listfile.csv"), b"").unwrap();
    }

    #[test]
    #[cfg(feature = "optional-tests")]
    fn gui_resolution_cache_preparation_uses_live_install() {
        assert_ne!(
            std::env::var("WOW_SIM_CASC").ok().as_deref(),
            Some("0"),
            "live CASC test requires CASC enabled"
        );
        let Some(install) = asset_resolver::wow_install_path() else {
            eprintln!("skipping live CASC smoke test: no WoW install discovered");
            return;
        };
        let product = std::env::var("WOW_PRODUCT")
            .unwrap_or_else(|_| super::active_profile_casc_product().to_owned());
        eprintln!(
            "live CASC cache smoke test: product={product} install={}",
            install.display()
        );
        super::prepare_gui_casc_resolution_cache()
            .expect("live install must provide readable GUI CASC bootstrap content");
        let cache_dir = asset_resolver::casc_resolver::casc_cache_dir_for_install(install)
            .expect("live install active build");
        assert!(cache_dir.join("resolution.sqlite").is_file());
        drop(
            asset_resolver::casc_resolver::open_resolution_cache_for_install(install)
                .expect("live resolution cache must be usable"),
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
