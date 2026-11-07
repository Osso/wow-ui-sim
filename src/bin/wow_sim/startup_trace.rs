use wow_ui_sim::font::WowFontSystem;
#[cfg(target_os = "linux")]
use wow_ui_sim::logging;

pub(super) fn time_load_step<T>(name: &str, action: impl FnOnce() -> T) -> T {
    wow_ui_sim::logging::eprintln_elapsed(&format!("[Startup] begin {name}"));
    let started = std::time::Instant::now();
    let result = action();
    wow_ui_sim::logging::eprintln_elapsed(&format!(
        "[Startup] end {name} in {:.2?}",
        started.elapsed()
    ));
    result
}

pub(super) fn print_process_started() {
    eprintln!(
        "[  0.000s] [startup] wow-sim process started pid={}",
        std::process::id()
    );
}

pub(super) fn font_system_for_command(args: &super::Args) -> WowFontSystem {
    if matches!(args.command, Some(super::Commands::LuaErrors)) {
        WowFontSystem::new_without_casc()
    } else {
        WowFontSystem::new()
    }
}

#[cfg(target_os = "linux")]
pub(super) fn apply_resource_limits() {
    let max_cores: usize = std::env::var("WOW_SIM_MAX_CORES")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(2);
    unsafe {
        let mut cpuset: libc::cpu_set_t = std::mem::zeroed();
        for i in 0..max_cores {
            libc::CPU_SET(i, &mut cpuset);
        }
        libc::sched_setaffinity(0, std::mem::size_of::<libc::cpu_set_t>(), &cpuset);
    }
    logging::eprintln_elapsed(&format!("Resource limits: {max_cores} CPU core(s)"));
}

#[cfg(not(target_os = "linux"))]
pub(super) fn apply_resource_limits() {
    // Linux builds pin CPU usage; other platforms currently run unconstrained.
}
