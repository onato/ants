use bevy::prelude::*;
use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin};

pub mod ant;
pub mod food;
pub mod game;
pub mod components;
pub mod pheromones;
pub mod systems;
pub mod utils;

fn main() {
    App::new()
        .add_plugins(FrameTimeDiagnosticsPlugin)
        .add_plugins(
            (
                //list of plugins added to the game
                DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: String::from("Ants"),
                        position: WindowPosition::At(IVec2::ZERO),
                        resolution: Vec2::new(1728., 1050.).into(),
                        resizable: false,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .set(ImagePlugin::default_nearest()),

                game::GamePlugin,
            ),
            
        )
        .add_systems(Update, print_fps)
        .run();
}

fn print_fps(diagnostics: Res<DiagnosticsStore>) {
    if let Some(fps) = diagnostics.get(&FrameTimeDiagnosticsPlugin::FPS) {
        if let Some(average) = fps.smoothed() {
            println!("FPS: {:.1}", average);
        }
    }
}

