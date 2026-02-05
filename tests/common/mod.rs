//! Shared test helpers.

use std::path::Path;
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

// Blizzard SharedXML paths for loading templates
pub const BLIZZARD_SHARED_XML_BASE_TOC: &str =
    "/home/osso/Projects/wow/reference-addons/wow-ui-source/Interface/AddOns/Blizzard_SharedXMLBase/Blizzard_SharedXMLBase.toc";
pub const BLIZZARD_SHARED_XML_TOC: &str =
    "/home/osso/Projects/wow/reference-addons/wow-ui-source/Interface/AddOns/Blizzard_SharedXML/Blizzard_SharedXML_Mainline.toc";

/// Helper to load Blizzard_SharedXML templates for tests that need them.
/// Returns the environment with templates loaded.
#[allow(dead_code)]
pub fn env_with_shared_xml() -> WowLuaEnv {
    let env = WowLuaEnv::new().expect("Failed to create Lua environment");

    // Load SharedXMLBase first (dependency)
    let base_path = Path::new(BLIZZARD_SHARED_XML_BASE_TOC);
    if base_path.exists() {
        if let Err(e) = load_addon(&env, base_path) {
            eprintln!("Warning: Failed to load SharedXMLBase: {}", e);
        }
    }

    // Load SharedXML (contains scroll templates)
    let shared_path = Path::new(BLIZZARD_SHARED_XML_TOC);
    if shared_path.exists() {
        if let Err(e) = load_addon(&env, shared_path) {
            eprintln!("Warning: Failed to load SharedXML: {}", e);
        }
    }

    env
}
