//! Marker movement over a normalized lifetime, with no separate animation state.

/// Motion and opacity over a marker's configured duration. Animated modes fade
/// in during the first fifth, then fade out during the final two fifths.
/// Movement uses cubic ease-out; the default retains the original linear fade.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum MarkerAnimation {
    /// Fixed geometry with a linear fade from full opacity to zero.
    /// This default preserves the original marker behavior and never changes its
    /// radial position.
    #[default]
    Fade,
    /// Starts near the center and moves outward while appearing and fading.
    /// Radial distance grows from one quarter to one and a half times its
    /// configured distance using cubic ease-out.
    Expand,
    /// Starts outside its resting gap and moves toward the center while fading.
    /// Radial distance shrinks from one and a half to one quarter of its
    /// configured distance using cubic ease-out.
    Contract,
    /// Moves outward while appearing, then returns to the center while fading.
    /// The outward phase occupies sixty percent of the lifetime; the remaining
    /// forty percent moves inward using cubic easing.
    Pulse,
}
