//! Bounded crosshair geometry shared by tests and the GPU backend.

use arrayvec::ArrayVec;
use bevy::prelude::*;

/// Generated primitives in paint order, using fixed storage. Borrow the slice
/// through AsRef
/// to feed another renderer without collecting shapes or allocating an
/// intermediate mesh.
#[derive(Clone, Debug, Default)]
pub struct Drawing {
    /// Fixed capacity bounds stack storage for every supported rendering mode.
    pub(crate) primitives: ArrayVec<Primitive, 64>,
}

impl Drawing {
    /// Represents a solid disk through the ring primitive, without a center hole.
    pub(crate) fn disk(&mut self, center: Vec2, radius: f32, color: LinearRgba) {
        if radius > 0.0 && color.alpha > 0.0 {
            self.primitives.push(Primitive::Ring {
                center,
                radius,
                // Twice the radius puts the inner edge outside the disk, including its center.
                thickness: radius * 2.0,
                fraction: 1.0,
                is_top_omitted: false,
                color,
            });
        }
    }

    /// Omits empty rectangles before they reach the GPU.
    pub(crate) fn rectangle(&mut self, center: Vec2, size: Vec2, rotation: f32, color: LinearRgba) {
        if size.min_element() > 0.0 && color.alpha > 0.0 {
            self.primitives.push(Primitive::Rectangle {
                center,
                size,
                rotation,
                color,
            });
        }
    }
}

impl AsRef<[Primitive]> for Drawing {
    fn as_ref(&self) -> &[Primitive] {
        &self.primitives
    }
}

/// One filled shape in physical pixels relative to the viewport center. Shapes
/// carry linear
/// RGBA colors and appear in the order returned by Drawing.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Primitive {
    /// Axis-aligned or rotated rectangle with a filled interior. Crosshair bars,
    /// outlines, scope
    /// lines, grenade guides, and diagonal hit-marker arms all use this shape.
    Rectangle {
        /// Rectangle center in physical pixels relative to the viewport center.
        /// Positive X moves
        /// right and positive Y moves down before clockwise rotation is applied.
        center: Vec2,
        /// Width and height in physical pixels before rotation. Geometry generation
        /// omits rectangles
        /// with a nonpositive dimension, keeping invisible shapes out of the returned
        /// drawing.
        size: Vec2,
        /// Clockwise rotation in radians around the rectangle center. Ordinary
        /// crosshair bars use
        /// zero rotation; diagonal hit-marker arms rotate without changing their stored
        /// dimensions.
        rotation: f32,
        /// Linear RGBA color for GPU blending, including opacity. The renderer
        /// composites each shape
        /// over earlier shapes; callers should preserve the returned paint order.
        color: LinearRgba,
    },
    /// Ring or four angular segments of a ring, sharing one center and radius. The
    /// angular
    /// fraction controls the filled portion of each cardinal quadrant.
    Ring {
        /// Ring center in physical pixels relative to the viewport center. Positive Y
        /// moves down;
        /// recoil shifts this center only when the configured style follows recoil.
        center: Vec2,
        /// Outer radius in physical pixels, measured from the ring center. Radial
        /// thickness extends
        /// inward from this boundary, and outlined rings include a separate border
        /// shape.
        radius: f32,
        /// Radial thickness in physical pixels, extending inward from the outer radius.
        /// The crosshair
        /// generator keeps ordinary ring thickness at least one physical pixel.
        thickness: f32,
        /// Angular fraction filled per quadrant, from zero to one. One produces a
        /// complete circle;
        /// smaller values retain symmetric arcs centered on the four cardinal
        /// directions.
        fraction: f32,
        /// Whether the quadrant centered above the crosshair is omitted. Dynamic quad
        /// uses this flag
        /// for its T shape while other ring styles retain their upper quadrant.
        is_top_omitted: bool,
        /// Linear RGBA color for GPU blending, including opacity. The renderer
        /// composites each shape
        /// over earlier shapes; callers should preserve the returned paint order.
        color: LinearRgba,
    },
    /// One circular arc of a ring, centered on an arbitrary direction. Custom
    /// layers use it for brackets and other curved marks.
    Arc {
        /// Center of the arc's own circle in physical pixels relative to the
        /// viewport center, with positive Y pointing down. It need not be the
        /// crosshair center.
        center: Vec2,
        /// Outer radius in physical pixels, measured from the arc's own circle
        /// center. Radial thickness extends inward from this outer boundary toward that center.
        radius: f32,
        /// Radial thickness in physical pixels, extending inward from the outer
        /// radius. Outlined arcs paint a thicker arc behind the foreground.
        thickness: f32,
        /// Direction of the arc's midpoint in radians, clockwise from straight
        /// up, measured around the arc's own circle center rather than the viewport center.
        direction: f32,
        /// Half of the arc's angular extent in radians, from zero through pi.
        /// The arc covers this angle on both sides of its direction.
        half_sweep: f32,
        /// Linear RGBA color for GPU blending, including opacity. The renderer
        /// composites each shape over all earlier shapes in the returned paint order.
        color: LinearRgba,
    },
    /// Rotated rectangle whose opacity fades along its length. The local
    /// negative-X half starts opaque; opacity falls linearly to zero at the
    /// local positive-X end after the solid fraction.
    FadingRectangle {
        /// Rectangle center in physical pixels relative to the viewport center.
        /// Positive X moves right and positive Y moves down before rotation.
        center: Vec2,
        /// Width along the fade and height across it, in physical pixels,
        /// before rotation. Geometry generation omits rectangles with a nonpositive dimension.
        size: Vec2,
        /// Clockwise rotation in radians around the rectangle center. The fade
        /// direction always rotates together with the rectangle's own local positive-X axis.
        rotation: f32,
        /// Fraction of the width at full opacity, from zero through one,
        /// measured from the local negative-X end. The rest fades to zero.
        solid: f32,
        /// Linear RGBA color at full opacity, before the fade multiplies its
        /// alpha. The renderer composites each shape over earlier shapes in paint order.
        color: LinearRgba,
    },
}
