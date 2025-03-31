// This shader creates a simplified CRT (cathode ray tube) monitor effect

// Since post processing is a fullscreen effect, we use the fullscreen vertex shader provided by bevy.
// This will import a vertex shader that renders a single fullscreen triangle.
#import bevy_core_pipeline::fullscreen_vertex_shader::FullscreenVertexOutput

@group(0) @binding(0) var screen_texture: texture_2d<f32>;
@group(0) @binding(1) var texture_sampler: sampler;
struct PostProcessSettings {
    intensity: f32,  // Controls the overall CRT effect strength
    time: f32,       // Used for animated effects
#ifdef SIXTEEN_BYTE_ALIGNMENT
    // WebGL2 structs must be 16 byte aligned.
    _webgl2_padding: vec2<f32>
#endif
}
@group(0) @binding(2) var<uniform> settings: PostProcessSettings;

@fragment
fn fragment(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
    let q = in.uv;
    var uv = q;
    
    // Get the original color for reference
    let oricol = textureSample(screen_texture, texture_sampler, vec2<f32>(q.x, q.y)).rgb;
    
    var col: vec3<f32>;
    
    // Subtle distortion effect - controlled by intensity
    let distortion_amount = settings.intensity * 0.0017;
    let x = sin(0.3 * settings.time + uv.y * 21.0) * sin(0.7 * settings.time + uv.y * 29.0) * sin(0.3 + 0.33 * settings.time + uv.y * 31.0) * distortion_amount;
    
    // Color ghosting - controlled by intensity
    // Lower intensity = less color separation
    let rgb_offset = settings.intensity * 0.001;
    
    col.r = textureSample(screen_texture, texture_sampler, vec2<f32>(x + uv.x + rgb_offset, uv.y + rgb_offset)).r + 0.05;
    col.g = textureSample(screen_texture, texture_sampler, vec2<f32>(x + uv.x, uv.y - rgb_offset)).g + 0.05;
    col.b = textureSample(screen_texture, texture_sampler, vec2<f32>(x + uv.x - rgb_offset, uv.y)).b + 0.05;
    
    // Secondary sampling
    let secondary_intensity = settings.intensity * 0.009;
    col.r += secondary_intensity * textureSample(screen_texture, texture_sampler, 0.75 * vec2<f32>(x + 0.025, -0.027) + vec2<f32>(uv.x + rgb_offset, uv.y + rgb_offset)).r;
    col.g += (secondary_intensity * 0.625) * textureSample(screen_texture, texture_sampler, 0.75 * vec2<f32>(x - 0.022, -0.02) + vec2<f32>(uv.x, uv.y - rgb_offset)).g;
    col.b += secondary_intensity * textureSample(screen_texture, texture_sampler, 0.75 * vec2<f32>(x - 0.02, -0.018) + vec2<f32>(uv.x - rgb_offset, uv.y)).b;

    // Color adjustments
    col = clamp(col * 0.6 + 0.4 * col * col * 1.0, vec3<f32>(0.0), vec3<f32>(1.0));
    
    // Vignette effect - controlled by intensity
    let vig = (0.0 + 1.0 * 16.0 * uv.x * uv.y * (1.0 - uv.x) * (1.0 - uv.y));
    let vignette_strength = settings.intensity * 0.5;
    col *= vec3<f32>(pow(vig, 0.3) * vignette_strength + (1.0 - vignette_strength));
    
    // Color tint
    col *= vec3<f32>(0.95, 1.05, 0.95);
    col *= 2.8;
    
    // Scanlines - controlled by intensity
    let scanline_frequency = 1080.0 * 1.5; 
    let scans = clamp(0.35 + 0.35 * sin(3.5 * settings.time + uv.y * scanline_frequency), 0.0, 1.0);
    
    let s = pow(scans, 1.7);
    let scanline_strength = settings.intensity * 0.7;
    col = col * vec3<f32>(1.0 - scanline_strength + scanline_strength * (0.4 + 0.6 * s));
    
    // Subtle flicker
    col *= 1.0 + 0.01 * sin(110.0 * settings.time);
    
    // Pixel grid effect - controlled by intensity
    let pixel_effect = settings.intensity * 0.3;
    col *= 1.0 - pixel_effect * vec3<f32>(clamp(((in.position.x % 2.0) - 1.0) * 2.0, 0.0, 1.0));
    
    // When intensity is very low, fade to original color to disable the effect entirely
    if (settings.intensity < 0.05) {
        let blend_factor = settings.intensity / 0.05;
        col = mix(oricol, col, blend_factor);
    }
    
    return vec4<f32>(col, 1.0);
}
