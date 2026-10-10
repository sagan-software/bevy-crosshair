//! Shared geometry conformance through the public API.

use super::*;

/// Exercises every style with static, dynamic, outline, and empty geometry.
fn styles() {
    let styles = 0..10;
    // Every style uses the same outline vocabulary.
    for code in styles {
        for outline in ["0", "1", "2"] {
            let mut crosshair = Crosshair::default()
                .with_style(Style::try_from(code).expect("the fixture supplies a valid value"));
            crosshair
                .set("cl_crosshair_drawoutline", outline)
                .expect("the fixture supplies a valid value");
            // Probe the split boundary and the spread cap with the same style.
            for spread in [0.0, 3.0, 4.0, 48.0, 512.0] {
                let input = CrosshairInput {
                    spread,
                    shot: spread,
                    recoil: Vec2::new(1.0, -2.0),
                    ..default()
                };
                assert!(!crosshair.draw(&input, None).as_ref().is_empty());
            }
            // Apply each modifier cumulatively to reach empty and transparent paths.
            for (name, value) in [
                ("cl_crosshair_t", "true"),
                ("cl_crosshairdot", "true"),
                ("cl_crosshair_recoil", "false"),
                ("cl_crosshair_length", "0"),
                ("cl_crosshair_thickness", "0"),
                ("cl_crosshair_gap", "-3840"),
                ("cl_crosshair_dynamic_maxdist_splitratio", "0.5"),
                ("cl_crosshair_dynamic_splitalpha_innermod", "1"),
                ("cl_crosshaircolor_a", "0"),
                ("cl_crosshairoutline_a", "0"),
            ] {
                crosshair
                    .set(name, value)
                    .expect("the fixture supplies a valid value");
                std::hint::black_box(crosshair.draw(
                    &CrosshairInput {
                        spread: 40.0,
                        ..default()
                    },
                    None,
                ));
            }
        }
    }
}

/// Checks exact coordinates and scaling for a static cross.
fn static_geometry() {
    let crosshair: Crosshair = "cl_crosshairstyle 4;cl_crosshair_drawoutline 0"
        .parse()
        .expect("the fixture supplies a valid value");
    // Static geometry ignores spread and retains exact physical coordinates.
    let drawing = crosshair.draw(&CrosshairInput::default(), None);
    assert_eq!(drawing.as_ref().len(), 4);
    assert!(
        matches!(drawing.as_ref().first(), Some(Primitive::Rectangle { center, size, .. })
        if *center == Vec2::new(8.0, 0.0) && *size == Vec2::new(8.0, 2.0))
    );
    // Static styles retain their coordinates when weapon spread changes.
    let moved = crosshair.draw(
        &CrosshairInput {
            spread: 50.0,
            ..default()
        },
        None,
    );
    assert_eq!(drawing.as_ref(), moved.as_ref());
    scaled_geometry(&crosshair);
}

/// Checks viewport scaling and global visibility independently.
fn scaled_geometry(crosshair: &Crosshair) {
    // Doubling viewport height doubles every authored measurement.
    let scaled = crosshair.draw(
        &CrosshairInput {
            viewport: Vec2::new(3840.0, 2160.0),
            ..default()
        },
        None,
    );
    assert!(
        matches!(scaled.as_ref().first(), Some(Primitive::Rectangle { center, size, .. })
        if *center == Vec2::new(16.0, 0.0) && *size == Vec2::new(16.0, 4.0))
    );
    // Global visibility suppresses the crosshair when no hit marker is supplied.
    let disabled: Crosshair = "crosshair false"
        .parse()
        .expect("the fixture supplies a valid value");
    assert!(
        disabled
            .draw(&CrosshairInput::default(), None)
            .as_ref()
            .is_empty()
    );
}

/// Rejects each invalid host measurement before any primitive is emitted.
fn invalid_inputs() {
    // Reject each invalid field before geometry generation.
    [
        CrosshairInput {
            viewport: Vec2::splat(32769.0),
            ..default()
        },
        CrosshairInput {
            spread: 32769.0,
            ..default()
        },
        CrosshairInput {
            shot: 32769.0,
            ..default()
        },
        CrosshairInput {
            recoil: Vec2::splat(-32769.0),
            ..default()
        },
        CrosshairInput {
            viewport: Vec2::ZERO,
            ..default()
        },
        CrosshairInput {
            viewport: Vec2::splat(f32::NAN),
            ..default()
        },
        CrosshairInput {
            spread: -1.0,
            ..default()
        },
        CrosshairInput {
            spread: f32::INFINITY,
            ..default()
        },
        CrosshairInput {
            shot: -1.0,
            ..default()
        },
        CrosshairInput {
            shot: f32::NAN,
            ..default()
        },
        CrosshairInput {
            recoil: Vec2::splat(f32::NAN),
            ..default()
        },
    ]
    .into_iter()
    .for_each(|input| {
        assert!(Crosshair::default().draw(&input, None).as_ref().is_empty());
    });
}

/// Covers every style and independently verifies coordinates and ingress limits.
pub(crate) fn run() {
    styles();
    static_geometry();
    invalid_inputs();
}
