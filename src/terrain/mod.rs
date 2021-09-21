use bevy;
use bevy::{
    ecs::system::{Res, ResMut},
    prelude::*,
    render::mesh::Mesh,
};
use bevy_inspector_egui::Inspectable;
use noise::{
    utils::{NoiseMap, NoiseMapBuilder, PlaneMapBuilder},
    Fbm, MultiFractal, Seedable,
};
mod mesh;
mod texture;

pub const MAP_CHUNK_SIZE: usize = 481;

#[derive(Inspectable)]
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
    level_of_detail: usize,
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
            level_of_detail: 0,
        }
    }
}

pub struct Terrain;

// Rebuild the terrain if it changes
pub fn rebuild_on_change(
    mut commands: Commands,
    config: Res<Config>,
    mut meshes: ResMut<Assets<Mesh>>,
    terrain_query: Query<(Entity, &Terrain)>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut textures: ResMut<Assets<Texture>>,
) {
    if config.is_changed() {
        // Destroy all the previous terrain entities like the water, ground, sun etc (we'll recreate them all)
        for (entity, _) in terrain_query.iter() {
            commands.entity(entity).despawn()
        }

        let noise_map = generate_noise_map(&config);
        let texture = texture::generate(&noise_map);
        let mut terrain_mesh_generator = mesh::Generator::new(noise_map, config.height_scale);
        let mesh = terrain_mesh_generator.generate();

        commands
            .spawn_bundle(PbrBundle {
                mesh: meshes.add(mesh),
                material: materials.add(StandardMaterial {
                    base_color_texture: Some(textures.add(texture)),
                    // unlit: true,
                    ..Default::default()
                }),
                ..Default::default()
            })
            .insert(Terrain);
    }
}

pub fn generate_noise_map(config: &Config) -> NoiseMap {
    let fbm = Fbm::new()
        .set_seed(config.seed)
        .set_lacunarity(config.lacunarity)
        .set_persistence(config.persistance)
        .set_octaves(config.octaves);
    let builder = PlaneMapBuilder::new(&fbm)
        .set_size(MAP_CHUNK_SIZE, MAP_CHUNK_SIZE)
        .set_x_bounds(-1.0 * config.noise_scale, 1.0 * config.noise_scale)
        .set_y_bounds(-1.0 * config.noise_scale, 1.0 * config.noise_scale);
    builder.build()
}
