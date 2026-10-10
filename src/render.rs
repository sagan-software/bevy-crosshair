//! Optional Bevy UI material backend with one bounded quad per crosshair.

use crate::Crosshair;
use crate::CrosshairInput;
use crate::HitMarker;
use crate::Primitive;
use bevy::asset::embedded_asset;
use bevy::prelude::*;
use bevy::render::render_resource::AsBindGroup;
use bevy::render::render_resource::ShaderType;
use bevy::shader::ShaderRef;
use bevy::ui::UiSystems;

/// Installs GPU assets only when the host has installed Bevy's asset
/// infrastructure.
pub(crate) fn install(app: &mut App) {
    embedded_asset!(app, "crosshair.wesl");
    app.add_plugins(UiMaterialPlugin::<CrosshairMaterial>::default())
        .add_systems(
            PostUpdate,
            (attach, update).chain().before(UiSystems::Layout),
        );
}

/// Fixed uniform record shared with the WGSL shader, with 16-byte alignment.
#[derive(Clone, Copy, Debug, Default, ShaderType, PartialEq)]
struct GpuPrimitive {
    /// Center followed by rectangle size or ring radius and thickness, in pixels.
    geometry: Vec4,
    /// Shape tag, rotation, angular fraction, and omitted-upper-quadrant flag.
    parameters: Vec4,
    /// Linear RGBA color.
    color: Vec4,
}

/// One draw call's bounded uniform buffer.
#[derive(Clone, Copy, Debug, ShaderType, PartialEq)]
struct Uniforms {
    /// Quad width, height, populated primitive count, and padding.
    dimensions: Vec4,
    /// Fixed-capacity primitive storage; only the populated prefix is evaluated.
    primitives: [GpuPrimitive; 64],
}

impl Default for Uniforms {
    fn default() -> Self {
        Self {
            dimensions: Vec4::ZERO,
            primitives: [GpuPrimitive::default(); 64],
        }
    }
}

/// One UI material per crosshair, reused across frames.
#[derive(AsBindGroup, Asset, TypePath, Clone, Debug)]
struct CrosshairMaterial {
    /// Complete draw description uploaded only when its value changes.
    #[uniform(0)]
    uniforms: Uniforms,
}

impl UiMaterial for CrosshairMaterial {
    fn fragment_shader() -> ShaderRef {
        "embedded://bevy_crosshair/crosshair.wesl".into()
    }
}

/// Allocates the material once for each newly spawned crosshair.
fn attach(
    mut commands: Commands<'_, '_>,
    mut materials: ResMut<'_, Assets<CrosshairMaterial>>,
    crosshairs: Query<'_, '_, Entity, (With<Crosshair>, Without<MaterialNode<CrosshairMaterial>>)>,
) {
    for entity in &crosshairs {
        commands.entity(entity).insert((
            Node {
                position_type: PositionType::Absolute,
                left: percent(50),
                top: percent(50),
                ..default()
            },
            UiTransform::from_translation(Val2::percent(-50.0, -50.0)),
            GlobalZIndex(100),
            MaterialNode(materials.add(CrosshairMaterial {
                uniforms: Uniforms::default(),
            })),
        ));
    }
}

/// Converts bounded geometry and leaves unchanged assets untouched.
fn update(
    mut materials: ResMut<'_, Assets<CrosshairMaterial>>,
    crosshairs: Query<'_, '_, (Entity, &Crosshair, &CrosshairInput, &HitMarker)>,
    mut nodes: Query<'_, '_, MaterialView>,
) {
    for (entity, crosshair, input, marker) in &crosshairs {
        // Newly spawned crosshairs receive their material after deferred commands apply.
        let Ok((handle, computed, mut node)) = nodes.get_mut(entity) else {
            continue;
        };
        // A marker may change while the underlying crosshair settings remain unchanged.
        let drawing = crosshair.draw(input, Some(marker));
        let uniforms = encode(drawing.as_ref(), input.viewport);
        // Asset mutation triggers GPU uploads, so stable drawings leave the asset untouched.
        if let Some(mut material) = materials.get_mut(handle.id())
            && material.uniforms != uniforms
        {
            material.uniforms = uniforms;
        }
        // Bevy layout uses logical pixels; shader coordinates retain physical pixels.
        let inverse_scale = computed.inverse_scale_factor.max(f32::EPSILON);
        let width = px(uniforms.dimensions.x * inverse_scale);
        let height = px(uniforms.dimensions.y * inverse_scale);
        if node.width != width || node.height != height {
            node.width = width;
            node.height = height;
        }
    }
}

/// Components borrowed while synchronizing one material with its physical viewport.
type MaterialView = (
    &'static MaterialNode<CrosshairMaterial>,
    &'static ComputedNode,
    &'static mut Node,
);

/// Bounds the fragment workload to visible geometry instead of the whole
/// viewport.
fn encode(primitives: &[Primitive], viewport: Vec2) -> Uniforms {
    // Retain a one-pixel quad for empty geometry so layout dimensions stay finite.
    let mut uniforms = Uniforms::default();
    let mut extent = Vec2::ONE;
    for (source, destination) in primitives.iter().zip(&mut uniforms.primitives) {
        *destination = source.into();
        // One physical pixel of padding retains antialiased edge coverage.
        extent = extent.max(bounds(source) + Vec2::ONE);
    }
    let size = (extent * 2.0).min(viewport.max(Vec2::ONE));
    let count = u8::try_from(primitives.len()).expect("Drawing capacity is 64 primitives");
    uniforms.dimensions = size.extend(f32::from(count)).extend(0.0);
    uniforms
}

impl From<&Primitive> for GpuPrimitive {
    fn from(source: &Primitive) -> Self {
        // Match the shader's fixed tags and retain physical pixel measurements unchanged.
        match *source {
            Primitive::Rectangle {
                center,
                size,
                rotation,
                color,
            } => Self {
                geometry: center.extend(size.x).extend(size.y),
                parameters: Vec4::new(0.0, rotation, 0.0, 0.0),
                color: color.to_vec4(),
            },
            Primitive::Ring {
                center,
                radius,
                thickness,
                fraction,
                is_top_omitted,
                color,
            } => Self {
                geometry: center.extend(radius).extend(thickness),
                parameters: Vec4::new(1.0, 0.0, fraction, f32::from(u8::from(is_top_omitted))),
                color: color.to_vec4(),
            },
        }
    }
}

/// Returns positive half-extents around the viewport center, in physical
/// pixels.
fn bounds(source: &Primitive) -> Vec2 {
    match *source {
        Primitive::Rectangle {
            center,
            size,
            rotation,
            ..
        } => {
            // Absolute sine and cosine project rotated width and height onto screen axes.
            let (sin, cos) = rotation.sin_cos();
            center.abs()
                + Vec2::new(
                    cos.abs() * size.x + sin.abs() * size.y,
                    sin.abs() * size.x + cos.abs() * size.y,
                ) * 0.5
        }
        Primitive::Ring { center, radius, .. } => center.abs() + Vec2::splat(radius),
    }
}
