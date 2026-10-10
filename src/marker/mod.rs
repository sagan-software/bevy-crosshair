//! Hit marker appearance and elapsed lifetime.

use bevy::prelude::*;
use std::time::Duration;

pub use self::animation::MarkerAnimation;
pub use self::shape::MarkerShape;

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
    /// Closed geometry selected by game metadata.
    shape: MarkerShape,
    /// Motion and fade derived from elapsed lifetime.
    animation: MarkerAnimation,
}

mod animation;
mod shape;

impl Default for HitMarker {
    fn default() -> Self {
        Self {
            elapsed: Duration::ZERO,
            duration: Duration::from_millis(180),
            size: Vec2::new(7.0, 2.0),
            gap: 9.0,
            color: LinearRgba::WHITE,
            shape: MarkerShape::default(),
            animation: MarkerAnimation::default(),
        }
    }
}

impl HitMarker {
    /// Chooses marker geometry while retaining its scale, color, and lifetime.
    /// Every shape supports the same animations, and shape selection does not
    /// change the metadata delivered to independent hit observers.
    ///
    /// # Examples
    ///
    /// ```
    /// use bevy_crosshair::{HitMarker, MarkerShape};
    /// let marker = HitMarker::default().with_shape(MarkerShape::Ring);
    /// ```
    pub fn with_shape(mut self, shape: MarkerShape) -> Self {
        self.shape = shape;
        self
    }

    /// Chooses lifetime-relative motion and opacity without resetting elapsed time.
    /// Triggering a new hit still restarts the complete animation. Fade preserves
    /// the original fixed geometry and linear opacity decay.
    ///
    /// # Examples
    ///
    /// ```
    /// use bevy_crosshair::{HitMarker, MarkerAnimation};
    /// let marker = HitMarker::default().with_animation(MarkerAnimation::Pulse);
    /// ```
    pub fn with_animation(mut self, animation: MarkerAnimation) -> Self {
        self.animation = animation;
        self
    }

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

    /// Keeps fade-in clocks advancing even while their first frame is transparent.
    pub(crate) fn is_active(&self) -> bool {
        self.elapsed < self.duration
    }

    /// Returns current animation opacity. Fade starts at one and decreases linearly.
    /// Animated modes start at zero and reach one after one fifth of their lifetime.
    /// Zero duration and expired lifetimes return zero for every animation.
    ///
    /// # Examples
    ///
    /// ```
    /// use bevy_crosshair::HitMarker;
    /// assert_eq!(HitMarker::default().opacity(), 1.0);
    /// ```
    pub fn opacity(&self) -> f32 {
        let progress = self.progress();
        match self.animation {
            MarkerAnimation::Fade => 1.0 - progress,
            MarkerAnimation::Expand | MarkerAnimation::Contract | MarkerAnimation::Pulse => {
                // Both operands are unitless fractions of lifetime: 20% in, final 40% out.
                ease((progress / 0.2).min(1.0)) * ease(((1.0 - progress) / 0.4).min(1.0))
            }
        }
    }

    /// Returns a bounded lifetime fraction, treating zero duration as expired.
    fn progress(&self) -> f32 {
        if self.duration.is_zero() {
            1.0
        } else {
            (self.elapsed.as_secs_f32() / self.duration.as_secs_f32()).min(1.0)
        }
    }

    /// Derives radial displacement without storing a second animation clock.
    fn motion(&self) -> f32 {
        // Radial multipliers are unitless and derive entirely from the shared lifetime clock.
        let progress = self.progress();
        match self.animation {
            MarkerAnimation::Fade => 1.0,
            MarkerAnimation::Expand => 0.25 + 1.25 * ease(progress),
            MarkerAnimation::Contract => 0.25 + 1.25 * (1.0 - ease(progress)),
            MarkerAnimation::Pulse => {
                // Outward motion takes 60% of lifetime; the remaining 40% returns inward.
                let phase = if progress < 0.6 {
                    progress / 0.6
                } else {
                    (1.0 - progress) / 0.4
                };
                0.25 + 0.75 * ease(phase)
            }
        }
    }

    /// Draws the selected marker after the ordinary crosshair geometry.
    pub(crate) fn draw(&self, drawing: &mut crate::Drawing) {
        let color = self.color.with_alpha(self.color.alpha * self.opacity());
        if color.alpha <= 0.0 || self.size.min_element() <= 0.0 {
            return;
        }
        // Arm centers lie half a length beyond the empty central gap.
        let distance = (self.gap + self.size.x * 0.5) * self.motion();
        match self.shape {
            MarkerShape::Ring => drawing.primitives.push(crate::Primitive::Ring {
                center: Vec2::ZERO,
                radius: distance,
                thickness: self.size.y,
                fraction: 1.0,
                is_top_omitted: false,
                color,
            }),
            MarkerShape::Chevron => {
                // The two arm endpoints meet above the center; gap moves the complete chevron.
                let half = self.size.x * 0.5 * std::f32::consts::FRAC_1_SQRT_2;
                for sign in [-1.0, 1.0] {
                    drawing.rectangle(
                        Vec2::new(sign * half, -self.gap * self.motion() - half),
                        self.size,
                        -sign * std::f32::consts::FRAC_PI_4,
                        color,
                    );
                }
            }
            MarkerShape::Cross | MarkerShape::Plus | MarkerShape::Diamond => {
                self.arms(drawing, distance, color);
            }
        }
    }

    /// Emits symmetric bars for the three four-sided marker shapes.
    fn arms(&self, drawing: &mut crate::Drawing, distance: f32, color: LinearRgba) {
        // Keep the four quadrants in a fixed paint order for every symmetric shape.
        for direction in [
            Vec2::ONE,
            Vec2::new(-1.0, 1.0),
            -Vec2::ONE,
            Vec2::new(1.0, -1.0),
        ] {
            let unit = direction * std::f32::consts::FRAC_1_SQRT_2;
            // Rotate diagonal directions for the plus; connect adjacent axial endpoints for the diamond.
            let (center, size, angle) = match self.shape {
                MarkerShape::Plus => {
                    let cardinal = Vec2::new(
                        (direction.x + direction.y) * 0.5,
                        (direction.y - direction.x) * 0.5,
                    );
                    (cardinal * distance, self.size, cardinal.to_angle())
                }
                MarkerShape::Diamond => (
                    direction * distance * 0.5,
                    Vec2::new(distance * std::f32::consts::SQRT_2, self.size.y),
                    Vec2::new(direction.x, -direction.y).to_angle(),
                ),
                MarkerShape::Cross | MarkerShape::Chevron | MarkerShape::Ring => {
                    (unit * distance, self.size, unit.to_angle())
                }
            };
            drawing.rectangle(center, size, angle, color);
        }
    }
}

/// Cubic ease-out for a unitless fraction in the closed interval zero through one.
fn ease(fraction: f32) -> f32 {
    1.0 - (1.0 - fraction).powi(3)
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

    #[test]
    fn transparent_color_does_not_disable_the_lifetime_clock() {
        // Visibility and expiry are independent: a transparent marker still consumes its lifetime.
        let mut marker = HitMarker::default().with_color(bevy::color::LinearRgba::NONE);
        assert!(marker.is_active());
        marker.advance(std::time::Duration::MAX);
        assert!(!marker.is_active());
        assert_eq!(marker.elapsed, marker.duration);
    }
}
