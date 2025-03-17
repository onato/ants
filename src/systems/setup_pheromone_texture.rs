use bevy::prelude::*;
use crate::pheromones::PheromoneGrid;
use bevy::window::PrimaryWindow;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};

// Setup pheromone texture
pub fn setup_pheromone_texture<T: Send + Sync + 'static>(
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
    
    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: Color::WHITE, // Fully white background
                custom_size: Some(Vec2::new(width as f32, height as f32)), // Match texture size
                ..default()
            },
            transform: Transform::from_xyz((width as f32) / 2., (height as f32) / 2., -1.0), // Lower Z index
            ..default()
        },
    ));

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

