//! Allocation measurements for geometry generation through the public API. The
//! allocator tracks every rendering style and empty output after fixtures are
//! constructed. These checks distinguish bounded stack geometry from
//! allocation-backed intermediate collections; they do not claim that Bevy
//! application startup or GPU asset creation allocates nothing.

use bevy as _;
use bevy_crosshair::Crosshair;
use bevy_crosshair::CrosshairInput;
use bevy_crosshair::HitMarker;
use bevy_crosshair::Style;
use stats_alloc::INSTRUMENTED_SYSTEM;
use stats_alloc::Region;
use stats_alloc::StatsAlloc;
use std::alloc::System;

/// Instruments allocation without introducing unsafe code into the plugin.
#[global_allocator]
static ALLOCATOR: &StatsAlloc<System> = &INSTRUMENTED_SYSTEM;

#[test]
fn geometry_allocates_nothing_for_every_style_and_empty_output() {
    // Construct owned fixtures before starting each measured region.
    let input = CrosshairInput::default();
    let marker = HitMarker::default();
    for code in 0..10 {
        let crosshair = Crosshair::default()
            .with_style(Style::try_from(code).expect("the fixture supplies a valid value"));
        // Rendering all styles must avoid heap allocations, including active markers.
        let region = Region::new(ALLOCATOR);
        std::hint::black_box(crosshair.draw(&input, Some(&marker)));
        assert_eq!(region.change().allocations, 0);
    }
    let empty: Crosshair = "crosshair false"
        .parse()
        .expect("the fixture supplies a valid value");
    // Empty geometry must satisfy the same allocation contract.
    let region = Region::new(ALLOCATOR);
    assert!(empty.draw(&input, None).as_ref().is_empty());
    assert_eq!(region.change().allocations, 0);
}

use arrayvec as _;

use thiserror as _;

use strum as _;

#[cfg(not(target_arch = "wasm32"))]
use criterion as _;

#[cfg(not(target_arch = "wasm32"))]
use naga as _;
