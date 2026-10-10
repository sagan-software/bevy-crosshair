//! Configurable Bevy crosshairs with typed hit feedback.
//!
//! Spawn [`Crosshair`] with [`CrosshairInput`], then trigger [`Hit`] events.
//! Implement [`HitFeedback`] on your game's metadata to choose marker
//! appearance.
//! Independent observers can use the same metadata for audio or scoring.
//! Enable the render feature after installing Bevy assets to draw through a UI
//! material.
//! Configuration parsing validates current CS2 console names before replacing
//! stored values.
//!
//! ```
//! use bevy::prelude::*;
//! use bevy_crosshair::Crosshair;
//! use bevy_crosshair::CrosshairPlugin;
//! use bevy_crosshair::Hit;
//! use bevy_crosshair::HitMarker;
//!
//! let mut app = App::new();
//! app.add_plugins(MinimalPlugins)
//!     .add_plugins(CrosshairPlugin::<HitMarker>::default());
//! let entity = app.world_mut().spawn(Crosshair::default()).id();
//! app.world_mut().trigger(Hit { entity, metadata: HitMarker::default() });
//! app.update();
//! ```

mod config;
mod crosshair;
mod drawing;
mod geometry;
mod hit;
mod marker;
mod plugin;
#[cfg(feature = "render")]
mod render;
mod settings;

pub use self::config::Grenade;
pub use self::config::SettingError;
pub use self::config::Style;
pub use self::config::WeaponState;
pub use self::crosshair::Crosshair;
pub use self::crosshair::CrosshairInput;
pub use self::crosshair::Spectator;
pub use self::drawing::Drawing;
pub use self::drawing::Primitive;
pub use self::hit::Hit;
pub use self::marker::HitFeedback;
pub use self::marker::HitMarker;
pub use self::plugin::CrosshairPlugin;

#[cfg(test)]
use criterion as _;
#[cfg(test)]
use naga as _;
#[cfg(test)]
use stats_alloc as _;
