use bevy::{
    prelude::*,
    reflect::TypePath,
    render::render_resource::AsBindGroup,
    shader::ShaderRef,
    sprite_render::{AlphaMode2d, Material2d, Material2dPlugin},
};

use super::AppSystems;

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
    // Not pausable ([094]): a kill of the claim that clears the level plays during the
    // transition. A pause stops virtual time, so it still holds the explosion still.
    app.add_systems(Update, update_explosion_time.in_set(AppSystems::Update));
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

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use bevy::{state::app::StatesPlugin, time::TimeUpdateStrategy};

    use super::*;
    use crate::{
        game::{
            PausableSystems,
            state::{IsPaused, Phase},
        },
        screens::Screen,
    };

    #[test]
    fn a_kill_explosion_plays_on_during_the_level_change() {
        let mut app = App::new();
        app.add_plugins((
            MinimalPlugins,
            AssetPlugin::default(),
            bevy::mesh::MeshPlugin,
            StatesPlugin,
            plugin,
        ))
        .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f64(
            1. / 60.,
        )))
        .init_state::<Screen>()
        .add_sub_state::<Phase>()
        .add_sub_state::<IsPaused>()
        .insert_state(Screen::Gameplay)
        // As `game::plugin` sets it.
        .configure_sets(Update, PausableSystems.run_if(in_state(IsPaused::Running)));
        let explosion = app
            .world_mut()
            .resource_mut::<Assets<PixelExplosionMaterial>>()
            .add(PixelExplosionMaterial {
                time: 0.,
                start_time: 0.,
                seed: 0.,
            });
        app.update();
        app.world_mut()
            .resource_mut::<NextState<Phase>>()
            .set(Phase::LevelTransition);
        app.update();
        let time = |app: &App| {
            app.world()
                .resource::<Assets<PixelExplosionMaterial>>()
                .get(&explosion)
                .unwrap()
                .time
        };
        let before = time(&app);
        for _ in 0..10 {
            app.update();
        }
        assert!(time(&app) > before, "the pixels do not freeze");
    }
}
