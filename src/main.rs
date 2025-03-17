use bevy::prelude::*;

pub mod ant;
pub mod food;
pub mod game;
pub mod components;
pub mod pheromones;
pub mod systems;
pub mod utils;

fn main() {
    App::new()
        .add_plugins(
            (
                //list of plugins added to the game
                DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: String::from("Ants"),
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
