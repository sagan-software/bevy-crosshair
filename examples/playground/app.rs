//! Shared native and WebAssembly application.

use super::motion::FiringMotion;
use bevy::prelude::*;
use bevy::window::PresentMode;
use bevy::window::WindowResizeConstraints;
use bevy_crosshair::Crosshair;
use bevy_crosshair::CrosshairInput;
use bevy_crosshair::CrosshairPlugin;
use bevy_crosshair::Hit;
use bevy_crosshair::HitFeedback;
use bevy_crosshair::HitMarker;
use bevy_crosshair::MarkerAnimation;
use bevy_crosshair::MarkerShape;
use std::time::Duration;

/// Damage kind supplied by the game, independent of the plugin.
#[derive(Clone, Copy, Debug)]
pub(super) enum DamageKind {
    /// Ordinary hit.
    Body,
    /// Headshot.
    Head,
    /// Eliminating hit.
    Elimination,
}

/// Game-owned metadata demonstrating damage-dependent feedback.
#[derive(Clone, Copy, Debug)]
pub(super) struct Damage {
    /// Damage in health points.
    pub points: f32,
    /// Hit classification.
    pub kind: DamageKind,
    /// Configured visual duration.
    pub duration: Duration,
    /// Size multiplier independent of damage.
    pub scale: f32,
    /// Whether damage changes marker size.
    pub is_damage_scaled: bool,
    /// Marker geometry chosen by the game.
    pub shape: MarkerShape,
    /// Marker motion chosen independently of hit classification.
    pub animation: MarkerAnimation,
}

impl Default for Damage {
    fn default() -> Self {
        Self {
            points: 40.0,
            kind: DamageKind::Body,
            duration: Duration::from_millis(250),
            scale: 1.0,
            is_damage_scaled: true,
            shape: MarkerShape::default(),
            animation: MarkerAnimation::default(),
        }
    }
}

impl HitFeedback for Damage {
    fn marker(&self) -> HitMarker {
        // The game owns damage units and maps each hit kind to a visual color.
        let color = match self.kind {
            DamageKind::Body => LinearRgba::WHITE,
            DamageKind::Head => LinearRgba::new(1.0, 0.48, 0.08, 1.0),
            DamageKind::Elimination => LinearRgba::new(1.0, 0.06, 0.08, 1.0),
        };
        let factor = if self.is_damage_scaled {
            1.0 + self.points / 100.0
        } else {
            1.0
        };
        HitMarker::default()
            .with_shape(self.shape)
            .with_animation(self.animation)
            .with_color(color)
            .with_duration(self.duration)
            .scaled(self.scale * factor)
    }
}

/// Frame measurements originate in Bevy's per-frame schedule.
#[derive(Resource, Default, Reflect)]
struct Frames {
    /// Frames counted in the current measurement interval.
    count: u32,
    /// Accumulated interval in seconds.
    seconds: f64,
    /// Longest frame in the interval, in seconds.
    longest: f64,
}

/// The editable crosshair, distinguished from gallery entries.
#[derive(Component, Clone, Copy, Default, Reflect)]
pub(super) struct Preview;

/// Builds the shared Bevy application.
pub(super) fn run() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            title: "bevy-crosshair".into(),
            canvas: cfg!(target_arch = "wasm32").then(|| "#crosshair".into()),
            fit_canvas_to_parent: true,
            // The page owns browser dimensions and paints the selected scene below the canvas.
            transparent: cfg!(target_arch = "wasm32"),
            resize_constraints: WindowResizeConstraints {
                min_width: 1.0,
                min_height: 1.0,
                ..default()
            },
            present_mode: PresentMode::AutoNoVsync,
            ..default()
        }),
        ..default()
    }))
    .insert_resource(bevy::winit::WinitSettings::continuous())
    .insert_resource(ClearColor(if cfg!(target_arch = "wasm32") {
        Color::NONE
    } else {
        Color::srgb(0.10, 0.15, 0.20)
    }))
    .add_plugins(CrosshairPlugin::<Damage>::default())
    .init_resource::<Frames>()
    .add_systems(Startup, setup)
    .add_systems(PreUpdate, (resize, controls, firing).chain())
    .add_systems(Last, measure)
    .add_observer(receipt);
    #[cfg(target_arch = "wasm32")]
    app.add_systems(
        PreUpdate,
        super::browser::synchronize.after(resize).before(firing),
    );
    app.run();
}

/// A normal camera and Crosshair component are enough to enable the renderer.
fn setup(mut commands: Commands<'_, '_>) {
    commands.spawn(Camera2d);
    commands.spawn((Crosshair::default(), Preview, FiringMotion::default()));
}

/// Supplies the current physical viewport to the renderer.
fn resize(
    windows: Query<'_, '_, &Window>,
    mut crosshairs: Query<'_, '_, &mut CrosshairInput, With<Preview>>,
) {
    if let Ok(window) = windows.single() {
        let viewport = Vec2::new(
            window.physical_width() as f32,
            window.physical_height() as f32,
        );
        // Unchanged dimensions must leave Bevy change detection untouched.
        crosshairs
            .iter_mut()
            .filter(|input| input.viewport != viewport)
            .for_each(|mut input| input.viewport = viewport);
    }
}

/// Native keyboard controls mirror the browser's style and hit actions.
fn controls(
    keys: Res<'_, ButtonInput<KeyCode>>,
    mut crosshairs: Query<
        '_,
        '_,
        (
            Entity,
            &mut Crosshair,
            &mut CrosshairInput,
            &mut FiringMotion,
        ),
        With<Preview>,
    >,
    mut commands: Commands<'_, '_>,
) {
    // Keyboard modifiers select game metadata before the event is dispatched.
    let kind = keyboard_kind(&keys);
    for (entity, mut crosshair, mut input, mut motion) in &mut crosshairs {
        // Browser controls own keyboard focus and firing; Space can activate its focused button.
        #[cfg(not(target_arch = "wasm32"))]
        if keys.just_pressed(KeyCode::Space) {
            motion.fire();
            commands.trigger(Hit {
                entity,
                metadata: Damage { kind, ..default() },
            });
        }
        // Cycling uses the published contiguous style identifiers.
        #[cfg(not(target_arch = "wasm32"))]
        if keys.just_pressed(KeyCode::ArrowRight) {
            let next = (crosshair.style() as u8 + 1) % 10;
            if let Ok(style) = bevy_crosshair::Style::try_from(next) {
                *crosshair = crosshair.clone().with_style(style);
            }
        }
        // Browser input owns spread on WebAssembly; native input uses ArrowUp.
        let spread: f32 = if keys.pressed(KeyCode::ArrowUp) {
            30.0
        } else {
            0.0
        };
        #[cfg(not(target_arch = "wasm32"))]
        {
            input.spread = spread;
            input.shot = 0.0;
            input.recoil = Vec2::ZERO;
            motion.accept(&input);
        }
        #[cfg(target_arch = "wasm32")]
        let _ = (
            &mut input,
            spread,
            entity,
            &mut crosshair,
            &mut motion,
            &mut commands,
            kind,
        );
    }
}

/// Reports measured Bevy updates instead of a separate JavaScript animation
/// loop.
fn measure(
    time: Res<'_, Time<Real>>,
    mut frames: ResMut<'_, Frames>,
    crosshairs: Query<'_, '_, Option<&Visibility>, With<Crosshair>>,
) {
    // Real time includes stalled frames; virtual time would clamp long delays.
    let seconds = time.delta_secs_f64();
    frames.count += 1;
    frames.seconds += seconds;
    frames.longest = frames.longest.max(seconds);
    // Publish one interval, then start an independent measurement window.
    if frames.seconds >= 1.0 {
        let fps = f64::from(frames.count) / frames.seconds;
        let longest_ms = frames.longest * 1000.0;
        #[cfg(target_arch = "wasm32")]
        super::browser::frame(
            fps,
            longest_ms,
            crosshairs
                .iter()
                .filter(|visibility| !matches!(visibility, Some(Visibility::Hidden)))
                .count(),
        );
        #[cfg(not(target_arch = "wasm32"))]
        {
            let _ = &crosshairs;
            info!(fps, longest_ms, "Frame interval");
        }
        // No elapsed time or maximum-frame value may leak into the next interval.
        *frames = Frames::default();
    }
}

/// Applies host weapon motion before the plugin derives its current geometry.
fn firing(
    time: Res<'_, Time>,
    mut crosshairs: Query<'_, '_, (&mut FiringMotion, &mut CrosshairInput)>,
) {
    for (mut motion, mut input) in &mut crosshairs {
        motion.advance(time.delta(), &mut input);
    }
}

/// This observer shares metadata with marker rendering without depending on it.
fn receipt(hit: On<'_, '_, Hit<Damage>>) {
    let points = hit.metadata.points;
    #[cfg(target_arch = "wasm32")]
    super::browser::hit(points, hit.metadata.kind);
    #[cfg(not(target_arch = "wasm32"))]
    info!(points, "Hit received");
}

/// Resolves keyboard hit classification with headshots taking precedence.
fn keyboard_kind(keys: &ButtonInput<KeyCode>) -> DamageKind {
    if keys.pressed(KeyCode::ShiftLeft) {
        DamageKind::Head
    } else if keys.pressed(KeyCode::ControlLeft) {
        DamageKind::Elimination
    } else {
        DamageKind::Body
    }
}
