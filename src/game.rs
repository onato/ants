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
            .add_systems(Startup, (setup_camera, setup_scene))
            .add_systems(Startup, setup_scene);
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

fn setup_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    window_query: Query<&Window, With<PrimaryWindow>>,
) {
    
    // Get the primary window dimensions
    let window = window_query.get_single().unwrap();
    let width = window.width();
    let height = window.height();

    // Create a shared material for all circles
    let circle_material = materials.add(Color::srgb(1.0, 0.4, 0.0));
    let circle_mesh = meshes.add(Circle::new(50.0));
    
    // Bottom-left corner
    commands.spawn((
        Mesh2d(circle_mesh.clone()),
        MeshMaterial2d(circle_material.clone()),
        Transform::from_xyz(0., 0., 0.0),
    ));
    
    // Bottom-right corner
    commands.spawn((
        Mesh2d(circle_mesh.clone()),
        MeshMaterial2d(circle_material.clone()),
        Transform::from_xyz(width, 0., 0.0),
    ));

    // Top-right corner
    commands.spawn((
        Mesh2d(circle_mesh.clone()),
        MeshMaterial2d(circle_material.clone()),
        Transform::from_xyz(width, height, 0.0),
    ));

    // Top-left corner
    commands.spawn((
        Mesh2d(circle_mesh),
        MeshMaterial2d(circle_material),
        Transform::from_xyz(0., height, 0.0),
    ));
}

