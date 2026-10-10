//! Browser boundary: validated configuration ingress and independent hit
//! receipts.

use super::app::Damage;
use super::app::DamageKind;
use super::app::Preview;
use super::motion::FiringMotion;
use bevy::prelude::*;
use bevy_crosshair::Crosshair;
use bevy_crosshair::CrosshairInput;
use bevy_crosshair::DotShape;
use bevy_crosshair::Grenade;
use bevy_crosshair::Hit;
use bevy_crosshair::HitFeedback;
use bevy_crosshair::HitMarker;
use bevy_crosshair::MarkerAnimation;
use bevy_crosshair::MarkerShape;
use bevy_crosshair::Spectator;
use bevy_crosshair::WeaponState;
use std::cell::RefCell;
use std::collections::VecDeque;
use std::time::Duration;
use wasm_bindgen::prelude::*;

thread_local! {
    /// Browser changes are consumed once by the next Bevy update.
    static PENDING: RefCell<Pending> = RefCell::new(Pending::default());
}

/// Pending controls never replace the viewport supplied by the actual window.
#[derive(Default)]
struct Pending {
    /// A complete validated crosshair configuration.
    configuration: Option<Crosshair>,
    /// Latest complete simulation state.
    input: Option<CrosshairInput>,
    /// Clicks retained until Bevy consumes them, bounded at 64 entries.
    hits: VecDeque<Damage>,
    /// Latest visual-only replay, coalesced while controls are dragged.
    preview: Option<Damage>,
    /// Fill and dot shape pending atomic application to the editable crosshair.
    appearance: Option<(LinearRgba, DotShape)>,
    /// Shape applied when the next hit or visual preview enters the queue.
    marker_shape: MarkerShape,
    /// Animation applied when the next hit or visual preview enters the queue.
    marker_animation: MarkerAnimation,
    /// A validated scene change, consumed once.
    scene: Option<Scene>,
}

/// Either one editable crosshair or a bounded gallery, never both visible.
enum Scene {
    /// Return to the existing editor entity and its preserved configuration.
    Editor,
    /// Complete validated gallery and its responsive column count.
    Gallery(Vec<Crosshair>, Columns),
}

/// The two gallery layouts shared with CSS.
#[derive(Clone, Copy)]
enum Columns {
    /// Narrow-screen gallery with twenty rows.
    Five,
    /// Desktop gallery with ten rows.
    Ten,
}

impl TryFrom<u8> for Columns {
    type Error = &'static str;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            5 => Ok(Self::Five),
            10 => Ok(Self::Ten),
            _ => Err("Gallery columns must be 5 or 10."),
        }
    }
}

/// Parent whose descendants are replaced only on a gallery scene change.
#[derive(Component, Clone, Copy, Default, Reflect)]
pub(super) struct GalleryRoot;

#[wasm_bindgen]
extern "C" {
    /// The page updates its measurement display from this Bevy callback.
    #[wasm_bindgen(js_namespace = window, js_name = crosshairFrame)]
    fn report_frame(fps: f64, longest_ms: f64, crosshairs: usize);
    /// Independent event receipt used by the page's optional audio observer.
    #[wasm_bindgen(js_namespace = window, js_name = crosshairHit)]
    fn report_hit(points: f32, kind: u8);
}

/// Validates a whole configuration before publishing it to Bevy.
#[wasm_bindgen]
pub fn configure(source: &str) -> Result<(), JsValue> {
    let crosshair = source
        .parse::<Crosshair>()
        .map_err(|error| JsValue::from_str(&error.to_string()))?;
    PENDING.with_borrow_mut(|pending| pending.configuration = Some(crosshair));
    Ok(())
}

/// Returns the exact current-profile defaults for controls and reset.
#[wasm_bindgen]
pub fn defaults() -> String {
    Crosshair::default().to_string()
}

/// Sets visual extensions without adding non-CS2 names to the console profile.
#[wasm_bindgen]
pub fn appearance(red: u8, green: u8, blue: u8, alpha: u8, dot: u8) -> Result<(), JsValue> {
    let dot = match dot {
        0 => DotShape::Square,
        1 => DotShape::Circle,
        _ => return Err(JsValue::from_str("Unknown center-dot shape.")),
    };
    let fill = LinearRgba::from(Srgba::rgba_u8(red, green, blue, alpha));
    PENDING.with_borrow_mut(|pending| pending.appearance = Some((fill, dot)));
    Ok(())
}

/// Validates marker style before it can affect queued hit metadata.
#[wasm_bindgen]
pub fn marker_style(shape: u8, animation: u8) -> Result<(), JsValue> {
    let shape = match shape {
        0 => MarkerShape::Cross,
        1 => MarkerShape::Plus,
        2 => MarkerShape::Chevron,
        3 => MarkerShape::Diamond,
        4 => MarkerShape::Ring,
        _ => return Err(JsValue::from_str("Unknown marker shape.")),
    };
    let animation = match animation {
        0 => MarkerAnimation::Fade,
        1 => MarkerAnimation::Expand,
        2 => MarkerAnimation::Contract,
        3 => MarkerAnimation::Pulse,
        _ => return Err(JsValue::from_str("Unknown marker animation.")),
    };
    PENDING.with_borrow_mut(|pending| {
        pending.marker_shape = shape;
        pending.marker_animation = animation;
    });
    Ok(())
}

/// Validates game metadata for both real hits and visual-only previews.
fn damage(
    points: f32,
    kind: u8,
    milliseconds: u32,
    scale: f32,
    is_damage_scaled: bool,
) -> Result<Damage, JsValue> {
    if !points.is_finite()
        || !(0.0..=1000.0).contains(&points)
        || !scale.is_finite()
        || !(0.0..=8.0).contains(&scale)
    {
        return Err(JsValue::from_str(
            "Damage must be 0–1000 and marker scale must be 0–8.",
        ));
    }
    let kind = match kind {
        0 => DamageKind::Body,
        1 => DamageKind::Head,
        2 => DamageKind::Elimination,
        _ => return Err(JsValue::from_str("Unknown hit kind.")),
    };
    Ok(PENDING.with_borrow(|pending| Damage {
        points,
        kind,
        duration: Duration::from_millis(u64::from(milliseconds)),
        scale,
        is_damage_scaled,
        shape: pending.marker_shape,
        animation: pending.marker_animation,
    }))
}

/// Replays visual feedback without dispatching a hit event, receipt, or sound.
#[wasm_bindgen]
pub fn preview_hit(
    points: f32,
    kind: u8,
    milliseconds: u32,
    scale: f32,
    is_damage_scaled: bool,
) -> Result<(), JsValue> {
    let metadata = damage(points, kind, milliseconds, scale, is_damage_scaled)?;
    PENDING.with_borrow_mut(|pending| pending.preview = Some(metadata));
    Ok(())
}

/// Replaces the gallery atomically after validating at most 100 console profiles.
/// An empty source returns to the retained editor. Entries are separated by NUL.
#[wasm_bindgen]
pub fn browse(source: &str, columns: u8) -> Result<(), JsValue> {
    let columns = Columns::try_from(columns).map_err(JsValue::from_str)?;
    let scene = if source.is_empty() {
        Scene::Editor
    } else {
        if source.len() > 1_000_000 || source.split('\0').count() > 100 {
            return Err(JsValue::from_str(
                "Gallery supports at most 100 crosshairs and 1 MB of configuration.",
            ));
        }
        let configurations = source
            .split('\0')
            .map(str::parse::<Crosshair>)
            .collect::<Result<Vec<_>, _>>()
            .map_err(|error| JsValue::from_str(&error.to_string()))?;
        Scene::Gallery(configurations, columns)
    };
    PENDING.with_borrow_mut(|pending| pending.scene = Some(scene));
    Ok(())
}

/// Queues game metadata; the plugin and independent observers receive one
/// event.
#[wasm_bindgen]
pub fn fire(
    points: f32,
    kind: u8,
    milliseconds: u32,
    scale: f32,
    is_damage_scaled: bool,
) -> Result<(), JsValue> {
    let metadata = damage(points, kind, milliseconds, scale, is_damage_scaled)?;
    PENDING.with_borrow_mut(|pending| {
        if pending.hits.len() >= 64 {
            return Err(JsValue::from_str(
                "Hit queue is full. Wait for the next frame.",
            ));
        }
        pending.hits.push_back(metadata);
        Ok(())
    })
}

/// Sets host simulation values in physical pixels and milliseconds.
#[wasm_bindgen]
#[expect(
    clippy::too_many_arguments,
    reason = "The existing JavaScript boundary accepts nine scalars; preserve its signature."
)]
pub fn simulate(
    spread: f32,
    shot: f32,
    recoil_x: f32,
    recoil_y: f32,
    scope: u8,
    is_friendly: bool,
    grenade: u8,
    held_ms: u32,
    spectator: u8,
) -> Result<(), JsValue> {
    if [spread, shot, recoil_x, recoil_y]
        .into_iter()
        .any(|value| !value.is_finite() || value.abs() > 8192.0)
        || spread < 0.0
        || shot < 0.0
    {
        return Err(JsValue::from_str(
            "Simulation values are outside their pixel limits.",
        ));
    }
    let scope = match scope {
        0 => WeaponState::Hip,
        1 => WeaponState::Sniper,
        2 => WeaponState::Cycling,
        3 => WeaponState::Ready,
        4 => WeaponState::Ironsight,
        _ => return Err(JsValue::from_str("Unknown scope phase.")),
    };
    let grenade = match grenade {
        0 => None,
        1 => Some(Grenade::Flash),
        2 => Some(Grenade::Explosive),
        3 => Some(Grenade::Fire),
        4 => Some(Grenade::Smoke),
        5 => Some(Grenade::Decoy),
        _ => return Err(JsValue::from_str("Unknown grenade kind.")),
    };
    let observed = "cl_crosshairstyle 6;cl_crosshair_thickness 12;cl_crosshaircolor_r 255;cl_crosshaircolor_g 0;cl_crosshaircolor_b 255"
        .parse::<Crosshair>().expect("the sample player uses a valid console profile");
    let spectator = match spectator {
        0 => Spectator::Local,
        1 => Spectator::Friend(Box::new(observed)),
        2 => Spectator::Player(Box::new(observed)),
        3 => Spectator::AvailableBot,
        4 => Spectator::UnavailableBot,
        _ => return Err(JsValue::from_str("Unknown spectator state.")),
    };
    PENDING.with_borrow_mut(|pending| {
        pending.input = Some(CrosshairInput {
            spread,
            shot,
            recoil: Vec2::new(recoil_x, recoil_y),
            weapon: grenade.map_or(scope, |kind| WeaponState::Grenade {
                kind,
                held: Duration::from_millis(u64::from(held_ms)),
            }),
            is_friendly,
            spectator,
            ..default()
        })
    });
    Ok(())
}

/// Consumes controls through the same Bevy components used by native games.
pub(super) fn synchronize(
    mut crosshairs: Query<'_, '_, EditorView, With<Preview>>,
    galleries: Query<'_, '_, Entity, With<GalleryRoot>>,
    mut commands: Commands<'_, '_>,
) {
    if let Ok((entity, mut crosshair, mut input, mut marker, mut motion)) = crosshairs.single_mut()
    {
        PENDING.with_borrow_mut(|pending| {
            if let Some(configuration) = pending.configuration.take() {
                *crosshair = configuration;
            }
            if let Some((fill, shape)) = pending.appearance.take() {
                *crosshair = crosshair.clone().with_fill(fill).with_dot_shape(shape);
            }
            if let Some(mut next) = pending.input.take() {
                next.viewport = input.viewport;
                motion.accept(&next);
                *input = next;
            }
            // Visual previews never trigger gameplay observers or firing motion.
            if let Some(metadata) = pending.preview.take() {
                *marker = metadata.marker();
            }
            for metadata in pending.hits.drain(..) {
                motion.fire();
                commands.trigger(Hit { entity, metadata });
            }
            if let Some(scene) = pending.scene.take() {
                for gallery in &galleries {
                    commands.entity(gallery).despawn();
                }
                let visibility = match scene {
                    Scene::Editor => Visibility::Visible,
                    Scene::Gallery(configurations, columns) => {
                        spawn_gallery(&mut commands, configurations, columns);
                        Visibility::Hidden
                    }
                };
                commands.entity(entity).insert(visibility);
            }
        });
    }
}

/// Components borrowed together while applying browser updates to the editor.
type EditorView = (
    Entity,
    &'static mut Crosshair,
    &'static mut CrosshairInput,
    &'static mut HitMarker,
    &'static mut FiringMotion,
);

/// Positions every actual Bevy crosshair under the matching HTML selection cell.
fn spawn_gallery(
    commands: &mut Commands<'_, '_>,
    configurations: Vec<Crosshair>,
    columns: Columns,
) {
    let columns = match columns {
        Columns::Five => 5_u16,
        Columns::Ten => 10_u16,
    };
    let count =
        u16::try_from(configurations.len()).expect("gallery validation caps entries at 100");
    let rows = count.div_ceil(columns);
    commands
        .spawn((
            GalleryRoot,
            Node {
                width: percent(100),
                height: percent(100),
                ..default()
            },
        ))
        .with_children(|parent| {
            // Fixed positions avoid a second grid algorithm disagreeing with the browser overlay.
            for (index, crosshair) in configurations.into_iter().enumerate() {
                let index = u16::try_from(index).expect("gallery validation caps entries at 100");
                parent
                    .spawn(Node {
                        position_type: PositionType::Absolute,
                        left: percent(f32::from(index % columns) * 100.0 / f32::from(columns)),
                        top: percent(f32::from(index / columns) * 100.0 / f32::from(rows)),
                        width: percent(100.0 / f32::from(columns)),
                        height: percent(100.0 / f32::from(rows)),
                        overflow: Overflow::clip(),
                        ..default()
                    })
                    .with_children(|cell| {
                        cell.spawn((
                            crosshair,
                            CrosshairInput {
                                // Gallery samples share a 720px reference viewport, independent of row count.
                                viewport: Vec2::splat(720.0),
                                ..default()
                            },
                        ));
                    });
            }
        });
}

/// Forwards a measured frame interval to the page.
pub(super) fn frame(fps: f64, longest_ms: f64, crosshairs: usize) {
    report_frame(fps, longest_ms, crosshairs);
}

/// Forwards the independent hit observer's receipt to the page.
pub(super) fn hit(points: f32, kind: DamageKind) {
    let kind = match kind {
        DamageKind::Body => 0,
        DamageKind::Head => 1,
        DamageKind::Elimination => 2,
    };
    report_hit(points, kind);
}
