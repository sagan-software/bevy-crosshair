//! Closed glyph vocabulary for custom crosshair layers.

/// One glyph that a [`Layer`](crate::Layer) repeats at each of its angles.
///
/// Dimensions use authored pixels, which the crosshair's
/// `cl_crosshair_screen_height` setting converts to physical pixels. Each glyph
/// starts at the layer's radial distance, which is its gap plus any followed
/// spread, and points away from the crosshair center.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Mark {
    /// Filled disk centered at the radial distance along each angle. A zero gap
    /// without spread puts a single dot on the aiming point.
    Dot {
        /// Disk diameter in authored pixels. The outline, when the layer has
        /// one, adds one authored pixel around this diameter on every side.
        diameter: f32,
    },
    /// Straight bar whose inner end lies at the radial distance. The bar keeps
    /// full opacity over its solid fraction, measured from the inner end, then
    /// fades linearly to transparent at its outer end.
    Bar {
        /// Bar length along its angle, in authored pixels, measured outward from
        /// the radial distance where the bar's inner end sits.
        length: f32,
        /// Bar width across its angle, in authored pixels. The bar stays
        /// centered on the line from the crosshair center along its angle.
        thickness: f32,
        /// Fraction of the length drawn at full opacity, from zero through one.
        /// One draws a solid bar; zero fades over the whole length. Non-finite
        /// values draw a solid bar.
        solid: f32,
    },
    /// Two straight arms meeting at an apex that points away from the center.
    /// The arm ends lie at the radial distance; the apex lies `depth` beyond it.
    Chevron {
        /// Distance between the two arm ends, in authored pixels, measured
        /// across the angle at the radial distance where both arms start.
        width: f32,
        /// Distance from the arm ends to the apex, in authored pixels. A
        /// negative depth points the apex toward the center instead of away.
        depth: f32,
        /// Arm stroke width, in authored pixels. Each arm extends by half this
        /// width past both of its ends so the two arms join cleanly.
        thickness: f32,
    },
    /// Circular arc that bulges away from the center. The midpoint of its
    /// inner edge lies at the radial distance; its own circle can be smaller
    /// or larger than that distance.
    Arc {
        /// Outer radius of the arc's own circle, in authored pixels. Smaller
        /// radii curve more tightly while the inner edge midpoint stays fixed.
        radius: f32,
        /// Angular extent of the arc, in degrees from zero through 360, centered
        /// on the layer angle. Non-finite or zero sweeps draw nothing.
        sweep: f32,
        /// Radial stroke width, in authored pixels, extending inward from the
        /// outer radius toward the arc's own circle center. Outlines add one authored pixel.
        thickness: f32,
    },
    /// Ring centered on the crosshair, ignoring the layer's angles. Its outer
    /// radius is the authored radius plus the radial distance, so a ring that
    /// follows spread grows with weapon inaccuracy.
    Ring {
        /// Outer radius at zero radial distance, in authored pixels. The layer's
        /// gap and followed spread add to this radius before drawing.
        radius: f32,
        /// Radial stroke width, in authored pixels, extending inward from the
        /// outer radius. A width at least the radius draws a filled disk.
        thickness: f32,
    },
}
