use crate::components::ant::Ant;
use crate::components::carrying_food::CarryingFood;
use crate::components::food::Food;
use crate::components::position::Position;
use bevy::prelude::*;

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
            let found_food = food_positions
                .iter()
                .any(|&food_pos| food_pos.distance(position.position) < 5.0);

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
