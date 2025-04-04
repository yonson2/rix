#import bevy_sprite::mesh2d_vertex_output::VertexOutput

// --- Uniforms ---
@group(2) @binding(0) var<uniform> time: f32;     // Current global time
@group(2) @binding(1) var<uniform> start_time: f32; // When this explosion was created

// --- Palettes ---
const boom_pal: array<vec3<f32>, 4> = array<vec3<f32>, 4>(vec3<f32>(0.2, 0.15, 0.3), vec3<f32>(0.9, 0.15, 0.05), vec3<f32>(0.9, 0.5, 0.1), vec3<f32>(0.95, 0.95, 0.35));
const BOOM_PAL_LEN: u32 = 4u;
const smoke_pal: array<vec3<f32>, 3> = array<vec3<f32>, 3>(vec3<f32>(0.2, 0.15, 0.3), vec3<f32>(0.35, 0.3, 0.45), vec3<f32>(0.5, 0.45, 0.6));
const SMOKE_PAL_LEN: u32 = 3u;

// --- Helper: fmod ---
fn fmod(x: f32, y: f32) -> f32 {
    let safe_y = select(y, 1.0, y == 0.0);
    return x - safe_y * floor(x / safe_y);
}

// --- Helper: Pseudo-random number generator (Closer GLSL version) ---
fn n_rand3(p: vec3<f32>) -> vec3<f32> {
    let r = fract(sin(vec3<f32>(dot(p, vec3<f32>(127.1, 311.7, 371.8)), dot(p, vec3<f32>(269.5, 183.3, 456.1)), dot(p, vec3<f32>(352.5, 207.3, 198.67)))) * 43758.5453) * 2.0 - 1.0;
    let cos_r = cos(r); let safe_cos_r = select(cos_r, vec3(1.0), abs(cos_r) < vec3(0.001)); return normalize(r / safe_cos_r);
}

// --- Helper: Perlin-like noise ---
fn noise(p: vec3<f32>) -> f32 {
    let fv = fract(p); let nv = floor(p); let u = fv * fv * fv * (fv * (fv * 6.0 - 15.0) + 10.0);
    return mix(mix(mix(dot(n_rand3(nv + vec3<f32>(0.0, 0.0, 0.0)), fv - vec3<f32>(0.0, 0.0, 0.0)), dot(n_rand3(nv + vec3<f32>(1.0, 0.0, 0.0)), fv - vec3<f32>(1.0, 0.0, 0.0)), u.x), mix(dot(n_rand3(nv + vec3<f32>(0.0, 1.0, 0.0)), fv - vec3<f32>(0.0, 1.0, 0.0)), dot(n_rand3(nv + vec3<f32>(1.0, 1.0, 0.0)), fv - vec3<f32>(1.0, 1.0, 0.0)), u.x), u.y), mix(mix(dot(n_rand3(nv + vec3<f32>(0.0, 0.0, 1.0)), fv - vec3<f32>(0.0, 0.0, 1.0)), dot(n_rand3(nv + vec3<f32>(1.0, 0.0, 1.0)), fv - vec3<f32>(1.0, 0.0, 1.0)), u.x), mix(dot(n_rand3(nv + vec3<f32>(0.0, 1.0, 1.0)), fv - vec3<f32>(0.0, 1.0, 1.0)), dot(n_rand3(nv + vec3<f32>(1.0, 1.0, 1.0)), fv - vec3<f32>(1.0, 1.0, 1.0)), u.x), u.y), u.z);
}

// --- Helper: Worley (cellular) noise ---
fn worley(s: vec3<f32>) -> f32 {
    let si = floor(s); let sf = fract(s); var m_dist = 1.0;
    for (var y: i32 = -1; y <= 1; y = y + 1) { for (var x: i32 = -1; x <= 1; x = x + 1) { for (var z: i32 = -1; z <= 1; z = z + 1) { let neighbor_offset = vec3<f32>(f32(x), f32(y), f32(z)); let neighbor_cell_index = si + neighbor_offset; var feature_point_in_cell = fract(n_rand3(neighbor_cell_index)); feature_point_in_cell = 0.5 + 0.5 * sin(time + 6.2831 * feature_point_in_cell); let diff_vec = neighbor_offset + feature_point_in_cell - sf; let dist = length(diff_vec); m_dist = min(m_dist, dist); } } }
    return m_dist;
}

// --- Helper: Posterize ---
fn posterize(v: f32, n: u32) -> f32 {
    if (n <= 1u) { return 0.0; } let num_levels_f32 = f32(n); let clamped_v = max(0.0, v); let denominator = max(1.0, num_levels_f32 - 1.0); return clamp(floor(clamped_v * num_levels_f32) / denominator, 0.0, 1.0);
}

// --- Shape: Explosion Core (Using elapsed time since creation) ---
fn boom(p: vec2<f32>) -> f32 {
    // Calculate elapsed time since this explosion was created (clamped at 2.0 seconds)
    let elapsed = min(time - start_time, 2.0);
    
    // Use elapsed time for animation progress
    let shape = 1.0 - pow(length(p), 2.0) / max(0.001, (elapsed * 12.0)) - elapsed * 2.0;
    let distortion = noise(vec3<f32>(p * 0.5, elapsed * 0.5));
    let bubbles = 0.5 - pow(worley(vec3<f32>(p * 1.2, elapsed * 2.0)), 3.0);
    let bw = 0.5; let effects = (bw * bubbles + (1.0 - bw) * distortion);
    return shape + effects;
}

// --- Shape: Smoke (Using elapsed time since creation) ---
fn smoke(p: vec2<f32>) -> f32 {
    // Calculate elapsed time since this explosion was created (clamped at 2.0 seconds)
    let elapsed = min(time - start_time, 2.0);
    
    let smoke_center_offset = vec2<f32>(0.0, 2.0) * pow(elapsed / 1.45, 2.0) * 1.5;
    let p_offset = p - smoke_center_offset;
    let shape = 1.0 - pow(length(p_offset), 2.0) / max(0.001, (elapsed * 16.0)) - pow(elapsed * 1.5, 0.5);
    let p_scaled_noise = p * 1.5;
    let distortion = noise(vec3<f32>(p_scaled_noise - smoke_center_offset, elapsed * 0.1));
    let offset_scaled_for_worley = vec2<f32>(0.0, 2.0) * pow(elapsed / 1.65, 2.0) * 1.5;
    let repeat_pow_035 = pow(max(0.01, elapsed), 0.35);
    let p_scaled_worley = p / repeat_pow_035;
    let bubbles = 0.5 - pow(worley(vec3<f32>(p_scaled_worley - offset_scaled_for_worley, elapsed * 0.1)), 2.0);
    let bw = 0.75; let effects = (bw * bubbles + (1.0 - bw) * distortion);
    return shape + effects;
}

// --- Combined Shape Function (Selects max) ---
fn f(p: vec2<f32>) -> f32 { let b = boom(p); let s = smoke(p); return select(s, b, b > s); }
// --- Gradient Calculation ---
fn grad(x: vec2<f32>) -> vec2<f32> { let h = vec2<f32>(0.01, 0.0); let fx_plus = f(x + h.xy); let fx_minus = f(x - h.xy); let fy_plus = f(x + h.yx); let fy_minus = f(x - h.yx); return vec2<f32>(fx_plus - fx_minus, fy_plus - fy_minus) / (2.0 * h.x); }
// --- Border Calculation ---
fn border(uv: vec2<f32>) -> f32 { let b = f(uv); let g = grad(uv); let g_len = length(g); let de = select(0.0, abs(b) / g_len, g_len > 0.001); let eps = 0.02 * 7.0; return smoothstep(1.0 * eps, 2.0 * eps, de); }


// --- Fragment Shader Entry Point ---
@fragment
fn fragment(mesh: VertexOutput) -> @location(0) vec4<f32> {

    let loop_duration : f32 = 2.0; // Hardcoded duration matching GLSL

    // --- Calculate elapsed time and fade factor ---
    let elapsed = time - start_time;
    
    // Define fade period towards the end of the 2s cycle
    let fade_start_time: f32 = 1.5; // Start fading at 1.5s after creation
    let fade_end_time: f32 = 2.0;   // Fully faded at 2.0s after creation
    
    // Calculate fade factor (1.0 -> 0.0) based on elapsed time
    let final_fade_factor = 1.0 - smoothstep(fade_start_time, fade_end_time, elapsed);
    // --- End Fade Factor ---

    // --- Early Exit if completely faded ---
    if (final_fade_factor < 0.01) {
        discard;
    }

    // --- Coordinate setup mirroring GLSL mainImage ---
    let uv = mesh.uv;
    var pos = uv - vec2<f32>(0.5, 0.4);
    pos = pos * 7.0;

    // --- Calculate RAW boom and smoke values ---
    // These use fmod(time, 2.0) internally, handling looping visuals
    let boom_val = boom(pos);
    let smoke_val = smoke(pos);

    // --- Apply Fade BEFORE Posterization/Alpha Calculation ---
    let faded_boom_val = boom_val * final_fade_factor;
    let faded_smoke_val = smoke_val * final_fade_factor;

    // --- Calculate Alpha based on FADED values ---
    let boom_a = step(0.0, faded_boom_val);
    let smoke_a = step(0.0, faded_smoke_val);

    // --- Early exit if invisible AFTER fade ---
    if (boom_a < 0.01 && smoke_a < 0.01) {
         discard;
    }

    // --- Posterize FADED values ---
    let boom_idx_f = posterize(faded_boom_val, BOOM_PAL_LEN) * f32(BOOM_PAL_LEN - 1u);
    let boom_idx = u32(round(boom_idx_f));
    let boom_col = boom_pal[boom_idx];

    let smoke_idx_f = posterize(faded_smoke_val, SMOKE_PAL_LEN) * f32(SMOKE_PAL_LEN - 1u);
    let smoke_idx = u32(round(smoke_idx_f));
    let smoke_col = smoke_pal[smoke_idx];

    // --- Calculate border factor (based on raw shapes) ---
    let border_fac = border(pos);

    // --- Determine dominant effect (using raw values) ---
    let is_boom = step(smoke_val * 1.25, boom_val); // Original GLSL bias

    // --- Mix final color and alpha ---
    let color = mix(smoke_col, boom_col, is_boom);
    // Alpha is calculated from faded values, incorporating the fade factor
    let final_alpha = mix(smoke_a, boom_a, is_boom);

    // --- Apply border (mix towards black) ---
    let final_color = mix(vec3<f32>(0.0), color, border_fac);

    // --- Final Checks ---
    let effective_alpha = final_alpha * step(0.001, final_alpha);
    if (effective_alpha < 0.01) {
        discard;
    }

    // --- Return final color ---
    return vec4<f32>(final_color, effective_alpha);
}
