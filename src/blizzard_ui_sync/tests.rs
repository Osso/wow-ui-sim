use super::{
    CacheProvenance, invalidate_cache_if_provenance_mismatched, manifest_entries,
    manifest_entry_fdid, manifest_entry_is_allowed_unmapped,
};
use std::cell::RefCell;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

#[cfg(feature = "casc")]
fn serve_cdn_responses(
    responses: Vec<Option<(&'static str, &'static [u8])>>,
) -> (String, std::thread::JoinHandle<usize>) {
    use std::io::{Read, Write};
    use std::time::{Duration, Instant};

    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let url = format!("http://{}/source", listener.local_addr().unwrap());
    let server = std::thread::spawn(move || {
        listener.set_nonblocking(true).unwrap();
        let mut requests = 0;
        let mut last_request = Instant::now();
        while last_request.elapsed() < Duration::from_secs(1) {
            match listener.accept() {
                Ok((mut socket, _)) => {
                    socket
                        .set_read_timeout(Some(Duration::from_secs(2)))
                        .unwrap();
                    let mut request = Vec::new();
                    let mut byte = [0];
                    while !request.ends_with(b"\r\n\r\n") {
                        socket.read_exact(&mut byte).unwrap();
                        request.push(byte[0]);
                    }
                    assert!(request.starts_with(b"GET /source HTTP/1.1\r\n"));
                    let response = responses.get(requests).expect("unexpected retry");
                    requests += 1;
                    last_request = Instant::now();
                    if let Some((status, body)) = response {
                        write!(
                            socket,
                            "HTTP/1.1 {status}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                            body.len()
                        )
                        .unwrap();
                        socket.write_all(body).unwrap();
                    }
                }
                Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                    std::thread::sleep(Duration::from_millis(10));
                }
                Err(error) => panic!("accept source request: {error}"),
            }
        }
        requests
    });
    (url, server)
}

#[cfg(feature = "casc")]
fn fetch_fixture_json(url: &str) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let client = reqwest::blocking::Client::builder()
        .no_proxy()
        .timeout(std::time::Duration::from_secs(2))
        .build()?;
    let response = client.get(url).send()?.error_for_status()?;
    Ok(serde_json::from_slice(&response.bytes()?)?)
}

#[test]
#[cfg(feature = "casc")]
fn cdn_transport_failure_retries_and_returns_verified_response() {
    let (url, server) = serve_cdn_responses(vec![None, Some(("200 OK", b"[79,75]"))]);
    let result = super::retry_cdn_fetch(
        || fetch_fixture_json(&url),
        |error: &Box<dyn std::error::Error>| {
            error
                .downcast_ref::<reqwest::Error>()
                .is_some_and(super::is_retryable_cdn_transport_error)
        },
    );
    assert_eq!(result.unwrap(), b"OK");
    assert_eq!(server.join().unwrap(), 2);
}

#[test]
#[cfg(feature = "casc")]
fn cdn_transport_failure_stops_after_three_attempts() {
    let (url, server) = serve_cdn_responses(vec![None, None, None]);
    let result = super::retry_cdn_fetch(
        || fetch_fixture_json(&url),
        |error: &Box<dyn std::error::Error>| {
            error
                .downcast_ref::<reqwest::Error>()
                .is_some_and(super::is_retryable_cdn_transport_error)
        },
    );
    assert!(result.is_err());
    assert_eq!(server.join().unwrap(), 3);
}

#[test]
#[cfg(feature = "casc")]
fn cdn_http_status_and_decode_failures_do_not_retry() {
    for response in [
        ("404 Not Found", b"missing".as_slice()),
        ("200 OK", b"invalid json".as_slice()),
    ] {
        let (url, server) = serve_cdn_responses(vec![Some(response)]);
        let result = super::retry_cdn_fetch(
            || fetch_fixture_json(&url),
            |error: &Box<dyn std::error::Error>| {
                error
                    .downcast_ref::<reqwest::Error>()
                    .is_some_and(super::is_retryable_cdn_transport_error)
            },
        );
        assert!(result.is_err());
        assert_eq!(server.join().unwrap(), 1);
    }
}

fn unique_temp_dir(label: &str) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time before unix epoch")
        .as_nanos();
    std::env::temp_dir().join(format!(
        "wow-ui-sim-blizzard-ui-sync-{label}-{}-{unique}",
        std::process::id()
    ))
}

#[test]
fn default_cache_addons_path_is_profile_scoped_addons_root() {
    let path = super::default_cache_addons_path().expect("cache path");

    assert!(
        path.ends_with(PathBuf::from(crate::client_profile::ACTIVE.cache_subdir()).join("AddOns")),
        "cache path should end with profile/AddOns, got {}",
        path.display()
    );
}

#[test]
#[cfg(feature = "casc")]
fn fdid_extraction_uses_local_casc_before_cdn() {
    let out_path = PathBuf::from("Interface/AddOns/Test.lua");
    let calls = RefCell::new(Vec::new());

    let extracted = super::extract_fdid_with_cdn_fallback(
        42,
        &out_path,
        |fdid, path| {
            calls
                .borrow_mut()
                .push(format!("local:{fdid}:{}", path.display()));
            Ok(true)
        },
        |fdid, path| {
            calls
                .borrow_mut()
                .push(format!("cdn:{fdid}:{}", path.display()));
            Ok(true)
        },
    )
    .expect("extract");

    assert!(extracted);
    assert_eq!(
        calls.into_inner(),
        vec!["local:42:Interface/AddOns/Test.lua"]
    );
}

#[test]
#[cfg(feature = "casc")]
fn fdid_extraction_uses_cdn_after_local_casc_miss() {
    let out_path = PathBuf::from("Interface/AddOns/Test.lua");
    let calls = RefCell::new(Vec::new());

    let extracted = super::extract_fdid_with_cdn_fallback(
        42,
        &out_path,
        |fdid, path| {
            calls
                .borrow_mut()
                .push(format!("local:{fdid}:{}", path.display()));
            Ok(false)
        },
        |fdid, path| {
            calls
                .borrow_mut()
                .push(format!("cdn:{fdid}:{}", path.display()));
            Ok(true)
        },
    )
    .expect("extract");

    assert!(extracted);
    assert_eq!(
        calls.into_inner(),
        vec![
            "local:42:Interface/AddOns/Test.lua",
            "cdn:42:Interface/AddOns/Test.lua"
        ]
    );
}

fn test_provenance(build_key: &str) -> CacheProvenance {
    CacheProvenance::new(
        crate::client_profile::ACTIVE.cache_subdir(),
        "wow",
        "12.1.0.69497",
        build_key,
        "install-key",
        "manifest-hash",
    )
}

#[cfg(feature = "casc")]
fn installed_product_fixture() -> tempfile::TempDir {
    let root = tempfile::tempdir().unwrap();
    std::fs::write(
        root.path().join(".product.db"),
        include_bytes!("../../tests/fixtures/casc-installed-products.db"),
    )
    .unwrap();
    std::fs::write(
        root.path().join(".build.info"),
        "Branch!STRING:0|Active!DEC:1|Build Key!HEX:16|Version!STRING:0|Product!STRING:0\nus|1|11111111111111111111111111111111|12.1.0.69933|wow\n",
    )
    .unwrap();
    root
}

#[test]
#[cfg(feature = "casc")]
fn build_identity_reads_forever_when_build_info_omits_the_product() {
    let root = installed_product_fixture();
    let metadata_before = std::fs::read(root.path().join(".build.info")).unwrap();
    let identity = super::read_active_build_identity(root.path(), "wow_classic_beta")
        .expect("installed Forever identity without a build-info row");

    assert_eq!(identity.version, "1.60.1.69977");
    assert_eq!(identity.build_key, "3bd89ce2721f7c75e7525dc83741076f");
    assert!(identity.install_key.is_empty());
    assert_eq!(
        std::fs::read(root.path().join(".build.info")).unwrap(),
        metadata_before,
    );
}

#[test]
#[cfg(feature = "casc")]
fn build_identity_uses_selected_product_not_stale_build_info() {
    let root = installed_product_fixture();
    let retail = super::read_active_build_identity(root.path(), "wow").unwrap();

    assert_eq!(retail.version, "12.1.0.69933");
    assert_eq!(retail.build_key, "dcfc90fffd79ba00406ae46f5f657592");
    assert_eq!(retail.install_key, "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa");
    assert!(super::read_active_build_identity(root.path(), "wowt").is_err());
}

#[test]
fn mismatched_provenance_removes_stale_profile_cache_before_sync() {
    let root = unique_temp_dir("mismatched-provenance");
    let stale_file = root.join("Blizzard_InspectUI/InspectPaperDollFrame.lua");
    std::fs::create_dir_all(stale_file.parent().expect("stale file parent"))
        .expect("create stale cache");
    std::fs::write(&stale_file, "legacy global").expect("write stale cache file");
    std::fs::write(
        root.join(super::PROVENANCE_FILE),
        test_provenance("old-build").contents(),
    )
    .expect("write stale provenance");

    let refreshed = invalidate_cache_if_provenance_mismatched(&root, &test_provenance("new-build"))
        .expect("invalidate stale cache");

    assert!(
        refreshed,
        "changed build identity must invalidate the cache"
    );
    assert!(
        !root.exists(),
        "invalidated cache must remove stale files before re-extraction"
    );
}

#[test]
fn legacy_provenance_removes_stale_profile_cache_before_sync() {
    let root = unique_temp_dir("legacy-provenance");
    let stale_file = root.join("Blizzard_TransmogShared/Blizzard_TransmogShared.lua");
    std::fs::create_dir_all(stale_file.parent().expect("stale file parent"))
        .expect("create stale cache");
    std::fs::write(&stale_file, "legacy global").expect("write stale cache file");
    std::fs::write(
        root.join(super::PROVENANCE_FILE),
        "profile=retail\nsource=casc-local-or-cdn\nfallback=none\n",
    )
    .expect("write legacy provenance");

    let refreshed = invalidate_cache_if_provenance_mismatched(&root, &test_provenance("build-key"))
        .expect("invalidate legacy cache");

    assert!(refreshed, "legacy provenance must invalidate the cache");
    assert!(
        !root.exists(),
        "legacy cache must remove stale files before re-extraction"
    );
}

#[test]
fn matching_provenance_preserves_existing_profile_cache() {
    let root = unique_temp_dir("matching-provenance");
    let existing_file = root.join("Blizzard_InspectUI/InspectPaperDollFrame.lua");
    std::fs::create_dir_all(existing_file.parent().expect("existing file parent"))
        .expect("create cache");
    std::fs::write(&existing_file, "current source").expect("write cache file");
    let expected = test_provenance("current-build");
    std::fs::write(root.join(super::PROVENANCE_FILE), expected.contents())
        .expect("write matching provenance");

    let refreshed = invalidate_cache_if_provenance_mismatched(&root, &expected)
        .expect("preserve matching cache");

    assert!(
        !refreshed,
        "matching cache identity must remain incremental"
    );
    assert_eq!(
        std::fs::read_to_string(existing_file).expect("read preserved cache file"),
        "current source"
    );
    std::fs::remove_dir_all(root).expect("remove cache root");
}

#[test]
fn complete_marker_writes_supplied_provenance_identity() {
    let root = unique_temp_dir("provenance");
    let expected = test_provenance("build-key");

    super::write_complete_marker(&root, &expected).expect("write complete marker");

    let provenance =
        std::fs::read_to_string(root.join(super::PROVENANCE_FILE)).expect("read provenance");
    assert_eq!(provenance, expected.contents());
    assert!(root.join(super::COMPLETE_MARKER).is_file());
    std::fs::remove_dir_all(root).expect("remove cache root");
}

#[test]
fn manifest_preserves_blizzard_addon_case() {
    let first = manifest_entries()
        .next()
        .expect("manifest should not be empty");
    assert!(first.starts_with("Blizzard_"));
}

#[test]
#[cfg(feature = "client-ptr")]
fn ptr_manifest_includes_ptr_only_aura_container() {
    let manifest: Vec<_> = manifest_entries().collect();

    assert!(manifest.contains(&"Blizzard_AuraContainer/Blizzard_AuraContainer.toc"));
}

#[test]
#[cfg(feature = "profile-retail")]
fn retail_manifest_includes_current_aura_container() {
    let manifest: Vec<_> = manifest_entries().collect();

    assert!(manifest.contains(&"Blizzard_AuraContainer/Blizzard_AuraContainer.toc"));
}

#[test]
#[cfg(feature = "profile-retail")]
fn retail_manifest_preserves_accessibility_family_sources_from_live_tree() {
    let manifest: Vec<_> = manifest_entries().collect();

    assert!(
        manifest.contains(&"Blizzard_AccessibilityTemplates/Classic/AccessibilityTemplates.lua")
    );
    assert!(
        manifest.contains(&"Blizzard_AccessibilityTemplates/Mainline/AccessibilityTemplates.lua")
    );
}

#[test]
#[cfg(feature = "client-ptr")]
fn ptr_aura_container_resolves_through_limited_listfile() {
    let entry = "Blizzard_AuraContainer/Blizzard_AuraContainer.toc";

    assert_eq!(manifest_entry_fdid(entry), Some(8154511));
    assert!(!manifest_entry_is_allowed_unmapped(entry));
}

#[test]
fn manifest_entries_resolve_through_limited_listfile() {
    let missing: Vec<_> = manifest_entries()
        .filter(|entry| manifest_entry_fdid(entry).is_none())
        .filter(|entry| !manifest_entry_is_allowed_unmapped(entry))
        .take(10)
        .collect();
    assert!(
        missing.is_empty(),
        "unmapped Blizzard UI files: {missing:?}"
    );
}

#[test]
#[cfg(feature = "client-ptr")]
fn ptr_sync_manifest_excludes_legacy_profile_entries() {
    let active: Vec<_> = super::sync_manifest_entries().collect();

    assert!(!active.contains(&"Blizzard_ActionBar/Classic/ActionButtonTemplate.xml"));
    assert!(!active.contains(&"Blizzard_UnitFrame/Mists/ShardBar.lua"));
    assert!(!active.contains(&"Blizzard_ChatFrame/Wrath/ChatConfigFrame.lua"));
}

#[test]
#[cfg(feature = "client-ptr")]
fn ptr_sync_manifest_excludes_removed_world_map_entries() {
    let active: Vec<_> = super::sync_manifest_entries().collect();

    assert!(!active.contains(&"Blizzard_WorldMap/Blizzard_WorldMapTooltip.xml"));
    assert!(!active.contains(&"Blizzard_WorldMap/WM_InvasionDataProvider.lua"));
    assert!(!active.contains(&"Blizzard_WorldMap/WM_InvasionDataProvider.xml"));
}

#[test]
#[cfg(feature = "client-mists")]
fn mists_required_cache_entries_are_in_manifest() {
    let manifest: std::collections::HashSet<_> = manifest_entries().collect();

    for entry in super::required_profile_cache_entries() {
        assert!(
            manifest.contains(entry),
            "Mists cache-required file must be synced by the Blizzard UI manifest: {entry}"
        );
    }
}
#[test]
#[cfg(feature = "client-mists")]
fn mists_cache_is_incomplete_when_required_profile_files_are_missing() {
    let root = unique_temp_dir("mists-required-files");
    std::fs::create_dir_all(&root).expect("create cache root");

    assert!(
        !super::cache_has_required_profile_files(&root),
        "Mists cache marker must not be trusted when profile-required TOC files are absent"
    );

    std::fs::remove_dir_all(root).expect("remove cache root");
}

#[test]
#[cfg(feature = "client-mists")]
fn mists_cache_rejects_old_classic_action_button_template() {
    let root = unique_temp_dir("mists-action-button-template");
    write_mists_required_cache_entries(&root);

    let action_button_template = root.join("Blizzard_ActionBar/Classic/ActionButtonTemplate.xml");
    std::fs::write(&action_button_template, "placeholder").expect("write placeholder");
    assert!(
        !super::cache_has_required_profile_files(&root),
        "Mists cache marker must not be trusted when ActionButtonTemplate.xml is the old Classic Era variant"
    );

    std::fs::write(
            action_button_template,
            r#"<CheckButton name="ActionBarButtonTemplate"><Cooldown parentKey="chargeCooldown"/></CheckButton>"#,
        )
        .expect("write Mists-compatible action button template");
    assert!(
        super::cache_has_required_profile_files(&root),
        "Mists cache should be complete when required files exist and ActionButtonTemplate.xml defines ActionBarButtonTemplate"
    );

    std::fs::remove_dir_all(root).expect("remove cache root");
}

#[test]
#[cfg(feature = "client-mists")]
fn mists_cache_rejects_mainline_nameplates_toc_without_game_type_gates() {
    let root = unique_temp_dir("mists-nameplates-toc");
    write_mists_required_cache_entries(&root);

    let nameplates_toc = root.join("Blizzard_NamePlates/Blizzard_NamePlates.toc");
    std::fs::write(&nameplates_toc, "Blizzard_ClassNameplateBar.lua\n")
        .expect("write ungated nameplates toc");
    assert!(
        !super::cache_has_required_profile_files(&root),
        "Mists cache marker must not be trusted when Blizzard_NamePlates.toc would load Mainline class bar files"
    );

    std::fs::write(
        nameplates_toc,
        "Mainline\\Blizzard_ClassNameplateBar.lua [AllowLoadGameType mainline]\n",
    )
    .expect("write Mists-compatible nameplates toc");
    assert!(
        super::cache_has_required_profile_files(&root),
        "Mists cache should be complete when Blizzard_NamePlates.toc preserves Mainline game-type gates"
    );

    std::fs::remove_dir_all(root).expect("remove cache root");
}

#[cfg(feature = "client-mists")]
include!("../blizzard_ui_sync_mists_test_fixture.rs");
