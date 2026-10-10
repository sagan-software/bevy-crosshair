//! Public regressions for fill, round dots, and animated hit feedback.
//! These tests import only the crate root, matching a downstream Bevy application.
//! Geometry assertions distinguish fill opacity from dot dimensions, while console
//! export remains unchanged. A controlled application clock exercises the first
//! transparent frame of an animated hit.

use bevy::prelude::*;
use bevy_crosshair::Crosshair;
use bevy_crosshair::CrosshairInput;
use bevy_crosshair::CrosshairPlugin;
use bevy_crosshair::DotShape;
use bevy_crosshair::Hit;
use bevy_crosshair::HitMarker;
use bevy_crosshair::MarkerAnimation;
use bevy_crosshair::MarkerShape;
use bevy_crosshair::Primitive;
use std::time::Duration;

#[test]
fn fill_and_round_dot_use_the_public_geometry_api() {
    // Fill is painted first; the circular center dot is painted last.
    let crosshair = configured_crosshair()
        .with_fill(LinearRgba::BLUE.with_alpha(0.25))
        .with_dot_shape(DotShape::Circle);
    let drawing = crosshair.draw(&CrosshairInput::default(), None);
    assert!(
        matches!(drawing.as_ref().first(), Some(Primitive::Ring { radius, color, .. })
        if radius.to_bits() == 32.0_f32.to_bits() && color.alpha.to_bits() == 0.25_f32.to_bits())
    );
    assert!(
        matches!(drawing.as_ref().last(), Some(Primitive::Ring { radius, thickness, .. })
        if radius.to_bits() == 1.0_f32.to_bits() && thickness.to_bits() == 2.0_f32.to_bits())
    );
}

#[test]
fn pulse_appears_then_expires() {
    // A complete lifetime must cross both transparent endpoints and its visible phase.
    let mut marker = HitMarker::default()
        .with_shape(MarkerShape::Ring)
        .with_animation(MarkerAnimation::Pulse)
        .with_duration(Duration::from_secs(1));
    assert_eq!(marker.opacity().to_bits(), 0.0_f32.to_bits());
    marker.advance(Duration::from_millis(200));
    assert_eq!(marker.opacity().to_bits(), 1.0_f32.to_bits());
    // A long frame must expire the marker without overflow.
    marker.advance(Duration::MAX);
    assert_eq!(marker.opacity().to_bits(), 0.0_f32.to_bits());
}

#[test]
fn animated_hit_advances_from_its_initial_transparent_frame() {
    // A fresh animated hit is transparent, but its lifetime still advances.
    let (mut app, entity) = application();
    app.world_mut().trigger(Hit {
        entity,
        metadata: HitMarker::default()
            .with_animation(MarkerAnimation::Pulse)
            .with_duration(Duration::from_millis(500)),
    });
    // The next fixed 100ms step reaches one fifth of the 500ms lifetime.
    app.update();
    let marker = app.world().get::<HitMarker>(entity).expect("marker exists");
    assert_eq!(marker.opacity().to_bits(), 1.0_f32.to_bits());
}

/// Initializes the clock before measuring an event's first update.
fn application() -> (App, Entity) {
    // Manual duration removes wall-clock scheduling from the regression.
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(CrosshairPlugin::<HitMarker>::default())
        .insert_resource(bevy::time::TimeUpdateStrategy::ManualDuration(
            Duration::from_millis(100),
        ));
    let entity = app.world_mut().spawn(Crosshair::default()).id();
    app.update();
    (app, entity)
}

#[test]
fn extensions_preserve_console_export() {
    let base = configured_crosshair();
    let exported = base.to_string();
    let extended = base
        .with_fill(LinearRgba::BLUE)
        .with_dot_shape(DotShape::Circle);
    assert_eq!(exported, extended.to_string());
}

/// Supplies a visible center dot and a nonzero interior without outline geometry.
fn configured_crosshair() -> Crosshair {
    "cl_crosshairstyle 9;cl_crosshair_gap 32;cl_crosshairdot true;cl_crosshair_drawoutline 0"
        .parse()
        .expect("valid fixture")
}

use arrayvec as _;
use criterion as _;
use naga as _;
use stats_alloc as _;
use strum as _;
use thiserror as _;
