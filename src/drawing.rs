//! Bounded crosshair geometry shared by tests and the GPU backend.

use arrayvec::ArrayVec;
use bevy::prelude::*;

/// Generated primitives in paint order, using fixed storage. Borrow the slice
/// through AsRef
/// to feed another renderer without collecting shapes or allocating an
/// intermediate mesh.
#[derive(Clone, Debug, Default)]
pub struct Drawing {
    /// At most 48 primitives are emitted by the complete configuration matrix.
    pub(crate) primitives: ArrayVec<Primitive, 64>,
}

impl Drawing {
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
}
