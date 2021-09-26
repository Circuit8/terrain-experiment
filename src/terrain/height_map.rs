use derive_more::{Deref, DerefMut};
use noise::{
    utils::{NoiseMap, NoiseMapBuilder, PlaneMapBuilder},
    Fbm, MultiFractal, Seedable,
};

use super::{endless::ChunkCoords, Config, MAP_CHUNK_SIZE};

#[derive(Deref, DerefMut)]
pub struct HeightMap(pub NoiseMap);

impl HeightMap {
    pub fn generate(config: &Config, chunk_coords: &ChunkCoords) -> HeightMap {
        let fbm = Fbm::new()
            .set_seed(config.seed)
            .set_lacunarity(config.lacunarity)
            .set_persistence(config.persistance)
            .set_octaves(config.octaves);
        let builder = PlaneMapBuilder::new(&fbm)
            .set_size(MAP_CHUNK_SIZE as usize, MAP_CHUNK_SIZE as usize)
            .set_x_bounds(
                chunk_coords.x as f64 * config.noise_scale,
                (chunk_coords.x as f64 + 1.0) * config.noise_scale,
            )
            .set_y_bounds(
                chunk_coords.y as f64 * config.noise_scale,
                (chunk_coords.y as f64 + 1.0) * config.noise_scale,
            );

        HeightMap(builder.build())
    }
}
