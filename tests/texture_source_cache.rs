use std::{fs, path::Path, time::Instant};
use wow_ui_sim::{lua_api::WowLuaEnv, texture::TextureManager};

fn png(root: &Path, relative: &str, rgba: [u8; 4]) {
    let path = root.join(relative);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    image::RgbaImage::from_pixel(2, 1, image::Rgba(rgba))
        .save(path)
        .unwrap();
}

fn roots(env: &WowLuaEnv, paths: &[&Path]) {
    env.state().borrow_mut().addon_base_paths = paths.iter().map(|p| p.to_path_buf()).collect();
}

fn setter(env: &WowLuaEnv, path: &str, expected: bool) {
    env.exec(&format!(
        r#"
        CacheProbe = CacheProbe or UIParent:CreateTexture('CacheProbe')
        local function check(...)
            assert(select('#', ...) == 1)
            assert(type((...)) == 'boolean' and (...) == {expected})
        end
        check(CacheProbe:SetTexture('{path}'))
        assert(CacheProbe:GetTexture() == '{path}')
    "#
    ))
    .unwrap();
}

#[test]
fn repeated_requests_live_create_delete_and_directory_rejection() {
    let root = tempfile::tempdir().unwrap();
    fs::create_dir_all(root.path().join("F1/Media")).unwrap();
    fs::write(root.path().join("F1/Media/Present.tga"), []).unwrap();
    fs::create_dir(root.path().join("F1/Media/Directory.tga")).unwrap();
    let env = WowLuaEnv::new().unwrap();
    roots(&env, &[root.path()]);
    for _ in 0..3 {
        setter(&env, "Interface/AddOns/F1/Media/Present", true);
        setter(&env, "interface/addons/f1/media/present", true);
        setter(&env, "Interface/AddOns/F1/Media/Missing", false);
        setter(&env, "interface/addons/f1/media/directory", false);
    }
    fs::write(root.path().join("F1/Media/Missing.tga"), []).unwrap();
    setter(&env, "interface/addons/f1/media/missing", true);
    fs::remove_file(root.path().join("F1/Media/Present.tga")).unwrap();
    setter(&env, "interface/addons/f1/media/present", false);
    setter(&env, "interface/addons/newaddon/media/late", false);
    fs::create_dir_all(root.path().join("NewAddon/Media")).unwrap();
    fs::write(root.path().join("NewAddon/Media/Late.tga"), []).unwrap();
    setter(&env, "interface/addons/newaddon/media/late", true);
}

#[test]
fn warm_source_checks_preserve_assignment_side_effects() {
    let root = tempfile::tempdir().unwrap();
    png(root.path(), "F1/Media/Logo.png", [255, 0, 0, 255]);
    let env = WowLuaEnv::new().unwrap();
    roots(&env, &[root.path()]);
    setter(&env, "interface/addons/f1/media/logo", true);
    env.exec(
        r#"
        local t = CacheProbe
        local path = 'interface/addons/f1/media/logo'
        t:SetColorTexture(1, 0, 0, 1)
        assert(t:SetTexture(path, true, false))
        assert(t:GetTexture() == path and t:GetHorizTile() and not t:GetVertTile())
        t:SetAtlas('checkbox-minimal')
        assert(t:GetAtlas() == 'checkbox-minimal')
        assert(t:SetTexture(path, false, true))
        assert(t:GetAtlas() == nil and t:GetTexture() == path)
        assert(not t:GetHorizTile() and t:GetVertTile())
        assert(not t:SetTexture(nil) and t:GetTexture() == nil)
        assert(t:SetTexture(path))
        assert(not t:SetTexture('') and t:GetTexture() == nil)
    "#,
    )
    .unwrap();
    let state = env.state().borrow();
    let id = state.widgets.get_id_by_name("CacheProbe").unwrap();
    let frame = state.widgets.get(id).unwrap();
    assert!(frame.color_texture.is_none());
    assert!(frame.atlas_tex_coords.is_none());
}

#[cfg(feature = "forbidden-aspects")]
#[test]
fn warm_source_checks_preserve_secret_replacement_and_clear() {
    let root = tempfile::tempdir().unwrap();
    png(root.path(), "F1/Media/Logo.png", [255, 0, 0, 255]);
    let env = WowLuaEnv::new().unwrap();
    roots(&env, &[root.path()]);
    setter(&env, "interface/addons/f1/media/logo", true);
    env.exec(
        r#"
        local t = CacheProbe
        local path = 'interface/addons/f1/media/logo'
        assert(t:SetTexture(secretwrap(path)))
        local function denied() assert(not pcall(t.GetTexture, t)) end
        debug.setobjecttaint(denied, 'CacheSecretProbe')
        denied()
        assert(t:SetTexture(path))
        local function public() assert(t:GetTexture() == path) end
        debug.setobjecttaint(public, 'CacheSecretProbe')
        public()
        assert(t:SetTexture(secretwrap(path)))
        assert(not t:SetTexture(secretwrap(nil)))
        assert(t:GetTexture() == nil)
    "#,
    )
    .unwrap();
}

#[test]
fn setter_discovers_root_changes_and_keeps_environments_isolated() {
    let a = tempfile::tempdir().unwrap();
    let b = tempfile::tempdir().unwrap();
    png(a.path(), "F1/Media/Logo.png", [255, 0, 0, 255]);
    let env = WowLuaEnv::new().unwrap();
    roots(&env, &[a.path()]);
    setter(&env, "interface/addons/f1/media/logo", true);
    roots(&env, &[b.path()]);
    setter(&env, "interface/addons/f1/media/logo", false);
    let other = WowLuaEnv::new().unwrap();
    roots(&other, &[a.path()]);
    setter(&other, "interface/addons/f1/media/logo", true);
    roots(&env, &[a.path()]);
    setter(&env, "interface/addons/f1/media/logo", true);
}

#[test]
fn renderer_retries_missing_source_after_creation() {
    let root = tempfile::tempdir().unwrap();
    fs::create_dir_all(root.path().join("F1/Media")).unwrap();
    let mut mgr = TextureManager::new().with_addons_path(root.path());
    let request = "Interface/AddOns/F1/Media/Late";
    assert!(mgr.load(request).is_none());
    png(root.path(), "F1/Media/Late.png", [16, 32, 48, 255]);
    let loaded = mgr
        .load(request)
        .expect("newly created texture must be discovered");
    assert_eq!((loaded.width, loaded.height), (2, 1));
    assert_eq!(&loaded.pixels[..4], &[16, 32, 48, 255]);
}

#[test]
fn renderer_retries_bc_after_non_bc_source_is_replaced() {
    use wow_ui_sim::texture::BcTextureFormat;
    let root = tempfile::tempdir().unwrap();
    png(root.path(), "F1/Media/Logo.png", [16, 32, 48, 255]);
    let mut manager = TextureManager::new().with_addons_path(root.path());
    let request = "interface/addons/f1/media/logo";
    assert!(manager.load_bc(request).is_none());
    // Concrete BLP2: direct DXT1, 4x4, palette padding required by image-blp,
    // one eight-byte mip following the 148-byte header and 1024-byte palette.
    const MIP_OFFSET: usize = 148 + 1024;
    let mut blp = vec![0u8; MIP_OFFSET + 8];
    blp[..4].copy_from_slice(b"BLP2");
    blp[4..8].copy_from_slice(&1u32.to_le_bytes());
    blp[8] = 2;
    blp[11] = 1;
    blp[12..16].copy_from_slice(&4u32.to_le_bytes());
    blp[16..20].copy_from_slice(&4u32.to_le_bytes());
    blp[20..24].copy_from_slice(&(MIP_OFFSET as u32).to_le_bytes());
    blp[84..88].copy_from_slice(&8u32.to_le_bytes());
    blp[MIP_OFFSET..].fill(0xaa);
    image_blp::parser::load_blp_from_buf(&blp).expect("valid BC fixture");
    fs::write(root.path().join("F1/Media/Logo.blp"), blp).unwrap();
    let loaded = manager
        .load_bc(request)
        .expect("new higher-priority BLP must replace non-BC decision");
    assert_eq!((loaded.width, loaded.height), (4, 4));
    assert_eq!(loaded.format, BcTextureFormat::Bc1);
    assert_eq!(&*loaded.bc_data, &[0xaa; 8]);
}

#[test]
fn renderer_manager_observes_setter_root_order_and_reload() {
    let a = tempfile::tempdir().unwrap();
    let b = tempfile::tempdir().unwrap();
    png(a.path(), "F1/Media/Logo.png", [255, 0, 0, 255]);
    png(b.path(), "F1/Media/Logo.png", [0, 0, 255, 255]);
    let env = WowLuaEnv::new().unwrap();
    let request = "interface/addons/f1/media/logo";
    roots(&env, &[a.path(), b.path()]);
    setter(&env, request, true);
    let manager = env.state().borrow().texture_manager();
    assert_eq!(
        manager.borrow().resolve_path(request),
        Some(a.path().join("F1/Media/Logo.png"))
    );
    roots(&env, &[b.path(), a.path()]);
    setter(&env, request, true);
    assert_eq!(
        manager.borrow().resolve_path(request),
        Some(b.path().join("F1/Media/Logo.png"))
    );
    env.exec("ReloadUI()").unwrap();
    assert_eq!(
        manager.borrow().resolve_path(request),
        Some(b.path().join("F1/Media/Logo.png"))
    );
}

#[test]
fn exact_case_phase_precedes_ci_extension_phase() {
    let root = tempfile::tempdir().unwrap();
    png(root.path(), "F1/Media/Logo.png", [255, 0, 0, 255]);
    png(root.path(), "F1/Media/logo.webp", [0, 0, 255, 255]);
    let mgr = TextureManager::new().with_addons_path(root.path());
    for _ in 0..3 {
        assert_eq!(
            mgr.resolve_path("Interface/AddOns/F1/Media/Logo"),
            Some(root.path().join("F1/Media/Logo.png"))
        );
        assert_eq!(
            mgr.resolve_path("Interface/AddOns/F1/Media/logo"),
            Some(root.path().join("F1/Media/logo.webp"))
        );
    }
}

#[test]
fn new_earlier_root_or_extension_visible_without_reload() {
    let a = tempfile::tempdir().unwrap();
    let b = tempfile::tempdir().unwrap();
    png(b.path(), "F1/Media/Logo.png", [0, 0, 255, 255]);
    let mgr =
        TextureManager::new().with_addons_paths([a.path().to_path_buf(), b.path().to_path_buf()]);
    let p = "interface/addons/f1/media/logo";
    assert_eq!(
        mgr.resolve_path(p),
        Some(b.path().join("F1/Media/Logo.png"))
    );
    png(a.path(), "F1/Media/Logo.png", [255, 0, 0, 255]);
    assert_eq!(
        mgr.resolve_path(p),
        Some(a.path().join("F1/Media/Logo.png"))
    );
    png(a.path(), "F1/Media/Logo.webp", [0, 255, 0, 255]);
    assert_eq!(
        mgr.resolve_path(p),
        Some(a.path().join("F1/Media/Logo.webp"))
    );
    fs::remove_file(a.path().join("F1/Media/Logo.webp")).unwrap();
    assert_eq!(
        mgr.resolve_path(p),
        Some(a.path().join("F1/Media/Logo.png"))
    );
}

#[test]
fn reload_and_same_spelling_root_replacement_do_not_retain_misses() {
    let root = tempfile::tempdir().unwrap();
    let env = WowLuaEnv::new().unwrap();
    roots(&env, &[root.path()]);
    setter(&env, "interface/addons/f1/media/logo", false);
    png(root.path(), "F1/Media/Logo.png", [17, 34, 51, 255]);
    env.exec("ReloadUI()").unwrap();
    setter(&env, "interface/addons/f1/media/logo", true);
    fs::rename(root.path().join("F1"), root.path().join("old-F1")).unwrap();
    setter(&env, "interface/addons/f1/media/logo", false);
    png(root.path(), "f1/media/LOGO.png", [68, 85, 102, 255]);
    setter(&env, "interface/addons/f1/media/logo", true);
}

#[cfg(feature = "casc")]
#[test]
fn failed_casc_extraction_stays_quiet_until_root_or_reload() {
    const CHILD_FLAG: &str = "F1_CASC_FAILURE_FIXTURE";
    if let Some(fixture) = std::env::var_os(CHILD_FLAG) {
        let fixture = std::path::PathBuf::from(fixture);
        let mut env = WowLuaEnv::new().unwrap();
        roots(&env, &[&fixture.join("addons-a")]);
        let manager = env.state().borrow().texture_manager();
        let request = "Interface/Icons/INV_Misc_QuestionMark";
        let load_missing_frame = || {
            assert!(manager.borrow_mut().load_bc(request).is_none());
            assert!(manager.borrow_mut().load(request).is_none());
        };
        eprintln!("F1_CASC_PHASE first");
        load_missing_frame();
        eprintln!("F1_CASC_PHASE warm");
        for _ in 0..64 {
            load_missing_frame();
        }
        eprintln!("F1_CASC_PHASE root");
        roots(&env, &[&fixture.join("addons-b")]);
        drop(env.state().borrow().texture_manager());
        for _ in 0..64 {
            load_missing_frame();
        }
        eprintln!("F1_CASC_PHASE reload");
        env.exec("ReloadUI()").unwrap();
        assert!(env.service_reload(&mut None).unwrap());
        for _ in 0..64 {
            load_missing_frame();
        }
        eprintln!("F1_CASC_PHASE end");
        return;
    }

    // Real resolver/extraction failure in a private incomplete install: no
    // real game files, network catalog, mocks, or synthetic call counters.
    let fixture = tempfile::tempdir().unwrap();
    for relative in [
        "install/Data/data",
        "catalog",
        "cache",
        "addons-a",
        "addons-b",
    ] {
        fs::create_dir_all(fixture.path().join(relative)).unwrap();
    }
    fs::write(
        fixture.path().join("catalog/community-listfile.csv"),
        "134400;interface/icons/inv_misc_questionmark.blp\n",
    )
    .unwrap();
    let output = std::process::Command::new(std::env::current_exe().unwrap())
        .args([
            "--exact",
            "texture_source_cache::failed_casc_extraction_stays_quiet_until_root_or_reload",
            "--nocapture",
        ])
        .env(CHILD_FLAG, fixture.path())
        .env("WOW_INSTALL_PATH", fixture.path().join("install"))
        .env("WOW_DATA_PATH", fixture.path().join("install/Data"))
        .env("WOW_SIM_CASC", "1")
        .env("ASSET_RESOLVER_DATA_DIR", fixture.path().join("catalog"))
        .env(
            "ASSET_RESOLVER_CACHE_DIR",
            fixture.path().join("resolver-cache"),
        )
        .env("XDG_CACHE_HOME", fixture.path().join("cache"))
        .output()
        .unwrap();
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(output.status.success(), "child failed: {stderr}");
    let failure = "asset-cache extraction failed: fdid 134400 ->";
    for (phase, next, expected) in [
        ("first", "warm", 2),
        ("warm", "root", 0),
        ("root", "reload", 2),
        ("reload", "end", 2),
    ] {
        let section = stderr
            .split_once(&format!("F1_CASC_PHASE {phase}\n"))
            .unwrap()
            .1
            .split_once(&format!("F1_CASC_PHASE {next}\n"))
            .unwrap()
            .0;
        assert_eq!(
            section.matches(failure).count(),
            expected,
            "phase {phase}: {stderr}"
        );
    }
    eprintln!("F1_CASC_FAILURES first=2 warm64=0 root64=2 reload64=2");
}

#[cfg(target_os = "linux")]
mod directory_io {
    use super::*;
    use std::{
        ffi::CString,
        io::Read,
        os::fd::{AsRawFd, FromRawFd},
        os::unix::ffi::OsStrExt,
    };

    // Observe actual directory opens through the kernel, not resolver internals.
    struct DirectoryObserver(fs::File);
    impl DirectoryObserver {
        fn new(path: &Path) -> Self {
            let fd = unsafe { libc::inotify_init1(libc::IN_NONBLOCK | libc::IN_CLOEXEC) };
            assert!(fd >= 0, "{}", std::io::Error::last_os_error());
            let file = unsafe { fs::File::from_raw_fd(fd) };
            let path = CString::new(path.as_os_str().as_bytes()).unwrap();
            let watch = unsafe {
                libc::inotify_add_watch(
                    file.as_raw_fd(),
                    path.as_ptr(),
                    libc::IN_OPEN | libc::IN_CLOSE_NOWRITE | libc::IN_ONLYDIR,
                )
            };
            assert!(watch >= 0, "{}", std::io::Error::last_os_error());
            Self(file)
        }
        fn opens(&mut self) -> usize {
            let mut count = 0;
            let mut buf = [0u8; 16384];
            loop {
                let size = match self.0.read(&mut buf) {
                    Ok(n) => n,
                    Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => break,
                    Err(e) => panic!("inotify read: {e}"),
                };
                assert!(size > 0);
                let mut offset = 0;
                while offset < size {
                    let event = unsafe {
                        (buf.as_ptr().add(offset) as *const libc::inotify_event).read_unaligned()
                    };
                    assert_eq!(event.mask & libc::IN_Q_OVERFLOW, 0, "observer overflow");
                    if event.mask & (libc::IN_OPEN | libc::IN_ISDIR)
                        == (libc::IN_OPEN | libc::IN_ISDIR)
                    {
                        count += 1;
                    }
                    offset += std::mem::size_of::<libc::inotify_event>() + event.len as usize;
                }
            }
            count
        }
    }

    #[test]
    fn repeated_frame_load_misses_are_cheap_until_file_creation() {
        let root = tempfile::tempdir().unwrap();
        let leaf = root.path().join("F1/Media");
        fs::create_dir_all(&leaf).unwrap();
        let mut manager = TextureManager::new().with_addons_path(root.path());
        let mut observer = DirectoryObserver::new(&leaf);
        let request = "interface/addons/f1/media/late";
        let mut opens = 0;
        for _frame in 0..64 {
            assert!(manager.load_bc(request).is_none());
            assert!(manager.load(request).is_none());
            opens += observer.opens();
        }
        png(root.path(), "F1/Media/Late.png", [16, 32, 48, 255]);
        let texture = manager
            .load(request)
            .expect("new file must invalidate the cheap negative");
        assert_eq!((texture.width, texture.height), (2, 1));
        assert_eq!(&texture.pixels[..4], &[16, 32, 48, 255]);
        eprintln!("F1_FRAME_MISSES frames=64 leaf_opens={opens}");
        assert!(
            opens <= 2,
            "unchanged BC/RGBA misses opened directory {opens} times, budget 2"
        );
    }

    #[test]
    fn renderer_reuses_setter_directory_observations_until_reload() {
        let root = tempfile::tempdir().unwrap();
        let leaf = root.path().join("F1/Media");
        fs::create_dir_all(&leaf).unwrap();
        fs::write(leaf.join("Present.blp"), []).unwrap();
        let mut env = WowLuaEnv::new().unwrap();
        roots(&env, &[root.path()]);
        let mut observer = DirectoryObserver::new(&leaf);
        let request = "interface/addons/f1/media/present";
        setter(&env, request, true);
        assert_eq!(observer.opens(), 1);
        let manager = env.state().borrow().texture_manager();
        assert_eq!(
            manager.borrow().resolve_path(request),
            Some(leaf.join("Present.blp"))
        );
        assert_eq!(
            observer.opens(),
            0,
            "renderer must reuse setter's source listing"
        );
        env.exec("ReloadUI()").unwrap();
        assert!(env.service_reload(&mut None).unwrap());
        assert_eq!(
            manager.borrow().resolve_path(request),
            Some(leaf.join("Present.blp"))
        );
        assert_eq!(
            observer.opens(),
            1,
            "reload must discard source observations"
        );
    }

    #[test]
    fn repeated_mixed_case_setters_bound_directory_enumerations() {
        let root = tempfile::tempdir().unwrap();
        let leaf = root.path().join("F1/Media/Rounded");
        fs::create_dir_all(&leaf).unwrap();
        for n in 0..64 {
            fs::write(leaf.join(format!("Present{n:03}.tga")), []).unwrap();
        }
        for n in 0..461 {
            fs::write(leaf.join(format!("Padding{n:03}.tga")), []).unwrap();
        }
        let env = WowLuaEnv::new().unwrap();
        roots(&env, &[root.path()]);
        let mut observer = DirectoryObserver::new(&leaf);
        // Calibrate independent observer: alternating open/close events prevent coalescing.
        for _ in 0..3 {
            drop(fs::read_dir(&leaf).unwrap());
        }
        assert_eq!(observer.opens(), 3);
        let start = Instant::now();
        let mut opens = 0;
        for _ in 0..4 {
            for n in 0..64 {
                setter(
                    &env,
                    &format!("interface/addons/f1/media/rounded/present{n:03}"),
                    true,
                );
                opens += observer.opens();
                setter(
                    &env,
                    &format!("interface/addons/f1/media/rounded/missing{n:03}"),
                    false,
                );
                opens += observer.opens();
            }
        }
        eprintln!(
            "F1_FIXTURE files=525 requests=512 leaf_opens={opens} elapsed_us={}",
            start.elapsed().as_micros()
        );
        assert!(
            opens <= 512,
            "repeated requests enumerated fixture directory {opens} times (budget 512)"
        );
    }
}
