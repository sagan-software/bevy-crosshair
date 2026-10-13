//! Headless two-layer crosshair: a static outlined center dot and four fading
//! wires that follow weapon spread. The example prints the generated primitives
//! at rest and at full spread, so the effect of the spread clamp is visible in
//! the wire positions. It needs no window, rendering backend, or asset files.

use bevy::prelude::*;
use bevy_crosshair::Crosshair;
use bevy_crosshair::CrosshairInput;
use bevy_crosshair::Layer;
use bevy_crosshair::Mark;
use bevy_crosshair::Spectator;
use bevy_crosshair::WeaponState;

/// Builds the crosshair and prints its geometry for two spread values.
fn main() {
    // The static layer stays on the aiming point; its outline paints first.
    let dot = Layer::new(Mark::Dot { diameter: 2.0 })
        .with_color(LinearRgba::WHITE)
        .with_outline(LinearRgba::BLACK);
    // The dynamic layer starts its wires at the spread, clamped to 4..=64 authored pixels.
    let wires = Layer::new(Mark::Bar {
        length: 17.0,
        thickness: 2.0,
        solid: 0.3,
    })
    .at_angles(&[0.0, 90.0, 180.0, 270.0])
    .following_spread(4.0, 64.0)
    .with_color(LinearRgba::WHITE.with_alpha(0.8));
    let crosshair = Crosshair::default().with_layer(wires).with_layer(dot);
    for spread in [0.0, 40.0] {
        // Supply every host field so only the spread differs between the two drawings.
        let input = CrosshairInput {
            viewport: Vec2::new(1920.0, 1080.0),
            spread,
            shot: 0.0,
            recoil: Vec2::ZERO,
            weapon: WeaponState::Hip,
            is_friendly: false,
            spectator: Spectator::Local,
        };
        let drawing = crosshair.draw(&input, None);
        let count = drawing.as_ref().len();
        println!("spread {spread} px: {count} primitives");
        for primitive in drawing.as_ref() {
            println!("  {primitive:?}");
        }
    }
}
