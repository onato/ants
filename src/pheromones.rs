use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use crate::components::position::Position;
use crate::ant::{Ant, AntGoal};
use std::marker::PhantomData;

// Constants
const PHEROMONE_DECAY_RATE: f32 = 0.9999;
const PHEROMONE_INCREMENT: f32 = 0.05;

// Pheromone types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PheromoneType {
    Nest,
    Food,
}

// Marker types for type-level programming
#[derive(Default)]
pub struct Nest;
#[derive(Default)]
pub struct Food;

pub struct PheromonePlugin;

impl Plugin for PheromonePlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<PheromoneGrid<Nest>>()
            .init_resource::<PheromoneGrid<Food>>()
            .add_systems(Startup, (
                setup_pheromone_grid::<Nest>, 
                setup_pheromone_grid::<Food>,
                setup_pheromone_texture::<Nest>,
                setup_pheromone_texture::<Food>
            ))
            .add_systems(Update, (
                update_pheromone_grid::<Nest>, 
                update_pheromone_grid::<Food>,
                update_pheromone_texture::<Nest>,
                update_pheromone_texture::<Food>
            ));
    }
}

#[derive(Resource, Default)]
pub struct PheromoneGrid<T: Send + Sync + 'static> {
    pub grid: Vec<Vec<f32>>,
    pub width: usize,
    pub height: usize,
    pub texture_handle: Option<Handle<Image>>,
    pub texture_entity: Option<Entity>,
    _marker: PhantomData<T>,
}

// Setup pheromone grid
fn setup_pheromone_grid<T: Send + Sync + 'static>(
    pheromone_grid: ResMut<PheromoneGrid<T>>,
    window_query: Query<&Window, With<PrimaryWindow>>,
) {
    let window = window_query.get_single().unwrap();
    let width = window.width() as usize;
    let height = window.height() as usize;
    
    // Initialize the grid with zeros
    let grid = vec![vec![0.0; height]; width];
    
    let grid_inner = pheromone_grid.into_inner();
    grid_inner.grid = grid;
    grid_inner.width = width;
    grid_inner.height = height;
}

// Setup pheromone texture
fn setup_pheromone_texture<T: Send + Sync + 'static>(
    mut commands: Commands,
    pheromone_grid: ResMut<PheromoneGrid<T>>,
    mut images: ResMut<Assets<Image>>,
    window_query: Query<&Window, With<PrimaryWindow>>,
) {
    let window = window_query.get_single().unwrap();
    let width = window.width() as u32;
    let height = window.height() as u32;
    
    // Create a new image
    let mut texture = Image::new_fill(
        Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        &[0, 0, 0, 255], // Initial color (black with full alpha)
        TextureFormat::Rgba8Unorm,
        bevy::render::render_asset::RenderAssetUsages::RENDER_WORLD,
    );
    
    // Set texture to be filtered in nearest mode and allow it to be updated
    texture.texture_descriptor.usage = bevy::render::render_resource::TextureUsages::TEXTURE_BINDING
        | bevy::render::render_resource::TextureUsages::COPY_DST;
    
    // Add the texture to the assets
    let texture_handle = images.add(texture);
    
    // Store the handle in the resource
    let grid_inner = pheromone_grid.into_inner();
    grid_inner.texture_handle = Some(texture_handle.clone());
    
    // Spawn the sprite entity and store its entity ID
    let entity = commands.spawn((
        Sprite::from_image(texture_handle.clone()),
        Transform {
            translation: Vec3::new((width as f32) / 2., (height as f32) / 2., 0.0),
            scale: Vec3::ONE,
            ..default()
        },
    )).id();

    // Store the entity ID in the resource
    grid_inner.texture_entity = Some(entity);
}

// Component to mark ants that are carrying food
#[derive(Component)]
pub struct CarryingFood;

// Trait to get query filter and color for each pheromone type
trait PheromoneTypeInfo: Send + Sync {
    type QueryFilter: bevy::ecs::query::QueryFilter;
    fn color() -> PheromoneColor;
}

// Implement for Nest type
impl PheromoneTypeInfo for Nest {
    type QueryFilter = (With<Ant>, With<AntGoal>, Without<CarryingFood>);
    
    fn color() -> PheromoneColor {
        PheromoneColor { r: 0, g: 0, b: 255 } // Blue for nest pheromones
    }
}

// Implement for Food type
impl PheromoneTypeInfo for Food {
    type QueryFilter = (With<Ant>, With<CarryingFood>);
    
    fn color() -> PheromoneColor {
        PheromoneColor { r: 0, g: 255, b: 0 } // Green for food pheromones
    }
}

// Generic function to update pheromone grids
fn update_pheromone_grid<T: Send + Sync + 'static + PheromoneTypeInfo>(
    pheromone_grid: ResMut<PheromoneGrid<T>>,
    ant_query: Query<&Position, T::QueryFilter>,
) {
    let grid_inner = pheromone_grid.into_inner();
    
    // Increase pheromone level at each ant's position
    for position in ant_query.iter() {
        // Convert world coordinates to grid coordinates
        let grid_x = (position.position.x as usize)
            .clamp(0, grid_inner.width.saturating_sub(1));
        let grid_y = (position.position.y as usize)
            .clamp(0, grid_inner.height.saturating_sub(1));
        
        // Increase pheromone level at this position
        let current_value = grid_inner.grid[grid_x][grid_y];
        let new_value = (current_value + PHEROMONE_INCREMENT).min(1.0);
        grid_inner.grid[grid_x][grid_y] = new_value;
    }
    
    // Apply pheromone decay over time
    for x in 0..grid_inner.width {
        for y in 0..grid_inner.height {
            grid_inner.grid[x][y] *= PHEROMONE_DECAY_RATE;
        }
    }
}

// Helper struct to define pheromone color channels
struct PheromoneColor {
    r: u8,
    g: u8,
    b: u8,
}

// Generic function to update pheromone textures
fn update_pheromone_texture<T: Send + Sync + 'static + PheromoneTypeInfo>(
    mut commands: Commands,
    pheromone_grid: Res<PheromoneGrid<T>>,
    mut images: ResMut<Assets<Image>>,
    window_query: Query<&Window, With<PrimaryWindow>>,
) {
    // Create a new texture each frame instead of trying to modify the existing one
    let window = window_query.get_single().unwrap();
    let width = window.width() as u32;
    let height = window.height() as u32;
    
    // Get the color for this pheromone type
    let color = T::color();
    
    // Create a new image with the current pheromone data
    let mut data = vec![0u8; (width * height * 4) as usize];
    
    let grid_inner = pheromone_grid.into_inner();
    
    // Fill the texture data based on the grid values
    for y in 0..grid_inner.height.min(height as usize) {
        for x in 0..grid_inner.width.min(width as usize) {
            // Get the pheromone value at this position
            let pheromone_value = grid_inner.grid[x][y];
            
            // Convert to a color intensity based on pheromone level
            let intensity = (pheromone_value * 255.0).min(255.0) as u8;
            
            // Calculate the pixel index in the texture data
            // Flip y-coordinate to match screen coordinates (0,0 at top-left)
            let pixel_index = (((height as usize - 1 - y) as u32 * width + x as u32) * 4) as usize;
            
            // Set the pixel color (RGBA) with the specified color channels
            if pixel_index + 3 < data.len() {
                data[pixel_index] = (color.r as u16 * intensity as u16 / 255) as u8;       // R
                data[pixel_index + 1] = (color.g as u16 * intensity as u16 / 255) as u8;   // G
                data[pixel_index + 2] = (color.b as u16 * intensity as u16 / 255) as u8;   // B
                data[pixel_index + 3] = intensity;                   // A (semi-transparent based on intensity)
            }
        }
    }
    
    // Create a new texture with the updated data
    let mut new_texture = Image::new_fill(
        Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        &data,
        TextureFormat::Rgba8Unorm,
        bevy::render::render_asset::RenderAssetUsages::RENDER_WORLD,
    );
    
    // Set texture properties
    new_texture.texture_descriptor.usage = bevy::render::render_resource::TextureUsages::TEXTURE_BINDING
        | bevy::render::render_resource::TextureUsages::COPY_DST;
    
    // Add the new texture to assets and update the sprite
    if let Some(entity) = grid_inner.texture_entity {
        let new_handle = images.add(new_texture);
        commands.entity(entity).insert(Sprite::from_image(new_handle));
    }
}
