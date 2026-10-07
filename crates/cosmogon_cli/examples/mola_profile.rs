//! Reproducible warm-query microbenchmark; excludes loading, textures, GPU and simulation.
use cosmogon_sim::planet::mola::Dataset;
use std::{hint::black_box, time::Instant};
fn main() {
    for dataset in [Dataset::Four, Dataset::Sixteen] {
        let mut runs = Vec::new();
        let mut checksum = 0.0;
        let samples = 200_000;
        for run in 0..6 {
            let start = Instant::now();
            for i in 0..samples {
                let lat = ((i * 73 % 18000) as f64) / 100.0 - 90.0;
                let lon = ((i * 317 % 36000) as f64) / 100.0;
                checksum += black_box(dataset.read(black_box(lat), black_box(lon)).unwrap().height_above_areoid_m);
            }
            if run > 0 { runs.push(start.elapsed().as_secs_f64() * 1e9 / samples as f64); }
        }
        runs.sort_by(f64::total_cmp);
        println!("{}: median {:.2} ns/query; five {}-query runs after warmup; checksum {}", dataset.id(), runs[2], samples, black_box(checksum));
    }
}
