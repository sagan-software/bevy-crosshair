//! Mutually exclusive weapon presentation states supplied by the host game.

use std::time::Duration;

/// Mutually exclusive weapon presentation states. The host controls
/// transitions, firing, and
/// elapsed grenade time; the plugin selects only the visible crosshair or scope
/// geometry.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum WeaponState {
    /// Unscoped weapon using ordinary crosshair geometry. Dynamic styles consume
    /// supplied weapon
    /// spread, and the shot-feedback style consumes the independently supplied shot
    /// gap.
    #[default]
    Hip,
    /// Sniper scope before firing, drawn with physical-pixel line widths. Supplied
    /// spread adds an
    /// inaccuracy ring when the corresponding sniper setting is enabled.
    Sniper,
    /// Sniper cycling after firing. The delayed-unscope setting keeps scope
    /// geometry visible
    /// during this state; otherwise the ordinary crosshair appears until the host
    /// changes state.
    Cycling,
    /// Sniper finished cycling after firing. Automatic rezoom selects scope
    /// geometry in this
    /// state; disabling it restores ordinary crosshair geometry until the host
    /// changes state.
    Ready,
    /// Optical sight using the independently scaled center dot. Its color is red
    /// unless the
    /// ironsight color setting selects the configured crosshair color instead.
    Ironsight,
    /// Held grenade with its elapsed pin-pull time. This excludes all scoped
    /// states;
    /// each grenade kind selects its own visibility flag and delay setting.
    Grenade {
        /// Grenade whose visibility flag and delay apply. Fire includes both the
        /// incendiary
        /// grenade and Molotov because the selected CS2 profile shares their controls.
        kind: Grenade,
        /// Elapsed pin-pull time supplied by the host controller. A guide appears when
        /// this duration reaches the selected grenade's configured delay, including
        /// equality.
        held: Duration,
    },
}

/// Grenade kinds with independent visibility and delay settings. Pair a kind
/// with elapsed
/// pin-pull time in the held-grenade weapon state to select its guide.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Grenade {
    /// Flashbang using the flash visibility flag and flash delay. Its alignment
    /// guide starts when
    /// the supplied hold duration reaches that configured delay exactly.
    Flash,
    /// High explosive grenade using the explosive visibility flag and delay. The
    /// guide uses
    /// screen coordinates and does not estimate blast radius or projectile
    /// trajectory.
    Explosive,
    /// Incendiary grenade or Molotov using the shared fire visibility flag and
    /// delay. The
    /// selected CS2 profile assigns both weapons to this single control group.
    Fire,
    /// Smoke grenade using the smoke visibility flag and delay. The host supplies
    /// hold duration;
    /// the renderer does not infer smoke deployment or projectile state.
    Smoke,
    /// Decoy grenade using the decoy visibility flag and delay. Its guide follows
    /// the same
    /// elapsed-time boundary as other kinds while retaining independent
    /// configuration.
    Decoy,
}
