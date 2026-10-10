//! Browser boundary: validated configuration ingress and independent hit
//! receipts.

use super::app::Damage;
use super::app::DamageKind;
use bevy::prelude::*;
use bevy_crosshair::Crosshair;
use bevy_crosshair::CrosshairInput;
use bevy_crosshair::Grenade;
use bevy_crosshair::Hit;
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
}

#[wasm_bindgen]
extern "C" {
    /// The page updates its measurement display from this Bevy callback.
    #[wasm_bindgen(js_namespace = window, js_name = crosshairFrame)]
    fn report_frame(fps: f64, longest_ms: f64);
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
    PENDING.with_borrow_mut(|pending| {
        if pending.hits.len() >= 64 {
            return Err(JsValue::from_str(
                "Hit queue is full. Wait for the next frame.",
            ));
        }
        pending.hits.push_back(Damage {
            points,
            kind,
            duration: Duration::from_millis(u64::from(milliseconds)),
            scale,
            is_damage_scaled,
        });
        Ok(())
    })
}

/// Sets host simulation values in physical pixels and milliseconds.
#[wasm_bindgen]
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
    let observed = Crosshair::default().with_style(bevy_crosshair::Style::Dot);
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
    mut crosshairs: Query<'_, '_, (Entity, &mut Crosshair, &mut CrosshairInput)>,
    mut commands: Commands<'_, '_>,
) {
    if let Ok((entity, mut crosshair, mut input)) = crosshairs.single_mut() {
        PENDING.with_borrow_mut(|pending| {
            if let Some(configuration) = pending.configuration.take() {
                *crosshair = configuration;
            }
            if let Some(mut next) = pending.input.take() {
                next.viewport = input.viewport;
                *input = next;
            }
            for metadata in pending.hits.drain(..) {
                commands.trigger(Hit { entity, metadata });
            }
        });
    }
}

/// Forwards a measured frame interval to the page.
pub(super) fn frame(fps: f64, longest_ms: f64) {
    report_frame(fps, longest_ms);
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
