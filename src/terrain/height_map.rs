use bevy::math::{DVec2, Vec2};
use derive_more::{Deref, DerefMut};
use noise::{
    utils::{NoiseMap, NoiseMapBuilder, PlaneMapBuilder},
    Fbm, MultiFractal, NoiseFn, OpenSimplex, Perlin, Seedable,
};

use super::{endless::ChunkCoords, Config, MAP_CHUNK_SIZE};

#[derive(Deref, DerefMut)]
// pub struct HeightMap(pub NoiseMap);
pub struct HeightMap(Vec<Vec<f32>>);

impl HeightMap {
    pub fn generate(config: &Config, chunk_coords: &ChunkCoords) -> HeightMap {
        let chunk_offset = chunk_coords.to_position();
        let noise = Perlin::new();

        let height_map = HeightMap(
            (0..MAP_CHUNK_SIZE)
                .map(|y| {
                    (0..MAP_CHUNK_SIZE)
                        .map(|x| {
                            let sample = (Vec2::new(x as f32, y as f32) + chunk_offset)
                                / Vec2::new(MAP_CHUNK_SIZE as f32, MAP_CHUNK_SIZE as f32);
                            // println!("sample: {:?}", sample);
                            let val = noise.get([sample.x as f64, sample.y as f64]) as f32;
                            // println!("{:?}", val);
                            val
                        })
                        .collect()
                })
                .collect(),
        );

        height_map
        // let fbm = Fbm::new()
        //     .set_seed(config.seed)
        //     .set_lacunarity(config.lacunarity)
        //     .set_persistence(config.persistance)
        //     .set_octaves(config.octaves);
        // let builder = PlaneMapBuilder::new(&fbm);
        // .set_size(MAP_CHUNK_SIZE as usize, MAP_CHUNK_SIZE as usize)
        // .set_x_bounds(chunk_coords.x as f64, chunk_coords.x as f64 + 1.0)
        // .set_y_bounds(chunk_coords.y as f64, chunk_coords.y as f64 + 1.0);

        // HeightMap(builder.build())
    }
}
