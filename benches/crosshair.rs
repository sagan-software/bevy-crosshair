//! Criterion timings and instrumented execution of the public conformance
//! matrix. Geometry cases separate each style and spread condition from
//! configuration and marker operations. Optional rendering cases measure
//! complete offscreen update and submission costs. Coverage runs execute the
//! same binary independently of release timing measurements.

#[path = "../tests/support/mod.rs"]
mod support;

use bevy_crosshair::Crosshair;
use bevy_crosshair::CrosshairInput;
use bevy_crosshair::HitMarker;
use bevy_crosshair::Style;
use criterion::BenchmarkId;
use criterion::Criterion;
use std::hint::black_box;
use std::time::Duration;

/// Measures each style independently with static and spread feedback.
fn geometry(criterion: &mut Criterion) {
    // Build fixtures outside timed iterations so only geometry work is measured.
    let mut group = criterion.benchmark_group("geometry");
    for code in 0..10 {
        let style =
            Style::try_from(code).expect("the current profile defines styles zero through nine");
        let crosshair = Crosshair::default().with_style(style);
        // Zero and positive spread separate static output from dynamic feedback.
        for spread in [0, 40] {
            let input = CrosshairInput {
                spread: spread as f32,
                viewport: bevy::math::Vec2::new(1920.0, 1080.0),
                shot: 0.0,
                recoil: bevy::math::Vec2::ZERO,
                weapon: bevy_crosshair::WeaponState::Hip,
                is_friendly: false,
                spectator: bevy_crosshair::Spectator::Local,
            };
            group.bench_with_input(
                BenchmarkId::new(style.to_string(), spread),
                &input,
                |bench, input| bench.iter(|| black_box(crosshair.draw(black_box(input), None))),
            );
        }
    }
    // Disabled output exercises the early visibility path without marker geometry.
    let empty: Crosshair = "crosshair false"
        .parse()
        .expect("the disabled fixture is valid");
    group.bench_function("empty", |bench| {
        bench.iter(|| black_box(empty.draw(&CrosshairInput::default(), None)))
    });
    group.finish();
}

/// Measures active and expired markers, parsing, and canonical export.
fn operations(criterion: &mut Criterion) {
    // Retain fixtures while measuring only each requested operation.
    let crosshair = Crosshair::default();
    let input = CrosshairInput::default();
    let mut expired = HitMarker::default();
    expired.advance(Duration::MAX);
    // Active and expired markers take distinct geometry paths.
    for (name, marker) in [("active", HitMarker::default()), ("expired", expired)] {
        criterion.bench_function(&format!("marker/{name}"), |bench| {
            bench.iter(|| black_box(crosshair.draw(black_box(&input), Some(black_box(&marker)))))
        });
    }
    // Parsing owns validated settings; export owns only the resulting text buffer.
    let source = crosshair.to_string();
    criterion.bench_function("configuration/parse", |bench| {
        bench.iter(|| black_box(black_box(&source).parse::<Crosshair>()))
    });
    criterion.bench_function("configuration/export", |bench| {
        bench.iter(|| black_box(crosshair.to_string()))
    });
}

/// Keeps validation and rare policy paths in the measured benchmark executable.
fn conformance(criterion: &mut Criterion) {
    let scenarios: [(&str, fn()); 6] = [
        ("configuration", support::configuration),
        ("geometry", support::geometry),
        ("policies", support::policies),
        ("markers", support::markers),
        ("events", support::events),
        ("reflection", support::reflection),
    ];
    for (name, scenario) in scenarios {
        criterion.bench_function(&format!("conformance/{name}"), |bench| bench.iter(scenario));
    }
}

/// Measures the complete offscreen Bevy update and render submission for every style.
#[cfg(feature = "playground")]
fn rendering(criterion: &mut Criterion) {
    // Complete shader compilation before measuring frame submission.
    let mut app = support::rendering();
    let target = app.world_mut().spawn(Crosshair::default()).id();
    let mut group = criterion.benchmark_group("render_submission");
    // Keep one entity alive while changing only its rendering mode.
    for code in 0..10 {
        let style = Style::try_from(code).expect("the current profile defines this style");
        app.world_mut()
            .get_entity_mut(target)
            .expect("the benchmark target is alive")
            .insert(Crosshair::default().with_style(style));
        app.update();
        group.bench_function(style.to_string(), |bench| bench.iter(|| app.update()));
    }
    group.finish();
}

/// Criterion retains its normal CLI, including --test for instrumented execution.
fn main() {
    // Keep timing settings stable across modes and preserve Criterion CLI overrides.
    let mut criterion = Criterion::default()
        .sample_size(20)
        .warm_up_time(Duration::from_millis(300))
        .measurement_time(Duration::from_secs(1))
        .configure_from_args();
    geometry(&mut criterion);
    operations(&mut criterion);
    // Coverage mode still executes boundary scenarios that ordinary timings rarely reach.
    conformance(&mut criterion);
    #[cfg(feature = "playground")]
    rendering(&mut criterion);
    criterion.final_summary();
}

use arrayvec as _;

use thiserror as _;

use strum as _;

#[cfg(not(target_arch = "wasm32"))]
use naga as _;

#[cfg(not(target_arch = "wasm32"))]
use stats_alloc as _;
