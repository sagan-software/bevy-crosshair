//! Exact color fidelity through public geometry, including updates and rejection.
//! Every byte value appears in each foreground and outline channel. Expected
//! primitives use Bevy's direct conversion independently of the plugin. Repeated
//! updates detect stale colors, rejected assignments must preserve the previous
//! values, and transparent shapes must retain their omission and paint order.

use bevy::color::ColorToComponents;
use bevy::color::LinearRgba;
use bevy::color::Srgba;
use bevy::math::Vec2;
use bevy_crosshair::Crosshair;
use bevy_crosshair::CrosshairInput;
use bevy_crosshair::Primitive;
use bevy_crosshair::Style;

/// Sweeps every byte in each position with independently changing foreground and outline.
#[test]
fn all_byte_colors_match_bevy_exactly_after_each_update() {
    let mut crosshair = Crosshair::default().with_style(Style::Dot);
    // Different wrapping offsets cover every value in all eight console channels.
    for value in 0..=u8::MAX {
        let foreground = [
            value,
            value.wrapping_add(67),
            value.wrapping_add(131),
            value,
        ];
        let outline = [
            value.wrapping_add(193),
            value.wrapping_add(17),
            value.wrapping_add(89),
            u8::MAX - value,
        ];
        set_color(&mut crosshair, "cl_crosshaircolor", foreground);
        set_color(&mut crosshair, "cl_crosshairoutline", outline);
        assert_dot(&crosshair, foreground, outline);
        // Invalid updates must preserve both console values and the next rendered color.
        assert_rejected_updates(&mut crosshair);
        assert_dot(&crosshair, foreground, outline);
    }
}

/// Checks rejection without rebuilding the component between failed assignments.
fn assert_rejected_updates(crosshair: &mut Crosshair) {
    // Preserve the current value independently for each foreground and outline channel.
    for prefix in ["cl_crosshaircolor", "cl_crosshairoutline"] {
        for channel in ["r", "g", "b", "a"] {
            let name = format!("{prefix}_{channel}");
            let original = crosshair.get(&name).expect("known channel");
            // Negative, overflowing, and nonnumeric input must leave the stored byte untouched.
            for invalid in ["-1", "256", "NaN"] {
                assert!(crosshair.set(&name, invalid).is_err());
                assert_eq!(crosshair.get(&name).expect("known channel"), original);
            }
        }
    }
}

/// Updates the same component repeatedly so stale derived colors cannot pass.
fn set_color(crosshair: &mut Crosshair, prefix: &str, bytes: [u8; 4]) {
    for (channel, value) in ["r", "g", "b", "a"].into_iter().zip(bytes) {
        crosshair
            .set(&format!("{prefix}_{channel}"), &value.to_string())
            .expect("valid byte");
    }
}

/// Compares full primitives, paint order, alpha omission, and floating-point bits.
fn assert_dot(crosshair: &Crosshair, foreground: [u8; 4], outline: [u8; 4]) {
    // Construct the outlined square independently, omitting only fully transparent primitives.
    let mut expected = Vec::new();
    for (bytes, size) in [(outline, 4.0), (foreground, 2.0)] {
        let [red, green, blue, alpha] = bytes;
        if alpha > 0 {
            expected.push(Primitive::Rectangle {
                center: Vec2::ZERO,
                size: Vec2::splat(size),
                rotation: 0.0,
                color: Srgba::rgba_u8(red, green, blue, alpha).into(),
            });
        }
    }
    let drawing = crosshair.draw(&CrosshairInput::default(), None);
    // Geometry equality checks paint order; bit comparisons also preserve signed zero.
    assert_eq!(drawing.as_ref(), expected);
    for (actual, expected) in drawing.as_ref().iter().zip(&expected) {
        assert_eq!(color_bits(actual), color_bits(expected));
    }
}

/// Preserves exact component bits, including zero, rather than approximate color equality.
fn color_bits(primitive: &Primitive) -> [u32; 4] {
    let color: &LinearRgba = match primitive {
        Primitive::Rectangle { color, .. }
        | Primitive::Ring { color, .. }
        | Primitive::Arc { color, .. }
        | Primitive::FadingRectangle { color, .. } => color,
    };
    color.to_vec4().to_array().map(f32::to_bits)
}

use arrayvec as _;
use criterion as _;
use naga as _;
use stats_alloc as _;
use strum as _;
use thiserror as _;
