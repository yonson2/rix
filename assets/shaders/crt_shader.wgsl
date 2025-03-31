// This shader creates a CRT (cathode ray tube) monitor effect

// Since post processing is a fullscreen effect, we use the fullscreen vertex shader provided by bevy.
// This will import a vertex shader that renders a single fullscreen triangle.
#import bevy_core_pipeline::fullscreen_vertex_shader::FullscreenVertexOutput

@group(0) @binding(0) var screen_texture: texture_2d<f32>;
@group(0) @binding(1) var texture_sampler: sampler;
struct PostProcessSettings {
    intensity: f32,
    time: f32,
#ifdef SIXTEEN_BYTE_ALIGNMENT
    // WebGL2 structs must be 16 byte aligned.
    _webgl2_padding: vec2<f32>
#endif
}
@group(0) @binding(2) var<uniform> settings: PostProcessSettings;

fn curve(uv: vec2<f32>) -> vec2<f32> {
    var curvedUv = (uv - 0.5) * 2.0;
    curvedUv *= 1.1;    
    curvedUv.x *= 1.0 + pow((abs(curvedUv.y) / 5.0), 2.0);
    curvedUv.y *= 1.0 + pow((abs(curvedUv.x) / 4.0), 2.0);
    curvedUv = (curvedUv / 2.0) + 0.5;
    curvedUv = curvedUv * 0.92 + 0.04;
    return curvedUv;
}

@fragment
fn fragment(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
    let q = in.uv;
    var uv = q;
    uv = curve(uv);
    
    let oricol = textureSample(screen_texture, texture_sampler, vec2<f32>(q.x, q.y)).rgb;
    
    var col: vec3<f32>;
    let x = sin(0.3 * settings.time + uv.y * 21.0) * sin(0.7 * settings.time + uv.y * 29.0) * sin(0.3 + 0.33 * settings.time + uv.y * 31.0) * 0.0017;

    col.r = textureSample(screen_texture, texture_sampler, vec2<f32>(x + uv.x + 0.001, uv.y + 0.001)).r + 0.05;
    col.g = textureSample(screen_texture, texture_sampler, vec2<f32>(x + uv.x + 0.000, uv.y - 0.002)).g + 0.05;
    col.b = textureSample(screen_texture, texture_sampler, vec2<f32>(x + uv.x - 0.002, uv.y + 0.000)).b + 0.05;
    col.r += 0.08 * textureSample(screen_texture, texture_sampler, 0.75 * vec2<f32>(x + 0.025, -0.027) + vec2<f32>(uv.x + 0.001, uv.y + 0.001)).r;
    col.g += 0.05 * textureSample(screen_texture, texture_sampler, 0.75 * vec2<f32>(x - 0.022, -0.02) + vec2<f32>(uv.x + 0.000, uv.y - 0.002)).g;
    col.b += 0.08 * textureSample(screen_texture, texture_sampler, 0.75 * vec2<f32>(x - 0.02, -0.018) + vec2<f32>(uv.x - 0.002, uv.y + 0.000)).b;

    col = clamp(col * 0.6 + 0.4 * col * col * 1.0, vec3<f32>(0.0), vec3<f32>(1.0));

    let vig = (0.0 + 1.0 * 16.0 * uv.x * uv.y * (1.0 - uv.x) * (1.0 - uv.y));
    col *= vec3<f32>(pow(vig, 0.3));

    col *= vec3<f32>(0.95, 1.05, 0.95);
    col *= 2.8;

    let scans = clamp(0.35 + 0.35 * sin(3.5 * settings.time + uv.y * 1080.0 * 1.5), 0.0, 1.0);
    
    let s = pow(scans, 1.7);
    col = col * vec3<f32>(0.4 + 0.7 * s);

    col *= 1.0 + 0.01 * sin(110.0 * settings.time);
    if (uv.x < 0.0 || uv.x > 1.0) {
        col *= 0.0;
    }
    if (uv.y < 0.0 || uv.y > 1.0) {
        col *= 0.0;
    }
    
    col *= 1.0 - 0.65 * vec3<f32>(clamp(((in.position.x % 2.0) - 1.0) * 2.0, 0.0, 1.0));
    
    // For comparison with original (uncomment if needed)
    // let comp = smoothstep(0.1, 0.9, sin(settings.time));
    // col = mix(col, oricol, comp);

    return vec4<f32>(col, 1.0);
}