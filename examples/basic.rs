//! Minimal headless integration with game-owned damage metadata and an
//! independent observer. The plugin converts each typed hit to a visual marker
//! while the application receives the same damage value for other responses.
//! This example needs no window, rendering backend, audio asset, or custom
//! event bus.

use bevy::prelude::*;
use bevy_crosshair::Crosshair;
use bevy_crosshair::CrosshairPlugin;
use bevy_crosshair::Hit;
use bevy_crosshair::HitFeedback;
use bevy_crosshair::HitMarker;

/// Damage data owned by the game, expressed in its own health-point units.
#[derive(Debug)]
struct Damage(f32);

impl HitFeedback for Damage {
    fn marker(&self) -> HitMarker {
        HitMarker::default().scaled(1.0 + self.0 / 100.0)
    }
}

/// Starts a headless app and demonstrates the same event used by the renderer.
fn main() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(CrosshairPlugin::<Damage>::default())
        .add_observer(|hit: On<'_, '_, Hit<Damage>>| {
            let damage = hit.metadata.0;
            println!("Hit for {damage} health points");
        });
    // The same typed event reaches marker conversion and the independent observer.
    let crosshair = app.world_mut().spawn(Crosshair::default()).id();
    app.world_mut().trigger(Hit {
        entity: crosshair,
        metadata: Damage(40.0),
    });
    app.update();
}

use arrayvec as _;

use thiserror as _;

use strum as _;

#[cfg(not(target_arch = "wasm32"))]
use criterion as _;

#[cfg(not(target_arch = "wasm32"))]
use naga as _;

#[cfg(not(target_arch = "wasm32"))]
use stats_alloc as _;
