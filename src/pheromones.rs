use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use crate::components::position::Position;
use crate::ant::Ant;

pub struct PheromonePlugin;

impl Plugin for PheromonePlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<PheromoneGrid>()
            .add_systems(Startup, setup_pheromone_grid)
            .add_systems(Update, update_pheromone_grid);
    }
}

#[derive(Resource, Default)]
pub struct PheromoneGrid {
    pub grid: Vec<Vec<f32>>,
    pub width: usize,
    pub height: usize,
}

fn setup_pheromone_grid(
    mut pheromone_grid: ResMut<PheromoneGrid>,
    window_query: Query<&Window, With<PrimaryWindow>>,
) {
    let window = window_query.get_single().unwrap();
    let width = window.width() as usize;
    let height = window.height() as usize;
    
    // Initialize the grid with zeros
    let grid = vec![vec![0.0; height]; width];
    
    pheromone_grid.grid = grid;
    pheromone_grid.width = width;
    pheromone_grid.height = height;
}

fn update_pheromone_grid(
    mut pheromone_grid: ResMut<PheromoneGrid>,
    ant_query: Query<&Position, With<Ant>>,
) {
    // Increase pheromone level at each ant's position
    for position in ant_query.iter() {
        let x = position.position.x as usize % pheromone_grid.width;
        let y = position.position.y as usize % pheromone_grid.height;
        
        // Increase pheromone level at this position
        pheromone_grid.grid[x][y] += 0.1;
        
        // Optional: Cap the maximum pheromone level
        if pheromone_grid.grid[x][y] > 10.0 {
            pheromone_grid.grid[x][y] = 10.0;
        }
    }
    
    // Optional: Add pheromone decay over time
    for x in 0..pheromone_grid.width {
        for y in 0..pheromone_grid.height {
            pheromone_grid.grid[x][y] *= 0.99; // 1% decay per frame
        }
    }
}
