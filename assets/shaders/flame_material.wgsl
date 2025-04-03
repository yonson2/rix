// assets/shaders/flame_material.wgsl
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

    // --- UV Mapping: Map Mesh [0,1]x[0,1] to Effect [-0.5,0.5]x[-1.0,0.0] ---
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

    // --- Re-enable Vertical Fade Term (from GLSL) ---
    let vert_fade_intensity = max(0.0, 2.0 - 0.3 * intensity);
    // Note: uv_scaled_centered.y is in range approx [-1.0, 0.0] now.
    // (uv_scaled_centered.y - 0.5) is approx [-1.5, -0.5]
    // Square is approx [0.25, 2.25]
    let vert_pos_factor = max(0.0, 2.0 * (uv_scaled_centered.y - 0.5) * (uv_scaled_centered.y - 0.5));
    // Add this back, mirroring the GLSL logic more closely
    var smoke_res = select(base_density + vert_fade_intensity * vert_pos_factor, 1.0, intensity < 0.1);
    // --- End Re-enable Vertical Fade Term ---


    // Horizontal edge fade
    if (abs(uv.x) > 0.4) {
        smoke_res = smoke_res + 10.0 * (abs(uv.x) - 0.4);
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
    var ret_col = colour_1;

    if (smoke_res >= 1.0) { // Keep >= for safety
        ret_col.a = 0.0; // Fully transparent
    } else {
        // Blend towards colour_2 based on uv.y position
        if (uv.y < 0.12) { // Often true since uv.y is in [-1.0, 0.0]
            let blend_factor = clamp(0.12 - uv.y, 0.0, 1.0);
            ret_col = ret_col * (1.0 - 0.5 * blend_factor) + 2.5 * blend_factor * colour_2;

            // --- Re-enable Color Adjustment Term (from GLSL) ---
            ret_col += ret_col * (-2.0 + 0.5 * intensity * smoke_res) * blend_factor;
            // --- End Re-enable Color Adjustment Term ---
        }

        // --- Use GLSL Hard Alpha ---
        ret_col.a = 1.0; // Fully opaque if smoke_res < 1.0
        // --- End Hard Alpha ---
    }

    // Clamp final color values (important after color adjustment)
    ret_col = clamp(ret_col, vec4(0.0), vec4(10.0));
    // Optional: Apply pre-multiplied alpha (less critical with hard alpha, but Bevy might still expect it)
    // ret_col.rgb *= ret_col.a;

    return ret_col;
}

// Entry point for the fragment shader
@fragment
fn fragment(mesh: VertexOutput) -> @location(0) vec4<f32> {
    let final_color = calculate_flame(mesh.uv);
    return final_color;
}
