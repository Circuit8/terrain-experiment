use bevy::{
    math::{Vec3, Vec3Swizzles},
    prelude::*,
    tasks::{AsyncComputeTaskPool, Task},
};
use bevy_flycam::FlyCam;
use futures_lite::future;
use std::collections::HashSet;

use super::{Config, MAP_CHUNK_SIZE};

const CHUNK_SIZE: u32 = MAP_CHUNK_SIZE - 1;

#[derive(Debug)]
pub struct ChunkCoordsSeen(HashSet<ChunkCoords>);

pub fn setup(mut commands: Commands) {
    commands.insert_resource(ChunkCoordsSeen(HashSet::new()));
}

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

pub fn start_generate_chunk_tasks(
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
            let viewed_chunk_coord = ChunkCoords {
                x: viewer_chunk_coords.x + x_offset,
                y: viewer_chunk_coords.y + y_offset,
            };

            if !chunk_coords_seen.0.contains(&viewed_chunk_coord) {
                chunk_coords_seen.0.insert(viewed_chunk_coord);

                let task = task_pool.spawn(async move {
                    // This will return the mesh texture and height map eventually
                    // Just the transform for now to test
                    Transform {
                        translation: viewed_chunk_coord.to_position(),
                        ..Default::default()
                    }
                });

                commands.spawn().insert(task);
            }
        }
    }
}

pub fn handle_generate_chunk_tasks(
    mut commands: Commands,
    mut transform_tasks: Query<(Entity, &mut Task<Transform>)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for (entity, mut task) in transform_tasks.iter_mut() {
        if let Some(transform) = future::block_on(future::poll_once(&mut *task)) {
            let chunk_coords = ChunkCoords::from_position(&transform.translation);

            commands
                .spawn_bundle(PbrBundle {
                    mesh: meshes.add(Mesh::from(shape::Plane {
                        size: CHUNK_SIZE as f32,
                    })),
                    material: materials.add(Color::rgb(1.0, 0.3, 0.3).into()),
                    transform,
                    ..Default::default()
                })
                .insert(Chunk {
                    coords: chunk_coords,
                });

            commands.entity(entity).remove::<Task<Transform>>();
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
