use bevy::prelude::*;

use crate::ant;
pub struct GamePlugin;

const X_EXTENT: f32 = 512.;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins(
                (
                    ant::AntPlugin,
                )
            )
            .add_systems(Startup, setup_scene);
    }
}
fn setup_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn(Camera2d);

    commands.spawn((
        Mesh2d(meshes.add(Circle::new(50.0))),
        MeshMaterial2d(materials.add(Color::srgb(1.0 as f32, 0.4 as f32, 0.0 as f32))),
        Transform::from_xyz( -X_EXTENT / 2., -X_EXTENT / 2., 0.0,),
    ));
    commands.spawn((
        Mesh2d(meshes.add(Circle::new(50.0))),
        MeshMaterial2d(materials.add(Color::srgb(1.0 as f32, 0.4 as f32, 0.0 as f32))),
        Transform::from_xyz( X_EXTENT / 2., -X_EXTENT / 2., 0.0,),
    ));
    commands.spawn((
        Mesh2d(meshes.add(Circle::new(50.0))),
        MeshMaterial2d(materials.add(Color::srgb(1.0 as f32, 0.4 as f32, 0.0 as f32))),
        Transform::from_xyz( X_EXTENT / 2., X_EXTENT / 2., 0.0,),
    ));
    commands.spawn((
        Mesh2d(meshes.add(Circle::new(50.0))),
        MeshMaterial2d(materials.add(Color::srgb(1.0 as f32, 0.4 as f32, 0.0 as f32))),
        Transform::from_xyz( -X_EXTENT / 2., X_EXTENT / 2., 0.0,),
    ));
}

