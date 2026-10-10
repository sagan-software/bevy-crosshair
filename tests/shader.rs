//! Shader syntax and validation checks for the fragment program used by the
//! Bevy material. The test replaces only the external WESL vertex-output import
//! with its interface declaration before invoking Naga. Actual shader linking
//! and GPU pipeline creation are checked separately by the offscreen rendering
//! conformance case.

#[test]
fn crosshair_shader_validates_with_its_ui_vertex_contract() {
    let interface = "struct UiVertexOutput { @builtin(position) position: vec4<f32>, @location(0) uv: vec2<f32>, };";
    let shader = include_str!("../src/crosshair.wesl").replacen(
        "import bevy_ui_render::ui_vertex_output::UiVertexOutput;",
        interface,
        1,
    );
    let module = naga::front::wgsl::parse_str(&shader).expect("the fixture supplies a valid value");
    naga::valid::Validator::new(
        naga::valid::ValidationFlags::all(),
        naga::valid::Capabilities::empty(),
    )
    .validate(&module)
    .expect("the fixture supplies a valid value");
}

use arrayvec as _;

use thiserror as _;

use strum as _;

#[cfg(not(target_arch = "wasm32"))]
use criterion as _;

#[cfg(not(target_arch = "wasm32"))]
use stats_alloc as _;

use bevy as _;
use bevy_crosshair as _;
