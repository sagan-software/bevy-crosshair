//! Native and browser playground for the public crosshair plugin. The
//! application supplies weapon state and typed damage metadata, while the
//! plugin owns marker timing and rendering. Browser controls update the same
//! components used by native applications, and reported frame intervals
//! originate in the Bevy update schedule.

#[path = "playground/app.rs"]
mod app;
#[cfg(target_arch = "wasm32")]
#[path = "playground/browser.rs"]
mod browser;
#[path = "playground/motion.rs"]
mod motion;

/// Starts Bevy with only the rendering capabilities selected in Cargo.toml.
fn main() {
    app::run();
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
