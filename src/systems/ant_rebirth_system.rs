use bevy::prelude::*;
use rand::Rng;
use crate::components::ant::Ant;
use crate::components::position::Position;
use crate::components::reset_lifetime::ResetLifetime;

// System to check ant lifetimes and handle expiration
pub fn ant_rebirth_system(
    mut commands: Commands,
    time: Res<Time>,
    mut ant_query: Query<(Entity, &mut Ant, &mut Position)>,
) {
    let mut rng = rand::thread_rng();
    for (entity, mut ant, mut position) in ant_query.iter_mut() {
        ant.lifetime.tick(time.delta());
        if ant.lifetime.finished() {
            ant.lifetime.reset();
            
            let x = rng.gen_range(-100f32..=100f32);
            let y = rng.gen_range(-100f32..=100f32);
            position.position = Vec2::new(x, y);
            commands.entity(entity).insert(ResetLifetime);
        }
    }
}

