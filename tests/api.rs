//! Public integration checks for component construction, console configuration,
//! and typed hit delivery. These tests use only crate-root exports, as a
//! downstream Bevy application would. Independent observers must receive
//! metadata while the plugin updates only the addressed entity and preserves
//! the supplied configuration.

use bevy::prelude::*;
use bevy_crosshair::Crosshair;
use bevy_crosshair::CrosshairPlugin;
use bevy_crosshair::Hit;
use bevy_crosshair::HitFeedback;
use bevy_crosshair::HitMarker;
use bevy_crosshair::Style;

/// User-defined metadata must remain available to independent observers.
#[derive(Debug)]
struct Damage(f32);

impl HitFeedback for Damage {
    fn marker(&self) -> HitMarker {
        HitMarker::default().scaled(self.0 / 20.0)
    }
}

/// Records observer delivery without coupling it to rendering.
#[derive(Resource, Default, Reflect)]
struct Receipt(f32);

#[test]
fn hit_reaches_crosshair_and_independent_observer() {
    // Register a game observer independently of the plugin observer.
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(CrosshairPlugin::<Damage>::default())
        .init_resource::<Receipt>()
        .add_observer(
            |hit: On<'_, '_, Hit<Damage>>, mut receipt: ResMut<'_, Receipt>| {
                receipt.0 = hit.metadata.0;
            },
        );
    // One event must reach both observers without consuming the metadata.
    let target = app.world_mut().spawn(Crosshair::default()).id();
    app.world_mut().trigger(Hit {
        entity: target,
        metadata: Damage(40.0),
    });
    assert_eq!(
        app.world()
            .get_resource::<Receipt>()
            .expect("receipt resource exists")
            .0
            .to_bits(),
        40.0_f32.to_bits()
    );
    assert!(app.world().get::<HitMarker>(target).is_some());
}

#[test]
fn console_configuration_uses_current_cs2_names() {
    let crosshair: Crosshair = "cl_crosshairstyle 4; cl_crosshair_length 12"
        .parse()
        .expect("the fixture supplies a valid value");
    assert_eq!(crosshair.style(), Style::StaticCross);
    assert_eq!(
        crosshair
            .get("cl_crosshair_length")
            .expect("the fixture supplies a valid value"),
        "12"
    );
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
