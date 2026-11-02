use bevy::{
    prelude::*,
    reflect::TypePath,
    render::render_resource::AsBindGroup,
    shader::ShaderRef,
    sprite_render::{AlphaMode2d, Material2d, Material2dPlugin},
};
use rix::colors::{GREEN, NEON_GREEN};

const FLAME_SHADER_ASSET_PATH: &str = "shaders/flame_material.wesl";
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

impl Material2d for FlameMaterial {
    fn fragment_shader() -> ShaderRef {
        FLAME_SHADER_ASSET_PATH.into()
    }

    fn alpha_mode(&self) -> AlphaMode2d {
        AlphaMode2d::Blend
    }
}
fn update_flame_time(time: Res<Time>, mut flame_materials: ResMut<Assets<FlameMaterial>>) {
    for (_, material) in flame_materials.iter_mut() {
        material.time = time.elapsed_secs();
    }
}

fn setup_flame_effect(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut flame_materials: ResMut<Assets<FlameMaterial>>,
    mut color_materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn(Camera2d);
    // --- Base Rectangle ---
    let rect_width = 500.0;
    let rect_height = 200.0;
    let base_rect_mesh = meshes.add(Rectangle::new(rect_width, rect_height));

    commands.spawn((
        Name::new("BaseRect"),                      // Optional: Add names for debugging
        Mesh2d(base_rect_mesh.clone()),             // Use the handle
        MeshMaterial2d(color_materials.add(COLOR)), // Use ColorMaterial
        // Position the base rectangle (e.g., centered at origin)
        Transform::from_xyz(0.0, 0.0, 0.0), // Z=0
    ));

    // --- Flame Effect Rectangle ---
    let flame_height = rect_height * 2.;
    let flame_rect_mesh = meshes.add(Rectangle::new(rect_width, flame_height));

    // Calculate position for the flame entity
    // Top edge of base rect is at y = 0.0 + rect_height / 2.0 = 100.0
    // Bottom edge of flame rect should align with top edge of base rect
    // Center of flame rect should be at y = (top of base) + flame_height / 2.0
    let flame_center_y = (rect_height / 2.0) + (flame_height / 2.0); // 100.0 + 75.0 = 175.0

    commands.spawn((
        Name::new("FlameEffect"),
        Mesh2d(flame_rect_mesh),
        MeshMaterial2d(flame_materials.add(FlameMaterial {
            time: 0.0,
            amount: 10.0,
            colour_1: LinearRgba::from(GREEN),
            colour_2: LinearRgba::from(NEON_GREEN),
            id: 99.0,
        })),
        Transform::from_xyz(0.0, flame_center_y, 0.1),
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
