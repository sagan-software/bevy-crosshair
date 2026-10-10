//! Shared policies conformance through the public API.

use super::*;

/// Covers grenade delay boundaries, scope phases, and spectator policy
/// selection.
fn grenades() {
    let kinds = [
        (Grenade::Flash, "flash"),
        (Grenade::Explosive, "explosive"),
        (Grenade::Fire, "fire"),
        (Grenade::Smoke, "smoke"),
        (Grenade::Decoy, "decoy"),
    ];
    kinds
        .into_iter()
        .for_each(|(kind, suffix)| grenade(kind, suffix));
}

/// Checks one grenade delay and its two visibility settings.
fn grenade(kind: Grenade, suffix: &str) {
    // The guide appears at the exact configured delay.
    let mut crosshair = Crosshair::default();
    let input = |millis| CrosshairInput {
        weapon: WeaponState::Grenade {
            kind,
            held: Duration::from_millis(millis),
        },
        ..default()
    };
    let baseline = count(&crosshair, &input(1999));
    assert_eq!(count(&crosshair, &input(2000)), baseline + 6);
    assert_eq!(count(&crosshair, &input(2001)), baseline + 6);
    // Keeping the player crosshair affects only the underlying shape.
    crosshair
        .set("cl_grenadecrosshair_keepusercrosshair", "false")
        .expect("the fixture supplies a valid value");
    assert_eq!(count(&crosshair, &input(2000)), 6);
    // Disabling this grenade restores the underlying crosshair.
    crosshair
        .set(&format!("cl_grenadecrosshair_{suffix}"), "false")
        .expect("the fixture supplies a valid value");
    assert_eq!(count(&crosshair, &input(2000)), baseline);
}

/// Checks scope policy for each lifecycle phase and zero or positive spread.
fn scopes() {
    let phases = [
        WeaponState::Hip,
        WeaponState::Sniper,
        WeaponState::Cycling,
        WeaponState::Ready,
        WeaponState::Ironsight,
    ];
    // Each lifecycle phase must remain drawable with either policy setting.
    for phase in phases {
        for is_enabled in ["true", "false"] {
            let mut crosshair = Crosshair::default();
            // Exercise both sides of every scope policy with shared state.
            for name in [
                "cl_sniper_delay_unscope",
                "cl_sniper_auto_rezoom",
                "cl_sniper_show_inaccuracy",
                "cl_ironsight_usecrosshaircolor",
            ] {
                crosshair
                    .set(name, is_enabled)
                    .expect("the fixture supplies a valid value");
            }
            // Zero spread suppresses only the inaccuracy ring.
            [0.0, 10.0].into_iter().for_each(|spread| {
                assert!(
                    !crosshair
                        .draw(
                            &CrosshairInput {
                                weapon: phase,
                                spread,
                                ..default()
                            },
                            None
                        )
                        .as_ref()
                        .is_empty()
                );
            });
        }
    }
}

/// Checks friendly warning and observed-player selection independently.
fn friendly_warning() {
    // Friendly warning adds exactly two primitives when enabled.
    let mut crosshair = Crosshair::default();
    let is_friendly = CrosshairInput {
        is_friendly: true,
        ..default()
    };
    let baseline = crosshair.draw(&is_friendly, None).as_ref().len();
    crosshair
        .set("cl_crosshair_friendly_warning", "0")
        .expect("the fixture supplies a valid value");
    assert_eq!(
        crosshair.draw(&is_friendly, None).as_ref().len() + 2,
        baseline
    );
}

/// Selects observed player settings according to friendship policy.
fn spectators() {
    // Hidden observed settings make selection visible through empty geometry.
    let mut crosshair = Crosshair::default();
    let observed: Crosshair = "crosshair false"
        .parse()
        .expect("the fixture supplies a valid value");
    for policy in ["0", "1", "2"] {
        crosshair
            .set("cl_show_observer_crosshair", policy)
            .expect("the fixture supplies a valid value");
        for spectator in [
            Spectator::Friend(Box::new(observed.clone())),
            Spectator::Player(Box::new(observed.clone())),
        ] {
            let is_hidden = crosshair
                .draw(
                    &CrosshairInput {
                        spectator: spectator.clone(),
                        ..default()
                    },
                    None,
                )
                .as_ref()
                .is_empty();
            // Friends are allowed by either observer policy; strangers require everyone.
            let should_hide = match spectator {
                Spectator::Friend(_) => policy != "0",
                Spectator::Player(_) => policy == "2",
                Spectator::Local | Spectator::AvailableBot | Spectator::UnavailableBot => false,
            };
            assert_eq!(is_hidden, should_hide);
        }
    }
}

/// Checks both bot takeover states under every bot policy.
fn bots() {
    let mut crosshair = Crosshair::default();
    // Both the local default and the selected bot default remain visible.
    for policy in ["0", "1", "2"] {
        crosshair
            .set("cl_observed_bot_crosshair", policy)
            .expect("the fixture supplies a valid value");
        [Spectator::AvailableBot, Spectator::UnavailableBot]
            .into_iter()
            .for_each(|spectator| {
                assert!(
                    !crosshair
                        .draw(
                            &CrosshairInput {
                                spectator,
                                ..default()
                            },
                            None
                        )
                        .as_ref()
                        .is_empty()
                );
            });
    }
}

/// Covers grenade, scope, warning, and spectator policies at the public seam.
pub(crate) fn run() {
    // Exercise each independent source of weapon and observer policy.
    grenades();
    scopes();
    friendly_warning();
    spectators();
    bots();
}

/// Counts emitted primitives for exact policy boundary assertions.
fn count(crosshair: &Crosshair, input: &CrosshairInput) -> usize {
    crosshair.draw(input, None).as_ref().len()
}
