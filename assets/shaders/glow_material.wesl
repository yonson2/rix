// File: assets/shaders/glow_material.wesl
// Version: HDR Sprite Color Glow Effect (Multiply), with a pulsing outline ([040])

import bevy_sprite_render::mesh2d::vertex_output::VertexOutput;
import bevy_sprite_render::mesh2d::view_bindings::globals;

// Uniforms struct matching the GlowMaterial fields tagged #[uniform(0)]
@group(2) @binding(0)
var<uniform> material: MaterialUniforms;

// Texture and Sampler bindings, matching field names in GlowMaterial
@group(2) @binding(1) var texture: texture_2d<f32>;
@group(2) @binding(2) var texture_sampler: sampler;

// Define the structure matching the uniform buffer contents
struct MaterialUniforms {
    intensity: f32, // Matches Rust 'intensity' field
    outline: vec4<f32>, // Matches Rust 'outline': its color, alpha 0 for none
};

// The outline's width in texels, and how much and how fast it pulses.
const OUTLINE_TEXELS: f32 = 3.0;
const PULSE_TEXELS: f32 = 1.5;
const PULSE_SPEED: f32 = 4.0;

// Fragment shader main function
@fragment
fn fragment(mesh: VertexOutput) -> @location(0) vec4<f32> {
    // --- Sample Base Color ---
    let base_color = textureSample(texture, texture_sampler, mesh.uv);

    // --- Calculate HDR Color ---
    // Multiply the original RGB values by the intensity factor.
    // This brightens the sprite's own colors.
    // Ensures intensity = 1.0 gives the original color.
    // Values > 1.0 push colors into the HDR range for bloom.
    // Use max to prevent negative intensity from inverting colors.
    let safe_intensity = max(0.0, material.intensity);
    let final_rgb = base_color.rgb * safe_intensity;

    // --- Final Alpha ---
    // Use the original base alpha to maintain the sprite's shape/transparency.
    if (base_color.a > 0.5) {
        return vec4(final_rgb, base_color.a);
    }

    // --- Outline ---
    // A clear pixel near an opaque one takes the outline color. It fades with the
    // intensity, so a blinking item blinks its outline too.
    let pulse = sin(globals.time * PULSE_SPEED) * 0.5 + 0.5;
    let reach = (OUTLINE_TEXELS + pulse * PULSE_TEXELS) / vec2<f32>(textureDimensions(texture));
    var near = 0.0;
    for (var i = 0; i < 8; i++) {
        let angle = f32(i) * 0.7853982;
        let uv = mesh.uv + vec2(cos(angle), sin(angle)) * reach;
        near = max(near, textureSampleLevel(texture, texture_sampler, uv, 0.0).a);
    }
    let edge = smoothstep(0.2, 0.6, near) * min(safe_intensity, 1.0) * material.outline.a;
    return vec4(mix(final_rgb, material.outline.rgb, edge), max(base_color.a, edge));
}
