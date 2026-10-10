//! Shared markers conformance through the public API.

use super::*;

/// Covers marker replacement values, saturation, color validation, and expiry.
fn lifetime() {
    // Isolate marker geometry from the ordinary crosshair.
    let crosshair: Crosshair = "crosshair false"
        .parse()
        .expect("the fixture supplies a valid value");
    let input = CrosshairInput::default();
    let mut marker = HitMarker::default()
        .with_duration(Duration::from_millis(200))
        .scaled(2.0);
    assert_opacity(marker, 1.0);
    assert_eq!(crosshair.draw(&input, Some(&marker)).as_ref().len(), 4);
    // Fading reaches half opacity before saturating at expiry.
    marker.advance(Duration::from_millis(100));
    assert_opacity(marker, 0.5);
    marker.advance(Duration::MAX);
    assert_opacity(marker, 0.0);
    assert!(crosshair.draw(&input, Some(&marker)).as_ref().is_empty());
    // Reusing an expired marker as feedback restarts its duration.
    assert_opacity(marker.marker(), 1.0);
}

/// Checks marker scaling through emitted geometry.
fn scaling() {
    // Isolate marker geometry from the ordinary crosshair.
    let crosshair: Crosshair = "crosshair false".parse().expect("fixture is valid");
    let input = CrosshairInput::default();
    [0.0, -1.0, f32::NAN, f32::INFINITY]
        .into_iter()
        .for_each(|factor| {
            assert!(
                crosshair
                    .draw(&input, Some(&HitMarker::default().scaled(factor)))
                    .as_ref()
                    .is_empty()
            );
        });
    // Repeated scaling must retain the per-dimension cap.
    let saturated = HitMarker::default().scaled(32.0).scaled(32.0).scaled(32.0);
    assert!(crosshair.draw(&input, Some(&saturated)).as_ref().iter().all(|primitive| {
        matches!(primitive, Primitive::Rectangle { size, .. } if size.max_element() <= 8192.0)
    }));
}

/// Checks marker colors through emitted geometry.
fn colors() {
    // Isolate marker geometry from the ordinary crosshair.
    let crosshair: Crosshair = "crosshair false".parse().expect("fixture is valid");
    let input = CrosshairInput::default();
    for alpha in [-1.0, 2.0] {
        let marker = HitMarker::default().with_color(LinearRgba::WHITE.with_alpha(alpha));
        let drawing = crosshair.draw(&input, Some(&marker));
        assert_eq!(drawing.as_ref().is_empty(), alpha < 0.0);
        assert!(drawing.as_ref().iter().all(|primitive| {
            matches!(primitive, Primitive::Rectangle { color, .. } if (0.0..=1.0).contains(&color.alpha))
        }));
    }
    // Finite, transparent, and invalid colors all pass through the public setter.
    for color in [
        LinearRgba::WHITE,
        LinearRgba::NONE,
        LinearRgba::new(f32::NAN, 0.0, 0.0, 1.0),
    ] {
        std::hint::black_box(crosshair.draw(&input, Some(&HitMarker::default().with_color(color))));
    }
    // Zero duration expires immediately, including before the first update.
    assert_eq!(
        HitMarker::default()
            .with_duration(Duration::ZERO)
            .opacity()
            .to_bits(),
        0.0_f32.to_bits()
    );
}

/// Covers lifetime, bounded scaling, and finite color validation separately.
pub(crate) fn run() {
    lifetime();
    scaling();
    colors();
}

/// Compares exact opacity values for deterministic duration boundaries.
fn assert_opacity(marker: HitMarker, expected: f32) {
    assert_eq!(marker.opacity().to_bits(), expected.to_bits());
}
