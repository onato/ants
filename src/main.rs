use bevy::prelude::*;

pub mod ant;
pub mod game;
pub mod components;
pub mod pheromones;

fn main() {
    App::new()
        .add_plugins(
            (
                //list of plugins added to the game
                DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: String::from("Space Invaders"),
                        position: WindowPosition::At(IVec2::ZERO),
                        resolution: Vec2::new(1024., 768.).into(),
                        resizable: false,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .set(ImagePlugin::default_nearest()),

                game::GamePlugin,
            ),
            
        )
        .run();
}
