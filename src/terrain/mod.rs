use bevy;
use bevy::{
    ecs::system::{Res, ResMut},
    prelude::*,
};
use bevy_inspector_egui::Inspectable;

pub mod endless;
pub mod mesh;
pub mod texture;

pub const MAP_CHUNK_SIZE: u32 = 241;

pub fn setup(commands: Commands) {
    endless::setup(commands);
}

#[derive(Inspectable, Clone)]
pub struct Config {
    #[inspectable(min = 0.0001)]
    noise_scale: f64,
    #[inspectable(min = 1)]
    seed: u32,
    #[inspectable(min = 0.0001)]
    lacunarity: f64, // increase for more hills closer together
    #[inspectable(min = 0.0001)]
    persistance: f64,
    #[inspectable(min = 1)]
    octaves: usize,
    #[inspectable(min = 1.0)]
    height_scale: f64,
    #[inspectable(min = 0, max = 6)]
    simplification_level: u32,
    wireframe: bool,
    #[inspectable(min = MAP_CHUNK_SIZE)]
    max_view_distance: u32,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            height_scale: 80.0,
            noise_scale: 1.2,
            seed: 2,
            octaves: 4,
            lacunarity: 2.0,
            persistance: 0.5,
            simplification_level: 0,
            wireframe: false,
            max_view_distance: MAP_CHUNK_SIZE * 2,
        }
    }
}

// Rebuild the terrain if it changes
pub fn rebuild_on_change(
    mut commands: Commands,
    config: Res<Config>,
    chunk_query: Query<(Entity, &endless::Chunk)>,
    mut chunk_coords_seen: ResMut<endless::ChunkCoordsSeen>,
) {
    if config.is_changed() {
        chunk_coords_seen.0.clear();
        // Destroy all the previous terrain entities
        for (entity, _) in chunk_query.iter() {
            commands.entity(entity).despawn()
        }
    }
}
