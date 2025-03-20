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

#[test]
fn ant_carries_food_when_touching_food() {
    let mut app = App::new();
    app.add_systems(Update, ant_goal_system);
    let position = Vec2::new(100., 100.);
    add_ant_at_position(position, app.world_mut(), false);
    app.world_mut().spawn((Food, Position { position }));

    app.update();

    assert_eq!(food_carrying_ants_count(app.world_mut()), 1);
}

#[test]
fn ant_drops_food_when_touching_nest() {
    let mut app = App::new();
    app.add_systems(Update, ant_goal_system);
    add_ant_at_position(Vec2::new(9., 0.), app.world_mut(), true);

    app.update();

    assert_eq!(food_carrying_ants_count(app.world_mut()), 0);
}

#[cfg(test)]
fn food_carrying_ants_count(world: &mut World) -> usize {
    world.query::<(&Ant, &CarryingFood)>().iter(world).count()
}

#[cfg(test)]
fn add_ant_at_position(position: Vec2, world: &mut World, carrying_food: bool) {
    let mut entity = world.spawn((
        Ant {
            lifetime: Timer::new(std::time::Duration::from_secs_f32(100.), TimerMode::Once),
        },
        Position { position },
    ));

    if carrying_food {
        entity.insert(CarryingFood);
    }
}
