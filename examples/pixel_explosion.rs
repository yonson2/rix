use bevy::{
    math::primitives::Rectangle,
    prelude::*,
    reflect::TypePath,
    render::render_resource::AsBindGroup,
    shader::ShaderRef,
    sprite_render::{AlphaMode2d, Material2d, Material2dPlugin, MeshMaterial2d},
};

const EXPLOSION_SHADER_ASSET_PATH: &str = "shaders/pixel_explosion.wgsl";

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct PixelExplosionMaterial {
    #[uniform(0)]
    pub time: f32,
}

impl Material2d for PixelExplosionMaterial {
    fn fragment_shader() -> ShaderRef {
        EXPLOSION_SHADER_ASSET_PATH.into()
    }
    fn alpha_mode(&self) -> AlphaMode2d {
        AlphaMode2d::Blend
    }
}

// System to update time uniform - No changes needed
fn update_explosion_time(
    time: Res<Time>,
    mut explosion_materials: ResMut<Assets<PixelExplosionMaterial>>,
) {
    let current_time = time.elapsed_secs();
    for (_, material) in explosion_materials.iter_mut() {
        material.time = current_time;
    }
}

// Setup system - Removed setting 'infinite'
fn setup_explosion_effect(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut explosion_materials: ResMut<Assets<PixelExplosionMaterial>>,
) {
    commands.spawn(Camera2d);

    let rect_width = 200.0;
    let rect_height = 200.0;
    let quad_mesh_handle: Handle<Mesh> =
        meshes.add(Mesh::from(Rectangle::new(rect_width, rect_height)));

    commands.spawn((
        Name::new("PixelExplosionEffect"),
        Mesh2d(quad_mesh_handle),
        // Add material - only sets time now
        MeshMaterial2d(explosion_materials.add(PixelExplosionMaterial {
            time: 0.0,
            // infinite: 0, // Removed field setting
        })),
        Transform::from_xyz(0.0, 0.0, 0.0),
    ));
}

// Main function - No changes needed
fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(Material2dPlugin::<PixelExplosionMaterial>::default())
        .add_systems(Startup, setup_explosion_effect)
        .add_systems(Update, update_explosion_time)
        .run();
}
