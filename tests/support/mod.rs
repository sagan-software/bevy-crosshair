//! Public conformance scenarios shared by tests and instrumented benchmarks.

use bevy::prelude::*;
use bevy::time::TimeUpdateStrategy;
use bevy_crosshair::Crosshair;
use bevy_crosshair::CrosshairInput;
use bevy_crosshair::CrosshairPlugin;
use bevy_crosshair::Grenade;
use bevy_crosshair::Hit;
use bevy_crosshair::HitFeedback;
use bevy_crosshair::HitMarker;
use bevy_crosshair::Primitive;
use bevy_crosshair::Spectator;
use bevy_crosshair::Style;
use bevy_crosshair::WeaponState;
use std::time::Duration;

mod configuration;
pub(crate) use self::configuration::run as configuration;
mod geometry;
pub(crate) use self::geometry::run as geometry;
mod policies;
pub(crate) use self::policies::run as policies;
mod markers;
pub(crate) use self::markers::run as markers;
mod events;
pub(crate) use self::events::run as events;
#[cfg(feature = "playground")]
mod rendering;
#[cfg(feature = "playground")]
pub(crate) use self::rendering::run as rendering;

mod reflection;
pub(crate) use self::reflection::run as reflection;
mod extensions;
pub(crate) use self::extensions::run as extensions;
mod layers;
pub(crate) use self::layers::layered;
pub(crate) use self::layers::run as layers;
