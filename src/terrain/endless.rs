use std::collections::HashMap;

use bevy::{
    math::{Vec3, Vec3Swizzles},
    prelude::*,
};
use bevy_flycam::FlyCam;

use super::{Config, MAP_CHUNK_SIZE};

const CHUNK_SIZE: u32 = MAP_CHUNK_SIZE - 1;

#[derive(Debug)]
pub struct ChunkDict(HashMap<ChunkCoords, Entity>);

pub fn setup(mut commands: Commands) {
    commands.insert_resource(ChunkDict(HashMap::new()));
}

pub fn update(
    mut commands: Commands,
    config: Res<Config>,
    mut chunk_dict: ResMut<ChunkDict>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut chunks_query: Query<(&Chunk, &mut Visible)>,
    player_query: Query<(&FlyCam, &Transform)>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let viewer_position = player_query.iter().nth(0).unwrap().1.translation;
    let viewer_chunk_coords = ChunkCoords::from_position(&viewer_position);

    // Set the chunks visible based on if they are within range
    for (chunk, mut visible) in chunks_query.iter_mut() {
        let distance_from_viewer = chunk
            .coords
            .to_position()
            .xz()
            .distance_squared(viewer_position.xz())
            .sqrt();

        visible.is_visible = distance_from_viewer <= config.max_view_distance as f32
    }

    // Create new chunks if needed
    let chunks_in_view_distance = config.max_view_distance / CHUNK_SIZE;
    let chunk_range = (-(chunks_in_view_distance as i32))..chunks_in_view_distance as i32;
    for y_offset in chunk_range.clone() {
        for x_offset in chunk_range.clone() {
            let viewed_chunk_coord = ChunkCoords {
                x: viewer_chunk_coords.x + x_offset,
                y: viewer_chunk_coords.y + y_offset,
            };

            if !chunk_dict.0.contains_key(&viewed_chunk_coord) {
                let chunk_entity = commands
                    .spawn_bundle(PbrBundle {
                        mesh: meshes.add(Mesh::from(shape::Plane {
                            size: CHUNK_SIZE as f32,
                        })),
                        material: materials.add(Color::rgb(0.3, 0.5, 0.3).into()),
                        transform: Transform {
                            translation: viewed_chunk_coord.to_position(),
                            ..Default::default()
                        },
                        ..Default::default()
                    })
                    .insert(Chunk {
                        coords: viewed_chunk_coord,
                    })
                    .id();

                // create the chunk here using commands, for now just make a plain
                // get the chunk_entity
                chunk_dict.0.insert(viewed_chunk_coord, chunk_entity);
            }
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
