use bevy::prelude::*;
use bevy::window::PrimaryWindow;

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins(
                (
                    crate::ant::AntPlugin,
                    crate::food::FoodPlugin,
                    crate::pheromones::PheromonePlugin,
                )
            )
            .add_systems(Startup, setup_camera);
    }
}

fn setup_camera(
    mut commands: Commands,
    window_query: Query<&Window, With<PrimaryWindow>>,
) {
    let window = window_query.get_single().unwrap();
    let width = window.width();
    let height = window.height();
    commands.spawn((
        Camera2d, // New way to spawn a 2D camera
        Camera {
            order: 0, // Default camera order
            ..default()
        },
        Projection::Orthographic(OrthographicProjection::default_2d()),
        Transform::from_xyz(width / 2., height / 2., 0.0),
    ));
}

