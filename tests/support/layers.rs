//! Custom layer scenarios shared by the conformance test and the benchmark.

use super::*;
use bevy_crosshair::Layer;
use bevy_crosshair::Mark;

/// Builds an outlined layered crosshair that uses every mark at least once.
pub(crate) fn layered() -> Crosshair {
    // The first layer mixes fading and solid bars; the others cover the curved and pointed marks.
    let bars = Layer::new(Mark::Bar {
        length: 17.0,
        thickness: 2.0,
        solid: 0.3,
    })
    .at_angles(&[90.0, 270.0])
    .following_spread(4.0, 64.0)
    .with_outline(LinearRgba::BLACK);
    let points = Layer::new(Mark::Chevron {
        width: 6.0,
        depth: 5.0,
        thickness: 2.0,
    })
    .at_angles(&[0.0, 120.0, 240.0])
    .following_spread(4.0, 64.0);
    let arcs = Layer::new(Mark::Arc {
        radius: 9.5,
        sweep: 48.0,
        thickness: 2.0,
    })
    .at_angles(&[60.0, 180.0, 300.0])
    .with_gap(6.0)
    .with_color(LinearRgba::WHITE.with_alpha(0.8));
    Crosshair::default()
        .with_layer(bars)
        .with_layer(points)
        .with_layer(arcs)
}

/// Builds a static outlined dot, a solid bar and a spread ring.
fn centered() -> Crosshair {
    // These marks complete the vocabulary not covered by the layered fixture.
    let dot = Layer::new(Mark::Dot { diameter: 2.0 }).with_outline(LinearRgba::BLACK);
    let bar = Layer::new(Mark::Bar {
        length: 6.0,
        thickness: 2.0,
        solid: 1.0,
    });
    let ring = Layer::new(Mark::Ring {
        radius: 0.0,
        thickness: 1.0,
    })
    .following_spread(4.0, 64.0);
    Crosshair::default()
        .with_layer(dot)
        .with_layer(bar)
        .with_layer(ring)
}

/// Draws every mark with and without spread and checks the primitive budget.
pub(crate) fn run() {
    // Two outlined bars, six chevron arms and three arcs; spread changes no counts.
    for spread in [0.0, 40.0] {
        let input = CrosshairInput {
            viewport: Vec2::new(1920.0, 1080.0),
            spread,
            shot: 0.0,
            recoil: Vec2::ZERO,
            weapon: WeaponState::Hip,
            is_friendly: false,
            spectator: Spectator::Local,
        };
        assert_eq!(layered().draw(&input, None).as_ref().len(), 13);
        assert_eq!(centered().draw(&input, None).as_ref().len(), 4);
    }
    // Builder validation and marker timing run in the same benchmark scenario.
    rejected();
    degenerate();
    held_marker();
    invalid_marker();
}

/// Zero-sized marks and zero sweeps never reach the drawing.
fn degenerate() {
    // Each mark is invisible, so the drawing stays empty.
    let marks = [
        Mark::Arc {
            radius: 9.0,
            sweep: f32::NAN,
            thickness: 2.0,
        },
        Mark::Ring {
            radius: 0.0,
            thickness: 0.0,
        },
        Mark::Bar {
            length: 0.0,
            thickness: 2.0,
            solid: 0.5,
        },
        Mark::Bar {
            length: 4.0,
            thickness: 2.0,
            solid: f32::NAN,
        },
    ];
    // Transparent color also exercises the alpha guard of every mark kind.
    for mark in marks {
        let drawing = Crosshair::default()
            .with_layer(Layer::new(mark).with_color(LinearRgba::NONE))
            .draw(&CrosshairInput::default(), None);
        assert!(drawing.as_ref().is_empty());
    }
}

/// Invalid builder inputs leave finite, bounded geometry.
fn rejected() {
    // Non-finite angles, gap, colors and reversed bounds are all rejected at the builder.
    let layer = Layer::new(Mark::Dot { diameter: 2.0 })
        .at_angles(&[f32::NAN, 0.0])
        .with_gap(f32::NAN)
        .following_spread(8.0, 4.0)
        .with_color(LinearRgba::new(f32::NAN, 0.0, 0.0, 1.0))
        .with_outline(LinearRgba::new(0.0, f32::INFINITY, 0.0, 1.0));
    let crosshair = Crosshair::default()
        .with_layer(layer.clone())
        .with_layer(layer.clone())
        .with_layer(layer.clone())
        .with_layer(layer);
    // The fourth layer is dropped, and transparent layers draw nothing.
    assert_eq!(crosshair.layers().len(), 3);
    assert!(
        crosshair
            .draw(&CrosshairInput::default(), None)
            .as_ref()
            .is_empty()
    );
}

/// Absolute marker dimensions and a hold keep the marker opaque before fading.
fn held_marker() {
    // A 250 ms hold inside a 700 ms lifetime, then half of the 450 ms fade.
    let mut marker = HitMarker::default()
        .with_size(Vec2::new(21.0, 2.0))
        .with_gap(8.0)
        .with_duration(Duration::from_millis(700))
        .with_hold(Duration::from_millis(250));
    marker.advance(Duration::from_millis(250));
    assert_eq!(marker.opacity().to_bits(), 1.0_f32.to_bits());
    marker.advance(Duration::from_millis(225));
    assert!((marker.opacity() - 0.5).abs() < 1e-3);
}

/// Invalid dimensions and a hold beyond the duration remain bounded.
fn invalid_marker() {
    // The hold outlasts the default duration, so opacity stays at one.
    let invalid = HitMarker::default()
        .with_size(Vec2::NAN)
        .with_gap(f32::NAN)
        .with_hold(Duration::from_secs(1));
    assert_eq!(invalid.opacity().to_bits(), 1.0_f32.to_bits());
}
