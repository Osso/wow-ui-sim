# Sealed PTR compile/authentication

PASS — both exact sealed executables authenticated, one SHA-256 stream per path; digest matches seal, before/after stats unchanged, executable modes preserved 0700.

Source: `62f115d262613a848180143d6b30e2a0b3eb63b0`; production delta: `e37f5426a`. Source association is metadata, not cryptographic source provenance.

Cargo JSON: 741 records, 665 compiler artifacts, 75 build-script records, build-finished success=true, no compiler-message records. Full stderr read: six iced-wgpu manifest deprecation warnings plus one summary line. Full stderr preserved in proof.json. No separately recorded compiler exit code.

Actual Cargo features: client-ptr, not client-retail; retail-12-1-5, retail-12-1-0, forbidden-aspects and forbidden-animation-aspects confirmed in both artifacts and normal fingerprint. Normal original/profile-unique deps hardlink device/inode and recorded/current fingerprint association verified without hashing originals.

- integration: `<private-state>/verification/widget-discrete-progress-current/widget-subtext-ptr-final-20261014T181140348741Z/artifacts/deps/integration-020202567c2bd5a4`
  SHA-256: `8a581aaa25055c8f763ec098e6f6408ab95fe8739ca0812dd3f51aab15cc90fc`
  Receipt: `<private-state>/verification/widget-discrete-progress-current/widget-subtext-ptr-final-20261014T181140348741Z/receipt-integration.json`
- wow-sim: `<private-state>/verification/widget-discrete-progress-current/widget-subtext-ptr-final-20261014T181140348741Z/artifacts/wow-sim`
  SHA-256: `d8d18c0ac250022117810d2a67c4e155c1c98434719d762b8a2d5f6c5a13467c`
  Receipt: `<private-state>/verification/widget-discrete-progress-current/widget-subtext-ptr-final-20261014T181140348741Z/receipt-wow-sim.json`

Receipt readback confirmed flat status exactly PASS, path exactly seal.sealed_path, sha256_matches_seal=true, integer hash_reads=1, stat_before/stat_after, source/profile/features. Output directory 0700; reports and receipts 0600, fsynced and read back.

Compile/authentication only — **NOT PTR GREEN, native parity, or parent completion**. No builds/tests/runtime, delegation, canonical edits, executable copy/create/rename, or old-binary/library/archive hashes.
