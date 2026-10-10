//! Typed hit events shared with independent game systems.

use crate::HitFeedback;
use bevy::prelude::*;

/// A hit addressed to one crosshair, retaining the game's metadata type.
/// Independent
/// observers receive the same event even when its target has no visible marker.
#[derive(EntityEvent, Debug, Reflect)]
pub struct Hit<T: HitFeedback> {
    /// Crosshair entity receiving the marker. A missing or unrelated entity is
    /// ignored by the
    /// rendering observer while other event observers still receive the metadata.
    pub entity: Entity,
    /// Metadata also available to independent event observers. The plugin borrows
    /// this value
    /// through HitFeedback and stores only the resulting marker on the target
    /// entity.
    pub metadata: T,
}
