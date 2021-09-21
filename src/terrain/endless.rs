use bevy::{
    math::{Vec3, Vec3Swizzles},
    prelude::*,
    render::wireframe::Wireframe,
    tasks::{AsyncComputeTaskPool, Task},
};
use bevy_flycam::FlyCam;
use futures_lite::future;
use noise::{
    utils::{NoiseMap, NoiseMapBuilder, PlaneMapBuilder},
    Fbm, MultiFractal, Seedable,
};
use std::collections::HashSet;

use super::{mesh, texture, Config, MAP_CHUNK_SIZE};

const CHUNK_SIZE: u32 = MAP_CHUNK_SIZE - 1;

#[derive(Debug)]
pub struct ChunkCoordsSeen(pub HashSet<ChunkCoords>);

pub fn setup(mut commands: Commands) {
    commands.insert_resource(ChunkCoordsSeen(HashSet::new()));
}

// Computes if chunks should be visible based on the distance between the edge of the chunk and the player
pub fn compute_chunk_visibility(
    config: Res<Config>,
    mut chunks_query: Query<(&Chunk, &mut Visible)>,
    player_query: Query<(&FlyCam, &Transform)>,
) {
    let viewer_position = player_query.iter().nth(0).unwrap().1.translation;

    // Set the chunks visible based on if they are within range
    for (chunk, mut visible) in chunks_query.iter_mut() {
        let distance_from_viewer = chunk
            .coords
            .to_position()
            .xz()
            .distance_squared(viewer_position.xz())
            .sqrt();

        visible.is_visible = distance_from_viewer <= config.max_view_distance as f32;
    }
}

// Starts async chunk generation tasks for chunks within range that dont yet exist
pub fn generate_chunks(
    mut commands: Commands,
    player_query: Query<(&FlyCam, &Transform)>,
    config: Res<Config>,
    mut chunk_coords_seen: ResMut<ChunkCoordsSeen>,
    task_pool: ResMut<AsyncComputeTaskPool>,
) {
    let viewer_position = player_query.iter().nth(0).unwrap().1.translation;
    let viewer_chunk_coords = ChunkCoords::from_position(&viewer_position);

    let chunks_in_view_distance = config.max_view_distance / CHUNK_SIZE;
    let chunk_range = (-(chunks_in_view_distance as i32))..chunks_in_view_distance as i32;
    for y_offset in chunk_range.clone() {
        for x_offset in chunk_range.clone() {
            let viewed_chunk_coords = ChunkCoords {
                x: viewer_chunk_coords.x + x_offset,
                y: viewer_chunk_coords.y + y_offset,
            };

            if !chunk_coords_seen.0.contains(&viewed_chunk_coords) {
                chunk_coords_seen.0.insert(viewed_chunk_coords);

                let config = config.clone();

                let task = task_pool
                    .spawn(async move { TerrainChunkData::generate(config, viewed_chunk_coords) });

                commands.spawn().insert(task);
            }
        }
    }
}

// This system polls the chunk generation tasks and when one is complete it adds it to the world
pub fn insert_chunks(
    mut commands: Commands,
    mut generate_chunk_data_tasks: Query<(Entity, &mut Task<TerrainChunkData>)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut textures: ResMut<Assets<Texture>>,
    config: Res<Config>,
) {
    for (entity, mut task) in generate_chunk_data_tasks.iter_mut() {
        if let Some(terrain_chunk_data) = future::block_on(future::poll_once(&mut *task)) {
            let mut builder = commands.spawn_bundle(PbrBundle {
                mesh: meshes.add(terrain_chunk_data.mesh),
                material: materials.add(StandardMaterial {
                    base_color_texture: Some(textures.add(terrain_chunk_data.texture)),
                    // unlit: true,
                    ..Default::default()
                }),
                transform: terrain_chunk_data.transform,
                ..Default::default()
            });

            builder.insert(Chunk {
                coords: terrain_chunk_data.coords,
            });

            if config.wireframe {
                builder.insert(Wireframe);
            }

            commands.entity(entity).remove::<Task<TerrainChunkData>>();
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ChunkCoords {
    x: i32,
    y: i32,
}

impl ChunkCoords {
    pub fn from_position(position: &Vec3) -> ChunkCoords {
        ChunkCoords {
            x: position.x as i32 / CHUNK_SIZE as i32,
            y: position.z as i32 / CHUNK_SIZE as i32,
        }
    }

    pub fn to_position(&self) -> Vec3 {
        Vec3::new(
            (self.x * CHUNK_SIZE as i32) as f32,
            0.0,
            (self.y * CHUNK_SIZE as i32) as f32,
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Chunk {
    coords: ChunkCoords,
}

#[derive(Debug)]
pub struct TerrainChunkData {
    texture: Texture,
    transform: Transform,
    mesh: Mesh,
    coords: ChunkCoords,
}

impl TerrainChunkData {
    pub fn generate(config: Config, coords: ChunkCoords) -> TerrainChunkData {
        let noise_map = generate_noise_map(&config);
        let texture = texture::generate(&noise_map);
        let mut terrain_mesh_generator =
            mesh::Generator::new(noise_map, config.height_scale, config.simplification_level);
        let mesh = terrain_mesh_generator.generate();

        let transform = Transform {
            translation: coords.to_position(),
            ..Default::default()
        };

        TerrainChunkData {
            transform,
            texture,
            mesh,
            coords,
        }
    }
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
