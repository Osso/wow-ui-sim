use super::*;
use std::process::Command;

const ENTRY: &str = "Blizzard_Test/Example.lua";
const VERSION: &str = "1.60.1.70532";
const HELLO_MD5: &str = "5d41402abc4b2a76b9719d911017c592";

fn git(root: &Path, args: &[&str]) -> String {
    let output = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(args)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).unwrap().trim().to_owned()
}

fn fixture(version: &str, bytes: &[u8]) -> (tempfile::TempDir, GetheTree) {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("Interface/AddOns").join(ENTRY);
    std::fs::create_dir_all(file.parent().unwrap()).unwrap();
    std::fs::write(file, bytes).unwrap();
    std::fs::write(dir.path().join("version.txt"), format!("{version}\n")).unwrap();
    git(dir.path(), &["init", "--quiet"]);
    git(dir.path(), &["-c", "core.autocrlf=false", "add", "."]);
    git(
        dir.path(),
        &[
            "-c",
            "user.name=Fixture",
            "-c",
            "user.email=fixture@example.invalid",
            "commit",
            "--quiet",
            "-m",
            "fixture",
        ],
    );
    let revision = git(dir.path(), &["rev-parse", "HEAD"]);
    let tree = GetheTree {
        root: dir.path().to_owned(),
        revision,
        version: VERSION.to_owned(),
    };
    (dir, tree)
}

#[test]
#[cfg(feature = "client-wowforever")]
fn missing_pin_fetch_failure_preserves_existing_cache() {
    const CHILD: &str = "WOW_SIM_GETHE_PREFLIGHT_TEST_CHILD";
    if let Some(root) = std::env::var_os(CHILD) {
        let root = PathBuf::from(root);
        let pin = read_source_pin().unwrap();
        let error = super::super::sync_blizzard_ui_to(&root).unwrap_err();
        assert_eq!(
            std::fs::read(root.join(super::super::COMPLETE_MARKER)).unwrap(),
            b"ok\n"
        );
        assert_eq!(
            std::fs::read(root.join(super::super::PROVENANCE_FILE)).unwrap(),
            b"old provenance\n"
        );
        assert!(error.to_string().contains(&pin.gethe_revision), "{error}");
        return;
    }

    let temp = tempfile::tempdir().unwrap();
    let install = temp.path().join("install");
    std::fs::create_dir_all(install.join("Data/data")).unwrap();
    let mut product = include_bytes!("../../tests/fixtures/casc-installed-products.db").to_vec();
    product.splice(0..7, [0x0a, 0xaa, 0x01, 0x12, 0x04, b'w', b'o', b'w', b'f']);
    let old = b"12.1.0.69933";
    let offset = product
        .windows(old.len())
        .position(|window| window == old)
        .unwrap();
    product[offset..offset + old.len()].copy_from_slice(b"1.60.1.70707");
    std::fs::write(install.join(".product.db"), product).unwrap();
    std::fs::write(
        install.join(".build.info"),
        "Branch!STRING:0|Active!DEC:1|Build Key!HEX:16|Version!STRING:0|Product!STRING:0\n",
    )
    .unwrap();

    let cache = temp.path().join("cache");
    let clone = cache.join("wow-ui-sim/wow-ui-source-git/forever");
    std::fs::create_dir_all(&clone).unwrap();
    git(&clone, &["init", "--quiet"]);
    git(
        &clone,
        &["remote", "add", "origin", "/nonexistent/gethe-fixture"],
    );
    let addons = temp.path().join("AddOns");
    std::fs::create_dir_all(&addons).unwrap();
    std::fs::write(addons.join(super::super::COMPLETE_MARKER), b"ok\n").unwrap();
    std::fs::write(
        addons.join(super::super::PROVENANCE_FILE),
        b"old provenance\n",
    )
    .unwrap();
    let status = Command::new(std::env::current_exe().unwrap())
        .args(["--exact", "blizzard_ui_sync::gethe_missing::tests::missing_pin_fetch_failure_preserves_existing_cache", "--nocapture"])
        .env(CHILD, &addons)
        .env("XDG_CACHE_HOME", &cache)
        .env("WOW_INSTALL_PATH", &install)
        .status().unwrap();
    assert!(status.success(), "child sync test failed");
}

fn file(resident: bool) -> MappedFile {
    MappedFile {
        path: ENTRY.to_owned(),
        fdid: 8379893,
        content_key: HELLO_MD5.to_owned(),
        resident,
    }
}

#[test]
fn nonresident_matching_gethe_is_written_and_recorded() {
    let (_source, tree) = fixture(VERSION, b"hello");
    let output = tempfile::tempdir().unwrap();
    let destination = output.path().join(ENTRY);
    let revision = tree.revision.clone();
    let (result, record) = sync_mapped_file(
        true,
        || Ok(Some(file(false))),
        &destination,
        || unreachable!(),
        || Ok(tree),
    )
    .unwrap();
    assert!(matches!(result, super::super::EntrySyncResult::Extracted));
    assert_eq!(std::fs::read(&destination).unwrap(), b"hello");
    let record = record.unwrap();
    let mut provenance = super::super::CacheProvenance {
        contents: "schema=fixture\n".to_owned(),
    };
    append_substitution(&mut provenance, &record).unwrap();
    let line = provenance
        .contents()
        .lines()
        .nth(1)
        .unwrap()
        .strip_prefix("gethe_file=")
        .unwrap();
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(line).unwrap(),
        serde_json::json!({
            "path": ENTRY, "fdid": 8379893, "ckey": HELLO_MD5, "gethe_revision": revision, "source": "gethe"
        })
    );
}

#[test]
fn nonresident_mismatching_gethe_fails_without_writing() {
    let (_source, tree) = fixture(VERSION, b"wrong");
    let output = tempfile::tempdir().unwrap();
    let destination = output.path().join(ENTRY);
    let error = sync_mapped_file(
        true,
        || Ok(Some(file(false))),
        &destination,
        || unreachable!(),
        || Ok(tree),
    )
    .err()
    .unwrap();
    assert_eq!(
        error.to_string(),
        "Gethe MD5 mismatch for Blizzard_Test/Example.lua (FDID 8379893): CKey=5d41402abc4b2a76b9719d911017c592, Gethe=2bda2998d9b0ee197da142a0447f6725"
    );
    assert!(!destination.exists());
}

#[test]
fn matching_lf_gethe_bytes_remain_unchanged() {
    let bytes = b"first\nsecond\n";
    let (_source, tree) = fixture(VERSION, bytes);
    let mut mapped = file(false);
    mapped.content_key = ContentKey::from_data(bytes).to_hex();
    assert_eq!(read_verified_gethe(&tree, &mapped).unwrap(), bytes);
}

#[test]
fn load_locale_recovers_exact_crlf_bytes_identified_by_casc() {
    // Exact published 70707 Gethe text; the installed enUS CKey identifies CRLF.
    let bytes = b"-- This is a symbol available for people who need to know the locale (separate from GetLocale())\nLOCALE_enUS = true;\nUI_LOCALE = \"enUS\";";
    let expected = b"-- This is a symbol available for people who need to know the locale (separate from GetLocale())\r\nLOCALE_enUS = true;\r\nUI_LOCALE = \"enUS\";";
    let (_source, tree) = fixture(VERSION, bytes);
    let mut mapped = file(false);
    mapped.fdid = 5612369;
    mapped.content_key = "2e4d4433534c95ee111c0e1a88863c01".into();
    assert_eq!(
        ContentKey::from_data(bytes).to_hex(),
        "b7ebc7bc164f5fc042e6619f8413a44a"
    );
    let output = tempfile::tempdir().unwrap();
    let destination = output.path().join(ENTRY);
    let (result, record) = sync_mapped_file(
        true,
        || Ok(Some(mapped)),
        &destination,
        || unreachable!(),
        || Ok(tree),
    )
    .unwrap();
    assert!(matches!(result, EntrySyncResult::Extracted));
    assert_eq!(std::fs::read(destination).unwrap(), expected);
    assert_eq!(record.unwrap().ckey, "2e4d4433534c95ee111c0e1a88863c01");
}

#[test]
fn crlf_recovery_preserves_existing_crlf_and_standalone_cr() {
    let bytes = b"first\r\nsecond\nthird\rfourth\n";
    let expected = b"first\r\nsecond\r\nthird\rfourth\r\n";
    let (_source, tree) = fixture(VERSION, bytes);
    assert_eq!(
        read_pinned_blob(&tree, &format!("Interface/AddOns/{ENTRY}")).unwrap(),
        bytes
    );
    let mut mapped = file(false);
    mapped.content_key = ContentKey::from_data(expected).to_hex();
    assert_eq!(read_verified_gethe(&tree, &mapped).unwrap(), expected);
}

#[test]
fn crlf_recovery_rejects_wrong_hash_and_locale_without_writing() {
    for (bytes, content_key) in [
        (b"first\nsecond\n".as_slice(), HELLO_MD5),
        (
            b"LOCALE_ptPT = true;\nUI_LOCALE = \"ptPT\";".as_slice(),
            "2e4d4433534c95ee111c0e1a88863c01",
        ),
    ] {
        let (_source, tree) = fixture(VERSION, bytes);
        let mut mapped = file(false);
        mapped.content_key = content_key.into();
        let output = tempfile::tempdir().unwrap();
        let destination = output.path().join(ENTRY);
        let error = sync_mapped_file(
            true,
            || Ok(Some(mapped)),
            &destination,
            || unreachable!(),
            || Ok(tree),
        )
        .err()
        .unwrap();
        assert!(error.to_string().contains("Gethe MD5 mismatch"));
        assert!(!destination.exists());
    }
}

#[test]
fn resident_uses_casc_with_no_gethe_tree() {
    let output = tempfile::tempdir().unwrap();
    let destination = output.path().join(ENTRY);
    let missing_tree = output.path().join("absent-gethe");
    let (result, record) = sync_mapped_file(
        true,
        || Ok(Some(file(true))),
        &destination,
        || {
            super::super::write_extracted_casc_path(&destination, b"resident CASC bytes")?;
            Ok(super::super::EntrySyncResult::Extracted)
        },
        || {
            Ok(GetheTree {
                root: missing_tree,
                revision: "0".repeat(40),
                version: VERSION.to_owned(),
            })
        },
    )
    .unwrap();
    assert!(matches!(result, super::super::EntrySyncResult::Extracted));
    assert_eq!(std::fs::read(destination).unwrap(), b"resident CASC bytes");
    assert!(record.is_none());
}

#[test]
fn mismatched_gethe_version_fails_without_writing() {
    let (_source, tree) = fixture("1.60.1.70374", b"hello");
    let output = tempfile::tempdir().unwrap();
    let destination = output.path().join(ENTRY);
    let error = sync_mapped_file(
        true,
        || Ok(Some(file(false))),
        &destination,
        || unreachable!(),
        || Ok(tree),
    )
    .err()
    .unwrap();
    assert_eq!(
        error.to_string(),
        "Gethe version mismatch: expected 1.60.1.70532, found 1.60.1.70374"
    );
    assert!(!destination.exists());
}

#[test]
fn pinned_revision_ignores_modified_working_tree_bytes() {
    let (source, tree) = fixture(VERSION, b"hello");
    std::fs::write(
        source.path().join("Interface/AddOns").join(ENTRY),
        b"uncommitted",
    )
    .unwrap();
    std::fs::write(source.path().join("version.txt"), b"wrong version").unwrap();
    let output = tempfile::tempdir().unwrap();
    let destination = output.path().join(ENTRY);
    sync_mapped_file(
        true,
        || Ok(Some(file(false))),
        &destination,
        || unreachable!(),
        || Ok(tree),
    )
    .unwrap();
    assert_eq!(std::fs::read(destination).unwrap(), b"hello");
}

#[test]
fn substitution_provenance_preserves_identity_comparison() {
    let (_source, tree) = fixture(VERSION, b"hello");
    let output = tempfile::tempdir().unwrap();
    let destination = output.path().join(ENTRY);
    let (_, record) = sync_mapped_file(
        true,
        || Ok(Some(file(false))),
        &destination,
        || unreachable!(),
        || Ok(tree),
    )
    .unwrap();
    let base = "schema=1\ngethe_revision=pinned\n";
    let mut provenance = super::super::CacheProvenance {
        contents: base.to_owned(),
    };
    append_substitution(&mut provenance, &record.unwrap()).unwrap();
    assert!(provenance_matches(provenance.contents(), base));
    assert!(!provenance_matches(
        provenance.contents(),
        "schema=1\ngethe_revision=other\n"
    ));
    assert!(!provenance_matches(
        &format!("{base}unrecognized=bad\n"),
        base
    ));
}

fn installation_provenance(version: &str) -> CacheProvenance {
    CacheProvenance::new(
        "wowforever",
        "wowf",
        version,
        "7d8ae4cd9c1f639a3ea04fd47e21c746",
        "",
        "manifest-hash",
    )
}

#[test]
fn matching_installed_version_preserves_gethe_policy() {
    let pin = read_source_pin().unwrap();
    let mut provenance = installation_provenance(&pin.version);
    append_source_identity(&mut provenance, &pin.version, false).unwrap();
    assert!(gethe_fallback_enabled(provenance.contents()));
    assert!(
        provenance
            .contents()
            .contains("source=casc-local-or-gethe-md5\n")
    );
    assert!(!provenance.contents().contains(DISABLED_VERSION_MISMATCH));
    assert!(provenance.contents().ends_with(&format!(
        "gethe_version={}\ngethe_revision={}\n",
        pin.version, pin.gethe_revision
    )));
}

#[test]
fn different_installed_version_is_rejected_without_opt_out() {
    let pin = read_source_pin().unwrap();
    let installed = format!("{}.different", pin.version);
    let mut provenance = installation_provenance(&installed);
    let original = provenance.contents().to_owned();
    let error = append_source_identity(&mut provenance, &installed, false).unwrap_err();
    assert!(error.to_string().contains(&installed));
    assert!(error.to_string().contains(&pin.version));
    assert!(error.to_string().contains("--allow-ui-version-mismatch"));
    assert_eq!(provenance.contents(), original);
}

#[test]
fn different_installed_version_uses_casc_without_gethe_lookup() {
    let installed = "1.60.1.70634";
    let mut provenance = installation_provenance(installed);
    append_source_identity(&mut provenance, installed, true).unwrap();
    assert!(
        provenance
            .contents()
            .contains(&format!("version={installed}\n"))
    );
    assert!(
        provenance
            .contents()
            .contains("source=casc-local-or-cdn\nfallback=none\n")
    );
    assert!(provenance.contents().contains(DISABLED_VERSION_MISMATCH));
    let output = tempfile::tempdir().unwrap();
    let destination = output.path().join(ENTRY);
    let (result, substitution) = sync_mapped_file(
        gethe_fallback_enabled(provenance.contents()),
        || panic!("disabled Gethe policy must not inspect substitution mappings"),
        &destination,
        || {
            super::super::write_extracted_casc_path(&destination, b"selected CASC bytes")?;
            Ok(EntrySyncResult::Extracted)
        },
        || panic!("disabled Gethe policy must not load a Gethe tree"),
    )
    .unwrap();
    assert!(matches!(result, EntrySyncResult::Extracted));
    assert!(substitution.is_none());
    assert_eq!(std::fs::read(destination).unwrap(), b"selected CASC bytes");
}

#[test]
fn disabled_gethe_policy_rejects_substitution_provenance() {
    let mut provenance = installation_provenance("1.60.1.70634");
    append_source_identity(&mut provenance, "1.60.1.70634", true).unwrap();
    let expected = provenance.contents().to_owned();
    assert!(provenance_matches(&expected, &expected));
    append_substitution(
        &mut provenance,
        &Substitution {
            path: ENTRY.into(),
            fdid: 8379893,
            ckey: HELLO_MD5.into(),
            gethe_revision: "0".repeat(40),
            source: "gethe".into(),
        },
    )
    .unwrap();
    assert!(!provenance_matches(provenance.contents(), &expected));
    assert!(!provenance_matches(
        &expected.replace("70634", "70532"),
        &expected
    ));
}
