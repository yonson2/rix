#import bevy_sprite::mesh2d_vertex_output::VertexOutput

// Uniforms defined individually to match Rust struct
@group(2) @binding(0) var<uniform> time: f32;
@group(2) @binding(1) var<uniform> amount: f32;
// Bindings 2 and 3 are intentionally skipped
@group(2) @binding(4) var<uniform> colour_1: vec4<f32>;
@group(2) @binding(5) var<uniform> colour_2: vec4<f32>;
@group(2) @binding(6) var<uniform> id: f32;

// Constants
const PIXEL_SIZE_FAC: f32 = 60.0;

// Helper function for modulo
fn fmod(x: f32, y: f32) -> f32 {
    return x - y * floor(x / y);
}

// Main flame calculation logic
fn calculate_flame(input_uv: vec2<f32>) -> vec4<f32> {
    let intensity = 1.0 * min(10.0, amount);
    if (intensity < 0.1) {
        return vec4(0.0, 0.0, 0.0, 0.0); // Exit if no intensity
    }

    // --- UV Mapping: Center X, Shift Y Up ---
    // Map input UV [0,1]x[0,1] to effect UV [-0.5,0.5]x[-1.0,0.0]
    // This places the effect's origin (0,0) at the center-top edge of the mesh.
    let uv = vec2(input_uv.x - 0.5, input_uv.y - 1.0);
    // --- End UV Mapping ---

    // Pixelation and initial distortion
    let floored_uv = floor(uv * PIXEL_SIZE_FAC) / PIXEL_SIZE_FAC;
    var uv_scaled_centered = floored_uv;
    uv_scaled_centered += uv_scaled_centered * 0.01 * (
        sin(-1.123 * floored_uv.x + 0.2 * time) *
        cos(5.3332 * floored_uv.y + time * 0.931)
    );

    // Flame animation vector and noise parameters
    let flame_up_vec = vec2(0.0,
        fmod(4.0 * time, 10000.0) - 5000.0 +
        fmod(1.781 * id, 1000.0)
    );
    let scale_fac = 7.5 + 3.0 / (2.0 + 2.0 * intensity);
    let speed = fmod(20.781 * id, 100.0) +
                1.0 * sin(time + id) * cos(time * 0.151 + id);

    // Noise generation loop
    var sv = uv_scaled_centered * scale_fac + flame_up_vec;
    var sv2 = vec2(0.0, 0.0);
    for (var i: i32 = 0; i < 5; i = i + 1) {
        let swirl_dir = select(1.0, -1.0, (i % 2) == 1);
        sv2 += sv + 0.05 * sv2.yx * swirl_dir + 0.3 * (
            cos(length(sv) * 0.411) + 0.3344 * sin(length(sv)) - 0.23 * cos(length(sv))
        );
        sv += 0.5 * vec2(
            cos(cos(sv2.y) + speed * 0.0812) * sin(3.22 + sv2.x - speed * 0.1531),
            sin(-sv2.x * 1.21222 + 0.113785 * speed) * cos(sv2.y * 0.91213 - 0.13582 * speed)
        );
    }

    // --- Calculate smoke density (smoke_res) ---
    let dist_from_origin = length((sv - flame_up_vec) / scale_fac * 5.0);
    let uv_center_dist_term = 0.1 * (length(uv_scaled_centered) - 0.5);
    let factor = 2.0 / (2.0 + intensity * 0.2);
    let base_density = max(0.0, (dist_from_origin + uv_center_dist_term)) * factor;

    var smoke_res = base_density; // Start with base density

    // --- Disable Vertical Fade Term ---
    // This term caused issues with previous negative UV shifts. Let's keep it disabled
    // while testing the new positive shift.
    // let vert_fade_intensity = max(0.0, 2.0 - 0.3 * intensity);
    // let vert_pos_factor = max(0.0, 2.0 * (uv_scaled_centered.y - 0.5) * (uv_scaled_centered.y - 0.5));
    // smoke_res = select(smoke_res + vert_fade_intensity * vert_pos_factor, 1.0, intensity >= 0.1);
    // --- Keep it disabled for now ---
    smoke_res = select(smoke_res, 1.0, intensity < 0.1); // Ensure smoke_res is 1.0 if intensity is too low


    // Horizontal edge fade
    if (abs(uv.x) > 0.4) {
        smoke_res = smoke_res + 10.0 * (abs(uv.x) - 0.4);
    }

    // Central adjustment (might make core slightly clearer)
    let center_dist_vec = (uv - vec2(0.0, 0.1)) * vec2(0.19, 1.0); // Note: uses uv, not uv_scaled_centered
    let center_dist = length(center_dist_vec);
    let intensity_threshold = min(0.1, intensity * 0.5);
    if (center_dist < intensity_threshold && smoke_res > 1.0) {
        smoke_res = smoke_res + min(8.5, intensity * 10.0) * (center_dist - 0.1);
    }
    // --- End smoke_res calculation ---


    // --- Determine final color and alpha ---
    var ret_col = colour_1; // Start with base flame color

    if (smoke_res >= 1.0) { // Use >= for safer floating point comparison
        // If smoke density value is high, make it transparent
        ret_col.a = 0.0;
    } else {
        // If it's part of the flame body
        // Blend towards colour_2 in the lower part of the effect space (negative uv.y)
        if (uv.y < 0.12) { // This condition might need adjustment based on new UV mapping
            // Blend factor increases as uv.y gets more negative
            let blend_factor = clamp(0.12 - uv.y, 0.0, 1.0); // Clamp factor just in case
            ret_col = ret_col * (1.0 - 0.5 * blend_factor) + 2.5 * blend_factor * colour_2;
            // Adjust brightness/intensity based on smoke_res and position
            ret_col += ret_col * (-2.0 + 0.5 * intensity * smoke_res) * blend_factor;
        }
        // Calculate alpha based on density (lower smoke_res = higher alpha)
        ret_col.a = clamp(1.0 - smoke_res, 0.0, 1.0);
        // Redundant check removed: if (smoke_res > 1.0) { ret_col.a = 0.0; }
    }

    // Clamp final color values
    ret_col = clamp(ret_col, vec4(0.0), vec4(10.0)); // Allow some HDR intensity

    // Optional: Apply pre-multiplied alpha if needed (usually needed for Bevy's default blend state)
    // ret_col.rgb *= ret_col.a;

    return ret_col;
}

// Entry point for the fragment shader
@fragment
fn fragment(mesh: VertexOutput) -> @location(0) vec4<f32> {
    // Calculate the flame color for the current fragment
    let final_color = calculate_flame(mesh.uv);
    return final_color;
}
