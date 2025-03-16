use bevy::prelude::*;
use rand::Rng;
use crate::components::position::Position;
use bevy::window::PrimaryWindow;

pub struct AntPlugin;

impl Plugin for AntPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Startup, setup)
            .add_systems(Update,(ant_movement_system, sync_transform_with_position));
    }
}

#[derive(Component)]
pub struct Ant
{
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
            Position { position: Vec2::new(0.0, 0.0) }, // Initial position
            Direction { direction: Vec2::new(1.0, 0.0) },
        ));
    }
}
pub fn ant_movement_system(
    mut query: Query<(&mut Position, &mut Direction), With<Ant>>,
    window_query: Query<&Window, With<PrimaryWindow>>,
) {
    let window = window_query.get_single().unwrap();
    let window_width = window.width();
    let window_height = window.height();
    for (mut position, mut direction) in query.iter_mut() {
        // Define the directions for "front", "front-left", and "front-right" based on the current direction
        // let front = position.position + direction.direction;
        // let front_left = position.position + rotate_vector(direction.direction, 45.0);
        // let front_right = position.position + rotate_vector(direction.direction, -45.0);

        // // Check for pheromones in these directions
        // let pheromone_in_front = pheromone_query.iter().any(|(pheromone_position, _)| pheromone_position.position == front);
        // let pheromone_in_front_left = pheromone_query.iter().any(|(pheromone_position, _)| pheromone_position.position == front_left);
        // let pheromone_in_front_right = pheromone_query.iter().any(|(pheromone_position, _)| pheromone_position.position == front_right);
        //
        // // Decision-making: Choose movement direction based on pheromones
        // if pheromone_in_front {
        //     direction.direction = front - position.position; // Move towards front
        // } else if pheromone_in_front_left {
        //     direction.direction = front_left - position.position; // Move towards front-left
        // } else if pheromone_in_front_right {
        //     direction.direction = front_right - position.position; // Move towards front-right
        // } else {
            let mut rng = rand::thread_rng();
            // Convert current direction to an angle
            let current_angle = direction.direction.y.atan2(direction.direction.x);
            // Pick a random angle within ±45° (in radians) from current_angle
            let deviation = rng.gen_range(-45_f32.to_radians()..45_f32.to_radians());
            let new_angle = current_angle + deviation;

            // Construct the new direction vector
            direction.direction = Vec2::new(new_angle.cos(), new_angle.sin()).normalize();
        // }

        // Normalize direction to ensure the movement is consistent
        direction.direction = direction.direction.normalize();

        // Move the ant by 1 pixel in the chosen direction
        position.position += direction.direction;
        
        // Wrap around the screen when ants go out of bounds
        // rem_euclid ensures negative values wrap to the positive side
        position.position.x = position.position.x.rem_euclid(window_width);
        position.position.y = position.position.y.rem_euclid(window_height);

        // // Look for Food or Nest at the new position
        // let found_food_or_nest = pheromone_query.iter().any(|(pheromone_position, pheromone)| {
        //     pheromone_position.position == position.position && matches!(pheromone.pheromone_type, PheromoneType::Food | PheromoneType::Nest)
        // });

        // if found_food_or_nest {
        //     // Change the Ant's goal based on the pheromone type (Food or Nest)
        //     if pheromone_query.iter().any(|(pheromone_position, pheromone)| {
        //         pheromone_position.position == position.position && pheromone.pheromone_type == PheromoneType::Food
        //     }) {
        //         *ant_goal = AntGoal::Food; // Change goal to Food
        //     } else {
        //         *ant_goal = AntGoal::Nest; // Change goal to Nest
        //     }
        // }
    }
}

fn sync_transform_with_position(
    mut query: Query<(&Position, &mut Transform)>,
) {
    for (position, mut transform) in query.iter_mut() {
        transform.translation = position.position.extend(0.0); // Update position
    }
}

// // Utility function to rotate a vector by an angle (in degrees)
// fn rotate_vector(vec: Vec2, angle_deg: f32) -> Vec2 {
//     let angle_rad = angle_deg.to_radians();
//     let cos_angle = angle_rad.cos();
//     let sin_angle = angle_rad.sin();
//
//     Vec2::new(
//         vec.x * cos_angle - vec.y * sin_angle,
//         vec.x * sin_angle + vec.y * cos_angle,
//     )
// }
