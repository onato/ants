use bevy::prelude::*;
use rand::Rng;
use crate::components::position::Position;
use crate::components::reset_lifetime::ResetLifetime;
use bevy::window::PrimaryWindow;
use crate::systems::ant_rebirth_system::ant_rebirth_system;
use crate::food::Food;
use crate::pheromones::CarryingFood;
use std::time::Duration;

pub struct AntPlugin;

impl Plugin for AntPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Startup, setup)
            .add_systems(Update,(
                ant_goal_system,
                ant_movement_system,
                ant_rebirth_system,
                ant_lifetime_reset_system,
                sync_transform_with_position
            ));
    }
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub enum AntGoal {
    Food,
    Nest,
}

#[derive(Component)]
pub struct Ant {
    pub lifetime: Timer,
}

#[derive(Component)]
pub struct Direction {
    pub direction: Vec2,
}

fn setup(
    mut commands: Commands,
    // mut meshes: ResMut<Assets<Mesh>>,
    // mut materials: ResMut<Assets<ColorMaterial>>,
) {
    for _ in 0..4000 {
        let mut rng = rand::thread_rng();
        // Random lifetime between 30 and 60 seconds
        let lifetime_secs = rng.gen_range(30.0..120.0);
        
        commands.spawn((
            // Mesh2d(meshes.add(Rectangle::new(1., 1.))),
            // MeshMaterial2d(materials.add(Color::srgb(0.3 as f32, 1.0 as f32, 0.0 as f32))),
            // Transform::from_xyz( 0., 0., 0.,),
            Ant { 
                lifetime: Timer::new(Duration::from_secs_f32(lifetime_secs), TimerMode::Once),
            },
            AntGoal::Food, // Initially set to Food goal
            Position { position: Vec2::new(0.0, 0.0) }, // Initial position
            Direction { direction: Vec2::new(1.0, 0.0) },
        ));
    }
}
// System for handling ant goals (finding food or returning to nest)
pub fn ant_goal_system(
    mut commands: Commands,
    mut query: Query<(Entity, &Position, &mut AntGoal), With<Ant>>,
    food_positions: Query<&Transform, With<Food>>,
) {
    // Collect food positions into a Vec to avoid query conflicts
    let food_positions: Vec<Vec2> = food_positions
        .iter()
        .map(|transform| Vec2::new(transform.translation.x, transform.translation.y))
        .collect();
        
    for (entity, position, mut ant_goal) in query.iter_mut() {
        match *ant_goal {
            AntGoal::Food => {
                // Check if ant found food
                let found_food = food_positions.iter().any(|&food_pos| 
                    food_pos.distance(position.position) < 5.0
                );
                
                if found_food {
                    // Change goal to return to nest
                    *ant_goal = AntGoal::Nest;
                    // Mark the ant as carrying food
                    commands.entity(entity).insert(CarryingFood);
                    // Reset lifetime when finding food
                    commands.entity(entity).insert(ResetLifetime);
                }
            },
            AntGoal::Nest => {
                // Check if ant reached the nest
                let reached_nest = position.position.length() < 10.0;
                
                if reached_nest {
                    // Change goal back to finding food
                    *ant_goal = AntGoal::Food;
                    // Remove the carrying food component
                    commands.entity(entity).remove::<CarryingFood>();
                }
            }
        }
    }
}

// System for ant movement based on pheromone trails
pub fn ant_movement_system(
    mut query: Query<(&mut Position, &mut Direction, &AntGoal), With<Ant>>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    food_pheromones: Res<crate::pheromones::PheromoneGrid<crate::pheromones::Food>>,
    nest_pheromones: Res<crate::pheromones::PheromoneGrid<crate::pheromones::Nest>>,
) {
    for (mut position, mut direction, ant_goal) in query.iter_mut() {
        // Define the directions for "front", "front-left", and "front-right" based on the current direction
        let front = position.position + direction.direction;
        let front_left = position.position + rotate_vector(direction.direction, 45.0);
        let front_right = position.position + rotate_vector(direction.direction, -45.0);

        // Get pheromone values at the three positions based on current goal
        let (pheromone_front, pheromone_left, pheromone_right) = if *ant_goal == AntGoal::Food {
            (
                get_pheromone_value(front, &food_pheromones),
                get_pheromone_value(front_left, &food_pheromones),
                get_pheromone_value(front_right, &food_pheromones)
            )
        } else {
            (
                get_pheromone_value(front, &nest_pheromones),
                get_pheromone_value(front_left, &nest_pheromones),
                get_pheromone_value(front_right, &nest_pheromones)
            )
        };

        // Decision-making for movement direction
        let mut rng = rand::thread_rng();
        
        let cutoff = 0.001;
        // Base direction decision on pheromones
        let base_direction = if pheromone_front > cutoff && pheromone_front >= pheromone_left && pheromone_front >= pheromone_right {
            // Follow strongest pheromone trail - front has highest concentration
            front - position.position
        } else if pheromone_left > cutoff && pheromone_left >= pheromone_right {
            // Front-left has highest concentration
            front_left - position.position
        } else if pheromone_right > cutoff {
            // Front-right has highest concentration
            front_right - position.position
        } else {
            // No strong pheromone trail, use random direction with larger deviation
            let current_angle = direction.direction.y.atan2(direction.direction.x);
            let deviation = rng.gen_range(-45_f32.to_radians()..45_f32.to_radians());
            let new_angle = current_angle + deviation;
            Vec2::new(new_angle.cos(), new_angle.sin())
        };
        
        // Add some randomness to the direction even when following pheromones
        let randomness_factor = if pheromone_front > cutoff || pheromone_left > cutoff || pheromone_right > cutoff {
            // Less randomness when following pheromones
            rng.gen_range(0.8..0.95)
        } else {
            // More randomness when exploring
            rng.gen_range(0.6..0.9)
        };
        
        // Apply small random deviation to direction
        let random_angle = rng.gen_range(-15_f32.to_radians()..15_f32.to_radians());
        let random_direction = Vec2::new(
            base_direction.x * random_angle.cos() - base_direction.y * random_angle.sin(),
            base_direction.x * random_angle.sin() + base_direction.y * random_angle.cos()
        );
        
        // Combine base direction with randomness
        direction.direction = (
            base_direction * randomness_factor 
            + random_direction * (1.0 - randomness_factor)
        ).normalize();

        // Move the ant by 1 pixel in the chosen direction
        position.position += direction.direction;
        
        // Wrap around the screen when ants go out of bounds
        // rem_euclid ensures negative values wrap to the positive side
        let window = window_query.get_single().unwrap();
        position.position.x = position.position.x.rem_euclid(window.width());
        position.position.y = position.position.y.rem_euclid(window.height());
    }
}


// System to handle resetting ant lifetimes
pub fn ant_lifetime_reset_system(
    mut commands: Commands,
    mut ant_query: Query<(Entity, &mut Ant, &mut AntGoal)>,
    reset_query: Query<Entity, With<ResetLifetime>>,
) {
    for entity in reset_query.iter() {
        if let Ok((_, mut ant, mut ant_goal)) = ant_query.get_mut(entity) {
            // Reset lifetime
            let mut rng = rand::thread_rng();
            let new_lifetime = rng.gen_range(30.0..60.0);
            ant.lifetime = Timer::new(Duration::from_secs_f32(new_lifetime), TimerMode::Once);
            
            // Only reset goal to Food if the ant reached the nest
            // (we don't want to change the goal if the ant just found food)
            if *ant_goal == AntGoal::Nest {
                // We'll check if the ant is at the nest by its position in a separate query
                *ant_goal = AntGoal::Food;
            }
        }
        // Remove the reset marker
        commands.entity(entity).remove::<ResetLifetime>();
    }
}

fn sync_transform_with_position(
    mut query: Query<(&Position, &mut Transform)>,
) {
    for (position, mut transform) in query.iter_mut() {
        transform.translation = position.position.extend(0.0); // Update position
    }
}

// Utility function to rotate a vector by an angle (in degrees)
fn rotate_vector(vec: Vec2, angle_deg: f32) -> Vec2 {
    let angle_rad = angle_deg.to_radians();
    let cos_angle = angle_rad.cos();
    let sin_angle = angle_rad.sin();

    Vec2::new(
        vec.x * cos_angle - vec.y * sin_angle,
        vec.x * sin_angle + vec.y * cos_angle,
    )
}

// Helper function to get pheromone value at a position
fn get_pheromone_value<T: Send + Sync + 'static>(
    position: Vec2, 
    pheromone_grid: &crate::pheromones::PheromoneGrid<T>
) -> f32 {
    let x = position.x as usize % pheromone_grid.width;
    let y = position.y as usize % pheromone_grid.height;
    pheromone_grid.grid[x][y]
}
