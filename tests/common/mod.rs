//! Shared test helpers.

use std::path::PathBuf;
use wow_ui_sim::loader::load_addon;
use wow_ui_sim::lua_api::WowLuaEnv;

/// Try to create a wgpu device for GPU tests.
/// Returns None if no adapter is available (e.g., headless CI).
#[allow(dead_code)]
pub fn try_create_gpu_device() -> Option<(wgpu::Device, wgpu::Queue)> {
    let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
        backends: wgpu::Backends::all(),
        ..Default::default()
    });

    let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::LowPower,
        compatible_surface: None,
        force_fallback_adapter: false,
    }))
    .ok()?;

    let (device, queue) = pollster::block_on(adapter.request_device(
        &wgpu::DeviceDescriptor {
            label: Some("Test GPU Device"),
            ..Default::default()
        },
    ))
    .ok()?;

    Some((device, queue))
}

fn blizzard_ui_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("Interface/BlizzardUI")
}

/// Helper to load Blizzard_SharedXML templates for tests that need them.
/// Returns the environment with templates loaded.
#[allow(dead_code)]
pub fn env_with_shared_xml() -> WowLuaEnv {
    let env = WowLuaEnv::new().expect("Failed to create Lua environment");
    let ui = blizzard_ui_dir();

    let base_toc = ui.join("Blizzard_SharedXMLBase/Blizzard_SharedXMLBase.toc");
    if base_toc.exists() {
        if let Err(e) = load_addon(&env, &base_toc) {
            eprintln!("Warning: Failed to load SharedXMLBase: {}", e);
        }
    }

    let shared_toc = ui.join("Blizzard_SharedXML/Blizzard_SharedXML_Mainline.toc");
    if shared_toc.exists() {
        if let Err(e) = load_addon(&env, &shared_toc) {
            eprintln!("Warning: Failed to load SharedXML: {}", e);
        }
    }

    env
}
