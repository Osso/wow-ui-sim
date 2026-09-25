//! Benchmark binary: settled-UI tick and draw CPU cost on the real GUI path.
//!
//! Usage: bench_steady_state [--warmup N] [--frames N] [--rounds N]
//!
//! Each frame forces a full OnUpdate interval, so results do not depend on
//! window pacing. Compare runs by the minimum p50 across rounds.

use wow_ui_sim::iced_app::{
    DurationStats, SteadyStateOptions, benchmark_steady_state_in_gui, load_benchmark_ui_env,
};

const DEFAULT_OPTIONS: SteadyStateOptions = SteadyStateOptions {
    warmup_frames: 300,
    measured_frames: 1000,
    rounds: 5,
};

fn main() {
    let options = parse_options(std::env::args().skip(1));
    let env = load_benchmark_ui_env();
    let rounds =
        benchmark_steady_state_in_gui(env, options).expect("steady-state benchmark failed");

    println!(
        "warmup={} frames={} rounds={}",
        options.warmup_frames, options.measured_frames, options.rounds
    );
    for (index, round) in rounds.iter().enumerate() {
        println!(
            "round {index}: tick {} | draw {} | strata_upload_frames={}",
            format_stats(&round.tick),
            format_stats(&round.draw),
            round.strata_upload_frames
        );
    }
    let min_tick = rounds.iter().map(|round| round.tick.p50).min();
    let min_draw = rounds.iter().map(|round| round.draw.p50).min();
    if let (Some(tick), Some(draw)) = (min_tick, min_draw) {
        println!(
            "min p50: tick={:.3}ms draw={:.3}ms",
            as_ms(tick),
            as_ms(draw)
        );
    }
}

fn parse_options(mut args: impl Iterator<Item = String>) -> SteadyStateOptions {
    let mut options = DEFAULT_OPTIONS;
    while let Some(flag) = args.next() {
        let value = args
            .next()
            .and_then(|value| value.parse::<usize>().ok())
            .unwrap_or_else(|| panic!("{flag} needs a positive integer"));
        match flag.as_str() {
            "--warmup" => options.warmup_frames = value,
            "--frames" => options.measured_frames = value.max(1),
            "--rounds" => options.rounds = value.max(1),
            _ => panic!("unknown flag {flag}; expected --warmup, --frames or --rounds"),
        }
    }
    options
}

fn format_stats(stats: &DurationStats) -> String {
    format!(
        "p50={:.3}ms p90={:.3}ms mean={:.3}ms",
        as_ms(stats.p50),
        as_ms(stats.p90),
        as_ms(stats.mean)
    )
}

fn as_ms(duration: std::time::Duration) -> f64 {
    duration.as_secs_f64() * 1000.0
}
