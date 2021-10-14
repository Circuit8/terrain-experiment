use bevy::math::Vec2;
use nalgebra_glm::smoothstep;
use noise::{NoiseFn, Perlin};

use super::{endless::ChunkCoords, Config, MAP_CHUNK_SIZE};

// values to estimate the maximum possible height of the noise map before normalization (global)
const AMPLITUDE_HEURISTIC: f32 = 0.9;
const HEIGHT_HEURISTIC: f32 = 1.1;

pub struct HeightMap {
    pub data: Vec<Vec<f32>>,
    pub size: usize,
}

impl HeightMap {
    pub fn generate(config: &Config, chunk_coords: &ChunkCoords) -> HeightMap {
        let mut height_map = HeightMap::generate_noise(config, chunk_coords);
        height_map.normalize(config);
        height_map
    }

    fn generate_noise(config: &Config, chunk_coords: &ChunkCoords) -> HeightMap {
        let noise = Perlin::new();

        // sanity check the scale
        let scale = config.scale.max(f32::EPSILON);

        let chunk_offset = chunk_coords.to_position();
        let map = (0..MAP_CHUNK_SIZE)
            .map(|y| {
                (0..MAP_CHUNK_SIZE)
                    .map(|x| {
                        let mut height = 0.0;
                        let mut amplitude = 1.0;
                        let mut frequency = 1.0;

                        for _ in 0..config.octaves {
                            let sample = (Vec2::new(x as f32, y as f32) + chunk_offset)
                                / Vec2::new(MAP_CHUNK_SIZE as f32, MAP_CHUNK_SIZE as f32)
                                / (scale * frequency);
                            let perlin_point = [sample.x as f64, sample.y as f64];
                            height += noise.get(perlin_point) as f32 * amplitude;

                            amplitude *= config.persistence;
                            frequency *= config.lacunarity;
                        }

                        height
                    })
                    .collect()
            })
            .collect();

        HeightMap {
            data: map,
            size: MAP_CHUNK_SIZE as usize,
        }
    }

    fn normalize(&mut self, config: &Config) {
        // determine an approximated maximum possible height difference
        // between the min an max height for global normalization
        let mut max_possible_height = 0.0;
        let mut amplitude = 1.0;

        for _ in 0..config.octaves {
            max_possible_height += amplitude;
            amplitude *= config.persistence * AMPLITUDE_HEURISTIC;
        }

        max_possible_height *= HEIGHT_HEURISTIC;

        // approximated spread around zero
        let spread = max_possible_height / 2.0;

        // normalize the map height between 0 and 1
        self.data.iter_mut().for_each(|row| {
            row.iter_mut().for_each(|height| {
                *height = smoothstep(-spread, spread, *height / max_possible_height);
            })
        });
    }
}
