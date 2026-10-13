//! Criterion timings and instrumented execution of the public conformance
//! matrix. Geometry cases separate each style and spread condition from
//! configuration and marker operations. Optional rendering cases measure
//! complete offscreen update and submission costs. Coverage runs execute the
//! same binary independently of release timing measurements.

#[path = "../tests/support/mod.rs"]
mod support;

use bevy::color::Alpha;
use bevy_crosshair::Crosshair;
use bevy_crosshair::CrosshairInput;
use bevy_crosshair::HitMarker;
use bevy_crosshair::MarkerAnimation;
use bevy_crosshair::MarkerShape;
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

/// Measures each marker shape and animation at a visible point in its lifetime.
fn marker_modes(criterion: &mut Criterion) {
    let crosshair = Crosshair::default();
    let input = CrosshairInput::default();
    // Each pair gets a retained fixture so setup is excluded from timing.
    for shape in [
        MarkerShape::Cross,
        MarkerShape::Plus,
        MarkerShape::Chevron,
        MarkerShape::Diamond,
        MarkerShape::Ring,
    ] {
        for animation in [
            MarkerAnimation::Fade,
            MarkerAnimation::Expand,
            MarkerAnimation::Contract,
            MarkerAnimation::Pulse,
        ] {
            let mut marker = HitMarker::default()
                .with_shape(shape)
                .with_animation(animation);
            // Exclude clock advancement from the geometry timing itself.
            marker.advance(Duration::from_millis(90));
            criterion.bench_function(&format!("marker/{shape:?}/{animation:?}"), |bench| {
                bench
                    .iter(|| black_box(crosshair.draw(black_box(&input), Some(black_box(&marker)))))
            });
        }
    }
}

/// Measures one hundred configurations without allocating between iterations.
fn gallery(criterion: &mut Criterion) {
    let input = CrosshairInput::default();
    // A full gallery retains its owned configurations outside the timed loop.
    let gallery: [Crosshair; 100] = std::array::from_fn(|index| {
        Crosshair::default()
            .with_style(Style::try_from((index % 10) as u8).expect("valid style"))
            .with_fill(bevy::color::LinearRgba::BLUE.with_alpha(0.2))
            .with_dot_shape(bevy_crosshair::DotShape::Circle)
    });
    criterion.bench_function("gallery/100_geometry", |bench| {
        bench.iter(|| {
            for crosshair in &gallery {
                black_box(crosshair.draw(black_box(&input), None));
            }
        })
    });
}

/// Measures arbitrary console colors, including the gallery's non-primary colors.
fn colored_geometry(criterion: &mut Criterion) {
    // Retain validated settings outside timing; changing input still regenerates every drawing.
    let gallery: [Crosshair; 100] = std::array::from_fn(|index| {
        let mut crosshair = Crosshair::default()
            .with_style(Style::try_from((index % 10) as u8).expect("valid style"))
            .with_fill(bevy::color::LinearRgba::BLUE.with_alpha(0.2))
            .with_dot_shape(bevy_crosshair::DotShape::Circle);
        // Fixed byte permutations make repeated baseline and candidate runs use identical colors.
        for (channel, offset) in [("r", 31), ("g", 97), ("b", 163), ("a", 229)] {
            let foreground = ((index * 37 + offset) % 256).to_string();
            let border = ((index * 53 + offset) % 256).to_string();
            crosshair
                .set(&format!("cl_crosshaircolor_{channel}"), &foreground)
                .expect("valid byte");
            crosshair
                .set(&format!("cl_crosshairoutline_{channel}"), &border)
                .expect("valid byte");
        }
        crosshair
    });
    let mut input = CrosshairInput::default();
    criterion.bench_function("gallery/100_colored_geometry", |bench| {
        bench.iter(|| {
            // Exercise new geometry on every iteration, including styles that react to spread.
            input.spread = (input.spread + 1.0) % 41.0;
            for crosshair in &gallery {
                black_box(crosshair.draw(black_box(&input), None));
            }
        })
    });
    // A single colored crosshair separates geometry cost from the batch loop.
    let mut group = criterion.benchmark_group("colored_geometry");
    for crosshair in gallery.iter().take(10) {
        let style = crosshair.style();
        group.bench_function(style.to_string(), |bench| {
            bench.iter(|| black_box(crosshair.draw(black_box(&input), None)))
        });
    }
    group.finish();
}

/// Keeps validation and rare policy paths in the measured benchmark executable.
fn conformance(criterion: &mut Criterion) {
    let scenarios: [(&str, fn()); 8] = [
        ("configuration", support::configuration),
        ("geometry", support::geometry),
        ("policies", support::policies),
        ("markers", support::markers),
        ("events", support::events),
        ("reflection", support::reflection),
        ("extensions", support::extensions),
        ("layers", support::layers),
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
    // Allow twenty GPU samples to complete without extending the requested measurement window.
    group.measurement_time(Duration::from_secs(3));
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
    marker_modes(&mut criterion);
    gallery(&mut criterion);
    colored_geometry(&mut criterion);
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
