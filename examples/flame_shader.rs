use bevy::{
    prelude::*,
    reflect::TypePath,
    render::render_resource::{AsBindGroup, ShaderRef},
    sprite::{AlphaMode2d, Material2d, Material2dPlugin},
};
use rix::colors::GREEN;

const FLAME_SHADER_ASSET_PATH: &str = "shaders/flame_material.wgsl";
const COLOR: Color = GREEN;

// This struct holds the data passed to the shader uniform buffer
#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct FlameMaterial {
    // Uniforms match the WGSL struct fields
    #[uniform(0)]
    pub time: f32,
    #[uniform(1)]
    pub amount: f32, // Controls flame intensity (0.0 to 10.0+)
    // Removed texture_details and image_details as we'll simplify UVs
    // If needed later, add them back as Vec4 and Vec2 respectively
    // #[uniform(2)] pub texture_details: Vec4,
    // #[uniform(3)] pub image_details: Vec2,
    #[uniform(4)]
    pub colour_1: LinearRgba, // Base flame color (e.g., orange/red)
    #[uniform(5)]
    pub colour_2: LinearRgba, // Highlight flame color (e.g., yellow/white)
    #[uniform(6)]
    pub id: f32, // For variation if using multiple flames
}

// .add_systems(Update, update_flame_time)
//     .add_plugins(Material2dPlugin::<FlameMaterial>::default())
/// Implement the Material2d trait for our flame effect
impl Material2d for FlameMaterial {
    fn fragment_shader() -> ShaderRef {
        FLAME_SHADER_ASSET_PATH.into()
    }

    // Crucial for transparency: Use Alpha Blending
    fn alpha_mode(&self) -> AlphaMode2d {
        AlphaMode2d::Blend
    }
}
fn update_flame_time(time: Res<Time>, mut flame_materials: ResMut<Assets<FlameMaterial>>) {
    for (_, material) in flame_materials.iter_mut() {
        material.time = time.elapsed_secs();
    }
}

// Example of how to set up and use this material
fn setup_flame_effect(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<FlameMaterial>>,
    mut materials2: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn(Camera2d);

    commands.spawn((
        Mesh2d(meshes.add(Rectangle::new(500., 200.))),
        MeshMaterial2d(materials2.add(COLOR)),
        MeshMaterial2d(materials.add(FlameMaterial {
            time: 0.0,                                              // Will be updated in a system
            amount: 10., // Increase intensity of the flame
            colour_1: LinearRgba::from(Color::srgb(1.0, 0.6, 0.1)), // Orange flame
            colour_2: LinearRgba::from(Color::srgb(1.0, 0.2, 0.0)), // Red flame core
            id: 99.0,
        })),
        Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)).with_scale(Vec3::new(1.0, 1.5, 1.0)), // Scale vertically for better flame shape
    ));
}

// Add to your app
fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Update, update_flame_time)
        .add_plugins(Material2dPlugin::<FlameMaterial>::default())
        .add_systems(Startup, setup_flame_effect)
        .run();
}
