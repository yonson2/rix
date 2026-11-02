// assets/shaders/flame_material.wesl
import bevy_sprite_render::mesh2d::vertex_output::VertexOutput;

// Uniforms defined individually to match Rust struct
@group(2) @binding(0) var<uniform> time: f32;
@group(2) @binding(1) var<uniform> amount: f32;
// Bindings 2 and 3 are intentionally skipped
@group(2) @binding(4) var<uniform> colour_1: vec4<f32>; // Base Orange
@group(2) @binding(5) var<uniform> colour_2: vec4<f32>; // Highlight Yellow-Orange
@group(2) @binding(6) var<uniform> id: f32;

// Constants
const PIXEL_SIZE_FAC: f32 = 120.0;
const MAX_INTENSITY: f32 = 10.0;

// Helper function for modulo
fn fmod(x: f32, y: f32) -> f32 {
    return x - y * floor(x / y);
}

// Main flame calculation logic
fn calculate_flame(input_uv: vec2<f32>) -> vec4<f32> {
    let intensity = 1.0 * min(MAX_INTENSITY, amount);
    if (intensity < 0.1) {
        return vec4(0.0, 0.0, 0.0, 0.0);
    }

    // --- UV Mapping: Map Mesh [0,1]x[0,1] to Effect [-0.5,0.5]x[-1.0,0.0] ---
    let uv = vec2(input_uv.x - 0.5, input_uv.y - 1.0);
    // --- End UV Mapping ---

    // --- Pixelation Enabled ---
    let floored_uv = floor(uv * PIXEL_SIZE_FAC) / PIXEL_SIZE_FAC;
    // --- End Pixelation Enabled ---

    var uv_scaled_centered = floored_uv; // Use pixelated value
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

    // Vertical fade term
    let vert_fade_intensity = max(0.0, 2.0 - 0.3 * intensity);
    let vert_pos_factor = max(0.0, 2.0 * (uv_scaled_centered.y - 0.5) * (uv_scaled_centered.y - 0.5));
    var smoke_res = select(base_density + vert_fade_intensity * vert_pos_factor, 1.0, intensity < 0.1);

    // Horizontal edge fade (dependent on intensity)
    let fade_multiplier = 1.0 - clamp(intensity / MAX_INTENSITY, 0.0, 1.0);
    let fade_threshold = 0.4;
    if (abs(uv.x) > fade_threshold) {
        smoke_res = smoke_res + fade_multiplier * 10.0 * (abs(uv.x) - fade_threshold);
    }

    // Central adjustment
    let center_dist_vec = (uv - vec2(0.0, 0.1)) * vec2(0.19, 1.0);
    let center_dist = length(center_dist_vec);
    let intensity_threshold = min(0.1, intensity * 0.5);
    if (center_dist < intensity_threshold && smoke_res > 1.0) {
        smoke_res = smoke_res + min(8.5, intensity * 10.0) * (center_dist - 0.1);
    }
    // --- End smoke_res calculation ---


    // --- Determine final color and alpha ---
    var ret_col: vec4<f32>;

    if (smoke_res >= 1.0) {
        ret_col = vec4(0.0); // Fully transparent black
    } else {
        // --- Color Blending with Faster Transition Curve ---
        // Define the vertical range in uv.y space [-1.0, 0.0] for the blend
        let blend_start_y: f32 = -0.3; // Start blending only in the top 70%
        let blend_end_y: f32 = 0.0;    // Finish blending at the very top edge
        let blend_range: f32 = blend_end_y - blend_start_y; // Now 0.3

        var linear_mix_factor: f32 = 0.0; // Default to 0.0
        if (uv.y > blend_start_y && blend_range > 0.001) { // Check if within blend range
            linear_mix_factor = clamp((uv.y - blend_start_y) / blend_range, 0.0, 1.0);
        } else if (uv.y >= blend_end_y) { // Ensure fully second color above the zone
             linear_mix_factor = 1.0;
        }
        // If uv.y <= blend_start_y, linear_mix_factor remains 0.0

        // --- Apply Curve to Mix Factor ---
        // Using sqrt (pow with 0.5) makes the factor increase faster initially
        let curved_mix_factor = pow(linear_mix_factor, 0.5); // Try 0.5 exponent, adjust (e.g., 0.7) if needed
        // --- End Apply Curve ---

        // Interpolate between colour_2 (start) and colour_1 (end) using the curved factor
        ret_col = mix(colour_2, colour_1, curved_mix_factor);

        // --- Keep Color Adjustment Term Disabled ---
        // let adj_blend_factor = clamp(0.12 - uv.y, 0.0, 1.0);
        // if (uv.y < 0.12) {
        //      ret_col += ret_col * (-2.0 + 0.5 * intensity * smoke_res) * adj_blend_factor;
        // }
        // --- End Keep Disabled ---

        // Use Hard Alpha
        ret_col.a = 1.0;
        // --- End Color Blending ---
    }

    // Clamp final color values
    ret_col = clamp(ret_col, vec4(0.0), vec4(10.0));

    // No premultiplied alpha needed for hard alpha output
    return ret_col;
}

// Entry point for the fragment shader
@fragment
fn fragment(mesh: VertexOutput) -> @location(0) vec4<f32> {
    let final_color = calculate_flame(mesh.uv);
    return final_color;
}
