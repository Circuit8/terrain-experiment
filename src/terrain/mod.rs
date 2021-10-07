use bevy::{self, prelude::*};
use bevy_inspector_egui::{Inspectable, InspectorPlugin};
use derive_more::{Add, Deref, From, Into, Mul};

mod endless;
mod height_map;
mod mesh;
mod texture;

const MAP_CHUNK_SIZE: u32 = 241;

#[derive(Inspectable, Clone, Debug)]
pub struct Config {
    #[inspectable(min = 1)]
    seed: u32,
    #[inspectable(min = 0.0001)]
    lacunarity: f32, // increase for more hills closer together
    #[inspectable(min = 0.0001)]
    persistence: f32,
    #[inspectable(min = 1)]
    octaves: usize,
    #[inspectable(min = 1.0)]
    height_scale: f32,
    #[inspectable(min = 0.0001)]
    scale: f32,
    wireframe: bool,
    #[inspectable(min = MAP_CHUNK_SIZE as f32)]
    max_view_distance: f32,
    low_simplification_threshold: SimplificationThreshold,
    medium_simplification_threshold: SimplificationThreshold,
    high_simplification_threshold: SimplificationThreshold,
    #[inspectable(min = 0.0, max = 1.0)]
    material_roughness: f32,
    #[inspectable(min = 0.0, max = 1.0)]
    material_reflectance: f32,
    endless: bool,
    terrain_thresholds: [TerrainThreshold; 6],
}

impl Default for Config {
    fn default() -> Self {
        Config {
            height_scale: 100.0,
            seed: 2,
            octaves: 6,
            lacunarity: 0.6,
            persistence: 0.5,
            scale: 1.0,
            wireframe: false,
            low_simplification_threshold: SimplificationThreshold {
                max_distance: 700.,
                level: SimplificationLevel(1),
            },
            medium_simplification_threshold: SimplificationThreshold {
                max_distance: 1000.,
                level: SimplificationLevel(2),
            },
            high_simplification_threshold: SimplificationThreshold {
                max_distance: 1300.,
                level: SimplificationLevel(4),
            },
            max_view_distance: 1500.,
            material_roughness: 0.98,
            material_reflectance: 0.1,
            endless: false,
            terrain_thresholds: [
                TerrainThreshold {
                    max_height: 0.35,
                    color: Color::rgb(0.0, 0.1, 0.8),
                },
                TerrainThreshold {
                    max_height: 0.4,
                    color: Color::rgb(0.9, 0.78, 0.01),
                },
                TerrainThreshold {
                    max_height: 0.45,
                    color: Color::hex("339D35").unwrap(),
                },
                TerrainThreshold {
                    max_height: 0.7,
                    color: Color::rgb_u8(61, 179, 72),
                },
                TerrainThreshold {
                    max_height: 0.85,
                    color: Color::rgb_u8(72, 56, 56),
                },
                TerrainThreshold {
                    max_height: 2.0,
                    color: Color::rgb(1.0, 1.0, 1.0),
                },
            ],
        }
    }
}

#[derive(Inspectable, Clone, Copy, Debug)]
struct TerrainThreshold {
    #[inspectable(min = 0.0, max = 1.1)]
    max_height: f32,
    color: Color,
}

#[derive(Inspectable, Clone, Copy, Debug)]
struct SimplificationThreshold {
    max_distance: f32,
    level: SimplificationLevel,
}

#[derive(
    Inspectable, PartialEq, From, Add, Mul, Into, Deref, Clone, Copy, Debug, Eq, Hash, Default,
)]
pub struct SimplificationLevel(#[inspectable(min = 1, max = 6)] u32);

impl SimplificationLevel {
    pub fn min() -> Self {
        SimplificationLevel(1)
    }

    pub fn max() -> Self {
        SimplificationLevel(6)
    }
}

pub struct Terrain;

impl Plugin for Terrain {
    fn build(&self, app: &mut AppBuilder) {
        app.add_plugin(InspectorPlugin::<Config>::new())
            .add_event::<endless::StartChunkUpdateEvent>()
            .add_startup_system(endless::setup.system())
            .add_system(
                endless::trigger_update
                    .system()
                    .label("endless::trigger_update"),
            )
            .add_system(
                endless::initialize_chunks
                    .system()
                    .before("endless::compute_chunk_visibility")
                    .after("endless::trigger_update"),
            )
            .add_system(
                endless::process_chunks
                    .system()
                    .before("endless::compute_chunk_visibility"),
            )
            .add_system(
                endless::insert_chunks
                    .system()
                    .before("endless::compute_chunk_visibility"),
            )
            .add_system(
                endless::compute_chunk_visibility
                    .system()
                    .label("endless::compute_chunk_visibility")
                    .after("endless::trigger_update"),
            )
            .add_system(
                endless::rebuild_on_change
                    .system()
                    .after("endless::compute_chunk_visibility"),
            );
    }
}
