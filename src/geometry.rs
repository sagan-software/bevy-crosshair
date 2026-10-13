//! CPU geometry with bounded storage and explicit host-state policies.

use crate::Crosshair;
use crate::CrosshairInput;
use crate::Drawing;
use crate::HitMarker;
use crate::Primitive;
use crate::Spectator;
use crate::Style;
use crate::WeaponState;
use crate::settings::BotPolicy;
use crate::settings::ObserverPolicy;
use crate::settings::Outline;
use crate::settings::Settings;
use bevy::prelude::*;

/// Chooses the observed configuration before deriving geometry.
pub(crate) fn draw(drawing: &mut Drawing, crosshair: &Crosshair, input: &CrosshairInput) {
    // The fallback remains on the stack and is borrowed only for default-bot policy.
    let fallback = Crosshair::default();
    let selected = select_crosshair(crosshair, input, &fallback);
    if selected.settings.is_enabled {
        draw_selected(drawing, selected, input);
    }
}

/// Borrows the permitted player's settings without copying its configuration.
fn select_crosshair<'a>(
    crosshair: &'a Crosshair,
    input: &'a CrosshairInput,
    fallback: &'a Crosshair,
) -> &'a Crosshair {
    let settings = &crosshair.settings;
    // Spectator relationship controls whether supplied player settings are trusted for display.
    match &input.spectator {
        Spectator::Friend(observed) if settings.observer != ObserverPolicy::Off => observed,
        Spectator::Player(observed) if settings.observer == ObserverPolicy::Everyone => observed,
        Spectator::AvailableBot if settings.bot == BotPolicy::Default => fallback,
        Spectator::UnavailableBot if settings.bot != BotPolicy::Own => fallback,
        Spectator::Local
        | Spectator::Friend(_)
        | Spectator::Player(_)
        | Spectator::AvailableBot
        | Spectator::UnavailableBot => crosshair,
    }
}

/// Resolves scope and grenade visibility before drawing ordinary geometry.
fn draw_selected(drawing: &mut Drawing, crosshair: &Crosshair, input: &CrosshairInput) {
    let settings = &crosshair.settings;
    // Scoped presentation replaces ordinary geometry before grenade policy is evaluated.
    if is_scoped(input.weapon, settings) {
        draw_scope(drawing, settings, input);
        return;
    }
    if input.weapon == WeaponState::Ironsight {
        draw_ironsight(drawing, settings);
        return;
    }
    // Equality starts the guide; the host retains ownership of the elapsed clock.
    let has_grenade_guide = match input.weapon {
        WeaponState::Grenade { kind, held } => settings
            .grenade_delay(kind)
            .is_some_and(|delay| held >= delay),
        WeaponState::Hip
        | WeaponState::Sniper
        | WeaponState::Cycling
        | WeaponState::Ready
        | WeaponState::Ironsight => false,
    };
    // The retention flag affects ordinary geometry only after the grenade delay expires.
    if !has_grenade_guide || settings.is_crosshair_kept {
        draw_shape(drawing, crosshair, input);
    }
    if has_grenade_guide {
        draw_grenade(drawing, settings, input);
    }
    if input.is_friendly && settings.is_friendly {
        draw_warning(drawing, input);
    }
}

/// Resolves scope visibility from the current weapon phase and user policy.
fn is_scoped(weapon: WeaponState, settings: &Settings) -> bool {
    match weapon {
        WeaponState::Sniper => true,
        WeaponState::Cycling => settings.is_unscope_delayed,
        WeaponState::Ready => settings.has_auto_rezoom,
        WeaponState::Hip | WeaponState::Ironsight | WeaponState::Grenade { .. } => false,
    }
}

/// Physical geometry parameters, derived once for each crosshair.
struct Shape<'a> {
    /// Source settings, borrowed throughout generation.
    settings: &'a Settings,
    /// Physical pixels per authored pixel.
    scale: f32,
    /// Center after optional host-supplied aim punch, in physical pixels.
    center: Vec2,
    /// Bar length in physical pixels.
    length: f32,
    /// Bar thickness in physical pixels, with a one-pixel minimum.
    thickness: f32,
    /// Nonnegative radial gap in physical pixels.
    gap: f32,
    /// Supplied weapon spread capped by the configured distance limit.
    spread: f32,
}

impl<'a> Shape<'a> {
    /// Converts authored measurements to physical pixels exactly once.
    fn new(settings: &'a Settings, input: &CrosshairInput) -> Self {
        // Height and every dimension are in pixels; their ratio is dimensionless.
        let scale = input.viewport.y / settings.height;
        let spread = input.spread.min((128.0 + settings.spread_limit) * scale);
        // Dynamic, static, and shot-feedback styles consume different host measurements.
        let feedback = match settings.style {
            Style::DynamicCross | Style::DynamicCircle | Style::DynamicQuad | Style::Legacy => {
                spread
            }
            Style::ShotCross => input.shot,
            Style::StaticCross
            | Style::StaticCircle
            | Style::Dot
            | Style::StaticSquare
            | Style::StaticQuad => 0.0,
        };
        Self {
            settings,
            scale,
            center: if settings.is_recoil_enabled {
                input.recoil
            } else {
                Vec2::ZERO
            },
            length: (settings.length * scale).round(),
            thickness: (settings.thickness * scale).round().max(1.0),
            gap: (settings.gap * scale + feedback).max(0.0).round(),
            spread,
        }
    }

    /// Draws one shape with its independently colored outline behind it.
    fn rectangle(&self, drawing: &mut Drawing, center: Vec2, size: Vec2, opacity: f32) {
        if size.min_element() <= 0.0 {
            return;
        }
        // Half outlines shift by half a pixel instead of expanding symmetrically.
        let (offset, expansion) = match self.settings.outline {
            Outline::None => (Vec2::ZERO, 0.0),
            Outline::Full => (Vec2::ZERO, 2.0),
            Outline::Half => (Vec2::splat(0.5), 1.0),
        };
        // Paint each outline before its matching foreground shape.
        if expansion > 0.0 {
            let border = self.settings.border();
            drawing.rectangle(
                center + offset,
                size + Vec2::splat(expansion),
                0.0,
                border.with_alpha(border.alpha * opacity),
            );
        }
        let color = self.settings.color();
        drawing.rectangle(center, size, 0.0, color.with_alpha(color.alpha * opacity));
    }

    /// Emits four bars, omitting the upper one only for T shape.
    fn bars(&self, drawing: &mut Drawing, gap: f32, length: f32, opacity: f32) {
        // Cardinal directions preserve deterministic paint order and an explicit upper bar.
        // Opposite sides use the same dimensions and differ only in their signed offset.
        for direction in [Vec2::X, -Vec2::X, Vec2::Y, -Vec2::Y] {
            if self.settings.is_t_shape && direction.y < 0.0 {
                continue;
            }
            // Horizontal and vertical bars exchange length and thickness.
            let size = if direction.x.abs() > 0.0 {
                Vec2::new(length, self.thickness)
            } else {
                Vec2::new(self.thickness, length)
            };
            self.rectangle(
                drawing,
                self.center + direction * (gap + length * 0.5),
                size,
                opacity,
            );
        }
    }

    /// Emits a circle or four curved segments in the current profile.
    fn ring(&self, drawing: &mut Drawing, radius: f32, fraction: f32, is_top_omitted: bool) {
        let radius = radius.max(self.thickness);
        // Ring outlines expand radial coverage while preserving the selected angular fraction.
        let border_size = match self.settings.outline {
            Outline::None => 0.0,
            Outline::Full => 1.0,
            Outline::Half => 0.5,
        };
        if border_size > 0.0 {
            drawing.primitives.push(Primitive::Ring {
                center: self.center,
                radius: radius + border_size,
                thickness: self.thickness + border_size * 2.0,
                fraction,
                is_top_omitted,
                color: self.settings.border(),
            });
        }
        drawing.primitives.push(Primitive::Ring {
            center: self.center,
            radius,
            thickness: self.thickness,
            fraction,
            is_top_omitted,
            color: self.settings.color(),
        });
    }
}

/// Dispatches each closed style to the geometry it owns.
fn draw_shape(drawing: &mut Drawing, crosshair: &Crosshair, input: &CrosshairInput) {
    let settings = &crosshair.settings;
    // Derive authored-to-physical scale once before dispatching the closed style vocabulary.
    let shape = Shape::new(settings, input);
    // Custom layers replace the console style, its fill and its center dot.
    if !crosshair.layers.is_empty() {
        for layer in &crosshair.layers {
            crate::layer::draw(drawing, layer, shape.center, shape.scale, input.spread);
        }
        return;
    }
    // Fill belongs to the selected player's configuration and precedes every outline and dot.
    if let Some(color) = crosshair.fill {
        draw_fill(drawing, &shape, color);
    }
    draw_style(drawing, &shape);
    // The dot style always draws its center; the optional flag augments other styles.
    if settings.has_dot || settings.style == Style::Dot {
        match crosshair.dot_shape {
            crate::DotShape::Square => {
                shape.rectangle(drawing, shape.center, Vec2::splat(shape.thickness), 1.0);
            }
            crate::DotShape::Circle => draw_dot(drawing, &shape),
        }
    }
}

/// Draws the selected bars, arcs, or square before the optional center dot.
fn draw_style(drawing: &mut Drawing, shape: &Shape<'_>) {
    // Share the validated measurements while keeping each style's geometry independent.
    let settings = shape.settings;
    match settings.style {
        Style::DynamicCross | Style::StaticCross | Style::ShotCross => {
            shape.bars(drawing, shape.gap, shape.length, 1.0)
        }
        Style::Legacy => draw_legacy(drawing, shape),
        Style::DynamicCircle | Style::StaticCircle => {
            shape.ring(drawing, shape.gap + shape.thickness, 1.0, false)
        }
        Style::DynamicQuad => {
            // Convert authored arc length to a bounded fraction of each quarter circle.
            let radius = shape.gap + shape.length + shape.thickness;
            let fraction = (shape.length / radius).clamp(0.05, 0.95);
            shape.ring(drawing, radius, fraction, settings.is_t_shape);
        }
        Style::StaticQuad => shape.ring(
            drawing,
            shape.gap + shape.thickness,
            settings.split_ratio,
            false,
        ),
        Style::StaticSquare => draw_square(drawing, shape),
        Style::Dot => {}
    }
}

/// Fills the open interior without inheriting foreground opacity.
fn draw_fill(drawing: &mut Drawing, shape: &Shape<'_>, color: LinearRgba) {
    // The square's inner edge is its gap; curved quads also include their arc-length offset.
    match shape.settings.style {
        Style::Dot => {}
        Style::StaticSquare => {
            drawing.rectangle(shape.center, Vec2::splat(shape.gap * 2.0), 0.0, color)
        }
        Style::DynamicQuad => drawing.disk(shape.center, shape.gap + shape.length, color),
        Style::DynamicCross
        | Style::DynamicCircle
        | Style::Legacy
        | Style::StaticCircle
        | Style::StaticCross
        | Style::ShotCross
        | Style::StaticQuad => {
            drawing.disk(shape.center, shape.gap, color);
        }
    }
}

/// Draws a circular center dot with the same outline width as other shapes.
fn draw_dot(drawing: &mut Drawing, shape: &Shape<'_>) {
    // Paint the larger outline disk first so the foreground retains its specified diameter.
    let border = match shape.settings.outline {
        Outline::None => 0.0,
        Outline::Full => 1.0,
        Outline::Half => 0.5,
    };
    let radius = shape.thickness * 0.5;
    if border > 0.0 {
        drawing.disk(shape.center, radius + border, shape.settings.border());
    }
    drawing.disk(shape.center, radius, shape.settings.color());
}

/// Splits legacy bars only after the configured threshold is exceeded.
fn draw_legacy(drawing: &mut Drawing, shape: &Shape<'_>) {
    // The threshold is authored in pixels and must use the same scale as bar geometry.
    let threshold = shape.settings.split_distance * shape.scale;
    if shape.spread <= threshold {
        shape.bars(drawing, shape.gap, shape.length, 1.0);
        return;
    }
    // Inner and outer lengths sum to the original length and fade independently.
    let inner = shape.length * (1.0 - shape.settings.split_ratio);
    let outer = shape.length * shape.settings.split_ratio;
    let inner_gap = (shape.settings.gap * shape.scale + threshold).max(0.0);
    shape.bars(drawing, inner_gap, inner, shape.settings.inner_alpha);
    shape.bars(
        drawing,
        shape.gap + inner,
        outer,
        shape.settings.outer_alpha,
    );
}

/// Builds the four sides of a hollow square without an intermediate mesh.
fn draw_square(drawing: &mut Drawing, shape: &Shape<'_>) {
    // Center each side on the square boundary so adjacent sides share their corners.
    let radius = shape.gap + shape.thickness * 0.5;
    // Opposite sides use the same dimensions and differ only in their signed offset.
    for direction in [Vec2::X, -Vec2::X, Vec2::Y, -Vec2::Y] {
        let size = if direction.x.abs() > 0.0 {
            Vec2::new(shape.thickness, radius * 2.0 + shape.thickness)
        } else {
            Vec2::new(radius * 2.0 + shape.thickness, shape.thickness)
        };
        shape.rectangle(drawing, shape.center + direction * radius, size, 1.0);
    }
}

/// Scope lines use physical pixels, independently of authored crosshair
/// height.
fn draw_scope(drawing: &mut Drawing, settings: &Settings, input: &CrosshairInput) {
    // Scope lines span the full physical viewport and ignore authored crosshair scaling.
    let color = LinearRgba::BLACK;
    let thickness = settings.sniper_width;
    drawing.rectangle(
        Vec2::ZERO,
        Vec2::new(input.viewport.x, thickness),
        0.0,
        color,
    );
    drawing.rectangle(
        Vec2::ZERO,
        Vec2::new(thickness, input.viewport.y),
        0.0,
        color,
    );
    // Zero spread omits the inaccuracy ring instead of producing a degenerate circle.
    if settings.is_sniper_inaccuracy_enabled && input.spread > 0.0 {
        drawing.primitives.push(Primitive::Ring {
            center: Vec2::ZERO,
            radius: input.spread,
            thickness: 1.0,
            fraction: 1.0,
            is_top_omitted: false,
            color: color.with_alpha(0.35),
        });
    }
}

/// Optical-sight color is independent unless the user enables color matching.
fn draw_ironsight(drawing: &mut Drawing, settings: &Settings) {
    // Optical sights retain their red default unless color matching is explicitly enabled.
    let color = if settings.uses_ironsight_color {
        settings.color()
    } else {
        LinearRgba::RED
    };
    let size = Vec2::splat(3.0 * settings.ironsight_scale);
    drawing.rectangle(Vec2::ZERO, size, 0.0, color);
}

/// Provides screen-spanning alignment guides after the grenade's hold delay.
fn draw_grenade(drawing: &mut Drawing, settings: &Settings, input: &CrosshairInput) {
    // These alignment ticks use viewport fractions; they do not predict grenade ballistics.
    let color = settings.color().with_alpha(0.65);
    drawing.rectangle(Vec2::ZERO, Vec2::new(input.viewport.x, 1.0), 0.0, color);
    drawing.rectangle(Vec2::ZERO, Vec2::new(1.0, input.viewport.y), 0.0, color);
    for fraction in [-0.25, -0.125, 0.125, 0.25] {
        drawing.rectangle(
            Vec2::new(0.0, input.viewport.y * fraction),
            Vec2::new(12.0, 1.0),
            0.0,
            color,
        );
    }
}

/// Draws a visible warning without borrowing Valve's icon artwork.
fn draw_warning(drawing: &mut Drawing, input: &CrosshairInput) {
    let center = Vec2::new(0.0, -24.0) + input.recoil;
    let color = LinearRgba::new(1.0, 0.25, 0.08, 1.0);
    drawing.rectangle(center, Vec2::new(3.0, 8.0), 0.0, color);
    drawing.rectangle(center + Vec2::new(0.0, 7.0), Vec2::splat(3.0), 0.0, color);
}

impl Crosshair {
    /// Generates bounded geometry without retaining the supplied input. Invalid
    /// host measurements
    /// produce an empty drawing; optional hit markers remain independent of
    /// ordinary crosshair
    /// visibility.
    ///
    /// # Examples
    ///
    /// ```
    /// use bevy_crosshair::Crosshair;
    /// use bevy_crosshair::CrosshairInput;
    /// let drawing = Crosshair::default().draw(&CrosshairInput::default(), None);
    /// assert!(!drawing.as_ref().is_empty());
    /// ```
    pub fn draw(&self, input: &CrosshairInput, marker: Option<&HitMarker>) -> Drawing {
        // Select local or observed settings before deriving visibility.
        // Convert authored pixels to physical viewport pixels exactly once.
        // Emit each outline before its foreground, then append hit markers.
        Drawing::generate(self, input, marker)
    }
}

impl Drawing {
    /// Derives geometry after rejecting invalid host input.
    pub(crate) fn generate(
        crosshair: &Crosshair,
        input: &CrosshairInput,
        marker: Option<&HitMarker>,
    ) -> Self {
        // Invalid host measurements cannot reach primitive construction or GPU uniforms.
        let mut drawing = Self::default();
        if !input.is_valid() {
            return drawing;
        }
        draw(&mut drawing, crosshair, input);
        // Hit feedback remains visible when ordinary crosshair visibility is disabled.
        if let Some(marker) = marker {
            marker.draw(&mut drawing);
        }
        drawing
    }
}
