//! Native ICU discovery is compiled and invoked only for retail-12-1-5.
use std::env;
use std::path::PathBuf;

const WINDOWS_TRIPLET: &str = "x64-windows-static-md";

pub(super) fn build() {
    for path in [
        "build/intl_native.rs",
        "native/intl/bridge.h",
        "native/intl/text.c",
        "native/intl/number_format.c",
        "native/intl/currency_metadata.c",
    ] {
        println!("cargo:rerun-if-changed={path}");
    }
    let mut compiler = cc::Build::new();
    compiler.files([
        "native/intl/text.c",
        "native/intl/number_format.c",
        "native/intl/currency_metadata.c",
    ]);
    compiler.include("native/intl").std("c11");
    let target = env::var("TARGET").expect("Cargo TARGET is required for ICU4C");
    if target == "x86_64-pc-windows-msvc" {
        build_windows_icu(compiler);
    } else if env::var("CARGO_CFG_TARGET_FAMILY").as_deref() == Ok("unix") {
        build_unix_icu(compiler);
    } else {
        panic!("ICU4C bridge supports Unix and x86_64-pc-windows-msvc, not {target}");
    }
}

fn build_unix_icu(mut compiler: cc::Build) {
    // Probe without emitting link flags until after the shim archive.
    for package in ["icu-i18n", "icu-uc"] {
        let library = pkg_config::Config::new()
            .atleast_version("72")
            .cargo_metadata(false)
            .probe(package)
            .unwrap_or_else(|error| panic!("PTR requires {package} >= 72 via pkg-config: {error}"));
        compiler.includes(library.include_paths);
    }
    compiler.compile("wow_intl_native");
    for package in ["icu-i18n", "icu-uc"] {
        pkg_config::Config::new()
            .atleast_version("72")
            .probe(package)
            .unwrap_or_else(|error| panic!("link PTR {package} >= 72: {error}"));
    }
}

fn build_windows_icu(mut compiler: cc::Build) {
    let library = find_windows_icu();
    compiler
        .includes(&library.include_paths)
        .define("U_STATIC_IMPLEMENTATION", None);
    compiler.compile("wow_intl_native");
    for line in library.cargo_metadata {
        println!("{line}");
    }
}

fn find_windows_icu() -> vcpkg::Library {
    validate_windows_configuration();
    let root = env::var_os("VCPKG_ROOT")
        .map(PathBuf::from)
        .expect("PTR MSVC requires explicit VCPKG_ROOT with icu:x64-windows-static-md installed");
    let library = vcpkg::Config::new()
        .vcpkg_root(root)
        .target_triplet(WINDOWS_TRIPLET)
        .cargo_metadata(false)
        .find_package("icu")
        .unwrap_or_else(|error| panic!("find static ICU4C in {WINDOWS_TRIPLET}: {error}"));
    assert!(
        library.is_static && library.found_dlls.is_empty(),
        "PTR MSVC ICU4C must be static, without DLL imports"
    );
    assert_eq!(library.vcpkg_triplet, WINDOWS_TRIPLET);
    library
}

fn validate_windows_configuration() {
    for name in ["VCPKG_ROOT", "VCPKGRS_TRIPLET", "VCPKGRS_DYNAMIC"] {
        println!("cargo:rerun-if-env-changed={name}");
    }
    assert!(
        env::var_os("VCPKGRS_DYNAMIC").is_none(),
        "PTR MSVC forbids VCPKGRS_DYNAMIC; install icu:x64-windows-static-md"
    );
    if let Some(triplet) = env::var_os("VCPKGRS_TRIPLET") {
        assert_eq!(
            triplet, WINDOWS_TRIPLET,
            "PTR MSVC requires VCPKGRS_TRIPLET=x64-windows-static-md"
        );
    }
    let features = env::var("CARGO_CFG_TARGET_FEATURE").unwrap_or_default();
    assert!(
        !features.split(',').any(|feature| feature == "crt-static"),
        "x64-windows-static-md requires the dynamic MSVC CRT (no +crt-static)"
    );
}
