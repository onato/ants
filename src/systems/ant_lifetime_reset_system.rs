use bevy::prelude::*;
use rand::Rng;
use std::time::Duration;
use crate::components::ant::Ant;
use crate::components::reset_lifetime::ResetLifetime;

pub const MIN_LIFETIME: f32 = 24.;
pub const MAX_LIFETIME: f32 = 99.;

pub fn ant_lifetime_reset_system(
    mut commands: Commands,
    mut ant_query: Query<(Entity, &mut Ant), With<ResetLifetime>>,
) {
    for (entity, mut ant) in ant_query.iter_mut() {
        let mut rng = rand::thread_rng();
        let new_lifetime = rng.gen_range(MIN_LIFETIME..=MAX_LIFETIME);
        ant.lifetime = Timer::new(Duration::from_secs_f32(new_lifetime), TimerMode::Once);
            
        commands.entity(entity).remove::<ResetLifetime>();
    }
}

