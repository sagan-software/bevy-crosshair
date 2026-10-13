//! Geometry, opacity, and policy boundaries for the visual extensions.

use super::*;
use bevy_crosshair::DotShape;
use bevy_crosshair::MarkerAnimation;
use bevy_crosshair::MarkerShape;

/// Exercises every new rendering branch through public configuration and metadata.
pub(crate) fn run() {
    // Keep the same public scenarios in test and instrumented benchmark execution.
    fills();
    assert_dots();
    observed();
    animated_shapes();
    animated_event();
}

/// Verifies every style's fill and its invalid-color boundaries.
fn fills() {
    // Each style must retain its original geometry when a fill cannot be drawn.
    for code in 0..10 {
        let base = Crosshair::default().with_style(Style::try_from(code).expect("valid style"));
        assert_fill(&base);
        assert_rejected_fills(&base);
        assert_empty_fill(base);
    }
    // Foreground alpha cannot suppress an independently opaque fill.
    let mut independent = Crosshair::default().with_fill(LinearRgba::BLUE.with_alpha(2.0));
    independent
        .set("cl_crosshaircolor_a", "0")
        .expect("valid alpha");
    assert!(
        matches!(independent.draw(&CrosshairInput::default(), None).as_ref().first(), Some(Primitive::Ring { color, .. }) if *color == LinearRgba::BLUE)
    );
}

/// Checks fill paint order and confirms the console profile excludes extensions.
fn assert_fill(base: &Crosshair) {
    // Fill alpha is independent of the foreground's default opacity.
    let input = CrosshairInput::default();
    let filled = base.clone().with_fill(LinearRgba::BLUE.with_alpha(0.25));
    assert_eq!(base.to_string(), filled.to_string());
    let drawing = filled.draw(&input, None);
    // Dot-only styles have no interior; every other fill precedes foreground geometry.
    if let Some(expected) = expected_fill(filled.style()) {
        assert_eq!(drawing.as_ref().first(), Some(&expected));
    } else {
        assert_eq!(drawing.as_ref(), base.draw(&input, None).as_ref());
    }
}

/// Defines physical geometry for the default four-pixel gap and eight-pixel length.
fn expected_fill(style: Style) -> Option<Primitive> {
    let color = LinearRgba::BLUE.with_alpha(0.25);
    // Dynamic quads place their inner arc edge one length beyond the gap.
    let radius = if style == Style::DynamicQuad {
        12.0
    } else {
        4.0
    };
    match style {
        Style::Dot => None,
        Style::StaticSquare => Some(Primitive::Rectangle {
            center: Vec2::ZERO,
            size: Vec2::splat(8.0),
            rotation: 0.0,
            color,
        }),
        Style::DynamicCross
        | Style::DynamicCircle
        | Style::Legacy
        | Style::StaticCircle
        | Style::StaticCross
        | Style::ShotCross
        | Style::DynamicQuad
        | Style::StaticQuad => Some(Primitive::Ring {
            center: Vec2::ZERO,
            radius,
            thickness: radius * 2.0,
            fraction: 1.0,
            is_top_omitted: false,
            color,
        }),
    }
}

/// Transparent and malformed colors must retain the unfilled geometry exactly.
fn assert_rejected_fills(base: &Crosshair) {
    let input = CrosshairInput::default();
    for color in [
        LinearRgba::NONE,
        LinearRgba::WHITE.with_alpha(-1.0),
        LinearRgba::new(f32::NAN, 0.0, 0.0, 1.0),
        LinearRgba::WHITE.with_alpha(f32::INFINITY),
    ] {
        assert_eq!(
            base.clone().with_fill(color).draw(&input, None).as_ref(),
            base.draw(&input, None).as_ref()
        );
    }
}

/// An interior with no gap or arc offset must not produce a degenerate fill.
fn assert_empty_fill(mut base: Crosshair) {
    // Remove both contributors to the largest possible interior radius.
    base.set("cl_crosshair_gap", "0").expect("valid gap");
    base.set("cl_crosshair_length", "0").expect("valid length");
    let input = CrosshairInput::default();
    let expected = base.draw(&input, None);
    assert_eq!(
        base.with_fill(LinearRgba::BLUE).draw(&input, None).as_ref(),
        expected.as_ref()
    );
}

/// Verifies round-dot dimensions, outlines, scaling, and transparent disks.
fn assert_dots() {
    // Outline policy changes the outside disk without changing the foreground diameter.
    for (outline, expansion) in [("0", 0.0), ("1", 1.0), ("2", 0.5)] {
        let mut crosshair = Crosshair::default()
            .with_style(Style::Dot)
            .with_dot_shape(DotShape::Circle);
        crosshair
            .set("cl_crosshair_drawoutline", outline)
            .expect("valid outline");
        assert_dot_outline(&crosshair, outline, expansion);
        assert_dot_scale(&crosshair);
        // Neither a transparent foreground nor a transparent outline may leave a disk.
        crosshair
            .set("cl_crosshaircolor_a", "0")
            .expect("valid alpha");
        crosshair
            .set("cl_crosshairoutline_a", "0")
            .expect("valid alpha");
        assert!(
            crosshair
                .draw(&CrosshairInput::default(), None)
                .as_ref()
                .is_empty()
        );
    }
}

/// Checks the exact disk count and outer diameter for one outline mode.
fn assert_dot_outline(crosshair: &Crosshair, outline: &str, expansion: f32) {
    // Every enabled outline adds one disk behind the foreground.
    let drawing = crosshair.draw(&CrosshairInput::default(), None);
    let expected = if outline == "0" { 1 } else { 2 };
    assert_eq!(drawing.as_ref().len(), expected);
    // The first disk's diameter includes the selected outline expansion on both sides.
    assert!(
        matches!(drawing.as_ref().first(), Some(Primitive::Ring { radius, thickness, .. })
        if radius.to_bits() == (1.0_f32 + expansion).to_bits() && thickness.to_bits() == (2.0_f32 + 2.0 * expansion).to_bits())
    );
}

/// Doubling viewport height doubles the dot radius before the fixed outline width.
fn assert_dot_scale(crosshair: &Crosshair) {
    let scaled = crosshair.draw(
        &CrosshairInput {
            viewport: Vec2::new(3840.0, 2160.0),
            ..default()
        },
        None,
    );
    assert!(
        matches!(scaled.as_ref().last(), Some(Primitive::Ring { radius, .. }) if radius.to_bits() == 2.0_f32.to_bits())
    );
}

/// Observed extensions follow the same ownership policy as console settings.
fn observed() {
    // Distinct fill colors reveal whether policy selected the entire observed configuration.
    let mut local = Crosshair::default().with_fill(LinearRgba::RED);
    local
        .set("cl_show_observer_crosshair", "2")
        .expect("valid policy");
    let observed = Crosshair::default()
        .with_fill(LinearRgba::BLUE)
        .with_dot_shape(DotShape::Circle);
    let expected = observed.draw(&CrosshairInput::default(), None);
    // The public spectator input owns the observed configuration after this move.
    let input = CrosshairInput {
        spectator: Spectator::Player(Box::new(observed)),
        ..default()
    };
    assert_eq!(local.draw(&input, None).as_ref(), expected.as_ref());
}

/// Exercises every shape and animation independently of the ordinary crosshair.
fn animated_shapes() {
    // Shape and animation are independent public choices, so test their complete product.
    let shapes = [
        MarkerShape::Cross,
        MarkerShape::Plus,
        MarkerShape::Chevron,
        MarkerShape::Diamond,
        MarkerShape::Ring,
    ];
    let animations = [
        MarkerAnimation::Fade,
        MarkerAnimation::Expand,
        MarkerAnimation::Contract,
        MarkerAnimation::Pulse,
    ];
    // Every pair must satisfy both lifetime and radial-motion contracts.
    for shape in shapes {
        for animation in animations {
            assert_marker_frames(shape, animation);
            assert_marker_motion(shape, animation);
        }
    }
}

/// Creates a one-second marker so millisecond boundaries have exact meanings.
fn marker_fixture(shape: MarkerShape, animation: MarkerAnimation) -> HitMarker {
    HitMarker::default()
        .with_shape(shape)
        .with_animation(animation)
        .with_duration(Duration::from_secs(1))
}

/// Checks fade endpoints, full opacity, restarts, and zero duration independently.
fn assert_marker_frames(shape: MarkerShape, animation: MarkerAnimation) {
    let fresh = marker_fixture(shape, animation);
    // Sample both endpoints and both sides of the pulse's change in direction.
    for milliseconds in [0, 200, 600, 800, 1000] {
        let mut marker = fresh;
        marker.advance(Duration::from_millis(milliseconds));
        assert!((0.0..=1.0).contains(&marker.opacity()));
        assert_marker_count(marker, shape, animation, milliseconds);
        assert_eq!(
            marker.marker().opacity().to_bits(),
            fresh.opacity().to_bits()
        );
    }
    // Explicit zero duration must suppress every shape and animation.
    assert!(
        marker_drawing(fresh.with_duration(Duration::ZERO))
            .as_ref()
            .is_empty()
    );
}

/// Visible geometry has a fixed primitive count for each marker shape.
fn assert_marker_count(
    marker: HitMarker,
    shape: MarkerShape,
    animation: MarkerAnimation,
    milliseconds: u64,
) {
    // Fade begins visible; moving markers have a transparent first frame.
    let is_visible =
        milliseconds < 1000 && (animation == MarkerAnimation::Fade || milliseconds > 0);
    let count = match shape {
        MarkerShape::Ring => 1,
        MarkerShape::Chevron => 2,
        MarkerShape::Cross | MarkerShape::Plus | MarkerShape::Diamond => 4,
    };
    assert_eq!(
        marker_drawing(marker).as_ref().len(),
        if is_visible { count } else { 0 }
    );
}

/// Compares radial positions after the outward phase, independent of opacity.
fn assert_marker_motion(shape: MarkerShape, animation: MarkerAnimation) {
    // Separate fresh clocks make the comparison independent of cumulative time steps.
    let fresh = marker_fixture(shape, animation);
    let previous = marker_extent(fresh, 600);
    let current = marker_extent(fresh, 800);
    // The pulse has reversed direction by 800ms; only expansion still moves outward.
    match animation {
        MarkerAnimation::Expand => assert!(current > previous),
        MarkerAnimation::Contract | MarkerAnimation::Pulse => assert!(current < previous),
        MarkerAnimation::Fade => assert_eq!(current.to_bits(), previous.to_bits()),
    }
}

/// Measures maximum arm-center displacement or ring radius in physical pixels.
fn marker_extent(mut marker: HitMarker, milliseconds: u64) -> f32 {
    marker.advance(Duration::from_millis(milliseconds));
    marker_drawing(marker)
        .as_ref()
        .iter()
        .map(|primitive| match primitive {
            Primitive::Rectangle { center, .. } | Primitive::FadingRectangle { center, .. } => {
                center.length()
            }
            Primitive::Ring { radius, .. } | Primitive::Arc { radius, .. } => *radius,
        })
        .fold(0.0, f32::max)
}

/// Disables ordinary geometry so every returned primitive belongs to the marker.
fn marker_drawing(marker: HitMarker) -> bevy_crosshair::Drawing {
    let crosshair: Crosshair = "crosshair false".parse().expect("valid fixture");
    crosshair.draw(&CrosshairInput::default(), Some(&marker))
}

/// Transparent fade-in starts must still consume the application's time step.
fn animated_event() {
    // Initialize virtual time before delivering the first animated hit.
    let mut app = event_application();
    let entity = app.world_mut().spawn(Crosshair::default()).id();
    app.update();
    app.world_mut().trigger(Hit {
        entity,
        metadata: marker_fixture(MarkerShape::Cross, MarkerAnimation::Pulse)
            .with_duration(Duration::from_millis(500)),
    });
    // One 100ms step reaches the end of the 500ms marker's fade-in interval.
    app.update();
    let marker = app
        .world()
        .get::<HitMarker>(entity)
        .expect("required marker exists");
    assert_eq!(marker.opacity().to_bits(), 1.0_f32.to_bits());
}

/// Supplies a fixed step without depending on wall-clock timing.
fn event_application() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(CrosshairPlugin::<HitMarker>::default())
        .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_millis(
            100,
        )));
    app
}
