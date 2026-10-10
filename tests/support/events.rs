//! Typed delivery, target isolation, replacement, and shared marker timing.

use super::*;

/// Metadata deliberately lacks Reflect to keep ordinary event delivery unconstrained.
#[derive(Debug)]
struct Damage(f32);
impl HitFeedback for Damage {
    fn marker(&self) -> HitMarker {
        HitMarker::default().scaled(self.0)
    }
}

/// Counts independent observer deliveries, including events without a live target.
#[derive(Resource, Default, Reflect)]
struct Receipt(usize);

/// Builds deterministic timing with two metadata types sharing the same marker system.
fn application() -> App {
    let mut app = App::new();
    // A fixed interval distinguishes one shared timing system from duplicate registration.
    app.add_plugins(MinimalPlugins)
        .add_plugins((
            CrosshairPlugin::<HitMarker>::default(),
            CrosshairPlugin::<Damage>::default(),
        ))
        .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_millis(
            50,
        )))
        .init_resource::<Receipt>()
        .add_observer(
            |_: On<'_, '_, Hit<Damage>>, mut receipt: ResMut<'_, Receipt>| receipt.0 += 1,
        );
    app
}

/// Compares exact opacity for binary fractions produced by the deterministic clock.
fn opacity(app: &App, entity: Entity, expected: f32) {
    let actual = app
        .world()
        .get::<HitMarker>(entity)
        .expect("required marker exists")
        .opacity();
    assert_eq!(actual.to_bits(), expected.to_bits());
}

/// Checks initial invisibility and delivery to only the addressed crosshair.
fn delivery(app: &mut App, first: Entity, second: Entity) {
    // Initialize time before triggering; a fresh crosshair must have no visible marker.
    app.update();
    opacity(app, first, 0.0);
    app.world_mut().trigger(Hit {
        entity: first,
        metadata: Damage(2.0),
    });
    // The independent observer and marker receive one event without changing another entity.
    opacity(app, first, 1.0);
    opacity(app, second, 0.0);
}

/// Checks replacement through a second metadata type and expiry after two intervals.
fn replacement(app: &mut App, first: Entity) {
    let metadata = HitMarker::default().with_duration(Duration::from_millis(100));
    // Replacement restarts the fade while retaining the shared timing system.
    app.world_mut().trigger(Hit {
        entity: first,
        metadata,
    });
    app.update();
    opacity(app, first, 0.5);
    // Extra updates cannot revive an expired marker.
    app.update();
    app.update();
    opacity(app, first, 0.0);
}

/// Confirms that independent observers still receive unrelated and despawned targets.
fn missing_targets(app: &mut App, first: Entity) {
    let unrelated = app.world_mut().spawn_empty().id();
    // A target without Crosshair is a valid ignored delivery for the visual observer.
    app.world_mut().trigger(Hit {
        entity: unrelated,
        metadata: Damage(1.0),
    });
    app.world_mut().despawn(first);
    // A despawn racing event delivery must not suppress independent metadata observers.
    app.world_mut().trigger(Hit {
        entity: first,
        metadata: Damage(1.0),
    });
    assert_eq!(
        app.world()
            .get_resource::<Receipt>()
            .expect("receipt exists")
            .0,
        3
    );
}

/// Exercises the ordered delivery and lifecycle transitions through the public API.
pub(crate) fn run() {
    let mut app = application();
    // Keep two targets alive while exercising initial delivery and replacement.
    let first = app.world_mut().spawn(Crosshair::default()).id();
    let second = app.world_mut().spawn(Crosshair::default()).id();
    delivery(&mut app, first, second);
    // Despawn only after proving replacement and shared timing on the original target.
    replacement(&mut app, first);
    missing_targets(&mut app, first);
}
