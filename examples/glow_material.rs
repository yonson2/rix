// --- File: src/main.rs ---

use bevy::{
    // Core camera bundle for HDR setting
    anti_alias::smaa::Smaa,
    // Need LinearRgba for the color field used by AsBindGroup internally
    camera::Hdr,
    core_pipeline::tonemapping::Tonemapping,
    post_process::bloom::Bloom,
    prelude::*,
    reflect::TypePath,
    render::render_resource::AsBindGroup,
    shader::ShaderRef,
    sprite_render::{AlphaMode2d, Material2d, Material2dPlugin, MeshMaterial2d},
};
use rix::crt::PostProcessSettings;

// Define path for the new shader
const GLOW_SHADER_ASSET_PATH: &str = "shaders/glow_material.wesl";

// --- Material Definition ---

// The Bevy Asset struct, deriving AsBindGroup.
// It directly holds the user-configurable fields and the texture.
#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
struct GlowMaterial {
    // Intensity controls the brightness (RGB value) of the white overlay.
    // Values > 1.0 should cause bloom with HDR enabled.
    #[uniform(0)] // Field goes into uniform buffer at binding 0
    pub intensity: f32,

    // The base texture of the sprite
    #[texture(1)]
    #[sampler(2)]
    pub texture: Handle<Image>,
}

// Implement the Material2d trait for our new material
impl Material2d for GlowMaterial {
    fn fragment_shader() -> ShaderRef {
        GLOW_SHADER_ASSET_PATH.into()
    }

    // Ensure alpha blending is used to correctly layer the glow effect
    fn alpha_mode(&self) -> AlphaMode2d {
        AlphaMode2d::Blend // Or potentially AlphaMode2d::Add if purely additive glow is desired
    }
}

// --- Setup Function ---

fn setup_glow_effect(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<GlowMaterial>>, // Use the new material type
    asset_server: Res<AssetServer>,
) {
    let main_texture: Handle<Image> = asset_server.load("items/bomb_basic.png");

    // --- Spawn Camera with HDR Enabled ---
    // This is crucial for bloom to work with intensity > 1.0
    commands.spawn((
        Camera2d,
        Hdr,
        Tonemapping::TonyMcMapface,
        Bloom::default(),
        Msaa::Off,
        Smaa::default(),
        PostProcessSettings {
            intensity: 1., // Controls overall CRT effect strength
            ambient_light: 0.5,
            time: 0.0, // Will be updated each frame
        },
    ));

    commands.spawn((
        Name::new("SpriteGlowEffect"),
        Mesh2d(meshes.add(Rectangle::default())),
        // Use MeshMaterial2d with the new material type
        MeshMaterial2d(materials.add(GlowMaterial {
            // --- Set the desired glow intensity ---
            intensity: 2.5, // Example: Value > 1.0 for bloom
            // --------------------------------------
            texture: main_texture, // Assign the texture
        })),
        Transform::from_xyz(0.0, 0.0, 0.0).with_scale(Vec3::splat(256.0)),
    ));
}

// --- Main Function ---

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            // Register the Material2d plugin for the new material type
            Material2dPlugin::<GlowMaterial>::default(),
        ))
        .add_systems(Startup, setup_glow_effect)
        // No update system needed unless you want to animate intensity
        .run();
}
