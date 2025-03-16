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

// Generic function to setup pheromone grids
fn setup_pheromone_grid<T>(
    mut pheromone_grid: ResMut<T>,
    window_query: Query<&Window, With<PrimaryWindow>>,
) where 
    T: Resource + PheromoneGridSetup,
{
    let window = window_query.get_single().unwrap();
    let width = window.width() as usize;
    let height = window.height() as usize;
    
    // Initialize the grid with zeros
    let grid = vec![vec![0.0; height]; width];
    
    pheromone_grid.set_grid(grid);
    pheromone_grid.set_dimensions(width, height);
}

fn setup_nest_pheromone_grid(
    pheromone_grid: ResMut<NestPheromoneGrid>,
    window_query: Query<&Window, With<PrimaryWindow>>,
) {
    setup_pheromone_grid(pheromone_grid, window_query);
}

fn setup_food_pheromone_grid(
    pheromone_grid: ResMut<FoodPheromoneGrid>,
    window_query: Query<&Window, With<PrimaryWindow>>,
) {
    setup_pheromone_grid(pheromone_grid, window_query);
}

// Generic function to setup pheromone textures
fn setup_pheromone_texture<T>(
    mut commands: Commands,
    mut pheromone_grid: ResMut<T>,
    mut images: ResMut<Assets<Image>>,
    window_query: Query<&Window, With<PrimaryWindow>>,
) where 
    T: Resource + PheromoneGridTexture,
{
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
    pheromone_grid.set_texture_handle(Some(texture_handle.clone()));
    
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
    pheromone_grid.set_texture_entity(Some(entity));
}

fn setup_nest_pheromone_texture(
    commands: Commands,
    pheromone_grid: ResMut<NestPheromoneGrid>,
    images: ResMut<Assets<Image>>,
    window_query: Query<&Window, With<PrimaryWindow>>,
) {
    setup_pheromone_texture(commands, pheromone_grid, images, window_query);
}

fn setup_food_pheromone_texture(
    commands: Commands,
    pheromone_grid: ResMut<FoodPheromoneGrid>,
    images: ResMut<Assets<Image>>,
    window_query: Query<&Window, With<PrimaryWindow>>,
) {
    setup_pheromone_texture(commands, pheromone_grid, images, window_query);
}

// Component to mark ants that are carrying food
#[derive(Component)]
pub struct CarryingFood;

// Generic function to update pheromone grids
fn update_pheromone_grid<T, F>(
    mut pheromone_grid: ResMut<T>,
    ant_query: Query<&Position, F>,
) where 
    T: Resource + PheromoneGridTrait,
    F: bevy::ecs::query::QueryFilter,
{
    // Increase pheromone level at each ant's position
    for position in ant_query.iter() {
        // Convert world coordinates to grid coordinates
        let grid_x = (position.position.x as usize)
            .clamp(0, pheromone_grid.width().saturating_sub(1));
        let grid_y = (position.position.y as usize)
            .clamp(0, pheromone_grid.height().saturating_sub(1));
        
        // Increase pheromone level at this position
        let current_value = pheromone_grid.get_value(grid_x, grid_y);
        let new_value = (current_value + PHEROMONE_INCREMENT).min(1.0);
        pheromone_grid.set_value(grid_x, grid_y, new_value);
    }
    
    // Apply pheromone decay over time
    for x in 0..pheromone_grid.width() {
        for y in 0..pheromone_grid.height() {
            let current_value = pheromone_grid.get_value(x, y);
            pheromone_grid.set_value(x, y, current_value * PHEROMONE_DECAY_RATE);
        }
    }
}

fn update_nest_pheromone_grid(
    pheromone_grid: ResMut<NestPheromoneGrid>,
    // Only ants carrying food will deposit nest pheromones
    ant_query: Query<&Position, (With<Ant>, With<CarryingFood>)>,
) {
    update_pheromone_grid(pheromone_grid, ant_query);
}

fn update_food_pheromone_grid(
    pheromone_grid: ResMut<FoodPheromoneGrid>,
    // Only ants NOT carrying food will deposit food pheromones
    ant_query: Query<&Position, (With<Ant>, Without<CarryingFood>)>,
) {
    update_pheromone_grid(pheromone_grid, ant_query);
}

// Helper struct to define pheromone color channels
struct PheromoneColor {
    r: u8,
    g: u8,
    b: u8,
}

// Generic function to update pheromone textures
fn update_pheromone_texture<T: Resource + PheromoneGridTrait>(
    mut commands: Commands,
    pheromone_grid: Res<T>,
    mut images: ResMut<Assets<Image>>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    color: PheromoneColor,
)
{
    // Create a new texture each frame instead of trying to modify the existing one
    let window = window_query.get_single().unwrap();
    let width = window.width() as u32;
    let height = window.height() as u32;
    
    // Create a new image with the current pheromone data
    let mut data = vec![0u8; (width * height * 4) as usize];
    
    // Fill the texture data based on the grid values
    for y in 0..pheromone_grid.height().min(height as usize) {
        for x in 0..pheromone_grid.width().min(width as usize) {
            // Get the pheromone value at this position
            let pheromone_value = pheromone_grid.get_value(x, y);
            
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
    if let Some(entity) = pheromone_grid.texture_entity() {
        let new_handle = images.add(new_texture);
        commands.entity(entity).insert(Sprite::from_image(new_handle));
    }
}

// Trait for setting up pheromone grids
trait PheromoneGridSetup {
    fn set_grid(&mut self, grid: Vec<Vec<f32>>);
    fn set_dimensions(&mut self, width: usize, height: usize);
}

// Trait for handling pheromone textures
trait PheromoneGridTexture {
    fn set_texture_handle(&mut self, handle: Option<Handle<Image>>);
    fn set_texture_entity(&mut self, entity: Option<Entity>);
}

// Trait to allow generic access to pheromone grid properties
trait PheromoneGridTrait {
    fn width(&self) -> usize;
    fn height(&self) -> usize;
    fn get_value(&self, x: usize, y: usize) -> f32;
    fn set_value(&mut self, x: usize, y: usize, value: f32);
    fn texture_entity(&self) -> Option<Entity>;
}

// Implement the setup trait for NestPheromoneGrid
impl PheromoneGridSetup for NestPheromoneGrid {
    fn set_grid(&mut self, grid: Vec<Vec<f32>>) {
        self.grid = grid;
    }
    
    fn set_dimensions(&mut self, width: usize, height: usize) {
        self.width = width;
        self.height = height;
    }
}

// Implement the texture trait for NestPheromoneGrid
impl PheromoneGridTexture for NestPheromoneGrid {
    fn set_texture_handle(&mut self, handle: Option<Handle<Image>>) {
        self.texture_handle = handle;
    }
    
    fn set_texture_entity(&mut self, entity: Option<Entity>) {
        self.texture_entity = entity;
    }
}

// Implement the trait for NestPheromoneGrid
impl PheromoneGridTrait for NestPheromoneGrid {
    fn width(&self) -> usize {
        self.width
    }
    
    fn height(&self) -> usize {
        self.height
    }
    
    fn get_value(&self, x: usize, y: usize) -> f32 {
        self.grid[x][y]
    }
    
    fn set_value(&mut self, x: usize, y: usize, value: f32) {
        self.grid[x][y] = value;
    }
    
    fn texture_entity(&self) -> Option<Entity> {
        self.texture_entity
    }
}

// Implement the setup trait for FoodPheromoneGrid
impl PheromoneGridSetup for FoodPheromoneGrid {
    fn set_grid(&mut self, grid: Vec<Vec<f32>>) {
        self.grid = grid;
    }
    
    fn set_dimensions(&mut self, width: usize, height: usize) {
        self.width = width;
        self.height = height;
    }
}

// Implement the texture trait for FoodPheromoneGrid
impl PheromoneGridTexture for FoodPheromoneGrid {
    fn set_texture_handle(&mut self, handle: Option<Handle<Image>>) {
        self.texture_handle = handle;
    }
    
    fn set_texture_entity(&mut self, entity: Option<Entity>) {
        self.texture_entity = entity;
    }
}

// Implement the trait for FoodPheromoneGrid
impl PheromoneGridTrait for FoodPheromoneGrid {
    fn width(&self) -> usize {
        self.width
    }
    
    fn height(&self) -> usize {
        self.height
    }
    
    fn get_value(&self, x: usize, y: usize) -> f32 {
        self.grid[x][y]
    }
    
    fn set_value(&mut self, x: usize, y: usize, value: f32) {
        self.grid[x][y] = value;
    }
    
    fn texture_entity(&self) -> Option<Entity> {
        self.texture_entity
    }
}

// Wrapper functions that call the generic function with the appropriate color
fn update_nest_pheromone_texture(
    commands: Commands,
    pheromone_grid: Res<NestPheromoneGrid>,
    images: ResMut<Assets<Image>>,
    window_query: Query<&Window, With<PrimaryWindow>>,
) {
    update_pheromone_texture(
        commands,
        pheromone_grid,
        images,
        window_query,
        PheromoneColor { r: 0, g: 0, b: 255 }, // Blue for nest pheromones
    );
}

fn update_food_pheromone_texture(
    commands: Commands,
    pheromone_grid: Res<FoodPheromoneGrid>,
    images: ResMut<Assets<Image>>,
    window_query: Query<&Window, With<PrimaryWindow>>,
) {
    update_pheromone_texture(
        commands,
        pheromone_grid,
        images,
        window_query,
        PheromoneColor { r: 0, g: 255, b: 0 }, // Green for food pheromones
    );
}
