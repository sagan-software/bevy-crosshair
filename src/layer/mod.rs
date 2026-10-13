//! Custom crosshair layers that replace the console style geometry.

use arrayvec::ArrayVec;
use bevy::prelude::*;

pub use self::mark::Mark;

/// One custom shape layer: a [`Mark`] repeated at up to four angles, with its
/// own color, optional outline and optional spread response.
///
/// Adding any layer to a [`Crosshair`](crate::Crosshair) replaces its console
/// style, fill and center dot. Visibility, screen-height scaling, recoil and
/// the scope, ironsight and grenade states still follow the console settings.
/// Console export excludes layers.
///
/// # Examples
///
/// ```
/// use bevy::color::LinearRgba;
/// use bevy_crosshair::{Crosshair, Layer, Mark};
/// let wires = Layer::new(Mark::Bar { length: 17.0, thickness: 2.0, solid: 0.3 })
///     .at_angles(&[0.0, 90.0, 180.0, 270.0])
///     .following_spread(4.0, 64.0)
///     .with_color(LinearRgba::WHITE);
/// let crosshair = Crosshair::default().with_layer(wires);
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct Layer {
    /// Glyph repeated at each angle.
    pub(crate) mark: Mark,
    /// Directions in degrees clockwise from straight up.
    pub(crate) angles: ArrayVec<f32, MAX_ANGLES>,
    /// Radial distance before spread, in authored pixels.
    pub(crate) gap: f32,
    /// Spread clamp range in authored pixels, absent for a static layer.
    pub(crate) spread: Option<(f32, f32)>,
    /// Linear foreground color, including opacity.
    pub(crate) color: LinearRgba,
    /// Linear outline color, absent when the layer has no outline.
    pub(crate) outline: Option<LinearRgba>,
}

mod mark;

/// Most angles one layer repeats its mark at.
pub(crate) const MAX_ANGLES: usize = 4;

/// Most layers one crosshair draws. Three outlined layers of four chevrons use
/// 48 primitives, leaving room for grenade guides and a hit marker in the
/// 64-primitive drawing.
pub(crate) const MAX_LAYERS: usize = 3;

impl Layer {
    /// Creates a static white layer that draws the mark once, straight up,
    /// with no gap, no outline and no spread response. Chain the other
    /// builders to place, color and animate it.
    ///
    /// # Examples
    ///
    /// ```
    /// use bevy_crosshair::{Layer, Mark};
    /// let dot = Layer::new(Mark::Dot { diameter: 2.0 });
    /// ```
    pub fn new(mark: Mark) -> Self {
        let mut angles = ArrayVec::new();
        angles.push(0.0);
        Self {
            mark,
            angles,
            gap: 0.0,
            spread: None,
            color: LinearRgba::WHITE,
            outline: None,
        }
    }

    /// Replaces the directions, in degrees clockwise from straight up. Only
    /// the first four finite angles are kept, in the given paint order; an
    /// empty or fully non-finite list leaves the layer without marks.
    ///
    /// # Examples
    ///
    /// ```
    /// use bevy_crosshair::{Layer, Mark};
    /// let tri = Layer::new(Mark::Dot { diameter: 2.0 }).at_angles(&[0.0, 120.0, 240.0]);
    /// ```
    pub fn at_angles(mut self, angles: &[f32]) -> Self {
        // Reject non-finite directions before they reach trigonometry.
        self.angles = angles
            .iter()
            .copied()
            .filter(|angle| angle.is_finite())
            .take(MAX_ANGLES)
            .collect();
        self
    }

    /// Sets the radial distance before spread, in authored pixels. Negative
    /// gaps move marks past the center toward the opposite side; non-finite
    /// gaps become zero so geometry stays finite.
    ///
    /// # Examples
    ///
    /// ```
    /// use bevy_crosshair::{Layer, Mark};
    /// let ring_of_dots = Layer::new(Mark::Dot { diameter: 2.0 }).with_gap(6.0);
    /// ```
    pub fn with_gap(mut self, gap: f32) -> Self {
        self.gap = if gap.is_finite() { gap } else { 0.0 };
        self
    }

    /// Makes the layer follow the host's spread input. The supplied physical
    /// spread is clamped to `min..=max` authored pixels, after conversion to
    /// physical pixels, and added to the gap. Invalid or reversed bounds leave
    /// the layer static.
    ///
    /// # Examples
    ///
    /// ```
    /// use bevy_crosshair::{Layer, Mark};
    /// let ring = Layer::new(Mark::Ring { radius: 0.0, thickness: 1.0 })
    ///     .following_spread(4.0, 64.0);
    /// ```
    pub fn following_spread(mut self, min: f32, max: f32) -> Self {
        // Validate both bounds together so clamping can never panic.
        let is_valid = min.is_finite() && max.is_finite() && (0.0..=max).contains(&min);
        self.spread = is_valid.then_some((min, max));
        self
    }

    /// Sets the foreground color of every mark in the layer. Non-finite colors
    /// become transparent and alpha is clamped to zero through one, so a
    /// transparent layer draws nothing.
    ///
    /// # Examples
    ///
    /// ```
    /// use bevy::color::LinearRgba;
    /// use bevy_crosshair::{Layer, Mark};
    /// let red = Layer::new(Mark::Dot { diameter: 2.0 }).with_color(LinearRgba::RED);
    /// ```
    pub fn with_color(mut self, color: LinearRgba) -> Self {
        self.color = valid_color(color);
        self
    }

    /// Draws a one-authored-pixel outline in this color behind every mark of
    /// the layer, never thinner than one physical pixel. Non-finite colors
    /// become transparent.
    ///
    /// # Examples
    ///
    /// ```
    /// use bevy::color::LinearRgba;
    /// use bevy_crosshair::{Layer, Mark};
    /// let outlined = Layer::new(Mark::Dot { diameter: 2.0 }).with_outline(LinearRgba::BLACK);
    /// ```
    pub fn with_outline(mut self, color: LinearRgba) -> Self {
        self.outline = Some(valid_color(color));
        self
    }
}

/// Replaces non-finite channels with transparency and clamps alpha.
fn valid_color(color: LinearRgba) -> LinearRgba {
    if color.to_vec4().is_finite() {
        color.with_alpha(color.alpha.clamp(0.0, 1.0))
    } else {
        LinearRgba::NONE
    }
}

/// Converts one layer to physical primitives, outlines first.
///
/// `scale` is physical pixels per authored pixel; `spread` is the host's
/// physical spread input.
pub(crate) fn draw(
    drawing: &mut crate::Drawing,
    layer: &Layer,
    center: Vec2,
    scale: f32,
    spread: f32,
) {
    // Clamp spread in physical pixels so the bounds follow screen-height scaling.
    let followed = layer
        .spread
        .map_or(0.0, |(min, max)| spread.clamp(min * scale, max * scale));
    // Rings ignore direction, so they use only the first angle.
    let angles = match layer.mark {
        Mark::Ring { .. } => &layer.angles[..layer.angles.len().min(1)],
        Mark::Dot { .. } | Mark::Bar { .. } | Mark::Chevron { .. } | Mark::Arc { .. } => {
            &layer.angles[..]
        }
    };
    let mut pen = Pen {
        center,
        scale,
        distance: layer.gap * scale + followed,
        grow: 0.0,
        color: layer.color,
    };
    // Paint every outline before any foreground so joined marks keep clean seams.
    // Outlines expand every mark by one authored pixel, never below one physical pixel.
    if let Some(outline) = layer.outline {
        let foreground = (pen.grow, pen.color);
        (pen.grow, pen.color) = (scale.max(1.0), outline);
        angles
            .iter()
            .for_each(|angle| pen.emit(drawing, layer.mark, *angle));
        (pen.grow, pen.color) = foreground;
    }
    angles
        .iter()
        .for_each(|angle| pen.emit(drawing, layer.mark, *angle));
}

/// Resolved physical measurements shared by every mark of one layer pass.
struct Pen {
    /// Crosshair center after recoil, in physical pixels.
    center: Vec2,
    /// Physical pixels per authored pixel.
    scale: f32,
    /// Radial distance of each mark's anchor, in physical pixels.
    distance: f32,
    /// Outline growth on each side, in physical pixels; zero for foreground.
    grow: f32,
    /// Color of this pass.
    color: LinearRgba,
}

/// Unit directions for one angle, in screen space with positive Y down.
struct Direction {
    /// The angle in radians, clockwise from straight up.
    angle: f32,
    /// Unit vector pointing away from the center.
    outward: Vec2,
    /// Unit vector a quarter turn clockwise from `outward`.
    across: Vec2,
}

impl Pen {
    /// Emits one mark at one angle in degrees.
    fn emit(&self, drawing: &mut crate::Drawing, mark: Mark, degrees: f32) {
        let angle = degrees.to_radians();
        let direction = Direction {
            angle,
            outward: Vec2::new(angle.sin(), -angle.cos()),
            across: Vec2::new(angle.cos(), angle.sin()),
        };
        // Each closed mark owns its geometry; all share the anchor distance.
        match mark {
            Mark::Dot { diameter } => drawing.disk(
                self.center + direction.outward * self.distance,
                diameter * self.scale * 0.5 + self.grow,
                self.color,
            ),
            Mark::Bar {
                length,
                thickness,
                solid,
            } => self.bar(drawing, &direction, Vec2::new(length, thickness), solid),
            Mark::Chevron {
                width,
                depth,
                thickness,
            } => self.chevron(drawing, &direction, Vec2::new(width, depth), thickness),
            Mark::Arc {
                radius,
                sweep,
                thickness,
            } => self.arc(drawing, &direction, Vec3::new(radius, sweep, thickness)),
            Mark::Ring { radius, thickness } => self.ring(drawing, radius, thickness),
        }
    }

    /// Emits a bar from the anchor outward; `size` is authored length and width.
    fn bar(&self, drawing: &mut crate::Drawing, direction: &Direction, size: Vec2, solid: f32) {
        let size = size * self.scale;
        let middle = self.center + direction.outward * (self.distance + size.x * 0.5);
        let size = size + Vec2::splat(self.grow * 2.0);
        // A rectangle's local +X axis points outward after this clockwise rotation.
        let rotation = direction.angle - std::f32::consts::FRAC_PI_2;
        let solid = if solid.is_finite() {
            solid.clamp(0.0, 1.0)
        } else {
            1.0
        };
        // A fully solid bar needs no gradient evaluation on the GPU.
        if solid >= 1.0 {
            drawing.rectangle(middle, size, rotation, self.color);
        } else if size.min_element() > 0.0 && self.color.alpha > 0.0 {
            drawing.primitives.push(crate::Primitive::FadingRectangle {
                center: middle,
                size,
                rotation,
                solid,
                color: self.color,
            });
        }
    }

    /// Emits two arms meeting at the apex; `extent` is authored width and depth.
    fn chevron(
        &self,
        drawing: &mut crate::Drawing,
        direction: &Direction,
        extent: Vec2,
        thickness: f32,
    ) {
        // Convert authored measurements once; the base is the midpoint between both arm ends.
        let extent = extent * self.scale;
        let thickness = thickness * self.scale;
        let base = self.center + direction.outward * self.distance;
        let apex = base + direction.outward * extent.y;
        for side in [-1.0, 1.0] {
            let end = base + direction.across * (side * extent.x * 0.5);
            let arm = apex - end;
            // Extend each arm by half its thickness at both ends so the arms meet without a notch.
            let size =
                Vec2::new(arm.length() + thickness, thickness) + Vec2::splat(self.grow * 2.0);
            drawing.rectangle((apex + end) * 0.5, size, arm.to_angle(), self.color);
        }
    }

    /// Emits an arc whose inner edge midpoint lies at the anchor; `shape` holds
    /// the authored radius, the sweep in degrees and the authored thickness.
    fn arc(&self, drawing: &mut crate::Drawing, direction: &Direction, shape: Vec3) {
        let radius = shape.x * self.scale;
        let thickness = shape.z * self.scale;
        // Move the arc's own circle inward so its inner edge touches the anchor.
        let circle = self.center + direction.outward * (self.distance - (radius - thickness));
        let half_sweep = if shape.y.is_finite() {
            shape.y.clamp(0.0, 360.0).to_radians() * 0.5
        } else {
            0.0
        };
        // Degenerate or transparent arcs never reach the GPU.
        if radius > 0.0 && thickness > 0.0 && half_sweep > 0.0 && self.color.alpha > 0.0 {
            drawing.primitives.push(crate::Primitive::Arc {
                center: circle,
                radius: radius + self.grow,
                thickness: thickness + self.grow * 2.0,
                direction: direction.angle,
                half_sweep,
                color: self.color,
            });
        }
    }

    /// Emits a centered ring whose radius grows with the anchor distance.
    fn ring(&self, drawing: &mut crate::Drawing, radius: f32, thickness: f32) {
        let radius = radius * self.scale + self.distance;
        let thickness = thickness * self.scale;
        // Degenerate or transparent rings never reach the GPU.
        if radius > 0.0 && thickness > 0.0 && self.color.alpha > 0.0 {
            drawing.primitives.push(crate::Primitive::Ring {
                center: self.center,
                radius: radius + self.grow,
                thickness: thickness + self.grow * 2.0,
                fraction: 1.0,
                is_top_omitted: false,
                color: self.color,
            });
        }
    }
}

#[cfg(test)]
mod tests {
    //! Private outline-width invariant, separate from the public geometry tests.

    use super::Layer;
    use super::Mark;
    use bevy::prelude::*;

    #[test]
    fn outlines_keep_one_physical_pixel_on_small_screens() {
        // At a quarter scale the authored one-pixel outline would vanish without the minimum.
        let layer = Layer::new(Mark::Dot { diameter: 4.0 }).with_outline(LinearRgba::BLACK);
        let mut drawing = crate::Drawing::default();
        super::draw(&mut drawing, &layer, Vec2::ZERO, 0.25, 0.0);
        let radii: Vec<f32> = drawing
            .as_ref()
            .iter()
            .map(|primitive| match primitive {
                crate::Primitive::Ring { radius, .. } => *radius,
                other => panic!("unexpected {other:?}"),
            })
            .collect();
        assert_eq!(radii, [1.5, 0.5]);
    }
}
