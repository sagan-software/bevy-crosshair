//! Hit marker appearance and elapsed lifetime.

use bevy::prelude::*;
use std::time::Duration;

/// The newest hit marker on a crosshair. A later hit replaces its appearance
/// and restarts its
/// lifetime; expired markers remain stored without drawing geometry.
#[derive(Component, Clone, Copy, Debug, Reflect)]
#[reflect(opaque)]
pub struct HitMarker {
    /// Elapsed lifetime since delivery.
    elapsed: Duration,
    /// Complete fade duration.
    duration: Duration,
    /// Marker length and thickness in physical pixels.
    size: Vec2,
    /// Distance from center to each marker arm in physical pixels.
    gap: f32,
    /// Marker color in linear space.
    color: LinearRgba,
}

impl Default for HitMarker {
    fn default() -> Self {
        Self {
            elapsed: Duration::ZERO,
            duration: Duration::from_millis(180),
            size: Vec2::new(7.0, 2.0),
            gap: 9.0,
            color: LinearRgba::WHITE,
        }
    }
}

impl HitMarker {
    /// Required initial component, avoiding allocation or insertion on each hit.
    pub(crate) fn inactive() -> Self {
        Self::default().with_duration(Duration::ZERO)
    }
    /// Scales marker dimensions by a nonnegative, finite factor.
    /// Invalid factors produce an invisible marker.
    /// Factors are capped at 32; each resulting dimension is capped at 8192 pixels.
    ///
    /// # Examples
    ///
    /// ```
    /// use bevy_crosshair::HitMarker;
    /// let large = HitMarker::default().scaled(2.0);
    /// ```
    pub fn scaled(mut self, factor: f32) -> Self {
        // Invalid metadata cannot inject non-finite geometry into the renderer.
        let factor = if factor.is_finite() {
            factor.clamp(0.0, 32.0)
        } else {
            0.0
        };
        // Repeated scaling remains bounded even when each individual factor is valid.
        self.size = (self.size * factor).min(Vec2::splat(8192.0));
        self.gap = (self.gap * factor).min(8192.0);
        self
    }

    /// Chooses a lifetime without advancing elapsed time. Zero duration produces an
    /// invisible
    /// marker, and a later hit restarts the lifetime from zero elapsed time.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::time::Duration;
    /// use bevy_crosshair::HitMarker;
    /// let marker = HitMarker::default().with_duration(Duration::from_millis(250));
    /// ```
    pub fn with_duration(mut self, duration: Duration) -> Self {
        self.duration = duration;
        self
    }

    /// Chooses the marker's linear RGBA color while retaining its dimensions and
    /// lifetime. Color
    /// affects only this hit marker, independently of crosshair and outline
    /// settings.
    /// Non-finite colors become transparent. Alpha is clamped to zero through one.
    ///
    /// # Examples
    ///
    /// ```
    /// use bevy::color::LinearRgba;
    /// use bevy_crosshair::HitMarker;
    /// let marker = HitMarker::default().with_color(LinearRgba::WHITE);
    /// ```
    pub fn with_color(mut self, color: LinearRgba) -> Self {
        self.color = if color.to_vec4().is_finite() {
            color.with_alpha(color.alpha.clamp(0.0, 1.0))
        } else {
            LinearRgba::NONE
        };
        self
    }

    /// Advances elapsed time, saturating at the duration limit. Large frame
    /// intervals cannot
    /// overflow elapsed time or make an expired marker visible again afterward.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::time::Duration;
    /// use bevy_crosshair::HitMarker;
    /// let mut marker = HitMarker::default();
    /// marker.advance(Duration::from_secs(1));
    /// assert_eq!(marker.opacity(), 0.0);
    /// ```
    pub fn advance(&mut self, delta: Duration) {
        // Saturate elapsed time so long frames cannot overflow or revive an expired marker.
        self.elapsed = self.elapsed.saturating_add(delta).min(self.duration);
    }

    /// Returns the remaining linear fade from one to zero. A fresh marker starts at
    /// one, while
    /// zero duration and expired lifetimes return zero.
    ///
    /// # Examples
    ///
    /// ```
    /// use bevy_crosshair::HitMarker;
    /// assert_eq!(HitMarker::default().opacity(), 1.0);
    /// ```
    pub fn opacity(&self) -> f32 {
        if self.duration.is_zero() {
            return 0.0;
        }
        1.0 - (self.elapsed.as_secs_f32() / self.duration.as_secs_f32()).min(1.0)
    }

    /// Draws four diagonal arms after the ordinary crosshair geometry.
    pub(crate) fn draw(&self, drawing: &mut crate::Drawing) {
        let color = self.color.with_alpha(self.color.alpha * self.opacity());
        // Arm centers lie half a length beyond the empty central gap.
        let distance = self.gap + self.size.x * 0.5;
        for direction in [
            Vec2::ONE,
            Vec2::new(-1.0, 1.0),
            -Vec2::ONE,
            Vec2::new(1.0, -1.0),
        ] {
            let unit = direction * std::f32::consts::FRAC_1_SQRT_2;
            drawing.rectangle(unit * distance, self.size, unit.to_angle(), color);
        }
    }
}

impl HitFeedback for HitMarker {
    fn marker(&self) -> HitMarker {
        Self {
            elapsed: Duration::ZERO,
            ..*self
        }
    }
}

/// Converts game-owned hit metadata into visual feedback. Implement this trait
/// on an existing
/// damage or combat type; audio and scoring remain independent observers.
pub trait HitFeedback: Send + Sync + 'static {
    /// Chooses the marker for this hit without consuming the metadata. Return a
    /// fresh appearance
    /// using damage, hit classification, or other values owned by the game.
    ///
    /// # Examples
    ///
    /// ```
    /// use bevy_crosshair::HitFeedback;
    /// use bevy_crosshair::HitMarker;
    /// struct Damage(f32);
    /// impl HitFeedback for Damage {
    ///     fn marker(&self) -> HitMarker {
    ///         HitMarker::default().scaled(1.0 + self.0 / 100.0)
    ///     }
    /// }
    /// ```
    fn marker(&self) -> HitMarker;
}

#[cfg(test)]
mod tests {
    //! Private initial-state invariant, separate from public event delivery tests.

    use super::HitMarker;

    #[test]
    fn inactive_marker_has_no_visible_lifetime() {
        let marker = HitMarker::inactive();
        assert!(marker.duration.is_zero());
        assert!(marker.elapsed.is_zero());
    }
}
