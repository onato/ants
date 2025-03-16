use bevy::prelude::*;
use rand::Rng;
use crate::components::position::Position;
use bevy::window::PrimaryWindow;
use crate::food::Food;
use crate::pheromones::CarryingFood;
use std::time::Duration;

// Marker component to indicate an ant's lifetime should be reset
#[derive(Component)]
pub struct ResetLifetime;

pub struct AntPlugin;

impl Plugin for AntPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Startup, setup)
            .add_systems(Update,(
                ant_goal_system,
                ant_movement_system,
                ant_lifetime_check_system,
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
                    
                    // We need to handle the lifetime reset in the ant_lifetime_system
                    // since we can't access the Ant component from this query
                    commands.entity(entity).insert(ResetLifetime);
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
    let window = window_query.get_single().unwrap();
    let window_width = window.width();
    let window_height = window.height();
    
    for (mut position, mut direction, ant_goal) in query.iter_mut() {
        // Define the directions for "front", "front-left", and "front-right" based on the current direction
        let front = position.position + direction.direction;
        let front_left = position.position + rotate_vector(direction.direction, 45.0);
        let front_right = position.position + rotate_vector(direction.direction, -45.0);

        // Get pheromone values at the three positions based on current goal
        let (pheromone_front, pheromone_left, pheromone_right) = if *ant_goal == AntGoal::Food {
            // When looking for food, follow food pheromones
            let front_x = front.x as usize % food_pheromones.width;
            let front_y = front.y as usize % food_pheromones.height;
            let front_left_x = front_left.x as usize % food_pheromones.width;
            let front_left_y = front_left.y as usize % food_pheromones.height;
            let front_right_x = front_right.x as usize % food_pheromones.width;
            let front_right_y = front_right.y as usize % food_pheromones.height;
            
            (
                food_pheromones.grid[front_x][front_y],
                food_pheromones.grid[front_left_x][front_left_y],
                food_pheromones.grid[front_right_x][front_right_y]
            )
        } else {
            // When returning to nest, follow nest pheromones
            let front_x = front.x as usize % nest_pheromones.width;
            let front_y = front.y as usize % nest_pheromones.height;
            let front_left_x = front_left.x as usize % nest_pheromones.width;
            let front_left_y = front_left.y as usize % nest_pheromones.height;
            let front_right_x = front_right.x as usize % nest_pheromones.width;
            let front_right_y = front_right.y as usize % nest_pheromones.height;
            
            (
                nest_pheromones.grid[front_x][front_y],
                nest_pheromones.grid[front_left_x][front_left_y],
                nest_pheromones.grid[front_right_x][front_right_y]
            )
        };
        
        // Decision-making for movement direction
        let mut rng = rand::thread_rng();
        
        // Base direction decision on pheromones
        let base_direction = if pheromone_front > 0.01 && pheromone_front >= pheromone_left && pheromone_front >= pheromone_right {
            // Follow strongest pheromone trail - front has highest concentration
            front - position.position
        } else if pheromone_left > 0.01 && pheromone_left >= pheromone_right {
            // Front-left has highest concentration
            front_left - position.position
        } else if pheromone_right > 0.01 {
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
        let randomness_factor = if pheromone_front > 0.01 || pheromone_left > 0.01 || pheromone_right > 0.01 {
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
        direction.direction = (base_direction * randomness_factor + random_direction * (1.0 - randomness_factor)).normalize();

        // Normalize direction to ensure the movement is consistent
        direction.direction = direction.direction.normalize();

        // Move the ant by 1 pixel in the chosen direction
        position.position += direction.direction;
        
        // Wrap around the screen when ants go out of bounds
        // rem_euclid ensures negative values wrap to the positive side
        position.position.x = position.position.x.rem_euclid(window_width);
        position.position.y = position.position.y.rem_euclid(window_height);
    }
}

// System to check ant lifetimes and handle expiration
pub fn ant_lifetime_check_system(
    mut commands: Commands,
    time: Res<Time>,
    mut ant_query: Query<(Entity, &mut Ant, &mut AntGoal, &mut Position)>,
) {
    for (entity, mut ant, mut ant_goal, mut position) in ant_query.iter_mut() {
        // Update the lifetime timer
        ant.lifetime.tick(time.delta());
        
        // Check if lifetime has expired
        if ant.lifetime.finished() {
            // Reset position to nest (0,0)
            position.position = Vec2::new(0.0, 0.0);
            
            // Change goal to Nest
            *ant_goal = AntGoal::Nest;
            
            // Remove CarryingFood if present
            commands.entity(entity).remove::<CarryingFood>();
        }
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
