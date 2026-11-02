// File: assets/shaders/glow_material.wesl
// Version: HDR Sprite Color Glow Effect (Multiply)

import bevy_sprite_render::mesh2d::vertex_output::VertexOutput;

// Uniforms struct matching the GlowMaterial fields tagged #[uniform(0)]
@group(2) @binding(0)
var<uniform> material: MaterialUniforms;

// Texture and Sampler bindings, matching field names in GlowMaterial
@group(2) @binding(1) var texture: texture_2d<f32>;
@group(2) @binding(2) var texture_sampler: sampler;

// Define the structure matching the uniform buffer contents
struct MaterialUniforms {
    intensity: f32, // Matches Rust 'intensity' field
};

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
    let final_alpha = base_color.a;

    // Return the brightened color with the original alpha.
    // If intensity > 1.0 and HDR is enabled, non-transparent pixels will bloom.
    return vec4(final_rgb, final_alpha);
}
