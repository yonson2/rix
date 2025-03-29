use bevy::prelude::*;

#[derive(Component)]
pub struct ColorTransition {
    pub timer: Timer,
    pub from: Color,
    pub to: Color,
}

//TODO: add these systems in grouped somewhere, not scattered all around

pub fn update_mesh_color(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut ColorTransition, &MeshMaterial2d<ColorMaterial>)>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    for (entity, mut transition, mesh_material) in &mut query {
        transition.timer.tick(time.delta());

        let t = transition.timer.fraction();

        let factor = if t < 0.5 {
            t * 2.0 // Scale 0.0-0.5 to 0.0-1.0
        } else {
            (1.0 - t) * 2.0 // Scale 0.5-1.0 to 1.0-0.0
        };

        let current_color = transition.from.mix(&transition.to, factor);

        if let Some(material) = materials.get_mut(&mesh_material.0) {
            material.color = current_color;
        }

        if transition.timer.finished() && transition.timer.mode() == TimerMode::Once {
            commands.entity(entity).remove::<ColorTransition>();
        }
    }
}

//TODO: Deduplicate
pub fn update_sprite_color(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut ColorTransition, &mut Sprite)>,
) {
    for (entity, mut transition, mut sprite) in &mut query {
        transition.timer.tick(time.delta());

        let t = transition.timer.fraction();
        let factor = if t < 0.5 { t * 2.0 } else { (1.0 - t) * 2.0 };

        let current_color = transition.from.mix(&transition.to, factor);

        sprite.color = current_color;

        if transition.timer.finished() && transition.timer.mode() == TimerMode::Once {
            commands.entity(entity).remove::<ColorTransition>();
        }
    }
}
