use bevy::prelude::*;
use crate::components::position::Position;
use bevy::window::PrimaryWindow;

pub struct FoodPlugin;

impl Plugin for FoodPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Startup, setup_food);
    }
}

#[derive(Component)]
pub struct Food;

fn setup_food(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    window_query: Query<&Window, With<PrimaryWindow>>,
) {
    let window = window_query.get_single().unwrap();
    let width = window.width();
    let height = window.height();

    let circle_material = materials.add(Color::srgb(0., 0., 0.));
    let circle_mesh = meshes.add(Circle::new(5.0));
    
    let num_foods = 5;
    let spacing = width / 2. / (num_foods as f32 + 1.0);
    let y_position = height / 2.0;

    for i in 1..=num_foods {
        let x_position = width / 4. + spacing * i as f32;
        commands.spawn((
            Food,
            Position { position: Vec2::new(x_position, y_position) },
            Mesh2d(circle_mesh.clone()),
            MeshMaterial2d(circle_material.clone()),
            Transform::from_xyz(0., 0., 0.0),
        ));
    }
}
