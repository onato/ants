use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use crate::components::position::Position;
use crate::ant::Ant;

// Constants
const PHEROMONE_DECAY_RATE: f32 = 0.999; // 0.5% decay per frame
const PHEROMONE_INCREMENT: f32 = 0.05; // Amount to increase per ant per frame

// Pheromone types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PheromoneType {
    Nest,
    Food,
}

pub struct PheromonePlugin;

impl Plugin for PheromonePlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<NestPheromoneGrid>()
            .init_resource::<FoodPheromoneGrid>()
            .add_systems(Startup, (
                setup_nest_pheromone_grid, 
                setup_food_pheromone_grid,
                setup_nest_pheromone_texture,
                setup_food_pheromone_texture
            ))
            .add_systems(Update, (
                update_nest_pheromone_grid, 
                update_food_pheromone_grid,
                update_nest_pheromone_texture,
                update_food_pheromone_texture
            ));
    }
}

#[derive(Resource, Default)]
pub struct NestPheromoneGrid {
    pub grid: Vec<Vec<f32>>,
    pub width: usize,
    pub height: usize,
    pub texture_handle: Option<Handle<Image>>,
    pub texture_entity: Option<Entity>,
}

#[derive(Resource, Default)]
pub struct FoodPheromoneGrid {
    pub grid: Vec<Vec<f32>>,
    pub width: usize,
    pub height: usize,
    pub texture_handle: Option<Handle<Image>>,
    pub texture_entity: Option<Entity>,
}

fn setup_nest_pheromone_grid(
    mut pheromone_grid: ResMut<NestPheromoneGrid>,
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

fn setup_food_pheromone_grid(
    mut pheromone_grid: ResMut<FoodPheromoneGrid>,
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

fn setup_nest_pheromone_texture(
    mut commands: Commands,
    mut pheromone_grid: ResMut<NestPheromoneGrid>,
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
    pheromone_grid.texture_handle = Some(texture_handle.clone());
    
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
    pheromone_grid.texture_entity = Some(entity);
}

fn setup_food_pheromone_texture(
    mut commands: Commands,
    mut pheromone_grid: ResMut<FoodPheromoneGrid>,
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
    pheromone_grid.texture_handle = Some(texture_handle.clone());
    
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
    pheromone_grid.texture_entity = Some(entity);
}

// Component to mark ants that are carrying food
#[derive(Component)]
pub struct CarryingFood;

fn update_nest_pheromone_grid(
    mut pheromone_grid: ResMut<NestPheromoneGrid>,
    // Only ants carrying food will deposit nest pheromones
    ant_query: Query<&Position, (With<Ant>, With<CarryingFood>)>,
) {
    // Increase pheromone level at each ant's position
    for position in ant_query.iter() {
        // Convert world coordinates to grid coordinates
        let grid_x = (position.position.x as usize)
            .clamp(0, pheromone_grid.width.saturating_sub(1));
        let grid_y = (position.position.y as usize)
            .clamp(0, pheromone_grid.height.saturating_sub(1));
        
        // Increase pheromone level at this position
        pheromone_grid.grid[grid_x][grid_y] += PHEROMONE_INCREMENT;
        
        // Optional: Cap the maximum pheromone level
        if pheromone_grid.grid[grid_x][grid_y] > 1.0 {
            pheromone_grid.grid[grid_x][grid_y] = 1.0;
        }
    }
    
    // Optional: Add pheromone decay over time
    for x in 0..pheromone_grid.width {
        for y in 0..pheromone_grid.height {
            pheromone_grid.grid[x][y] *= PHEROMONE_DECAY_RATE;
        }
    }
}

fn update_food_pheromone_grid(
    mut pheromone_grid: ResMut<FoodPheromoneGrid>,
    // Only ants NOT carrying food will deposit food pheromones
    ant_query: Query<&Position, (With<Ant>, Without<CarryingFood>)>,
) {
    // Increase pheromone level at each ant's position
    for position in ant_query.iter() {
        // Convert world coordinates to grid coordinates
        let grid_x = (position.position.x as usize)
            .clamp(0, pheromone_grid.width.saturating_sub(1));
        let grid_y = (position.position.y as usize)
            .clamp(0, pheromone_grid.height.saturating_sub(1));
        
        // Increase pheromone level at this position
        pheromone_grid.grid[grid_x][grid_y] += PHEROMONE_INCREMENT;
        
        // Optional: Cap the maximum pheromone level
        if pheromone_grid.grid[grid_x][grid_y] > 1.0 {
            pheromone_grid.grid[grid_x][grid_y] = 1.0;
        }
    }
    
    // Optional: Add pheromone decay over time
    for x in 0..pheromone_grid.width {
        for y in 0..pheromone_grid.height {
            pheromone_grid.grid[x][y] *= PHEROMONE_DECAY_RATE;
        }
    }
}

fn update_nest_pheromone_texture(
    mut commands: Commands,
    pheromone_grid: Res<NestPheromoneGrid>,
    mut images: ResMut<Assets<Image>>,
    window_query: Query<&Window, With<PrimaryWindow>>,
) {
    // Create a new texture each frame instead of trying to modify the existing one
    let window = window_query.get_single().unwrap();
    let width = window.width() as u32;
    let height = window.height() as u32;
    
    // Create a new image with the current pheromone data
    let mut data = vec![0u8; (width * height * 4) as usize];
    
    // Fill the texture data based on the grid values
    for y in 0..pheromone_grid.height.min(height as usize) {
        for x in 0..pheromone_grid.width.min(width as usize) {
            // Get the pheromone value at this position
            let pheromone_value = pheromone_grid.grid[x][y];
            
            // Convert to a color (blue intensity for nest pheromones)
            let intensity = (pheromone_value * 255.0).min(255.0) as u8;
            
            // Calculate the pixel index in the texture data
            // Flip y-coordinate to match screen coordinates (0,0 at top-left)
            let pixel_index = (((height as usize - 1 - y) as u32 * width + x as u32) * 4) as usize;
            
            // Set the pixel color (RGBA) - Blue for nest pheromones
            if pixel_index + 3 < data.len() {
                data[pixel_index] = 0;           // R
                data[pixel_index + 1] = 0;       // G
                data[pixel_index + 2] = intensity; // B
                data[pixel_index + 3] = intensity; // A (semi-transparent based on intensity)
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
    if let Some(entity) = pheromone_grid.texture_entity {
        let new_handle = images.add(new_texture);
        commands.entity(entity).insert(Sprite::from_image(new_handle));
    }
}

fn update_food_pheromone_texture(
    mut commands: Commands,
    pheromone_grid: Res<FoodPheromoneGrid>,
    mut images: ResMut<Assets<Image>>,
    window_query: Query<&Window, With<PrimaryWindow>>,
) {
    // Create a new texture each frame instead of trying to modify the existing one
    let window = window_query.get_single().unwrap();
    let width = window.width() as u32;
    let height = window.height() as u32;
    
    // Create a new image with the current pheromone data
    let mut data = vec![0u8; (width * height * 4) as usize];
    
    // Fill the texture data based on the grid values
    for y in 0..pheromone_grid.height.min(height as usize) {
        for x in 0..pheromone_grid.width.min(width as usize) {
            // Get the pheromone value at this position
            let pheromone_value = pheromone_grid.grid[x][y];
            
            // Convert to a color (green intensity for food pheromones)
            let intensity = (pheromone_value * 255.0).min(255.0) as u8;
            
            // Calculate the pixel index in the texture data
            // Flip y-coordinate to match screen coordinates (0,0 at top-left)
            let pixel_index = (((height as usize - 1 - y) as u32 * width + x as u32) * 4) as usize;
            
            // Set the pixel color (RGBA) - Green for food pheromones
            if pixel_index + 3 < data.len() {
                data[pixel_index] = 0;           // R
                data[pixel_index + 1] = intensity; // G
                data[pixel_index + 2] = 0;       // B
                data[pixel_index + 3] = intensity; // A (semi-transparent based on intensity)
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
    if let Some(entity) = pheromone_grid.texture_entity {
        let new_handle = images.add(new_texture);
        commands.entity(entity).insert(Sprite::from_image(new_handle));
    }
}
