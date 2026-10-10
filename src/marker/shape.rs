//! Closed hit-marker shapes sharing one size, gap, color, and lifetime.

/// Geometry for a hit marker. Shape selection does not change hit-event delivery
/// or the game's metadata, and works with every marker animation.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum MarkerShape {
    /// Four diagonal arms pointing away from the central gap.
    /// Each arm retains the configured length and thickness as animation changes
    /// its distance from the center.
    #[default]
    Cross,
    /// Four horizontal and vertical arms pointing away from the central gap.
    /// Each arm retains the configured length and thickness while radial motion
    /// changes its center.
    Plus,
    /// Two diagonal arms forming a downward-pointing chevron above the center.
    /// The arms retain their length and thickness; animation moves their shared
    /// tip vertically toward or away from the center.
    Chevron,
    /// Four connected diagonal sides surrounding the central gap.
    /// The vertices lie on the horizontal and vertical axes; animation changes
    /// side length while retaining stroke thickness.
    Diamond,
    /// Complete circular ring surrounding the central gap.
    /// Animation changes its outer radius while retaining radial stroke thickness;
    /// the center remains fixed at the aiming point.
    Ring,
}
