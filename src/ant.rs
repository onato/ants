use bevy::prelude::*;
use rand::Rng;
use std::time::Duration;
use std::f32::consts::TAU;
use bevy::math::Vec2;
use crate::components::ant::Ant;
use crate::components::carrying_food::CarryingFood;
use crate::components::direction::Direction;
use crate::components::food::Food;
use crate::components::position::Position;
use crate::systems::ant_rebirth_system::ant_rebirth_system;
use crate::systems::ant_lifetime_reset_system::{MIN_LIFETIME, MAX_LIFETIME, ant_lifetime_reset_system};
use crate::systems::follow_pheromone_system::follow_pheromones_system;

pub struct AntPlugin;

impl Plugin for AntPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Startup, setup)
            .add_systems(Update,(
                ant_goal_system,
                follow_pheromones_system,
                ant_rebirth_system,
                ant_lifetime_reset_system,
                sync_transform_with_position
            ));
    }
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    for _ in 0..5000 {
        let mut rng = rand::thread_rng();
        // Random lifetime between 30 and 60 seconds
        let lifetime_secs = rng.gen_range(MIN_LIFETIME..=MAX_LIFETIME);
        let mut rng = rand::thread_rng();
        let random_angle = rng.gen_range(0.0..TAU);
        
        commands.spawn((
            Mesh2d(meshes.add(Rectangle::new(3., 3.))),
            MeshMaterial2d(materials.add(Color::srgb(0.65, 0.145, 0.145))),
            Transform::from_xyz( 0., 0., 0.,),
            Ant { 
                lifetime: Timer::new(Duration::from_secs_f32(lifetime_secs), TimerMode::Once),
            },
            Position { position: Vec2::new(0.0, 0.0) }, // Initial position
            Direction { direction: Vec2::new(random_angle.cos(), random_angle.sin()).normalize()},
        ));
    }
}
// System for handling ant goals (finding food or returning to nest)
pub fn ant_goal_system(
    mut commands: Commands,
    mut query: Query<(Entity, &Position, Option<&CarryingFood>), With<Ant>>,
    food_positions: Query<&Position, With<Food>>,
) {
    // Collect food positions into a Vec to avoid query conflicts
    let food_positions: Vec<Vec2> = food_positions
        .iter()
        .map(|position| Vec2::new(position.position.x, position.position.y))
        .collect();

    for (entity, position, carrying_food) in query.iter_mut() {
        if carrying_food.is_none() {
            // Check if ant found food
            let found_food = food_positions.iter().any(|&food_pos| 
                food_pos.distance(position.position) < 5.0
            );
            
            if found_food {
                commands.entity(entity).insert(CarryingFood);
            }
        } else {
            // Check if ant reached the nest
            let reached_nest = position.position.length() < 10.0;
            
            if reached_nest {
                // Change goal back to finding food
                commands.entity(entity).remove::<CarryingFood>();
            }
        }
    }
}

fn sync_transform_with_position(
    mut query: Query<(&Position, &mut Transform)>,
) {
    for (position, mut transform) in query.iter_mut() {
        transform.translation = position.position.extend(0.0); // Update position
    }
}

