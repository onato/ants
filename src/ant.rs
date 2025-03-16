use bevy::prelude::*;
use rand::Rng;
use crate::components::position::Position;
use bevy::window::PrimaryWindow;
use crate::food::Food;
use crate::pheromones::CarryingFood;

pub struct AntPlugin;

impl Plugin for AntPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Startup, setup)
            .add_systems(Update,(ant_movement_system, sync_transform_with_position));
    }
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub enum AntGoal {
    Food,
    Nest,
}

#[derive(Component)]
pub struct Ant {
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
        commands.spawn((
            // Mesh2d(meshes.add(Rectangle::new(1., 1.))),
            // MeshMaterial2d(materials.add(Color::srgb(0.3 as f32, 1.0 as f32, 0.0 as f32))),
            // Transform::from_xyz( 0., 0., 0.,),
            Ant { },
            AntGoal::Food, // Initially set to Food goal
            Position { position: Vec2::new(0.0, 0.0) }, // Initial position
            Direction { direction: Vec2::new(1.0, 0.0) },
        ));
    }
}
pub fn ant_movement_system(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Position, &mut Direction, &mut AntGoal), With<Ant>>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    food_positions: Query<&Transform, With<Food>>,
    food_pheromones: Res<crate::pheromones::PheromoneGrid<crate::pheromones::Food>>,
    nest_pheromones: Res<crate::pheromones::PheromoneGrid<crate::pheromones::Nest>>,
) {
    let window = window_query.get_single().unwrap();
    let window_width = window.width();
    let window_height = window.height();
    
    // Collect food positions into a Vec to avoid query conflicts
    let food_positions: Vec<Vec2> = food_positions
        .iter()
        .map(|transform| Vec2::new(transform.translation.x, transform.translation.y))
        .collect();
        
    for (entity, mut position, mut direction, mut ant_goal) in query.iter_mut() {
        // Define the directions for "front", "front-left", and "front-right" based on the current direction
        let front = position.position + direction.direction;
        let front_left = position.position + rotate_vector(direction.direction, 45.0);
        let front_right = position.position + rotate_vector(direction.direction, -45.0);

        // Check for food in these directions
        let food_in_front = food_positions.iter().any(|&food_pos| food_pos.distance(front) < 5.0);
        let food_in_front_left = food_positions.iter().any(|&food_pos| food_pos.distance(front_left) < 5.0);
        let food_in_front_right = food_positions.iter().any(|&food_pos| food_pos.distance(front_right) < 5.0);
        
        // If food is found in any of the three directions, change the ant's goal to Nest
        if food_in_front || food_in_front_left || food_in_front_right {
            *ant_goal = AntGoal::Nest;
            // Mark the ant as carrying food
            commands.entity(entity).insert(CarryingFood);
        }

        // Check if we've reached the nest when that's our goal
        let home_reached = if *ant_goal == AntGoal::Nest {
            let is_home = position.position.length() < 10.0; // Consider nest reached if close to origin
            if is_home {
                *ant_goal = AntGoal::Food; // Switch goal back to finding food
                commands.entity(entity).remove::<CarryingFood>();
            }
            is_home
        } else {
            false
        };

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
        if food_in_front {
            direction.direction = front - position.position; // Move towards food if visible
        } else if food_in_front_left {
            direction.direction = front_left - position.position;
        } else if food_in_front_right {
            direction.direction = front_right - position.position;
        } else if pheromone_front > 0.1 && pheromone_front >= pheromone_left && pheromone_front >= pheromone_right {
            // Follow strongest pheromone trail - front has highest concentration
            direction.direction = front - position.position;
        } else if pheromone_left > 0.1 && pheromone_left >= pheromone_right {
            // Front-left has highest concentration
            direction.direction = front_left - position.position;
        } else if pheromone_right > 0.1 {
            // Front-right has highest concentration
            direction.direction = front_right - position.position;
        } else {
            let mut rng = rand::thread_rng();
            // Convert current direction to an angle
            let current_angle = direction.direction.y.atan2(direction.direction.x);
            // Pick a random angle within ±45° (in radians) from current_angle
            let deviation = rng.gen_range(-45_f32.to_radians()..45_f32.to_radians());
            let new_angle = current_angle + deviation;

            // Construct the new direction vector
            direction.direction = Vec2::new(new_angle.cos(), new_angle.sin()).normalize();
        }

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
