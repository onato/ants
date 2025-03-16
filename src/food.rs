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

    // Get the primary window dimensions
    let window = window_query.get_single().unwrap();
    let width = window.width();
    let height = window.height();

    let circle_material = materials.add(Color::srgb(0.0, 0.4, 1.0));
    let circle_mesh = meshes.add(Circle::new(5.0));
    
    // Bottom-left corner
    commands.spawn((
        Food,
        Position { position: Vec2::new((width as f32) / 2., (height as f32) / 2.) },
        Mesh2d(circle_mesh.clone()),
        MeshMaterial2d(circle_material.clone()),
        Transform::from_xyz(0., 0., 0.0),
    ));
}
