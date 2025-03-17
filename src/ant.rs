use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use rand::Rng;
use std::time::Duration;
use crate::components::position::Position;
use crate::components::carrying_food::CarryingFood;
use crate::components::reset_lifetime::ResetLifetime;
use crate::pheromones::PheromoneGridTrait;
use crate::systems::ant_rebirth_system::ant_rebirth_system;
use crate::food::Food;

pub struct AntPlugin;

const MIN_LIFETIME: f32 = 30.;
const MAX_LIFETIME: f32 = 120.;

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
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    for _ in 0..999 {
        let mut rng = rand::thread_rng();
        // Random lifetime between 30 and 60 seconds
        let lifetime_secs = rng.gen_range(MIN_LIFETIME..=MAX_LIFETIME);
        
        commands.spawn((
            Mesh2d(meshes.add(Rectangle::new(1., 3.))),
            MeshMaterial2d(materials.add(Color::srgb(0.3 as f32, 1.0 as f32, 0.0 as f32))),
            Transform::from_xyz( 0., 0., 0.,),
            Ant { 
                lifetime: Timer::new(Duration::from_secs_f32(lifetime_secs), TimerMode::Once),
            },
            Position { position: Vec2::new(0.0, 0.0) }, // Initial position
            Direction { direction: Vec2::new(1.0, 0.0) },
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
                if let Some(first_element) = food_positions.first() {
                    println!("{} -> {}", position.position, first_element);
                }
                // Change goal to return to nest
                commands.entity(entity).insert(CarryingFood);
                // Reset lifetime when finding food
                // commands.entity(entity).insert(ResetLifetime);
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

// System for ant movement based on pheromone trails
pub fn follow_pheromones_system(
    mut query: Query<(&mut Position, &mut Direction, Option<&CarryingFood>), With<Ant>>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    food_pheromones: Res<crate::pheromones::PheromoneGrid<crate::pheromones::Food>>,
    nest_pheromones: Res<crate::pheromones::PheromoneGrid<crate::pheromones::Nest>>,
) {

    const VIEW_RADIUS: f32 = 10.0;
    const VIEW_ANGLE: f32 = 60.0; // in degrees

    for (mut position, mut direction, carrying_food) in query.iter_mut() {
        let pheromone_grid: &dyn PheromoneGridTrait = if carrying_food.is_some() {
            &*nest_pheromones
        } else {
            &*food_pheromones
        };

        let mut best_direction = direction.direction;
        let mut max_pheromone = 0.0;

        for angle in (-VIEW_ANGLE as i32..=VIEW_ANGLE as i32).step_by(5) {
            let angle_rad = angle as f32 * std::f32::consts::PI / 180.0;
            let rotated_direction = rotate_vector(direction.direction, angle_rad.to_degrees());

            for dist in 1..=VIEW_RADIUS as i32 {
                let check_position = position.position + rotated_direction * dist as f32;
                let pheromone_value = get_pheromone_value(check_position, pheromone_grid);

                if pheromone_value > max_pheromone {
                    max_pheromone = pheromone_value;
                    best_direction = rotated_direction;
                }
            }
        }

        if max_pheromone == 0.0 {
            // If no pheromone is found, move randomly within VIEW_ANGLE
            let mut rng = rand::thread_rng();
            let random_angle: f32 = rng.gen_range(-VIEW_ANGLE..=VIEW_ANGLE);
            direction.direction = rotate_vector(direction.direction, random_angle.to_radians()).normalize();
        } else {
            direction.direction = best_direction.normalize();
        }
        position.position += direction.direction;

        let window = window_query.get_single().unwrap();
        position.position.x = position.position.x.rem_euclid(window.width());
        position.position.y = position.position.y.rem_euclid(window.height());
    }
}

// System to handle resetting ant lifetimes
pub fn ant_lifetime_reset_system(
    mut commands: Commands,
    mut ant_query: Query<(Entity, &mut Ant), With<ResetLifetime>>,
) {
    for (entity, mut ant) in ant_query.iter_mut() {
        let mut rng = rand::thread_rng();
        let new_lifetime = rng.gen_range(MIN_LIFETIME..=MAX_LIFETIME);
        ant.lifetime = Timer::new(Duration::from_secs_f32(new_lifetime), TimerMode::Once);
        println!("Reset the timer");
            
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
fn get_pheromone_value(
    position: Vec2, 
    pheromone_grid: &dyn PheromoneGridTrait
) -> f32 {
    let x = position.x as usize % pheromone_grid.get_width();
    let y = position.y as usize % pheromone_grid.get_height();
    pheromone_grid.get_grid()[x][y]
}
