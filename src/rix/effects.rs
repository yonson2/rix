use bevy::{
    prelude::*,
    reflect::TypePath,
    render::render_resource::{AsBindGroup, ShaderRef},
    sprite::{AlphaMode2d, Material2d, Material2dPlugin},
};

use super::state::IsPaused;

const EXPLOSION_SHADER_ASSET_PATH: &str = "shaders/cartoon_explosion.wgsl";

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct ExplosionMaterial {
    #[uniform(0)]
    pub time: f32,
    #[uniform(1)]
    pub start_time: f32,
}

impl Material2d for ExplosionMaterial {
    fn fragment_shader() -> ShaderRef {
        EXPLOSION_SHADER_ASSET_PATH.into()
    }
    fn alpha_mode(&self) -> AlphaMode2d {
        AlphaMode2d::Blend
    }
}

pub struct EffectsPlugin;
impl Plugin for EffectsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(Material2dPlugin::<ExplosionMaterial>::default());
        app.add_systems(
            Update,
            update_explosion_time.run_if(in_state(IsPaused::Running)),
        );
    }
}

fn update_explosion_time(
    time: Res<Time>,
    mut explosion_materials: ResMut<Assets<ExplosionMaterial>>,
) {
    let current_time = time.elapsed_secs();
    for (_, material) in explosion_materials.iter_mut() {
        material.time = current_time;
    }
}
