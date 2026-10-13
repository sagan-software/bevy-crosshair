// Coordinates and distances use physical viewport pixels; angles use radians.
import bevy_ui_render::ui_vertex_output::UiVertexOutput;

struct Primitive {
    geometry: vec4<f32>,
    parameters: vec4<f32>,
    color: vec4<f32>,
};
struct Uniforms {
    dimensions: vec4<f32>,
    primitives: array<Primitive, 64>,
};
@group(1) @binding(0) var<uniform> drawing: Uniforms;

fn coverage(shape: Primitive, point: vec2<f32>) -> f32 {
    let relative = point - shape.geometry.xy;
    // Tag zero is a rotated rectangle and tag three a fading one; tag one is a
    // ring or segmented ring and tag two a single arc.
    if shape.parameters.x < 0.5 || shape.parameters.x > 2.5 {
        let angle = shape.parameters.y;
        let local = vec2<f32>(cos(angle) * relative.x + sin(angle) * relative.y,
            -sin(angle) * relative.x + cos(angle) * relative.y);
        let distance = abs(local) - shape.geometry.zw * 0.5;
        var alpha = clamp(0.5 - max(distance.x, distance.y), 0.0, 1.0);
        if shape.parameters.x > 2.5 {
            // Opacity stays full over the solid fraction from local -X, then falls to zero.
            let along = clamp(local.x / max(shape.geometry.z, 1.0) + 0.5, 0.0, 1.0);
            let solid = shape.parameters.z;
            alpha *= clamp(1.0 - (along - solid) / max(1.0 - solid, 0.0001), 0.0, 1.0);
        }
        return alpha;
    }
    let radius = length(relative);
    let outside = radius - shape.geometry.z;
    let inside = shape.geometry.z - shape.geometry.w - radius;
    var alpha = clamp(0.5 - max(outside, inside), 0.0, 1.0);
    // Angle zero points up, matching CPU geometry with positive Y downward.
    let angle = atan2(relative.x, -relative.y);
    if shape.parameters.x > 1.5 {
        // Wrap the difference to -pi..pi before comparing it with the half sweep.
        let difference = abs(atan2(sin(angle - shape.parameters.y), cos(angle - shape.parameters.y)));
        return alpha * clamp(0.5 - (difference - shape.parameters.z) * max(radius, 1.0), 0.0, 1.0);
    }
    if shape.parameters.w > 0.5 && abs(angle) < 0.785398163 {
        alpha = 0.0;
    }
    if shape.parameters.z < 1.0 {
        let quadrant = 1.570796327;
        let local = abs(angle - floor((angle + quadrant * 0.5) / quadrant) * quadrant);
        let half_angle = shape.parameters.z * quadrant * 0.5;
        alpha *= clamp(0.5 - (local - half_angle) * max(radius, 1.0), 0.0, 1.0);
    }
    return alpha;
}

@fragment
fn fragment(input: UiVertexOutput) -> @location(0) vec4<f32> {
    let point = (input.uv - vec2<f32>(0.5)) * drawing.dimensions.xy;
    var output = vec4<f32>(0.0);
    // Composite in CPU paint order using premultiplied color internally.
    for (var index = 0u; index < u32(drawing.dimensions.z); index += 1u) {
        let shape = drawing.primitives[index];
        let alpha = shape.color.a * coverage(shape, point);
        output = vec4<f32>(shape.color.rgb * alpha + output.rgb * (1.0 - alpha),
            alpha + output.a * (1.0 - alpha));
    }
    // Bevy UI blending expects straight-alpha color at the fragment boundary.
    if output.a > 0.0 { return vec4<f32>(output.rgb / output.a, output.a); }
    return output;
}
