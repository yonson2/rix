 #import bevy_sprite::mesh2d_vertex_output::VertexOutput

// --- Uniforms ---
@group(2) @binding(0) var<uniform> time: f32;       // Global time from Bevy
@group(2) @binding(1) var<uniform> start_time: f32; // Instance spawn time
@group(2) @binding(2) var<uniform> seed: f32;       // Initial seed from Rust

// --- Constants ---
const PARTICLES: f32 = 50.0;
const RES: f32 = 80.0;
const GRAVITY: f32 = 0.32;
const EXPLOSION_DURATION: f32 = 0.8; // Duration remains 2.0 seconds

@fragment
fn fragment(mesh: VertexOutput) -> @location(0) vec4<f32> {

    let elapsed = time - start_time;

    // --- Play Once Logic based on elapsed time ---
    if (elapsed < 0.0 || elapsed > EXPLOSION_DURATION) {
        discard;
    }

    let uv = (mesh.uv - 0.5) * 2.0;
    var clr: f32 = 0.0;

    let time_for_physics = clamp(elapsed, 0.0, EXPLOSION_DURATION);
    var current_seed: f32 = seed;
    let time_for_flashing = elapsed;

    // *** MODIFIED: Calculate linear fade factor ***
    // Linearly fades from 1.0 at elapsed=0.0 down to 0.0 at elapsed=EXPLOSION_DURATION
    let fade_factor = clamp(1.0 - (elapsed / EXPLOSION_DURATION), 0.0, 1.0);

    // Discard check remains - will trigger if fade_factor is ~0 or if elapsed > duration
    if (fade_factor < 0.01 && elapsed > EXPLOSION_DURATION * 0.99) { // Adjusted slightly to ensure fade completes
         discard;
    }
    // Note: The primary discard is still the elapsed > EXPLOSION_DURATION check at the top.
    // This secondary check handles the case where alpha becomes near-zero just before the hard cutoff.


    let inv_res: f32 = 1.0 / RES;
    let inv_particles: f32 = 1.0 / PARTICLES;

    // --- Particle Loop ---
    for (var i: f32 = 0.0; i < PARTICLES; i = i + 1.0) {
        let seed_for_angle = current_seed;
        current_seed += i + tan(current_seed);

        let angle = seed_for_angle;
        let magnitude = i * inv_particles;
        let tPos = vec2<f32>(cos(angle), sin(angle)) * magnitude;

        var pPos: vec2<f32> = vec2(0.0, 0.0);
        pPos.x = tPos.x * time_for_physics;
        pPos.y = -GRAVITY * (time_for_physics * time_for_physics) + tPos.y * time_for_physics;

        pPos = floor(pPos * RES) * inv_res;

        let p1 = pPos;
        let p2 = pPos + inv_res;
        let bounds_check = step(p1, uv) * (vec2<f32>(1.0, 1.0) - step(p2, uv));
        var px1: f32 = bounds_check.x * bounds_check.y;
        let center_of_pixel = pPos + inv_res * 0.5;
        let dist = distance(uv, center_of_pixel);
        let safe_inv_dist = 1.0 / max(dist, 0.0001);
        let px2 = smoothstep(0.0, 200.0, safe_inv_dist);
        px1 = max(px1, px2);

        let flash_factor = sin(time_for_flashing * 20.0 + i) + 1.0;
        clr += px1 * flash_factor;
    }

    // --- Final Color Calculation (using linear fade_factor) ---
    let final_intensity = clr * fade_factor;
    let base_color_multiplier = vec4<f32>(0.9176, 0.5882, 0.0, 1.0); // Orange color
    let final_color_rgb = base_color_multiplier.rgb * final_intensity;
    let final_alpha = clamp(final_intensity, 0.0, 1.0);

    // Final discard check (may be redundant with top check)
    if (final_alpha < 0.01) {
        discard;
    }

    return vec4<f32>(final_color_rgb, final_alpha);
}
