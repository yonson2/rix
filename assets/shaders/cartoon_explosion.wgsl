#import bevy_sprite::mesh2d_vertex_output::VertexOutput

// Uniforms from Bevy
@group(2) @binding(0) var<uniform> time: f32;
// resolution uniform REMOVED

// --- Palettes (Unchanged) ---
const boom_pal: array<vec3<f32>, 4> = array<vec3<f32>, 4>(vec3<f32>(0.2, 0.15, 0.3), vec3<f32>(0.9, 0.15, 0.05), vec3<f32>(0.9, 0.5, 0.1), vec3<f32>(0.95, 0.95, 0.35));
const BOOM_PAL_LEN: u32 = 4u;
const smoke_pal: array<vec3<f32>, 3> = array<vec3<f32>, 3>(vec3<f32>(0.2, 0.15, 0.3), vec3<f32>(0.35, 0.3, 0.45), vec3<f32>(0.5, 0.45, 0.6));
const SMOKE_PAL_LEN: u32 = 3u;

// --- Helper Functions (Unchanged: fmod, n_rand3, noise, worley) ---
fn fmod(x: f32, y: f32) -> f32 { return x - y * floor(x / y); }
fn n_rand3(p: vec3<f32>) -> vec3<f32> { let r = fract(sin(vec3<f32>(dot(p, vec3<f32>(127.1, 311.7, 371.8)), dot(p, vec3<f32>(269.5, 183.3, 456.1)), dot(p, vec3<f32>(352.5, 207.3, 198.67)))) * 43758.5453) * 2.0 - 1.0; return normalize(r); }
fn noise(p: vec3<f32>) -> f32 { let fv = fract(p); let nv = floor(p); let u = fv * fv * fv * (fv * (fv * 6.0 - 15.0) + 10.0); return mix(mix(mix(dot(n_rand3(nv + vec3<f32>(0.0, 0.0, 0.0)), fv - vec3<f32>(0.0, 0.0, 0.0)), dot(n_rand3(nv + vec3<f32>(1.0, 0.0, 0.0)), fv - vec3<f32>(1.0, 0.0, 0.0)), u.x), mix(dot(n_rand3(nv + vec3<f32>(0.0, 1.0, 0.0)), fv - vec3<f32>(0.0, 1.0, 0.0)), dot(n_rand3(nv + vec3<f32>(1.0, 1.0, 0.0)), fv - vec3<f32>(1.0, 1.0, 0.0)), u.x), u.y), mix(mix(dot(n_rand3(nv + vec3<f32>(0.0, 0.0, 1.0)), fv - vec3<f32>(0.0, 0.0, 1.0)), dot(n_rand3(nv + vec3<f32>(1.0, 0.0, 1.0)), fv - vec3<f32>(1.0, 0.0, 1.0)), u.x), mix(dot(n_rand3(nv + vec3<f32>(0.0, 1.0, 1.0)), fv - vec3<f32>(0.0, 1.0, 1.0)), dot(n_rand3(nv + vec3<f32>(1.0, 1.0, 1.0)), fv - vec3<f32>(1.0, 1.0, 1.0)), u.x), u.y), u.z); }
fn worley(s: vec3<f32>) -> f32 { let si = floor(s); let sf = fract(s); var m_dist = 1.0; for (var y: i32 = -1; y <= 1; y = y + 1) { for (var x: i32 = -1; x <= 1; x = x + 1) { for (var z: i32 = -1; z <= 1; z = z + 1) { let neighbor = vec3<f32>(f32(x), f32(y), f32(z)); var point = fract(n_rand3(si + neighbor)); point = 0.5 + 0.5 * sin(time + 6.2831 * point); let diff = neighbor + point - sf; let dist = length(diff); m_dist = min(m_dist, dist); } } } return m_dist; }

// Posterize function with 'fn' keyword fix
fn posterize(v: f32, n: u32) -> f32 {
    if (n <= 1u) { return 0.0; }
    let num_levels_f32 = f32(n);
    let clamped_v = max(0.0, v);
    let denominator = max(1.0, num_levels_f32 - 1.0);
    return clamp(floor(clamped_v * num_levels_f32) / denominator, 0.0, 1.0);
}

// --- Shape and Effect Functions (No aspect parameter) ---
fn boom(p: vec2<f32>) -> f32 {
    let repeat = fmod(time, 2.0);
    let shape_divisor = mix(0.01, 0.6, repeat / 2.0); // Tune?
    let shape = 1.0 - pow(length(p), 2.0) / shape_divisor - repeat * 0.5;
    let noise_scale = 8.0; // Tune?
    let worley_scale = 10.0; // Tune?
    let distortion = noise(vec3<f32>(p * noise_scale, time * 0.5));
    let bubbles = 0.5 - pow(worley(vec3<f32>(p * worley_scale, time * 2.0)), 3.0);
    let bw = 0.5;
    let effects = (bw * bubbles + (1.0 - bw) * distortion) * 0.5; // Tune?
    return shape + effects;
}

fn smoke(p: vec2<f32>) -> f32 {
    let repeat = fmod(time, 2.0);
    let smoke_center_offset = vec2<f32>(0.0, 0.5) * pow(repeat / 1.45, 2.0) * 1.5;
    let p_offset = p - smoke_center_offset;
    let shape_divisor = mix(0.02, 0.8, repeat / 2.0); // Tune?
    let shape = 1.0 - pow(length(p_offset), 2.0) / shape_divisor - pow(repeat * 0.8, 0.5);
    let noise_scale = 12.0; // Tune?
    let worley_scale = 15.0; // Tune?
    let distortion = noise(vec3<f32>(p * noise_scale - smoke_center_offset, time * 0.1));

    let clamped_repeat = clamp(repeat, 0.1, 2.0);
    let p_scaled_for_worley: vec2<f32> = (p / pow(clamped_repeat, 0.35)) * worley_scale;
    let offset_scaled_for_worley: vec2<f32> = smoke_center_offset * (1.65 / 1.45);
    let worley_vec2_component: vec2<f32> = p_scaled_for_worley - offset_scaled_for_worley;
    let worley_time_component: f32 = time * 0.1;
    let worley_input_vec3: vec3<f32> = vec3<f32>(worley_vec2_component, worley_time_component);

    let bubbles = 0.5 - pow(worley(worley_input_vec3), 2.0); // Tune?

    let bw = 0.75;
    let effects = (bw * bubbles + (1.0 - bw) * distortion) * 0.4; // Tune?
    return shape + effects;
}

// Combined function f(p) - no aspect
fn f(p: vec2<f32>) -> f32 {
    let b = boom(p);
    let s = smoke(p);
    return select(s, b, b > s);
}

// Gradient calculation - no aspect
fn grad(x: vec2<f32>) -> vec2<f32> {
    let h = vec2<f32>(0.01, 0.0);
    let fx_plus = f(x + h.xy);
    let fx_minus = f(x - h.xy);
    let fy_plus = f(x + h.yx);
    let fy_minus = f(x - h.yx);
    return vec2<f32>(fx_plus - fx_minus, fy_plus - fy_minus) / (2.0 * h.x);
}

// Border calculation - no aspect
fn border(uv: vec2<f32>) -> f32 {
    let b = f(uv);
    let g = grad(uv);
    let g_len = length(g);
    let de = select(0.0, abs(b) / g_len, g_len > 0.001);
    let eps = 0.02 / 4.0; // Tune border thickness?
    return smoothstep(1.0 * eps, 2.0 * eps, de);
}

// --- Fragment Shader Entry Point ---
@fragment
fn fragment(mesh: VertexOutput) -> @location(0) vec4<f32> {
    // 1. Center UVs ONLY. The 'pos' variable is now in [-0.5, 0.5] space.
    var pos = mesh.uv - vec2<f32>(0.5, 0.5);

    // 2. Calculate boom and smoke values (functions no longer need aspect)
    let boom_val = boom(pos);
    let boom_a = step(0.0, boom_val);
    let boom_idx_f = posterize(boom_val, BOOM_PAL_LEN) * f32(BOOM_PAL_LEN - 1u);
    let boom_idx = u32(round(boom_idx_f));
    let boom_col = boom_pal[boom_idx];

    let smoke_val = smoke(pos);
    let smoke_a = step(0.0, smoke_val);
    let smoke_idx_f = posterize(smoke_val, SMOKE_PAL_LEN) * f32(SMOKE_PAL_LEN - 1u);
    let smoke_idx = u32(round(smoke_idx_f));
    let smoke_col = smoke_pal[smoke_idx];

    // 3. Calculate border factor (function no longer needs aspect)
    let border_fac = border(pos);

    // 4. Determine dominant effect
    let is_boom = step(smoke_val * 1.25, boom_val);

    // 5. Mix colors and alpha
    let color = mix(smoke_col, boom_col, is_boom);
    let alpha = mix(smoke_a, boom_a, is_boom);

    // 6. Apply border
    let final_color = mix(vec3<f32>(0.0), color, border_fac);

    // 7. Final alpha
    let final_alpha = alpha * step(0.001, alpha);

    // 8. Discard transparent fragments
    if (final_alpha < 0.01) {
        discard;
    }

    // 9. Return final color
    return vec4<f32>(final_color, final_alpha);
}
