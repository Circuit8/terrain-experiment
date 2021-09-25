use super::{mesh, texture, Config, SimplificationLevel, MAP_CHUNK_SIZE};
use bevy::{
    math::{Vec3, Vec3Swizzles},
    prelude::*,
    render::wireframe::Wireframe,
    tasks::{AsyncComputeTaskPool, Task},
};
use bevy_flycam::FlyCam;
use derive_more::{Deref, DerefMut};
use futures_lite::future;
use noise::{
    utils::{NoiseMap, NoiseMapBuilder, PlaneMapBuilder},
    Fbm, MultiFractal, Seedable,
};
use std::collections::HashMap;

const CHUNK_SIZE: u32 = MAP_CHUNK_SIZE - 1;
const CHUNK_UPDATE_MOVEMENT_THRESHOLD: f32 = CHUNK_SIZE as f32 * 0.1;

pub fn setup(mut commands: Commands, mut events: EventWriter<StartChunkUpdateEvent>) {
    commands.insert_resource(SeenChunks::default());
    commands.insert_resource(LastChunkUpdatePosition::default());
    events.send(StartChunkUpdateEvent);
}

// Ensures the chunks are updated only if the player has moved a set distance since the last update
pub fn trigger_update(
    mut events: EventWriter<StartChunkUpdateEvent>,
    mut last_chunk_update_position: ResMut<LastChunkUpdatePosition>,
    player_query: Query<(&FlyCam, &Transform)>,
) {
    let viewer_position = player_query.iter().nth(0).unwrap().1.translation.xz();
    if viewer_position.distance(last_chunk_update_position.0) > CHUNK_UPDATE_MOVEMENT_THRESHOLD {
        last_chunk_update_position.0 = viewer_position;
        events.send(StartChunkUpdateEvent);
    }
}

// Creates / updates chunk entities with the correct simplification level and coordinates
pub fn initialize_chunks(
    mut commands: Commands,
    config: Res<Config>,
    mut seen_chunks: ResMut<SeenChunks>,
    mut start_chunk_update_events: EventReader<StartChunkUpdateEvent>,
    player_query: Query<(&FlyCam, &Transform)>,
) {
    if start_chunk_update_events.iter().next().is_none() {
        return;
    }

    println!("Initializing chunks");

    let viewer_position = player_query.iter().nth(0).unwrap().1.translation.xz();
    let viewer_chunk_coords = ChunkCoords::from_position(&viewer_position);

    let chunks_in_view_distance = config.max_view_distance / CHUNK_SIZE as f32;
    let chunk_range = (-(chunks_in_view_distance as i32))..chunks_in_view_distance as i32;
    for y_offset in chunk_range.clone() {
        for x_offset in chunk_range.clone() {
            let chunk_coords = ChunkCoords {
                x: viewer_chunk_coords.x + x_offset,
                y: viewer_chunk_coords.y + y_offset,
            };

            let distance_from_viewer = chunk_coords.to_position().distance(viewer_position);

            let simplification_level = if distance_from_viewer
                < config.low_simplification_threshold.max_distance
            {
                config.low_simplification_threshold.level
            } else if distance_from_viewer < config.medium_simplification_threshold.max_distance {
                config.medium_simplification_threshold.level
            } else if distance_from_viewer < config.high_simplification_threshold.max_distance {
                config.high_simplification_threshold.level
            } else {
                SimplificationLevel::max()
            };

            if let Some((existing_simplification_level, entity)) =
                seen_chunks.get_mut(&chunk_coords)
            {
                if *existing_simplification_level != simplification_level {
                    *existing_simplification_level = simplification_level;
                    commands.entity(*entity).insert(Processing).insert(Chunk {
                        coords: chunk_coords,
                        simplification_level,
                    });
                }
            } else {
                let entity = commands
                    .spawn()
                    .insert(Chunk {
                        coords: chunk_coords,
                        simplification_level,
                    })
                    .insert(Processing)
                    .id();
                seen_chunks.insert(chunk_coords, (simplification_level, entity));
            }
        }
    }
}

// Computes the chunk mesh and texture
pub fn process_chunks(
    newly_processing_chunks_query: Query<(Entity, &Chunk), Added<Processing>>,
    config: Res<Config>,
    task_pool: ResMut<AsyncComputeTaskPool>,
    mut commands: Commands,
) {
    for (entity, chunk) in newly_processing_chunks_query.iter() {
        let config = config.clone();
        let simplification_level = chunk.simplification_level.clone();
        let entity = entity.clone();

        let task = task_pool.spawn(async move {
            let noise_map = generate_noise_map(&config);
            let texture = texture::generate(&noise_map);
            let mut terrain_mesh_generator =
                mesh::Generator::new(noise_map, config.height_scale, simplification_level);
            let mesh = terrain_mesh_generator.generate();

            (texture, mesh)
        });

        commands.entity(entity).insert(task);
    }
}

// This system polls the chunk generation tasks and when one is complete updates the entity with a proper mesh and texture
pub fn insert_chunks(
    mut commands: Commands,
    mut chunks_query: Query<(Entity, &Chunk, &mut Task<(Texture, Mesh)>)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut textures: ResMut<Assets<Texture>>,
    config: Res<Config>,
) {
    for (entity, chunk, mut task) in chunks_query.iter_mut() {
        if let Some((texture, mesh)) = future::block_on(future::poll_once(&mut *task)) {
            let position = chunk.coords.to_position();

            commands.entity(entity).insert_bundle(PbrBundle {
                mesh: meshes.add(mesh),
                material: materials.add(StandardMaterial {
                    base_color_texture: Some(textures.add(texture)),
                    // unlit: true,
                    ..Default::default()
                }),
                transform: Transform {
                    translation: Vec3::new(position.x, 0.0, position.y),
                    ..Default::default()
                },
                ..Default::default()
            });

            if config.wireframe {
                commands.entity(entity).insert(Wireframe);
            }

            commands
                .entity(entity)
                .remove::<Processing>()
                .remove::<Task<(Texture, Mesh)>>();
        }
    }
}

// Rebuild the terrain if it changes
pub fn rebuild_on_change(
    mut commands: Commands,
    config: Res<Config>,
    chunk_query: Query<(Entity, &Chunk)>,
    mut seen_chunks: ResMut<SeenChunks>,
    mut events: EventWriter<StartChunkUpdateEvent>,
) {
    if config.is_changed() {
        println!("Config has changed, going to despawn");
        // Destroy all the previous terrain entities
        for (entity, _) in chunk_query.iter() {
            commands.entity(entity).despawn_recursive()
        }

        seen_chunks.clear();
        events.send(StartChunkUpdateEvent);
    }
}

// Computes if chunks should be visible based on the distance between the edge of the chunk and the player
pub fn compute_chunk_visibility(
    config: Res<Config>,
    mut chunks_query: Query<(&mut Visible, &Chunk)>,
    player_query: Query<(&FlyCam, &Transform)>,
    mut start_chunk_update_events: EventReader<StartChunkUpdateEvent>,
) {
    if start_chunk_update_events.iter().next().is_none() {
        return;
    }

    println!("Computing visibility");

    let viewer_position = player_query.iter().nth(0).unwrap().1.translation.xz();

    for (mut visible, chunk) in chunks_query.iter_mut() {
        let distance_from_viewer = chunk.coords.to_position().distance(viewer_position);

        if distance_from_viewer > config.max_view_distance as f32 {
            visible.is_visible = false;
        } else {
            visible.is_visible = true;
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct ChunkCoords {
    x: i32,
    y: i32,
}

impl ChunkCoords {
    pub fn from_position(position: &Vec2) -> ChunkCoords {
        ChunkCoords {
            x: position.x as i32 / CHUNK_SIZE as i32,
            y: position.y as i32 / CHUNK_SIZE as i32,
        }
    }

    pub fn to_position(&self) -> Vec2 {
        Vec2::new(
            (self.x * CHUNK_SIZE as i32) as f32,
            (self.y * CHUNK_SIZE as i32) as f32,
        )
    }
}

#[derive(Debug, Default)]
pub struct Chunk {
    coords: ChunkCoords,
    simplification_level: SimplificationLevel,
}

pub fn generate_noise_map(config: &Config) -> NoiseMap {
    let fbm = Fbm::new()
        .set_seed(config.seed)
        .set_lacunarity(config.lacunarity)
        .set_persistence(config.persistance)
        .set_octaves(config.octaves);
    let builder = PlaneMapBuilder::new(&fbm)
        .set_size(MAP_CHUNK_SIZE as usize, MAP_CHUNK_SIZE as usize)
        .set_x_bounds(-1.0 * config.noise_scale, 1.0 * config.noise_scale)
        .set_y_bounds(-1.0 * config.noise_scale, 1.0 * config.noise_scale);
    builder.build()
}

pub struct Processing;

// Acts as a cache for the chunks or were constantly looping through all chunks
#[derive(Deref, DerefMut, Clone, Debug, Default)]
pub struct SeenChunks(pub HashMap<ChunkCoords, (SimplificationLevel, Entity)>);

// Track how far the player has moved since the last time we updated the chunks, indicating to the systems when they need to run again
#[derive(Deref, DerefMut, Clone, Debug, Default)]
pub struct LastChunkUpdatePosition(pub Vec2);

#[derive(Clone, Copy, Debug, Default)]
pub struct StartChunkUpdateEvent;
