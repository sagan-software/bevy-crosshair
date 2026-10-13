//! Public regressions for custom layers and absolute hit-marker dimensions.
//! A 1080-pixel viewport keeps one authored pixel equal to one physical pixel,
//! except where a test doubles the height to check screen-height scaling.
//! Each test imports only the crate root, as a downstream game would, and
//! checks one mark, builder bound, or marker timing rule through public
//! geometry.

use bevy::prelude::*;
use bevy_crosshair::Crosshair;
use bevy_crosshair::CrosshairInput;
use bevy_crosshair::HitMarker;
use bevy_crosshair::Layer;
use bevy_crosshair::Mark;
use bevy_crosshair::Primitive;
use bevy_crosshair::Spectator;
use bevy_crosshair::WeaponState;
use std::time::Duration;

/// Draws one crosshair with the given physical spread and viewport height.
fn draw(crosshair: &Crosshair, spread: f32, height: f32) -> Vec<Primitive> {
    // Every other host field keeps the plugin's ordinary hip-fire defaults.
    let input = CrosshairInput {
        viewport: Vec2::new(1920.0, height),
        spread,
        shot: 0.0,
        recoil: Vec2::ZERO,
        weapon: WeaponState::Hip,
        is_friendly: false,
        spectator: Spectator::Local,
    };
    crosshair.draw(&input, None).as_ref().to_vec()
}

/// Returns true when two floats agree within a thousandth of a pixel.
fn is_near(actual: f32, expected: f32) -> bool {
    (actual - expected).abs() < 1e-3
}

/// A right-pointing fading bar that starts at gap 1 and follows spread 4..=10.
fn spread_bar() -> Crosshair {
    let wires = Layer::new(Mark::Bar {
        length: 10.0,
        thickness: 2.0,
        solid: 0.25,
    })
    .at_angles(&[90.0])
    .with_gap(1.0)
    .following_spread(4.0, 10.0);
    Crosshair::default().with_layer(wires)
}

/// Returns the center X and size of the single fading bar in a drawing.
fn fading_bar(primitives: &[Primitive]) -> (f32, Vec2) {
    match primitives {
        [Primitive::FadingRectangle { center, size, .. }] => (center.x, *size),
        other => panic!("one fading bar expected, got {other:?}"),
    }
}

#[test]
fn a_layer_replaces_the_console_style_fill_and_dot() {
    // Fill and the dot flag would add primitives if the console style still drew.
    let mut crosshair = Crosshair::default()
        .with_fill(LinearRgba::BLUE)
        .with_layer(Layer::new(Mark::Dot { diameter: 2.0 }));
    crosshair.set("cl_crosshairdot", "1").unwrap();
    let primitives = draw(&crosshair, 0.0, 1080.0);
    // Only the layer's centered two-pixel dot remains.
    assert!(
        matches!(primitives.as_slice(), [Primitive::Ring { center, radius, thickness, .. }]
        if *center == Vec2::ZERO && is_near(*radius, 1.0) && is_near(*thickness, 2.0))
    );
}

#[test]
fn console_export_excludes_layers() {
    // Layers are an extension, so the exported console text matches a plain crosshair.
    let layered = Crosshair::default().with_layer(Layer::new(Mark::Dot { diameter: 2.0 }));
    assert_eq!(layered.to_string(), Crosshair::default().to_string());
}

#[test]
fn bars_use_the_minimum_spread_below_it() {
    // Zero spread clamps to four authored pixels, after the one-pixel gap.
    let (center, size) = fading_bar(&draw(&spread_bar(), 0.0, 1080.0));
    assert!(is_near(center, 10.0));
    assert_eq!(size, Vec2::new(10.0, 2.0));
}

#[test]
fn bars_follow_spread_inside_the_range() {
    // Seven pixels of spread plus the gap puts the inner end at eight pixels.
    let (center, _) = fading_bar(&draw(&spread_bar(), 7.0, 1080.0));
    assert!(is_near(center, 13.0));
}

#[test]
fn bars_use_the_maximum_spread_above_it() {
    // Fifty pixels of spread clamps to ten authored pixels.
    let (center, _) = fading_bar(&draw(&spread_bar(), 50.0, 1080.0));
    assert!(is_near(center, 16.0));
}

#[test]
fn layers_scale_with_screen_height() {
    // A doubled viewport doubles every authored measurement and the spread bounds.
    let (center, size) = fading_bar(&draw(&spread_bar(), 0.0, 2160.0));
    assert!(is_near(center, 20.0));
    assert_eq!(size, Vec2::new(20.0, 4.0));
}

#[test]
fn bars_keep_their_fade_and_rotation() {
    // A right-pointing bar has no rotation and keeps its solid fraction.
    let primitives = draw(&spread_bar(), 0.0, 1080.0);
    assert!(
        matches!(primitives.as_slice(), [Primitive::FadingRectangle { rotation, solid, .. }]
        if is_near(*rotation, 0.0) && is_near(*solid, 0.25))
    );
}

#[test]
fn a_solid_bar_is_a_plain_rectangle_pointing_up() {
    // A non-finite solid fraction falls back to a solid bar.
    let bar = Mark::Bar {
        length: 6.0,
        thickness: 2.0,
        solid: f32::NAN,
    };
    let crosshair = Crosshair::default().with_layer(Layer::new(bar).with_gap(2.0));
    // The default angle points up, so the bar center lies five pixels above the center.
    let primitives = draw(&crosshair, 0.0, 1080.0);
    assert!(
        matches!(primitives.as_slice(), [Primitive::Rectangle { center, size, .. }]
        if is_near(center.x, 0.0) && is_near(center.y, -5.0) && *size == Vec2::new(6.0, 2.0))
    );
}

#[test]
fn outlines_paint_before_every_foreground_mark() {
    // Two outlined dots above and below the center.
    let dots = Layer::new(Mark::Dot { diameter: 2.0 })
        .at_angles(&[0.0, 180.0])
        .with_gap(4.0)
        .with_color(LinearRgba::WHITE)
        .with_outline(LinearRgba::BLACK);
    let primitives = draw(&Crosshair::default().with_layer(dots), 0.0, 1080.0);
    // Both black outlines precede both white foregrounds.
    let order: Vec<(f32, f32)> = primitives
        .iter()
        .map(|primitive| match primitive {
            Primitive::Ring { radius, color, .. } => (*radius, color.red),
            other => panic!("unexpected {other:?}"),
        })
        .collect();
    assert_eq!(order, [(2.0, 0.0), (2.0, 0.0), (1.0, 1.0), (1.0, 1.0)]);
}

#[test]
fn chevron_arms_meet_at_an_outward_apex() {
    // Each arm runs from (+-4, 0) to the apex (0, -3): length 5, plus the thickness overlap.
    let chevron = Mark::Chevron {
        width: 8.0,
        depth: 3.0,
        thickness: 2.0,
    };
    let primitives = draw(
        &Crosshair::default().with_layer(Layer::new(chevron)),
        0.0,
        1080.0,
    );
    // The arm centers sit halfway between each end and the apex.
    assert!(matches!(primitives.as_slice(), [
        Primitive::Rectangle { center: left, size, .. },
        Primitive::Rectangle { center: right, .. },
    ] if is_near(left.x, -2.0) && is_near(left.y, -1.5) && is_near(right.x, 2.0)
        && *size == Vec2::new(7.0, 2.0)));
}

#[test]
fn arcs_bulge_outward_from_their_inner_edge() {
    // A left-pointing arc whose inner edge midpoint sits five pixels left of the center.
    let arc = Mark::Arc {
        radius: 9.0,
        sweep: 90.0,
        thickness: 2.0,
    };
    let layer = Layer::new(arc).at_angles(&[270.0]).with_gap(5.0);
    let primitives = draw(&Crosshair::default().with_layer(layer), 0.0, 1080.0);
    // The arc's own circle center lies seven pixels right of that inner edge.
    assert!(matches!(primitives.as_slice(), [Primitive::Arc {
        center, radius, thickness, direction, half_sweep, ..
    }] if is_near(center.x, 2.0) && is_near(center.y, 0.0) && is_near(*radius, 9.0)
        && is_near(*thickness, 2.0) && is_near(*direction, 270f32.to_radians())
        && is_near(*half_sweep, 45f32.to_radians())));
}

#[test]
fn rings_ignore_angles_and_grow_with_spread() {
    // Two angles still draw one ring, grown by thirty pixels of spread.
    let ring = Layer::new(Mark::Ring {
        radius: 1.0,
        thickness: 1.0,
    })
    .at_angles(&[0.0, 90.0])
    .following_spread(4.0, 100.0);
    let primitives = draw(&Crosshair::default().with_layer(ring), 30.0, 1080.0);
    assert!(
        matches!(primitives.as_slice(), [Primitive::Ring { radius, .. }] if is_near(*radius, 31.0))
    );
}

#[test]
fn builders_reject_invalid_inputs() {
    // Four finite angles remain; the infinite gap and reversed bounds are rejected.
    let layer = Layer::new(Mark::Dot { diameter: 2.0 })
        .at_angles(&[0.0, f32::NAN, 90.0, 180.0, 270.0, 45.0])
        .with_gap(f32::INFINITY)
        .following_spread(10.0, 4.0);
    let primitives = draw(&Crosshair::default().with_layer(layer), 50.0, 1080.0);
    // Without gap or spread every dot stays on the center.
    assert_eq!(primitives.len(), 4);
    assert!(primitives.iter().all(
        |primitive| matches!(primitive, Primitive::Ring { center, .. } if center.length() < 1e-3)
    ));
}

#[test]
fn crosshairs_keep_at_most_three_layers() {
    // The fourth layer is ignored to keep the drawing within its fixed capacity.
    let layer = Layer::new(Mark::Dot { diameter: 2.0 });
    let crosshair = Crosshair::default()
        .with_layer(layer.clone())
        .with_layer(layer.clone())
        .with_layer(layer.clone())
        .with_layer(layer);
    assert_eq!(crosshair.layers().len(), 3);
}

#[test]
fn transparent_and_empty_layers_draw_nothing() {
    // A NaN color becomes transparent; an empty angle list has no marks.
    let transparent = Layer::new(Mark::Dot { diameter: 2.0 }).with_color(LinearRgba::new(
        f32::NAN,
        0.0,
        0.0,
        1.0,
    ));
    let empty = Layer::new(Mark::Dot { diameter: 2.0 }).at_angles(&[]);
    let crosshair = Crosshair::default()
        .with_layer(transparent)
        .with_layer(empty);
    assert!(draw(&crosshair, 0.0, 1080.0).is_empty());
}

#[test]
fn a_non_finite_arc_sweep_draws_nothing() {
    // Validation drops the arc before it reaches the GPU.
    let arc = Mark::Arc {
        radius: 9.0,
        sweep: f32::NAN,
        thickness: 2.0,
    };
    assert!(
        draw(
            &Crosshair::default().with_layer(Layer::new(arc)),
            0.0,
            1080.0
        )
        .is_empty()
    );
}

#[test]
fn a_zero_ring_draws_nothing() {
    // A ring needs positive radius and thickness.
    let ring = Mark::Ring {
        radius: 0.0,
        thickness: 0.0,
    };
    assert!(
        draw(
            &Crosshair::default().with_layer(Layer::new(ring)),
            0.0,
            1080.0
        )
        .is_empty()
    );
}

#[test]
fn a_zero_length_bar_draws_nothing() {
    // A fading bar needs a positive length.
    let bar = Mark::Bar {
        length: 0.0,
        thickness: 2.0,
        solid: 0.5,
    };
    assert!(
        draw(
            &Crosshair::default().with_layer(Layer::new(bar)),
            0.0,
            1080.0
        )
        .is_empty()
    );
}

/// A hidden crosshair, so drawings contain only hit-marker geometry.
fn hidden_crosshair() -> Crosshair {
    let mut crosshair = Crosshair::default();
    crosshair.set("crosshair", "0").unwrap();
    crosshair
}

#[test]
fn absolute_marker_dimensions_place_each_arm() {
    // Each diagonal arm's center lies half its 21-pixel length beyond the 8-pixel gap.
    let marker = HitMarker::default()
        .with_size(Vec2::new(21.0, 2.0))
        .with_gap(8.0);
    let drawing = hidden_crosshair().draw(&CrosshairInput::default(), Some(&marker));
    assert!(drawing.as_ref().iter().all(|primitive| matches!(primitive,
        Primitive::Rectangle { center, size, .. }
        if is_near(center.length(), 18.5) && *size == Vec2::new(21.0, 2.0))));
}

#[test]
fn invalid_marker_dimensions_hide_the_marker() {
    // Non-finite size and gap become zero, so the marker draws nothing.
    let hidden = HitMarker::default().with_size(Vec2::NAN).with_gap(f32::NAN);
    let drawing = hidden_crosshair().draw(&CrosshairInput::default(), Some(&hidden));
    assert!(drawing.as_ref().is_empty());
}

#[test]
fn a_hold_keeps_full_opacity_before_the_fade() {
    // A 250 ms hold inside a 700 ms lifetime leaves 450 ms of linear fade.
    let mut marker = HitMarker::default()
        .with_duration(Duration::from_millis(700))
        .with_hold(Duration::from_millis(250));
    marker.advance(Duration::from_millis(250));
    assert!(is_near(marker.opacity(), 1.0));
    // Half of the fade later the opacity is one half.
    marker.advance(Duration::from_millis(225));
    assert!(is_near(marker.opacity(), 0.5));
}

#[test]
fn a_hold_past_the_duration_stays_opaque() {
    // The animation never starts, so the marker stays opaque until it expires.
    let mut held = HitMarker::default().with_hold(Duration::from_secs(1));
    held.advance(Duration::from_millis(100));
    assert!(is_near(held.opacity(), 1.0));
}
