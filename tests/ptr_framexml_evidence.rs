#![cfg(feature = "client-ptr")]
use sha2::{Digest, Sha256};
use std::{fs, path::Path, process::Command};

fn evidence_bytes(root: &Path, reference: &str) -> Vec<u8> {
    if let Some(object) = reference.strip_prefix("git:") {
        let output = Command::new("git")
            .args(["cat-file", "blob", object])
            .current_dir(root)
            .output()
            .expect("read historical evidence");
        assert!(output.status.success(), "{reference}");
        return output.stdout;
    }
    fs::read(root.join(reference.split("::").next().unwrap())).expect("read evidence")
}

#[test]
fn ptr_framexml_manifest_evidence_hashes_match() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let manifest: serde_json::Value =
        serde_json::from_slice(&fs::read(root.join("data/patch-api/12.1-framexml.json")).unwrap())
            .unwrap();
    let mut checked = 0;
    for row in manifest["rows"].as_array().unwrap() {
        for evidence in row["evidence"].as_array().unwrap() {
            let reference = evidence["reference"].as_str().unwrap();
            let hash = format!("{:x}", Sha256::digest(evidence_bytes(root, reference)));
            assert_eq!(
                Some(hash.as_str()),
                evidence["source_hash"].as_str(),
                "{}: {reference}",
                row["id"]
            );
            checked += 1;
        }
    }
    assert_eq!(checked, 433);
}
