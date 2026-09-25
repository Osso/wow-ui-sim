//! Benchmark binary: measure spellbook open on the real GUI app path.

use wow_ui_sim::iced_app::{
    BenchmarkPhase, benchmark_spellbook_open_in_gui, load_benchmark_ui_env,
};

fn main() {
    let env = load_benchmark_ui_env();
    let report = benchmark_spellbook_open_in_gui(env).expect("spellbook GUI benchmark failed");

    print_phase(&report.startup_idle);
    print_phase(&report.first_open);
    print_phase(&report.first_close);
    print_phase(&report.second_open);
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
