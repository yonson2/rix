use bevy::{
    prelude::*,
    reflect::TypePath,
    render::render_resource::AsBindGroup,
    shader::ShaderRef,
    sprite_render::{AlphaMode2d, Material2d, Material2dPlugin},
};

use super::{AppSystems, PausableSystems};

const EXPLOSION_SHADER_PATH: &str = "shaders/cartoon_explosion.wesl";
#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct ExplosionMaterial {
    #[uniform(0)]
    pub time: f32,
    #[uniform(1)]
    pub start_time: f32,
}

impl Material2d for ExplosionMaterial {
    fn fragment_shader() -> ShaderRef {
        EXPLOSION_SHADER_PATH.into()
    }
    fn alpha_mode(&self) -> AlphaMode2d {
        AlphaMode2d::Blend
    }
}

const PIXEL_EXPLOSION_SHADER_PATH: &str = "shaders/pixel_explosion.wesl";
#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct PixelExplosionMaterial {
    #[uniform(0)]
    pub time: f32,
    #[uniform(1)]
    pub start_time: f32,
    #[uniform(2)]
    pub seed: f32,
}

impl Material2d for PixelExplosionMaterial {
    fn fragment_shader() -> ShaderRef {
        PIXEL_EXPLOSION_SHADER_PATH.into()
    }
    fn alpha_mode(&self) -> AlphaMode2d {
        AlphaMode2d::Blend
    }
}

pub(super) fn plugin(app: &mut App) {
    app.add_plugins(Material2dPlugin::<ExplosionMaterial>::default());
    app.add_plugins(Material2dPlugin::<PixelExplosionMaterial>::default());
    app.add_plugins(Material2dPlugin::<GlowMaterial>::default());
    app.add_systems(
        Update,
        update_explosion_time
            .in_set(AppSystems::Update)
            .in_set(PausableSystems),
    );
}

fn update_explosion_time(
    time: Res<Time>,
    mut explosion_materials: ResMut<Assets<ExplosionMaterial>>,
    mut pixel_explosion_materials: ResMut<Assets<PixelExplosionMaterial>>,
) {
    let current_time = time.elapsed_secs();
    for (_, material) in explosion_materials.iter_mut() {
        material.time = current_time;
    }
    for (_, material) in pixel_explosion_materials.iter_mut() {
        material.time = current_time;
    }
}

const GLOW_SHADER_ASSET_PATH: &str = "shaders/glow_material.wesl";
#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct GlowMaterial {
    // Intensity controls the brightness of the overlay.
    // (because we are using bloom & hdr)
    #[uniform(0)]
    pub intensity: f32,
    /// A pulsing outline around the opaque pixels ([040]); alpha 0 for none.
    #[uniform(0)]
    pub outline: LinearRgba,

    #[texture(1)]
    #[sampler(2)]
    pub texture: Handle<Image>,
}

impl Material2d for GlowMaterial {
    fn fragment_shader() -> ShaderRef {
        GLOW_SHADER_ASSET_PATH.into()
    }

    fn alpha_mode(&self) -> AlphaMode2d {
        AlphaMode2d::Blend
    }
}
