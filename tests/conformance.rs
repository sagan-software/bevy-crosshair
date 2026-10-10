//! External conformance tests for configuration, geometry, hit feedback,
//! reflection, and rendering. Shared scenarios also execute inside the
//! Criterion binary so benchmark line coverage measures the same observable
//! contracts. The optional rendering case initializes a real GPU camera and
//! requires the material pipeline to compile successfully.

mod support;

#[test]
fn configuration() {
    support::configuration();
}
#[test]
fn geometry() {
    support::geometry();
}
#[test]
fn policies() {
    support::policies();
}
#[test]
fn markers() {
    support::markers();
}
#[test]
fn events() {
    support::events();
}

#[test]
#[cfg(feature = "playground")]
fn rendering() {
    drop(support::rendering());
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

/// Reflection preserves validated values and rejects incompatible replacements.
#[test]
fn reflection() {
    support::reflection();
}

#[test]
fn extensions() {
    support::extensions();
}
