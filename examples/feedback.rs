//! Fill, round dots, and game-owned animated hit feedback through the public API.
//! Run this headless guide with `cargo run --example feedback`. The same crosshair
//! and marker values can be stored as Bevy components. Advancing a marker explicitly
//! demonstrates its fade-in without creating a window or dispatching a gameplay event.

use bevy::prelude::*;
use bevy_crosshair::Crosshair;
use bevy_crosshair::CrosshairInput;
use bevy_crosshair::DotShape;
use bevy_crosshair::HitMarker;
use bevy_crosshair::MarkerAnimation;
use bevy_crosshair::MarkerShape;
use bevy_crosshair::Style;
use std::time::Duration;

/// Generates geometry without a window; the same values work as Bevy components.
fn main() -> Result<(), bevy_crosshair::SettingError> {
    // Console settings still enable the dot; the extension chooses its geometry.
    let mut crosshair = Crosshair::default()
        .with_style(Style::StaticQuad)
        .with_fill(LinearRgba::BLUE.with_alpha(0.2))
        .with_dot_shape(DotShape::Circle);
    crosshair.set("cl_crosshairdot", "true")?;
    // Advance to the visible phase before drawing the independently animated marker.
    let mut marker = HitMarker::default()
        .with_shape(MarkerShape::Chevron)
        .with_animation(MarkerAnimation::Pulse)
        .with_duration(Duration::from_millis(500));
    marker.advance(Duration::from_millis(100));
    let drawing = crosshair.draw(&CrosshairInput::default(), Some(&marker));
    let count = drawing.as_ref().len();
    println!("{count} primitives");
    Ok(())
}

use arrayvec as _;
use criterion as _;
use naga as _;
use stats_alloc as _;
use strum as _;
use thiserror as _;
