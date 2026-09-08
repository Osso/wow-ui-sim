use super::{Result, evaluate, records};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{path::Path, process::Command};

const DIRECTORY: &str = "Interface/AddOns/Blizzard_APIDocumentationGenerated";

pub(super) struct Snapshot {
    pub metadata: Value,
    pub records: records::Records,
}

fn git(repository: &Path, args: &[&str]) -> Result<Vec<u8>> {
    let output = Command::new("git")
        .arg("-C")
        .arg(repository)
        .args(args)
        .output()?;
    if !output.status.success() {
        return Err(format!("git {args:?}: {}", String::from_utf8_lossy(&output.stderr)).into());
    }
    Ok(output.stdout)
}

pub(super) fn read(repository: &Path, revision: &str) -> Result<Snapshot> {
    let commit = String::from_utf8(git(
        repository,
        &[
            "rev-parse",
            "--verify",
            "--end-of-options",
            &format!("{revision}^{{commit}}"),
        ],
    )?)?
    .trim()
    .to_owned();
    let version_bytes = git(repository, &["show", &format!("{commit}:version.txt")])?;
    let full_version = std::str::from_utf8(&version_bytes)?.trim();
    let (version, build) = parse_version(full_version)?;
    let files = list_files(repository, &commit)?;
    let mut digest = Sha256::new();
    let mut records = records::Records::new();
    for path in &files {
        let bytes = git(repository, &["show", &format!("{commit}:{path}")])?;
        hash_file(&mut digest, path, &bytes);
        for document in evaluate::documents(&bytes, path)? {
            records::insert_document(&mut records, document)
                .map_err(|error| format!("{commit}:{path}: {error}"))?;
        }
    }
    Ok(Snapshot {
        metadata: json!({"commit": commit, "version": version, "build": build,
            "full_version": full_version, "files": files, "sha256": format!("{:x}", digest.finalize())}),
        records,
    })
}

fn list_files(repository: &Path, commit: &str) -> Result<Vec<String>> {
    let bytes = git(
        repository,
        &[
            "ls-tree",
            "-z",
            "--name-only",
            commit,
            &format!("{DIRECTORY}/"),
        ],
    )?;
    let mut files = Vec::new();
    for path in bytes.split(|b| *b == 0).filter(|p| !p.is_empty()) {
        let path = std::str::from_utf8(path)?;
        let Some(name) = path.strip_prefix(&format!("{DIRECTORY}/")) else {
            continue;
        };
        if !name.contains('/') && name.ends_with("Documentation.lua") {
            files.push(path.to_owned());
        }
    }
    files.sort();
    if files.is_empty() {
        return Err(format!("no documentation files at {commit}").into());
    }
    Ok(files)
}

fn parse_version(full: &str) -> Result<(&str, &str)> {
    let parts: Vec<_> = full.split('.').collect();
    if parts.len() != 4
        || parts
            .iter()
            .any(|s| s.is_empty() || !s.bytes().all(|b| b.is_ascii_digit()))
    {
        return Err(format!("invalid version.txt build identity: {full:?}").into());
    }
    Ok(full.rsplit_once('.').unwrap())
}

fn hash_file(hash: &mut Sha256, path: &str, bytes: &[u8]) {
    hash.update((path.len() as u64).to_be_bytes());
    hash.update(path.as_bytes());
    hash.update((bytes.len() as u64).to_be_bytes());
    hash.update(bytes);
}
