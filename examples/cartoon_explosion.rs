use bevy::{
    prelude::*,
    reflect::TypePath,
    render::render_resource::AsBindGroup,
    shader::ShaderRef,
    sprite_render::{AlphaMode2d, Material2d, Material2dPlugin, MeshMaterial2d},
};

const EXPLOSION_SHADER_ASSET_PATH: &str = "shaders/cartoon_explosion.wgsl";

// Material struct - Added 'infinite' parameter back
#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct ExplosionMaterial {
    #[uniform(0)]
    pub time: f32,
    #[uniform(1)] // Added binding for infinite
    pub infinite: u32, // 1u = loop (true), 0u = play once (false)
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

// Setup system - Initialize 'infinite' parameter
fn setup_explosion_effect(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut explosion_materials: ResMut<Assets<ExplosionMaterial>>,
) {
    commands.spawn(Camera2d);

    let rect_width = 200.0;
    let rect_height = 200.0;
    let quad_mesh_handle: Handle<Mesh> = meshes.add(Rectangle::new(rect_width, rect_height));

    commands.spawn((
        Name::new("ExplosionEffect"),
        Mesh2d(quad_mesh_handle),
        MeshMaterial2d(explosion_materials.add(ExplosionMaterial {
            time: 0.0,
            infinite: 0, // Default to looping. Set to 0u to play only once.
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
