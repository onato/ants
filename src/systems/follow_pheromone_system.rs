use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use rand::Rng;
use crate::components::ant::Ant;
use crate::components::position::Position;
use crate::components::carrying_food::CarryingFood;
use crate::components::direction::Direction;
use crate::pheromones::PheromoneGridTrait;
use crate::utils::geometry::*;

pub fn follow_pheromones_system(
    mut query: Query<(&mut Position, &mut Direction, Option<&CarryingFood>), With<Ant>>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    food_pheromones: Res<crate::pheromones::PheromoneGrid<crate::pheromones::Food>>,
    nest_pheromones: Res<crate::pheromones::PheromoneGrid<crate::pheromones::Nest>>,
) {

    const VIEW_ANGLE: f32 = 45.0; // in degrees
    let mut rng = rand::thread_rng();
    let view_radius: i32 = 6;

    for (mut position, mut direction, carrying_food) in query.iter_mut() {
        let pheromone_grid: &dyn PheromoneGridTrait = if carrying_food.is_some() {
            &*nest_pheromones
        } else {
            &*food_pheromones
        };

        let mut best_direction = direction.direction;
        let mut max_pheromone = 0.0;

        for angle in (-VIEW_ANGLE as i32..=VIEW_ANGLE as i32).step_by(1) {
            let angle_rad = (angle as f32).to_radians();
            let rotated_direction = rotate_vector(direction.direction, angle_rad.to_degrees());

            for dist in 1..=view_radius as i32 {
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
            let random_angle_rad: f32 = rng.gen_range((-VIEW_ANGLE/2.)..=VIEW_ANGLE/2.);
            direction.direction = rotate_vector(direction.direction, random_angle_rad).normalize();
        } else {
            direction.direction = best_direction.normalize();
        }
        // Add some randomness to the direction
        let random_offset: Vec2 = random_normalized_direction() * rng.gen_range(0.0..0.8);
        position.position += (direction.direction + random_offset).normalize();

        let window = window_query.get_single().unwrap();
        position.position.x = position.position.x.rem_euclid(window.width());
        position.position.y = position.position.y.rem_euclid(window.height());
    }
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
