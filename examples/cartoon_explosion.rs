use bevy::{
    prelude::*,
    reflect::TypePath,
    render::render_resource::{AsBindGroup, ShaderRef},
    sprite::{AlphaMode2d, Material2d, Material2dPlugin, MeshMaterial2d},
    // PrimaryWindow is no longer needed for this material
};

const EXPLOSION_SHADER_ASSET_PATH: &str = "shaders/cartoon_explosion.wgsl";

// Material struct - ONLY time uniform
#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct ExplosionMaterial {
    #[uniform(0)]
    pub time: f32,
}

// Implement Material2d using AlphaMode2d
impl Material2d for ExplosionMaterial {
    fn fragment_shader() -> ShaderRef {
        EXPLOSION_SHADER_ASSET_PATH.into()
    }
    fn alpha_mode(&self) -> AlphaMode2d {
        AlphaMode2d::Blend
    }
}

// System to update time uniform ONLY
fn update_explosion_time(
    time: Res<Time>,
    mut explosion_materials: ResMut<Assets<ExplosionMaterial>>,
) {
    let current_time = time.elapsed_secs();
    for (_, material) in explosion_materials.iter_mut() {
        material.time = current_time;
    }
}

// Setup system - no longer needs window query
fn setup_explosion_effect(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut explosion_materials: ResMut<Assets<ExplosionMaterial>>,
) {
    commands.spawn(Camera2d);

    // Define a specific size for the rectangle
    let rect_width = 50.0;
    let rect_height = 50.0;
    let quad_mesh_handle: Handle<Mesh> = meshes.add(Rectangle::new(rect_width, rect_height));

    // Spawn the rectangle - material only needs time
    commands.spawn((
        Name::new("ExplosionEffect"),
        Mesh2d(quad_mesh_handle),
        MeshMaterial2d(explosion_materials.add(ExplosionMaterial {
            time: 0.0, // Only time is needed
        })),
        Transform::from_xyz(0.0, 0.0, 0.0),
    ));
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(Material2dPlugin::<ExplosionMaterial>::default())
        .add_systems(Startup, setup_explosion_effect)
        .add_systems(Update, update_explosion_time)
        .run();
}
