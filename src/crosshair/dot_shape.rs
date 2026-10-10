//! Closed center-dot shapes, independent of the CS2 console profile.

/// Shape of the optional center dot and the dedicated dot crosshair style.
/// Its diameter follows crosshair thickness, including screen-height scaling.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum DotShape {
    /// Filled axis-aligned square, preserving the original center-dot appearance.
    /// Its side length follows crosshair thickness after conversion from authored
    /// pixels to physical viewport pixels.
    #[default]
    Square,
    /// Filled disk with an antialiased edge. Its diameter follows crosshair
    /// thickness after screen-height scaling; outline width remains independent
    /// of the selected dot shape.
    Circle,
}
