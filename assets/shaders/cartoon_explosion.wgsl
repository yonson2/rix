#import bevy_sprite::mesh2d_vertex_output::VertexOutput

// --- Uniforms ---
@group(2) @binding(0) var<uniform> time: f32; // Corresponds to iTime

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
    let r = fract(
        sin(
            vec3<f32>(
                dot(p, vec3<f32>(127.1, 311.7, 371.8)),
                dot(p, vec3<f32>(269.5, 183.3, 456.1)),
                dot(p, vec3<f32>(352.5, 207.3, 198.67))
            )
        ) * 43758.5453
    ) * 2.0 - 1.0;
    // Using the potentially unstable version from GLSL to match visuals
    // If this causes black pixels or crashes, switch back to normalize(r)
    let cos_r = cos(r);
    // Avoid division by zero
    let safe_cos_r = select(cos_r, vec3(1.0), abs(cos_r) < vec3(0.001));
    return normalize(r / safe_cos_r);
    // Safer alternative: return normalize(r);
}

// --- Helper: Perlin-like noise ---
fn noise(p: vec3<f32>) -> f32 {
    let fv = fract(p);
    let nv = floor(p);
    let u = fv * fv * fv * (fv * (fv * 6.0 - 15.0) + 10.0);
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
    if (n <= 1u) { return 0.0; }
    let num_levels_f32 = f32(n); // Renamed variable
    let clamped_v = max(0.0, v);
    let denominator = max(1.0, num_levels_f32 - 1.0);
    return clamp(floor(clamped_v * num_levels_f32) / denominator, 0.0, 1.0);
}


// --- Shape: Explosion Core (Exact GLSL Formula) ---
fn boom(p: vec2<f32>) -> f32 {
    let repeat = fmod(time, 2.0); // Time loop 0-2s
    // Exact GLSL shape formula
    let shape = 1.0 - pow(length(p), 2.0) / max(0.001, (repeat * 12.0)) - repeat * 2.0; // Added max to avoid div by zero if repeat is 0

    // Effects using original GLSL scaling factors for noise coords
    let distortion = noise(vec3<f32>(p * 0.5, time * 0.5)); // Use p*0.5
    let bubbles = 0.5 - pow(worley(vec3<f32>(p * 1.2, time * 2.0)), 3.0); // Use p*1.2
    let bw = 0.5;
    let effects = (bw * bubbles + (1.0 - bw) * distortion); // No extra scaling factor here

    return shape + effects;
}

// --- Shape: Smoke (Exact GLSL Formula) ---
fn smoke(p: vec2<f32>) -> f32 {
    let repeat = fmod(time, 2.0); // Time loop 0-2s
    // GLSL offset calculation
    let smoke_center_offset = vec2<f32>(0.0, 2.0) * pow(repeat / 1.45, 2.0) * 1.5;
    let p_offset = p - smoke_center_offset;

    // Exact GLSL shape formula
    let shape = 1.0 - pow(length(p_offset), 2.0) / max(0.001, (repeat * 16.0)) - pow(repeat * 1.5, 0.5); // Added max to avoid div by zero

    // Effects using original GLSL scaling factors for noise coords
    // GLSL: noise(vec3(p*1.5 - smoke_center_offset, time*.1)); -> Apply offset AFTER scaling p
    let p_scaled_noise = p * 1.5;
    let distortion = noise(vec3<f32>(p_scaled_noise - smoke_center_offset, time * 0.1));

    // GLSL: worley(vec3((p/pow(repeat,.35)) - offset_scaled, time*.1))
    // Need offset_scaled_for_worley calculation (used simplified one before, let's recalculate like GLSL)
    let offset_scaled_for_worley = vec2<f32>(0.0, 2.0) * pow(repeat / 1.65, 2.0) * 1.5; // Slightly different divisor from shape offset
    let repeat_pow_035 = pow(max(0.01, repeat), 0.35); // Avoid pow(0, 0.35), clamp repeat slightly away from 0
    let p_scaled_worley = p / repeat_pow_035;
    let bubbles = 0.5 - pow(worley(vec3<f32>(p_scaled_worley - offset_scaled_for_worley, time * 0.1)), 2.0);

    let bw = 0.75;
    let effects = (bw * bubbles + (1.0 - bw) * distortion); // No extra scaling factor here

    return shape + effects;
}

// --- Combined Shape Function (Selects max) ---
fn f(p: vec2<f32>) -> f32 {
    let b = boom(p);
    let s = smoke(p);
    return select(s, b, b > s);
}

// --- Gradient Calculation ---
fn grad(x: vec2<f32>) -> vec2<f32> {
    let h = vec2<f32>(0.01, 0.0);
    let fx_plus = f(x + h.xy); let fx_minus = f(x - h.xy);
    let fy_plus = f(x + h.yx); let fy_minus = f(x - h.yx);
    return vec2<f32>(fx_plus - fx_minus, fy_plus - fy_minus) / (2.0 * h.x);
}

// --- Border Calculation ---
fn border(uv: vec2<f32>) -> f32 {
    let b = f(uv);
    let g = grad(uv);
    let g_len = length(g);
    let de = select(0.0, abs(b) / g_len, g_len > 0.001);
    // Use GLSL border thickness? eps=0.02 was used there.
    // The coordinate space is now scaled by 7.0, so maybe eps needs scaling too?
    let eps = 0.02 * 7.0; // Scale epsilon relative to coordinate scaling? Try this.
    // let eps = 0.02; // Or keep original GLSL value?
    return smoothstep(1.0 * eps, 2.0 * eps, de); // 0 = border, 1 = inside
}


// --- Fragment Shader Entry Point ---
@fragment
fn fragment(mesh: VertexOutput) -> @location(0) vec4<f32> {

    // --- Coordinate setup mirroring GLSL mainImage ---
    let uv = mesh.uv; // Input UV [0, 1]
    var pos = uv - vec2<f32>(0.5, 0.4); // Center around (0.5, 0.4) like GLSL
    // No window aspect correction: pos.x *= aspect;
    pos = pos * 7.0; // Apply scaling like GLSL (pos *= 7.)
    // 'pos' is now the coordinate space for boom/smoke/etc.

    // --- Calculate RAW boom and smoke values ---
    let boom_val = boom(pos);
    let smoke_val = smoke(pos);

    // --- Calculate Alpha based on RAW values ---
    let boom_a = step(0.0, boom_val);
    let smoke_a = step(0.0, smoke_val);

    // --- Early exit if invisible ---
    if (boom_a < 0.01 && smoke_a < 0.01) {
         discard;
    }

    // --- Posterize RAW values ---
    let boom_idx_f = posterize(boom_val, BOOM_PAL_LEN) * f32(BOOM_PAL_LEN - 1u);
    let boom_idx = u32(round(boom_idx_f));
    // Original GLSL color calc: pal[idx] - vec3(1.-alpha) effectively makes color dark if alpha=0
    // We can simplify: just use pal[idx] as color will be discarded if alpha=0
    let boom_col = boom_pal[boom_idx];

    let smoke_idx_f = posterize(smoke_val, SMOKE_PAL_LEN) * f32(SMOKE_PAL_LEN - 1u);
    let smoke_idx = u32(round(smoke_idx_f));
    let smoke_col = smoke_pal[smoke_idx];

    // --- Calculate border factor ---
    let border_fac = border(pos); // border_fac = 0 on border, 1 inside

    // --- Determine dominant effect ---
    let is_boom = step(smoke_val * 1.25, boom_val); // Use original GLSL bias

    // --- Mix final color and alpha ---
    let color = mix(smoke_col, boom_col, is_boom);
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
