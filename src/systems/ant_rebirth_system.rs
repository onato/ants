use bevy::prelude::*;
use crate::ant::Ant;
use crate::components::position::Position;
use crate::components::reset_lifetime::ResetLifetime;

// System to check ant lifetimes and handle expiration
pub fn ant_rebirth_system(
    mut commands: Commands,
    time: Res<Time>,
    mut ant_query: Query<(Entity, &mut Ant, &mut Position)>,
) {
    for (entity, mut ant, mut position) in ant_query.iter_mut() {
        ant.lifetime.tick(time.delta());
        if ant.lifetime.finished() {
            ant.lifetime.reset();
            position.position = Vec2::new(0.0, 0.0);
            commands.entity(entity).insert(ResetLifetime);
        }
    }
}

