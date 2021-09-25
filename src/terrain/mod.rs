use bevy;
use bevy_inspector_egui::Inspectable;
use derive_more::{Add, Deref, From, Into, Mul};

pub mod endless;
pub mod mesh;
pub mod texture;

pub const MAP_CHUNK_SIZE: u32 = 241;

#[derive(Inspectable, Clone, Debug)]
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
    wireframe: bool,
    #[inspectable(min = MAP_CHUNK_SIZE as f32)]
    max_view_distance: f32,
    low_simplification_threshold: SimplificationThreshold,
    medium_simplification_threshold: SimplificationThreshold,
    high_simplification_threshold: SimplificationThreshold,
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
            wireframe: false,
            low_simplification_threshold: SimplificationThreshold {
                max_distance: MAP_CHUNK_SIZE as f32,
                level: SimplificationLevel(1),
            },
            medium_simplification_threshold: SimplificationThreshold {
                max_distance: MAP_CHUNK_SIZE as f32 * 2.,
                level: SimplificationLevel(2),
            },
            high_simplification_threshold: SimplificationThreshold {
                max_distance: MAP_CHUNK_SIZE as f32 * 3.,
                level: SimplificationLevel(4),
            },
            max_view_distance: MAP_CHUNK_SIZE as f32 * 4.,
        }
    }
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
