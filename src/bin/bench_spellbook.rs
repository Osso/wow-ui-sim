//! Benchmark binary: measure spellbook open on the real GUI app path.
//!
//! Usage: bench_spellbook [--cycles N]
//!
//! Without `--cycles`, prints each phase once. With `--cycles N`, repeats
//! close/open N times after the first open and prints p50/p90/mean for the
//! repeated phases, which is what to compare between builds.

use wow_ui_sim::iced_app::{
    BenchmarkPhase, DurationStats, PhaseStats, benchmark_spellbook_cycles_in_gui,
    benchmark_spellbook_open_in_gui, load_benchmark_ui_env,
};

fn main() {
    let cycles = parse_cycles(std::env::args().skip(1));
    let env = load_benchmark_ui_env();
    match cycles {
        Some(cycles) => run_cycles(env, cycles),
        None => run_single_pass(env),
    }
}

fn parse_cycles(mut args: impl Iterator<Item = String>) -> Option<usize> {
    let flag = args.next()?;
    assert_eq!(flag, "--cycles", "unknown flag {flag}; expected --cycles N");
    let cycles = args
        .next()
        .and_then(|value| value.parse::<usize>().ok())
        .filter(|cycles| *cycles > 0)
        .expect("--cycles needs a positive integer");
    Some(cycles)
}

fn run_single_pass(env: wow_ui_sim::lua_api::WowLuaEnv) {
    let report = benchmark_spellbook_open_in_gui(env).expect("spellbook GUI benchmark failed");
    print_phase(&report.startup_idle);
    print_phase(&report.first_open);
    print_phase(&report.first_close);
    print_phase(&report.second_open);
}

fn run_cycles(env: wow_ui_sim::lua_api::WowLuaEnv, cycles: usize) {
    let report =
        benchmark_spellbook_cycles_in_gui(env, cycles).expect("spellbook cycle benchmark failed");
    print_phase(&report.first_open);
    println!("cycles={}", report.cycles);
    print_stats("repeat_open", &report.repeat_open);
    print_stats("close", &report.close);
}

fn print_stats(name: &str, stats: &PhaseStats) {
    println!(
        "{name}: total {} | draw {}",
        format_stats(&stats.total),
        format_stats(&stats.draw)
    );
}

fn format_stats(stats: &DurationStats) -> String {
    format!(
        "p50={:.2}ms p90={:.2}ms mean={:.2}ms",
        as_ms(stats.p50),
        as_ms(stats.p90),
        as_ms(stats.mean)
    )
}

fn as_ms(duration: std::time::Duration) -> f64 {
    duration.as_secs_f64() * 1000.0
}

fn print_phase(phase: &BenchmarkPhase) {
    eprintln!("=== {} ===", phase.name);
    eprintln!("keypress: {:?}", phase.keypress_elapsed);
    eprintln!("settle:   {:?}", phase.settle_elapsed);
    eprintln!("ticks:    {:?}", phase.tick_elapsed);
    eprintln!("draws:    {:?}", phase.draw_elapsed);
    eprintln!("frames:   {}", phase.frames);
    eprintln!(
        "textures: rgba={} bc={} max_pending_dirty_ids={}",
        phase.textures_loaded, phase.bc_textures_loaded, phase.max_pending_dirty_ids
    );
    eprintln!("shown:    {}", phase.spellbook_shown);
    eprintln!();
}
