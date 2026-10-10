//! Host-owned firing motion, separate from hit-marker appearance and delivery.

use bevy::prelude::*;
use bevy_crosshair::CrosshairInput;
use std::time::Duration;

/// Baseline measurements and an optional transient firing clock.
#[derive(Component, Default, Reflect)]
pub(super) struct FiringMotion {
    /// Resting weapon spread in physical pixels.
    spread: f32,
    /// Resting shot feedback in physical pixels.
    shot: f32,
    /// Resting aim punch in physical pixels, positive Y downward.
    recoil: Vec2,
    /// Elapsed time since the latest shot, absent while resting.
    elapsed: Option<Duration>,
}

impl FiringMotion {
    /// Retains scalar simulation values without cloning observed-player configuration.
    pub(super) fn accept(&mut self, input: &CrosshairInput) {
        self.spread = input.spread;
        self.shot = input.shot;
        self.recoil = input.recoil;
    }

    /// Restarts one bounded impulse; rapid shots do not accumulate unbounded recoil.
    pub(super) fn fire(&mut self) {
        self.elapsed = Some(Duration::ZERO);
    }

    /// Returns to the supplied baseline after 450 milliseconds of cubic decay.
    pub(super) fn advance(&mut self, delta: Duration, input: &mut CrosshairInput) {
        let Some(elapsed) = self.elapsed else { return };
        // Seconds divided by seconds yield a unitless phase; 24 is physical pixels.
        let progress = (elapsed.as_secs_f32() / 0.45).min(1.0);
        let impulse = 24.0 * (1.0 - progress).powi(3);
        // Derive each frame from the baseline so impulses never accumulate rounding error.
        input.spread = self.spread + impulse;
        input.shot = self.shot + impulse;
        input.recoil = self.recoil + Vec2::new(impulse * 0.12, -impulse * 0.35);
        self.elapsed = if progress >= 1.0 {
            None
        } else {
            Some(elapsed.saturating_add(delta))
        };
    }
}

#[cfg(test)]
mod tests {
    //! The host impulse must be visible, restartable, and restore manual input exactly.
    use super::*;

    #[test]
    fn firing_restores_the_configured_baseline() {
        let mut motion = FiringMotion::default();
        let mut input = CrosshairInput {
            spread: 9.0,
            shot: 4.0,
            recoil: Vec2::new(3.0, -2.0),
            ..default()
        };
        motion.accept(&input);
        motion.advance(Duration::from_secs(1), &mut input);
        assert_eq!(input.spread.to_bits(), 9.0_f32.to_bits());
        motion.fire();
        motion.advance(Duration::from_millis(225), &mut input);
        assert!(input.spread > 9.0 && input.recoil.y < -2.0);
        motion.advance(Duration::from_millis(225), &mut input);
        assert!(input.spread > 9.0 && input.spread < 33.0);
        motion.fire();
        motion.advance(Duration::from_secs(1), &mut input);
        assert_eq!(input.spread.to_bits(), 33.0_f32.to_bits());
        motion.advance(Duration::ZERO, &mut input);
        assert_eq!(input.spread.to_bits(), 9.0_f32.to_bits());
        assert_eq!(input.shot.to_bits(), 4.0_f32.to_bits());
        assert_eq!(input.recoil, Vec2::new(3.0, -2.0));
    }
}
