//! Bevy registration for typed hit delivery and marker timing.

use crate::Crosshair;
use crate::Hit;
use crate::HitFeedback;
use crate::HitMarker;
use bevy::prelude::*;
use std::marker::PhantomData;

/// Registers hit feedback for one metadata type. Install multiple
/// instantiations to accept
/// different game events; shared marker timing and rendering are installed only
/// once.
///
/// ```
/// use bevy::prelude::*;
/// use bevy_crosshair::CrosshairPlugin;
/// use bevy_crosshair::HitMarker;
/// App::new().add_plugins(MinimalPlugins)
///     .add_plugins(CrosshairPlugin::<HitMarker>::default());
/// ```
#[derive(Clone, Copy, Debug)]
pub struct CrosshairPlugin<T: HitFeedback = HitMarker>(PhantomData<fn() -> T>);

impl<T: HitFeedback> Default for CrosshairPlugin<T> {
    fn default() -> Self {
        Self(PhantomData)
    }
}

impl<T: HitFeedback> Plugin for CrosshairPlugin<T> {
    fn build(&self, app: &mut App) {
        // Register shared timing only once when multiple metadata types are installed.
        // Observe typed hits independently of audio, score, or analytics observers.
        // Ignore targets without Crosshair and replace only the addressed marker.
        if !app.is_plugin_added::<MarkerPlugin>() {
            app.add_plugins(MarkerPlugin);
        }
        app.add_observer(receive::<T>);
    }
}

/// Shared marker timing is installed once, regardless of metadata type count.
#[derive(Clone, Copy, Debug)]
struct MarkerPlugin;

impl Plugin for MarkerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Time>().add_systems(PreUpdate, tick);
        #[cfg(feature = "render")]
        if app.world().contains_resource::<AssetServer>() {
            crate::render::install(app);
        }
    }
}

/// A missing target is a valid outcome after the host despawns an actor.
fn receive<T: HitFeedback>(
    hit: On<'_, '_, Hit<T>>,
    mut crosshairs: Query<'_, '_, &mut HitMarker, With<Crosshair>>,
) {
    if let Ok(mut marker) = crosshairs.get_mut(hit.entity) {
        *marker = hit.metadata.marker();
    }
}

/// Keeps expired markers in place to avoid structural ECS changes for every
/// fade.
fn tick(time: Res<'_, Time>, mut markers: Query<'_, '_, &mut HitMarker>) {
    markers
        .iter_mut()
        .filter(|marker| marker.opacity() > 0.0)
        .for_each(|mut marker| marker.advance(time.delta()));
}
